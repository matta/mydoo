// Derived in part from Dioxus upstream `dioxus-cli` installer code:
// - context/dioxus/packages/cli/src/cli/component.rs
// - context/dioxus/packages/cli/src/workspace.rs
// Upstream package metadata:
// - authors = ["Jonathan Kelley"]
// - license = "MIT OR Apache-2.0"
// See `UPSTREAM_ATTRIBUTION.md` in this directory for provenance and licensing details.

use std::collections::{HashMap, HashSet, VecDeque};
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

use anyhow::{Context, Result, bail};

use super::manifest::{CargoDependency, Component, ComponentDependency};
use super::registry::{RegistrySpec, resolve_registry_checkout};

/// Lightweight component metadata for CLI listing output.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct ComponentSummary {
    pub(crate) name: String,
    pub(crate) description: String,
}

/// Internal component with resolved filesystem location and registry provenance.
#[derive(Clone, Debug)]
struct ResolvedComponent {
    path: PathBuf,
    component: Component,
    registry_root: PathBuf,
    registry: RegistrySpec,
}

/// Install behavior for a resolved component when destination paths already exist.
#[derive(Clone, Copy, Debug)]
enum ComponentExistsBehavior {
    /// Keep existing destination and skip copy.
    Return,
    /// Remove and overwrite destination.
    Overwrite,
}

/// Planned install unit with source component and destination behavior.
#[derive(Clone, Debug)]
struct PlannedComponent {
    component: ResolvedComponent,
    behavior: ComponentExistsBehavior,
}

/// Stable identity for deduplicating component installs across registries.
#[derive(Clone, Debug, Hash, PartialEq, Eq)]
struct ComponentKey {
    registry_git: String,
    registry_rev: String,
    component_name: String,
}

/// Registry inventory cache used during dependency closure resolution.
#[derive(Clone, Debug)]
struct RegistryInventory {
    components: Vec<ResolvedComponent>,
}

/// Discovers components available in a registry checkout for list-style operations.
pub(crate) fn discover_registry_components(
    registry_root: &Path,
    registry: &RegistrySpec,
) -> Result<Vec<ComponentSummary>> {
    let components = discover_components(registry_root, registry)?;
    Ok(components
        .into_iter()
        .filter(|component| component.component.members.is_empty())
        .map(|component| ComponentSummary {
            name: component.component.name,
            description: component.component.description,
        })
        .collect())
}

/// Installs requested components and dependencies into a UI crate module path.
pub(crate) fn install_components_from_registry(
    repo_root: &Path,
    ui_crate_root: &Path,
    module_path: &str,
    root_registry: &RegistrySpec,
    requested_components: &[String],
) -> Result<()> {
    if requested_components.is_empty() {
        bail!("at least one component must be requested");
    }

    let mut registry_cache = HashMap::new();
    let root_inventory = load_registry_inventory(repo_root, root_registry, &mut registry_cache)?;

    let mut planned = Vec::new();
    let mut queued = VecDeque::new();
    let mut seen = HashSet::new();

    for name in requested_components {
        let component = find_component(&root_inventory.components, name)?;
        let key = key_for_component(&component);
        if seen.insert(key) {
            planned.push(PlannedComponent {
                component: component.clone(),
                behavior: ComponentExistsBehavior::Overwrite,
            });
            queued.push_back(component);
        }
    }

    while let Some(component) = queued.pop_front() {
        for dependency in &component.component.component_dependencies {
            let dependency_component = resolve_dependency_component(
                repo_root,
                &component,
                dependency,
                &mut registry_cache,
            )?;
            let key = key_for_component(&dependency_component);
            if seen.insert(key) {
                planned.push(PlannedComponent {
                    component: dependency_component.clone(),
                    behavior: ComponentExistsBehavior::Return,
                });
                queued.push_back(dependency_component);
            }
        }
    }

    let mut cargo_dependencies = HashSet::new();
    for planned_component in &planned {
        cargo_dependencies.extend(
            planned_component
                .component
                .component
                .cargo_dependencies
                .iter()
                .cloned(),
        );
    }

    let mut cargo_dependencies = cargo_dependencies.into_iter().collect::<Vec<_>>();
    cargo_dependencies.sort_by(|left, right| left.name().cmp(right.name()));
    add_rust_dependencies(ui_crate_root, &cargo_dependencies)?;

    let components_root = ui_crate_root.join(module_path);
    ensure_components_module_exists(&components_root)?;
    let assets_root = ui_crate_root.join("assets");

    for planned_component in planned {
        add_component(&components_root, &assets_root, &planned_component)?;
    }

    Ok(())
}

