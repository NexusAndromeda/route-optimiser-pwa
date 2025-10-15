use yew::prelude::*;
use serde::{Deserialize, Serialize};
use gloo_net::http::Request;
use wasm_bindgen_futures::spawn_local;
use web_sys::MouseEvent;

// Modelos de datos
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct SinglePackage {
    pub id: String,
    pub tracking: String,
    pub customer_name: String,
    pub phone_number: Option<String>,
    pub customer_indication: Option<String>,
    pub official_label: String,
    pub latitude: f64,
    pub longitude: f64,
    pub mailbox_access: bool,
    pub driver_notes: String,
    pub address_id: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct PackageInfo {
    pub id: String,
    pub tracking: String,
    pub customer_indication: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct CustomerGroup {
    pub customer_name: String,
    pub phone_number: Option<String>,
    pub packages: Vec<PackageInfo>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct DeliveryGroup {
    pub id: String,
    pub official_label: String,
    pub latitude: f64,
    pub longitude: f64,
    pub mailbox_access: bool,
    pub driver_notes: String,
    pub customers: Vec<CustomerGroup>,
    pub total_packages: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct GroupedPackages {
    pub singles: Vec<SinglePackage>,
    pub groups: Vec<DeliveryGroup>,
    pub total_packages: usize,
    pub total_addresses: usize,
}

#[derive(Debug, Clone, Serialize)]
struct GetPackagesRequest {
    matricule: String,
    societe: String,
    date: Option<String>,
}

pub struct UseGroupedPackagesHandle {
    pub singles: UseStateHandle<Vec<SinglePackage>>,
    pub groups: UseStateHandle<Vec<DeliveryGroup>>,
    pub loading: UseStateHandle<bool>,
    pub error: UseStateHandle<Option<String>>,
    pub selected_index: UseStateHandle<Option<usize>>,
    pub selected_type: UseStateHandle<Option<String>>, // "single" o "group"
    pub expanded_groups: UseStateHandle<Vec<String>>, // IDs de groups expandidos
    
    pub fetch_packages: Callback<MouseEvent>,
    pub select_point: Callback<(usize, String)>, // (index, type)
    pub toggle_group: Callback<String>, // group_id
}

#[hook]
pub fn use_grouped_packages(login_data: Option<(String, String)>) -> UseGroupedPackagesHandle {
    let singles = use_state(|| Vec::<SinglePackage>::new());
    let groups = use_state(|| Vec::<DeliveryGroup>::new());
    let loading = use_state(|| false);
    let error = use_state(|| None::<String>);
    let selected_index = use_state(|| None::<usize>);
    let selected_type = use_state(|| None::<String>);
    let expanded_groups = use_state(|| Vec::<String>::new());
    
    // Fetch packages
    let fetch_packages = {
        let singles = singles.clone();
        let groups = groups.clone();
        let loading = loading.clone();
        let error = error.clone();
        let login_data = login_data.clone();
        
        Callback::from(move |_: MouseEvent| {
            let singles = singles.clone();
            let groups = groups.clone();
            let loading = loading.clone();
            let error = error.clone();
            
            if let Some((societe, matricule)) = login_data.clone() {
                loading.set(true);
                error.set(None);
                
                spawn_local(async move {
                    log::info!("📦 Obteniendo paquetes agrupados para: {}:{}", societe, matricule);
                    
                    let request_body = GetPackagesRequest {
                        matricule,
                        societe,
                        date: Some(chrono::Utc::now().format("%Y-%m-%d").to_string()),
                    };
                    
                    match Request::post("https://api.delivery.nexuslabs.one/api/packages/grouped")
                        .json(&request_body)
                        .unwrap()
                        .send()
                        .await
                    {
                        Ok(response) => {
                            if response.ok() {
                                match response.json::<GroupedPackages>().await {
                                    Ok(grouped) => {
                                        log::info!("✅ {} singles, {} groups obtenidos", 
                                            grouped.singles.len(), grouped.groups.len());
                                        singles.set(grouped.singles);
                                        groups.set(grouped.groups);
                                    }
                                    Err(e) => {
                                        log::error!("❌ Error parseando respuesta: {:?}", e);
                                        error.set(Some(format!("Error parseando datos: {}", e)));
                                    }
                                }
                            } else {
                                log::error!("❌ Error HTTP: {}", response.status());
                                error.set(Some(format!("Error HTTP: {}", response.status())));
                            }
                        }
                        Err(e) => {
                            log::error!("❌ Error en request: {:?}", e);
                            error.set(Some(format!("Error de conexión: {}", e)));
                        }
                    }
                    
                    loading.set(false);
                });
            } else {
                error.set(Some("No hay datos de login disponibles".to_string()));
            }
        })
    };
    
    // Select point (single o group)
    let select_point = {
        let selected_index = selected_index.clone();
        let selected_type = selected_type.clone();
        
        Callback::from(move |(index, point_type): (usize, String)| {
            log::info!("📍 Seleccionando punto {} de tipo {}", index, point_type);
            selected_index.set(Some(index));
            selected_type.set(Some(point_type));
        })
    };
    
    // Toggle group expansion
    let toggle_group = {
        let expanded_groups = expanded_groups.clone();
        
        Callback::from(move |group_id: String| {
            let mut expanded = (*expanded_groups).clone();
            
            if let Some(pos) = expanded.iter().position(|id| id == &group_id) {
                // Ya está expandido, colapsar
                expanded.remove(pos);
                log::info!("📥 Colapsando group {}", group_id);
            } else {
                // No está expandido, expandir
                expanded.push(group_id.clone());
                log::info!("📤 Expandiendo group {}", group_id);
            }
            
            expanded_groups.set(expanded);
        })
    };
    
    UseGroupedPackagesHandle {
        singles,
        groups,
        loading,
        error,
        selected_index,
        selected_type,
        expanded_groups,
        fetch_packages,
        select_point,
        toggle_group,
    }
}

