use std::ffi::OsStr;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, ExitStatus, Output};
use std::time::{SystemTime, UNIX_EPOCH};

use anstyle::{AnsiColor, Style};
use anyhow::{Context, Result, bail};
use clap::Args;
use toml_edit::{DocumentMut, InlineTable, Item, Value};

use crate::commands::dioxus_info::{
    DEFAULT_DIOXUS_VENDOR_COMPONENTS_FILE, DioxusVendorComponentsConfig,
    DioxusVendorRegistryConfig, extract_dioxus_primitives_rev_from_lock_content,
    extract_manifest_dioxus_primitives_pin_from_content, read_dioxus_vendor_components_config,
    validate_dioxus_vendor_module_path,
};
use crate::commands::dx_components::{installer, registry::RegistrySpec};

/// The default branch that stores pristine vendored component snapshots.
const DEFAULT_VENDOR_BRANCH: &str = "vendor/dioxus-components-pristine";
/// Relative path to the UI crate root.
const UI_CRATE_DIR: &str = "crates/tasklens-ui";

/// CLI options for updating vendored Dioxus components.
#[derive(Args, Debug)]
pub(crate) struct UpdateDioxusComponentsArgs {
    /// Use this git revision for the `dioxus-primitives` crate pin
    /// in `crates/tasklens-ui/Cargo.toml` for this run.
    #[arg(long, verbatim_doc_comment)]
    primitives_rev: Option<String>,
    /// Check whether these pins are behind upstream HEAD:
    /// - `[registry].rev` in `dioxus-vendor-components.toml`
    /// - `dependencies.dioxus-primitives.rev` in `Cargo.toml`
    ///
    /// Prints guidance only; does not vendor components.
    #[arg(long, default_value_t = false, verbatim_doc_comment)]
    check_revisions: bool,
    /// Branch storing pristine vendor snapshot commits:
    /// direct internal dx-components installer output before local edits.
    #[arg(long, default_value = DEFAULT_VENDOR_BRANCH, verbatim_doc_comment)]
    vendor_branch: String,
    /// Path to the TOML file listing components to install.
    #[arg(long, default_value = DEFAULT_DIOXUS_VENDOR_COMPONENTS_FILE)]
    components_file: PathBuf,
    /// Keep temporary worktrees after the command finishes.
    #[arg(long, default_value_t = false)]
    keep_temp: bool,
    /// Disable the strict vendor-branch allowlist check.
    ///
    /// Allows vendoring to continue even when the vendor branch tracks files
    /// outside snapshot-managed paths:
    /// - component source
    /// - selected assets
    /// - UI Cargo.toml
    /// - UI Dioxus.toml
    /// - vendor config
    ///
    /// Use only for one-off migration/cleanup work.
    /// Keep this disabled for normal runs.
    #[arg(long, default_value_t = false, verbatim_doc_comment)]
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
    if args.check_revisions {
        return check_revisions_only(args);
    }

