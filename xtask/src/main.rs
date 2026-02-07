use anyhow::{Context, Result};
use clap::{Parser, Subcommand};
use glob::glob;
use regex::Regex;
use std::collections::{HashMap, HashSet};
use std::fs;
use std::path::PathBuf;
use std::sync::LazyLock;

#[derive(Parser)]
#[command(name = "xtask")]
#[command(about = "Development automation scripts", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Check for unused entries in the pnpm-workspace.yaml catalog
    CheckCatalog,
    /// Lint filenames for naming conventions
    LintFilenames,
    /// Run all checks
    CheckAll,
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    match &cli.command {
        Commands::CheckCatalog => check_catalog()?,
        Commands::LintFilenames => lint_filenames()?,
        Commands::CheckAll => {
            check_catalog()?;
            lint_filenames()?;
        }
    }

    Ok(())
}

/// Checks that all entries defined in `pnpm-workspace.yaml` catalogs are
/// actually used by some `package.json` in the workspace. Fails if any
/// entries are unused.
fn check_catalog() -> Result<()> {
    let root_dir = std::env::current_dir()?;
    let workspace_path = root_dir.join("pnpm-workspace.yaml");

    if !workspace_path.exists() {
        anyhow::bail!(
            "pnpm-workspace.yaml not found at {}",
            workspace_path.display()
        );
    }

    // 1. Parse pnpm-workspace.yaml to find DEFINED catalog entries
    let workspace_content =
        fs::read_to_string(&workspace_path).context("Failed to read pnpm-workspace.yaml")?;

    let docs = yaml_rust2::YamlLoader::load_from_str(&workspace_content)
        .context("Failed to parse pnpm-workspace.yaml")?;

    if docs.is_empty() {
        anyhow::bail!("pnpm-workspace.yaml is empty");
    }

    let workspace = &docs[0];

    // Access "catalogs" field
    let catalogs = &workspace["catalogs"];

    // Check if it's a hash map
    let catalogs_map = catalogs.as_hash().ok_or_else(|| {
        anyhow::anyhow!("No 'catalogs' section found in pnpm-workspace.yaml useable as a map")
    })?;

    let mut defined_catalog_entries: HashMap<String, HashSet<String>> = HashMap::new();

    for (catalog_name_key, entries_val) in catalogs_map {
        let catalog_name_str = catalog_name_key
            .as_str()
            .ok_or_else(|| anyhow::anyhow!("Invalid catalog key (not a string)"))?
            .to_string();

        let entries_map = entries_val
            .as_hash()
            .ok_or_else(|| anyhow::anyhow!("Catalog '{}' is not a map", catalog_name_str))?;

        let mut entry_names = HashSet::new();
        for (key, _) in entries_map {
            if let Some(key_str) = key.as_str() {
                entry_names.insert(key_str.to_string());
            }
        }
        defined_catalog_entries.insert(catalog_name_str, entry_names);
    }

    // 2. Find all package.json files
    let pattern = "**/package.json";

    let mut package_json_paths: Vec<PathBuf> = Vec::new();

    for entry in glob(pattern).context("Failed to read glob pattern")? {
        match entry {
            Ok(path) => {
                // Filter out node_modules, dist, etc.
                if path.components().any(|c| {
                    c.as_os_str() == "node_modules"
                        || c.as_os_str() == "dist"
                        || c.as_os_str() == ".git"
                        || c.as_os_str() == "target"
                }) {
                    continue;
                }
                package_json_paths.push(path);
            }
            Err(e) => println!("Warning: Glob error: {:?}", e),
        }
    }

    // 3. Scan package.json files for USED entries
    let mut used_catalog_entries: HashMap<String, HashSet<String>> = HashMap::new();
    // Initialize sets for all known catalogs
    for name in defined_catalog_entries.keys() {
        used_catalog_entries.insert(name.clone(), HashSet::new());
    }

    for path in package_json_paths {
        // Read and parse package.json
        let content = match fs::read_to_string(&path) {
            Ok(c) => c,
            Err(e) => {
                eprintln!("Warning: Failed to read {}: {}", path.display(), e);
                continue;
            }
        };

        if content.trim().is_empty() {
            continue;
        }

        let pkg: serde_json::Value = match serde_json::from_str(&content) {
            Ok(v) => v,
            Err(e) => {
                eprintln!("Warning: Failed to parse {}: {}", path.display(), e);
                continue;
            }
        };

        // Check dependencies, devDependencies, etc.
        let dep_sections = [
            "dependencies",
            "devDependencies",
            "peerDependencies",
            "optionalDependencies",
        ];

        for section in dep_sections {
            if let Some(deps) = pkg.get(section).and_then(|d| d.as_object()) {
                for (dep_name, version_val) in deps {
                    if let Some(version) = version_val.as_str() {
                        if !version.starts_with("catalog:") {
                            continue;
                        }

                        // Format is "catalog:" (default) or "catalog:name"
                        let parts: Vec<&str> = version.split(':').collect();
                        let catalog_name = if parts.len() > 1 && !parts[1].is_empty() {
                            parts[1]
                        } else {
                            "default"
                        };

                        if let Some(used_set) = used_catalog_entries.get_mut(catalog_name) {
                            used_set.insert(dep_name.clone());
                        }
                    }
                }
            }
        }
    }

    // 4. Compare and Report
    let mut has_unused = false;
    for (catalog_name, defined_set) in &defined_catalog_entries {
        if let Some(used_set) = used_catalog_entries.get(catalog_name) {
            let unused: Vec<&String> = defined_set.difference(used_set).collect();

            if !unused.is_empty() {
                has_unused = true;
                println!(
                    "\nCatalog '{}' has {} unused entries:",
                    catalog_name,
                    unused.len()
                );
                for dep in unused {
                    println!("  - {}", dep);
                }
            }
        }
    }

    if has_unused {
        anyhow::bail!("Found unused catalog entries.");
    }

    Ok(())
}

