use yew::prelude::*;
use wasm_bindgen::JsCast;

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
                        class="btn-retry-map"
                        onclick={Callback::from(|_| {
                            web_sys::js_sys::Reflect::get(&web_sys::window().unwrap(), &"reinitializeMap".into())
                                .unwrap()
                                .dyn_into::<web_sys::js_sys::Function>()
                                .unwrap()
                                .call0(&web_sys::js_sys::Object::new())
                                .unwrap();
                        })}
                        title="Reinicializar mapa si no carga"
                    >
                        {"🗺️ Réinitialiser la carte"}
                    </button>
                    
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

