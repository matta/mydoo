pub(crate) mod commands;

use anyhow::Result;
use clap::{Parser, Subcommand};

use commands::{
    check_biome_schema::check_biome_schema, check_catalog::check_catalog,
    lint_context::lint_context, lint_filenames::lint_filenames,
};

#[derive(Parser)]
#[command(name = "xtask")]
#[command(about = "Development automation scripts", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Check for unused entries in the pnpm-workspace.yaml catalog
    CheckCatalog,
    /// Check that biome.json $schema version matches installed Biome version
    CheckBiomeSchema,
    /// Lint filenames for naming conventions
    LintFilenames,
    /// Lint context directory for unauthorized files
    LintContext,
    /// Run all checks
    CheckAll,
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    match &cli.command {
        Commands::CheckCatalog => check_catalog()?,
        Commands::CheckBiomeSchema => check_biome_schema()?,
        Commands::LintFilenames => lint_filenames()?,
        Commands::LintContext => lint_context()?,
        Commands::CheckAll => {
            check_catalog()?;
            check_biome_schema()?;
            lint_filenames()?;
            lint_context()?;
        }
    }

    Ok(())
}
