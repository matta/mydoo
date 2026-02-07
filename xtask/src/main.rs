use anyhow::{Context, Result};
use clap::{Parser, Subcommand};
use glob::glob;
use std::collections::{HashMap, HashSet};
use std::fs;
use std::path::PathBuf;

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
    /// Run all checks
    CheckAll,
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    match &cli.command {
        Commands::CheckCatalog => check_catalog()?,
        Commands::CheckAll => {
            check_catalog()?;
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
