use anyhow::{Context, Result};
use glob::glob;
use std::collections::{HashMap, HashSet};
use std::fs;
use std::path::PathBuf;

/// Checks that all entries defined in `pnpm-workspace.yaml` catalogs are
/// actually used by some `package.json` in the workspace. Fails if any
/// entries are unused.
pub(crate) fn check_catalog() -> Result<()> {
    let defined_catalog_entries = get_defined_catalog_entries()?;
    let package_json_paths = get_package_json_paths()?;

    let catalog_names: HashSet<String> = defined_catalog_entries.keys().cloned().collect();
    let used_catalog_entries =
        get_used_catalog_entries_from_packages(package_json_paths, &catalog_names);

    report_and_fail_on_unused(&defined_catalog_entries, &used_catalog_entries)
}

fn get_defined_catalog_entries() -> Result<HashMap<String, HashSet<String>>> {
    let root_dir = std::env::current_dir()?;
    let workspace_path = root_dir.join("pnpm-workspace.yaml");

    if !workspace_path.exists() {
        anyhow::bail!(
            "pnpm-workspace.yaml not found at {}",
            workspace_path.display()
        );
    }

    let workspace_content =
        fs::read_to_string(&workspace_path).context("Failed to read pnpm-workspace.yaml")?;

    let docs = yaml_rust2::YamlLoader::load_from_str(&workspace_content)
        .context("Failed to parse pnpm-workspace.yaml")?;

    if docs.is_empty() {
        anyhow::bail!("pnpm-workspace.yaml is empty");
    }

    let workspace = &docs[0];
    let catalogs = &workspace["catalogs"];

    let catalogs_map = catalogs.as_hash().ok_or_else(|| {
        anyhow::anyhow!("No 'catalogs' section found in pnpm-workspace.yaml useable as a map")
    })?;

    let mut defined_catalog_entries = HashMap::new();

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

    Ok(defined_catalog_entries)
}

fn get_package_json_paths() -> Result<Vec<PathBuf>> {
    let pattern = "**/package.json";
    let mut package_json_paths = Vec::new();

    for entry in glob(pattern).context("Failed to read glob pattern")? {
        match entry {
            Ok(path) => {
                if path.components().any(|c| {
                    let s = c.as_os_str();
                    s == "node_modules" || s == "dist" || s == ".git" || s == "target"
                }) {
                    continue;
                }
                package_json_paths.push(path);
            }
            Err(e) => println!("Warning: Glob error: {:?}", e),
        }
    }

    Ok(package_json_paths)
}

fn get_used_catalog_entries_from_packages(
    paths: Vec<PathBuf>,
    catalog_names: &HashSet<String>,
) -> HashMap<String, HashSet<String>> {
    let mut used_catalog_entries = HashMap::new();
    for name in catalog_names {
        used_catalog_entries.insert(name.clone(), HashSet::new());
    }

    for path in paths {
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

    used_catalog_entries
}

fn report_and_fail_on_unused(
    defined: &HashMap<String, HashSet<String>>,
    used: &HashMap<String, HashSet<String>>,
) -> Result<()> {
    let mut has_unused = false;
    for (catalog_name, defined_set) in defined {
        if let Some(used_set) = used.get(catalog_name) {
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