/// Lints filenames against rules defined in `.ls-lint.toml`.
fn lint_filenames() -> Result<()> {
    let config = load_ls_config().context("Failed to load .ls-lint.toml")?;
    let files = get_tracked_files().context("Failed to get tracked files")?;

    let mut error_count = 0;

    for file in files {
        if is_ignored(&file, &config) {
            continue;
        }

        let errors = check_file(&file, &config)?;
        if !errors.is_empty() {
            error_count += 1;
            eprintln!("ERROR: {}", file);
            for err in errors {
                eprintln!("  - {}", err);
            }
        }
    }

    if error_count > 0 {
        anyhow::bail!("Found {} files with naming errors.", error_count);
    }

    println!("Filename linting passed.");
    Ok(())
}

/// Configuration for filename linting.
struct LsConfig {
    /// Mapping of filename extensions or glob patterns to naming rules.
    ls: HashMap<String, String>,
    /// List of file/directory patterns to ignore.
    ignore: Vec<String>,
    /// Pre-compiled glob set for fast ignore matching.
    ignore_globset: globset::GlobSet,
}

fn load_ls_config() -> Result<LsConfig> {
    let content = fs::read_to_string(".ls-lint.toml").context("Failed to read .ls-lint.toml")?;
    let value: toml::Value = toml::from_str(&content).context("Failed to parse .ls-lint.toml")?;

    let mut ls = HashMap::new();
    if let Some(ls_table) = value.get("ls").and_then(|v| v.as_table()) {
        for (k, v) in ls_table {
            if let Some(v_str) = v.as_str() {
                ls.insert(k.clone(), v_str.to_string());
            }
        }
    }

    let mut ignore = Vec::new();
    let mut builder = globset::GlobSetBuilder::new();
    if let Some(ignore_arr) = value.get("ignore").and_then(|v| v.as_array()) {
        for v in ignore_arr {
            if let Some(s) = v.as_str() {
                ignore.push(s.to_string());
                builder.add(globset::Glob::new(s)?);
            }
        }
    }

    Ok(LsConfig {
        ls,
        ignore,
        ignore_globset: builder.build()?,
    })
}

fn get_tracked_files() -> Result<Vec<String>> {
    let output = std::process::Command::new("git")
        .arg("ls-files")
        .output()
        .context("Failed to run git ls-files")?;

    if !output.status.success() {
        anyhow::bail!("git ls-files failed");
    }

    let stdout = String::from_utf8(output.stdout)?;
    Ok(stdout.lines().map(|s| s.to_string()).collect())
}

fn is_ignored(file_path: &str, config: &LsConfig) -> bool {
    if config.ignore_globset.is_match(file_path) {
        return true;
    }

    // Directory match: if the file starts with the pattern followed by a slash
    for pattern in &config.ignore {
        let clean = pattern.trim_end_matches('/');
        if file_path.starts_with(&format!("{}/", clean)) {
            return true;
        }
    }

    false
}

#[must_use]
fn validate_name(name: &str, rule: &str) -> bool {
    if let Some(pattern) = rule.strip_prefix("regex:") {
        return Regex::new(pattern)
            .map(|r| r.is_match(name))
            .unwrap_or(true);
    }

    let sub_rules: Vec<&str> = rule.split('|').map(|r| r.trim()).collect();
    if sub_rules.len() > 1 {
        return sub_rules.iter().any(|r| validate_name(name, r));
    }

    static RE_KEBAB: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"^[a-z0-9-.]+$").unwrap());
    static RE_SNAKE: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"^[a-z0-9_.]+$").unwrap());
    static RE_CAMEL: LazyLock<Regex> =
        LazyLock::new(|| Regex::new(r"^[a-z][a-zA-Z0-9.]*$").unwrap());
    static RE_PASCAL: LazyLock<Regex> =
        LazyLock::new(|| Regex::new(r"^[A-Z][a-zA-Z0-9.]*$").unwrap());
    static RE_SCREAMING: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"^[A-Z0-9_.]+$").unwrap());

    match rule {
        "kebab-case" => RE_KEBAB.is_match(name),
        "snake_case" => RE_SNAKE.is_match(name),
        "camelCase" => RE_CAMEL.is_match(name),
        "PascalCase" => RE_PASCAL.is_match(name),
        "SCREAMING_SNAKE_CASE" => RE_SCREAMING.is_match(name),
        _ => true,
    }
}

