use yew::prelude::*;
use crate::context::{get_current_language, get_text, set_language, Language};

#[derive(Properties, PartialEq)]
pub struct SettingsPopupProps {
    pub on_close: Callback<()>,
    pub on_logout: Callback<()>,
}

#[function_component(SettingsPopup)]
pub fn settings_popup(props: &SettingsPopupProps) -> Html {
    let close = props.on_close.clone();
    let current_language = get_current_language();
    
    html! {
        <div class="settings-popup active">
            <div class="settings-content">
                <div class="settings-header">
                    <h3>{get_text("settings")}</h3>
                    <button class="btn-close-settings" onclick={Callback::from(move |_| close.emit(()))}>
                        {"✕"}
                    </button>
                </div>
                <div class="settings-body">
                    // Language toggle
                    <div class="language-section">
                        <div class="language-label">{get_text("language")}</div>
                        <div class="language-toggle">
                            <button
                                class={if current_language == Language::French { "toggle-btn active" } else { "toggle-btn" }}
                                onclick={Callback::from(|_| {
                                    set_language(&Language::French);
                                    // Force page reload to update all text
                                    if let Some(window) = web_sys::window() {
                                        let _ = window.location().reload();
                                    }
                                })}
                            >
                                {"FR"}
                            </button>
                            <button
                                class={if current_language == Language::Spanish { "toggle-btn active" } else { "toggle-btn" }}
                                onclick={Callback::from(|_| {
                                    set_language(&Language::Spanish);
                                    // Force page reload to update all text
                                    if let Some(window) = web_sys::window() {
                                        let _ = window.location().reload();
                                    }
                                })}
                            >
                                {"ES"}
                            </button>
                        </div>
                    </div>
                    
                    <button
                        class="btn-logout"
                        onclick={{
                            let on_logout = props.on_logout.clone();
                            Callback::from(move |_| on_logout.emit(()))
                        }}
                    >
                        {format!("🚪 {}", get_text("logout"))}
                    </button>
                </div>
            </div>
        </div>
    }
}

