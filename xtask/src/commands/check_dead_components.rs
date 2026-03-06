use anyhow::{Context, Result};
use miette::{LabeledSpan, NamedSource, Report, miette};
use regex::Regex;
use serde::Deserialize;
use std::collections::BTreeSet;
use std::path::{Path, PathBuf};
use std::sync::LazyLock;

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
}

fn load_source_files(src_root: &Path) -> Result<Vec<SourceFile>> {
    let pattern = format!("{}/**/*.rs", src_root.display());
    let mut files = Vec::new();

    for entry in glob::glob(&pattern).context("Invalid glob pattern")? {
        let path = entry.with_context(|| format!("Failed to read glob entry for {pattern}"))?;
        let source = std::fs::read_to_string(&path)
            .with_context(|| format!("Failed to read {}", path.display()))?;
        files.push(SourceFile::new(path, source));
    }

    files.sort_by(|left, right| left.path.cmp(&right.path));
    Ok(files)
}

impl SourceFile {
    fn new(path: PathBuf, source: String) -> Self {
        let raw_lines: Vec<&str> = source.split('\n').collect();
        let line_starts = line_start_offsets(&raw_lines);
        let code_lines = strip_comments_for_search(&raw_lines);
        Self {
            path,
            source,
            code_lines,
            line_starts,
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
    let pattern = format!(r"\b{}\b", regex::escape(&component.name));
    let reference_re = Regex::new(&pattern).expect("component name regex");

    for file in files {
        for (line_index, code_line) in file.code_lines.iter().enumerate() {
            for name_match in reference_re.find_iter(code_line) {
                let is_declaration = file.path == component.path
                    && line_index + 1 == component.line
                    && name_match.start() + 1 == component.column;
                if !is_declaration {
                    return true;
                }
            }
        }
    }

    false
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
