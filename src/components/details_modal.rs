// ============================================================================
// DETAILS MODAL COMPONENT
// ============================================================================
// ✅ COPIADO DEL ORIGINAL - Preserva UI/UX exacta
// Versión simplificada pero funcional
// ============================================================================

use yew::prelude::*;
use crate::models::package::Package;

#[derive(Properties, PartialEq)]
pub struct DetailsModalProps {
    pub package: Package,
    pub on_close: Callback<()>,
}

#[derive(Clone)]
pub struct DetailsModal;

pub enum Msg {
    Close,
    EditField(String, String), // (field, value)
}

impl Component for DetailsModal {
    type Message = Msg;
    type Properties = DetailsModalProps;

    fn create(_ctx: &Context<Self>) -> Self {
        Self
    }

    fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            Msg::Close => {
                ctx.props().on_close.emit(());
                false // No re-renderizar, se cierra
            }
            Msg::EditField(_field, _value) => {
                // TODO: Implementar actualización de campos
                log::info!("📝 Edit field: {} = {}", _field, _value);
                true
            }
        }
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        // ✅ HTML EXACTO DEL ORIGINAL preservado
        let package = &ctx.props().package;
        
        html! {
            <div class="modal active">
                <div class="modal-overlay" onclick={ctx.link().callback(|_| Msg::Close)}></div>
                <div class="modal-content" onclick={Callback::from(|e: MouseEvent| e.stop_propagation())}>
                    <div class="modal-header">
                        <h2>{format!("Paquete {}", package.tracking.clone())}</h2>
                        <button 
                            class="btn-close" 
                            onclick={ctx.link().callback(|_| Msg::Close)}
                        >
                            {"✕"}
                        </button>
                    </div>
                    
                    <div class="modal-body">
                        <div class="detail-section">
                            <div class="detail-label">{"Cliente"}</div>
                            <div class="detail-value">{&package.customer_name}</div>
                        </div>
                        
                        <div class="detail-section">
                            <div class="detail-label">{"Tracking"}</div>
                            <div class="detail-value">{&package.tracking}</div>
                        </div>
                        
                        <div class="detail-section">
                            <div class="detail-label">{"Teléfono"}</div>
                            <div class="detail-value">
                                {if let Some(ref phone) = package.phone_number {
                                    html! {
                                        <a href={format!("tel:{}", phone)} class="phone-link">
                                            {phone.clone()}
                                        </a>
                                    }
                                } else {
                                    html! { <span class="empty-value">{"No proporcionado"}</span> }
                                }}
                            </div>
                        </div>
                        
                        <div class="detail-section">
                            <div class="detail-label">{"Estado"}</div>
                            <div class="detail-value">
                                <span class={format!("status {}", package.status.to_lowercase().replace("_", "-"))}>
                                    {&package.status}
                                </span>
                            </div>
                        </div>
                        
                        {if let Some(ref indication) = package.customer_indication {
                            if !indication.is_empty() {
                                html! {
                                    <div class="detail-section">
                                        <div class="detail-label">{"Indicaciones del cliente"}</div>
                                        <div class="detail-value">{indication}</div>
                                    </div>
                                }
                            } else {
                                html! {}
                            }
                        } else {
                            html! {}
                        }}
                    </div>
                    
                    <div class="modal-footer">
                        <button 
                            class="btn-secondary" 
                            onclick={ctx.link().callback(|_| Msg::Close)}
                        >
                            {"Cerrar"}
                        </button>
                    </div>
                </div>
            </div>
        }
    }
}

