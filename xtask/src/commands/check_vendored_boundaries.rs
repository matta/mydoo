use anyhow::{Context, Result, bail};
use globset::{Glob, GlobSet, GlobSetBuilder};
use std::fs;
use std::path::Path;
use std::process::Command;

use crate::commands::rust_tokens::{ParsedRustFile, parse_rust_file_cached};

const VENDORED_RS_GLOB: &str = "crates/tasklens-ui/src/dioxus_components/**/*.rs";
const ALLOWLIST_PATH: &str = "xtask/config/vendored-boundary.allowlist";

#[derive(Debug, Clone, PartialEq, Eq)]
struct Violation {
    file: String,
    line: usize,
    snippet: String,
}

pub(crate) fn check_vendored_boundaries() -> Result<()> {
    let tracked_files = get_tracked_vendored_files()?;
    let allowlist = load_allowlist(ALLOWLIST_PATH)?;

    let mut violations = Vec::new();
    for file in tracked_files {
        if allowlist.is_match(&file) {
            continue;
        }

        let parsed = parse_rust_file_cached(Path::new(&file))
            .with_context(|| format!("failed to parse vendored file: {file}"))?;
        violations.extend(find_violations(&file, &parsed));
    }

    if violations.is_empty() {
        return Ok(());
    }

    eprintln!("Vendored boundary violations detected.");
    eprintln!(
        "Vendored Dioxus code must not use `crate::...` paths. Keep vendored internals self-contained."
    );
    eprintln!(
        "If an exception is intentional, add the vendored file path to {ALLOWLIST_PATH} with a rationale comment."
    );
    eprintln!();

    for violation in violations {
        eprintln!(
            "{}:{}: {}",
            violation.file, violation.line, violation.snippet
        );
    }

    bail!("vendored boundary check failed");
}

fn get_tracked_vendored_files() -> Result<Vec<String>> {
    let output = Command::new("git")
        .args(["ls-files", VENDORED_RS_GLOB])
        .output()
        .context("failed to run `git ls-files` for vendored component files")?;

    if !output.status.success() {
        bail!("`git ls-files` failed while listing vendored component files");
    }

    let stdout = String::from_utf8(output.stdout).context("git output was not utf-8")?;
    Ok(stdout
        .lines()
        .map(str::trim)
        .filter(|line| !line.is_empty())
        .map(ToOwned::to_owned)
        .collect())
}

fn find_violations(file: &str, parsed: &ParsedRustFile) -> Vec<Violation> {
    let mut violations = Vec::new();

    for window in parsed.tokens.windows(3) {
        let first = &window[0];
        let second = &window[1];
        let third = &window[2];

        if first.kind == ra_ap_rustc_lexer::TokenKind::Ident
            && second.kind == ra_ap_rustc_lexer::TokenKind::Colon
            && third.kind == ra_ap_rustc_lexer::TokenKind::Colon
            && parsed.content[first.start..first.end] == *"crate"
        {
            let line_number = line_number_for_offset(&parsed.content, first.start);
            let snippet = parsed
                .content
                .lines()
                .nth(line_number - 1)
                .map(str::trim)
                .unwrap_or_default()
                .to_string();

            violations.push(Violation {
                file: file.to_string(),
                line: line_number,
                snippet,
            });
        }
    }

    violations
}

fn line_number_for_offset(content: &str, offset: usize) -> usize {
    content[..offset].bytes().filter(|b| *b == b'\n').count() + 1
}

fn load_allowlist(path: &str) -> Result<GlobSet> {
    let mut builder = GlobSetBuilder::new();
    let mut require_comment = false;

    let content =
        fs::read_to_string(path).with_context(|| format!("missing allowlist file: {path}"))?;
    for (idx, raw_line) in content.lines().enumerate() {
        let line_number = idx + 1;
        let line = raw_line.trim();
        if line.is_empty() {
            continue;
        }

        if line.starts_with('#') {
            require_comment = true;
            continue;
        }

        if !require_comment {
            bail!(
                "allowlist entry at {path}:{line_number} must be preceded by a rationale comment"
            );
        }

        let glob = Glob::new(line)
            .with_context(|| format!("invalid allowlist glob at {path}:{line_number}: {line}"))?;
        builder.add(glob);
        require_comment = false;
    }

    builder
        .build()
        .with_context(|| format!("failed to compile allowlist globs from {path}"))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::commands::rust_tokens::parse_rust_source;
    use tempfile::tempdir;

    #[test]
    fn finds_any_crate_path_violation() {
        let content = r#"
use crate::app_components::layout::Stack;

fn render() {
    let _ = crate::sync::State::default();
}
"#;

        let parsed = parse_rust_source(content);
        let violations = find_violations("vendored.rs", &parsed);
        assert_eq!(violations.len(), 2);
        assert_eq!(violations[0].line, 2);
        assert_eq!(violations[1].line, 5);
    }

    #[test]
    fn ignores_comment_and_string_mentions() {
        let content = r#"
// crate::app_components::layout::Stack
let s = "crate::views::Route";
"#;

        let parsed = parse_rust_source(content);
        let violations = find_violations("vendored.rs", &parsed);
        assert!(violations.is_empty());
    }

    #[test]
    fn ignores_relative_paths() {
        let content = r#"
use self::component::Dialog;
use super::state::DialogState;
"#;

        let parsed = parse_rust_source(content);
        let violations = find_violations("vendored.rs", &parsed);
        assert!(violations.is_empty());
    }

    #[test]
    fn allowlist_requires_rationale_comment() {
        let temp = tempdir().unwrap();
        let path = temp.path().join("allowlist.txt");
        fs::write(
            &path,
            "crates/tasklens-ui/src/dioxus_components/dialog/component.rs\n",
        )
        .unwrap();

        let err = load_allowlist(path_to_str(&path)).unwrap_err().to_string();
        assert!(err.contains("must be preceded by a rationale comment"));
    }

    #[test]
    fn allowlist_accepts_commented_entry() {
        let temp = tempdir().unwrap();
        let path = temp.path().join("allowlist.txt");
        fs::write(
            &path,
            "# Needed for explicit product divergence\ncrates/tasklens-ui/src/dioxus_components/dialog/component.rs\n",
        )
        .unwrap();

        let allowlist = load_allowlist(path_to_str(&path)).unwrap();
        assert!(allowlist.is_match("crates/tasklens-ui/src/dioxus_components/dialog/component.rs"));
        assert!(
            !allowlist.is_match("crates/tasklens-ui/src/dioxus_components/button/component.rs")
        );
    }

    fn path_to_str(path: &Path) -> &str {
        path.to_str().expect("temp path should be utf-8")
    }
}
