use crate::dioxus_components::dialog as dioxus_dialog;
use dioxus::prelude::*;
use dioxus_primitives::dialog::{
    DialogContentProps, DialogDescriptionProps, DialogRootProps, DialogTitleProps,
};

#[component]
pub fn DialogRoot(props: DialogRootProps) -> Element {
    rsx! {
        dioxus_dialog::DialogRoot {
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
    let class = match &props.class {
        Some(c) => format!("dialog {}", c),
        None => "dialog".to_string(),
    };
    rsx! {
        dioxus_primitives::dialog::DialogContent {
            id: props.id,
            attributes: props.attributes,
            class: "{class}",
            {props.children}
        }
    }
}

#[component]
pub fn DialogTitle(props: DialogTitleProps) -> Element {
    rsx! {
        dioxus_dialog::DialogTitle {
            id: props.id,
            attributes: props.attributes,
            {props.children}
        }
    }
}

#[component]
pub fn DialogDescription(props: DialogDescriptionProps) -> Element {
    rsx! {
        dioxus_dialog::DialogDescription {
            id: props.id,
            attributes: props.attributes,
            {props.children}
        }
    }
}
