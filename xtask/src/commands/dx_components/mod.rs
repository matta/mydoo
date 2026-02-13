pub(crate) mod installer;
pub(crate) mod manifest;
pub(crate) mod registry;
pub(crate) mod vendor;

use std::path::PathBuf;

use anyhow::Result;
use clap::{Args, Subcommand};

use crate::commands::dioxus_info::DEFAULT_DIOXUS_VENDOR_COMPONENTS_FILE;

/// Top-level namespace for Dioxus component acquisition and registry operations.
#[derive(Args, Debug)]
pub(crate) struct DxComponentsArgs {
    #[command(subcommand)]
    command: DxComponentsSubcommand,
}

/// Subcommands under `cargo xtask dx-components`.
#[derive(Subcommand, Debug)]
pub(crate) enum DxComponentsSubcommand {
    /// Vendor components into the UI crate using pinned registry configuration.
    Vendor(vendor::UpdateDioxusComponentsArgs),
    /// List components available in the configured pinned registry.
    List(DxComponentsConfigArgs),
    /// Refresh local cache for the configured pinned registry.
    Update(DxComponentsConfigArgs),
    /// Remove cached dx-components registry checkouts from git common-dir.
    Clean,
    /// Print component manifest schema JSON.
    Schema,
}

/// Common config file argument for registry-oriented commands.
#[derive(Args, Debug)]
pub(crate) struct DxComponentsConfigArgs {
    /// Path to the TOML file listing components and registry pin.
    #[arg(long, default_value = DEFAULT_DIOXUS_VENDOR_COMPONENTS_FILE)]
    components_file: PathBuf,
}

/// Executes the selected `dx-components` subcommand.
pub(crate) fn run(args: &DxComponentsArgs) -> Result<()> {
    match &args.command {
        DxComponentsSubcommand::Vendor(vendor_args) => {
            vendor::update_dioxus_components(vendor_args)
        }
        DxComponentsSubcommand::List(list_args) => {
            registry::list_components(&list_args.components_file)
        }
        DxComponentsSubcommand::Update(update_args) => {
            registry::update_registry(&update_args.components_file)
        }
        DxComponentsSubcommand::Clean => registry::clean_registry_cache(),
        DxComponentsSubcommand::Schema => {
            println!("{}", manifest::render_component_manifest_schema_pretty()?);
            Ok(())
        }
    }
}

#[cfg(test)]
mod tests {
    use clap::Parser;

    use super::*;

    #[derive(Parser)]
    struct TestCli {
        #[command(subcommand)]
        command: DxComponentsSubcommand,
    }

    #[test]
    fn parses_vendor_subcommand() {
        let cli = TestCli::parse_from([
            "xtask",
            "vendor",
            "--vendor-branch",
            "vendor/custom",
            "--components-file",
            "crates/tasklens-ui/dioxus-vendor-components.toml",
        ]);

        match cli.command {
            DxComponentsSubcommand::Vendor(_) => {}
            _ => panic!("expected vendor subcommand"),
        }
    }

    #[test]
    fn parses_list_subcommand() {
        let cli = TestCli::parse_from([
            "xtask",
            "list",
            "--components-file",
            "crates/tasklens-ui/dioxus-vendor-components.toml",
        ]);

        match cli.command {
            DxComponentsSubcommand::List(_) => {}
            _ => panic!("expected list subcommand"),
        }
    }
}
