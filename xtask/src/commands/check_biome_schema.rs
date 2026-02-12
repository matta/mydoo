use anyhow::{Context, Result};
use regex::Regex;
use semver::{Version, VersionReq};
use serde_json::Value;
use std::fs;
use std::path::Path;
use std::process::Command;
use yaml_rust2::YamlLoader;

/// Checks that:
/// 1. The installed biome version matches the version requirement in `pnpm-workspace.yaml`.
/// 2. The locked biome version (in `pnpm-lock.yaml`) matches the version requirement in `pnpm-workspace.yaml`.
/// 3. The `biome.json` $schema version matches the installed/locked biome version.
///
/// Exits with code 1 if any of these checks fail, ensuring that `pnpm install`, `pnpm-lock.yaml`,
/// and `biome.json` are all in sync.
pub(crate) fn check_biome_schema() -> Result<()> {
    let root_dir = std::env::current_dir().context("Failed to get current directory")?;
    let biome_config_path = root_dir.join("biome.json");

    if !biome_config_path.exists() {
        // If biome.json doesn't exist, we might not be using biome or it's not at the root.
        // For now, let's assume it should exist if we are running this check.
        anyhow::bail!("biome.json not found at {}", biome_config_path.display());
    }

    let configured_req = get_configured_biome_requirement(&root_dir)?;
    let installed_version = get_installed_biome_version(&root_dir)?;
    let locked_version = get_locked_biome_version(&root_dir)?;
    let schema_version_str = get_schema_version(&biome_config_path)?;
    let schema_version = Version::parse(&schema_version_str).context(format!(
        "Failed to parse schema version '{}'",
        schema_version_str
    ))?;

    // Check 1: Installed version matches pnpm-workspace.yaml requirement
    if !configured_req.matches(&installed_version) {
        anyhow::bail!(
            "[check-biome-schema] ERROR: Installed biome version {} does not satisfy configured version requirement {} (from pnpm-workspace.yaml). Please run 'pnpm install'.",
            installed_version,
            configured_req
        );
    }

    // Check 2: Locked version matches pnpm-workspace.yaml requirement
    if !configured_req.matches(&locked_version) {
        anyhow::bail!(
            "[check-biome-schema] ERROR: Locked biome version {} does not satisfy configured version requirement {} (from pnpm-workspace.yaml). Please run 'pnpm install' to update the lockfile.",
            locked_version,
            configured_req
        );
    }

    // Check 3: Installed version matches locked version
    if installed_version != locked_version {
        anyhow::bail!(
            "[check-biome-schema] ERROR: Installed biome version {} does not match locked version {}. Please run 'pnpm install'.",
            installed_version,
            locked_version
        );
    }

    // Check 4: Schema version matches installed/locked version
    if installed_version != schema_version {
        anyhow::bail!(
            "[check-biome-schema] ERROR: biome.json schema version {} does not match installed CLI version {}. Run 'pnpm biome migrate' to fix.",
            schema_version,
            installed_version
        );
    }

    Ok(())
}

fn get_configured_biome_requirement(root_dir: &Path) -> Result<VersionReq> {
    let workspace_path = root_dir.join("pnpm-workspace.yaml");
    let content = fs::read_to_string(&workspace_path)
        .context(format!("Failed to read {}", workspace_path.display()))?;

    parse_workspace_requirement(&content)
}

fn parse_workspace_requirement(content: &str) -> Result<VersionReq> {
    let docs = YamlLoader::load_from_str(content)?;
    if docs.is_empty() {
        anyhow::bail!("pnpm-workspace.yaml is empty");
    }
    let doc = &docs[0];

    // pnpm catalogs are defined in catalogs -> default -> package
    let version_str = doc["catalogs"]["default"]["@biomejs/biome"]
        .as_str()
        .context(
            "Could not find @biomejs/biome version in pnpm-workspace.yaml (under catalogs.default)",
        )?;

    VersionReq::parse(version_str).context(format!(
        "Failed to parse biome version requirement '{}'",
        version_str
    ))
}

fn get_installed_biome_version(root_dir: &Path) -> Result<Version> {
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
    parse_pnpm_list_output(&stdout)
}

fn get_locked_biome_version(root_dir: &Path) -> Result<Version> {
    let output = Command::new("pnpm")
        .args(["list", "@biomejs/biome", "--json", "--lockfile-only"])
        .current_dir(root_dir)
        .output()
        .context("Failed to execute pnpm list --lockfile-only")?;

    if !output.status.success() {
        anyhow::bail!(
            "pnpm list --lockfile-only failed: {}",
            String::from_utf8_lossy(&output.stderr)
        );
    }

    let stdout = String::from_utf8(output.stdout)?;
    parse_pnpm_list_output(&stdout)
}

fn parse_pnpm_list_output(stdout: &str) -> Result<Version> {
    let parsed: Value = serde_json::from_str(stdout)?;

    // pnpm list --json returns an array of objects
    let entries = parsed
        .as_array()
        .context("pnpm list output is not an array")?;
    if entries.is_empty() {
        anyhow::bail!("pnpm list output is empty");
    }
    let entry = &entries[0];

    let version_str = entry["devDependencies"]["@biomejs/biome"]["version"]
        .as_str()
        .or_else(|| entry["dependencies"]["@biomejs/biome"]["version"].as_str())
        .context("Could not find @biomejs/biome version in pnpm list output.")?;

    Version::parse(version_str).context(format!("Failed to parse biome version '{}'", version_str))
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_workspace_requirement() {
        let content = r#"
packages:
  - apps/*
catalogs:
  default:
    "@biomejs/biome": ^1.5.0
"#;
        let req = parse_workspace_requirement(content).unwrap();
        assert!(req.matches(&Version::parse("1.5.0").unwrap()));
        assert!(req.matches(&Version::parse("1.5.3").unwrap()));
        assert!(!req.matches(&Version::parse("1.4.0").unwrap()));
        assert!(!req.matches(&Version::parse("2.0.0").unwrap()));
    }

    #[test]
    fn test_parse_pnpm_list_output_dev_dep() {
        let output = r#"[
  {
    "name": "my-project",
    "version": "1.0.0",
    "path": "/path/to/project",
    "private": true,
    "devDependencies": {
      "@biomejs/biome": {
        "from": "@biomejs/biome",
        "version": "1.5.3",
        "resolved": "https://registry.npmjs.org/@biomejs/biome/-/biome-1.5.3.tgz"
      }
    }
  }
]"#;
        let version = parse_pnpm_list_output(output).unwrap();
        assert_eq!(version, Version::parse("1.5.3").unwrap());
    }

    #[test]
    fn test_parse_pnpm_list_output_prod_dep() {
        let output = r#"[
  {
    "name": "my-project",
    "version": "1.0.0",
    "dependencies": {
      "@biomejs/biome": {
        "version": "1.6.0"
      }
    }
  }
]"#;
        let version = parse_pnpm_list_output(output).unwrap();
        assert_eq!(version, Version::parse("1.6.0").unwrap());
    }
}
