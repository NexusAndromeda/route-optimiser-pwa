use yew::prelude::*;
use crate::models::{Address, LegacyPackage, DeliveryType};
use crate::context::get_text;
use std::collections::HashMap;

/// Mapea el tipo de entrega a un color
fn get_delivery_type_color(delivery_type: &DeliveryType) -> &'static str {
    match delivery_type {
        DeliveryType::Home => "type-domicile",
        DeliveryType::Rcs => "type-rcs", 
        DeliveryType::PickupPoint => "type-relais",
    }
}

/// Mapea el status a un color para el número de paquete
fn get_package_status_color(status: &str) -> &'static str {
    match status {
        // Estados de recepción/carga
        "STATUT_RECEPTIONNER" => "yellow",
        "STATUT_CHARGER" => "normal",
        
        // Estados de entrega exitosa (verde)
        "STATUT_LIVRER_DOMICILE" => "green",
        "STATUT_LIVRER_TIERS" => "green",
        "STATUT_LIVRER_BAL" => "green",
        "STATUT_LIVRER_LOCKER" => "green",
        s if s.starts_with("STATUT_LIVRER_") => "green",
        
        // Estados de no entrega (rojo)
        "STATUT_NONLIV_NPAI" => "red",
        "STATUT_NONLIV_ABS" => "red",
        "STATUT_NONLIV_REFUS" => "red",
        "STATUT_NONLIV_ERRADRESSE" => "red",
        s if s.starts_with("STATUT_NONLIV_") => "red",
        
        // Estado por defecto
        _ => "normal",
    }
}

#[derive(Properties, PartialEq)]
pub struct AddressCardProps {
    pub address: Address,
    pub packages: Vec<LegacyPackage>,
    pub is_selected: bool,
    pub is_expanded: bool,
    pub on_select: Callback<String>, // address_id
    pub on_toggle_expand: Callback<String>, // address_id
    pub on_show_package_details: Callback<LegacyPackage>,
    pub on_navigate: Callback<String>, // address_id
    pub reorder_mode: bool,
    pub is_reorder_origin: bool,
    pub animation_class: Option<String>,
}