/// Resolves registry inventory from cache or by reading manifests from checkout.
fn load_registry_inventory(
    repo_root: &Path,
    registry: &RegistrySpec,
    cache: &mut HashMap<RegistrySpec, RegistryInventory>,
) -> Result<RegistryInventory> {
    if let Some(existing) = cache.get(registry) {
        return Ok(existing.clone());
    }

    let registry_root = resolve_registry_checkout(repo_root, registry, false)?;
    let inventory = RegistryInventory {
        components: discover_components(&registry_root, registry)?,
    };
    cache.insert(registry.clone(), inventory.clone());
    Ok(inventory)
}

/// Resolves one dependency component and returns its fully resolved source metadata.
fn resolve_dependency_component(
    repo_root: &Path,
    parent: &ResolvedComponent,
    dependency: &ComponentDependency,
    cache: &mut HashMap<RegistrySpec, RegistryInventory>,
) -> Result<ResolvedComponent> {
    let (registry, name) = dependency_registry_and_name(parent, dependency)?;
    let inventory = load_registry_inventory(repo_root, &registry, cache)?;
    find_component(&inventory.components, &name)
}

/// Determines dependency registry selector and requested component name.
fn dependency_registry_and_name(
    parent: &ResolvedComponent,
    dependency: &ComponentDependency,
) -> Result<(RegistrySpec, String)> {
    match dependency {
        ComponentDependency::Builtin(name) => Ok((parent.registry.clone(), name.clone())),
        ComponentDependency::ThirdParty { name, git, rev } => {
            let revision = rev.as_deref().ok_or_else(|| {
                anyhow::anyhow!(
                    "third-party dependency '{}' in component '{}' is missing `rev`; deterministic vendoring requires an explicit revision",
                    name,
                    parent.component.name
                )
            })?;
            Ok((
                RegistrySpec {
                    git: git.clone(),
                    rev: revision.to_string(),
                },
                name.clone(),
            ))
        }
    }
}

/// Returns a component identity key used to deduplicate install operations.
fn key_for_component(component: &ResolvedComponent) -> ComponentKey {
    ComponentKey {
        registry_git: component.registry.git.clone(),
        registry_rev: component.registry.rev.clone(),
        component_name: component.component.name.clone(),
    }
}

/// Finds a component by name in a discovered registry inventory.
fn find_component(components: &[ResolvedComponent], name: &str) -> Result<ResolvedComponent> {
    components
        .iter()
        .find(|component| component.component.name == name)
        .cloned()
        .ok_or_else(|| anyhow::anyhow!("component '{}' not found in registry", name))
}

/// Adds a single component by copying files, assets, and module registration.
fn add_component(
    components_root: &Path,
    assets_root: &Path,
    planned_component: &PlannedComponent,
) -> Result<()> {
    let component = &planned_component.component;
    let destination = components_root.join(&component.component.name);

    let copied = copy_component_files(
        &component.path,
        &destination,
        &component.component.exclude,
        planned_component.behavior,
    )?;
    if !copied {
        return Ok(());
    }

    copy_global_assets(
        &component.registry_root,
        assets_root,
        &component.component,
        &component.path,
    )?;

    add_component_module_line(components_root, &component.component.name)
}

/// Adds missing `pub mod` line for a component in the target module tree.
fn add_component_module_line(components_root: &Path, component_name: &str) -> Result<()> {
    let mod_rs_path = components_root.join("mod.rs");
    let mut content = fs::read_to_string(&mod_rs_path)
        .with_context(|| format!("failed to read {}", mod_rs_path.display()))?;

    let mod_line = format!("pub mod {};", component_name);
    if content.lines().any(|line| line.trim() == mod_line) {
        return Ok(());
    }

    if !content.ends_with('\n') {
        content.push('\n');
    }
    content.push_str(&format!("{mod_line}\n"));

    fs::write(&mod_rs_path, content)
        .with_context(|| format!("failed to write {}", mod_rs_path.display()))
}

