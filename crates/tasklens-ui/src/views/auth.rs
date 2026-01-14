//! Authentication and Settings Views.
//!
//! This module defines the user interface for identity management and synchronization settings.
//! It includes the [`SettingsModal`] which acts as the main container, and various sub-views
//! for different authentication states (Landing, New User, Login).

use crate::components::*;
use dioxus::prelude::*;
use tasklens_store::crypto;

/// The primary modal for managing user identity and sync settings.
///
/// This component orchestrates the authentication flow, switching between
/// landing, login, and new user creation views. It also handles the persistent
/// storage of the master key.
///
/// # Props
/// * `master_key` - A read/write signal capable of updating the global master key.
/// * `on_close` - Event handler triggered when the modal should be closed.
#[component]
pub fn SettingsModal(master_key: Signal<Option<[u8; 32]>>, on_close: EventHandler<()>) -> Element {
    let mut mode = use_signal(|| AuthMode::Landing);
    let mut error_msg = use_signal(String::new);

    // New User State
    let mut generated_mnemonic = use_signal(String::new);
    let mut saved_confirm = use_signal(|| false);

    // Login State
    let input_mnemonic = use_signal(String::new);

    // Shared
    let remember_me = use_signal(|| false);
    let show_copy_toast = use_signal(|| false);

    // Generate a mnemonic IMMEDIATELY when switching to NewUser mode
    use_effect(move || {
        if mode() == AuthMode::NewUser && generated_mnemonic().is_empty() {
            generated_mnemonic.set(crypto::generate_key());
        }
    });

    let handle_login = move |phrase: String| match crypto::derive_key(&phrase) {
        Ok(key) => {
            let storage_mode = if remember_me() {
                crypto::StorageMode::Local
            } else {
                crypto::StorageMode::Session
            };
            if let Err(e) = crypto::save_key(&key, storage_mode) {
                tracing::error!("Failed to save key: {:?}", e);
            }
            master_key.set(Some(key));
            on_close.call(());
        }
        Err(e) => error_msg.set(format!("Error: {}", e)),
    };

    let handle_logout = move |_| {
        crypto::clear_key();
        master_key.set(None);
        mode.set(AuthMode::Landing);
    };

    rsx! {
        div { class: "fixed inset-0 z-50 overflow-y-auto",

            div { class: "flex items-center justify-center min-h-screen px-4 pt-4 pb-20 text-center sm:block sm:p-0",
                div {
                    class: "fixed inset-0 transition-opacity",
                    aria_hidden: "true",
                    onclick: move |_| on_close.call(()),
                    div { class: "absolute inset-0 bg-gray-500 opacity-75" }
                }

                span {
                    class: "hidden sm:inline-block sm:align-middle sm:h-screen",
                    aria_hidden: "true",
                    "\u{200b}"
                }

                div { class: "relative z-10 inline-block align-bottom bg-white rounded-lg text-left overflow-hidden shadow-xl transform transition-all sm:my-8 sm:align-middle sm:max-w-lg sm:w-full",
                    div { class: "bg-white px-4 pt-5 pb-4 sm:p-6 sm:pb-4",
                        div { class: "sm:flex sm:items-start",
                            div { class: "mt-3 text-center sm:mt-0 sm:ml-4 sm:text-left w-full",
                                h3 { class: "text-lg leading-6 font-medium text-gray-900 mb-4",
                                    "Sync Settings"
                                }

                                if !error_msg().is_empty() {
                                    Alert {
                                        variant: AlertVariant::Error,
                                        title: "Authentication Error",
                                        "{error_msg}"
                                    }
                                }

                                if let Some(_) = master_key() {
                                    SyncActiveView { on_logout: handle_logout }
                                } else {
                                    match mode() {
                                        AuthMode::Landing => rsx! {
                                            AuthLandingView {
                                                on_create: move |_| {
                                                    mode.set(AuthMode::NewUser);
                                                    error_msg.set(String::new());
                                                },
                                                on_login: move |_| {
                                                    mode.set(AuthMode::Login);
                                                    error_msg.set(String::new());
                                                },
                                            }
                                        },
                                        AuthMode::NewUser => rsx! {
                                            NewUserView {
                                                mnemonic: generated_mnemonic,
                                                saved_confirm,
                                                remember_me,
                                                show_copy_toast,
                                                on_cancel: move |_| {
                                                    generated_mnemonic.set(String::new());
                                                    saved_confirm.set(false);
                                                    mode.set(AuthMode::Landing);
                                                },
                                                on_confirm: handle_login,
                                            }
                                        },
                                        AuthMode::Login => rsx! {
                                            LoginView {
                                                mnemonic: input_mnemonic,
                                                remember_me,
                                                on_cancel: move |_| mode.set(AuthMode::Landing),
                                                on_confirm: handle_login,
                                            }
                                        },
                                    }
                                }
                            }
                        }
                    }
                    div { class: "bg-gray-50 px-4 py-3 sm:px-6 sm:flex sm:flex-row-reverse",
                        Button {
                            variant: ButtonVariant::Secondary,
                            class: "mt-3 w-full sm:mt-0 sm:ml-3 sm:w-auto",
                            onclick: move |_| on_close.call(()),
                            "Close"
                        }
                    }
                }
            }
        }
    }
}

