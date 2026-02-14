// Derived in part from Dioxus upstream registry/cache resolution code:
// - context/dioxus/packages/cli/src/cli/component.rs
// - context/dioxus/packages/cli/src/workspace.rs
// Upstream package metadata:
// - authors = ["Jonathan Kelley"]
// - license = "MIT OR Apache-2.0"
// See `UPSTREAM_ATTRIBUTION.md` in this directory for provenance and licensing details.

use std::collections::hash_map::DefaultHasher;
use std::fs;
use std::hash::{Hash, Hasher};
use std::path::{Path, PathBuf};
use std::process::Command;

use anyhow::{Context, Result, bail};

const DX_COMPONENTS_CACHE_DIR: &str = "x-xtask-dx-components";

use crate::commands::dioxus_info::{
    DioxusVendorRegistryConfig, read_dioxus_vendor_components_config,
};

use super::installer;

/// Immutable git+rev selector for a component registry checkout.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub(crate) struct RegistrySpec {
    pub(crate) git: String,
    pub(crate) rev: String,
}

impl RegistrySpec {
    /// Builds a `RegistrySpec` from parsed vendor registry config.
    pub(crate) fn from_vendor_registry(registry: &DioxusVendorRegistryConfig) -> Self {
        Self {
            git: registry.git.clone(),
            rev: registry.rev.clone(),
        }
    }
}

/// Lists components available in the configured pinned registry.
pub(crate) fn list_components(components_file: &Path) -> Result<()> {
    let repo_root = find_repo_root()?;
    let components_file_relative = normalize_repo_relative_path(&repo_root, components_file)?;
    let components_config =
        read_dioxus_vendor_components_config(&repo_root.join(&components_file_relative))?;
    let registry = RegistrySpec::from_vendor_registry(&components_config.registry);
    let registry_root = resolve_registry_checkout(&repo_root, &registry, false)?;
    let mut components = installer::discover_registry_components(&registry_root, &registry)?;
    components.sort_by(|left, right| left.name.cmp(&right.name));

    for component in components {
        println!("- {}: {}", component.name, component.description);
    }

    Ok(())
}

/// Refreshes the configured pinned registry checkout in local cache.
pub(crate) fn update_registry(components_file: &Path) -> Result<()> {
    let repo_root = find_repo_root()?;
    let components_file_relative = normalize_repo_relative_path(&repo_root, components_file)?;
    let components_config =
        read_dioxus_vendor_components_config(&repo_root.join(&components_file_relative))?;
    let registry = RegistrySpec::from_vendor_registry(&components_config.registry);
    let path = resolve_registry_checkout(&repo_root, &registry, true)?;

    println!(
        "updated registry cache: git={}, rev={}, path={}",
        registry.git,
        registry.rev,
        path.display()
    );

    Ok(())
}

/// Removes local dx-components cache under the repository git common-dir.
pub(crate) fn clean_registry_cache() -> Result<()> {
    let repo_root = find_repo_root()?;
    let cache_root = registry_cache_root_for_repo(&repo_root)?;
    let dx_components_root = cache_root.parent().map(Path::to_path_buf).ok_or_else(|| {
        anyhow::anyhow!("failed to resolve {} cache parent", DX_COMPONENTS_CACHE_DIR)
    })?;

    if !dx_components_root.exists() {
        println!(
            "{} cache is already clean ({})",
            DX_COMPONENTS_CACHE_DIR,
            dx_components_root.display()
        );
        return Ok(());
    }

    fs::remove_dir_all(&dx_components_root).with_context(|| {
        format!(
            "failed to remove {} cache root {}",
            DX_COMPONENTS_CACHE_DIR,
            dx_components_root.display()
        )
    })?;

    println!(
        "removed {} cache root {}",
        DX_COMPONENTS_CACHE_DIR,
        dx_components_root.display()
    );
    Ok(())
}

/// Resolves and materializes a registry checkout in the git-common-dir cache.
pub(crate) fn resolve_registry_checkout(
    repo_root: &Path,
    registry: &RegistrySpec,
    refresh: bool,
) -> Result<PathBuf> {
    let repo_dir = component_cache_path(repo_root, registry)?;
    let parent = repo_dir
        .parent()
        .ok_or_else(|| anyhow::anyhow!("failed to resolve registry cache parent directory"))?;
    fs::create_dir_all(parent).with_context(|| {
        format!(
            "failed to create registry cache parent {}",
            parent.display()
        )
    })?;

    if !repo_dir.exists() {
        run_command_checked(
            repo_root,
            "git",
            &[
                "clone",
                registry.git.as_str(),
                path_to_string(&repo_dir).as_str(),
            ],
        )
        .with_context(|| {
            format!(
                "failed to clone component registry '{}' into {}",
                registry.git,
                repo_dir.display()
            )
        })?;
    } else if refresh {
        run_command_checked(&repo_dir, "git", &["fetch", "origin", "--prune", "--tags"])
            .with_context(|| {
                format!(
                    "failed to refresh component registry checkout at {}",
                    repo_dir.display()
                )
            })?;
    }

    run_command_checked(
        &repo_dir,
        "git",
        &["checkout", "--force", registry.rev.as_str()],
    )
    .with_context(|| {
        format!(
            "failed to checkout rev '{}' in registry {}",
            registry.rev,
            repo_dir.display()
        )
    })?;

    Ok(repo_dir)
}

