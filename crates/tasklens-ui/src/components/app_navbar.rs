use crate::components::navbar::{Navbar, NavbarItem, NavbarNav};
use crate::router::Route;
use dioxus::prelude::*;

#[component]
pub fn AppNavBar() -> Element {
    let active_index = use_signal(|| 0);
    rsx! {
        Navbar {
            NavbarNav { index: active_index,
                NavbarItem {
                    index: active_index,
                    value: 0usize,
                    to: Route::PlanPage {},
                    "Plan"
                }
                NavbarItem {
                    index: active_index,
                    value: 1usize,
                    to: Route::DoPage {},
                    "Do"
                }
                NavbarItem {
                    index: active_index,
                    value: 2usize,
                    to: Route::Balance {},
                    "Balance"
                }
            }
        }
        Outlet::<Route> {}
    }
}
