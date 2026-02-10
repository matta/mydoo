use std::fs;
use std::path::{Component, Path};

use anyhow::{Context, Result, anyhow, bail};
use regex::Regex;
use toml_edit::{DocumentMut, Item, Value};

/// Default workspace path for Dioxus vendor component configuration.
pub(crate) const DEFAULT_DIOXUS_VENDOR_COMPONENTS_FILE: &str =
    "crates/tasklens-ui/dioxus-vendor-components.toml";

/// Registry pin data read from `dioxus-vendor-components.toml`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct DioxusVendorRegistryConfig {
    pub(crate) git: String,
    pub(crate) rev: String,
}

/// Vendor component configuration used by Dioxus installer automation.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct DioxusVendorComponentsConfig {
    pub(crate) registry: DioxusVendorRegistryConfig,
    pub(crate) module_path: String,
    pub(crate) components: Vec<String>,
}

/// Parses the dioxus-primitives git+rev pin from tasklens-ui Cargo.toml content.
pub(crate) fn extract_manifest_dioxus_primitives_pin_from_content(
    content: &str,
) -> Result<DioxusVendorRegistryConfig> {
    let document = content
        .parse::<DocumentMut>()
        .context("failed to parse tasklens-ui Cargo.toml")?;

    let dependencies = document
        .get("dependencies")
        .and_then(Item::as_table)
        .ok_or_else(|| anyhow!("[dependencies] is missing in tasklens-ui Cargo.toml"))?;

    let dioxus_primitives_item = dependencies
        .get("dioxus-primitives")
        .ok_or_else(|| anyhow!("dependencies.dioxus-primitives is missing"))?;

    let table_like = dioxus_primitives_item
        .as_table_like()
        .ok_or_else(|| anyhow!("dependencies.dioxus-primitives must be a table or inline table"))?;

    let git = table_like
        .get("git")
        .and_then(Item::as_value)
        .and_then(Value::as_str)
        .ok_or_else(|| anyhow!("dependencies.dioxus-primitives.git is missing"))?
        .to_string();

    let rev = table_like
        .get("rev")
        .and_then(Item::as_value)
        .and_then(Value::as_str)
        .ok_or_else(|| anyhow!("dependencies.dioxus-primitives.rev is missing"))?
        .to_string();

    Ok(DioxusVendorRegistryConfig { git, rev })
}

/// Reads and parses the Dioxus vendor components configuration file.
pub(crate) fn read_dioxus_vendor_components_config(
    components_file: &Path,
) -> Result<DioxusVendorComponentsConfig> {
    let content = fs::read_to_string(components_file)
        .with_context(|| format!("failed to read {}", components_file.display()))?;
    parse_dioxus_vendor_components_config_content(&content).with_context(|| {
        format!(
            "failed to parse components file {}; expected `module_path = \"src/...\"`, `[registry].git`, `[registry].rev`, and `components = [\"name\", ...]`",
            components_file.display()
        )
    })
}

/// Validates that a configured module path is relative and safely scoped under `src/`.
pub(crate) fn validate_dioxus_vendor_module_path(module_path: &str) -> Result<()> {
    if module_path.trim().is_empty() {
        bail!("`module_path` cannot be empty");
    }

    let path = Path::new(module_path);
    if path.is_absolute() {
        bail!("`module_path` must be a relative path (example: `src/dioxus_components`)");
    }

    if !path.starts_with("src") || path == Path::new("src") {
        bail!("`module_path` must point to a subdirectory under `src/`");
    }

    for component in path.components() {
        match component {
            Component::Normal(_) => {}
            _ => bail!(
                "`module_path` must contain only normal path segments (example: `src/dioxus_components`)"
            ),
        }
    }

    Ok(())
}

/// Parses TOML content expected to contain `module_path`, `[registry]`, and `components`.
pub(crate) fn parse_dioxus_vendor_components_config_content(
    content: &str,
) -> Result<DioxusVendorComponentsConfig> {
    let document = content
        .parse::<DocumentMut>()
        .context("components file is not valid TOML")?;

    let module_path = document
        .get("module_path")
        .ok_or_else(|| anyhow!("missing `module_path` string"))?
        .as_str()
        .ok_or_else(|| anyhow!("`module_path` must be a string"))?
        .trim()
        .to_string();
    validate_dioxus_vendor_module_path(&module_path)?;

    let registry = document
        .get("registry")
        .ok_or_else(|| anyhow!("missing `registry` table"))?
        .as_table_like()
        .ok_or_else(|| anyhow!("`registry` must be a table"))?;
    let registry_git = registry
        .get("git")
        .and_then(Item::as_value)
        .and_then(Value::as_str)
        .ok_or_else(|| anyhow!("`registry.git` must be a string"))?
        .trim()
        .to_string();
    validate_registry_git(&registry_git)?;
    let registry_rev = registry
        .get("rev")
        .and_then(Item::as_value)
        .and_then(Value::as_str)
        .ok_or_else(|| anyhow!("`registry.rev` must be a string"))?
        .trim()
        .to_string();
    validate_registry_rev(&registry_rev)?;

    let components = document
        .get("components")
        .ok_or_else(|| anyhow!("missing `components` array"))?
        .as_array()
        .ok_or_else(|| anyhow!("`components` must be an array"))?;

    let mut parsed_components = Vec::new();
    for item in components {
        let component = item
            .as_str()
            .ok_or_else(|| anyhow!("all `components` entries must be strings"))?;
        if component.trim().is_empty() {
            bail!("component names cannot be empty strings");
        }
        parsed_components.push(component.to_string());
    }

    Ok(DioxusVendorComponentsConfig {
        registry: DioxusVendorRegistryConfig {
            git: registry_git,
            rev: registry_rev,
        },
        module_path,
        components: parsed_components,
    })
}

