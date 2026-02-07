use anyhow::{Context, Result};
use regex::Regex;
use std::fs;
use std::process::Command;
use std::sync::LazyLock;

struct DarkModeRule {
    light_pattern: Regex,
    dark_prefix: &'static str,
    description: &'static str,
    suggestion: &'static str,
}

impl DarkModeRule {
    fn new(
        light_pattern: &str,
        dark_prefix: &'static str,
        description: &'static str,
        suggestion: &'static str,
    ) -> Self {
        Self {
            light_pattern: Regex::new(light_pattern).expect("Invalid regex"),
            dark_prefix,
            description,
            suggestion,
        }
    }
}

static RULES: LazyLock<Vec<DarkModeRule>> = LazyLock::new(|| {
    vec![
        DarkModeRule::new(
            r"^bg-white$",
            "dark:bg-",
            "bg-white without dark variant",
            "Add dark:bg-stone-900 or dark:bg-stone-950",
        ),
        DarkModeRule::new(
            r"^bg-gray-50$",
            "dark:bg-",
            "bg-gray-50 without dark variant",
            "Add dark:bg-stone-800",
        ),
        DarkModeRule::new(
            r"^bg-gray-100$",
            "dark:bg-",
            "bg-gray-100 without dark variant",
            "Add dark:bg-stone-700",
        ),
        DarkModeRule::new(
            r"^bg-gray-200$",
            "dark:bg-",
            "bg-gray-200 without dark variant",
            "Add dark:bg-stone-700",
        ),
        DarkModeRule::new(
            r"^bg-gray-300$",
            "dark:bg-",
            "bg-gray-300 without dark variant",
            "Add dark:bg-stone-600",
        ),
        DarkModeRule::new(
            r"^text-gray-[4-9]00$",
            "dark:text-",
            "text-gray-* without dark variant",
            "Add dark:text-stone-* variant (e.g., text-gray-500 -> dark:text-stone-400)",
        ),
        DarkModeRule::new(
            r"^text-gray-900$",
            "dark:text-",
            "text-gray-900 without dark variant",
            "Add dark:text-stone-100 or dark:text-white",
        ),
        DarkModeRule::new(
            r"^text-gray-800$",
            "dark:text-",
            "text-gray-800 without dark variant",
            "Add dark:text-stone-100 or dark:text-stone-200",
        ),
        DarkModeRule::new(
            r"^text-gray-700$",
            "dark:text-",
            "text-gray-700 without dark variant",
            "Add dark:text-stone-200 or dark:text-stone-300",
        ),
        DarkModeRule::new(
            r"^border-gray-[1-3]00$",
            "dark:border-",
            "border-gray-* without dark variant",
            "Add dark:border-stone-600 or dark:border-stone-700",
        ),
        DarkModeRule::new(
            r"^hover:bg-gray-50$",
            "dark:hover:bg-",
            "hover:bg-gray-50 without dark variant",
            "Add dark:hover:bg-stone-800",
        ),
        DarkModeRule::new(
            r"^hover:bg-gray-100$",
            "dark:hover:bg-",
            "hover:bg-gray-100 without dark variant",
            "Add dark:hover:bg-stone-700",
        ),
        DarkModeRule::new(
            r"^hover:bg-gray-200$",
            "dark:hover:bg-",
            "hover:bg-gray-200 without dark variant",
            "Add dark:hover:bg-stone-700",
        ),
        DarkModeRule::new(
            r"^hover:text-gray-700$",
            "dark:hover:text-",
            "hover:text-gray-700 without dark variant",
            "Add dark:hover:text-stone-200",
        ),
        DarkModeRule::new(
            r"^hover:text-gray-800$",
            "dark:hover:text-",
            "hover:text-gray-800 without dark variant",
            "Add dark:hover:text-stone-100",
        ),
        DarkModeRule::new(
            r"^placeholder-gray-[3-5]00$",
            "dark:placeholder-",
            "placeholder-gray-* without dark variant",
            "Add dark:placeholder-stone-500",
        ),
    ]
});

struct Violation {
    file: String,
    line: usize,
    content: String,
    description: &'static str,
    suggestion: &'static str,
}

pub(crate) fn lint_dark_mode() -> Result<()> {
    let files = get_ui_rust_files().context("Failed to get UI Rust files")?;
    let mut all_violations = Vec::new();

    for file in files {
        let violations =
            check_file(&file).with_context(|| format!("Failed to check file: {}", file))?;
        all_violations.extend(violations);
    }

    if all_violations.is_empty() {
        return Ok(());
    }

    eprintln!("\n🌙 Dark Mode Lint Results\n");
    eprintln!(
        "Found {} potential dark mode violation(s) in UI components:\n",
        all_violations.len()
    );

    let mut current_file = String::new();
    for v in all_violations {
        if v.file != current_file {
            eprintln!("📁 {}", v.file);
            current_file = v.file.clone();
        }
        eprintln!("   Line {}: {}", v.line, v.description);
        eprintln!("   {}", v.content.trim());
        eprintln!("   💡 {}\n", v.suggestion);
    }

    anyhow::bail!("Dark mode linting failed.");
}

fn get_ui_rust_files() -> Result<Vec<String>> {
    let output = Command::new("git")
        .args(["ls-files", "crates/tasklens-ui/**/*.rs"])
        .output()
        .context("Failed to run git ls-files")?;

    if !output.status.success() {
        anyhow::bail!("git ls-files failed");
    }

    let stdout = String::from_utf8(output.stdout)?;
    Ok(stdout.lines().map(|s| s.to_string()).collect())
}

fn check_file(file_path: &str) -> Result<Vec<Violation>> {
    let content = fs::read_to_string(file_path)?;
    let mut violations = Vec::new();

    static RE_SIMPLE: LazyLock<Regex> =
        LazyLock::new(|| Regex::new(r#"class[:\s=]+["']([^"']+)["']"#).unwrap());
    static RE_FORMAT_ARGS: LazyLock<Regex> =
        LazyLock::new(|| Regex::new(r#"class:\s*format_args!\s*\(\s*["']([^"']+)["']"#).unwrap());

    for (line_num, line) in content.lines().enumerate() {
        if !line.contains("class:") && !line.contains("class =") {
            continue;
        }

        let mut class_strings = Vec::new();
        for cap in RE_SIMPLE.captures_iter(line) {
            class_strings.push(cap[1].to_string());
        }
        for cap in RE_FORMAT_ARGS.captures_iter(line) {
            class_strings.push(cap[1].to_string());
        }

        for class_string in class_strings {
            let tokens: Vec<&str> = class_string.split_whitespace().collect();

            for rule in RULES.iter() {
                // Check if any token matches the light pattern
                let matches_light = tokens.iter().any(|&t| rule.light_pattern.is_match(t));

                if matches_light {
                    // Check if there's a corresponding dark variant anywhere in the same class string
                    let has_dark = tokens.iter().any(|&t| t.starts_with(rule.dark_prefix));

                    if !has_dark {
                        violations.push(Violation {
                            file: file_path.to_string(),
                            line: line_num + 1,
                            content: line.to_string(),
                            description: rule.description,
                            suggestion: rule.suggestion,
                        });
                    }
                }
            }
        }
    }

    Ok(violations)
}
