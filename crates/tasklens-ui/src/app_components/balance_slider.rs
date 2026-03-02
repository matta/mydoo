use crate::dioxus_components::slider::{
    Slider, SliderRange, SliderThumb, SliderTrack, SliderValue,
};
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
        div {
            style: "display: contents",
            onpointerup: move |_| { props.onchange.call(props.value) },
            onkeyup: move |_| { props.onchange.call(props.value) },
            Slider {
                min: props.min,
                max: props.max,
                step: props.step,
                value: Some(SliderValue::Single(props.value)),
                on_value_change: move |new_value| {
                    let SliderValue::Single(val) = new_value;
                    props.oninput.call(val);
                },
                SliderTrack {
                    SliderRange {}
                    SliderThumb {}
                }
            }
        }
    }
}
