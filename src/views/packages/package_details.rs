use yew::prelude::*;
use crate::models::Package;
use crate::context::get_text;
use web_sys::window;
use gloo_net::http::Request;
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_name = updatePackageCoordinates)]
    fn update_package_coordinates(package_id: &str, latitude: f64, longitude: f64) -> bool;
    
    #[wasm_bindgen(js_name = addPackageToMap)]
    fn add_package_to_map(package_id: &str, latitude: f64, longitude: f64, address: &str, code_statut_article: Option<String>) -> bool;
    
    #[wasm_bindgen(js_name = removePackageFromMap)]
    fn remove_package_from_map(package_id: &str) -> bool;
}

#[derive(Properties, PartialEq)]
pub struct PackageDetailsProps {
    pub package: Package,
    pub on_close: Callback<()>,
    pub on_edit_bal: Callback<()>,
    pub on_update_package: Callback<(String, f64, f64, String)>, // (id, lat, lng, new_address)
    pub on_mark_problematic: Callback<String>, // (package_id)
}

#[function_component(PackageDetails)]
pub fn package_details(props: &PackageDetailsProps) -> Html {
    let close = props.on_close.clone();
    let close_overlay = props.on_close.clone();
    
    // Handler para geocodificación de dirección
    let package_id = props.package.id.clone();
    let on_street_settings = {
        let package_id = package_id.clone();
        let on_update = props.on_update_package.clone();
        let on_mark_problematic = props.on_mark_problematic.clone();
        let code_statut = props.package.code_statut_article.clone();
        Callback::from(move |e: MouseEvent| {
            e.stop_propagation();
            if let Some(win) = window() {
                if let Ok(Some(new_address)) = win.prompt_with_message(&get_text("geocoding_prompt")) {
                    let trimmed_address = new_address.trim().to_string();
                    
                    if trimmed_address.is_empty() {
                        // Si se deja vacío, marcar como problemático
                        log::info!("⚠️ Dirección vacía para paquete {}, marcando como problemático", package_id);
                        on_mark_problematic.emit(package_id.clone());
                        
                        // Quitar del mapa
                        if remove_package_from_map(&package_id) {
                            log::info!("🗑️ Paquete {} removido del mapa", package_id);
                        }
                        
                        if let Ok(_) = win.alert_with_message(&get_text("package_marked_problematic")) {
                            // Cerrar modal después de confirmar
                        }
                    } else {
                        // Geocodificar la nueva dirección
                        let package_id = package_id.clone();
                        let on_update = on_update.clone();
                        let code_statut_clone = code_statut.clone();
                        log::info!("🌍 Géocodage demandé pour paquete {}: {}", package_id, trimmed_address);
                        
                        // Llamar al endpoint de geocodificación
                        wasm_bindgen_futures::spawn_local(async move {
                            match geocode_address(trimmed_address.clone()).await {
                                Ok(response) => {
                                    if response.success {
                                        let lat = response.latitude.unwrap_or(0.0);
                                        let lng = response.longitude.unwrap_or(0.0);
                                        let formatted = response.formatted_address.unwrap_or(trimmed_address);
                                        
                                        log::info!("✅ Géocodage réussi: {} -> ({}, {})", 
                                            formatted, lat, lng
                                        );
                                        
                                        // Agregar/actualizar el paquete en el mapa
                                        if add_package_to_map(&package_id, lat, lng, &formatted, code_statut_clone) {
                                            log::info!("📍 Package ajouté/mis à jour sur la carte: {}", package_id);
                                            
                                            // Actualizar el paquete en el estado de Yew
                                            on_update.emit((package_id.clone(), lat, lng, formatted));
                                        } else {
                                            log::error!("❌ Échec de l'ajout du package sur la carte");
                                        }
                                    } else {
                                        log::error!("❌ Géocodage échoué: {}", response.message.clone().unwrap_or_default());
                                        if let Some(win) = window() {
                                            let _ = win.alert_with_message(&format!("❌ Error de geocodificación: {}", response.message.unwrap_or_default()));
                                        }
                                    }
                                }
                                Err(e) => {
                                    log::error!("❌ Erreur lors du géocodage: {}", e);
                                    if let Some(win) = window() {
                                        let _ = win.alert_with_message(&format!("❌ Error de geocodificación: {}", e));
                                    }
                                }
                            }
                        });
                    }
                }
            }
        })
    };
    
    // Handler para editar código de puerta
    let on_edit_door_code = Callback::from(move |e: MouseEvent| {
        e.stop_propagation();
        if let Some(win) = window() {
            if let Ok(Some(value)) = win.prompt_with_message(&get_text("edit_door_code")) {
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
            if let Ok(Some(value)) = win.prompt_with_message(&get_text("edit_client_instructions")) {
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
            if let Ok(Some(value)) = win.prompt_with_message(&get_text("edit_driver_notes")) {
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
                        <div class="detail-label">{get_text("recipient")}</div>
                        <div class="detail-value">{&props.package.recipient}</div>
                    </div>

                    // Adresse
                    <div class="detail-section">
                        <div class="detail-label">{get_text("address")}</div>
                        <div class="detail-value-with-action">
                            <span>{&props.package.address}</span>
                            <button 
                                class="btn-icon" 
                                title={get_text("edit_address")}
                                onclick={on_street_settings}
                            >
                                {"⚙️"}
                            </button>
                        </div>
                    </div>

                    // Téléphone
                    <div class="detail-section">
                        <div class="detail-label">{get_text("phone")}</div>
                        <div class="detail-value">
                            {if let Some(phone) = &props.package.phone {
                                html! {
                                    <a href={format!("tel:{}", phone)} class="phone-link">
                                        {phone.clone()}
                                    </a>
                                }
                            } else if let Some(phone_fixed) = &props.package.phone_fixed {
                                html! {
                                    <a href={format!("tel:{}", phone_fixed)} class="phone-link">
                                        {phone_fixed.clone()}
                                    </a>
                                }
                            } else {
                                html! { <span class="empty-value">{get_text("not_provided")}</span> }
                            }}
                        </div>
                    </div>

                    // Codes de porte
                    <div class="detail-section editable">
                        <div class="detail-label">{get_text("door_codes")}</div>
                        <div class="detail-value-with-action">
                            <span class="empty-value">{get_text("not_provided")}</span>
                            <button 
                                class="btn-icon-edit" 
                                title={get_text("modify")}
                                onclick={on_edit_door_code}
                            >
                                {"✏️"}
                            </button>
                        </div>
                    </div>

                    // BAL
                    <div class="detail-section editable">
                        <div class="detail-label">{get_text("mailbox_access")}</div>
                        <div class="detail-value-with-action">
                            <span class="empty-value">{get_text("not_provided")}</span>
                            <button 
                                class="btn-icon-edit" 
                                title={get_text("modify")}
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
                        <div class="detail-label">{get_text("client_instructions")}</div>
                        <div class="detail-value-with-action">
                            {if let Some(instructions) = &props.package.instructions {
                                html! { <span>{format!("\"{}\"", instructions)}</span> }
                            } else {
                                html! { <span class="empty-value">{get_text("not_provided")}</span> }
                            }}
                            <button 
                                class="btn-icon-edit" 
                                title={get_text("modify")}
                                onclick={on_edit_client_notes}
                            >
                                {"✏️"}
                            </button>
                        </div>
                    </div>

                    // Notes chauffeur
                    <div class="detail-section editable">
                        <div class="detail-label">{get_text("driver_notes")}</div>
                        <div class="detail-value-with-action">
                            <span class="empty-value">{get_text("add_note")}</span>
                            <button 
                                class="btn-icon-edit" 
                                title={get_text("modify")}
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

#[derive(serde::Deserialize)]
struct GeocodeResponse {
    success: bool,
    latitude: Option<f64>,
    longitude: Option<f64>,
    formatted_address: Option<String>,
    message: Option<String>,
}

async fn geocode_address(address: String) -> Result<GeocodeResponse, String> {
    let url = "https://api.delivery.nexuslabs.one/address/geocode";
    let body = serde_json::json!({ "address": address });
    
    let response = Request::post(url)
        .header("Content-Type", "application/json")
        .json(&body)
        .map_err(|e| format!("Failed to create request: {:?}", e))?
        .send()
        .await
        .map_err(|e| format!("Request failed: {:?}", e))?;
    
    if !response.ok() {
        return Err(format!("HTTP error: {}", response.status()));
    }
    
    let json: serde_json::Value = response
        .json()
        .await
        .map_err(|e| format!("Failed to parse JSON: {:?}", e))?;
    
    // Extraer data del response
    if let Some(data) = json.get("data") {
        serde_json::from_value(data.clone()).map_err(|e| format!("Failed to parse response data: {}", e))
    } else {
        Err("No data in response".to_string())
    }
}

