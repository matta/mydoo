use anyhow::{Result, anyhow};
use regex::Regex;

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

#[cfg(test)]
mod tests {
    use super::*;

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
