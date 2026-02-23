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
    let on_change_input = props.onchange;
    let on_change_keyup = props.onchange;
    let on_input_change = props.oninput;

    rsx! {
        // TODO(issue-u9cGaU): Switch back to the Dioxus Slider component once upstream is fixed.
        input {
            r#type: "range",
            min: props.min,
            max: props.max,
            step: props.step,
            value: props.value,
            role: "slider",
            "aria-valuemin": props.min,
            "aria-valuemax": props.max,
            "aria-valuenow": props.value,
            style: "width: 100%; accent-color: var(--secondary-color-2);",
            oninput: move |evt: FormEvent| {
                let raw = evt.value();
                let Ok(parsed) = raw.parse::<f64>() else { return };
                let clamped = parsed.clamp(props.min, props.max);
                on_input_change.call(clamped);
            },
            onchange: move |evt: FormEvent| {
                let committed = evt.value().parse::<f64>().unwrap_or(props.value);
                let committed = committed.clamp(props.min, props.max);
                on_change_input.call(committed);
            },
            onkeyup: move |_| { on_change_keyup.call(props.value) },
        }
    }
}
