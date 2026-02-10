use std::ffi::OsStr;
use std::fs;
use std::path::{Component, Path, PathBuf};
use std::process::{Command, ExitStatus, Output};
use std::time::{SystemTime, UNIX_EPOCH};

use anyhow::{Context, Result, bail};
use clap::Args;
use toml_edit::{DocumentMut, InlineTable, Item, Value, value};

use crate::commands::dioxus_lock::extract_dioxus_primitives_rev_from_lock_content;

/// The default branch that stores pristine vendored component snapshots.
const DEFAULT_VENDOR_BRANCH: &str = "vendor/dioxus-components-pristine";
/// The default versioned source of truth for components selected for vendoring.
const DEFAULT_COMPONENTS_FILE: &str = "crates/tasklens-ui/dioxus-vendor-components.toml";
/// Relative path to the UI crate root.
const UI_CRATE_DIR: &str = "crates/tasklens-ui";

/// Parsed vendor registry configuration loaded from TOML.
struct VendorRegistryConfig {
    git: String,
    rev: String,
}

/// Parsed vendor components configuration loaded from TOML.
struct VendorComponentsConfig {
    registry: VendorRegistryConfig,
    module_path: String,
    components: Vec<String>,
}

/// CLI options for updating vendored Dioxus components.
#[derive(Args, Debug)]
pub(crate) struct UpdateDioxusComponentsArgs {
    /// The vendor branch storing pristine snapshot commits.
    #[arg(long, default_value = DEFAULT_VENDOR_BRANCH)]
    vendor_branch: String,
    /// Path to the TOML file listing components to install.
    #[arg(long, default_value = DEFAULT_COMPONENTS_FILE)]
    components_file: PathBuf,
    /// Upgrade `dioxus-primitives` and use the provided revision in this run.
    #[arg(long)]
    upgrade_primitives: Option<String>,
    /// Keep temporary worktrees after the command finishes.
    #[arg(long, default_value_t = false)]
    keep_temp: bool,
    /// Allow running against a vendor branch that contains non-snapshot files.
    #[arg(long, default_value_t = false)]
    allow_non_orphan_vendor_branch: bool,
}

/// Immutable execution context shared across workflow steps.
struct WorkflowContext {
    repo_root: PathBuf,
    current_branch: String,
    head_sha: String,
    source_worktree: PathBuf,
    vendor_worktree: PathBuf,
}

/// Runs the full vendor update workflow through the final merge attempt.
pub(crate) fn update_dioxus_components(args: &UpdateDioxusComponentsArgs) -> Result<()> {
    let repo_root = find_repo_root()?;
    ensure_clean_working_tree(&repo_root)?;
    ensure_dx_available(&repo_root)?;

    let current_branch = current_branch(&repo_root)?;
    let head_sha = git_stdout(&repo_root, &["rev-parse", "HEAD"])?;

    let source_worktree = new_temp_worktree_path("dx-source")?;
    let vendor_worktree = new_temp_worktree_path("dx-vendor")?;
    let mut cleanup = WorktreeCleanup::new(repo_root.clone(), args.keep_temp);
    cleanup.track(source_worktree.clone());
    cleanup.track(vendor_worktree.clone());

    let context = WorkflowContext {
        repo_root: repo_root.clone(),
        current_branch,
        head_sha,
        source_worktree: source_worktree.clone(),
        vendor_worktree: vendor_worktree.clone(),
    };

    let run_result = run_update_workflow(args, &context);

    if let Err(cleanup_error) = cleanup.cleanup() {
        if run_result.is_ok() {
            return Err(cleanup_error);
        }
        eprintln!("warning: failed to clean worktrees: {cleanup_error:#}");
    }

    run_result
}

/// Executes the core workflow in isolated source and vendor worktrees.
fn run_update_workflow(args: &UpdateDioxusComponentsArgs, context: &WorkflowContext) -> Result<()> {
    add_detached_worktree(
        &context.repo_root,
        &context.source_worktree,
        &context.head_sha,
    )?;
    add_detached_worktree(
        &context.repo_root,
        &context.vendor_worktree,
        &context.head_sha,
    )?;

    let component_file_relative =
        normalize_repo_relative_path(&context.repo_root, &args.components_file)?;

    let source_components_file = context.source_worktree.join(&component_file_relative);
    let source_ui_cargo_toml = context
        .source_worktree
        .join(UI_CRATE_DIR)
        .join("Cargo.toml");
    let source_dioxus_toml = context
        .source_worktree
        .join(UI_CRATE_DIR)
        .join("Dioxus.toml");
    let source_lockfile = context.source_worktree.join("Cargo.lock");
    let mut components_config = read_components_file(&source_components_file)?;

    let requested_rev = args.upgrade_primitives.as_deref();
    let mut effective_rev = requested_rev
        .unwrap_or(&components_config.registry.rev)
        .to_string();

    ensure_cargo_toml_pin(
        &source_ui_cargo_toml,
        &components_config.registry.git,
        &effective_rev,
    )?;

    if requested_rev.is_some() {
        run_command_checked(
            &context.source_worktree,
            "cargo",
            &["update", "-p", "dioxus-primitives"],
        )?;
        effective_rev = extract_dioxus_primitives_rev_from_lockfile(&source_lockfile)?;
        ensure_cargo_toml_pin(
            &source_ui_cargo_toml,
            &components_config.registry.git,
            &effective_rev,
        )?;
    }

    clean_dioxus_toml_components_config(&source_dioxus_toml)?;
    ensure_components_file_registry_pin(
        &source_components_file,
        &components_config.registry.git,
        &effective_rev,
    )?;
    components_config.registry.rev = effective_rev.clone();
    if components_config.components.is_empty() {
        bail!(
            "components file '{}' does not contain any components",
            source_components_file.display()
        );
    }
    let vendored_components_dir =
        vendored_components_repo_dir_from_module_path(&components_config.module_path)?;

    checkout_or_init_vendor_branch(
        &context.repo_root,
        &context.vendor_worktree,
        &args.vendor_branch,
        &vendored_components_dir,
        args.allow_non_orphan_vendor_branch,
    )?;

    run_dx_components_add(
        &context.source_worktree,
        &components_config.registry.git,
        &components_config.module_path,
        &components_config.components,
        &effective_rev,
    )?;
    reapply_primitives_pin_after_dx(
        &context.source_worktree,
        &source_ui_cargo_toml,
        &components_config.registry.git,
        &effective_rev,
    )?;

    let changed_assets = changed_paths_in(&context.source_worktree, "crates/tasklens-ui/assets")?;

    sync_vendor_snapshot(
        &context.source_worktree,
        &context.vendor_worktree,
        &component_file_relative,
        &vendored_components_dir,
        &changed_assets,
    )?;

    let commit_message = build_vendor_snapshot_commit_message(
        args,
        &context.current_branch,
        &component_file_relative,
        &components_config,
        &effective_rev,
    );
    let snapshot_commit_created =
        commit_vendor_snapshot(&context.vendor_worktree, &commit_message)?;
    push_vendor_branch(&context.repo_root, &args.vendor_branch)?;

    attempt_final_merge(&context.repo_root, &args.vendor_branch)?;

    println!(
        "update-dioxus-components completed: target_branch={current_branch}, vendor_branch={}, rev={}, components=[{}], snapshot_commit_created={snapshot_commit_created}, vendor_branch_pushed=true",
        args.vendor_branch,
        effective_rev,
        components_config.components.join(", "),
        current_branch = context.current_branch,
    );

    Ok(())
}

