pub(crate) mod commands;

use anyhow::Result;
use clap::{Parser, Subcommand};

use commands::{check_catalog::check_catalog, lint_filenames::lint_filenames};

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
    /// Lint filenames for naming conventions
    LintFilenames,
    /// Run all checks
    CheckAll,
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    match &cli.command {
        Commands::CheckCatalog => check_catalog()?,
        Commands::LintFilenames => lint_filenames()?,
        Commands::CheckAll => {
            check_catalog()?;
            lint_filenames()?;
        }
    }

    Ok(())
}
