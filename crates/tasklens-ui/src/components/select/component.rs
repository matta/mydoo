use dioxus::prelude::*;
use dioxus_primitives::select::{
    self, SelectGroupLabelProps, SelectGroupProps, SelectListProps, SelectOptionProps, SelectProps,
    SelectTriggerProps, SelectValueProps,
};

#[component]
pub fn Select<T: Clone + PartialEq + 'static>(props: SelectProps<T>) -> Element {
    rsx! {
        select::Select {
            // DaisyUI: 'dropdown' is the container for the trigger and content.
            // Tailwind: 'w-full' (layout).
            class: "dropdown w-full",
            value: props.value,
            default_value: props.default_value,
            on_value_change: props.on_value_change,
            disabled: props.disabled,
            name: props.name,
            placeholder: props.placeholder,
            roving_loop: props.roving_loop,
            typeahead_timeout: props.typeahead_timeout,
            attributes: props.attributes,
            {props.children}
        }
    }
}

#[component]
pub fn SelectTrigger(props: SelectTriggerProps) -> Element {
    rsx! {
        select::SelectTrigger {
            // DaisyUI: 'select select-bordered' gives the input look.
            // Tailwind: 'w-full flex items-center justify-between font-normal'.
            // Justification: 'select' is typically an input; repurposing as a trigger requires layout fixups to center the icon and text.
            class: "select select-bordered w-full flex items-center justify-between font-normal",
            attributes: props.attributes,
            {props.children}
            svg {
                // Tailwind: Sizing for the icon.
                class: "h-4 w-4 opacity-50",
                view_box: "0 0 24 24",
                xmlns: "http://www.w3.org/2000/svg",
                polyline { points: "6 9 12 15 18 9" }
            }
        }
    }
}

#[component]
pub fn SelectValue(props: SelectValueProps) -> Element {
    rsx! {
        select::SelectValue { attributes: props.attributes }
    }
}

#[component]
pub fn SelectList(props: SelectListProps) -> Element {
    rsx! {
        select::SelectList {
            // DaisyUI: 'dropdown-content menu bg-base-100 rounded-box shadow'.
            // This is the canonical way to style a dropdown list in DaisyUI.
            // Tailwind: 'w-full mt-1 z-50 p-2'.
            // Justification: 'mt-1' for spacing, 'z-50' to overlay, 'w-full' to match trigger width.
            class: "dropdown-content menu bg-base-100 rounded-box shadow w-full mt-1 z-50 p-2",
            id: props.id,
            attributes: props.attributes,
            {props.children}
        }
    }
}

#[component]
pub fn SelectGroup(props: SelectGroupProps) -> Element {
    rsx! {
        select::SelectGroup {
            class: "w-full",
            disabled: props.disabled,
            id: props.id,
            attributes: props.attributes,
            {props.children}
        }
    }
}

#[component]
pub fn SelectGroupLabel(props: SelectGroupLabelProps) -> Element {
    rsx! {
        select::SelectGroupLabel {
            // Check if 'menu-title' is appropriate, or just padding/typography.
            // daisyUI 'menu-title' adds padding and semantic color.
            class: "menu-title",
            id: props.id,
            attributes: props.attributes,
            {props.children}
        }
    }
}

#[component]
pub fn SelectOption<T: Clone + PartialEq + 'static>(props: SelectOptionProps<T>) -> Element {
    rsx! {
        select::SelectOption::<T> {
            // DaisyUI: We rely on the parent 'menu' class to style <li> children.
            // However, 'menu' expects <li><a>...</a></li> structure.
            // If SelectOption renders a div/button, we might need to enforce 'btn-ghost'.
            // Let's try to align with 'menu' item structure or use 'btn-ghost' as fallback.
            // Using 'btn btn-ghost btn-sm w-full justify-start' ensures consistent item look.
            class: "btn btn-ghost btn-sm w-full justify-start font-normal",
            value: props.value,
            text_value: props.text_value,
            disabled: props.disabled,
            id: props.id,
            index: props.index,
            aria_label: props.aria_label,
            aria_roledescription: props.aria_roledescription,
            attributes: props.attributes,
            {props.children}
            // Indicator hook if needed, but 'btn' doesn't easily support checkmark without layout.
            // For now, we omit the check icon or need to insert it manually into children.
        }
    }
}

#[component]
pub fn SelectItemIndicator() -> Element {
    rsx! {
        select::SelectItemIndicator {
            svg {
                class: "h-4 w-4 ml-2",
                view_box: "0 0 24 24",
                xmlns: "http://www.w3.org/2000/svg",
                path { d: "M5 13l4 4L19 7" }
            }
        }
    }
}