/// Creates a detached worktree at `path` from `head_sha`.
fn add_detached_worktree(repo_root: &Path, path: &Path, head_sha: &str) -> Result<()> {
    run_command_checked(
        repo_root,
        "git",
        &[
            "worktree",
            "add",
            "--detach",
            &path_to_string(path),
            head_sha,
        ],
    )
    .with_context(|| format!("failed to create worktree at {}", path.display()))
    .map(|_| ())
}

/// Resolves and validates the repository root.
fn find_repo_root() -> Result<PathBuf> {
    let cwd = std::env::current_dir().context("failed to resolve current directory")?;
    let output = run_command_checked(&cwd, "git", &["rev-parse", "--show-toplevel"])?;
    Ok(PathBuf::from(output.trim()))
}

/// Verifies that the current branch has no tracked or untracked changes.
fn ensure_clean_working_tree(repo_root: &Path) -> Result<()> {
    let status_output = run_command_checked(repo_root, "git", &["status", "--porcelain"])?;
    if !status_output.trim().is_empty() {
        bail!(
            "working tree must be clean before running update-dioxus-components. Found:\n{}",
            status_output
        );
    }
    Ok(())
}

/// Checks that `dx` is available on PATH before creating worktrees.
fn ensure_dx_available(repo_root: &Path) -> Result<()> {
    run_command_checked(repo_root, "dx", &["--version"]).map(|_| ())
}