/// Copies component files recursively while applying manifest exclusion rules.
fn copy_component_files(
    src: &Path,
    dest: &Path,
    exclude: &[String],
    behavior: ComponentExistsBehavior,
) -> Result<bool> {
    if dest.exists() {
        match behavior {
            ComponentExistsBehavior::Return => {
                return Ok(false);
            }
            ComponentExistsBehavior::Overwrite => {
                fs::remove_dir_all(dest)
                    .with_context(|| format!("failed to remove {}", dest.display()))?;
            }
        }
    }

    fs::create_dir_all(dest).with_context(|| format!("failed to create {}", dest.display()))?;

    let canonical_src = fs::canonicalize(src)
        .with_context(|| format!("failed to canonicalize {}", src.display()))?;
    let excluded_paths = exclude
        .iter()
        .map(|excluded| {
            fs::canonicalize(canonical_src.join(excluded)).with_context(|| {
                format!(
                    "failed to resolve excluded path '{}' in component source {}",
                    excluded,
                    canonical_src.display()
                )
            })
        })
        .collect::<Result<Vec<_>>>()?;

    let mut pending = vec![canonical_src.clone()];
    while let Some(current_dir) = pending.pop() {
        for entry in fs::read_dir(&current_dir)
            .with_context(|| format!("failed to read {}", current_dir.display()))?
        {
            let entry = entry.with_context(|| {
                format!(
                    "failed to read directory entry under {}",
                    current_dir.display()
                )
            })?;
            let entry_path = fs::canonicalize(entry.path())
                .with_context(|| format!("failed to canonicalize {}", entry.path().display()))?;

            if excluded_paths
                .iter()
                .any(|excluded| entry_path == *excluded || entry_path.starts_with(excluded))
            {
                continue;
            }

            let relative = entry_path.strip_prefix(&canonical_src).with_context(|| {
                format!(
                    "path '{}' is outside canonical source '{}'",
                    entry_path.display(),
                    canonical_src.display()
                )
            })?;
            let destination_path = dest.join(relative);

            if entry_path.is_dir() {
                fs::create_dir_all(&destination_path).with_context(|| {
                    format!("failed to create directory {}", destination_path.display())
                })?;
                pending.push(entry_path);
                continue;
            }

            if let Some(parent) = destination_path.parent() {
                fs::create_dir_all(parent)
                    .with_context(|| format!("failed to create {}", parent.display()))?;
            }
            fs::copy(&entry_path, &destination_path).with_context(|| {
                format!(
                    "failed to copy component file from {} to {}",
                    entry_path.display(),
                    destination_path.display()
                )
            })?;
        }
    }

    Ok(true)
}

/// Copies declared global assets while enforcing per-component registry boundaries.
fn copy_global_assets(
    component_registry_root: &Path,
    assets_root: &Path,
    component: &Component,
    component_root: &Path,
) -> Result<()> {
    let canonical_registry_root = fs::canonicalize(component_registry_root).with_context(|| {
        format!(
            "failed to canonicalize component registry root {}",
            component_registry_root.display()
        )
    })?;

    for global_asset in &component.global_assets {
        let source = component_root.join(global_asset);
        let absolute_source = fs::canonicalize(&source).with_context(|| {
            format!(
                "failed to find global asset '{}' for component '{}'",
                source.display(),
                component.name
            )
        })?;

        if !asset_is_within_registry_root(&absolute_source, &canonical_registry_root) {
            bail!(
                "cannot copy global asset '{}' for component '{}' because it is outside component registry root '{}'",
                absolute_source.display(),
                component.name,
                canonical_registry_root.display()
            );
        }

        let file_name = absolute_source.file_name().ok_or_else(|| {
            anyhow::anyhow!(
                "global asset path '{}' has no terminal file name",
                absolute_source.display()
            )
        })?;

        let destination = assets_root.join(file_name);
        if let Some(parent) = destination.parent() {
            fs::create_dir_all(parent)
                .with_context(|| format!("failed to create {}", parent.display()))?;
        }

        fs::copy(&source, &destination).with_context(|| {
            format!(
                "failed to copy global asset from {} to {}",
                source.display(),
                destination.display()
            )
        })?;
    }

    Ok(())
}

/// Returns true if a resolved asset path is inside the component's registry root.
fn asset_is_within_registry_root(asset_path: &Path, registry_root: &Path) -> bool {
    asset_path.starts_with(registry_root)
}

/// Ensures the component module directory and `mod.rs` file exist.
fn ensure_components_module_exists(components_root: &Path) -> Result<()> {
    if !components_root.exists() {
        fs::create_dir_all(components_root)
            .with_context(|| format!("failed to create {}", components_root.display()))?;
    }

    let mod_rs_path = components_root.join("mod.rs");
    if !mod_rs_path.exists() {
        fs::write(&mod_rs_path, "// AUTOGENERATED Components module\n")
            .with_context(|| format!("failed to create {}", mod_rs_path.display()))?;
    }

    Ok(())
}