fn check_file(file_path: &str, config: &LsConfig) -> Result<Vec<String>> {
    let mut errors = Vec::new();
    let segments: Vec<&str> = file_path.split('/').collect();
    let filename = match segments.last() {
        Some(&f) => f,
        None => return Ok(errors),
    };

    let dir_segments = &segments[..segments.len() - 1];

    // Check directories
    if let Some(dir_rule) = config.ls.get(".dir") {
        for dir in dir_segments {
            if !validate_name(dir, dir_rule) {
                errors.push(format!(
                    "Directory \"{}\" does not match rules: {}",
                    dir, dir_rule
                ));
            }
        }
    }

    // Check filename
    let mut file_rule: Option<&String> = None;
    let mut name_to_check = filename.to_string();

    for (pattern, rule) in &config.ls {
        if pattern == ".dir" {
            continue;
        }

        if pattern.starts_with('.') {
            // Extension match: TS uses minimatch(filePath, `**/*${pattern}`)
            let glob = globset::Glob::new(&format!("**/*{}", pattern))?.compile_matcher();
            if glob.is_match(file_path) {
                file_rule = Some(rule);
                name_to_check = filename.trim_end_matches(pattern).to_string();
                break;
            }
        } else {
            // Glob match
            let glob1 = globset::Glob::new(pattern)?.compile_matcher();
            let glob2 = globset::Glob::new(&format!("**/{}", pattern))?.compile_matcher();
            if glob1.is_match(file_path) || glob2.is_match(file_path) {
                file_rule = Some(rule);
                name_to_check = filename.to_string();
                break;
            }
        }
    }

    if let Some(rule) = file_rule.filter(|r| !validate_name(&name_to_check, r)) {
        errors.push(format!(
            "File \"{}\" (stem: \"{}\") does not match rules: {}",
            filename, name_to_check, rule
        ));
    }

    Ok(errors)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validate_name() {
        assert!(validate_name("kebab-case", "kebab-case"));
        assert!(!validate_name("Snake_Case", "kebab-case"));

        assert!(validate_name("snake_case", "snake_case"));
        assert!(!validate_name("CamelCase", "snake_case"));

        assert!(validate_name("camelCase", "camelCase"));
        assert!(!validate_name("PascalCase", "camelCase"));

        assert!(validate_name("PascalCase", "PascalCase"));
        assert!(!validate_name("kebab-case", "PascalCase"));

        assert!(validate_name("SCREAMING_SNAKE", "SCREAMING_SNAKE_CASE"));

        // Multi-rule
        assert!(validate_name("kebab-case", "kebab-case | snake_case"));
        assert!(validate_name("snake_case", "kebab-case | snake_case"));
        assert!(!validate_name("PascalCase", "kebab-case | snake_case"));

        // Regex
        assert!(validate_name("use-js-instead", "regex:use-js-instead"));
    }

    #[test]
    fn test_check_file() -> Result<()> {
        let mut ls = HashMap::new();
        ls.insert(".dir".to_string(), "kebab-case".to_string());
        ls.insert(
            ".md".to_string(),
            "kebab-case | SCREAMING_SNAKE_CASE".to_string(),
        );
        ls.insert(".ts".to_string(), "kebab-case".to_string());

        let config = LsConfig {
            ls,
            ignore: vec![],
            ignore_globset: globset::GlobSetBuilder::new().build()?,
        };

        // Valid path (stem "AGENTS" matches SCREAMING_SNAKE_CASE)
        assert!(check_file("AGENTS.md", &config)?.is_empty());

        // Valid path (stem "my-file" matches kebab-case)
        assert!(check_file("src/my-file.ts", &config)?.is_empty());

        // Invalid directory
        let errors = check_file("MyPkg/src/index.ts", &config)?;
        assert_eq!(errors.len(), 1);
        assert!(errors[0].contains("Directory \"MyPkg\" does not match rules: kebab-case"));

        // Invalid filename (stem "Index" does not match kebab-case)
        let errors = check_file("src/Index.ts", &config)?;
        assert_eq!(errors.len(), 1);
        assert!(
            errors[0]
                .contains("File \"Index.ts\" (stem: \"Index\") does not match rules: kebab-case")
        );

        Ok(())
    }
}
