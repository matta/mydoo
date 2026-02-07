use anyhow::{Context, Result};
use regex::Regex;
use std::process::Command;

const ALLOWED_FILES: &[&str] = &["context/README.md"];
const ALLOWED_PREFIXES: &[&str] = &[
    // Example: "context/allowed-sub-repo/"
];

/// Context Lint Command
///
/// Prevents accidental commits of external git repositories or unauthorized files
/// into the context directory.
pub(crate) fn check_context() -> Result<()> {
    let tracked_files = get_context_tracked_files()?;

    let mut error_count = 0;

    // Patterns that are strictly forbidden (e.g., git internal files)
    let forbidden_patterns = [
        Regex::new(r"/\.git/").expect("Invalid regex"),
        Regex::new(r"/\.git$").expect("Invalid regex"),
    ];

    for file in tracked_files {
        let mut is_forbidden = false;

        for pattern in &forbidden_patterns {
            if pattern.is_match(&file) {
                eprintln!("❌ ERROR: Forbidden git-related file detected: {}", file);
                error_count += 1;
                is_forbidden = true;
                break;
            }
        }

        if is_forbidden {
            continue;
        }

        // Check whitelist
        let is_whitelisted_file = ALLOWED_FILES.contains(&file.as_str());
        let is_whitelisted_prefix = ALLOWED_PREFIXES
            .iter()
            .any(|prefix| file.starts_with(prefix));

        if !is_whitelisted_file && !is_whitelisted_prefix {
            eprintln!("❌ ERROR: Untrusted file in context: {}", file);
            eprintln!(
                "   If this file is intentional, add it to the whitelist in xtask/src/commands/lint_context.rs"
            );
            error_count += 1;
        }
    }

    if error_count > 0 {
        anyhow::bail!("🚨 Found {} lint error(s) in context/", error_count);
    }

    Ok(())
}

fn get_context_tracked_files() -> Result<Vec<String>> {
    let output = Command::new("git")
        .args(["ls-files", "context/"])
        .output()
        .context("Failed to run git ls-files context/")?;

    if !output.status.success() {
        // If directory doesn't exist or git fails, return empty
        return Ok(Vec::new());
    }

    let stdout = String::from_utf8(output.stdout)?;
    Ok(stdout
        .lines()
        .map(|s| s.to_string())
        .filter(|s| !s.is_empty())
        .collect())
}
