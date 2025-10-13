use yew::prelude::*;
use crate::models::Package;
use web_sys::window;

#[derive(Properties, PartialEq)]
pub struct DetailsModalProps {
    pub package: Package,
    pub on_close: Callback<()>,
    pub on_edit_bal: Callback<()>,
}

#[function_component(DetailsModal)]
pub fn details_modal(props: &DetailsModalProps) -> Html {
    let close = props.on_close.clone();
    let close_overlay = props.on_close.clone();
    
    // Handler para opciones de calle
    let on_street_settings = Callback::from(move |e: MouseEvent| {
        e.stop_propagation();
        if let Some(win) = window() {
            let _ = win.alert_with_message("Options de la rue:\n\n• Voir historique de livraisons\n• Notes partagées par autres chauffeurs\n• Informations du quartier\n\n(À implémenter)");
        }
    });
    
    // Handler para editar código de puerta
    let on_edit_door_code = Callback::from(move |e: MouseEvent| {
        e.stop_propagation();
        if let Some(win) = window() {
            if let Ok(Some(value)) = win.prompt_with_message("Modifier Code de porte:") {
                if !value.trim().is_empty() {
                    let _ = win.alert_with_message(&format!("✅ Code de porte enregistré:\n{}", value));
                }
            }
        }
    });
    
    // Handler para editar indicaciones cliente
    let on_edit_client_notes = Callback::from(move |e: MouseEvent| {
        e.stop_propagation();
        if let Some(win) = window() {
            if let Ok(Some(value)) = win.prompt_with_message("Modifier Indications du client:") {
                if !value.trim().is_empty() {
                    let _ = win.alert_with_message(&format!("✅ Indications du client enregistré:\n{}", value));
                }
            }
        }
    });
    
    // Handler para editar notas del chauffeur
    let on_edit_driver_notes = Callback::from(move |e: MouseEvent| {
        e.stop_propagation();
        if let Some(win) = window() {
            if let Ok(Some(value)) = win.prompt_with_message("Modifier Notes du chauffeur:") {
                if !value.trim().is_empty() {
                    let _ = win.alert_with_message(&format!("✅ Notes du chauffeur enregistré:\n{}", value));
                }
            }
        }
    });
    
    html! {
        <div class="modal active">
            <div class="modal-overlay" onclick={Callback::from(move |_| close_overlay.emit(()))}></div>
            <div class="modal-content" onclick={Callback::from(|e: MouseEvent| e.stop_propagation())}>
                <div class="modal-header">
                    <h2>{format!("Colis #{}", props.package.id)}</h2>
                    <button class="btn-close" onclick={Callback::from(move |_| close.emit(()))}>
                        {"✕"}
                    </button>
                </div>
                <div class="modal-body">
                    // Destinataire
                    <div class="detail-section">
                        <div class="detail-label">{"Destinataire"}</div>
                        <div class="detail-value">{&props.package.recipient}</div>
                    </div>

                    // Adresse
                    <div class="detail-section">
                        <div class="detail-label">{"Adresse"}</div>
                        <div class="detail-value-with-action">
                            <span>{&props.package.address}</span>
                            <button 
                                class="btn-icon" 
                                title="Options de la rue"
                                onclick={on_street_settings}
                            >
                                {"⚙️"}
                            </button>
                        </div>
                    </div>

                    // Téléphone
                    <div class="detail-section">
                        <div class="detail-label">{"Téléphone"}</div>
                        <div class="detail-value">
                            <a href="tel:0612345678" class="phone-link">{"06 12 34 56 78"}</a>
                        </div>
                    </div>

                    // Codes de porte
                    <div class="detail-section editable">
                        <div class="detail-label">{"Codes de porte"}</div>
                        <div class="detail-value-with-action">
                            <span class="empty-value">{"Non renseigné"}</span>
                            <button 
                                class="btn-icon-edit" 
                                title="Modifier"
                                onclick={on_edit_door_code}
                            >
                                {"✏️"}
                            </button>
                        </div>
                    </div>

                    // BAL
                    <div class="detail-section editable">
                        <div class="detail-label">{"Accès boîte aux lettres (BAL)"}</div>
                        <div class="detail-value-with-action">
                            <span class="empty-value">{"Non renseigné"}</span>
                            <button 
                                class="btn-icon-edit" 
                                title="Modifier"
                                onclick={{
                                    let on_edit = props.on_edit_bal.clone();
                                    Callback::from(move |e: MouseEvent| {
                                        e.stop_propagation();
                                        on_edit.emit(());
                                    })
                                }}
                            >
                                {"✏️"}
                            </button>
                        </div>
                    </div>

                    // Indications client
                    <div class="detail-section editable">
                        <div class="detail-label">{"Indications du client"}</div>
                        <div class="detail-value-with-action">
                            <span>{"\"Laisser au gardien si absent\""}</span>
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
                        <div class="detail-label">{"Notes propres du chauffeur"}</div>
                        <div class="detail-value-with-action">
                            <span class="empty-value">{"Ajouter une note..."}</span>
                            <button 
                                class="btn-icon-edit" 
                                title="Modifier"
                                onclick={on_edit_driver_notes}
                            >
                                {"✏️"}
                            </button>
                        </div>
                    </div>
                </div>
            </div>
        </div>
    }
}

