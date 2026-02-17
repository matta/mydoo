use anyhow::{Context, Result};
use clap::Parser;
use regex::Regex;
use serde::Deserialize;
use std::collections::HashSet;
use std::fs;
use std::io::{BufRead, BufReader};
use std::path::Path;
use std::process::Command;

pub(crate) const DEFAULT_LIMIT: usize = 1645;

#[derive(Parser, Debug, Clone)]
pub(crate) struct CheckRustLineCountArgs {
    /// Scan all tracked files instead of just edited ones
    #[arg(long)]
    pub all: bool,

    /// Set the maximum number of non-comment, non-whitespace lines allowed
    #[arg(long, default_value_t = DEFAULT_LIMIT)]
    pub limit: usize,

    /// Just print the line counts for all files and exit
    #[arg(long)]
    pub print_counts: bool,
}

#[derive(Deserialize, Default)]
struct ExceptionsConfig {
    #[serde(default)]
    exceptions: Vec<String>,
}

pub(crate) fn check_rust_line_count(args: &CheckRustLineCountArgs) -> Result<()> {
    let root = std::env::current_dir()?;
    let config_path = root.join(".rust-line-count-exceptions.toml");

    let mut exceptions = Vec::new();
    if config_path.exists() {
        let content = fs::read_to_string(&config_path)?;
        let config: ExceptionsConfig =
            toml::from_str(&content).context("Failed to parse .rust-line-count-exceptions.toml")?;
        for pattern in config.exceptions {
            exceptions.push(Regex::new(&pattern).context(format!("Invalid regex: {}", pattern))?);
        }
    }

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
            count_lines(&path).context(format!("Failed to count lines in {}", file_path))?;
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
        println!("Max line count: {} (in {})", max_count, max_file);
        return Ok(());
    }

    if !violations.is_empty() {
        violations.sort_by(|a, b| b.1.cmp(&a.1));
        println!("The following Rust files exceed {} lines:", args.limit);
        for (file, count) in violations {
            println!("{}: {} lines", file, count);
        }
        anyhow::bail!("Files too long. Please refactor or split them.");
    }

    println!(
        "All Rust files are within the line limit ({} lines).",
        args.limit
    );
    Ok(())
}

fn count_lines(path: &Path) -> Result<usize> {
    let file = fs::File::open(path)?;
    let reader = BufReader::new(file);
    let mut count = 0;
    for line in reader.lines() {
        let line = line?;
        let trimmed = line.trim();
        if trimmed.is_empty() || trimmed.starts_with("//") {
            continue;
        }
        count += 1;
    }
    Ok(count)
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
        anyhow::bail!("Git command failed: git {}", args.join(" "));
    }

    let stdout = String::from_utf8(output.stdout)?;
    Ok(stdout.lines().map(|s| s.to_string()).collect())
}