    let repo_root = find_repo_root()?;
    ensure_clean_working_tree(&repo_root)?;

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

/// Checks for newer upstream revisions and prints revision guidance without mutating files.
fn check_revisions_only(args: &UpdateDioxusComponentsArgs) -> Result<()> {
    if args.primitives_rev.is_some() {
        bail!(
            "--check-revisions cannot be combined with --primitives-rev; this mode only reports newer upstream revisions"
        );
    }

    let repo_root = find_repo_root()?;
    fetch_origin(&repo_root)?;

    let component_file_relative = normalize_repo_relative_path(&repo_root, &args.components_file)?;
    let components_file = repo_root.join(component_file_relative);
    let ui_cargo_toml = repo_root.join(UI_CRATE_DIR).join("Cargo.toml");

    let components_config = read_dioxus_vendor_components_config(&components_file)?;
    let primitives_pin = read_manifest_dioxus_primitives_pin(&ui_cargo_toml)?;

    check_upstream_revisions(
        &components_config.registry,
        &primitives_pin,
        RevisionReportContext {
            repo_root: Some(&repo_root),
            vendor_branch: Some(&args.vendor_branch),
        },
    )?;

    println!(
        "check-revisions completed: components_rev={}, primitives_rev={}",
        components_config.registry.rev, primitives_pin.rev
    );

    Ok(())
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
    let components_config = read_dioxus_vendor_components_config(&source_components_file)?;
    let components_revision = components_config.registry.rev.clone();

    let mut primitives_pin = read_manifest_dioxus_primitives_pin(&source_ui_cargo_toml)?;
    if let Some(requested_primitives_rev) = args.primitives_rev.as_deref() {
        ensure_cargo_toml_pin(
            &source_ui_cargo_toml,
            &primitives_pin.git,
            requested_primitives_rev,
        )?;
        run_command_checked(
            &context.source_worktree,
            "cargo",
            &[
                "update",
                "-p",
                "dioxus-primitives",
                "--precise",
                requested_primitives_rev,
            ],
        )?;
        let resolved_primitives_rev =
            extract_dioxus_primitives_rev_from_lockfile(&source_lockfile)?;
        ensure_cargo_toml_pin(
            &source_ui_cargo_toml,
            &primitives_pin.git,
            &resolved_primitives_rev,
        )?;
        primitives_pin.rev = resolved_primitives_rev;
    }

    clean_dioxus_toml_components_config(&source_dioxus_toml)?;
    if components_config.components.is_empty() {
        bail!(
            "components file '{}' does not contain any components",
            source_components_file.display()
        );
    }
    let vendored_components_dir =
        vendored_components_repo_dir_from_module_path(&components_config.module_path)?;
    ensure_components_registry_is_compatible_with_primitives_pin(
        &components_config.registry,
        &primitives_pin,
        RevisionReportContext {
            repo_root: Some(&context.repo_root),
            vendor_branch: Some(&args.vendor_branch),
        },
    )?;

    checkout_or_init_vendor_branch(
        &context.repo_root,
        &context.vendor_worktree,
        &args.vendor_branch,
        &vendored_components_dir,
        args.allow_non_orphan_vendor_branch,
    )?;

    run_dx_components_vendor(
        &context.source_worktree,
        &components_config.module_path,
        &components_config.components,
        RegistrySpec {
            git: components_config.registry.git.clone(),
            rev: components_revision.clone(),
        },
    )?;
    reapply_primitives_pin_after_dx(
        &context.source_worktree,
        &source_ui_cargo_toml,
        &primitives_pin.git,
        &primitives_pin.rev,
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
        &primitives_pin,
    );
    let snapshot_commit_created =
        commit_vendor_snapshot(&context.vendor_worktree, &commit_message)?;
    push_vendor_branch(&context.repo_root, &args.vendor_branch)?;

    attempt_final_merge(&context.repo_root, &args.vendor_branch)?;

    println!(
        "dx-components vendor completed: target_branch={current_branch}, vendor_branch={}, components_rev={}, primitives_rev={}, components=[{}], snapshot_commit_created={snapshot_commit_created}, vendor_branch_pushed=true",
        args.vendor_branch,
        components_revision,
        primitives_pin.rev,
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
            "working tree must be clean before running dx-components vendor. Found:\n{}",
            status_output
        );
    }
    Ok(())
}

/// Returns the currently checked-out branch name.
fn current_branch(repo_root: &Path) -> Result<String> {
    let branch = git_stdout(repo_root, &["branch", "--show-current"])?;
    if branch.is_empty() {
        bail!("dx-components vendor requires a non-detached current branch");
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
            "could not find git source for dioxus-primitives in Cargo.lock; pass --primitives-rev <rev> to re-pin explicitly"
        )
    })
}

/// Parses the dioxus-primitives git+rev pin from `crates/tasklens-ui/Cargo.toml`.
fn read_manifest_dioxus_primitives_pin(
    cargo_toml_path: &Path,
) -> Result<DioxusVendorRegistryConfig> {
    let content = fs::read_to_string(cargo_toml_path)
        .with_context(|| format!("failed to read {}", cargo_toml_path.display()))?;
    extract_manifest_dioxus_primitives_pin_from_content(&content).with_context(|| {
        format!(
            "failed to parse dioxus-primitives pin from {}",
            cargo_toml_path.display()
        )
    })
}

