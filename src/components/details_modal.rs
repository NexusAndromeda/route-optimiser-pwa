use yew::prelude::*;
use crate::models::Package;
use crate::context::get_text;
use web_sys::window;
use gloo_net::http::Request;
use wasm_bindgen::prelude::*;
use wasm_bindgen_futures::spawn_local;

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_name = updatePackageCoordinates)]
    fn update_package_coordinates(package_id: &str, latitude: f64, longitude: f64) -> bool;
    
    #[wasm_bindgen(js_name = removePackageFromMap)]
    fn remove_package_from_map(package_id: &str) -> bool;
}

#[derive(Properties, PartialEq)]
pub struct DetailsModalProps {
    pub package: Package,
    pub on_close: Callback<()>,
    pub on_edit_bal: Callback<()>,
    pub on_update_package: Callback<(String, f64, f64, String)>, // (id, lat, lng, new_address)
    pub on_mark_problematic: Callback<String>, // (package_id)
}

#[function_component(DetailsModal)]
pub fn details_modal(props: &DetailsModalProps) -> Html {
    // Debug: log package data
    log::info!("🔍 DetailsModal - Package ID: {}", props.package.id);
    log::info!("🔍 DetailsModal - door_code: {:?}", props.package.door_code);
    log::info!("🔍 DetailsModal - has_mailbox_access: {}", props.package.has_mailbox_access);
    log::info!("🔍 DetailsModal - driver_notes: {:?}", props.package.driver_notes);
    
    let close = props.on_close.clone();
    let close_overlay = props.on_close.clone();
    
    // Handler para geocodificación de dirección
    let package_id = props.package.id.clone();
    let on_street_settings = {
        let package_id = package_id.clone();
        let on_update = props.on_update_package.clone();
        let on_mark_problematic = props.on_mark_problematic.clone();
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
                                        
                                        // Actualizar el paquete en el mapa
                                        if update_package_coordinates(&package_id, lat, lng) {
                                            log::info!("📍 Coordonnées mises à jour sur la carte: {}", package_id);
                                            
                                            // Enviar corrección al backend
                                            let package_id_for_backend = package_id.clone();
                                            let formatted_for_backend = formatted.clone();
                                            wasm_bindgen_futures::spawn_local(async move {
                                                match send_address_correction_to_backend(
                                                    package_id_for_backend.clone(),
                                                    formatted_for_backend.clone(),
                                                    lat,
                                                    lng,
                                                    None, // door_code
                                                    None, // has_mailbox_access
                                                    None, // driver_notes
                                                ).await {
                                                    Ok(_) => {
                                                        log::info!("✅ Corrección enviada al backend: {}", package_id_for_backend);
                                                    }
                                                    Err(e) => {
                                                        log::error!("❌ Error enviando corrección al backend: {}", e);
                                                    }
                                                }
                                            });
                                            
                                            // Actualizar el paquete en el estado de Yew
                                            on_update.emit((package_id.clone(), lat, lng, formatted));
                                        } else {
                                            log::error!("❌ Échec de la mise à jour des coordonnées sur la carte");
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
    let package_id_1 = props.package.id.clone();
    let address_1 = props.package.address.clone();
    let coords_1 = props.package.coords.clone();
    let has_mailbox_access_1 = props.package.has_mailbox_access;
    let driver_notes_1 = props.package.driver_notes.clone();
    let door_code_1 = props.package.door_code.clone();
    
    let on_edit_door_code = Callback::from(move |e: MouseEvent| {
        e.stop_propagation();
        if let Some(win) = window() {
            if let Ok(Some(value)) = win.prompt_with_message(&get_text("edit_door_code")) {
                let trimmed_value = value.trim().to_string();
                if !trimmed_value.is_empty() {
                    // Enviar al backend
                    let package_id = package_id_1.clone();
                    let address = address_1.clone();
                    let coords = coords_1.clone();
                    let has_mailbox_access = has_mailbox_access_1;
                    let driver_notes = driver_notes_1.clone();
                    
                    spawn_local(async move {
                        let [lat, lng] = coords.unwrap_or([0.0, 0.0]);
                        let trimmed_value_clone = trimmed_value.clone();
                        match send_address_correction_to_backend(
                            package_id,
                            address,
                            lat,
                            lng,
                            Some(trimmed_value),
                            Some(has_mailbox_access),
                            driver_notes,
                        ).await {
                            Ok(_) => {
                                log::info!("✅ Code de porte envoyé au backend: {}", trimmed_value_clone);
                                if let Some(win) = window() {
                                    let _ = win.alert_with_message(&format!("✅ Code de porte enregistré:\n{}", trimmed_value_clone));
                                }
                            }
                            Err(e) => {
                                log::error!("❌ Erreur lors de l'envoi du code de porte: {}", e);
                                if let Some(win) = window() {
                                    let _ = win.alert_with_message(&format!("❌ Erreur lors de l'enregistrement: {}", e));
                                }
                            }
                        }
                    });
                }
            }
        }
    });
    
    // Handler para editar indicaciones cliente
    let package_id_2 = props.package.id.clone();
    let address_2 = props.package.address.clone();
    let coords_2 = props.package.coords.clone();
    let has_mailbox_access_2 = props.package.has_mailbox_access;
    let driver_notes_2 = props.package.driver_notes.clone();
    let door_code_2 = props.package.door_code.clone();
    
    let on_edit_client_notes = Callback::from(move |e: MouseEvent| {
        e.stop_propagation();
        if let Some(win) = window() {
            if let Ok(Some(value)) = win.prompt_with_message(&get_text("edit_client_instructions")) {
                let trimmed_value = value.trim().to_string();
                if !trimmed_value.is_empty() {
                    // Enviar al backend
                    let package_id = package_id_2.clone();
                    let address = address_2.clone();
                    let coords = coords_2.clone();
                    let door_code = door_code_2.clone();
                    let has_mailbox_access = has_mailbox_access_2;
                    
                    spawn_local(async move {
                        let [lat, lng] = coords.unwrap_or([0.0, 0.0]);
                        let trimmed_value_clone = trimmed_value.clone();
                        match send_address_correction_to_backend(
                            package_id,
                            address,
                            lat,
                            lng,
                            door_code,
                            Some(has_mailbox_access),
                            Some(trimmed_value),
                        ).await {
                            Ok(_) => {
                                log::info!("✅ Indications client envoyées au backend: {}", trimmed_value_clone);
                                if let Some(win) = window() {
                                    let _ = win.alert_with_message(&format!("✅ Indications du client enregistré:\n{}", trimmed_value_clone));
                                }
                            }
                            Err(e) => {
                                log::error!("❌ Erreur lors de l'envoi des indications client: {}", e);
                                if let Some(win) = window() {
                                    let _ = win.alert_with_message(&format!("❌ Erreur lors de l'enregistrement: {}", e));
                                }
                            }
                        }
                    });
                }
            }
        }
    });
    
    // Handler para editar notas del chauffeur
    let package_id_3 = props.package.id.clone();
    let address_3 = props.package.address.clone();
    let coords_3 = props.package.coords.clone();
    let has_mailbox_access_3 = props.package.has_mailbox_access;
    let driver_notes_3 = props.package.driver_notes.clone();
    let door_code_3 = props.package.door_code.clone();
    
    let on_edit_driver_notes = Callback::from(move |e: MouseEvent| {
        e.stop_propagation();
        if let Some(win) = window() {
            if let Ok(Some(value)) = win.prompt_with_message(&get_text("edit_driver_notes")) {
                let trimmed_value = value.trim().to_string();
                if !trimmed_value.is_empty() {
                    // Enviar al backend
                    let package_id = package_id_3.clone();
                    let address = address_3.clone();
                    let coords = coords_3.clone();
                    let door_code = door_code_3.clone();
                    let has_mailbox_access = has_mailbox_access_3;
                    
                    spawn_local(async move {
                        let [lat, lng] = coords.unwrap_or([0.0, 0.0]);
                        let trimmed_value_clone = trimmed_value.clone();
                        match send_address_correction_to_backend(
                            package_id,
                            address,
                            lat,
                            lng,
                            door_code,
                            Some(has_mailbox_access),
                            Some(trimmed_value),
                        ).await {
                            Ok(_) => {
                                log::info!("✅ Notes chauffeur envoyées au backend: {}", trimmed_value_clone);
                                if let Some(win) = window() {
                                    let _ = win.alert_with_message(&format!("✅ Notes du chauffeur enregistré:\n{}", trimmed_value_clone));
                                }
                            }
                            Err(e) => {
                                log::error!("❌ Erreur lors de l'envoi des notes chauffeur: {}", e);
                                if let Some(win) = window() {
                                    let _ = win.alert_with_message(&format!("❌ Erreur lors de l'enregistrement: {}", e));
                                }
                            }
                        }
                    });
                }
            }
        }
    });
    
    // Handler para editar acceso al buzón (BAL)
    let package_id_4 = props.package.id.clone();
    let address_4 = props.package.address.clone();
    let coords_4 = props.package.coords.clone();
    let has_mailbox_access_4 = props.package.has_mailbox_access;
    let driver_notes_4 = props.package.driver_notes.clone();
    let door_code_4 = props.package.door_code.clone();
    
    let on_edit_bal = Callback::from(move |e: MouseEvent| {
        e.stop_propagation();
        if let Some(win) = window() {
            let current_value = has_mailbox_access_4;
            let message = if current_value {
                "Accès boîte aux lettres (BAL):\n\nCliquez OK pour DÉSACTIVER l'accès\nCliquez Annuler pour garder l'accès"
            } else {
                "Accès boîte aux lettres (BAL):\n\nCliquez OK pour ACTIVER l'accès\nCliquez Annuler pour ne pas donner accès"
            };
            
            // confirm() retorna true si el usuario hace clic en OK, false si hace clic en Cancelar
            if win.confirm_with_message(message).unwrap_or(false) {
                // Usuario hizo clic en OK → invertir el estado actual
                let new_value = !current_value;
                
                log::info!("📬 Usuario cambió accès BAL: {} → {}", current_value, new_value);
                
                // Enviar al backend
                let package_id = package_id_4.clone();
                let address = address_4.clone();
                let coords = coords_4.clone();
                let door_code = door_code_4.clone();
                let driver_notes = driver_notes_4.clone();
                
                spawn_local(async move {
                    let [lat, lng] = coords.unwrap_or([0.0, 0.0]);
                    match send_address_correction_to_backend(
                        package_id,
                        address,
                        lat,
                        lng,
                        door_code,
                        Some(new_value),
                        driver_notes,
                    ).await {
                        Ok(_) => {
                            log::info!("✅ Accès buzón envoyé au backend: {}", new_value);
                            if let Some(win) = window() {
                                let status = if new_value { "ACTIVÉ ✅" } else { "DÉSACTIVÉ ❌" };
                                let _ = win.alert_with_message(&format!("✅ Accès boîte aux lettres {}!", status));
                            }
                        }
                        Err(e) => {
                            log::error!("❌ Erreur lors de l'envoi de l'accès buzón: {}", e);
                            if let Some(win) = window() {
                                let _ = win.alert_with_message(&format!("❌ Erreur lors de l'enregistrement: {}", e));
                            }
                        }
                    }
                });
            } else {
                log::info!("❌ Usuario canceló el cambio de accès BAL");
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
                            {if let Some(door_code) = &props.package.door_code {
                                html! { <span>{door_code}</span> }
                            } else {
                                html! { <span class="empty-value">{get_text("not_provided")}</span> }
                            }}
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
                            <span>{if props.package.has_mailbox_access { "✅ Oui" } else { "❌ Non" }}</span>
                            <button 
                                class="btn-icon-edit" 
                                title={get_text("modify")}
                                onclick={on_edit_bal}
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
                            {if let Some(notes) = &props.package.driver_notes {
                                if !notes.is_empty() {
                                    html! { <span>{format!("\"{}\"", notes)}</span> }
                                } else {
                                    html! { <span class="empty-value">{get_text("add_note")}</span> }
                                }
                            } else {
                                html! { <span class="empty-value">{get_text("add_note")}</span> }
                            }}
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
    let url = "https://api.delivery.nexuslabs.one/api/address/geocode";
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

async fn send_address_correction_to_backend(
    tracking: String,
    address: String,
    latitude: f64,
    longitude: f64,
    door_code: Option<String>,
    has_mailbox_access: Option<bool>,
    driver_notes: Option<String>,
) -> Result<(), String> {
    let url = "https://api.delivery.nexuslabs.one/address/update";
    let body = serde_json::json!({
        "tracking": tracking,
        "address": address,
        "latitude": latitude,
        "longitude": longitude,
        "door_code": door_code,
        "driver_notes": driver_notes,
        "has_mailbox_access": has_mailbox_access
    });
    
    log::info!("📤 Enviando corrección al backend: {} -> {}", tracking, address);
    
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
    
    // Verificar si la respuesta indica éxito
    if let Some(success) = json.get("success") {
        if success.as_bool().unwrap_or(false) {
            log::info!("✅ Corrección guardada en backend exitosamente");
            Ok(())
        } else {
            Err("Backend returned success: false".to_string())
        }
    } else {
        Err("No success field in response".to_string())
    }
}
