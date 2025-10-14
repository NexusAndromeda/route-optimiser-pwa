use yew::prelude::*;

#[derive(Properties, PartialEq)]
pub struct SettingsPopupProps {
    pub on_close: Callback<()>,
    pub on_logout: Callback<()>,
}

#[function_component(SettingsPopup)]
pub fn settings_popup(props: &SettingsPopupProps) -> Html {
    let close = props.on_close.clone();
    
    html! {
        <div class="settings-popup active">
            <div class="settings-content">
                <div class="settings-header">
                    <h3>{"Paramètres"}</h3>
                    <button class="btn-close-settings" onclick={Callback::from(move |_| close.emit(()))}>
                        {"✕"}
                    </button>
                </div>
                <div class="settings-body">
                    <button
                        class="btn-logout"
                        onclick={{
                            let on_logout = props.on_logout.clone();
                            Callback::from(move |_| on_logout.emit(()))
                        }}
                    >
                        {"🚪 Déconnexion"}
                    </button>
                </div>
            </div>
        </div>
    }
}