/// Ensures vendored component code is not pinned ahead of dioxus-primitives.
///
/// We intentionally allow `dioxus-primitives` to be ahead of vendored
/// components, but never the reverse.
fn ensure_components_registry_is_compatible_with_primitives_pin(
    components_registry: &DioxusVendorRegistryConfig,
    primitives_pin: &DioxusVendorRegistryConfig,
    report_context: RevisionReportContext,
) -> Result<()> {
    if components_registry.git != primitives_pin.git {
        bail!(
            "cannot compare revisions across different git sources: components registry is '{}', dioxus-primitives pin is '{}'",
            components_registry.git,
            primitives_pin.git
        );
    }

    ensure_components_revision_not_ahead_of_primitives_revision(
        &components_registry.git,
        &components_registry.rev,
        &primitives_pin.rev,
        report_context,
    )
}

/// Checks upstream for newer revisions and prints revision guidance.
fn check_upstream_revisions(
    components_registry: &DioxusVendorRegistryConfig,
    primitives_pin: &DioxusVendorRegistryConfig,
    report_context: RevisionReportContext,
) -> Result<()> {
    if components_registry.git != primitives_pin.git {
        eprintln!(
            "warning: cannot compare components and dioxus-primitives revisions across different git sources: components='{}', dioxus-primitives='{}'",
            components_registry.git, primitives_pin.git
        );
        return Ok(());
    }

    let comparison_repo = new_temp_worktree_path("dx-revision-check")?;
    fs::create_dir_all(&comparison_repo).with_context(|| {
        format!(
            "failed to create temporary revision-check directory {}",
            comparison_repo.display()
        )
    })?;
    run_command_checked(&comparison_repo, "git", &["init", "--quiet"])?;

    let check_result = emit_registry_revision_report(
        &comparison_repo,
        &components_registry.git,
        &components_registry.rev,
        &primitives_pin.rev,
        report_context,
    );

    if let Err(cleanup_error) = remove_path_if_exists(&comparison_repo) {
        if check_result.is_ok() {
            return Err(cleanup_error);
        }
        eprintln!(
            "warning: failed to clean temporary revision-check directory {}: {cleanup_error:#}",
            comparison_repo.display()
        );
    }

    check_result
}

/// Verifies that `components_revision` is an ancestor (or equal) of `primitives_revision`.
fn ensure_components_revision_not_ahead_of_primitives_revision(
    registry_git: &str,
    components_revision: &str,
    primitives_revision: &str,
    report_context: RevisionReportContext,
) -> Result<()> {
    if components_revision == primitives_revision {
        return Ok(());
    }

    let comparison_repo = new_temp_worktree_path("dx-rev-order-check")?;
    fs::create_dir_all(&comparison_repo).with_context(|| {
        format!(
            "failed to create temporary revision comparison directory {}",
            comparison_repo.display()
        )
    })?;

    let comparison_result = (|| -> Result<()> {
        run_command_checked(&comparison_repo, "git", &["init", "--quiet"])?;
        run_command_checked(
            &comparison_repo,
            "git",
            &[
                "fetch",
                "--quiet",
                "--no-tags",
                registry_git,
                components_revision,
                primitives_revision,
            ],
        )
        .with_context(|| {
            format!(
                "failed to fetch revisions '{components_revision}' and '{primitives_revision}' from '{registry_git}' for ancestry check"
            )
        })?;

        emit_registry_revision_report(
            &comparison_repo,
            registry_git,
            components_revision,
            primitives_revision,
            report_context,
        )?;

        let components_is_ancestor =
            is_ancestor(&comparison_repo, components_revision, primitives_revision)?;
        if components_is_ancestor {
            return Ok(());
        }

        bail!(
            "vendored components revision '{}' is ahead of dioxus-primitives revision '{}'. Use `--primitives-rev <rev>` with '{}' or newer, or vendor an older components revision.",
            components_revision,
            primitives_revision,
            components_revision
        );
    })();

    if let Err(cleanup_error) = remove_path_if_exists(&comparison_repo) {
        if comparison_result.is_ok() {
            return Err(cleanup_error);
        }
        eprintln!(
            "warning: failed to clean temporary revision comparison directory {}: {cleanup_error:#}",
            comparison_repo.display()
        );
    }

    comparison_result
}

