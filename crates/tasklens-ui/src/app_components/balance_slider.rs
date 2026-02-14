use dioxus::prelude::*;

#[derive(Props, Clone, PartialEq)]
pub(crate) struct BalanceSliderProps {
    min: f64,
    max: f64,
    step: f64,
    value: f64,
    oninput: EventHandler<f64>,
    onchange: EventHandler<f64>,
}

#[component]
pub(crate) fn BalanceSlider(props: BalanceSliderProps) -> Element {
    rsx! {
        input {
            r#type: "range",
            class: "range range-primary range-xs",
            min: "{props.min}",
            max: "{props.max}",
            step: "{props.step}",
            value: "{props.value}",
            oninput: move |evt| {
                if let Ok(new_value) = evt.value().parse::<f64>() {
                    props.oninput.call(new_value);
                }
            },
            onchange: move |evt| {
                if let Ok(new_value) = evt.value().parse::<f64>() {
                    props.onchange.call(new_value);
                }
            }
        }
    }
}
