// ============================================================================
// DETAILS MODAL COMPONENT
// ============================================================================
// ✅ UI/UX EXACTA DEL BACKUP - Preserva diseño y funcionalidad
// Por ahora solo UI, lógica se implementará después
// ============================================================================

use yew::prelude::*;
use crate::models::package::Package;
use crate::models::address::Address;
use wasm_bindgen::JsCast;

#[derive(Properties, PartialEq)]
pub struct DetailsModalProps {
    pub package: Package,
    pub address: Address, // Dirección completa asociada al paquete
    pub on_close: Callback<()>,
    // Por ahora los callbacks de edición están sin implementar
    #[prop_or_default]
    pub on_edit_address: Option<Callback<String>>,
    #[prop_or_default]
    pub on_edit_door_code: Option<Callback<String>>,
    #[prop_or_default]
    pub on_toggle_mailbox: Option<Callback<bool>>,
    #[prop_or_default]
    pub on_edit_client_instructions: Option<Callback<String>>,
    #[prop_or_default]
    pub on_edit_driver_notes: Option<Callback<String>>,
}

#[function_component(DetailsModal)]
pub fn details_modal(props: &DetailsModalProps) -> Html {
    let package = &props.package;
    let address = &props.address;
    
    let close = props.on_close.clone();
    let close_overlay = props.on_close.clone();
    
    // Estados para edición
    let editing_address = use_state(|| false);
    let editing_door_code = use_state(|| false);
    let editing_driver_notes = use_state(|| false);
    let address_input = use_state(|| address.label.clone());
    let door_code_input = use_state(|| address.door_code.clone().unwrap_or_default());
    let driver_notes_input = use_state(|| address.driver_notes.clone().unwrap_or_default());
    let saving_address = use_state(|| false);
    let saving_door_code = use_state(|| false);
    let saving_mailbox = use_state(|| false);
    let saving_driver_notes = use_state(|| false);
    let error_message = use_state(|| Option::<String>::None);
    
    // Estado para has_mailbox_access que se actualiza cuando address cambia
    let has_mailbox_access_state = use_state(|| {
        address.mailbox_access.as_ref()
            .map(|s| s == "true" || s == "1" || s.to_lowercase() == "oui")
            .unwrap_or(false)
    });
    
    // Actualizar has_mailbox_access cuando props.address.mailbox_access cambia
    {
        let address_mailbox = address.mailbox_access.clone();
        let has_mailbox_access_state = has_mailbox_access_state.clone();
        use_effect_with(address_mailbox, move |mailbox_val| {
            let new_value = mailbox_val.as_ref()
                .map(|s| s == "true" || s == "1" || s.to_lowercase() == "oui")
                .unwrap_or(false);
            has_mailbox_access_state.set(new_value);
            || ()
        });
    }
    
    // Handler para editar dirección
    let on_street_settings = {
        let editing_address = editing_address.clone();
        let address_input = address_input.clone();
        let current_address = address.label.clone();
        Callback::from(move |e: MouseEvent| {
            e.stop_propagation();
            address_input.set(current_address.clone());
            editing_address.set(true);
        })
    };
    
    let on_save_address = {
        let editing_address = editing_address.clone();
        let address_input = address_input.clone();
        let on_edit = props.on_edit_address.clone();
        let close_edit = editing_address.clone();
        let saving_address = saving_address.clone();
        let error_message = error_message.clone();
        Callback::from(move |_| {
            let new_address = (*address_input).clone().trim().to_string();
            if new_address.is_empty() {
                error_message.set(Some("La dirección no puede estar vacía".to_string()));
                return;
            }
            
            saving_address.set(true);
            error_message.set(None);
            
            if let Some(cb) = &on_edit {
                cb.emit(new_address);
            }
            
            // Cerrar después de un breve delay para mostrar el estado de guardado
            let close_edit_clone = close_edit.clone();
            let saving_address_clone = saving_address.clone();
            gloo_timers::callback::Timeout::new(500, move || {
                saving_address_clone.set(false);
                close_edit_clone.set(false);
            }).forget();
        })
    };
    
    let on_cancel_edit_address = {
        let editing_address = editing_address.clone();
        let address_input = address_input.clone();
        let current_address = address.label.clone();
        Callback::from(move |_| {
            address_input.set(current_address.clone());
            editing_address.set(false);
        })
    };
    
    // Handler para editar código de puerta
    let on_edit_door_code = {
        let editing_door_code = editing_door_code.clone();
        let door_code_input = door_code_input.clone();
        let current_door_code = address.door_code.clone().unwrap_or_default();
        Callback::from(move |e: MouseEvent| {
            e.stop_propagation();
            door_code_input.set(current_door_code.clone());
            editing_door_code.set(true);
        })
    };
    
    let on_save_door_code = {
        let editing_door_code = editing_door_code.clone();
        let door_code_input = door_code_input.clone();
        let on_edit = props.on_edit_door_code.clone();
        let close_edit = editing_door_code.clone();
        let saving_door_code = saving_door_code.clone();
        Callback::from(move |_| {
            let new_code = (*door_code_input).clone().trim().to_string();
            saving_door_code.set(true);
            
            if let Some(cb) = &on_edit {
                if new_code.is_empty() {
                cb.emit(String::new());
                } else {
                    cb.emit(new_code);
                }
            }
            
            let close_edit_clone = close_edit.clone();
            let saving_door_code_clone = saving_door_code.clone();
            gloo_timers::callback::Timeout::new(500, move || {
                saving_door_code_clone.set(false);
                close_edit_clone.set(false);
            }).forget();
        })
    };
    
    let on_cancel_edit_door_code = {
        let editing_door_code = editing_door_code.clone();
        let door_code_input = door_code_input.clone();
        let current_door_code = address.door_code.clone().unwrap_or_default();
        Callback::from(move |_| {
            door_code_input.set(current_door_code.clone());
            editing_door_code.set(false);
        })
    };
    
    // Handler para toggle BAL - usar el estado actualizado
    let on_edit_bal = {
        let current_value = *has_mailbox_access_state;
        let on_toggle = props.on_toggle_mailbox.clone();
        let saving_mailbox = saving_mailbox.clone();
        let has_mailbox_access_state = has_mailbox_access_state.clone();
        Callback::from(move |e: MouseEvent| {
            e.stop_propagation();
            saving_mailbox.set(true);
            
            // Actualizar estado optimistamente
            let new_value = !current_value;
            has_mailbox_access_state.set(new_value);
            
            if let Some(cb) = &on_toggle {
                cb.emit(new_value);
            }
            
            let saving_mailbox_clone = saving_mailbox.clone();
            gloo_timers::callback::Timeout::new(500, move || {
                saving_mailbox_clone.set(false);
            }).forget();
        })
    };
    
    // Handler para editar notas chofer
    let on_edit_driver_notes = {
        let editing_driver_notes = editing_driver_notes.clone();
        let driver_notes_input = driver_notes_input.clone();
        let current_notes = address.driver_notes.clone().unwrap_or_default();
        Callback::from(move |e: MouseEvent| {
            e.stop_propagation();
            driver_notes_input.set(current_notes.clone());
            editing_driver_notes.set(true);
        })
    };
    
    let on_save_driver_notes = {
        let editing_driver_notes = editing_driver_notes.clone();
        let driver_notes_input = driver_notes_input.clone();
        let on_edit = props.on_edit_driver_notes.clone();
        let close_edit = editing_driver_notes.clone();
        let saving_driver_notes = saving_driver_notes.clone();
        Callback::from(move |_| {
            let new_notes = (*driver_notes_input).clone().trim().to_string();
            saving_driver_notes.set(true);
            
            if let Some(cb) = &on_edit {
                if new_notes.is_empty() {
                cb.emit(String::new());
                } else {
                    cb.emit(new_notes);
                }
            }
            
            let close_edit_clone = close_edit.clone();
            let saving_driver_notes_clone = saving_driver_notes.clone();
            gloo_timers::callback::Timeout::new(500, move || {
                saving_driver_notes_clone.set(false);
                close_edit_clone.set(false);
            }).forget();
        })
    };
    
    let on_cancel_edit_driver_notes = {
        let editing_driver_notes = editing_driver_notes.clone();
        let driver_notes_input = driver_notes_input.clone();
        let current_notes = address.driver_notes.clone().unwrap_or_default();
        Callback::from(move |_| {
            driver_notes_input.set(current_notes.clone());
            editing_driver_notes.set(false);
        })
    };
    
    // Handler para indicaciones cliente (solo lectura por ahora)
    let on_edit_client_notes = {
        Callback::from(move |e: MouseEvent| {
            e.stop_propagation();
            log::info!("✏️ Indicaciones cliente son solo lectura (vienen de Colis Privé)");
        })
    };
    
    // Determinar si tiene acceso a buzón (usar estado que se actualiza)
    let has_mailbox_access = *has_mailbox_access_state;
    
    html! {
        <div class="modal active">
            <div class="modal-overlay" onclick={Callback::from(move |_| close_overlay.emit(()))}></div>
            <div class="modal-content" onclick={Callback::from(|e: MouseEvent| e.stop_propagation())}>
                <div class="modal-header">
                    <h2>{format!("Colis {}", package.tracking.clone())}</h2>
                    <button class="btn-close" onclick={Callback::from(move |_| close.emit(()))}>
                        {"✕"}
                    </button>
                </div>
                <div class="modal-body">
                    // Mensaje de error (si existe)
                    {if let Some(ref error) = *error_message {
                        html! {
                            <div class="error-message" style="color: red; padding: 10px; margin-bottom: 10px; background: #ffe6e6; border-radius: 4px;">
                                {error}
                            </div>
                        }
                    } else {
                        html! {}
                    }}
                    
                    // Destinataire
                    <div class="detail-section">
                        <div class="detail-label">{"Destinataire"}</div>
                        <div class="detail-value">{&package.customer_name}</div>
                    </div>

                    // Adresse
                    <div class="detail-section">
                        <div class="detail-label">{"Adresse"}</div>
                        <div class="detail-value-with-action">
                            {if *editing_address {
                                html! {
                                    <div class="edit-input-group">
                                        <input 
                                            type="text"
                                            class="edit-input"
                                            value={(*address_input).clone()}
                                            oninput={Callback::from({
                                                let address_input = address_input.clone();
                                                move |e: InputEvent| {
                                                    if let Some(input) = e.target_dyn_into::<web_sys::HtmlInputElement>() {
                                                        address_input.set(input.value());
                                                    }
                                                }
                                            })}
                                            placeholder="Nouvelle adresse"
                                        />
                                        <button 
                                            class="btn-save" 
                                            onclick={on_save_address.clone()}
                                            title="Enregistrer"
                                            disabled={*saving_address}
                                        >
                                            {if *saving_address { "⏳" } else { "✓" }}
                                        </button>
                                        <button 
                                            class="btn-cancel" 
                                            onclick={on_cancel_edit_address.clone()}
                                            title="Annuler"
                                        >
                                            {"✕"}
                                        </button>
                                    </div>
                                }
                            } else {
                                html! {
                                    <>
                            <span>{&address.label}</span>
                            <button 
                                class="btn-icon" 
                                title="Modifier l'adresse"
                                onclick={on_street_settings}
                            >
                                {"⚙️"}
                            </button>
                                    </>
                                }
                            }}
                        </div>
                    </div>

                    // Téléphone
                    <div class="detail-section">
                        <div class="detail-label">{"Téléphone"}</div>
                        <div class="detail-value">
                            {if let Some(phone) = &package.phone_number {
                                html! {
                                    <a href={format!("tel:{}", phone)} class="phone-link">
                                        {phone.clone()}
                                    </a>
                                }
                            } else {
                                html! { <span class="empty-value">{"Non renseigné"}</span> }
                            }}
                        </div>
                    </div>

                    // Codes de porte
                    <div class="detail-section editable">
                        <div class="detail-label">{"Codes de porte"}</div>
                        <div class="detail-value-with-action">
                            {if *editing_door_code {
                                html! {
                                    <div class="edit-input-group">
                                        <input 
                                            type="text"
                                            class="edit-input"
                                            value={(*door_code_input).clone()}
                                            oninput={Callback::from({
                                                let door_code_input = door_code_input.clone();
                                                move |e: InputEvent| {
                                                    if let Some(input) = e.target_dyn_into::<web_sys::HtmlInputElement>() {
                                                        door_code_input.set(input.value());
                                                    }
                                                }
                                            })}
                                            placeholder="Code de porte"
                                        />
                                        <button 
                                            class="btn-save" 
                                            onclick={on_save_door_code.clone()}
                                            title="Enregistrer"
                                            disabled={*saving_door_code}
                                        >
                                            {if *saving_door_code { "⏳" } else { "✓" }}
                                        </button>
                                        <button 
                                            class="btn-cancel" 
                                            onclick={on_cancel_edit_door_code.clone()}
                                            title="Annuler"
                                        >
                                            {"✕"}
                                        </button>
                                    </div>
                                }
                            } else {
                                html! {
                                    <>
                            {if let Some(door_code) = &address.door_code {
                                html! { <span>{door_code}</span> }
                            } else {
                                html! { <span class="empty-value">{"Non renseigné"}</span> }
                            }}
                            <button 
                                class="btn-icon-edit" 
                                title="Modifier"
                                onclick={on_edit_door_code}
                            >
                                {"✏️"}
                            </button>
                                    </>
                                }
                            }}
                        </div>
                    </div>

                    // BAL
                    <div class="detail-section editable">
                        <div class="detail-label">{"Accès BAL"}</div>
                        <div class="detail-value-with-action">
                            <span>{if has_mailbox_access { "✅ Oui" } else { "❌ Non" }}</span>
                            <label class="toggle-switch">
                                <input 
                                    type="checkbox" 
                                    checked={has_mailbox_access}
                                    onclick={on_edit_bal}
                                    disabled={*saving_mailbox}
                                />
                                <span class="toggle-slider"></span>
                            </label>
                            {if *saving_mailbox {
                                html! { <span class="saving-indicator">{"⏳"}</span> }
                            } else {
                                html! {}
                            }}
                        </div>
                    </div>

                    // Indications client
                    <div class="detail-section editable">
                        <div class="detail-label">{"Indications client"}</div>
                        <div class="detail-value-with-action">
                            {if let Some(instructions) = &package.customer_indication {
                                if !instructions.is_empty() {
                                    html! { <span>{format!("\"{}\"", instructions)}</span> }
                                } else {
                                    html! { <span class="empty-value">{"Non renseigné"}</span> }
                                }
                            } else {
                                html! { <span class="empty-value">{"Non renseigné"}</span> }
                            }}
                            <button 
                                class="btn-icon-edit" 
                                title="Modifier"
                                onclick={on_edit_client_notes}
                            >
                                {"✏️"}
                            </button>
                        </div>
                    </div>

                    // Notes chauffeur
                    <div class="detail-section editable">
                        <div class="detail-label">{"Notes chauffeur"}</div>
                        <div class="detail-value-with-action">
                            {if *editing_driver_notes {
                                html! {
                                    <div class="edit-input-group">
                                        <textarea 
                                            class="edit-textarea"
                                            value={(*driver_notes_input).clone()}
                                            oninput={Callback::from({
                                                let driver_notes_input = driver_notes_input.clone();
                                                move |e: InputEvent| {
                                                    if let Some(textarea) = e.target_dyn_into::<web_sys::HtmlTextAreaElement>() {
                                                        driver_notes_input.set(textarea.value());
                                                    }
                                                }
                                            })}
                                            placeholder="Ajouter une note"
                                            rows="3"
                                        />
                                        <button 
                                            class="btn-save" 
                                            onclick={on_save_driver_notes.clone()}
                                            title="Enregistrer"
                                            disabled={*saving_driver_notes}
                                        >
                                            {if *saving_driver_notes { "⏳" } else { "✓" }}
                                        </button>
                                        <button 
                                            class="btn-cancel" 
                                            onclick={on_cancel_edit_driver_notes.clone()}
                                            title="Annuler"
                                        >
                                            {"✕"}
                                        </button>
                                    </div>
                                }
                            } else {
                                html! {
                                    <>
                            {if let Some(notes) = &address.driver_notes {
                                if !notes.is_empty() {
                                    html! { <span>{format!("\"{}\"", notes)}</span> }
                                } else {
                                    html! { <span class="empty-value">{"Ajouter une note"}</span> }
                                }
                            } else {
                                html! { <span class="empty-value">{"Ajouter une note"}</span> }
                            }}
                            <button 
                                class="btn-icon-edit" 
                                title="Modifier"
                                onclick={on_edit_driver_notes}
                            >
                                {"✏️"}
                            </button>
                                    </>
                                }
                            }}
                        </div>
                    </div>
                </div>
            </div>
        </div>
    }
}