/// Relationship between a pinned revision and the upstream default-branch head.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum RevisionHeadStatus {
    At,
    Behind,
    Ahead,
    Diverged,
}

/// Integration status of a vendor snapshot in the current branch.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum IntegrationStatus {
    Integrated,
    NotIntegrated,
}

/// Optional integration context for revision reports.
#[derive(Debug, Clone, Copy, Default)]
struct RevisionReportContext<'a> {
    repo_root: Option<&'a Path>,
    vendor_branch: Option<&'a str>,
}

/// Prints revision status and actionable guidance when newer head pins exist.
fn emit_registry_revision_report(
    comparison_repo: &Path,
    registry_git: &str,
    components_revision: &str,
    primitives_revision: &str,
    report_context: RevisionReportContext,
) -> Result<()> {
    let Some(head_revision) = resolve_remote_head_revision(comparison_repo, registry_git)? else {
        eprintln!(
            "warning: could not determine upstream HEAD for '{registry_git}', so non-head pin guidance is unavailable in this run."
        );
        return Ok(());
    };

    run_command_checked(
        comparison_repo,
        "git",
        &[
            "fetch",
            "--quiet",
            "--no-tags",
            registry_git,
            &head_revision,
        ],
    )
    .with_context(|| {
        format!("failed to fetch upstream HEAD revision '{head_revision}' from '{registry_git}'")
    })?;

    let components_status =
        classify_revision_against_head(comparison_repo, components_revision, &head_revision)?;

    let primitives_status =
        classify_revision_against_head(comparison_repo, primitives_revision, &head_revision)?;

    let components_integration = if let (Some(root), Some(branch)) =
        (report_context.repo_root, report_context.vendor_branch)
    {
        let is_integrated = match find_vendor_snapshot_commit(root, branch, components_revision)? {
            Some(commit) => is_ancestor(root, &commit, "HEAD")?,
            None => false,
        };
        Some(if is_integrated {
            IntegrationStatus::Integrated
        } else {
            IntegrationStatus::NotIntegrated
        })
    } else {
        None
    };

    eprintln!("Dioxus Components revision report:");

    eprintln!(
        "  components pin : {}",
        render_revision_status(
            components_revision,
            components_status,
            components_integration
        ),
    );
    eprintln!(
        "  primitives pin : {}",
        render_revision_status(primitives_revision, primitives_status, None),
    );
    eprintln!("  upstream head  : {head_revision}");

    if primitives_status == RevisionHeadStatus::Behind {
        emit_loud_warning(
            "Dioxus-primitives pin is behind upstream head",
            &format_primitives_head_upgrade_guidance(primitives_revision, &head_revision),
        );
    }

    if components_status == RevisionHeadStatus::Behind {
        emit_loud_warning(
            "Vendored components pin is behind upstream head",
            &format_components_head_upgrade_guidance(
                components_revision,
                &head_revision,
                primitives_status,
            ),
        );
    }

    if components_integration == Some(IntegrationStatus::NotIntegrated) {
        emit_loud_warning(
            "Vendored components are not integrated",
            &format_unintegrated_components_guidance(components_revision),
        );
    }

    Ok(())
}

/// Resolves the latest snapshot commit for a revision on the vendor branch.
fn find_vendor_snapshot_commit(
    repo_root: &Path,
    vendor_branch: &str,
    revision: &str,
) -> Result<Option<String>> {
    let remote_ref = format!("origin/{vendor_branch}");
    if !remote_tracking_branch_exists(repo_root, vendor_branch)? {
        return Ok(None);
    }

    let subject = format!("chore(vendor): update Dioxus Components source to {revision}");
    let output = run_command(
        repo_root,
        "git",
        &[
            "log",
            &remote_ref,
            "--oneline",
            "-n",
            "1",
            "--grep",
            &subject,
            "--format=%H",
        ],
    )?;

    if !output.status.success() {
        anyhow::bail!(
            "git log failed while searching for vendor snapshot for revision {}:\n{}",
            revision,
            String::from_utf8_lossy(&output.stderr)
        );
    }

    let sha = String::from_utf8_lossy(&output.stdout).trim().to_string();
    if sha.is_empty() {
        Ok(None)
    } else {
        Ok(Some(sha))
    }
}

