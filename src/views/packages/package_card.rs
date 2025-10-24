use yew::prelude::*;
use crate::models::Package;
use crate::context::get_text;

/// Mapea el code_statut_article a un color para el número de paquete
fn get_package_status_color(code_statut_article: &Option<String>) -> &'static str {
    let color = match code_statut_article.as_deref() {
        Some("STATUT_RECEPTIONNER") => "yellow",
        Some("STATUT_CHARGER") => "normal",
        Some(s) if s.starts_with("STATUT_LIVRER_") => "green",
        Some(s) if s.starts_with("STATUT_NONLIV_") => "red",
        _ => "normal",
    };
    
    if let Some(code) = code_statut_article {
        log::info!("🎨 Code '{}' → color '{}'", code, color);
    } else {
        log::warn!("⚠️ Sin code_statut_article, usando color normal");
    }
    
    color
}

#[derive(Properties, PartialEq)]
pub struct PackageCardProps {
    pub index: usize,
    pub package: Package,
    pub is_selected: bool,
    pub on_select: Callback<usize>,
    pub on_show_details: Callback<usize>,
    pub on_navigate: Callback<usize>,
    pub on_reorder: Callback<(usize, String)>,
    pub total_packages: usize,
    #[prop_or_default]
    pub animation_class: Option<String>,
    #[prop_or_default]
    pub is_expanded: bool,
    #[prop_or_default]
    pub on_toggle_group: Option<Callback<String>>,
    #[prop_or_default]
    pub on_show_package_details: Option<Callback<Package>>,
    #[prop_or_default]
    pub reorder_mode: bool,
    #[prop_or_default]
    pub is_reorder_origin: bool,
}

