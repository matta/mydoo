use anyhow::{Context, Result};
use clap::Parser;
use regex::Regex;
use serde::Deserialize;
use std::collections::HashSet;
use std::fs;
use std::path::Path;
use std::process::Command;

use crate::commands::rust_tokens::parse_rust_file_cached;

pub(crate) const DEFAULT_LIMIT: usize = 9201;

#[derive(Parser, Debug, Clone)]
pub(crate) struct CheckRustTokenCountArgs {
    /// Scan all tracked files instead of just edited ones
    #[arg(long)]
    pub all: bool,

    /// Set the maximum number of non-comment, non-whitespace tokens allowed
    #[arg(long, default_value_t = DEFAULT_LIMIT)]
    pub limit: usize,

    /// Just print the token counts for all files and exit
    #[arg(long)]
    pub print_counts: bool,
}

#[derive(Deserialize, Default)]
struct ExceptionsConfig {
    #[serde(default)]
    exceptions: Vec<String>,
}

pub(crate) fn check_rust_token_count(args: &CheckRustTokenCountArgs) -> Result<()> {
    let root = std::env::current_dir()?;
    let config_path = root.join(".rust-line-count-exceptions.toml");

    let exceptions = if config_path.exists() {
        let content = fs::read_to_string(&config_path)?;
        let config: ExceptionsConfig =
            toml::from_str(&content).context("Failed to parse .rust-line-count-exceptions.toml")?;
        config
            .exceptions
            .into_iter()
            .map(|pattern| {
                Regex::new(&pattern).with_context(|| format!("Invalid regex: {}", pattern))
            })
            .collect::<Result<Vec<_>>>()?
    } else {
        Vec::new()
    };

    let files = get_files_to_check(&root, args.all)?;
    let mut violations = Vec::new();
    let mut max_count = 0;
    let mut max_file = String::new();

    for file_path in files {
        let path = root.join(&file_path);
        if !path.exists() || path.is_dir() {
            continue;
        }

        // Only check Rust files
        if path.extension().and_then(|s| s.to_str()) != Some("rs") {
            continue;
        }

        if exceptions.iter().any(|re| re.is_match(&file_path)) {
            continue;
        }

        let count =
            count_tokens(&path).context(format!("Failed to count tokens in {}", file_path))?;
        if count > max_count {
            max_count = count;
            max_file = file_path.clone();
        }

        if args.print_counts {
            println!("{}: {}", file_path, count);
        }

        if count > args.limit {
            violations.push((file_path, count));
        }
    }

    if args.print_counts {
        println!("Max token count: {} (in {})", max_count, max_file);
        return Ok(());
    }

    if !violations.is_empty() {
        violations.sort_by(|a, b| b.1.cmp(&a.1));
        println!("The following Rust files exceed {} tokens:", args.limit);
        for (file, count) in violations {
            println!("{}: {} tokens", file, count);
        }
        anyhow::bail!("Files too long. Please refactor or split them.");
    }

    println!(
        "All Rust files are within the token limit ({} tokens).",
        args.limit
    );
    Ok(())
}

fn count_tokens(path: &Path) -> Result<usize> {
    let parsed = parse_rust_file_cached(path)?;
    Ok(parsed.non_trivia_token_count)
}

fn get_files_to_check(root: &Path, all: bool) -> Result<HashSet<String>> {
    if all {
        Ok(get_git_files(root, &["ls-files"])?.into_iter().collect())
    } else {
        // Get both staged and unstaged changes
        let mut files: HashSet<String> = get_git_files(root, &["diff", "--name-only", "HEAD"])?
            .into_iter()
            .collect();
        // Also get untracked files
        files.extend(get_git_files(
            root,
            &["ls-files", "--others", "--exclude-standard"],
        )?);

        if files.is_empty() {
            // If the repo is clean, default to checking all tracked files
            Ok(get_git_files(root, &["ls-files"])?.into_iter().collect())
        } else {
            Ok(files)
        }
    }
}

