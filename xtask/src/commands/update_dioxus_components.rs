use anyhow::Result;

pub(crate) use crate::commands::dx_components::vendor::UpdateDioxusComponentsArgs;
use crate::commands::dx_components::vendor::update_dioxus_components as run_vendor;

/// Compatibility alias for `cargo xtask update-dioxus-components`.
///
/// This forwards to `cargo xtask dx-components vendor` and prints a deprecation warning.
pub(crate) fn update_dioxus_components(args: &UpdateDioxusComponentsArgs) -> Result<()> {
    eprintln!(
        "warning: `cargo xtask update-dioxus-components` is deprecated; use `cargo xtask dx-components vendor` instead"
    );
    run_vendor(args)
}