/// Returns the currently checked-out branch name.
fn current_branch(repo_root: &Path) -> Result<String> {
    let branch = git_stdout(repo_root, &["branch", "--show-current"])?;
    if branch.is_empty() {
        bail!("update-dioxus-components requires a non-detached current branch");
    }
    Ok(branch)
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

/// Extracts the pinned `dioxus-primitives` revision from `Cargo.lock`.
fn extract_dioxus_primitives_rev_from_lockfile(lockfile_path: &Path) -> Result<String> {
    let content = fs::read_to_string(lockfile_path)
        .with_context(|| format!("failed to read {}", lockfile_path.display()))?;
    extract_dioxus_primitives_rev_from_lock_content(&content).map_err(|_| {
        anyhow::anyhow!(
            "could not find git source for dioxus-primitives in Cargo.lock; pass --upgrade-primitives <rev> to re-pin explicitly"
        )
    })
}

/// Ensures `crates/tasklens-ui/Cargo.toml` pins `dioxus-primitives` with git+rev.
fn ensure_cargo_toml_pin(cargo_toml_path: &Path, registry_git: &str, revision: &str) -> Result<()> {
    let original = fs::read_to_string(cargo_toml_path)
        .with_context(|| format!("failed to read {}", cargo_toml_path.display()))?;
    let updated = ensure_cargo_toml_pin_content(&original, registry_git, revision)?;
    if updated != original {
        fs::write(cargo_toml_path, updated)
            .with_context(|| format!("failed to write {}", cargo_toml_path.display()))?;
    }
    Ok(())
}

/// Returns updated Cargo.toml content with `dioxus-primitives` git/rev pins.
fn ensure_cargo_toml_pin_content(
    content: &str,
    registry_git: &str,
    revision: &str,
) -> Result<String> {
    let mut document = content
        .parse::<DocumentMut>()
        .context("failed to parse Cargo.toml as TOML")?;

    let dependencies = document
        .entry("dependencies")
        .or_insert(Item::Table(Default::default()));

    let dependencies_table = dependencies
        .as_table_mut()
        .ok_or_else(|| anyhow::anyhow!("[dependencies] must be a table in Cargo.toml"))?;

    let dependency_item = dependencies_table
        .entry("dioxus-primitives")
        .or_insert(Item::Value(Value::InlineTable(Default::default())));

    let dependency_table_like = dependency_item.as_table_like_mut().ok_or_else(|| {
        anyhow::anyhow!(
            "`dependencies.dioxus-primitives` must be an inline table or table, not a scalar"
        )
    })?;

    let version_value = dependency_table_like
        .get("version")
        .and_then(Item::as_value)
        .cloned();
    let default_features_value = dependency_table_like
        .get("default-features")
        .and_then(Item::as_value)
        .cloned();
    let features_value = dependency_table_like
        .get("features")
        .and_then(Item::as_value)
        .cloned();
    let mut remaining_values: Vec<(String, Value)> = dependency_table_like
        .iter()
        .filter_map(|(key, item)| {
            if matches!(
                key,
                "git" | "rev" | "version" | "default-features" | "features"
            ) {
                return None;
            }
            item.as_value()
                .cloned()
                .map(|value| (key.to_string(), value))
        })
        .collect();
    remaining_values.sort_by(|(left_key, _), (right_key, _)| left_key.cmp(right_key));

    let git_is_pinned = dependency_table_like
        .get("git")
        .and_then(Item::as_value)
        .and_then(Value::as_str)
        .is_some_and(|current| current == registry_git);
    let rev_is_pinned = dependency_table_like
        .get("rev")
        .and_then(Item::as_value)
        .and_then(Value::as_str)
        .is_some_and(|current| current == revision);

    // Avoid rewriting when the pin already matches exactly; this prevents
    // formatting-only churn that can cause avoidable merge conflicts.
    if git_is_pinned && rev_is_pinned {
        return Ok(content.to_string());
    }

    let mut canonical_table = InlineTable::default();
    canonical_table.insert("git", Value::from(registry_git));
    canonical_table.insert("rev", Value::from(revision));

    if let Some(version) = version_value {
        canonical_table.insert("version", version);
    }
    if let Some(default_features) = default_features_value {
        canonical_table.insert("default-features", default_features);
    }
    if let Some(features) = features_value {
        canonical_table.insert("features", features);
    }
    for (key, value) in remaining_values {
        canonical_table.insert(&key, value);
    }

    *dependency_item = Item::Value(Value::InlineTable(canonical_table));

    Ok(document.to_string())
}

/// Removes Dioxus component installer config so xtask config is the only source of truth.
fn clean_dioxus_toml_components_config(dioxus_toml_path: &Path) -> Result<()> {
    let original = fs::read_to_string(dioxus_toml_path)
        .with_context(|| format!("failed to read {}", dioxus_toml_path.display()))?;
    let updated = clean_dioxus_toml_components_config_content(&original)?;
    if updated != original {
        fs::write(dioxus_toml_path, updated)
            .with_context(|| format!("failed to write {}", dioxus_toml_path.display()))?;
    }
    Ok(())
}

/// Returns Dioxus.toml content with `[components]` removed when present.
fn clean_dioxus_toml_components_config_content(content: &str) -> Result<String> {
    let mut document = content
        .parse::<DocumentMut>()
        .context("failed to parse Dioxus.toml as TOML")?;

    let _ = document.remove("components");

    Ok(document.to_string())
}

/// Ensures vendor config contains the selected registry pin.
fn ensure_components_file_registry_pin(
    components_file: &Path,
    registry_git: &str,
    registry_rev: &str,
) -> Result<()> {
    let original = fs::read_to_string(components_file)
        .with_context(|| format!("failed to read {}", components_file.display()))?;
    let updated =
        ensure_components_file_registry_pin_content(&original, registry_git, registry_rev)?;
    if updated != original {
        fs::write(components_file, updated)
            .with_context(|| format!("failed to write {}", components_file.display()))?;
    }
    Ok(())
}

/// Returns updated vendor config content with required registry pins.
fn ensure_components_file_registry_pin_content(
    content: &str,
    registry_git: &str,
    registry_rev: &str,
) -> Result<String> {
    let mut document = content
        .parse::<DocumentMut>()
        .context("failed to parse components file TOML")?;

    let registry_item = document
        .entry("registry")
        .or_insert(Item::Table(Default::default()));
    let registry_table = registry_item
        .as_table_mut()
        .ok_or_else(|| anyhow::anyhow!("[registry] must be a table in components file"))?;

    registry_table.insert("git", value(registry_git));
    registry_table.insert("rev", value(registry_rev));

    Ok(document.to_string())
}

/// Converts a configured UI-relative module path to a repository-relative path.
fn vendored_components_repo_dir_from_module_path(module_path: &str) -> Result<PathBuf> {
    validate_module_path(module_path)?;
    Ok(Path::new(UI_CRATE_DIR).join(module_path))
}

/// Parses a component list TOML file and returns vendor configuration.
fn read_components_file(components_file: &Path) -> Result<VendorComponentsConfig> {
    let content = fs::read_to_string(components_file).with_context(|| {
        format!(
            "failed to read components file {}",
            components_file.display()
        )
    })?;
    parse_components_file_content(&content).with_context(|| {
        format!(
            "failed to parse components file {}; expected `module_path = \"src/...\"`, `[registry].git`, `[registry].rev`, and `components = [\"name\", ...]`",
            components_file.display()
        )
    })
}

/// Validates that a module path is relative and safely scoped under `src/`.
fn validate_module_path(module_path: &str) -> Result<()> {
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

/// Parses TOML content expected to contain `module_path`, `[registry]`, and `components`.
fn parse_components_file_content(content: &str) -> Result<VendorComponentsConfig> {
    let document = content
        .parse::<DocumentMut>()
        .context("components file is not valid TOML")?;

    let module_path = document
        .get("module_path")
        .ok_or_else(|| anyhow::anyhow!("missing `module_path` string"))?
        .as_str()
        .ok_or_else(|| anyhow::anyhow!("`module_path` must be a string"))?
        .trim()
        .to_string();
    validate_module_path(&module_path)?;

    let registry = document
        .get("registry")
        .ok_or_else(|| anyhow::anyhow!("missing `registry` table"))?
        .as_table_like()
        .ok_or_else(|| anyhow::anyhow!("`registry` must be a table"))?;
    let registry_git = registry
        .get("git")
        .and_then(Item::as_value)
        .and_then(Value::as_str)
        .ok_or_else(|| anyhow::anyhow!("`registry.git` must be a string"))?
        .trim()
        .to_string();
    validate_registry_git(&registry_git)?;
    let registry_rev = registry
        .get("rev")
        .and_then(Item::as_value)
        .and_then(Value::as_str)
        .ok_or_else(|| anyhow::anyhow!("`registry.rev` must be a string"))?
        .trim()
        .to_string();
    validate_registry_rev(&registry_rev)?;

    let components = document
        .get("components")
        .ok_or_else(|| anyhow::anyhow!("missing `components` array"))?
        .as_array()
        .ok_or_else(|| anyhow::anyhow!("`components` must be an array"))?;

    let mut result = Vec::new();
    for item in components {
        let component = item
            .as_str()
            .ok_or_else(|| anyhow::anyhow!("all `components` entries must be strings"))?;
        if component.trim().is_empty() {
            bail!("component names cannot be empty strings");
        }
        result.push(component.to_string());
    }

    Ok(VendorComponentsConfig {
        registry: VendorRegistryConfig {
            git: registry_git,
            rev: registry_rev,
        },
        module_path,
        components: result,
    })
}

/// Relationship between local and remote vendor branch history.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum BranchRelation {
    Equal,
    LocalAhead,
    LocalBehind,
    Diverged,
}

/// Checks out and synchronizes the vendor branch, or initializes a new orphan branch.
fn checkout_or_init_vendor_branch(
    repo_root: &Path,
    vendor_worktree: &Path,
    vendor_branch: &str,
    vendored_components_dir: &Path,
    allow_non_orphan_vendor_branch: bool,
) -> Result<()> {
    fetch_origin(repo_root)?;

    let local_exists = local_branch_exists(repo_root, vendor_branch)?;
    let remote_exists = remote_tracking_branch_exists(repo_root, vendor_branch)?;
    let remote_ref = format!("origin/{vendor_branch}");

    if local_exists && remote_exists {
        let relation = vendor_branch_relation_to_origin(repo_root, vendor_branch)?;
        if relation == BranchRelation::Diverged {
            let local_sha = git_stdout(repo_root, &["rev-parse", vendor_branch])?;
            let remote_sha = git_stdout(repo_root, &["rev-parse", remote_ref.as_str()])?;
            bail!(
                "vendor branch '{vendor_branch}' has diverged from '{remote_ref}'. local={local_sha}, remote={remote_sha}. Reconcile history manually, then rerun update-dioxus-components."
            );
        }

        run_command_checked(vendor_worktree, "git", &["checkout", vendor_branch])?;
        if relation == BranchRelation::LocalBehind {
            run_command_checked(
                vendor_worktree,
                "git",
                &["merge", "--ff-only", remote_ref.as_str()],
            )?;
        }
        validate_vendor_branch_shape(
            vendor_worktree,
            vendored_components_dir,
            allow_non_orphan_vendor_branch,
        )?;
        return Ok(());
    }

    if !local_exists && remote_exists {
        run_command_checked(
            vendor_worktree,
            "git",
            &[
                "checkout",
                "--track",
                "-b",
                vendor_branch,
                remote_ref.as_str(),
            ],
        )?;
        validate_vendor_branch_shape(
            vendor_worktree,
            vendored_components_dir,
            allow_non_orphan_vendor_branch,
        )?;
        return Ok(());
    }

    if local_exists {
        run_command_checked(vendor_worktree, "git", &["checkout", vendor_branch])?;
        validate_vendor_branch_shape(
            vendor_worktree,
            vendored_components_dir,
            allow_non_orphan_vendor_branch,
        )?;
        return Ok(());
    }

    run_command_checked(
        vendor_worktree,
        "git",
        &["checkout", "--orphan", vendor_branch],
    )?;

    // An orphan checkout leaves files from the previous tree as untracked; clear them.
    let _ = run_command(vendor_worktree, "git", &["rm", "-rf", "."])?;
    run_command_checked(vendor_worktree, "git", &["clean", "-fdx"])?;

    Ok(())
}

/// Validates that tracked files in the vendor branch remain within snapshot-managed paths.
fn validate_vendor_branch_shape(
    vendor_worktree: &Path,
    vendored_components_dir: &Path,
    allow_non_orphan_vendor_branch: bool,
) -> Result<()> {
    let tracked_files = run_command_checked(vendor_worktree, "git", &["ls-files"])?;

    let unexpected_files: Vec<String> = tracked_files
        .lines()
        .filter(|line| !line.trim().is_empty())
        .filter(|line| !is_allowed_vendor_path(Path::new(line), vendored_components_dir))
        .map(ToString::to_string)
        .collect();

    if unexpected_files.is_empty() {
        return Ok(());
    }

    if allow_non_orphan_vendor_branch {
        eprintln!(
            "warning: vendor branch contains non-snapshot files and --allow-non-orphan-vendor-branch was set"
        );
        return Ok(());
    }

    bail!(
        "vendor branch contains files outside managed snapshot paths (first few: {}). Use --allow-non-orphan-vendor-branch to override intentionally.",
        unexpected_files
            .iter()
            .take(5)
            .map(String::as_str)
            .collect::<Vec<_>>()
            .join(", ")
    )
}

/// Returns true when a path is allowed to exist on the vendor branch.
///
/// This allowlist is strict and matches snapshot-managed content only.
/// `Cargo.lock` is intentionally excluded so vendor snapshots never track it.
fn is_allowed_vendor_path(path: &Path, vendored_components_dir: &Path) -> bool {
    if path == Path::new("crates/tasklens-ui/Cargo.toml") {
        return true;
    }

    if path == Path::new("crates/tasklens-ui/Dioxus.toml") {
        return true;
    }

    if path == Path::new(DEFAULT_COMPONENTS_FILE) {
        return true;
    }

    path.starts_with(vendored_components_dir) || path.starts_with("crates/tasklens-ui/assets")
}

/// Executes `dx components add` in the source worktree UI crate.
fn run_dx_components_add(
    source_worktree: &Path,
    registry_git: &str,
    module_path: &str,
    components: &[String],
    revision: &str,
) -> Result<()> {
    let ui_dir = source_worktree.join(UI_CRATE_DIR);

    let mut args: Vec<String> = vec![
        "components".to_string(),
        "add".to_string(),
        "--module-path".to_string(),
        module_path.to_string(),
        "--git".to_string(),
        registry_git.to_string(),
        "--rev".to_string(),
        revision.to_string(),
        "--force".to_string(),
    ];

    args.extend(components.iter().cloned());

    run_command_checked_owned(&ui_dir, "dx", &args).map(|_| ())
}

/// Re-applies and locks the `dioxus-primitives` pin after `dx` mutates manifests.
fn reapply_primitives_pin_after_dx(
    source_worktree: &Path,
    source_ui_cargo_toml: &Path,
    registry_git: &str,
    revision: &str,
) -> Result<()> {
    ensure_cargo_toml_pin(source_ui_cargo_toml, registry_git, revision)?;
    run_command_checked(
        source_worktree,
        "cargo",
        &["update", "-p", "dioxus-primitives", "--precise", revision],
    )?;
    Ok(())
}

/// Returns changed paths under `pathspec` in the source worktree.
fn changed_paths_in(source_worktree: &Path, pathspec: &str) -> Result<Vec<PathBuf>> {
    let output = run_command_checked(
        source_worktree,
        "git",
        &["diff", "--name-only", "--", pathspec],
    )?;
    Ok(output
        .lines()
        .filter(|line| !line.trim().is_empty())
        .map(PathBuf::from)
        .collect())
}

/// Mirrors snapshot-managed paths from source into vendor worktree.
fn sync_vendor_snapshot(
    source_worktree: &Path,
    vendor_worktree: &Path,
    component_file_relative: &Path,
    vendored_components_dir: &Path,
    changed_assets: &[PathBuf],
) -> Result<()> {
    mirror_directory(
        &source_worktree.join(vendored_components_dir),
        &vendor_worktree.join(vendored_components_dir),
    )?;

    sync_file(
        &source_worktree.join("crates/tasklens-ui/Cargo.toml"),
        &vendor_worktree.join("crates/tasklens-ui/Cargo.toml"),
    )?;

    sync_file(
        &source_worktree.join("crates/tasklens-ui/Dioxus.toml"),
        &vendor_worktree.join("crates/tasklens-ui/Dioxus.toml"),
    )?;

    sync_file(
        &source_worktree.join(component_file_relative),
        &vendor_worktree.join(component_file_relative),
    )?;

    // Intentionally do not mirror Cargo.lock in vendor snapshots (no copy,
    // update, or deletion). Lockfile ownership stays with the target branch,
    // and vendor branches are expected to never track Cargo.lock.

    for relative_asset in changed_assets {
        let source_path = source_worktree.join(relative_asset);
        let destination_path = vendor_worktree.join(relative_asset);

        if source_path.is_file() {
            sync_file(&source_path, &destination_path)?;
            continue;
        }

        if source_path.is_dir() {
            mirror_directory(&source_path, &destination_path)?;
            continue;
        }

        remove_path_if_exists(&destination_path)?;
    }

    Ok(())
}

/// Stages and commits snapshot changes on the vendor branch.
fn commit_vendor_snapshot(
    vendor_worktree: &Path,
    commit_message: &VendorSnapshotCommitMessage,
) -> Result<bool> {
    run_command_checked(vendor_worktree, "git", &["add", "-A"])?;

    if !has_staged_changes(vendor_worktree)? {
        return Ok(false);
    }

    run_command_checked(
        vendor_worktree,
        "git",
        &[
            "commit",
            "-m",
            &commit_message.subject,
            "-m",
            &commit_message.body,
        ],
    )?;
    Ok(true)
}

/// Pushes the vendor branch to origin before merge so snapshots are durable.
fn push_vendor_branch(repo_root: &Path, vendor_branch: &str) -> Result<()> {
    run_command_checked(
        repo_root,
        "git",
        &["push", "--set-upstream", "origin", vendor_branch],
    )
    .with_context(|| {
        format!("failed to push vendor branch '{vendor_branch}' to origin before merge")
    })?;
    Ok(())
}

/// Multi-line commit message payload for vendor snapshot commits.
struct VendorSnapshotCommitMessage {
    subject: String,
    body: String,
}

/// Builds a provenance-rich commit message so snapshot origin is reconstructible.
fn build_vendor_snapshot_commit_message(
    args: &UpdateDioxusComponentsArgs,
    target_branch: &str,
    component_file_relative: &Path,
    components_config: &VendorComponentsConfig,
    revision: &str,
) -> VendorSnapshotCommitMessage {
    let subject = format!("chore(vendor): update Dioxus Components source to {revision}");
    let canonical_invocation = render_canonical_update_invocation(args, component_file_relative);
    let observed_invocation = render_observed_invocation();
    let installer_invocation = render_dx_components_add_invocation(
        &components_config.registry.git,
        &components_config.module_path,
        &components_config.components,
        revision,
    );
    let body = format!(
        "Automated vendor snapshot created by xtask.

Reason:
- Capture pristine installer output from Dioxus Components at a pinned revision.

Intended merge flow:
- Commit branch: {vendor_branch}
- Intended target branch: {target_branch}
- Integration method: git merge --no-ff {vendor_branch}

Automation provenance:
- Tool: cargo xtask update-dioxus-components
- Canonical invocation: {canonical_invocation}
- Observed argv: {observed_invocation}
- Installer invocation (in {UI_CRATE_DIR}): {installer_invocation}

Snapshot inputs:
- Registry: {registry_git}
- Revision: {revision}
- Components file: {}
- Components: {}",
        path_to_string(component_file_relative),
        components_config.components.join(", "),
        registry_git = components_config.registry.git,
        vendor_branch = args.vendor_branch,
    );

    VendorSnapshotCommitMessage { subject, body }
}

/// Renders the current process argv for provenance logging.
fn render_observed_invocation() -> String {
    std::env::args().collect::<Vec<_>>().join(" ")
}

/// Renders an equivalent cargo invocation from resolved xtask options.
fn render_canonical_update_invocation(
    args: &UpdateDioxusComponentsArgs,
    component_file_relative: &Path,
) -> String {
    let mut command_args = vec!["xtask".to_string(), "update-dioxus-components".to_string()];

    if args.vendor_branch != DEFAULT_VENDOR_BRANCH {
        command_args.push("--vendor-branch".to_string());
        command_args.push(args.vendor_branch.clone());
    }

    if component_file_relative != Path::new(DEFAULT_COMPONENTS_FILE) {
        command_args.push("--components-file".to_string());
        command_args.push(path_to_string(component_file_relative));
    }

    if let Some(revision) = args.upgrade_primitives.as_ref() {
        command_args.push("--upgrade-primitives".to_string());
        command_args.push(revision.clone());
    }

    if args.keep_temp {
        command_args.push("--keep-temp".to_string());
    }

    if args.allow_non_orphan_vendor_branch {
        command_args.push("--allow-non-orphan-vendor-branch".to_string());
    }

    render_command("cargo", &command_args)
}

/// Renders the `dx components add` command xtask executes in the source worktree.
fn render_dx_components_add_invocation(
    registry_git: &str,
    module_path: &str,
    components: &[String],
    revision: &str,
) -> String {
    let mut args = vec![
        "components".to_string(),
        "add".to_string(),
        "--module-path".to_string(),
        module_path.to_string(),
        "--git".to_string(),
        registry_git.to_string(),
        "--rev".to_string(),
        revision.to_string(),
        "--force".to_string(),
    ];
    args.extend(components.iter().cloned());

    render_command("dx", &args)
}

/// Performs the terminal merge attempt into the caller's current branch.
fn attempt_final_merge(repo_root: &Path, vendor_branch: &str) -> Result<()> {
    let merge_base_status =
        run_command_status(repo_root, "git", &["merge-base", "HEAD", vendor_branch])?;
    let allow_unrelated = !merge_base_status.success();

    let mut merge_args: Vec<String> = vec!["merge".to_string(), "--no-ff".to_string()];
    if allow_unrelated {
        merge_args.push("--allow-unrelated-histories".to_string());
    }
    merge_args.push(vendor_branch.to_string());

    let merge_output = run_command_owned(repo_root, "git", &merge_args)?;
    if merge_output.status.success() {
        return Ok(());
    }

    let conflicts = run_command_checked(repo_root, "git", &["ls-files", "-u"])?;
    if !conflicts.trim().is_empty() {
        bail!(
            "merge conflict encountered while merging '{vendor_branch}'. Resolve conflicts manually, then complete the merge.\n{}",
            String::from_utf8_lossy(&merge_output.stderr)
        );
    }

    bail!(
        "final merge attempt failed for branch '{vendor_branch}'.\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&merge_output.stdout),
        String::from_utf8_lossy(&merge_output.stderr)
    )
}

/// Returns true if there are staged changes.
fn has_staged_changes(worktree: &Path) -> Result<bool> {
    let status = run_command_status(worktree, "git", &["diff", "--cached", "--quiet"])?;
    Ok(!status.success())
}

/// Returns true when a local branch exists.
fn local_branch_exists(repo_root: &Path, branch: &str) -> Result<bool> {
    let status = run_command_status(
        repo_root,
        "git",
        &[
            "show-ref",
            "--verify",
            "--quiet",
            &format!("refs/heads/{branch}"),
        ],
    )?;
    Ok(status.success())
}

/// Returns true when `origin/<branch>` exists locally after a fetch.
fn remote_tracking_branch_exists(repo_root: &Path, branch: &str) -> Result<bool> {
    let status = run_command_status(
        repo_root,
        "git",
        &[
            "show-ref",
            "--verify",
            "--quiet",
            &format!("refs/remotes/origin/{branch}"),
        ],
    )?;
    Ok(status.success())
}

/// Fetches remote refs so vendor branch sync decisions are based on origin state.
fn fetch_origin(repo_root: &Path) -> Result<()> {
    run_command_checked(repo_root, "git", &["fetch", "origin", "--prune"])
        .context("failed to fetch origin for vendor branch synchronization")
        .map(|_| ())
}

/// Computes local-vs-remote relationship for `branch` and `origin/branch`.
fn vendor_branch_relation_to_origin(repo_root: &Path, branch: &str) -> Result<BranchRelation> {
    let remote_ref = format!("origin/{branch}");
    let local_is_ancestor = is_ancestor(repo_root, branch, remote_ref.as_str())?;
    let remote_is_ancestor = is_ancestor(repo_root, remote_ref.as_str(), branch)?;
    Ok(classify_branch_relation(
        local_is_ancestor,
        remote_is_ancestor,
    ))
}

/// Classifies ancestry flags into a high-level branch relation.
fn classify_branch_relation(local_is_ancestor: bool, remote_is_ancestor: bool) -> BranchRelation {
    match (local_is_ancestor, remote_is_ancestor) {
        (true, true) => BranchRelation::Equal,
        (false, true) => BranchRelation::LocalAhead,
        (true, false) => BranchRelation::LocalBehind,
        (false, false) => BranchRelation::Diverged,
    }
}

/// Returns true when `ancestor_ref` is an ancestor of `descendant_ref`.
fn is_ancestor(repo_root: &Path, ancestor_ref: &str, descendant_ref: &str) -> Result<bool> {
    let status = run_command_status(
        repo_root,
        "git",
        &["merge-base", "--is-ancestor", ancestor_ref, descendant_ref],
    )?;

    if status.success() {
        return Ok(true);
    }

    if status.code() == Some(1) {
        return Ok(false);
    }

    bail!(
        "failed to compare ancestry between '{ancestor_ref}' and '{descendant_ref}' (exit code {:?})",
        status.code()
    )
}

/// Copies a file and ensures destination parent directories exist.
fn sync_file(source: &Path, destination: &Path) -> Result<()> {
    if !source.exists() {
        bail!("source file does not exist: {}", source.display());
    }

    let parent = destination.parent().ok_or_else(|| {
        anyhow::anyhow!(
            "destination path '{}' does not have a parent directory",
            destination.display()
        )
    })?;

    fs::create_dir_all(parent)
        .with_context(|| format!("failed to create directory {}", parent.display()))?;

    fs::copy(source, destination).with_context(|| {
        format!(
            "failed to copy file from '{}' to '{}'",
            source.display(),
            destination.display()
        )
    })?;

    Ok(())
}

/// Mirrors a directory recursively by replacing destination with source contents.
fn mirror_directory(source: &Path, destination: &Path) -> Result<()> {
    if !source.exists() {
        bail!("source directory does not exist: {}", source.display());
    }

    remove_path_if_exists(destination)?;
    copy_directory_recursive(source, destination)
}

/// Recursively copies all files and directories from source to destination.
fn copy_directory_recursive(source: &Path, destination: &Path) -> Result<()> {
    fs::create_dir_all(destination)
        .with_context(|| format!("failed to create directory {}", destination.display()))?;

    for entry in fs::read_dir(source)
        .with_context(|| format!("failed to read directory {}", source.display()))?
    {
        let entry =
            entry.with_context(|| format!("failed to read entry in {}", source.display()))?;
        let entry_path = entry.path();
        let destination_path = destination.join(entry.file_name());

        if entry_path.is_dir() {
            copy_directory_recursive(&entry_path, &destination_path)?;
        } else {
            sync_file(&entry_path, &destination_path)?;
        }
    }

    Ok(())
}

/// Removes a filesystem path if it exists.
fn remove_path_if_exists(path: &Path) -> Result<()> {
    if !path.exists() {
        return Ok(());
    }

    if path.is_dir() {
        fs::remove_dir_all(path)
            .with_context(|| format!("failed to remove directory {}", path.display()))?;
    } else {
        fs::remove_file(path)
            .with_context(|| format!("failed to remove file {}", path.display()))?;
    }

    Ok(())
}

/// Runs a command and returns stdout as a trimmed UTF-8 string.
fn git_stdout(repo_root: &Path, args: &[&str]) -> Result<String> {
    run_command_checked(repo_root, "git", args)
}

/// Converts a path to a UTF-8 owned string for CLI arguments.
fn path_to_string(path: &Path) -> String {
    path.to_string_lossy().into_owned()
}

/// Creates a unique temporary path for a worktree.
fn new_temp_worktree_path(prefix: &str) -> Result<PathBuf> {
    let nonce = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .context("system clock appears to be before UNIX_EPOCH")?
        .as_nanos();

    Ok(std::env::temp_dir().join(format!("mydoo-{prefix}-{nonce}")))
}

/// Executes a command with borrowed string args and returns trimmed stdout on success.
fn run_command_checked(current_dir: &Path, program: &str, args: &[&str]) -> Result<String> {
    let owned_args = args.iter().map(ToString::to_string).collect::<Vec<_>>();
    let output = run_command_owned(current_dir, program, &owned_args)?;
    ensure_command_success(current_dir, program, &owned_args, output)
}

/// Executes a command with owned args and returns full process output.
fn run_command_owned(current_dir: &Path, program: &str, args: &[String]) -> Result<Output> {
    let output = Command::new(program)
        .current_dir(current_dir)
        .args(args.iter().map(OsStr::new))
        .output()
        .with_context(|| {
            format!(
                "failed to execute `{}` in {}",
                render_command(program, args),
                current_dir.display()
            )
        })?;

    Ok(output)
}

/// Executes a command with borrowed args and returns full process output.
fn run_command(current_dir: &Path, program: &str, args: &[&str]) -> Result<Output> {
    let owned_args = args.iter().map(ToString::to_string).collect::<Vec<_>>();
    run_command_owned(current_dir, program, &owned_args)
}

/// Returns only the process exit status.
fn run_command_status(current_dir: &Path, program: &str, args: &[&str]) -> Result<ExitStatus> {
    run_command(current_dir, program, args).map(|output| output.status)
}

/// Executes an owned-args command and enforces exit code success.
fn run_command_checked_owned(current_dir: &Path, program: &str, args: &[String]) -> Result<String> {
    let output = run_command_owned(current_dir, program, args)?;
    ensure_command_success(current_dir, program, args, output)
}

/// Validates command success and returns trimmed UTF-8 stdout.
fn ensure_command_success(
    current_dir: &Path,
    program: &str,
    args: &[String],
    output: Output,
) -> Result<String> {
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
fn render_command(program: &str, args: &[String]) -> String {
    if args.is_empty() {
        return program.to_string();
    }

    format!("{} {}", program, args.join(" "))
}

/// Tracks temporary worktrees and removes them on completion.
struct WorktreeCleanup {
    repo_root: PathBuf,
    keep_temp: bool,
    tracked_paths: Vec<PathBuf>,
}

impl WorktreeCleanup {
    /// Creates a new cleanup guard for worktree paths.
    fn new(repo_root: PathBuf, keep_temp: bool) -> Self {
        Self {
            repo_root,
            keep_temp,
            tracked_paths: Vec::new(),
        }
    }

    /// Registers a worktree path for cleanup.
    fn track(&mut self, path: PathBuf) {
        self.tracked_paths.push(path);
    }

    /// Cleans up all tracked worktrees unless keep-temp is enabled.
    fn cleanup(&self) -> Result<()> {
        if self.keep_temp {
            eprintln!(
                "keeping temporary worktrees: {}",
                self.tracked_paths
                    .iter()
                    .map(|path| path.display().to_string())
                    .collect::<Vec<_>>()
                    .join(", ")
            );
            return Ok(());
        }

        let mut errors = Vec::new();
        for path in &self.tracked_paths {
            let args = vec![
                "worktree".to_string(),
                "remove".to_string(),
                "--force".to_string(),
                path_to_string(path),
            ];
            let output = run_command_owned(&self.repo_root, "git", &args)?;
            if !output.status.success() {
                errors.push(format!(
                    "failed to remove worktree {}: {}",
                    path.display(),
                    String::from_utf8_lossy(&output.stderr)
                ));
            }
        }

        if errors.is_empty() {
            return Ok(());
        }

        bail!("{}", errors.join("\n"))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn extracts_rev_from_lock_content() {
        let input = r#"
[[package]]
name = "dioxus-primitives"
version = "0.0.1"
source = "git+https://github.com/DioxusLabs/components#deadbeef"
"#;

        let revision = extract_dioxus_primitives_rev_from_lock_content(input).unwrap();
        assert_eq!(revision, "deadbeef");
    }

    #[test]
    fn extracts_rev_from_lock_content_with_query_string() {
        let input = r#"
[[package]]
name = "dioxus-primitives"
version = "0.0.1"
source = "git+https://github.com/DioxusLabs/components?rev=deadbeef#deadbeef"
"#;

        let revision = extract_dioxus_primitives_rev_from_lock_content(input).unwrap();
        assert_eq!(revision, "deadbeef");
    }

    #[test]
    fn parses_components_file() {
        let input = r#"
module_path = "src/dioxus_components"
components = ["button", "dialog"]
[registry]
git = "https://github.com/DioxusLabs/components"
rev = "abc123"
"#;
        let config = parse_components_file_content(input).unwrap();
        assert_eq!(
            config.registry.git,
            "https://github.com/DioxusLabs/components"
        );
        assert_eq!(config.registry.rev, "abc123");
        assert_eq!(config.module_path, "src/dioxus_components");
        assert_eq!(config.components, vec!["button", "dialog"]);
    }

    #[test]
    fn rejects_missing_components_array() {
        let input = r#"
module_path = "src/dioxus_components"
not_components = ["button"]
[registry]
git = "https://github.com/DioxusLabs/components"
rev = "abc123"
"#;
        assert!(parse_components_file_content(input).is_err());
    }

    #[test]
    fn rejects_missing_module_path() {
        let input = r#"
components = ["button"]
[registry]
git = "https://github.com/DioxusLabs/components"
rev = "abc123"
"#;
        assert!(parse_components_file_content(input).is_err());
    }

    #[test]
    fn rejects_unsafe_module_path() {
        let input = r#"
module_path = "../outside"
components = ["button"]
[registry]
git = "https://github.com/DioxusLabs/components"
rev = "abc123"
"#;
        assert!(parse_components_file_content(input).is_err());
    }

    #[test]
    fn rejects_missing_registry_table() {
        let input = r#"
module_path = "src/dioxus_components"
components = ["button"]
"#;
        assert!(parse_components_file_content(input).is_err());
    }

    #[test]
    fn cleans_dioxus_toml_components_config_content() {
        let input = r#"
[application]
platform = "web"

[components]

[components.registry]
git = "https://github.com/DioxusLabs/components"
rev = "abc123"
"#;

        let updated = clean_dioxus_toml_components_config_content(input).unwrap();
        assert!(!updated.contains("[components]"));
        assert!(!updated.contains("[components.registry]"));
    }

    #[test]
    fn clean_dioxus_toml_components_config_content_noop_when_missing() {
        let input = r#"
[application]
platform = "web"
"#;
        let updated = clean_dioxus_toml_components_config_content(input).unwrap();
        assert_eq!(updated, input);
    }

    #[test]
    fn updates_components_file_registry_pin_content() {
        let input = r#"
module_path = "src/dioxus_components"
components = ["button"]
[registry]
git = "https://github.com/example/components"
rev = "old"
"#;

        let updated = ensure_components_file_registry_pin_content(
            input,
            "https://github.com/DioxusLabs/components",
            "abc123",
        )
        .unwrap();
        assert!(updated.contains("git = \"https://github.com/DioxusLabs/components\""));
        assert!(updated.contains("rev = \"abc123\""));
    }

    #[test]
    fn updates_cargo_toml_content_with_rev() {
        let input = r#"
[dependencies]
dioxus-primitives = { git = "https://github.com/DioxusLabs/components", version = "0.0.1", default-features = false, features = ["router"] }
"#;

        let updated = ensure_cargo_toml_pin_content(
            input,
            "https://github.com/DioxusLabs/components",
            "abc123",
        )
        .unwrap();
        assert!(updated.contains("git = \"https://github.com/DioxusLabs/components\""));
        assert!(updated.contains("rev = \"abc123\""));
    }

    #[test]
    fn updates_cargo_toml_content_with_canonical_key_order() {
        let input = r#"
[dependencies]
dioxus-primitives = { git = "https://github.com/DioxusLabs/components", version = "0.0.1", default-features = false, features = ["router"] }
"#;

        let updated = ensure_cargo_toml_pin_content(
            input,
            "https://github.com/DioxusLabs/components",
            "abc123",
        )
        .unwrap();
        assert!(updated.contains(
            "dioxus-primitives = { git = \"https://github.com/DioxusLabs/components\", rev = \"abc123\", version = \"0.0.1\", default-features = false, features = [\"router\"] }"
        ));
    }

    #[test]
    fn cargo_toml_pin_noop_when_already_pinned() {
        let input = r#"
[dependencies]
dioxus-primitives = { git = "https://github.com/DioxusLabs/components", rev = "abc123", version = "0.0.1", default-features = false, features = ["router"] }
"#;

        let updated = ensure_cargo_toml_pin_content(
            input,
            "https://github.com/DioxusLabs/components",
            "abc123",
        )
        .unwrap();
        assert_eq!(updated, input);
    }

    #[test]
    fn allows_only_strict_vendor_paths() {
        let vendored_components_dir =
            vendored_components_repo_dir_from_module_path("src/dioxus_components").unwrap();

        assert!(!is_allowed_vendor_path(
            Path::new("Cargo.lock"),
            &vendored_components_dir
        ));
        assert!(is_allowed_vendor_path(
            Path::new("crates/tasklens-ui/src/dioxus_components/button/component.rs"),
            &vendored_components_dir
        ));
        assert!(is_allowed_vendor_path(
            Path::new("crates/tasklens-ui/assets/dx-components-theme.css"),
            &vendored_components_dir
        ));
        assert!(!is_allowed_vendor_path(
            Path::new("crates/tasklens-ui/src/main.rs"),
            &vendored_components_dir
        ));
    }

    #[test]
    fn classifies_branch_relation_states() {
        assert_eq!(classify_branch_relation(true, true), BranchRelation::Equal);
        assert_eq!(
            classify_branch_relation(false, true),
            BranchRelation::LocalAhead
        );
        assert_eq!(
            classify_branch_relation(true, false),
            BranchRelation::LocalBehind
        );
        assert_eq!(
            classify_branch_relation(false, false),
            BranchRelation::Diverged
        );
    }

    #[test]
    fn renders_canonical_update_invocation_with_overrides() {
        let args = UpdateDioxusComponentsArgs {
            vendor_branch: "vendor/custom".to_string(),
            components_file: PathBuf::from("crates/tasklens-ui/custom-components.toml"),
            upgrade_primitives: Some("deadbeef".to_string()),
            keep_temp: true,
            allow_non_orphan_vendor_branch: true,
        };

        let rendered = render_canonical_update_invocation(
            &args,
            Path::new("crates/tasklens-ui/custom-components.toml"),
        );

        assert!(rendered.contains("cargo xtask update-dioxus-components"));
        assert!(rendered.contains("--vendor-branch vendor/custom"));
        assert!(rendered.contains("--components-file crates/tasklens-ui/custom-components.toml"));
        assert!(rendered.contains("--upgrade-primitives deadbeef"));
        assert!(rendered.contains("--keep-temp"));
        assert!(rendered.contains("--allow-non-orphan-vendor-branch"));
    }

    #[test]
    fn builds_vendor_snapshot_commit_message_with_provenance_fields() {
        let args = UpdateDioxusComponentsArgs {
            vendor_branch: DEFAULT_VENDOR_BRANCH.to_string(),
            components_file: PathBuf::from(DEFAULT_COMPONENTS_FILE),
            upgrade_primitives: None,
            keep_temp: false,
            allow_non_orphan_vendor_branch: false,
        };
        let components_config = VendorComponentsConfig {
            registry: VendorRegistryConfig {
                git: "https://github.com/DioxusLabs/components".to_string(),
                rev: "abc123".to_string(),
            },
            module_path: "src/dioxus_components".to_string(),
            components: vec!["button".to_string(), "dialog".to_string()],
        };

        let message = build_vendor_snapshot_commit_message(
            &args,
            "docs/dioxus-migration",
            Path::new(DEFAULT_COMPONENTS_FILE),
            &components_config,
            "abc123",
        );

        assert_eq!(
            message.subject,
            "chore(vendor): update Dioxus Components source to abc123"
        );
        assert!(
            message
                .body
                .contains("Intended target branch: docs/dioxus-migration")
        );
        assert!(
            message
                .body
                .contains("Commit branch: vendor/dioxus-components-pristine")
        );
        assert!(
            message
                .body
                .contains("Canonical invocation: cargo xtask update-dioxus-components")
        );
        assert!(message.body.contains("Observed argv: "));
        assert!(message.body.contains("dx components add --module-path src/dioxus_components --git https://github.com/DioxusLabs/components --rev abc123 --force button dialog"));
    }
}