/// Represents the current state of the authentication flow within the settings modal.
#[derive(PartialEq, Clone, Copy)]
enum AuthMode {
    /// The initial view presenting "Create New" or "Existing Key" options.
    Landing,
    /// The flow for generating and backing up a new identity.
    NewUser,
    /// The flow for restoring an existing identity from a mnemonic.
    Login,
}

/// Renders the active sync state view.
///
/// Displays a success message indicating that data is being synced and provides
/// a button to disconnect the current identity (logout).
///
/// # Props
/// * `on_logout` - Event handler triggered when the user clicks "Disconnect Identity".
#[component]
fn SyncActiveView(on_logout: EventHandler<()>) -> Element {
    rsx! {
        div { class: "space-y-4",
            Alert { variant: AlertVariant::Success, title: "Sync Active",
                "Your data is ready to be synced with this identity."
            }

            Button {
                variant: ButtonVariant::Destructive,
                onclick: move |_| on_logout.call(()),
                class: "w-full",
                "Disconnect Identity"
            }
        }
    }
}

/// Renders the initial authentication landing page.
///
/// Presents options for the user to either create a new identity or log in with
/// an existing one. This is the entry point for the authentication flow.
///
/// # Props
/// * `on_create` - Event handler to transition to the "New Identity" flow.
/// * `on_login` - Event handler to transition to the "Existing Key" flow.
#[component]
fn AuthLandingView(on_create: EventHandler<()>, on_login: EventHandler<()>) -> Element {
    rsx! {
        div { class: "space-y-4",
            p { class: "text-sm text-gray-500",
                "Create an identity to sync your data across devices."
            }
            Button {
                variant: ButtonVariant::Primary,
                class: "w-full py-4 text-lg",
                onclick: move |_| on_create.call(()),
                "Create New Identity"
            }
            div { class: "relative",
                div { class: "absolute inset-0 flex items-center",
                    div { class: "w-full border-t border-gray-300" }
                }
                div { class: "relative flex justify-center text-sm",
                    span { class: "px-2 bg-white text-gray-500", "Or" }
                }
            }
            Button {
                variant: ButtonVariant::Secondary,
                class: "w-full py-3",
                onclick: move |_| on_login.call(()),
                "I have an Existing Key"
            }
        }
    }
}

