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
    /// Update vendored Dioxus components via a pristine vendor branch workflow
    UpdateDioxusComponents(UpdateDioxusComponentsArgs),
    /// Run all checks
    CheckAll,
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
        Commands::UpdateDioxusComponents(args) => update_dioxus_components(args)?,
    }

    Ok(())
}
