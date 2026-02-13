pub(crate) mod commands;

use anyhow::Result;
use clap::{Parser, Subcommand};

use commands::{
    check_biome_schema::check_biome_schema,
    check_catalog::check_catalog,
    check_context::check_context,
    check_dark_mode::check_dark_mode,
    check_dioxus_lock_pin::check_dioxus_lock_pin,
    check_filenames::check_filenames,
    dx_components,
    fix_junit::{self, FixJunitArgs},
    update_dioxus_components::{UpdateDioxusComponentsArgs, update_dioxus_components},
};

#[derive(Parser)]
#[command(name = "xtask")]
#[command(about = "Development automation scripts", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
#[allow(clippy::enum_variant_names)]
enum Commands {
    /// Check for unused entries in the pnpm-workspace.yaml catalog
    CheckCatalog,
    /// Check that biome.json $schema version matches installed Biome version
    CheckBiomeSchema,
    /// Check filenames for naming conventions
    CheckFilenames,
    /// Check context directory for unauthorized files
    CheckContext,
    /// Check for dark mode violations in UI components
    CheckDarkMode,
    /// Check that dioxus-primitives rev pin matches Cargo.lock resolution
    CheckDioxusLockPin,
    /// Manage vendored Dioxus Components and registry cache via in-repo installer
    DxComponents(dx_components::DxComponentsArgs),
    /// Update vendored Dioxus components via a pristine vendor branch workflow
    ///
    /// Deprecated compatibility alias for `dx-components vendor`.
    UpdateDioxusComponents(UpdateDioxusComponentsArgs),
    /// Run all checks
    CheckAll,
    /// Fix JUnit XML report for Trunk.io compatibility
    FixJunit(FixJunitArgs),
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    match &cli.command {
        Commands::CheckAll => {
            check_catalog()?;
            check_biome_schema()?;
            check_filenames()?;
            check_context()?;
            check_dark_mode()?;
            check_dioxus_lock_pin()?;
        }
        Commands::CheckBiomeSchema => check_biome_schema()?,
        Commands::CheckCatalog => check_catalog()?,
        Commands::CheckContext => check_context()?,
        Commands::CheckDarkMode => check_dark_mode()?,
        Commands::CheckDioxusLockPin => check_dioxus_lock_pin()?,
        Commands::CheckFilenames => check_filenames()?,
        Commands::DxComponents(args) => dx_components::run(args)?,
        Commands::FixJunit(args) => fix_junit::run(FixJunitArgs {
            junit_path: args.junit_path.clone(),
            package_dir: args.package_dir.clone(),
        })?,
        Commands::UpdateDioxusComponents(args) => update_dioxus_components(args)?,
    }

    Ok(())
}
