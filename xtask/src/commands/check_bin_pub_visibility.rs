use anyhow::{Context, Result};
use cargo_metadata::{MetadataCommand, TargetKind};
use miette::{LabeledSpan, NamedSource, Report, miette};
use regex::Regex;
use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};
use std::sync::LazyLock;

const EXCLUDED_PATH_PREFIXES: &[&str] = &["crates/tasklens-ui/src/dioxus_components/"];
const LINT_NAME: &str = "xtask-pub-visibility";

static DECLARATION_RE: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(
        r"^\s*pub\s+(fn|struct|enum|trait|type|const|static|mod|use)\b(?:\s+([A-Za-z_][A-Za-z0-9_]*))?",
    )
    .expect("valid declaration regex")
});

static SUPPRESSION_RE: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"^//\s*xtask-pub-visibility:\s*allow\s+\S.*$").expect("valid suppression regex")
});
static LINT_SAVE_RE: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"^\s*(//|/\*)\s*lint\s*-\s*save\s+-([A-Za-z_][A-Za-z0-9_-]*)\b.*$")
        .expect("valid lint save regex")
});
static LINT_RESTORE_RE: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"^\s*(//|/\*)\s*lint\s*-\s*restore(?:\s+-([A-Za-z_][A-Za-z0-9_-]*))?\b.*$")
        .expect("valid lint restore regex")
});

pub(crate) fn check_bin_pub_visibility() -> Result<()> {
    let bin_crates = discover_workspace_bin_crates()?;
    let mut violations = Vec::new();

    for bin_crate in bin_crates {
        for source_file in list_source_files(&bin_crate.src_root)? {
            if is_excluded_path(&source_file) {
                continue;
            }

            let source = std::fs::read_to_string(&source_file)
                .with_context(|| format!("Failed to read {}", source_file.display()))?;
            let source_violations = scan_source_for_violations(&source);

            for violation in source_violations {
                violations.push(WorkspaceViolation {
                    crate_name: bin_crate.package_name.clone(),
                    target_names: bin_crate.target_names.clone(),
                    path: source_file.clone(),
                    source: source.clone(),
                    line: violation.line,
                    column: violation.column,
                    span_offset: violation.span_offset,
                    item_kind: violation.item_kind,
                    item_name: violation.item_name,
                });
            }
        }
    }

    if violations.is_empty() {
        return Ok(());
    }

    for violation in &violations {
        eprintln!("{:?}", violation.to_report());
    }

    anyhow::bail!(
        "Found {} bare `pub` item(s) in binary crates.",
        violations.len()
    );
}

#[derive(Debug, Clone)]
struct BinCrate {
    package_name: String,
    target_names: Vec<String>,
    src_root: PathBuf,
}

fn discover_workspace_bin_crates() -> Result<Vec<BinCrate>> {
    let metadata = MetadataCommand::new()
        .no_deps()
        .exec()
        .context("Failed to run cargo metadata")?;

    let workspace_members: BTreeSet<_> = metadata.workspace_members.into_iter().collect();
    let mut bins_by_package: BTreeMap<String, BinCrate> = BTreeMap::new();

    for package in metadata.packages {
        if !workspace_members.contains(&package.id) {
            continue;
        }

        let target_names: Vec<String> = package
            .targets
            .iter()
            .filter(|target| target.kind.iter().any(|kind| kind == &TargetKind::Bin))
            .map(|target| target.name.clone())
            .collect();

        if target_names.is_empty() {
            continue;
        }

        let manifest_dir = package
            .manifest_path
            .parent()
            .context("manifest path has no parent directory")?
            .as_std_path()
            .to_path_buf();
        let default_src = manifest_dir.join("src");
        let src_root = if default_src.is_dir() {
            default_src
        } else {
            package
                .targets
                .iter()
                .find(|target| target.kind.iter().any(|kind| kind == &TargetKind::Bin))
                .and_then(|target| target.src_path.parent())
                .map(|path| path.as_std_path().to_path_buf())
                .unwrap_or(manifest_dir)
        };

        bins_by_package.insert(
            package.id.repr.clone(),
            BinCrate {
                package_name: package.name.to_string(),
                target_names,
                src_root,
            },
        );
    }

    Ok(bins_by_package.into_values().collect())
}