fn get_git_files(root: &Path, args: &[&str]) -> Result<Vec<String>> {
    let output = Command::new("git")
        .current_dir(root)
        .args(args)
        .output()
        .context(format!("Failed to run git {}", args.join(" ")))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        anyhow::bail!("Git command `git {}` failed:\n{}", args.join(" "), stderr);
    }

    let stdout = String::from_utf8(output.stdout)?;
    Ok(stdout.lines().map(|s| s.to_string()).collect())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::path::Path;
    use std::process::Command;
    use tempfile::tempdir;

    fn git(dir: &Path, args: &[&str]) {
        let status = Command::new("git")
            .current_dir(dir)
            .args(args)
            .status()
            .expect("Failed to run git");
        assert!(
            status.success(),
            "Git command failed: git {}",
            args.join(" ")
        );
    }

    #[test]
    fn test_get_files_to_check_staged_and_unstaged() {
        let temp = tempdir().unwrap();
        let root = temp.path();

        // Init git repo
        git(root, &["init"]);
        git(root, &["config", "user.email", "test@example.com"]);
        git(root, &["config", "user.name", "Test User"]);
        git(root, &["config", "commit.gpgsign", "false"]);

        // Create initial commit
        let f1 = "file1.rs";
        let f4 = "file4.rs";
        fs::write(root.join(f1), "fn main() {}\n").unwrap();
        fs::write(root.join(f4), "fn main() {}\n").unwrap();
        git(root, &["add", f1, f4]);
        git(root, &["commit", "-m", "initial"]);

        // 1. Modify f1 (unstaged)
        fs::write(root.join(f1), "fn main() { // unstaged }\n").unwrap();

        // 2. Create f2 (staged)
        let f2 = "file2.rs";
        fs::write(root.join(f2), "fn main() {}\n").unwrap();
        git(root, &["add", f2]);

        // 3. Create f3 (untracked)
        let f3 = "file3.rs";
        fs::write(root.join(f3), "fn main() {}\n").unwrap();

        // 4. Modify f4 (staged)
        fs::write(root.join(f4), "fn main() { // staged }\n").unwrap();
        git(root, &["add", f4]);

        let files = get_files_to_check(root, false).unwrap();

        assert!(files.contains(f1), "Should contain unstaged f1.rs");
        assert!(files.contains(f2), "Should contain staged new f2.rs");
        assert!(files.contains(f3), "Should contain untracked f3.rs");
        assert!(files.contains(f4), "Should contain staged modified f4.rs");
        assert_eq!(files.len(), 4);
    }

    #[test]
    fn test_get_files_to_check_clean_repo() {
        let temp = tempdir().unwrap();
        let root = temp.path();
        git(root, &["init"]);
        git(root, &["config", "user.email", "test@example.com"]);
        git(root, &["config", "user.name", "Test User"]);
        git(root, &["config", "commit.gpgsign", "false"]);

        let f1 = "file1.rs";
        fs::write(root.join(f1), "fn main() {}\n").unwrap();
        git(root, &["add", f1]);
        git(root, &["commit", "-m", "initial"]);

        let files = get_files_to_check(root, false).unwrap();
        // If repo is clean, it should return all tracked files
        assert!(files.contains(f1));
        assert_eq!(files.len(), 1);
    }

    #[test]
    fn test_get_files_to_check_all() {
        let temp = tempdir().unwrap();
        let root = temp.path();
        git(root, &["init"]);
        git(root, &["config", "user.email", "test@example.com"]);
        git(root, &["config", "user.name", "Test User"]);
        git(root, &["config", "commit.gpgsign", "false"]);

        let f1 = "file1.rs";
        fs::write(root.join(f1), "fn main() {}\n").unwrap();
        git(root, &["add", f1]);
        git(root, &["commit", "-m", "initial"]);

        let f2 = "file2.rs";
        fs::write(root.join(f2), "fn main() {}\n").unwrap();

        let files = get_files_to_check(root, true).unwrap();
        assert!(files.contains(f1));
        assert!(!files.contains(f2)); // ls-files only shows tracked by default
        assert_eq!(files.len(), 1);
    }

    #[test]
    fn test_count_tokens() {
        let temp = tempdir().unwrap();
        let path = temp.path().join("test.rs");

        let code = r##"
            fn main() {
                // This is a comment
                let x = 1; /* This is also a comment */
                let s = "This is a string // with a comment inside";
            }
        "##;
        fs::write(&path, code).unwrap();

        let count = count_tokens(&path).unwrap();
        // Tokens:
        // 1: fn
        // 2: main
        // 3: (
        // 4: )
        // 5: {
        // 6: let
        // 7: x
        // 8: =
        // 9: 1
        // 10: ;
        // 11: let
        // 12: s
        // 13: =
        // 14: "This is a string // with a comment inside"
        // 15: ;
        // 16: }
        assert_eq!(count, 16);
    }
}
