use crate::router::Route;
use dioxus::prelude::*;
use tasklens_core::types::{TaskID, TaskStatus, TunnelState};
const MAX_RESULTS: usize = 20;
const MIN_QUERY_LEN: usize = 1;

#[derive(Debug, Clone, PartialEq)]
struct SearchResult {
    id: TaskID,
    title: String,
    is_done: bool,
    parent_path: String,
}

/// Builds a breadcrumb path string for a task by walking up parent_id links.
fn build_parent_path(task_id: &TaskID, state: &TunnelState) -> String {
    let mut segments = Vec::new();
    let mut current_id = state.tasks.get(task_id).and_then(|t| t.parent_id.clone());

    while let Some(pid) = current_id {
        if let Some(parent) = state.tasks.get(&pid) {
            segments.push(parent.title.clone());
            current_id = parent.parent_id.clone();
        } else {
            break;
        }
    }

    segments.reverse();
    segments.join(" › ")
}

/// An inline search panel that slides down below the navbar.
///
/// Filters tasks by case-insensitive substring match on title.
/// Clicking a result navigates to the Plan view with the task focused.
#[component]
pub fn SearchPanel(open: Signal<bool>, on_close: EventHandler) -> Element {
    let mut query = use_signal(String::new);
    let state = use_context::<Memo<TunnelState>>();
    let nav = use_navigator();

    let results: Memo<Vec<SearchResult>> = use_memo(move || {
        let q = query.read().trim().to_lowercase();
        if q.len() < MIN_QUERY_LEN {
            return Vec::new();
        }

        let state = state.read();
        let mut matches: Vec<(SearchResult, bool)> = state
            .tasks
            .values()
            .filter(|t| t.title.to_lowercase().contains(&q))
            .map(|t| {
                let is_prefix = t.title.to_lowercase().starts_with(&q);
                let result = SearchResult {
                    id: t.id.clone(),
                    title: t.title.clone(),
                    is_done: t.status == TaskStatus::Done,
                    parent_path: build_parent_path(&t.id, &state),
                };
                (result, is_prefix)
            })
            .collect();

        // Prefix matches first, then alphabetical by title.
        matches.sort_by(|(a, a_prefix), (b, b_prefix)| {
            b_prefix
                .cmp(a_prefix)
                .then_with(|| a.title.to_lowercase().cmp(&b.title.to_lowercase()))
        });

        matches
            .into_iter()
            .take(MAX_RESULTS)
            .map(|(r, _)| r)
            .collect()
    });

    let mut navigate_to = move |id: TaskID| {
        nav.push(Route::PlanPage {
            focus_task: Some(id),
            seed: None,
        });
        query.set(String::new());
        on_close.call(());
    };

    let handle_keydown = move |evt: KeyboardEvent| {
        if evt.key() == Key::Escape {
            query.set(String::new());
            on_close.call(());
        }
    };

    // Focus the search input when the panel opens.
    // Delayed by one animation frame because the panel transitions from
    // zero height to visible. Browsers refuse to focus an element inside a
    // zero-height overflow-hidden container, so we wait one frame for the
    // CSS transition to start and the element to become focusable.
    use_effect(move || {
        if open() {
            #[cfg(target_arch = "wasm32")]
            {
                use wasm_bindgen::prelude::*;

                let cb = Closure::once_into_js(move || {
                    if let Some(window) = web_sys::window() {
                        if let Some(doc) = window.document() {
                            if let Some(el) = doc
                                .query_selector("[data-testid='search-input']")
                                .ok()
                                .flatten()
                            {
                                if let Some(html_el) = el.dyn_ref::<web_sys::HtmlElement>() {
                                    if let Err(e) = html_el.focus() {
                                        tracing::warn!("Failed to focus search input: {:?}", e);
                                    }
                                }
                            }
                        }
                    }
                });

                if let Some(window) = web_sys::window() {
                    if let Err(e) = window.request_animation_frame(cb.as_ref().unchecked_ref()) {
                        tracing::warn!("Failed to schedule focus callback: {:?}", e);
                    }
                }
            }
        }
    });

    let panel_classes = if open() {
        "max-h-96 opacity-100 translate-y-0"
    } else {
        "max-h-0 opacity-0 -translate-y-1 pointer-events-none"
    };

    rsx! {
        div {
            class: "transition-all duration-200 overflow-hidden bg-base-100 border-b border-base-200 {panel_classes}",
            "data-testid": "search-panel",
            div { class: "px-4 py-3 container mx-auto max-w-2xl",
                div { class: "flex items-center gap-2",
                    input {
                        class: "input input-bordered w-full text-base",
                        r#type: "text",
                        placeholder: "Search tasks...",
                        value: "{query}",
                        "data-testid": "search-input",
                        oninput: move |evt| query.set(evt.value()),
                        onkeydown: handle_keydown,
                    }
                    button {
                        class: "btn btn-ghost btn-sm btn-square text-base-content/70 hover:text-base-content",
                        onclick: move |_| {
                            query.set(String::new());
                            on_close.call(());
                        },
                        aria_label: "Close search",
                        "data-testid": "search-close",
                        svg {
                            class: "h-5 w-5",
                            fill: "none",
                            view_box: "0 0 24 24",
                            stroke: "currentColor",
                            path {
                                stroke_linecap: "round",
                                stroke_linejoin: "round",
                                stroke_width: "2",
                                d: "M6 18L18 6M6 6l12 12",
                            }
                        }
                    }
                }

                if query.read().trim().len() >= MIN_QUERY_LEN {
                    div {
                        class: "mt-2 max-h-64 overflow-y-auto",
                        "data-testid": "search-results",
                        if results().is_empty() {
                            div { class: "py-4 text-center text-base-content/60",
                                "No tasks found"
                            }
                        } else {
                            for result in results() {
                                {
                                    let id = result.id.clone();
                                    rsx! {
                                        button {
                                            key: "{result.id}",
                                            class: "w-full text-left px-3 py-2 rounded hover:bg-base-200 transition-colors flex flex-col",
                                            "data-testid": "search-result",
                                            onclick: move |_| navigate_to(id.clone()),
                                            span {
                                                class: if result.is_done { "line-through text-base-content/50" } else { "text-base-content" },
                                                "{result.title}"
                                            }
                                            if !result.parent_path.is_empty() {
                                                span { class: "text-xs text-base-content/40 mt-0.5",
                                                    "{result.parent_path}"
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}
