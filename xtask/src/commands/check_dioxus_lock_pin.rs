use std::fs;

use crate::commands::dioxus_info::{
    extract_dioxus_primitives_rev_from_lock_content,
    extract_manifest_dioxus_primitives_pin_from_content,
};
use anyhow::{Context, Result, bail};

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

    let manifest_pin = extract_manifest_dioxus_primitives_pin_from_content(&manifest_content)?;
    let lock_rev = extract_dioxus_primitives_rev_from_lock_content(&lock_content)?;

    if manifest_pin.git != DIOXUS_COMPONENTS_GIT {
        bail!(
            "dioxus-primitives git pin mismatch in {UI_CARGO_TOML_PATH}: expected '{DIOXUS_COMPONENTS_GIT}', found '{}'",
            manifest_pin.git
        );
    }

    if manifest_pin.rev != lock_rev {
        bail!(
            "dioxus-primitives rev mismatch: manifest has '{}', lockfile has '{lock_rev}'. Run `cargo update -p dioxus-primitives --precise {}`.",
            manifest_pin.rev,
            manifest_pin.rev
        );
    }

    Ok(())
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

        let pin = extract_manifest_dioxus_primitives_pin_from_content(input).unwrap();
        assert_eq!(pin.git, "https://github.com/DioxusLabs/components");
        assert_eq!(pin.rev, "deadbeef");
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