/// Extracts the resolved `dioxus-primitives` git revision from Cargo.lock
/// content, supporting both URL styles used by Cargo:
/// - `.../components#<rev>`
/// - `.../components?rev=<rev>#<rev>`
pub(crate) fn extract_dioxus_primitives_rev_from_lock_content(
    lock_content: &str,
) -> Result<String> {
    let pattern = Regex::new(
        r#"(?ms)name = \"dioxus-primitives\"\s+version = \"[^\"]+\"\s+source = \"git\+https://github\.com/DioxusLabs/components(?:\?[^\"]*)?#([0-9a-fA-F]+)\""#,
    )
    .expect("valid regex");

    let captures = pattern.captures(lock_content).ok_or_else(|| {
        anyhow!("could not find dioxus-primitives git source entry in Cargo.lock")
    })?;

    let revision = captures
        .get(1)
        .map(|capture| capture.as_str().to_string())
        .ok_or_else(|| anyhow!("missing dioxus-primitives revision capture in Cargo.lock"))?;

    Ok(revision)
}

/// Validates that a registry git URL is non-empty.
fn validate_registry_git(registry_git: &str) -> Result<()> {
    if registry_git.trim().is_empty() {
        bail!("`registry.git` cannot be empty");
    }
    Ok(())
}

/// Validates that a registry revision string is non-empty.
fn validate_registry_rev(registry_rev: &str) -> Result<()> {
    if registry_rev.trim().is_empty() {
        bail!("`registry.rev` cannot be empty");
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_vendor_components_config() {
        let input = r#"
module_path = "src/dioxus_components"
components = ["button", "dialog"]
[registry]
git = "https://github.com/DioxusLabs/components"
rev = "abc123"
"#;

        let parsed = parse_dioxus_vendor_components_config_content(input).unwrap();
        assert_eq!(parsed.module_path, "src/dioxus_components");
        assert_eq!(
            parsed.registry.git,
            "https://github.com/DioxusLabs/components"
        );
        assert_eq!(parsed.registry.rev, "abc123");
        assert_eq!(parsed.components, vec!["button", "dialog"]);
    }

    #[test]
    fn parses_manifest_dioxus_primitives_pin() {
        let input = r#"
[dependencies]
dioxus-primitives = { git = "https://github.com/DioxusLabs/components", rev = "deadbeef", version = "0.0.1", default-features = false, features = ["router"] }
"#;

        let parsed = extract_manifest_dioxus_primitives_pin_from_content(input).unwrap();
        assert_eq!(parsed.git, "https://github.com/DioxusLabs/components");
        assert_eq!(parsed.rev, "deadbeef");
    }

    #[test]
    fn fails_when_manifest_rev_missing() {
        let input = r#"
[dependencies]
dioxus-primitives = { git = "https://github.com/DioxusLabs/components" }
"#;

        assert!(extract_manifest_dioxus_primitives_pin_from_content(input).is_err());
    }

    #[test]
    fn rejects_vendor_config_without_registry() {
        let input = r#"
module_path = "src/dioxus_components"
components = ["button"]
"#;

        assert!(parse_dioxus_vendor_components_config_content(input).is_err());
    }

    #[test]
    fn rejects_vendor_config_with_unsafe_module_path() {
        let input = r#"
module_path = "../outside"
components = ["button"]
[registry]
git = "https://github.com/DioxusLabs/components"
rev = "abc123"
"#;

        assert!(parse_dioxus_vendor_components_config_content(input).is_err());
    }

    #[test]
    fn parses_lock_rev_without_query() {
        let input = r#"
[[package]]
name = "dioxus-primitives"
version = "0.0.1"
source = "git+https://github.com/DioxusLabs/components#deadbeef"
"#;

        let rev = extract_dioxus_primitives_rev_from_lock_content(input).unwrap();
        assert_eq!(rev, "deadbeef");
    }

    #[test]
    fn parses_lock_rev_with_query() {
        let input = r#"
[[package]]
name = "dioxus-primitives"
version = "0.0.1"
source = "git+https://github.com/DioxusLabs/components?rev=deadbeef#deadbeef"
"#;

        let rev = extract_dioxus_primitives_rev_from_lock_content(input).unwrap();
        assert_eq!(rev, "deadbeef");
    }
}
