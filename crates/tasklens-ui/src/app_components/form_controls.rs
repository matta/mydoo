use crate::dioxus_components::input::Input;
use crate::dioxus_components::textarea::{Textarea, TextareaVariant};
use dioxus::prelude::*;

#[css_module("/src/app_components/form_controls.css")]
struct Styles;

#[derive(Copy, Clone, PartialEq, Eq, Default)]
pub(crate) enum AppInputStyle {
    #[default]
    Default,
    Large,
    Monospace,
}

impl AppInputStyle {
    fn class_name<'a>(self, large: &'a str, monospace: &'a str) -> Option<&'a str> {
        match self {
            Self::Default => None,
            Self::Large => Some(large),
            Self::Monospace => Some(monospace),
        }
    }
}

fn strip_class_attr(attributes: Vec<Attribute>) -> Vec<Attribute> {
    attributes
        .into_iter()
        .filter(|attribute| attribute.name != "class")
        .collect()
}

struct InputClassTokens {
    base: &'static str,
    full_width: &'static str,
    large: &'static str,
    monospace: &'static str,
}

fn compose_input_wrapper_class(
    full_width: bool,
    style: AppInputStyle,
    tokens: InputClassTokens,
) -> String {
    let mut classes = String::from(tokens.base);
    if full_width {
        classes.push(' ');
        classes.push_str(tokens.full_width);
    }
    if let Some(style_class) = style.class_name(tokens.large, tokens.monospace) {
        classes.push(' ');
        classes.push_str(style_class);
    }
    classes
}

fn input_wrapper_class(full_width: bool, style: AppInputStyle) -> String {
    compose_input_wrapper_class(
        full_width,
        style,
        InputClassTokens {
            base: Styles::input_wrapper.inner,
            full_width: Styles::input_full_width.inner,
            large: Styles::input_large.inner,
            monospace: Styles::input_monospace.inner,
        },
    )
}

fn compose_textarea_wrapper_class(
    full_width: bool,
    base: &'static str,
    full_width_class: &'static str,
) -> String {
    let mut classes = String::from(base);
    if full_width {
        classes.push(' ');
        classes.push_str(full_width_class);
    }
    classes
}

fn textarea_wrapper_class(full_width: bool) -> String {
    compose_textarea_wrapper_class(
        full_width,
        Styles::textarea_wrapper.inner,
        Styles::textarea_full_width.inner,
    )
}

#[component]
pub(crate) fn AppInput(
    oninput: Option<EventHandler<FormEvent>>,
    onchange: Option<EventHandler<FormEvent>>,
    oninvalid: Option<EventHandler<FormEvent>>,
    onselect: Option<EventHandler<SelectionEvent>>,
    onselectionchange: Option<EventHandler<SelectionEvent>>,
    onfocus: Option<EventHandler<FocusEvent>>,
    onblur: Option<EventHandler<FocusEvent>>,
    onfocusin: Option<EventHandler<FocusEvent>>,
    onfocusout: Option<EventHandler<FocusEvent>>,
    onkeydown: Option<EventHandler<KeyboardEvent>>,
    onkeypress: Option<EventHandler<KeyboardEvent>>,
    onkeyup: Option<EventHandler<KeyboardEvent>>,
    oncompositionstart: Option<EventHandler<CompositionEvent>>,
    oncompositionupdate: Option<EventHandler<CompositionEvent>>,
    oncompositionend: Option<EventHandler<CompositionEvent>>,
    oncopy: Option<EventHandler<ClipboardEvent>>,
    oncut: Option<EventHandler<ClipboardEvent>>,
    onpaste: Option<EventHandler<ClipboardEvent>>,
    #[props(default)] full_width: bool,
    #[props(default)] style: AppInputStyle,
    #[props(extends = GlobalAttributes)]
    #[props(extends = input)]
    attributes: Vec<Attribute>,
    children: Element,
) -> Element {
    let wrapper_class = input_wrapper_class(full_width, style);
    let filtered_attrs = strip_class_attr(attributes);

    rsx! {
        div { class: "{wrapper_class}",
            Input {
                oninput: oninput,
                onchange: onchange,
                oninvalid: oninvalid,
                onselect: onselect,
                onselectionchange: onselectionchange,
                onfocus: onfocus,
                onblur: onblur,
                onfocusin: onfocusin,
                onfocusout: onfocusout,
                onkeydown: onkeydown,
                onkeypress: onkeypress,
                onkeyup: onkeyup,
                oncompositionstart: oncompositionstart,
                oncompositionupdate: oncompositionupdate,
                oncompositionend: oncompositionend,
                oncopy: oncopy,
                oncut: oncut,
                onpaste: onpaste,
                attributes: filtered_attrs,
                {children}
            }
        }
    }
}