#[function_component(AddressCard)]
pub fn address_card(props: &AddressCardProps) -> Html {
    let address_id = props.address.address_id.clone();
    let package_count = props.packages.len();
    let is_first = false; // TODO: Implementar lógica de orden
    let is_last = false; // TODO: Implementar lógica de orden
    
    let onclick = {
        let on_select = props.on_select.clone();
        let address_id = address_id.clone();
        Callback::from(move |_| on_select.emit(address_id.clone()))
    };
    
    // Determinar el tipo de entrega predominante
    let main_delivery_type = props.packages.first()
        .and_then(|p| p.type_livraison.as_ref())
        .map(|t| match t.as_str() {
            "RELAIS" => DeliveryType::PickupPoint,
            "RCS" => DeliveryType::Rcs,
            _ => DeliveryType::Home,
        })
        .unwrap_or(DeliveryType::Home);
    
    let type_class = get_delivery_type_color(&main_delivery_type);
    
    // Contar paquetes por status
    let status_counts: HashMap<String, usize> = props.packages.iter()
        .fold(HashMap::new(), |mut acc, pkg| {
            *acc.entry(pkg.status.clone()).or_insert(0) += 1;
            acc
        });
    
    // Determinar si hay paquetes problemáticos
    let has_problematic = props.packages.iter().any(|p| p.is_problematic);
    
    let card_class = classes!(
        "address-card",
        props.is_selected.then_some("selected"),
        props.animation_class.as_ref(),
        props.reorder_mode.then_some("reorder-mode-active"),
        props.is_reorder_origin.then_some("reorder-origin"),
        has_problematic.then_some("problematic"),
        type_class
    );
    
    html! {
        <div class={card_class} {onclick}>
            // Header: número de paquetes + info (detalles)
            <div class="address-header">
                <div class="address-number">
                    {package_count}
                </div>
                <button
                    class="btn-info"
                    onclick={{
                        let on_toggle = props.on_toggle_expand.clone();
                        let address_id = address_id.clone();
                        Callback::from(move |e: MouseEvent| {
                            e.stop_propagation();
                            on_toggle.emit(address_id.clone());
                        })
                    }}
                >
                    {"i"}
                </button>
            </div>
            
            // Contenido principal: dirección + info
            <div class="address-main">
                <div class="address-info">
                    <div class="address-label">
                        {&props.address.label}
                    </div>
                    <div class="address-details">
                        <div class="address-coords">
                            {format!("📍 {:.6}, {:.6}", props.address.latitude, props.address.longitude)}
                        </div>
                        if props.address.door_code.is_some() || props.address.mailbox_access || !props.address.driver_notes.is_empty() {
                            <div class="address-extras">
                                if let Some(door_code) = &props.address.door_code {
                                    <span class="door-code">{"🚪 "}{door_code}</span>
                                }
                                if props.address.mailbox_access {
                                    <span class="mailbox">{"📬 BAL"}</span>
                                }
                                if !props.address.driver_notes.is_empty() {
                                    <span class="notes">{"📝 "}{&props.address.driver_notes}</span>
                                }
                            </div>
                        }
                    </div>
                </div>
                
                // Botones de acción (solo cuando está seleccionado y NO en modo reordenar)
                if props.is_selected && !props.reorder_mode {
                    <div class="address-actions">
                        <button
                            class="btn-navigate"
                            onclick={{
                                let on_navigate = props.on_navigate.clone();
                                let address_id = address_id.clone();
                                Callback::from(move |e: MouseEvent| {
                                    e.stop_propagation();
                                    on_navigate.emit(address_id.clone());
                                })
                            }}
                        >
                            {get_text("go")}
                        </button>
                    </div>
                }
            </div>
            
            // Mostrar paquetes si está expandido
            if props.is_expanded {
                <div class="packages-expanded">
                    <div class="packages-list">
                        { for props.packages.iter().enumerate().map(|(idx, package)| {
                            let status_color = get_package_status_color(
                                package.code_statut_article.as_ref().map(|s| s.as_str()).unwrap_or("")
                            );
                            let delivery_type_class = package.type_livraison.as_ref()
                                .map(|t| match t.as_str() {
                                    "RELAIS" => "type-relais",
                                    "RCS" => "type-rcs",
                                    _ => "type-domicile",
                                })
                                .unwrap_or("type-domicile");
                            
                            html! {
                                <>
                                    <div class={classes!("package-item", delivery_type_class)} key={package.id.clone()}>
                                        <div class="package-header">
                                            <span class={classes!("package-number", format!("package-number-{}", status_color))}>
                                                {idx + 1}
                                            </span>
                                            <strong class="package-customer">{&package.recipient}</strong>
                                        </div>
                                        <div class="package-details">
                                            <div class="package-info">
                                                <div class="package-tracking">
                                                    <span class="tracking-label">{"📦 ID:"}</span>
                                                    <span class="tracking-value">{&package.id}</span>
                                                </div>
                                                if let Some(phone) = &package.phone {
                                                    <div class="package-phone">
                                                        <span class="phone-label">{"📞"}</span>
                                                        <span class="phone-value">{phone}</span>
                                                    </div>
                                                }
                                                if let Some(instructions) = &package.instructions {
                                                    <div class="package-indication">
                                                        <span class="indication-label">{"💬"}</span>
                                                        <span class="indication-value">{instructions}</span>
                                                    </div>
                                                }
                                                <div class="package-status">
                                                    <span class="status-label">{"Status:"}</span>
                                                    <span class="status-value">{&package.status}</span>
                                                </div>
                                            </div>
                                            // Botón Detalles para cada paquete
                                            <button
                                                class="btn-package-details"
                                                onclick={{
                                                    let pkg_clone = package.clone();
                                                    let on_show_package_details = props.on_show_package_details.clone();
                                                    Callback::from(move |e: MouseEvent| {
                                                        e.stop_propagation();
                                                        on_show_package_details.emit(pkg_clone.clone());
                                                    })
                                                }}
                                            >
                                                {"i"}
                                            </button>
                                        </div>
                                    </div>
                                    // Separador entre paquetes (excepto el último)
                                    if idx < props.packages.len() - 1 {
                                        <div class="package-divider"></div>
                                    }
                                </>
                            }
                        })}
                    </div>
                </div>
            }
            
            // Botón expandir discreto (solo cuando está seleccionado)
            if props.is_selected {
                <div 
                    class={classes!("expand-handle", if !props.is_expanded { Some("pulse") } else { None })}
                    onclick={{
                        let on_toggle = props.on_toggle_expand.clone();
                        let address_id = address_id.clone();
                        Callback::from(move |e: MouseEvent| {
                            e.stop_propagation();
                            on_toggle.emit(address_id.clone());
                        })
                    }}
                >
                    <div class="expand-handle-line"></div>
                </div>
            }
        </div>
    }
}
