use anyhow::{Context, Result};
use miette::{LabeledSpan, NamedSource, Report, miette};
use ra_ap_rustc_lexer::TokenKind;
use regex::Regex;
use serde::Deserialize;
use std::collections::BTreeSet;
use std::path::{Path, PathBuf};
use std::sync::LazyLock;

#[cfg(test)]
use crate::commands::rust_tokens::parse_rust_source;
use crate::commands::rust_tokens::{TokenSpan, parse_rust_file_cached};

const TASKLENS_UI_SRC_ROOT: &str = "crates/tasklens-ui/src";
const SUPPRESSION_CONFIG_PATH: &str = "xtask/config/dead-components.toml";

static COMPONENT_ATTR_RE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"^\s*#\s*\[\s*component\b").expect("valid component regex"));
static FN_DECL_RE: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"^\s*(?:pub(?:\s*\([^)]*\))?\s+)?fn\s+([A-Za-z_][A-Za-z0-9_]*)\b")
        .expect("valid function declaration regex")
});

pub(crate) fn check_dead_components() -> Result<()> {
    let files = load_source_files(Path::new(TASKLENS_UI_SRC_ROOT))?;
    let suppressions = load_suppressions(Path::new(SUPPRESSION_CONFIG_PATH))?;
    let scan_result = scan_for_dead_components(&files, &suppressions);
    let stale_suppressions =
        find_unused_suppressions(&suppressions, &scan_result.used_suppressions);

    if scan_result.violations.is_empty() && stale_suppressions.is_empty() {
        return Ok(());
    }

    for violation in &scan_result.violations {
        eprintln!("{:?}", violation.to_report());
    }
    for stale in &stale_suppressions {
        eprintln!(
            "{:?}",
            stale.to_report(Path::new(SUPPRESSION_CONFIG_PATH).display().to_string())
        );
    }

    anyhow::bail!(
        "Found {} potentially dead component(s) and {} stale suppression(s) in tasklens-ui.",
        scan_result.violations.len(),
        stale_suppressions.len()
    );
}

#[derive(Debug, Clone)]
struct SourceFile {
    path: PathBuf,
    source: String,
    code_lines: Vec<String>,
    line_starts: Vec<usize>,
    tokens: Vec<TokenSpan>,
}

fn load_source_files(src_root: &Path) -> Result<Vec<SourceFile>> {
    let pattern = format!("{}/**/*.rs", src_root.display());
    let mut files = Vec::new();

    for entry in glob::glob(&pattern).context("Invalid glob pattern")? {
        let path = entry.with_context(|| format!("Failed to read glob entry for {pattern}"))?;
        let parsed = parse_rust_file_cached(&path)
            .with_context(|| format!("Failed to parse {}", path.display()))?;
        files.push(SourceFile::from_parsed(
            path,
            &parsed.content,
            parsed.tokens.clone(),
        ));
    }

    files.sort_by(|left, right| left.path.cmp(&right.path));
    Ok(files)
}

impl SourceFile {
    fn from_parsed(path: PathBuf, source: &str, tokens: Vec<TokenSpan>) -> Self {
        let raw_lines: Vec<&str> = source.split('\n').collect();
        let line_starts = line_start_offsets(&raw_lines);
        let code_lines = strip_comments_for_search(&raw_lines);
        Self {
            path,
            source: source.to_string(),
            code_lines,
            line_starts,
            tokens,
        }
    }

