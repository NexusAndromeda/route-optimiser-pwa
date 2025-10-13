use yew::prelude::*;
use crate::models::Package;

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
                            <button class="btn-icon" title="Options de la rue">
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
                            <button class="btn-icon-edit" title="Modifier">
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
                            <button class="btn-icon-edit" title="Modifier">
                                {"✏️"}
                            </button>
                        </div>
                    </div>

                    // Notes chauffeur
                    <div class="detail-section editable">
                        <div class="detail-label">{"Notes propres du chauffeur"}</div>
                        <div class="detail-value-with-action">
                            <span class="empty-value">{"Ajouter une note..."}</span>
                            <button class="btn-icon-edit" title="Modifier">
                                {"✏️"}
                            </button>
                        </div>
                    </div>
                </div>
            </div>
        </div>
    }
}

