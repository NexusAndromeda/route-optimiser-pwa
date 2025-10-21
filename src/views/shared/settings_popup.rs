use yew::prelude::*;
use crate::context::{get_current_language, get_text, set_language, Language};

#[derive(Properties, PartialEq)]
pub struct SettingsPopupProps {
    pub on_close: Callback<()>,
    pub on_logout: Callback<()>,
    #[prop_or_default]
    pub reorder_mode: bool,
    #[prop_or_default]
    pub on_toggle_reorder: Option<Callback<()>>,
    #[prop_or_default]
    pub filter_mode: bool,
    #[prop_or_default]
    pub on_toggle_filter: Option<Callback<()>>,
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
                    
                    // Reorder mode toggle
                    <div class="reorder-mode-section">
                        <span class="reorder-mode-label">{get_text("edit_mode")}</span>
                        <label class="toggle-switch">
                            <input
                                type="checkbox"
                                checked={props.reorder_mode}
                                onchange={{
                                    let on_toggle = props.on_toggle_reorder.clone();
                                    Callback::from(move |_| {
                                        if let Some(toggle) = &on_toggle {
                                            toggle.emit(());
                                        }
                                    })
                                }}
                            />
                            <span class="toggle-slider"></span>
                        </label>
                    </div>
                    
                    // Filter mode toggle
                    <div class="reorder-mode-section">
                        <span class="reorder-mode-label">{get_text("filter")}</span>
                        <label class="toggle-switch">
                            <input
                                type="checkbox"
                                checked={props.filter_mode}
                                onchange={{
                                    let on_toggle = props.on_toggle_filter.clone();
                                    Callback::from(move |_| {
                                        if let Some(toggle) = &on_toggle {
                                            toggle.emit(());
                                        }
                                    })
                                }}
                            />
                            <span class="toggle-slider"></span>
                        </label>
                    </div>
                    
                    // Color codes section
                    <div class="color-codes-section">
                        <div class="color-codes-label">{format!("🎨 {}", get_text("color_codes"))}</div>
                        <div class="color-codes-list">
                            // Tipos de entrega
                            <div class="color-code-item">
                                <div class="color-indicator relais"></div>
                                <span class="color-description">{"RELAIS"}</span>
                            </div>
                            <div class="color-code-item">
                                <div class="color-indicator rcs"></div>
                                <span class="color-description">{"RCS (Premium)"}</span>
                            </div>
                            // Estados de entrega
                            <div class="color-code-item">
                                <div class="color-indicator green"></div>
                                <span class="color-description">{get_text("delivered_status")}</span>
                            </div>
                            <div class="color-code-item">
                                <div class="color-indicator red"></div>
                                <span class="color-description">{get_text("not_delivered")}</span>
                            </div>
                            <div class="color-code-item">
                                <div class="color-indicator blue"></div>
                                <span class="color-description">{get_text("in_transit")}</span>
                            </div>
                            <div class="color-code-item">
                                <div class="color-indicator cyan"></div>
                                <span class="color-description">{"Réceptionné"}</span>
                            </div>
                            <div class="color-code-item">
                                <div class="color-indicator magenta"></div>
                                <span class="color-description">{"En collecte"}</span>
                            </div>
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