/// Runs `cargo add` for required Rust dependencies.
fn add_rust_dependencies(ui_crate_root: &Path, dependencies: &[CargoDependency]) -> Result<()> {
    for dependency in dependencies {
        let args = dependency.add_command_args();
        let output = Command::new("cargo")
            .current_dir(ui_crate_root)
            .args(&args)
            .output()
            .with_context(|| {
                format!(
                    "failed to execute `cargo {}` for dependency '{}'",
                    args.join(" "),
                    dependency.name()
                )
            })?;

        if !output.status.success() {
            bail!(
                "failed to add Rust dependency '{}' via `cargo {}`\nstdout:\n{}\nstderr:\n{}",
                dependency.name(),
                args.join(" "),
                String::from_utf8_lossy(&output.stdout),
                String::from_utf8_lossy(&output.stderr)
            );
        }
    }

    Ok(())
}

/// Discovers all registry components starting from root manifest and member graph.
fn discover_components(
    registry_root: &Path,
    registry: &RegistrySpec,
) -> Result<Vec<ResolvedComponent>> {
    let root = read_component(registry_root, registry_root, registry)?;
    let mut components = vec![root.clone()];
    let mut queued = root.member_paths();

    while let Some(member_path) = queued.pop() {
        let component = read_component(&member_path, registry_root, registry)?;
        queued.extend(component.member_paths());
        components.push(component);
    }

    Ok(components)
}

impl ResolvedComponent {
    /// Returns absolute paths for component members declared in manifest.
    fn member_paths(&self) -> Vec<PathBuf> {
        self.component
            .members
            .iter()
            .map(|member| self.path.join(member))
            .collect()
    }
}

/// Reads and parses one component manifest rooted at `path/component.json`.
fn read_component(
    path: &Path,
    registry_root: &Path,
    registry: &RegistrySpec,
) -> Result<ResolvedComponent> {
    let json_path = path.join("component.json");
    let bytes = fs::read(&json_path)
        .with_context(|| format!("failed to read component manifest {}", json_path.display()))?;
    let component = serde_json::from_slice::<Component>(&bytes).with_context(|| {
        format!(
            "failed to parse component manifest JSON at {}",
            json_path.display()
        )
    })?;

    Ok(ResolvedComponent {
        path: fs::canonicalize(path)
            .with_context(|| format!("failed to canonicalize {}", path.display()))?,
        component,
        registry_root: fs::canonicalize(registry_root)
            .with_context(|| format!("failed to canonicalize {}", registry_root.display()))?,
        registry: registry.clone(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn builtin_dependency_uses_parent_registry() {
        let parent = ResolvedComponent {
            path: PathBuf::from("/tmp/component"),
            component: Component {
                name: "date_picker".to_string(),
                description: String::new(),
                authors: Vec::new(),
                component_dependencies: Vec::new(),
                cargo_dependencies: Vec::new(),
                members: Vec::new(),
                exclude: Vec::new(),
                global_assets: Vec::new(),
            },
            registry_root: PathBuf::from("/tmp/registry-a"),
            registry: RegistrySpec {
                git: "https://example.com/registry-a.git".to_string(),
                rev: "abc123".to_string(),
            },
        };

        let (registry, name) = dependency_registry_and_name(
            &parent,
            &ComponentDependency::Builtin("calendar".to_string()),
        )
        .unwrap();

        assert_eq!(name, "calendar");
        assert_eq!(registry, parent.registry);
    }

    #[test]
    fn third_party_dependency_requires_revision() {
        let parent = ResolvedComponent {
            path: PathBuf::from("/tmp/component"),
            component: Component {
                name: "date_picker".to_string(),
                description: String::new(),
                authors: Vec::new(),
                component_dependencies: Vec::new(),
                cargo_dependencies: Vec::new(),
                members: Vec::new(),
                exclude: Vec::new(),
                global_assets: Vec::new(),
            },
            registry_root: PathBuf::from("/tmp/registry-a"),
            registry: RegistrySpec {
                git: "https://example.com/registry-a.git".to_string(),
                rev: "abc123".to_string(),
            },
        };

        let result = dependency_registry_and_name(
            &parent,
            &ComponentDependency::ThirdParty {
                name: "popover".to_string(),
                git: "https://example.com/registry-b.git".to_string(),
                rev: None,
            },
        );

        assert!(result.is_err());
    }

    #[test]
    fn validates_asset_with_component_registry_root() {
        let registry_root = Path::new("/tmp/registry");
        let inside = Path::new("/tmp/registry/components/calendar/style.css");
        let outside = Path::new("/tmp/other/style.css");

        assert!(asset_is_within_registry_root(inside, registry_root));
        assert!(!asset_is_within_registry_root(outside, registry_root));
    }
}
