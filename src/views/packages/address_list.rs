use yew::prelude::*;
use crate::models::{DeliverySession, Address, LegacyPackage};
use crate::context::get_text;
use super::AddressCard;
use std::collections::HashMap;

#[derive(Properties, PartialEq)]
pub struct AddressListProps {
    pub session: Option<DeliverySession>,
    pub selected_address_id: Option<String>,
    pub expanded_addresses: Vec<String>,
    pub on_select_address: Callback<String>,
    pub on_toggle_expand: Callback<String>,
    pub on_show_package_details: Callback<LegacyPackage>,
    pub on_navigate: Callback<String>,
    pub reorder_mode: bool,
    pub reorder_origin: Option<String>,
    pub animations: HashMap<String, String>,
    pub loading: bool,
}

#[function_component(AddressList)]
pub fn address_list(props: &AddressListProps) -> Html {
    let show_progress_bar = true; // TODO: Basado en sheet state
    
    // Calcular estadísticas
    let (total_packages, delivered_packages, percentage) = if let Some(session) = &props.session {
        let total = session.packages.len();
        let delivered = session.packages.values()
            .filter(|p| !p.status.starts_with("STATUT_CHARGER"))
            .count();
        let percentage = if total > 0 { (delivered * 100) / total } else { 0 };
        (total, delivered, percentage)
    } else {
        (0, 0, 0)
    };
    
    // Obtener direcciones ordenadas
    let ordered_addresses = if let Some(session) = &props.session {
        session.get_ordered_addresses()
    } else {
        Vec::new()
    };
    
    html! {
        <>
            // Drag Handle Container
            <div class="drag-handle-container">
                <div class="drag-handle"></div>
                
                // Progress Info
                <div class="progress-info">
                    <div class="progress-text">
                        <span class="progress-count">
                            {format!("✓ {}/{} {}", delivered_packages, total_packages, get_text("delivered"))}
                        </span>
                    </div>
                    <div class="progress-percentage">
                        <span>{format!("{}%", percentage)}</span>
                    </div>
                </div>
                
                // Progress Bar
                if show_progress_bar {
                    <div class="progress-bar-container">
                        <div class="progress-bar" style={format!("width: {}%", percentage)}></div>
                    </div>
                }
            </div>
            
            // Address List
            <div class="address-list">
                {
                    if props.loading {
                        html! {
                            <div class="no-addresses">
                                <div class="no-addresses-icon">{"⏳"}</div>
                                <div class="no-addresses-text">{format!("{}...", get_text("loading"))}</div>
                                <div class="no-addresses-subtitle">{get_text("please_wait")}</div>
                            </div>
                        }
                    } else if ordered_addresses.is_empty() {
                        html! {
                            <div class="no-addresses">
                                <div class="no-addresses-icon">{"📍"}</div>
                                <div class="no-addresses-text">{get_text("no_addresses")}</div>
                                <div class="no-addresses-subtitle">{get_text("addresses_after_login")}</div>
                            </div>
                        }
                    } else {
                        html! {
                            <>
                                { for ordered_addresses.iter().map(|address| {
                                    let address_id = address.address_id.clone();
                                    let is_selected = props.selected_address_id == Some(address_id.clone());
                                    let is_expanded = props.expanded_addresses.contains(&address_id);
                                    let is_reorder_origin = props.reorder_origin == Some(address_id.clone());
                                    let animation_class = props.animations.get(&address_id).cloned();
                                    
                    // Obtener paquetes para esta dirección
                    let packages = if let Some(session) = &props.session {
                        address.package_ids.iter()
                            .filter_map(|pkg_id| session.packages.get(pkg_id))
                            .map(|pkg| {
                                // Convert delivery_session::Package to LegacyPackage
                                LegacyPackage {
                                    id: pkg.internal_id.clone(),
                                    tracking: Some(pkg.tracking.clone()),
                                    recipient: pkg.customer_name.clone(),
                                    address: address.label.clone(), // Use the address label
                                    status: pkg.status.clone(),
                                    code_statut_article: Some(pkg.status.clone()),
                                    coords: Some([address.latitude, address.longitude]), // Use real coordinates
                                    phone: pkg.phone_number.clone(),
                                    phone_fixed: None, // Not available in delivery_session::Package
                                    instructions: pkg.customer_indication.clone(),
                                    door_code: address.door_code.clone(),
                                    has_mailbox_access: address.mailbox_access,
                                    driver_notes: Some(address.driver_notes.clone()),
                                    is_group: false,
                                    total_packages: None,
                                    group_packages: None,
                                    is_problematic: pkg.is_problematic,
                                    type_livraison: Some(match pkg.delivery_type {
                                        crate::models::DeliveryType::Home => "DOMICILE".to_string(),
                                        crate::models::DeliveryType::Rcs => "RCS".to_string(),
                                        crate::models::DeliveryType::PickupPoint => "RELAIS".to_string(),
                                    }),
                                }
                            })
                            .collect::<Vec<LegacyPackage>>()
                    } else {
                        Vec::new()
                    };
                                    
                                    html! {
                                        <AddressCard
                                            key={address_id.clone()}
                                            address={address.clone()}
                                            packages={packages}
                                            is_selected={is_selected}
                                            is_expanded={is_expanded}
                                            on_select={props.on_select_address.clone()}
                                            on_toggle_expand={props.on_toggle_expand.clone()}
                                            on_show_package_details={props.on_show_package_details.clone()}
                                            on_navigate={props.on_navigate.clone()}
                                            reorder_mode={props.reorder_mode}
                                            is_reorder_origin={is_reorder_origin}
                                            animation_class={animation_class}
                                        />
                                    }
                                }) }
                            </>
                        }
                    }
                }
            </div>
        </>
    }
}