    #[cfg(test)]
    fn new(path: PathBuf, source: String) -> Self {
        let raw_lines: Vec<&str> = source.split('\n').collect();
        let line_starts = line_start_offsets(&raw_lines);
        let code_lines = strip_comments_for_search(&raw_lines);
        let tokens = parse_rust_source(&source).tokens;
        Self {
            path,
            source,
            code_lines,
            line_starts,
            tokens,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct ComponentDecl {
    name: String,
    path: PathBuf,
    line: usize,
    column: usize,
    span_offset: usize,
}

#[derive(Debug, Clone)]
struct DeadComponentViolation {
    component: ComponentDecl,
    source: String,
}

impl DeadComponentViolation {
    fn to_report(&self) -> Report {
        let display_path = self.component.path.display().to_string();
        let message = format!(
            "potentially dead Dioxus component `{}` at {}:{}:{}",
            self.component.name, display_path, self.component.line, self.component.column
        );
        let suppression_hint = format!(
            "To suppress intentionally dead components, add this pair to {SUPPRESSION_CONFIG_PATH}:\n[[allow]]\npath = \"{}\"\nname = \"{}\"",
            normalize_repo_relative_path(&display_path),
            self.component.name
        );

        miette!(
            labels = vec![LabeledSpan::at(
                self.component.span_offset..self.component.span_offset + self.component.name.len(),
                "component name declaration"
            )],
            help = suppression_hint,
            "{message}"
        )
        .with_source_code(NamedSource::new(display_path, self.source.clone()))
    }
}

fn scan_for_dead_components(
    files: &[SourceFile],
    suppressions: &BTreeSet<SuppressionKey>,
) -> DeadComponentScanResult {
    let declarations: Vec<ComponentDecl> =
        files.iter().flat_map(find_component_declarations).collect();

    let mut dead = Vec::new();
    let mut used_suppressions = BTreeSet::new();
    for declaration in declarations {
        if !has_any_reference(&declaration, files) {
            if let Some(key) = suppression_key_for_component(&declaration)
                && suppressions.contains(&key)
            {
                used_suppressions.insert(key);
                continue;
            }

            let Some(file) = files.iter().find(|file| file.path == declaration.path) else {
                continue;
            };
            dead.push(DeadComponentViolation {
                component: declaration,
                source: file.source.clone(),
            });
        }
    }

    DeadComponentScanResult {
        violations: dead,
        used_suppressions,
    }
}

#[derive(Debug, Clone)]
struct DeadComponentScanResult {
    violations: Vec<DeadComponentViolation>,
    used_suppressions: BTreeSet<SuppressionKey>,
}

fn suppression_key_for_component(component: &ComponentDecl) -> Option<SuppressionKey> {
    if component.name.trim().is_empty() {
        return None;
    }
    Some(SuppressionKey {
        path: normalize_repo_relative_path(&component.path.display().to_string()),
        name: component.name.clone(),
    })
}

fn find_component_declarations(file: &SourceFile) -> Vec<ComponentDecl> {
    let mut declarations = Vec::new();
    let mut pending_component_attr = false;

    for (index, code_line) in file.code_lines.iter().enumerate() {
        let trimmed = code_line.trim();

        if COMPONENT_ATTR_RE.is_match(trimmed) {
            pending_component_attr = true;
            continue;
        }

        if !pending_component_attr {
            continue;
        }

        if trimmed.is_empty()
            || trimmed.starts_with("#[")
            || trimmed.starts_with("///")
            || trimmed.starts_with("//!")
        {
            continue;
        }

        if let Some(captures) = FN_DECL_RE.captures(code_line) {
            let Some(name_match) = captures.get(1) else {
                pending_component_attr = false;
                continue;
            };
            let line = index + 1;
            let column = name_match.start() + 1;
            let line_start = file.line_starts.get(index).copied().unwrap_or_default();
            declarations.push(ComponentDecl {
                name: name_match.as_str().to_string(),
                path: file.path.clone(),
                line,
                column,
                span_offset: line_start + name_match.start(),
            });
        }

        pending_component_attr = false;
    }

    declarations
}

fn has_any_reference(component: &ComponentDecl, files: &[SourceFile]) -> bool {
    for file in files {
        for (index, token) in file.tokens.iter().enumerate() {
            if !is_ident_token(&token.kind) {
                continue;
            }
            if token_text(file, token) != component.name {
                continue;
            }

            let is_declaration_site =
                file.path == component.path && token.start == component.span_offset;
            if is_declaration_site {
                continue;
            }

            if is_component_use_token(file, index) {
                return true;
            }
        }
    }

    false
}

fn is_component_use_token(file: &SourceFile, token_index: usize) -> bool {
    if is_rsx_component_invocation(file, token_index) {
        return true;
    }

    if is_layout_attribute_reference(file, token_index) {
        return true;
    }

    is_component_fn_pointer_argument(file, token_index)
}

fn is_rsx_component_invocation(file: &SourceFile, token_index: usize) -> bool {
    let Some((next_index, next_token)) = next_non_trivia_token(file, token_index) else {
        return false;
    };

    // RSX-like component invocation: `ComponentName { ... }`
    if matches!(next_token.kind, TokenKind::OpenBrace) {
        return true;
    }

    // RSX with explicit turbofish: `ComponentName::<T> { ... }`
    if !matches!(next_token.kind, TokenKind::Colon) {
        return false;
    }
    let Some((second_colon_index, second_colon)) = next_non_trivia_token(file, next_index) else {
        return false;
    };
    if !matches!(second_colon.kind, TokenKind::Colon) {
        return false;
    }
    let Some((lt_index, lt_token)) = next_non_trivia_token(file, second_colon_index) else {
        return false;
    };
    if !matches!(lt_token.kind, TokenKind::Lt) {
        return false;
    }

    let Some(after_generics_index) = scan_past_generic_args(file, lt_index) else {
        return false;
    };
    matches!(
        next_non_trivia_token(file, after_generics_index).map(|(_, token)| token.kind),
        Some(TokenKind::OpenBrace)
    )
}

fn is_layout_attribute_reference(file: &SourceFile, token_index: usize) -> bool {
    // Router layout attribute: `#[layout(ComponentName)]`
    if let Some((prev_index, prev_token)) = prev_non_trivia_token(file, token_index)
        && matches!(prev_token.kind, TokenKind::OpenParen)
        && let Some((layout_index, layout_token)) = prev_non_trivia_token(file, prev_index)
        && is_ident_token(&layout_token.kind)
        && token_text(file, layout_token) == "layout"
        && let Some((before_layout_index, before_layout_token)) =
            prev_non_trivia_token(file, layout_index)
        && matches!(before_layout_token.kind, TokenKind::OpenBracket)
        && let Some((_before_bracket_index, before_bracket_token)) =
            prev_non_trivia_token(file, before_layout_index)
        && matches!(before_bracket_token.kind, TokenKind::Pound)
    {
        return true;
    }
    false
}

fn is_component_fn_pointer_argument(file: &SourceFile, token_index: usize) -> bool {
    // Function pointer-style usage in tests/helpers:
    // `VirtualDom::new_with_props(ComponentName, props)`
    let Some((_, prev_token)) = prev_non_trivia_token(file, token_index) else {
        return false;
    };
    let Some((_, next_token)) = next_non_trivia_token(file, token_index) else {
        return false;
    };

    if !(matches!(prev_token.kind, TokenKind::OpenParen | TokenKind::Comma)
        && matches!(next_token.kind, TokenKind::Comma | TokenKind::CloseParen))
    {
        return false;
    }

    let Some((open_paren_index, _open_paren)) = nearest_enclosing_open_paren(file, token_index)
    else {
        return false;
    };
    let Some((_, token_before_open_paren)) = prev_non_trivia_token(file, open_paren_index) else {
        return false;
    };

    matches!(
        token_before_open_paren.kind,
        TokenKind::Ident
            | TokenKind::RawIdent
            | TokenKind::CloseParen
            | TokenKind::CloseBracket
            | TokenKind::CloseBrace
    )
}

fn scan_past_generic_args(file: &SourceFile, lt_index: usize) -> Option<usize> {
    let mut depth = 0usize;
    for index in lt_index..file.tokens.len() {
        let token = &file.tokens[index];
        if is_trivia_token(&token.kind) {
            continue;
        }
        match token.kind {
            TokenKind::Lt => {
                depth += 1;
            }
            TokenKind::Gt => {
                depth = depth.saturating_sub(1);
                if depth == 0 {
                    return Some(index);
                }
            }
            _ => {}
        }
    }
    None
}

fn nearest_enclosing_open_paren(
    file: &SourceFile,
    token_index: usize,
) -> Option<(usize, &TokenSpan)> {
    let mut depth = 0usize;
    for index in (0..token_index).rev() {
        let token = &file.tokens[index];
        if is_trivia_token(&token.kind) {
            continue;
        }
        match token.kind {
            TokenKind::CloseParen => depth += 1,
            TokenKind::OpenParen => {
                if depth == 0 {
                    return Some((index, token));
                }
                depth -= 1;
            }
            _ => {}
        }
    }
    None
}

fn is_ident_token(kind: &TokenKind) -> bool {
    matches!(kind, TokenKind::Ident | TokenKind::RawIdent)
}

fn is_trivia_token(kind: &TokenKind) -> bool {
    matches!(
        kind,
        TokenKind::Whitespace | TokenKind::LineComment { .. } | TokenKind::BlockComment { .. }
    )
}

fn token_text<'a>(file: &'a SourceFile, token: &TokenSpan) -> &'a str {
    &file.source[token.start..token.end]
}

fn next_non_trivia_token(file: &SourceFile, token_index: usize) -> Option<(usize, &TokenSpan)> {
    file.tokens
        .iter()
        .enumerate()
        .skip(token_index + 1)
        .find(|(_index, token)| !is_trivia_token(&token.kind))
}

fn prev_non_trivia_token(file: &SourceFile, token_index: usize) -> Option<(usize, &TokenSpan)> {
    for index in (0..token_index).rev() {
        let token = &file.tokens[index];
        if !is_trivia_token(&token.kind) {
            return Some((index, token));
        }
    }
    None
}

fn line_start_offsets(lines: &[&str]) -> Vec<usize> {
    let mut offsets = Vec::with_capacity(lines.len());
    let mut current = 0usize;
    for line in lines {
        offsets.push(current);
        current = current.saturating_add(line.len() + 1);
    }
    offsets
}

fn strip_comments_for_search(lines: &[&str]) -> Vec<String> {
    let mut in_block_comment = false;
    let mut output = Vec::with_capacity(lines.len());

    for line in lines {
        let line_without_blocks = strip_block_comments(line, &mut in_block_comment);
        output.push(strip_line_comment(&line_without_blocks));
    }

    output
}

fn strip_line_comment(line: &str) -> String {
    if let Some(index) = line.find("//") {
        line[..index].to_string()
    } else {
        line.to_string()
    }
}

fn strip_block_comments(line: &str, in_block_comment: &mut bool) -> String {
    let bytes = line.as_bytes();
    let mut output = String::new();
    let mut index = 0usize;

    while index < bytes.len() {
        if *in_block_comment {
            if index + 1 < bytes.len() && bytes[index] == b'*' && bytes[index + 1] == b'/' {
                *in_block_comment = false;
                index += 2;
            } else {
                index += 1;
            }
            continue;
        }

        if index + 1 < bytes.len() && bytes[index] == b'/' && bytes[index + 1] == b'*' {
            *in_block_comment = true;
            index += 2;
            continue;
        }

        output.push(bytes[index] as char);
        index += 1;
    }

    output
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
struct SuppressionKey {
    path: String,
    name: String,
}

#[derive(Debug, Deserialize)]
struct SuppressionsConfig {
    #[serde(default)]
    allow: Vec<SuppressionEntry>,
}

#[derive(Debug, Deserialize)]
struct SuppressionEntry {
    path: String,
    name: String,
}

fn load_suppressions(config_path: &Path) -> Result<BTreeSet<SuppressionKey>> {
    let content = std::fs::read_to_string(config_path).with_context(|| {
        format!(
            "missing dead component suppression config: {}",
            config_path.display()
        )
    })?;
    parse_suppressions_toml(
        &content,
        &config_path
            .to_str()
            .map(ToOwned::to_owned)
            .unwrap_or_else(|| config_path.display().to_string()),
    )
}

fn parse_suppressions_toml(content: &str, source_label: &str) -> Result<BTreeSet<SuppressionKey>> {
    let parsed: SuppressionsConfig = toml::from_str(content)
        .with_context(|| format!("failed to parse suppression config: {source_label}"))?;

    let mut keys = BTreeSet::new();
    for entry in parsed.allow {
        let path = normalize_repo_relative_path(entry.path.trim());
        let name = entry.name.trim().to_string();

        if path.is_empty() {
            anyhow::bail!("suppression entry in {source_label} has empty `path`");
        }
        if name.is_empty() {
            anyhow::bail!("suppression entry in {source_label} has empty `name`");
        }
        if Path::new(&path).is_absolute() {
            anyhow::bail!(
                "suppression entry in {source_label} must use repo-relative `path`, got absolute path `{path}`"
            );
        }

        let key = SuppressionKey { path, name };
        if !keys.insert(key.clone()) {
            anyhow::bail!(
                "duplicate suppression entry in {source_label}: path=`{}` name=`{}`",
                key.path,
                key.name
            );
        }
    }
    Ok(keys)
}

fn normalize_repo_relative_path(path: &str) -> String {
    path.replace('\\', "/").trim_start_matches("./").to_string()
}

fn find_unused_suppressions(
    suppressions: &BTreeSet<SuppressionKey>,
    used: &BTreeSet<SuppressionKey>,
) -> Vec<StaleSuppressionViolation> {
    suppressions
        .difference(used)
        .cloned()
        .map(|key| StaleSuppressionViolation { key })
        .collect()
}

#[derive(Debug, Clone)]
struct StaleSuppressionViolation {
    key: SuppressionKey,
}

impl StaleSuppressionViolation {
    fn to_report(&self, config_path: String) -> Report {
        let message = format!(
            "unused dead-component suppression in {config_path}: path=`{}` name=`{}`",
            self.key.path, self.key.name
        );
        miette!(
            help = "Remove this `[[allow]]` entry because the component is no longer dead (or no longer exists).",
            "{message}"
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn source_file(path: &str, source: &str) -> SourceFile {
        SourceFile::new(PathBuf::from(path), source.to_string())
    }

    #[test]
    fn marks_component_dead_when_unreferenced() {
        let files = vec![source_file(
            "a.rs",
            r#"
#[component]
fn Lonely() -> Element { rsx! { div {} } }
"#,
        )];

        let result = scan_for_dead_components(&files, &BTreeSet::new());
        assert_eq!(result.violations.len(), 1);
        assert_eq!(result.violations[0].component.name, "Lonely");
    }

    #[test]
    fn marks_component_live_when_referenced_in_rsx() {
        let files = vec![
            source_file(
                "a.rs",
                r#"
#[component]
fn Used() -> Element { rsx! { div {} } }
"#,
            ),
            source_file(
                "b.rs",
                r#"
fn Parent() -> Element {
    rsx! { Used {} }
}
"#,
            ),
        ];

        let result = scan_for_dead_components(&files, &BTreeSet::new());
        assert!(result.violations.is_empty());
    }

    #[test]
    fn marks_component_live_when_referenced_in_layout_attribute() {
        let files = vec![
            source_file(
                "a.rs",
                r#"
#[component]
fn AppNavBar() -> Element { rsx! { div {} } }
"#,
            ),
            source_file(
                "b.rs",
                r#"
#[derive(Clone, Routable, Debug, PartialEq)]
enum Route {
    #[layout(AppNavBar)]
    #[route("/")]
    Home {},
}
"#,
            ),
        ];

        let result = scan_for_dead_components(&files, &BTreeSet::new());
        assert!(result.violations.is_empty());
    }

    #[test]
    fn enum_variants_do_not_count_as_component_references() {
        let files = vec![source_file(
            "a.rs",
            r#"
#[component]
fn Do() -> Element { rsx! { div {} } }

enum ViewContext { Do, Plan }

fn map(ctx: ViewContext) -> bool {
    matches!(ctx, ViewContext::Do)
}
"#,
        )];

        let result = scan_for_dead_components(&files, &BTreeSet::new());
        assert_eq!(result.violations.len(), 1);
        assert_eq!(result.violations[0].component.name, "Do");
    }

    #[test]
    fn string_literals_do_not_count_as_component_references() {
        let files = vec![source_file(
            "a.rs",
            r#"
#[component]
fn Do() -> Element { rsx! { div {} } }

fn label() -> &'static str { "Do" }
"#,
        )];

        let result = scan_for_dead_components(&files, &BTreeSet::new());
        assert_eq!(result.violations.len(), 1);
        assert_eq!(result.violations[0].component.name, "Do");
    }

    #[test]
    fn rsx_invocation_with_turbofish_counts_as_reference() {
        let files = vec![
            source_file(
                "a.rs",
                r#"
#[component]
fn SelectOption<T: Clone + PartialEq + 'static>() -> Element { rsx! { div {} } }
"#,
            ),
            source_file(
                "b.rs",
                r#"
fn Parent() -> Element {
    rsx! { SelectOption::<String> {} }
}
"#,
            ),
        ];

        let result = scan_for_dead_components(&files, &BTreeSet::new());
        assert!(result.violations.is_empty());
    }

    #[test]
    fn function_pointer_component_argument_counts_as_reference() {
        let files = vec![source_file(
            "a.rs",
            r#"
#[component]
fn TestApp() -> Element { rsx! { div {} } }

fn boot() {
    let _ = VirtualDom::new_with_props(TestApp, ());
}
"#,
        )];

        let result = scan_for_dead_components(&files, &BTreeSet::new());
        assert!(result.violations.is_empty());
    }

    #[test]
    fn ignores_comment_mentions_when_counting_references() {
        let files = vec![
            source_file(
                "a.rs",
                r#"
#[component]
fn Hidden() -> Element { rsx! { div {} } }
"#,
            ),
            source_file(
                "b.rs",
                r#"
// Hidden {}
/* Hidden */
"#,
            ),
        ];

        let result = scan_for_dead_components(&files, &BTreeSet::new());
        assert_eq!(result.violations.len(), 1);
        assert_eq!(result.violations[0].component.name, "Hidden");
    }

    #[test]
    fn accepts_attribute_block_between_component_and_function() {
        let files = vec![source_file(
            "a.rs",
            r#"
#[component]
#[derive(Clone, Debug)]
pub(crate) fn Fancy() -> Element { rsx! { div {} } }

fn Parent() -> Element {
    rsx! { Fancy {} }
}
"#,
        )];

        let result = scan_for_dead_components(&files, &BTreeSet::new());
        assert!(result.violations.is_empty());
    }

    #[test]
    fn ignores_non_component_functions() {
        let files = vec![source_file(
            "a.rs",
            r#"
fn plain_function() {}
"#,
        )];

        let result = scan_for_dead_components(&files, &BTreeSet::new());
        assert!(result.violations.is_empty());
    }

    #[test]
    fn suppression_requires_path_and_name_pair() {
        let config = r#"
[[allow]]
name = "CardHeader"
"#;

        let err = parse_suppressions_toml(config, "test-config").unwrap_err();
        let message = format!("{err:#}");
        assert!(message.contains("failed to parse suppression config"));
        assert!(message.contains("missing field `path`"));
    }

    #[test]
    fn suppression_matches_path_and_name() {
        let files = vec![source_file(
            "crates/tasklens-ui/src/dioxus_components/card/component.rs",
            r#"
#[component]
	fn CardHeader() -> Element { rsx! { div {} } }
"#,
        )];
        let suppressions = parse_suppressions_toml(
            r#"
[[allow]]
path = "crates/tasklens-ui/src/dioxus_components/card/component.rs"
name = "CardHeader"
"#,
            "test-config",
        )
        .unwrap();

        let result = scan_for_dead_components(&files, &suppressions);
        assert!(result.violations.is_empty());
        assert_eq!(result.used_suppressions.len(), 1);
    }

    #[test]
    fn suppression_for_same_name_other_path_does_not_apply() {
        let files = vec![source_file(
            "crates/tasklens-ui/src/dioxus_components/card/component.rs",
            r#"
#[component]
	fn CardHeader() -> Element { rsx! { div {} } }
"#,
        )];
        let suppressions = parse_suppressions_toml(
            r#"
[[allow]]
path = "crates/tasklens-ui/src/dioxus_components/dialog/component.rs"
name = "CardHeader"
"#,
            "test-config",
        )
        .unwrap();

        let result = scan_for_dead_components(&files, &suppressions);
        assert_eq!(result.violations.len(), 1);
    }

    #[test]
    fn rejects_absolute_suppression_paths() {
        let config = r#"
[[allow]]
path = "/tmp/component.rs"
name = "CardHeader"
"#;
        let err = parse_suppressions_toml(config, "test-config").unwrap_err();
        assert!(format!("{err:#}").contains("must use repo-relative `path`"));
    }

    #[test]
    fn rejects_duplicate_suppression_entries() {
        let config = r#"
[[allow]]
path = "crates/tasklens-ui/src/dioxus_components/card/component.rs"
name = "CardHeader"

[[allow]]
path = "crates/tasklens-ui/src/dioxus_components/card/component.rs"
name = "CardHeader"
"#;
        let err = parse_suppressions_toml(config, "test-config").unwrap_err();
        assert!(format!("{err:#}").contains("duplicate suppression entry"));
    }

    #[test]
    fn reports_unused_suppression_entries() {
        let suppressions = parse_suppressions_toml(
            r#"
[[allow]]
path = "crates/tasklens-ui/src/dioxus_components/card/component.rs"
name = "CardHeader"
"#,
            "test-config",
        )
        .unwrap();
        let used = BTreeSet::new();
        let stale = find_unused_suppressions(&suppressions, &used);
        assert_eq!(stale.len(), 1);
    }

    #[test]
    fn does_not_report_suppression_when_it_is_used() {
        let suppressions = parse_suppressions_toml(
            r#"
[[allow]]
path = "crates/tasklens-ui/src/dioxus_components/card/component.rs"
name = "CardHeader"
"#,
            "test-config",
        )
        .unwrap();

        let used = BTreeSet::from([SuppressionKey {
            path: "crates/tasklens-ui/src/dioxus_components/card/component.rs".to_string(),
            name: "CardHeader".to_string(),
        }]);
        let stale = find_unused_suppressions(&suppressions, &used);
        assert!(stale.is_empty());
    }
}