fn list_source_files(src_root: &Path) -> Result<Vec<PathBuf>> {
    let pattern = format!("{}/**/*.rs", src_root.display());
    let mut files = Vec::new();

    for entry in glob::glob(&pattern).context("Invalid glob pattern")? {
        let path = entry.with_context(|| format!("Failed to read glob entry for {pattern}"))?;
        files.push(path);
    }

    files.sort();
    Ok(files)
}

fn is_excluded_path(path: &Path) -> bool {
    let normalized = path.to_string_lossy().replace('\\', "/");
    EXCLUDED_PATH_PREFIXES
        .iter()
        .any(|prefix| normalized.contains(prefix))
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct SourceViolation {
    line: usize,
    column: usize,
    span_offset: usize,
    item_kind: String,
    item_name: String,
}

fn scan_source_for_violations(source: &str) -> Vec<SourceViolation> {
    let lines: Vec<&str> = source.split('\n').collect();
    let line_starts = line_start_offsets(&lines);

    let mut violations = Vec::new();
    let mut in_block_comment = false;
    let mut lint_suppression_depth = 0usize;

    for (index, line) in lines.iter().enumerate() {
        match lint_region_directive(line.trim()) {
            LintRegionDirective::Save => {
                lint_suppression_depth = lint_suppression_depth.saturating_add(1);
                continue;
            }
            LintRegionDirective::Restore => {
                lint_suppression_depth = lint_suppression_depth.saturating_sub(1);
                continue;
            }
            LintRegionDirective::None => {}
        }

        let line_without_blocks = strip_block_comments(line, &mut in_block_comment);
        if line_without_blocks.trim_start().starts_with("//") {
            continue;
        }

        let Some(captures) = DECLARATION_RE.captures(&line_without_blocks) else {
            continue;
        };

        if lint_suppression_depth > 0 {
            continue;
        }

        if has_same_line_suppression(line) || has_previous_line_suppression(&lines, index) {
            continue;
        }

        let pub_column = line_without_blocks.find("pub").unwrap_or_default();
        let line_start = line_starts.get(index).copied().unwrap_or_default();
        let kind = captures
            .get(1)
            .map(|m| m.as_str().to_string())
            .unwrap_or_else(|| "item".to_string());
        let name = captures
            .get(2)
            .map(|m| m.as_str().to_string())
            .unwrap_or_else(|| "<unknown>".to_string());

        violations.push(SourceViolation {
            line: index + 1,
            column: pub_column + 1,
            span_offset: line_start + pub_column,
            item_kind: kind,
            item_name: name,
        });
    }

    violations
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum LintRegionDirective {
    Save,
    Restore,
    None,
}

fn lint_region_directive(line: &str) -> LintRegionDirective {
    if let Some(captures) = LINT_SAVE_RE.captures(line)
        && captures
            .get(2)
            .map(|m| is_matching_lint_name(m.as_str()))
            .unwrap_or(false)
    {
        return LintRegionDirective::Save;
    }

    if let Some(captures) = LINT_RESTORE_RE.captures(line) {
        // `lint -restore` without name restores all; with a name it must match.
        if captures
            .get(2)
            .map(|m| is_matching_lint_name(m.as_str()))
            .unwrap_or(true)
        {
            return LintRegionDirective::Restore;
        }
    }

    LintRegionDirective::None
}

fn is_matching_lint_name(candidate: &str) -> bool {
    normalize_lint_name(candidate) == normalize_lint_name(LINT_NAME)
}

fn normalize_lint_name(input: &str) -> String {
    input.trim().to_lowercase().replace('_', "-")
}

fn has_same_line_suppression(line: &str) -> bool {
    let Some(comment_start) = line.find("//") else {
        return false;
    };
    let comment = line[comment_start..].trim();
    SUPPRESSION_RE.is_match(comment)
}

fn has_previous_line_suppression(lines: &[&str], declaration_index: usize) -> bool {
    if declaration_index == 0 {
        return false;
    }

    let mut index = declaration_index as isize - 1;
    while index >= 0 {
        let line = lines[index as usize].trim();
        if SUPPRESSION_RE.is_match(line) {
            return true;
        }

        if line.is_empty()
            || line.starts_with("#[")
            || line.starts_with("///")
            || line.starts_with("//!")
        {
            index -= 1;
            continue;
        }

        return false;
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

#[derive(Debug, Clone)]
struct WorkspaceViolation {
    crate_name: String,
    target_names: Vec<String>,
    path: PathBuf,
    source: String,
    line: usize,
    column: usize,
    span_offset: usize,
    item_kind: String,
    item_name: String,
}

impl WorkspaceViolation {
    fn to_report(&self) -> Report {
        let target_names = self.target_names.join(", ");
        let display_path = self.path.display().to_string();
        let message = format!(
            "bare `pub` in binary crate `{}` (targets: {}): `{}` `{}` at {}:{}:{}",
            self.crate_name,
            target_names,
            self.item_kind,
            self.item_name,
            display_path,
            self.line,
            self.column,
        );
        miette!(
            labels = vec![LabeledSpan::at(
                self.span_offset..self.span_offset + 3,
                "bare `pub` should be `pub(crate)` or narrower"
            )],
            help = "Use `pub(crate)` (or narrower visibility), or add a local xtask suppression comment with a reason.",
            "{message}"
        )
        .with_source_code(NamedSource::new(display_path, self.source.clone()))
    }
}

// lint -save -xtask-pub-visibility
#[cfg(test)]
mod tests {
    use super::*;

    fn scan(source: &str) -> Vec<SourceViolation> {
        scan_source_for_violations(source)
    }

    #[test]
    fn finds_bare_pub_fn() {
        let source = r#"
pub fn run() {}
"#;
        let violations = scan(source);
        assert_eq!(violations.len(), 1);
        assert_eq!(violations[0].line, 2);
        assert_eq!(violations[0].item_kind, "fn");
        assert_eq!(violations[0].item_name, "run");
    }

    #[test]
    fn ignores_pub_crate_fn() {
        let source = r#"
pub(crate) fn run() {}
"#;
        assert!(scan(source).is_empty());
    }

    #[test]
    fn ignores_line_comment_text() {
        let source = r#"
// pub fn fake() {}
fn real() {}
"#;
        assert!(scan(source).is_empty());
    }

    #[test]
    fn ignores_block_comment_text() {
        let source = r#"
/*
pub fn fake() {}
*/
fn real() {}
"#;
        assert!(scan(source).is_empty());
    }

    #[test]
    fn allows_same_line_suppression() {
        let source = r#"
pub fn run() {} // xtask-pub-visibility: allow integration test surface
"#;
        assert!(scan(source).is_empty());
    }

    #[test]
    fn allows_previous_line_suppression_across_attribute_block() {
        let source = r#"
// xtask-pub-visibility: allow integration test surface
#[derive(Debug, Clone)]
pub struct AppState {}
"#;
        assert!(scan(source).is_empty());
    }

    #[test]
    fn accepts_suppression_comment_above_attribute_block() {
        let source = r#"
// xtask-pub-visibility: allow external adapter
#[cfg(test)]
#[derive(Debug)]
pub struct AppState {}
"#;
        assert!(scan(source).is_empty());
    }

    #[test]
    fn malformed_suppression_does_not_suppress() {
        let source = r#"
// xtask-pub-visibility: allow
pub fn run() {}
"#;
        let violations = scan(source);
        assert_eq!(violations.len(), 1);
    }

    #[test]
    fn line_break_evasion_is_not_detected_in_v1() {
        let source = r#"
pub
fn run() {}
"#;
        assert!(scan(source).is_empty());
    }

    #[test]
    fn allows_region_suppression_with_lint_save_restore() {
        let source = r#"
// lint -save -xtask-pub-visibility
pub fn run() {}
// lint -restore
"#;
        assert!(scan(source).is_empty());
    }

    #[test]
    fn lint_restore_reenables_detection() {
        let source = r#"
// lint -save -xtask-pub-visibility
pub fn suppressed() {}
// lint -restore -xtask_pub_visibility
pub fn active() {}
"#;
        let violations = scan(source);
        assert_eq!(violations.len(), 1);
        assert_eq!(violations[0].item_name, "active");
    }

    #[test]
    fn ignores_save_for_other_lint_names() {
        let source = r#"
// lint -save -some_other_lint
pub fn run() {}
// lint -restore
"#;
        let violations = scan(source);
        assert_eq!(violations.len(), 1);
    }
}
// lint -restore
