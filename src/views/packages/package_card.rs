use yew::prelude::*;
use crate::models::Package;
use crate::context::get_text;

/// Mapea el code_statut_article a un color para el número de paquete
fn get_package_status_color(code_statut_article: &Option<String>) -> &'static str {
    match code_statut_article.as_deref() {
        Some("STATUT_RECEPTIONNER") => "yellow",
        Some("STATUT_CHARGER") => "normal",
        Some(s) if s.starts_with("STATUT_LIVRER_") => "green",
        Some(s) if s.starts_with("STATUT_NONLIV_") => "red",
        _ => "normal",
    }
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
    
    let card_class = classes!(
        "package-card",
        props.is_selected.then_some("selected"),
        props.animation_class.as_ref()
    );
    
    html! {
        <div class={card_class} {onclick}>
            // Header: número (con color) + info (detalles)
            <div class="package-header">
                <div class={classes!("package-number", format!("package-number-{}", status_color))}>
                    {index + 1}
                </div>
                // Info button (detalles)
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
                
                // Botones de acción (solo cuando está seleccionado)
                if props.is_selected {
                    <div class="package-actions">
                        // Reorder button
                        <button
                            class="btn-reorder-mode"
                            onclick={{
                                // TODO: Implementar modo reordenar
                                Callback::from(move |e: MouseEvent| {
                                    e.stop_propagation();
                                    log::info!("🔄 Modo reordenar activado para paquete {}", index);
                                })
                            }}
                        >
                            {"reordenar"}
                        </button>
                        
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
                            {"ir"}
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
                    class="group-expand-handle"
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

