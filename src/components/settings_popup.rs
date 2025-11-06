use yew::prelude::*;
use crate::utils::t;

#[derive(Properties, PartialEq, Clone)]
pub struct SettingsPopupProps {
    pub active: bool,
    pub on_close: Callback<()>,
    pub on_logout: Callback<()>,
    pub on_retry_map: Callback<()>,
    #[prop_or_default]
    pub language: String,
    #[prop_or_default]
    pub on_toggle_language: Option<Callback<String>>,
    #[prop_or_default]
    pub on_toggle_edit_mode: Option<Callback<bool>>,
    #[prop_or_default]
    pub edit_mode: bool,
    #[prop_or_default]
    pub on_toggle_filter: Option<Callback<bool>>,
    #[prop_or_default]
    pub filter_mode: bool,
}

#[function_component(SettingsPopup)]
pub fn settings_popup(props: &SettingsPopupProps) -> Html {
    let stop = Callback::from(|e: MouseEvent| e.stop_propagation());
    let close_click = {
        let cb = props.on_close.clone();
        Callback::from(move |_e: MouseEvent| cb.emit(()))
    };
    let logout_click = {
        let cb = props.on_logout.clone();
        Callback::from(move |_e: MouseEvent| cb.emit(()))
    };
    let _retry_click = {
        let cb = props.on_retry_map.clone();
        Callback::from(move |_e: MouseEvent| cb.emit(()))
    };

    let class = if props.active { "settings-popup active" } else { "settings-popup" };

    let lang = props.language.clone();
    let current_lang = lang.clone();
    
    let on_toggle_lang_fr = {
        let lang_state = props.on_toggle_language.clone();
        Callback::from(move |_| {
            if let Some(cb) = &lang_state {
                cb.emit("FR".to_string());
            }
        })
    };
    
    let on_toggle_lang_es = {
        let lang_state = props.on_toggle_language.clone();
        Callback::from(move |_| {
            if let Some(cb) = &lang_state {
                cb.emit("ES".to_string());
            }
        })
    };
    
    let on_toggle_edit = {
        let edit_cb = props.on_toggle_edit_mode.clone();
        let current_edit = props.edit_mode;
        Callback::from(move |e: Event| {
            if let Some(input) = e.target_dyn_into::<web_sys::HtmlInputElement>() {
                if let Some(cb) = &edit_cb {
                    cb.emit(input.checked());
                }
            }
        })
    };
    
    let on_toggle_filter = {
        let filter_cb = props.on_toggle_filter.clone();
        let current_filter = props.filter_mode;
        Callback::from(move |e: Event| {
            if let Some(input) = e.target_dyn_into::<web_sys::HtmlInputElement>() {
                if let Some(cb) = &filter_cb {
                    cb.emit(input.checked());
                }
            }
        })
    };

    html! {
        <div class={class} onclick={stop}>
            <div class="settings-content">
                <div class="settings-header">
                    <h3>{t("parametres", &lang)}</h3>
                    <button class="btn-close-settings" onclick={close_click}>{"✕"}</button>
                </div>
                <div class="settings-body">
                    <div class="language-section">
                        <div class="language-label">{t("langue", &lang)}</div>
                        <div class="language-toggle">
                            <button 
                                class={if current_lang == "FR" { "toggle-btn active" } else { "toggle-btn" }}
                                onclick={on_toggle_lang_fr}
                            >{"FR"}</button>
                            <button 
                                class={if current_lang == "ES" { "toggle-btn active" } else { "toggle-btn" }}
                                onclick={on_toggle_lang_es}
                            >{"ES"}</button>
                        </div>
                    </div>

                    <div class="reorder-mode-section">
                        <span class="reorder-mode-label">{t("mode_edition", &lang)}</span>
                        <label class="toggle-switch">
                            <input 
                                type="checkbox" 
                                checked={props.edit_mode}
                                onchange={on_toggle_edit}
                            />
                            <span class="toggle-slider"></span>
                        </label>
                    </div>

                    <div class="reorder-mode-section">
                        <span class="reorder-mode-label">{t("filtrer", &lang)}</span>
                        <label class="toggle-switch">
                            <input 
                                type="checkbox" 
                                checked={props.filter_mode}
                                onchange={on_toggle_filter}
                            />
                            <span class="toggle-slider"></span>
                        </label>
                    </div>

                    <div class="color-codes-section">
                        <div class="color-codes-label">{t("codes_couleur", &lang)}</div>
                        <div class="color-codes-list">
                            <div class="color-code-item"><div class="color-indicator relais"></div><span class="color-description">{t("relais", &lang)}</span></div>
                            <div class="color-code-item"><div class="color-indicator rcs"></div><span class="color-description">{t("rcs_premium", &lang)}</span></div>
                            <div class="color-code-item"><div class="color-indicator green"></div><span class="color-description">{t("livre", &lang)}</span></div>
                            <div class="color-code-item"><div class="color-indicator red"></div><span class="color-description">{t("non_livre", &lang)}</span></div>
                            <div class="color-code-item"><div class="color-indicator blue"></div><span class="color-description">{t("en_transit", &lang)}</span></div>
                            <div class="color-code-item"><div class="color-indicator cyan"></div><span class="color-description">{t("receptionne", &lang)}</span></div>
                            <div class="color-code-item"><div class="color-indicator magenta"></div><span class="color-description">{t("en_collecte", &lang)}</span></div>
                        </div>
                    </div>

                    <button class="btn-logout" onclick={logout_click}>{t("deconnexion", &lang)}</button>
                </div>
            </div>
        </div>
    }
}