#[function_component(PackageCard)]
pub fn package_card(props: &PackageCardProps) -> Html {
    let index = props.index;
    let is_first = index == 0;
    let is_last = index >= props.total_packages - 1;
    
    let onclick = {
        let on_select = props.on_select.clone();
        Callback::from(move |_| on_select.emit(index))
    };
    
    // Obtener el color del número basado en code_statut_article
    let status_color = get_package_status_color(&props.package.code_statut_article);
    
    // Obtener clase CSS para el tipo de entrega
    let type_class = props.package.type_livraison.as_ref().map(|t| {
        match t.as_str() {
            "RELAIS" => "type-relais",
            "RCS" => "type-rcs",
            "DOMICILE" => "type-domicile",
            _ => "type-domicile"
        }
    });
    
    let card_class = classes!(
        "package-card",
        props.is_selected.then_some("selected"),
        props.animation_class.as_ref(),
        props.reorder_mode.then_some("reorder-mode-active"),
        props.is_reorder_origin.then_some("reorder-origin"),
        props.package.is_problematic.then_some("problematic"),
        type_class
    );
    
    html! {
        <div class={card_class} {onclick}>
            // Header: número (con color) + info (detalles)
            <div class="package-header">
                <div class={classes!("package-number", format!("package-number-{}", status_color))}>
                    {index + 1}
                </div>
                // Info button (detalles) - solo para singles, NO para grupos
                if !props.package.is_group {
                    <button
                        class="btn-info"
                        onclick={{
                            let on_show_details = props.on_show_details.clone();
                            Callback::from(move |e: MouseEvent| {
                                e.stop_propagation();
                                on_show_details.emit(index);
                            })
                        }}
                    >
                        {"i"}
                    </button>
                }
            </div>
            
            // Contenido principal: cliente/dirección + botones
            <div class="package-main">
                <div class="package-info">
                    <div class="package-recipient">
                        {&props.package.recipient}
                    </div>
                    <div class="package-address">
                        {&props.package.address}
                    </div>
                </div>
                
                // Botones de acción (solo cuando está seleccionado y NO en modo reordenar)
                if props.is_selected && !props.reorder_mode {
                    <div class="package-actions">
                        // Navigate button
                        <button
                            class="btn-navigate"
                            onclick={{
                                let on_navigate = props.on_navigate.clone();
                                Callback::from(move |e: MouseEvent| {
                                    e.stop_propagation();
                                    on_navigate.emit(index);
                                })
                            }}
                        >
                            {get_text("go")}
                        </button>
                    </div>
                }
            </div>
            
            // Mostrar paquetes internos si está expandido (ABAJO de los botones)
            if props.package.is_group && props.is_expanded {
                if let Some(group_packages) = &props.package.group_packages {
                    <div class="group-packages-expanded">
                        <div class="group-packages-list">
                            { for group_packages.iter().enumerate().map(|(idx, pkg)| {
                                html! {
                                    <>
                                        <div class="group-package-item" key={pkg.id.clone()}>
                                            <div class="group-package-header">
                                                <span class="group-package-number">{format!("#{}", idx + 1)}</span>
                                                <strong class="group-package-customer">{&pkg.customer_name}</strong>
                                            </div>
                                            <div class="group-package-details">
                                                <div class="group-package-info">
                                                    <div class="group-package-tracking">
                                                        <span class="tracking-label">{"📦 Tracking:"}</span>
                                                        <span class="tracking-value">{&pkg.tracking}</span>
                                                    </div>
                                                    if let Some(phone) = &pkg.phone_number {
                                                        <div class="group-package-phone">
                                                            <span class="phone-label">{"📞"}</span>
                                                            <span class="phone-value">{phone}</span>
                                                        </div>
                                                    }
                                                    if let Some(indication) = &pkg.customer_indication {
                                                        <div class="group-package-indication">
                                                            <span class="indication-label">{"💬"}</span>
                                                            <span class="indication-value">{indication}</span>
                                                        </div>
                                                    }
                                                </div>
                                                // Botón Detalles para cada paquete del grupo
                                                <button
                                                    class="btn-group-package-details"
                                                    onclick={{
                                                    let pkg_clone = pkg.clone();
                                                    let group_address = props.package.address.clone();
                                                    let group_coords = props.package.coords;
                                                    let group_door_code = props.package.door_code.clone();
                                                    let group_mailbox_access = props.package.has_mailbox_access;
                                                    let group_driver_notes = props.package.driver_notes.clone();
                                                    let group_type_livraison = props.package.type_livraison.clone();
                                                    let on_show_package_details = props.on_show_package_details.clone();
                                                    Callback::from(move |e: MouseEvent| {
                                                        e.stop_propagation();
                                                        if let Some(callback) = &on_show_package_details {
                                                            // Crear Package completo desde GroupPackageInfo
                                                            let package = pkg_clone.to_package(
                                                                &group_address, 
                                                                group_coords,
                                                                group_door_code.clone(),
                                                                group_mailbox_access,
                                                                group_driver_notes.clone(),
                                                                group_type_livraison.clone()
                                                            );
                                                            callback.emit(package);
                                                        }
                                                    })
                                                    }}
                                                >
                                                    {"i"}
                                                </button>
                                            </div>
                                        </div>
                                        // Separador entre paquetes (excepto el último)
                                        if idx < group_packages.len() - 1 {
                                            <div class="group-package-divider"></div>
                                        }
                                    </>
                                }
                            })}
                        </div>
                    </div>
                }
            }
            
            // Botón expandir discreto para grupos (solo cuando está seleccionado)
            if props.package.is_group && props.is_selected {
                <div 
                    class={classes!("group-expand-handle", if !props.is_expanded { Some("pulse") } else { None })}
                    onclick={{
                        let package_id = props.package.id.clone();
                        let on_toggle = props.on_toggle_group.clone();
                        Callback::from(move |e: MouseEvent| {
                            e.stop_propagation();
                            if let Some(toggle_cb) = &on_toggle {
                                toggle_cb.emit(package_id.clone());
                            }
                        })
                    }}
                >
                    <div class="expand-handle-line"></div>
                </div>
            }
        </div>
    }
}

