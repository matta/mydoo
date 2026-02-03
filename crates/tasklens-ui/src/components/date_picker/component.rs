use dioxus::prelude::*;

use dioxus_primitives::{
    ContentAlign,
    date_picker::{self, DatePickerInputProps, DatePickerProps},
    popover::{PopoverContentProps, PopoverTriggerProps},
};

use super::super::calendar::*;
use super::super::popover::*;

#[component]
pub fn DatePicker(props: DatePickerProps) -> Element {
    rsx! {
        div {
            date_picker::DatePicker {
                // Tailwind: 'relative inline-flex items-center'.
                // Justification: Layout for the input group.
                class: "relative inline-flex items-center",
                on_value_change: props.on_value_change,
                selected_date: props.selected_date,
                disabled: props.disabled,
                read_only: props.read_only,
                attributes: props.attributes,
                date_picker::DatePickerPopover { popover_root: PopoverRoot, {props.children} }
            }
        }
    }
}

#[component]
pub fn DatePickerInput(props: DatePickerInputProps) -> Element {
    rsx! {
        date_picker::DatePickerInput {
            on_format_day_placeholder: props.on_format_day_placeholder,
            on_format_month_placeholder: props.on_format_month_placeholder,
            on_format_year_placeholder: props.on_format_year_placeholder,
            attributes: props.attributes,
            {props.children}
            DatePickerPopoverTrigger {}
            DatePickerPopoverContent { align: ContentAlign::Center,
                date_picker::DatePickerCalendar { calendar: Calendar,
                    CalendarHeader {
                        CalendarNavigation {
                            CalendarPreviousMonthButton {}
                            CalendarSelectMonth {}
                            CalendarSelectYear {}
                            CalendarNextMonthButton {}
                        }
                    }
                    CalendarGrid {}
                }
            }
        }
    }
}

#[component]
pub fn DatePickerPopoverTrigger(props: PopoverTriggerProps) -> Element {
    rsx! {
        PopoverTrigger { aria_label: "Show Calendar", attributes: props.attributes,
            svg {
                // Tailwind: 'h-5 w-5 text-primary'.
                // Justification: Icon sizing and coloring.
                class: "h-5 w-5 text-primary cursor-pointer",
                view_box: "0 0 24 24",
                xmlns: "http://www.w3.org/2000/svg",
                polyline { points: "6 9 12 15 18 9" }
            }
        }
    }
}

#[component]
pub fn DatePickerPopoverContent(props: PopoverContentProps) -> Element {
    rsx! {
        PopoverContent {
            // DaisyUI: 'dropdown-content menu bg-base-100 rounded-box shadow'.
            // Reusing the dropdown card aesthetic.
            // Tailwind: 'p-4 w-auto z-50'.
            class: "dropdown-content bg-base-100 rounded-box shadow z-50 p-4 w-auto",
            id: props.id,
            side: props.side,
            align: props.align,
            attributes: props.attributes,
            {props.children}
        }
    }
}
