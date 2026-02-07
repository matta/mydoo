use anyhow::{Context, Result};
use regex::Regex;
use serde_json::Value;
use std::fs;
use std::path::Path;
use std::process::Command;

/// Checks that the biome.json $schema version matches the installed biome CLI version.
/// Exits with code 1 if there's a mismatch, making CI fail when biome is upgraded
/// but the config isn't updated.
pub(crate) fn check_biome_schema() -> Result<()> {
    let root_dir = std::env::current_dir().context("Failed to get current directory")?;
    let biome_config_path = root_dir.join("biome.json");

    if !biome_config_path.exists() {
        // If biome.json doesn't exist, we might not be using biome or it's not at the root.
        // For now, let's assume it should exist if we are running this check.
        anyhow::bail!("biome.json not found at {}", biome_config_path.display());
    }

    let installed_version = get_installed_biome_version(&root_dir)?;
    let schema_version = get_schema_version(&biome_config_path)?;

    let relative_config_path = biome_config_path
        .strip_prefix(&root_dir)
        .unwrap_or(&biome_config_path);

    if installed_version != schema_version {
        anyhow::bail!(
            "ERROR [check-biome-schema]: {} schema version {} does not match installed CLI version {}. Run 'pnpm biome migrate' to fix.",
            relative_config_path.display(),
            schema_version,
            installed_version
        );
    }

    Ok(())
}

fn get_installed_biome_version(root_dir: &Path) -> Result<String> {
    let output = Command::new("pnpm")
        .args(["list", "@biomejs/biome", "--json"])
        .current_dir(root_dir)
        .output()
        .context("Failed to execute pnpm list")?;

    if !output.status.success() {
        anyhow::bail!(
            "pnpm list failed: {}",
            String::from_utf8_lossy(&output.stderr)
        );
    }

    let stdout = String::from_utf8(output.stdout)?;
    let parsed: Value = serde_json::from_str(&stdout)?;

    // pnpm list --json returns an array of objects
    let entries = parsed
        .as_array()
        .context("pnpm list output is not an array")?;
    if entries.is_empty() {
        anyhow::bail!("pnpm list output is empty");
    }
    let entry = &entries[0];

    let version = entry["devDependencies"]["@biomejs/biome"]["version"]
        .as_str()
        .or_else(|| entry["dependencies"]["@biomejs/biome"]["version"].as_str())
        .context("Could not find @biomejs/biome version in pnpm list output. (Note: if it is in catalog, this check might need updating to look at pnpm-lock.yaml or similar)")?;

    Ok(version.to_string())
}

fn get_schema_version(biome_config_path: &Path) -> Result<String> {
    let content = fs::read_to_string(biome_config_path).context("Failed to read biome.json")?;
    let config: Value = serde_json::from_str(&content).context("Failed to parse biome.json")?;

    let schema = config["$schema"]
        .as_str()
        .context("biome.json does not have a $schema field or it is not a string")?;

    let re = Regex::new(r"/schemas/([^/]+)/schema\.json$")?;
    let caps = re
        .captures(schema)
        .context("Could not extract version from biome.json $schema")?;

    Ok(caps.get(1).unwrap().as_str().to_string())
}