#[component]
pub(crate) fn AppTextarea(
    oninput: Option<EventHandler<FormEvent>>,
    onchange: Option<EventHandler<FormEvent>>,
    oninvalid: Option<EventHandler<FormEvent>>,
    onselect: Option<EventHandler<SelectionEvent>>,
    onselectionchange: Option<EventHandler<SelectionEvent>>,
    onfocus: Option<EventHandler<FocusEvent>>,
    onblur: Option<EventHandler<FocusEvent>>,
    onfocusin: Option<EventHandler<FocusEvent>>,
    onfocusout: Option<EventHandler<FocusEvent>>,
    onkeydown: Option<EventHandler<KeyboardEvent>>,
    onkeypress: Option<EventHandler<KeyboardEvent>>,
    onkeyup: Option<EventHandler<KeyboardEvent>>,
    oncompositionstart: Option<EventHandler<CompositionEvent>>,
    oncompositionupdate: Option<EventHandler<CompositionEvent>>,
    oncompositionend: Option<EventHandler<CompositionEvent>>,
    oncopy: Option<EventHandler<ClipboardEvent>>,
    oncut: Option<EventHandler<ClipboardEvent>>,
    onpaste: Option<EventHandler<ClipboardEvent>>,
    #[props(default)] full_width: bool,
    #[props(default)] variant: TextareaVariant,
    #[props(extends = GlobalAttributes)]
    #[props(extends = textarea)]
    attributes: Vec<Attribute>,
    children: Element,
) -> Element {
    let wrapper_class = textarea_wrapper_class(full_width);
    let filtered_attrs = strip_class_attr(attributes);

    rsx! {
        div { class: "{wrapper_class}",
            Textarea {
                oninput: oninput,
                onchange: onchange,
                oninvalid: oninvalid,
                onselect: onselect,
                onselectionchange: onselectionchange,
                onfocus: onfocus,
                onblur: onblur,
                onfocusin: onfocusin,
                onfocusout: onfocusout,
                onkeydown: onkeydown,
                onkeypress: onkeypress,
                onkeyup: onkeyup,
                oncompositionstart: oncompositionstart,
                oncompositionupdate: oncompositionupdate,
                oncompositionend: oncompositionend,
                oncopy: oncopy,
                oncut: oncut,
                onpaste: onpaste,
                variant: variant,
                attributes: filtered_attrs,
                {children}
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn input_wrapper_includes_full_width_and_large() {
        let class_name = compose_input_wrapper_class(
            true,
            AppInputStyle::Large,
            InputClassTokens {
                base: "base",
                full_width: "full",
                large: "lg",
                monospace: "mono",
            },
        );
        assert!(class_name.contains("base"));
        assert!(class_name.contains("full"));
        assert!(class_name.contains("lg"));
        assert!(!class_name.contains("mono"));
    }

    #[test]
    fn input_wrapper_includes_monospace_variant() {
        let class_name = compose_input_wrapper_class(
            true,
            AppInputStyle::Monospace,
            InputClassTokens {
                base: "base",
                full_width: "full",
                large: "lg",
                monospace: "mono",
            },
        );
        assert!(class_name.contains("base"));
        assert!(class_name.contains("full"));
        assert!(class_name.contains("mono"));
        assert!(!class_name.contains("lg"));
    }

    #[test]
    fn textarea_wrapper_defaults_to_base_class() {
        let class_name = compose_textarea_wrapper_class(false, "base", "full");
        assert_eq!(class_name, "base");
    }
}