/// Resolves the commit hash for upstream HEAD using `git ls-remote`.
fn resolve_remote_head_revision(
    comparison_repo: &Path,
    registry_git: &str,
) -> Result<Option<String>> {
    let output = run_command(comparison_repo, "git", &["ls-remote", registry_git, "HEAD"])?;
    if !output.status.success() {
        return Ok(None);
    }

    let stdout =
        String::from_utf8(output.stdout).context("git ls-remote stdout was not valid UTF-8")?;
    let head_revision = stdout
        .lines()
        .find_map(|line| line.split_whitespace().next())
        .map(ToString::to_string);

    Ok(head_revision)
}

/// Classifies how `revision` relates to `head_revision`.
fn classify_revision_against_head(
    comparison_repo: &Path,
    revision: &str,
    head_revision: &str,
) -> Result<RevisionHeadStatus> {
    let revision_is_ancestor_of_head = is_ancestor(comparison_repo, revision, head_revision)?;
    let head_is_ancestor_of_revision = is_ancestor(comparison_repo, head_revision, revision)?;
    Ok(classify_revision_head_status(
        revision_is_ancestor_of_head,
        head_is_ancestor_of_revision,
    ))
}

/// Converts revision ancestry booleans to a relation status.
fn classify_revision_head_status(
    revision_is_ancestor_of_head: bool,
    head_is_ancestor_of_revision: bool,
) -> RevisionHeadStatus {
    match (revision_is_ancestor_of_head, head_is_ancestor_of_revision) {
        (true, true) => RevisionHeadStatus::At,
        (true, false) => RevisionHeadStatus::Behind,
        (false, true) => RevisionHeadStatus::Ahead,
        (false, false) => RevisionHeadStatus::Diverged,
    }
}

/// Renders a compact status marker for revision report output.
fn render_revision_status(
    revision: &str,
    status: RevisionHeadStatus,
    integration: Option<IntegrationStatus>,
) -> String {
    let status_suffix = match status {
        RevisionHeadStatus::At => "at-head",
        RevisionHeadStatus::Behind => "behind-head",
        RevisionHeadStatus::Ahead => "ahead-of-head",
        RevisionHeadStatus::Diverged => "diverged-from-head",
    };

    let integration_suffix = match integration {
        Some(IntegrationStatus::Integrated) => ", integrated",
        Some(IntegrationStatus::NotIntegrated) => ", NOT INTEGRATED",
        None => "",
    };

    format!("{revision} ({status_suffix}{integration_suffix})")
}

/// Emits a high-contrast warning header followed by actionable guidance.
fn emit_loud_warning(title: &str, guidance: &str) {
    let warning_style = Style::new().bold().fg_color(Some(AnsiColor::Yellow.into()));
    anstream::eprintln!("");
    anstream::eprintln!("{warning_style}WARNING: {title}{warning_style:#}");
    anstream::eprintln!("{guidance}");
}

/// Explains exact changes required to upgrade dioxus-primitives to head.
fn format_primitives_head_upgrade_guidance(
    primitives_revision: &str,
    head_revision: &str,
) -> String {
    format!(
        "  current pin: {primitives_revision}\n  upstream HEAD: {head_revision}\n  to upgrade dioxus-primitives without re-vendoring:\n  1. Edit crates/tasklens-ui/Cargo.toml and set `dependencies.dioxus-primitives.rev = \"{head_revision}\"`.\n  2. Run `cargo update -p dioxus-primitives` to sync Cargo.lock to the manifest pin."
    )
}

