use yew::prelude::*;

#[derive(Properties, PartialEq, Clone)]
pub struct SettingsPopupProps {
    pub active: bool,
    pub on_close: Callback<()>,
    pub on_logout: Callback<()>,
    pub on_retry_map: Callback<()>,
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

    html! {
        <div class={class} onclick={stop}>
            <div class="settings-content">
                <div class="settings-header">
                    <h3>{"Paramètres"}</h3>
                    <button class="btn-close-settings" onclick={close_click}>{"✕"}</button>
                </div>
                <div class="settings-body">
                    <div class="language-section">
                        <div class="language-label">{"Langue"}</div>
                        <div class="language-toggle">
                            <button class="toggle-btn active">{"FR"}</button>
                            <button class="toggle-btn">{"ES"}</button>
                        </div>
                    </div>

                    <div class="reorder-mode-section">
                        <span class="reorder-mode-label">{"Mode édition"}</span>
                        <label class="toggle-switch">
                            <input type="checkbox" disabled=true />
                            <span class="toggle-slider"></span>
                        </label>
                    </div>

                    <div class="reorder-mode-section">
                        <span class="reorder-mode-label">{"Filtrer"}</span>
                        <label class="toggle-switch">
                            <input type="checkbox" disabled=true />
                            <span class="toggle-slider"></span>
                        </label>
                    </div>

                    <div class="color-codes-section">
                        <div class="color-codes-label">{"🎨 Codes couleur"}</div>
                        <div class="color-codes-list">
                            <div class="color-code-item"><div class="color-indicator relais"></div><span class="color-description">{"RELAIS"}</span></div>
                            <div class="color-code-item"><div class="color-indicator rcs"></div><span class="color-description">{"RCS (Premium)"}</span></div>
                            <div class="color-code-item"><div class="color-indicator green"></div><span class="color-description">{"Livré"}</span></div>
                            <div class="color-code-item"><div class="color-indicator red"></div><span class="color-description">{"Non livré"}</span></div>
                            <div class="color-code-item"><div class="color-indicator blue"></div><span class="color-description">{"En transit"}</span></div>
                            <div class="color-code-item"><div class="color-indicator cyan"></div><span class="color-description">{"Réceptionné"}</span></div>
                            <div class="color-code-item"><div class="color-indicator magenta"></div><span class="color-description">{"En collecte"}</span></div>
                        </div>
                    </div>

                    <button class="btn-logout" onclick={logout_click}>{"⎋ Déconnexion"}</button>
                </div>
            </div>
        </div>
    }
}


