use std::fs;

use anyhow::{Context, Result, bail};
use regex::Regex;
use toml_edit::{DocumentMut, Item, Value};

/// Dependency source URL expected for dioxus-primitives pinning.
const DIOXUS_COMPONENTS_GIT: &str = "https://github.com/DioxusLabs/components";
/// Workspace-relative manifest path for the UI crate.
const UI_CARGO_TOML_PATH: &str = "crates/tasklens-ui/Cargo.toml";
/// Workspace lockfile path.
const CARGO_LOCK_PATH: &str = "Cargo.lock";

/// Verifies that the pinned dioxus-primitives revision in Cargo.toml matches
/// the resolved dioxus-primitives revision in Cargo.lock.
///
/// This is used by `check-all` to block commits/pushes when the dependency pin
/// and lockfile drift apart.
pub(crate) fn check_dioxus_lock_pin() -> Result<()> {
    let repo_root = std::env::current_dir().context("failed to resolve current directory")?;

    let manifest_content = fs::read_to_string(repo_root.join(UI_CARGO_TOML_PATH))
        .with_context(|| format!("failed to read {UI_CARGO_TOML_PATH}"))?;
    let lock_content = fs::read_to_string(repo_root.join(CARGO_LOCK_PATH))
        .with_context(|| format!("failed to read {CARGO_LOCK_PATH}"))?;

    let (manifest_git, manifest_rev) = extract_manifest_dioxus_primitives_pin(&manifest_content)?;
    let lock_rev = extract_lock_dioxus_primitives_rev(&lock_content)?;

    if manifest_git != DIOXUS_COMPONENTS_GIT {
        bail!(
            "dioxus-primitives git pin mismatch in {UI_CARGO_TOML_PATH}: expected '{DIOXUS_COMPONENTS_GIT}', found '{manifest_git}'"
        );
    }

    if manifest_rev != lock_rev {
        bail!(
            "dioxus-primitives rev mismatch: manifest has '{manifest_rev}', lockfile has '{lock_rev}'. Run `cargo update -p dioxus-primitives --precise {manifest_rev}`."
        );
    }

    Ok(())
}

/// Parses the dioxus-primitives git+rev pin from the tasklens-ui Cargo.toml content.
fn extract_manifest_dioxus_primitives_pin(content: &str) -> Result<(String, String)> {
    let document = content
        .parse::<DocumentMut>()
        .context("failed to parse tasklens-ui Cargo.toml")?;

    let dependencies = document
        .get("dependencies")
        .and_then(Item::as_table)
        .ok_or_else(|| anyhow::anyhow!("[dependencies] is missing in tasklens-ui Cargo.toml"))?;

    let dioxus_primitives_item = dependencies
        .get("dioxus-primitives")
        .ok_or_else(|| anyhow::anyhow!("dependencies.dioxus-primitives is missing"))?;

    let table_like = dioxus_primitives_item.as_table_like().ok_or_else(|| {
        anyhow::anyhow!("dependencies.dioxus-primitives must be a table or inline table")
    })?;

    let git = table_like
        .get("git")
        .and_then(Item::as_value)
        .and_then(Value::as_str)
        .ok_or_else(|| anyhow::anyhow!("dependencies.dioxus-primitives.git is missing"))?
        .to_string();

    let rev = table_like
        .get("rev")
        .and_then(Item::as_value)
        .and_then(Value::as_str)
        .ok_or_else(|| anyhow::anyhow!("dependencies.dioxus-primitives.rev is missing"))?
        .to_string();

    Ok((git, rev))
}

/// Parses the dioxus-primitives resolved git revision from Cargo.lock content.
fn extract_lock_dioxus_primitives_rev(content: &str) -> Result<String> {
    let pattern = Regex::new(
        r#"(?ms)name = \"dioxus-primitives\"\s+version = \"[^\"]+\"\s+source = \"git\+https://github\.com/DioxusLabs/components(?:\?[^\"]*)?#([0-9a-fA-F]+)\""#,
    )
    .expect("valid regex");

    let captures = pattern.captures(content).ok_or_else(|| {
        anyhow::anyhow!("could not find dioxus-primitives git source entry in Cargo.lock")
    })?;

    let rev = captures
        .get(1)
        .map(|capture| capture.as_str().to_string())
        .ok_or_else(|| {
            anyhow::anyhow!("missing dioxus-primitives revision capture in Cargo.lock")
        })?;

    Ok(rev)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_manifest_pin() {
        let input = r#"
[dependencies]
dioxus-primitives = { git = "https://github.com/DioxusLabs/components", rev = "deadbeef", version = "0.0.1", default-features = false, features = ["router"] }
"#;

        let (git, rev) = extract_manifest_dioxus_primitives_pin(input).unwrap();
        assert_eq!(git, "https://github.com/DioxusLabs/components");
        assert_eq!(rev, "deadbeef");
    }

    #[test]
    fn fails_when_manifest_rev_missing() {
        let input = r#"
[dependencies]
dioxus-primitives = { git = "https://github.com/DioxusLabs/components" }
"#;

        assert!(extract_manifest_dioxus_primitives_pin(input).is_err());
    }

    #[test]
    fn parses_lock_rev_without_query() {
        let input = r#"
[[package]]
name = "dioxus-primitives"
version = "0.0.1"
source = "git+https://github.com/DioxusLabs/components#deadbeef"
"#;

        let rev = extract_lock_dioxus_primitives_rev(input).unwrap();
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

        let rev = extract_lock_dioxus_primitives_rev(input).unwrap();
        assert_eq!(rev, "deadbeef");
    }
}