/// Explains exact changes required to vendor components at head.
fn format_components_head_upgrade_guidance(
    components_revision: &str,
    head_revision: &str,
    primitives_status: RevisionHeadStatus,
) -> String {
    if matches!(
        primitives_status,
        RevisionHeadStatus::At | RevisionHeadStatus::Ahead
    ) {
        return format!(
            "  current pin: {components_revision}\n  upstream HEAD: {head_revision}\n  to vendor at upstream HEAD:\n  1. Edit crates/tasklens-ui/dioxus-vendor-components.toml and set `[registry].rev = \"{head_revision}\"`.\n  2. Run `cargo xtask dx-components vendor`."
        );
    }

    format!(
        "  current pin: {components_revision}\n  upstream HEAD: {head_revision}\n  to vendor at upstream HEAD while preserving `components_rev <= dioxus-primitives_rev`:\n  1. Edit crates/tasklens-ui/Cargo.toml and set `dependencies.dioxus-primitives.rev = \"{head_revision}\"`.\n  2. Run `cargo update -p dioxus-primitives` to sync Cargo.lock to the manifest pin.\n  3. Edit crates/tasklens-ui/dioxus-vendor-components.toml and set `[registry].rev = \"{head_revision}\"`.\n  4. Run `cargo xtask dx-components vendor`."
    )
}