/// Renders the new user registration view.
///
/// Generates and displays a new random 12-word mnemonic phrase. It requires the
/// user to confirm they have saved the key before allowing them to proceed.
/// Includes functionality to copy the key to the clipboard.
///
/// # Props
/// * `mnemonic` - Signal containing the generated mnemonic phrase.
/// * `saved_confirm` - Signal tracking whether the user has confirmed saving the key.
/// * `remember_me` - Signal for the "Remember me" checkbox state.
/// * `show_copy_toast` - Signal to control the visibility of the "Copied" toast notification.
/// * `on_cancel` - Event handler to return to the landing view.
/// * `on_confirm` - Event handler called with the mnemonic when the user proceeds.
#[component]
fn NewUserView(
    mnemonic: Signal<String>,
    saved_confirm: Signal<bool>,
    remember_me: Signal<bool>,
    show_copy_toast: Signal<bool>,
    on_cancel: EventHandler<()>,
    on_confirm: EventHandler<String>,
) -> Element {
    let mut show_copy_toast = show_copy_toast;
    rsx! {
        div {
            BackButton { onclick: move |_| on_cancel.call(()) }

            Alert { variant: AlertVariant::Info, title: "This is your Password",
                "If you lose this key then you must re-create a new key to restore sync functionality on other devices."
            }

            // Copy Button & Grid
            div { class: "mb-6 relative",
                div { class: "flex justify-end mb-2",
                    Button {
                        variant: ButtonVariant::Secondary,
                        onclick: move |_| {
                            let phrase = mnemonic();
                            spawn(async move {
                                match document::eval(
                                        &format!(
                                            "return navigator.clipboard.writeText('{}').then(() => true)",
                                            phrase,
                                        ),
                                    )
                                    .await
                                {
                                    Ok(_) => {
                                        tracing::info!("Clipboard copy successful");
                                        show_copy_toast.set(true);
                                        if let Err(e) = document::eval(
                                                "return new Promise(r => setTimeout(() => r(true), 3000))",
                                            )
                                            .await
                                        {
                                            tracing::error!("Timer failed: {:?}", e);
                                        }
                                        show_copy_toast.set(false);
                                    }
                                    Err(e) => {
                                        tracing::error!("Clipboard copy failed: {:?}", e);
                                    }
                                }
                            });
                        },
                        "Copy to Clipboard"
                    }
                }

                div { class: "grid grid-cols-3 gap-2",
                    for (i , word) in mnemonic().split_whitespace().enumerate() {
                        div { class: "relative bg-gray-50 border border-gray-200 rounded px-2 py-2 flex items-center",
                            span { class: "absolute left-2 text-xs text-gray-400 select-none",
                                "{i+1}"
                            }
                            span { class: "w-full text-center font-mono font-medium text-gray-800 select-all",
                                "{word}"
                            }
                        }
                    }
                }

                if show_copy_toast() {
                    div { class: "absolute top-0 left-1/2 transform -translate-x-1/2 -translate-y-12 bg-gray-800 text-white px-4 py-2 rounded shadow-lg text-sm font-medium flex items-center transition-opacity duration-300",
                        svg {
                            class: "h-4 w-4 mr-2 text-green-400",
                            fill: "none",
                            view_box: "0 0 24 24",
                            stroke: "currentColor",
                            path {
                                stroke_linecap: "round",
                                stroke_linejoin: "round",
                                stroke_width: "2",
                                d: "M5 13l4 4L19 7",
                            }
                        }
                        "Copied to clipboard"
                    }
                }
            }

            div { class: "space-y-4",
                Checkbox {
                    id: "confirm-save",
                    checked: saved_confirm(),
                    onchange: move |val| saved_confirm.set(val),
                    label: "I have saved my secret password securely",
                }

                Checkbox {
                    id: "remember-me-new",
                    checked: remember_me(),
                    onchange: move |val| remember_me.set(val),
                    label: "Remember me on this device",
                }

                Button {
                    variant: ButtonVariant::Primary,
                    class: "w-full",
                    disabled: !saved_confirm(),
                    onclick: move |_| on_confirm.call(mnemonic()),
                    "Use This Identity"
                }
            }
        }
    }
}

/// Renders the login view for existing users.
///
/// Allows the user to input their existing 12-word mnemonic phrase to restore
/// their identity.
///
/// # Props
/// * `mnemonic` - Signal for the input text content of the mnemonic.
/// * `remember_me` - Signal for the "Remember me" checkbox state.
/// * `on_cancel` - Event handler to return to the landing view.
/// * `on_confirm` - Event handler called with the entered mnemonic to attempt login.
#[component]
fn LoginView(
    mnemonic: Signal<String>,
    remember_me: Signal<bool>,
    on_cancel: EventHandler<()>,
    on_confirm: EventHandler<String>,
) -> Element {
    let mut mnemonic = mnemonic;
    rsx! {
        div {
            BackButton { onclick: move |_| on_cancel.call(()) }

            div { class: "space-y-6",
                div {
                    label { class: "block text-sm font-medium text-gray-700", "Secret Key Phrase" }
                    div { class: "mt-1",
                        textarea {
                            class: "appearance-none block w-full px-3 py-2 border border-gray-300 rounded-md shadow-sm placeholder-gray-400 focus:outline-none focus:ring-blue-500 focus:border-blue-500 sm:text-sm font-mono h-32",
                            placeholder: "Enter your 12 words separated by spaces...",
                            value: "{mnemonic}",
                            oninput: move |evt| mnemonic.set(evt.value()),
                        }
                    }
                }

                Checkbox {
                    id: "remember-me-login",
                    checked: remember_me(),
                    onchange: move |val| remember_me.set(val),
                    label: "Remember me on this device",
                }

                Button {
                    variant: ButtonVariant::Primary,
                    class: "w-full",
                    onclick: move |_| on_confirm.call(mnemonic()),
                    "Unlock Identity"
                }
            }
        }
    }
}
