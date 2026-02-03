use dioxus::prelude::*;
use dioxus_primitives::dialog::{
    self, DialogContentProps, DialogDescriptionProps, DialogRootProps, DialogTitleProps,
};

#[component]
pub fn DialogRoot(props: DialogRootProps) -> Element {
    // DaisyUI: 'modal-open' is the utility to force the modal to show.
    // We conditionally apply it based on the `open` prop.
    // Fix: dereference the signal to get the boolean value.
    let is_open = props.open.read().unwrap_or(false);
    let open_class = if is_open { "modal-open" } else { "" };
    rsx! {
        dialog::DialogRoot {
            // DaisyUI: 'modal' provides the backdrop and container.
            class: "modal {open_class}",
            id: props.id,
            is_modal: props.is_modal,
            open: props.open,
            default_open: props.default_open,
            on_open_change: props.on_open_change,
            attributes: props.attributes,
            {props.children}
        }
    }
}

#[component]
pub fn DialogContent(props: DialogContentProps) -> Element {
    rsx! {
        dialog::DialogContent {
            // DaisyUI: 'modal-box' provides the card styling.
            class: if let Some(c) = &props.class {
                format!("modal-box {}", c)
            } else {
                "modal-box".to_string()
            },
            id: props.id,
            attributes: props.attributes,
            {props.children}
        }
    }
}

#[component]
pub fn DialogTitle(props: DialogTitleProps) -> Element {
    rsx! {
        dialog::DialogTitle {
            // Tailwind: 'font-bold text-lg'. Justification: DaisyUI's 'modal-box' does not strictly enforce title typography.
            // These utilities match the standard modal header style in DaisyUI docs.
            class: "font-bold text-lg",
            id: props.id,
            attributes: props.attributes,
            {props.children}
        }
    }
}

#[component]
pub fn DialogDescription(props: DialogDescriptionProps) -> Element {
    rsx! {
        dialog::DialogDescription {
            // Tailwind: 'py-4'. Justification: Standard padding utility to separate description from title.
            class: "py-4",
            id: props.id,
            attributes: props.attributes,
            {props.children}
        }
    }
}