/// Explains that the pinned components revision has not been merged into HEAD.
fn format_unintegrated_components_guidance(revision: &str) -> String {
    format!(
        "  current pin: {revision}\n  the vendored source for this revision is not integrated into HEAD history.\n  run `cargo xtask dx-components vendor` to complete the integration."
    )
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
    let has_version = dependency_table_like.get("version").is_some();

    // Avoid rewriting when the pin already matches exactly and no version
    // attribute is present; this prevents formatting-only churn that can
    // cause avoidable merge conflicts.
    if git_is_pinned && rev_is_pinned && !has_version {
        return Ok(content.to_string());
    }

    let mut canonical_table = InlineTable::default();
    canonical_table.insert("git", Value::from(registry_git));
    canonical_table.insert("rev", Value::from(revision));

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

/// Converts a configured UI-relative module path to a repository-relative path.
fn vendored_components_repo_dir_from_module_path(module_path: &str) -> Result<PathBuf> {
    validate_dioxus_vendor_module_path(module_path)?;
    Ok(Path::new(UI_CRATE_DIR).join(module_path))
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
                "vendor branch '{vendor_branch}' has diverged from '{remote_ref}'. local={local_sha}, remote={remote_sha}. Reconcile history manually, then rerun dx-components vendor."
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
        let sample_paths = unexpected_files
            .iter()
            .take(5)
            .map(String::as_str)
            .collect::<Vec<_>>()
            .join(", ");
        eprintln!(
            "warning: strict vendor-branch allowlist check is disabled (--allow-non-orphan-vendor-branch). Continuing even though '{}' tracks non-snapshot files (examples: {}). These files can be carried forward by merge and may blur vendor snapshot provenance.",
            git_stdout(vendor_worktree, &["branch", "--show-current"])
                .unwrap_or_else(|_| "<unknown-branch>".to_string()),
            sample_paths,
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

    if path == Path::new(DEFAULT_DIOXUS_VENDOR_COMPONENTS_FILE) {
        return true;
    }

    path.starts_with(vendored_components_dir) || path.starts_with("crates/tasklens-ui/assets")
}

/// Executes the in-repo dx-components installer in the source worktree UI crate.
fn run_dx_components_vendor(
    source_worktree: &Path,
    module_path: &str,
    components: &[String],
    registry: RegistrySpec,
) -> Result<()> {
    let ui_dir = source_worktree.join(UI_CRATE_DIR);
    installer::install_components_from_registry(
        source_worktree,
        &ui_dir,
        module_path,
        &registry,
        components,
    )
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
    components_config: &DioxusVendorComponentsConfig,
    primitives_pin: &DioxusVendorRegistryConfig,
) -> VendorSnapshotCommitMessage {
    let components_revision = components_config.registry.rev.as_str();
    let subject =
        format!("chore(vendor): update Dioxus Components source to {components_revision}");
    let canonical_invocation = render_canonical_vendor_invocation(args, component_file_relative);
    let observed_invocation = render_observed_invocation();
    let installer_invocation = render_internal_installer_invocation(
        &components_config.registry.git,
        &components_config.module_path,
        &components_config.components,
        components_revision,
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
- Tool: cargo xtask dx-components vendor
- Canonical invocation: {canonical_invocation}
- Observed argv: {observed_invocation}
- Installer invocation (in {UI_CRATE_DIR}): {installer_invocation}

Snapshot inputs:
- Components registry: {components_registry_git}
- Components revision: {components_revision}
- dioxus-primitives git pin: {primitives_git}
- dioxus-primitives revision pin: {primitives_revision}
- Components file: {}
- Components: {}",
        path_to_string(component_file_relative),
        components_config.components.join(", "),
        components_registry_git = components_config.registry.git,
        primitives_git = primitives_pin.git,
        primitives_revision = primitives_pin.rev,
        vendor_branch = args.vendor_branch,
    );

    VendorSnapshotCommitMessage { subject, body }
}

/// Renders the current process argv for provenance logging.
fn render_observed_invocation() -> String {
    std::env::args().collect::<Vec<_>>().join(" ")
}

/// Renders an equivalent cargo invocation from resolved xtask options.
fn render_canonical_vendor_invocation(
    args: &UpdateDioxusComponentsArgs,
    component_file_relative: &Path,
) -> String {
    let mut command_args = vec![
        "xtask".to_string(),
        "dx-components".to_string(),
        "vendor".to_string(),
    ];

    if let Some(revision) = args.primitives_rev.as_ref() {
        command_args.push("--primitives-rev".to_string());
        command_args.push(revision.clone());
    }

    if args.check_revisions {
        command_args.push("--check-revisions".to_string());
    }

    if args.vendor_branch != DEFAULT_VENDOR_BRANCH {
        command_args.push("--vendor-branch".to_string());
        command_args.push(args.vendor_branch.clone());
    }

    if component_file_relative != Path::new(DEFAULT_DIOXUS_VENDOR_COMPONENTS_FILE) {
        command_args.push("--components-file".to_string());
        command_args.push(path_to_string(component_file_relative));
    }

    if args.keep_temp {
        command_args.push("--keep-temp".to_string());
    }

    if args.allow_non_orphan_vendor_branch {
        command_args.push("--allow-non-orphan-vendor-branch".to_string());
    }

    render_command("cargo", &command_args)
}

/// Renders the equivalent internal installer invocation for provenance logging.
fn render_internal_installer_invocation(
    registry_git: &str,
    module_path: &str,
    components: &[String],
    revision: &str,
) -> String {
    let mut args = vec![
        "install".to_string(),
        "--module-path".to_string(),
        module_path.to_string(),
        "--git".to_string(),
        registry_git.to_string(),
        "--rev".to_string(),
        revision.to_string(),
        "--force".to_string(),
    ];
    args.extend(components.iter().cloned());

    render_command("internal dx-components-installer", &args)
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
    fn rejects_components_and_primitives_git_mismatch() {
        let components_registry = DioxusVendorRegistryConfig {
            git: "https://github.com/DioxusLabs/components".to_string(),
            rev: "abc123".to_string(),
        };
        let primitives_pin = DioxusVendorRegistryConfig {
            git: "https://github.com/example/other-components".to_string(),
            rev: "abc123".to_string(),
        };

        assert!(
            ensure_components_registry_is_compatible_with_primitives_pin(
                &components_registry,
                &primitives_pin,
                RevisionReportContext::default(),
            )
            .is_err()
        );
    }

    #[test]
    fn allows_equal_components_and_primitives_revisions() {
        assert!(
            ensure_components_revision_not_ahead_of_primitives_revision(
                "https://github.com/DioxusLabs/components",
                "abc123",
                "abc123",
                RevisionReportContext::default(),
            )
            .is_ok()
        );
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
        assert!(!updated.contains("version = \"0.0.1\""));
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
            "dioxus-primitives = { git = \"https://github.com/DioxusLabs/components\", rev = \"abc123\", default-features = false, features = [\"router\"] }"
        ));
    }

    #[test]
    fn cargo_toml_pin_noop_when_already_pinned() {
        let input = r#"
[dependencies]
dioxus-primitives = { git = "https://github.com/DioxusLabs/components", rev = "abc123", default-features = false, features = ["router"] }
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
    fn renders_canonical_vendor_invocation_with_overrides() {
        let args = UpdateDioxusComponentsArgs {
            vendor_branch: "vendor/custom".to_string(),
            components_file: PathBuf::from("crates/tasklens-ui/custom-components.toml"),
            primitives_rev: Some("deadbeef".to_string()),
            check_revisions: false,
            keep_temp: true,
            allow_non_orphan_vendor_branch: true,
        };

        let rendered = render_canonical_vendor_invocation(
            &args,
            Path::new("crates/tasklens-ui/custom-components.toml"),
        );

        assert!(rendered.contains("cargo xtask dx-components vendor"));
        assert!(rendered.contains("--vendor-branch vendor/custom"));
        assert!(rendered.contains("--components-file crates/tasklens-ui/custom-components.toml"));
        assert!(rendered.contains("--primitives-rev deadbeef"));
        assert!(rendered.contains("--keep-temp"));
        assert!(rendered.contains("--allow-non-orphan-vendor-branch"));
    }

    #[test]
    fn renders_canonical_vendor_invocation_with_revision_check_mode() {
        let args = UpdateDioxusComponentsArgs {
            vendor_branch: DEFAULT_VENDOR_BRANCH.to_string(),
            components_file: PathBuf::from(DEFAULT_DIOXUS_VENDOR_COMPONENTS_FILE),
            primitives_rev: None,
            check_revisions: true,
            keep_temp: false,
            allow_non_orphan_vendor_branch: false,
        };

        let rendered = render_canonical_vendor_invocation(
            &args,
            Path::new(DEFAULT_DIOXUS_VENDOR_COMPONENTS_FILE),
        );

        assert!(rendered.contains("--check-revisions"));
    }

    #[test]
    fn rejects_check_revisions_when_primitives_rev_is_set() {
        let args = UpdateDioxusComponentsArgs {
            vendor_branch: DEFAULT_VENDOR_BRANCH.to_string(),
            components_file: PathBuf::from(DEFAULT_DIOXUS_VENDOR_COMPONENTS_FILE),
            primitives_rev: Some("abc123".to_string()),
            check_revisions: true,
            keep_temp: false,
            allow_non_orphan_vendor_branch: false,
        };

        assert!(check_revisions_only(&args).is_err());
    }

    #[test]
    fn builds_vendor_snapshot_commit_message_with_provenance_fields() {
        let args = UpdateDioxusComponentsArgs {
            vendor_branch: DEFAULT_VENDOR_BRANCH.to_string(),
            components_file: PathBuf::from(DEFAULT_DIOXUS_VENDOR_COMPONENTS_FILE),
            primitives_rev: None,
            check_revisions: false,
            keep_temp: false,
            allow_non_orphan_vendor_branch: false,
        };
        let components_config = DioxusVendorComponentsConfig {
            registry: DioxusVendorRegistryConfig {
                git: "https://github.com/DioxusLabs/components".to_string(),
                rev: "abc123".to_string(),
            },
            module_path: "src/dioxus_components".to_string(),
            components: vec!["button".to_string(), "dialog".to_string()],
        };
        let primitives_pin = DioxusVendorRegistryConfig {
            git: "https://github.com/DioxusLabs/components".to_string(),
            rev: "def456".to_string(),
        };

        let message = build_vendor_snapshot_commit_message(
            &args,
            "docs/dioxus-migration",
            Path::new(DEFAULT_DIOXUS_VENDOR_COMPONENTS_FILE),
            &components_config,
            &primitives_pin,
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
                .contains("Canonical invocation: cargo xtask dx-components vendor")
        );
        assert!(message.body.contains("Observed argv: "));
        assert!(message.body.contains("Components revision: abc123"));
        assert!(
            message
                .body
                .contains("dioxus-primitives revision pin: def456")
        );
        assert!(message.body.contains("internal dx-components-installer install --module-path src/dioxus_components --git https://github.com/DioxusLabs/components --rev abc123 --force button dialog"));
    }
}