/// Returns the cache root used for registry checkouts for this repository.
pub(crate) fn registry_cache_root_for_repo(repo_root: &Path) -> Result<PathBuf> {
    let git_common_dir = git_common_dir(repo_root)?;
    Ok(git_common_dir
        .join(DX_COMPONENTS_CACHE_DIR)
        .join("registries"))
}

/// Returns the deterministic cache path for a specific registry pin.
pub(crate) fn component_cache_path(repo_root: &Path, registry: &RegistrySpec) -> Result<PathBuf> {
    let mut hasher = DefaultHasher::new();
    registry.git.hash(&mut hasher);
    registry.rev.hash(&mut hasher);
    let hash = hasher.finish();
    Ok(registry_cache_root_for_repo(repo_root)?.join(format!("{hash:016x}")))
}

/// Resolves and validates the repository root.
fn find_repo_root() -> Result<PathBuf> {
    let cwd = std::env::current_dir().context("failed to resolve current directory")?;
    let output = run_command_checked(&cwd, "git", &["rev-parse", "--show-toplevel"])?;
    Ok(PathBuf::from(output.trim()))
}

/// Ensures a path is inside the repository and returns the path relative to root.
fn normalize_repo_relative_path(repo_root: &Path, raw_path: &Path) -> Result<PathBuf> {
    let absolute = if raw_path.is_absolute() {
        raw_path.to_path_buf()
    } else {
        repo_root.join(raw_path)
    };

    let canonical_repo_root = repo_root
        .canonicalize()
        .with_context(|| format!("failed to canonicalize repo root {}", repo_root.display()))?;

    let canonical_absolute = absolute
        .canonicalize()
        .with_context(|| format!("failed to resolve path {}", absolute.display()))?;

    let relative = canonical_absolute
        .strip_prefix(&canonical_repo_root)
        .with_context(|| {
            format!(
                "path '{}' must be inside repository root '{}'",
                absolute.display(),
                repo_root.display()
            )
        })?
        .to_path_buf();

    Ok(relative)
}

/// Resolves the absolute git common-dir for this repository.
fn git_common_dir(repo_root: &Path) -> Result<PathBuf> {
    let output = run_command_checked(
        repo_root,
        "git",
        &["rev-parse", "--path-format=absolute", "--git-common-dir"],
    )?;
    let path = PathBuf::from(output);
    if !path.is_absolute() {
        bail!("git common-dir must be absolute, got '{}'", path.display());
    }
    Ok(path)
}

/// Runs a command and returns stdout as a trimmed UTF-8 string.
fn run_command_checked(current_dir: &Path, program: &str, args: &[&str]) -> Result<String> {
    let output = Command::new(program)
        .current_dir(current_dir)
        .args(args)
        .output()
        .with_context(|| {
            format!(
                "failed to execute `{}` in {}",
                render_command(program, args),
                current_dir.display()
            )
        })?;

    if !output.status.success() {
        bail!(
            "command failed (exit code {:?}): `{}` in {}\nstdout:\n{}\nstderr:\n{}",
            output.status.code(),
            render_command(program, args),
            current_dir.display(),
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)
        );
    }

    let stdout = String::from_utf8(output.stdout)
        .context("command stdout was not valid UTF-8")?
        .trim()
        .to_string();

    Ok(stdout)
}

/// Renders a command line for diagnostics.
fn render_command(program: &str, args: &[&str]) -> String {
    if args.is_empty() {
        return program.to_string();
    }

    format!("{} {}", program, args.join(" "))
}

/// Converts a path to a UTF-8 owned string for CLI arguments.
fn path_to_string(path: &Path) -> String {
    path.to_string_lossy().into_owned()
}

#[cfg(test)]
mod tests {
    use std::time::{SystemTime, UNIX_EPOCH};

    use super::*;

    /// Creates a temporary git repository for cache path resolution tests.
    fn create_temp_git_repo() -> PathBuf {
        let nonce = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("clock before epoch")
            .as_nanos();
        let path = std::env::temp_dir().join(format!("mydoo-dx-components-registry-test-{nonce}"));
        fs::create_dir_all(&path).expect("create temp dir");
        run_command_checked(&path, "git", &["init"]).expect("git init temp repo");
        path
    }

    #[test]
    fn cache_path_is_stable_for_same_registry_pin() {
        let root = create_temp_git_repo();
        let pin = RegistrySpec {
            git: "https://github.com/DioxusLabs/components".to_string(),
            rev: "abc123".to_string(),
        };

        let first = component_cache_path(&root, &pin).unwrap();
        let second = component_cache_path(&root, &pin).unwrap();
        assert_eq!(first, second);

        fs::remove_dir_all(root).expect("remove temp repo");
    }

    #[test]
    fn cache_path_changes_when_revision_changes() {
        let root = create_temp_git_repo();
        let first = RegistrySpec {
            git: "https://github.com/DioxusLabs/components".to_string(),
            rev: "abc123".to_string(),
        };
        let second = RegistrySpec {
            git: "https://github.com/DioxusLabs/components".to_string(),
            rev: "def456".to_string(),
        };

        let first_path = component_cache_path(&root, &first).unwrap();
        let second_path = component_cache_path(&root, &second).unwrap();
        assert_ne!(first_path, second_path);

        fs::remove_dir_all(root).expect("remove temp repo");
    }
}
