use yew::prelude::*;
use crate::models::package::Package;

#[derive(Properties, PartialEq, Clone)]
pub struct PackageCardProps {
    pub package: Package,
    pub index: usize,
    pub address: Option<String>,
    pub on_info: Callback<String>,
    #[prop_or(false)]
    pub is_selected: bool,
    #[prop_or(false)]
    pub is_expanded: bool, // Para mostrar paquetes internos si es grupo
    #[prop_or_default]
    pub on_select: Option<Callback<usize>>, // click en el card
    #[prop_or_default]
    pub on_navigate: Option<Callback<usize>>, // botón "ir"
    #[prop_or_default]
    pub on_toggle_expand: Option<Callback<usize>>, // toggle para expandir/colapsar
    #[prop_or_default]
    pub animation_class: Option<String>, // moving-up, moving-down, moved
}

#[function_component(PackageCard)]
pub fn package_card(props: &PackageCardProps) -> Html {
    let p = &props.package;

    let number_color_class = match p.status.as_str() {
        s if s.contains("RECEPTIONNER") => "package-number-yellow",
        s if s.contains("LIVRER") => "package-number-green",
        s if s.contains("NONLIV") => "package-number-red",
        _ => "package-number-normal",
    };

    let type_class = match p.delivery_type {
        crate::models::package::DeliveryType::PickupPoint => "type-relais",
        crate::models::package::DeliveryType::Rcs => "type-rcs",
        _ => "type-domicile",
    };

    let card_classes = classes!(
        "package-card",
        type_class,
        p.is_problematic.then_some("problematic"),
        props.is_selected.then_some("selected"),
        props.animation_class.as_ref(),
    );

    let on_card_click = {
        let cb = props.on_select.clone();
        let idx = props.index;
        Callback::from(move |_| {
            if let Some(sel) = &cb { sel.emit(idx); }
        })
    };

    let on_info_click = {
        let tracking = p.tracking.clone();
        let cb = props.on_info.clone();
        Callback::from(move |e: MouseEvent| {
            e.stop_propagation();
            cb.emit(tracking.clone());
        })
    };

    let on_expand_click = {
        let cb = props.on_toggle_expand.clone();
        let idx = props.index;
        Callback::from(move |e: MouseEvent| {
            e.stop_propagation();
            if let Some(toggle) = &cb { toggle.emit(idx); }
        })
    };

    html! {
        <div class={card_classes} onclick={on_card_click} data-index={props.index.to_string()}>
            <div class="package-header">
                <div class={classes!("package-number", number_color_class)}>
                    {format!("{}", props.index + 1)}
                </div>
                <button
                    class="btn-info"
                    onclick={on_info_click}
                >
                    {"i"}
                </button>
            </div>
            <div class="package-main">
                <div class="package-info">
                    <div class="package-recipient-row">
                    <div class="package-recipient">{p.customer_name.clone()}</div>
                        {
                            if props.is_selected {
                                if let Some(on_nav) = &props.on_navigate {
                                    let cb = on_nav.clone();
                                    let idx = props.index;
                                    html! {
                                        <button
                                            class="btn-navigate"
                                            onclick={Callback::from(move |e: MouseEvent| {
                                                e.stop_propagation();
                                                cb.emit(idx);
                                            })}
                                        >
                                            {"Go"}
                                        </button>
                                    }
                                } else {
                                    html!{}
                                }
                            } else {
                                html!{}
                            }
                        }
                    </div>
                    {
                        if let Some(addr) = props.address.clone() {
                            html!{ <div class="package-address">{addr}</div> }
                        } else { html!{} }
                    }
                </div>
            </div>
            
            // PAQUETES EXPANDIDOS - solo para grupos Y cuando está expandido
            if p.is_group && props.is_expanded {
                if let Some(packages) = &p.group_packages {
                    <div class="packages-expanded">
                        { for packages.iter().enumerate().map(|(idx, pkg)| {
                            let pkg_status_color = match pkg.status.as_str() {
                                s if s.contains("RECEPTIONNER") => "package-number-yellow",
                                s if s.contains("LIVRER") => "package-number-green",
                                s if s.contains("NONLIV") => "package-number-red",
                                _ => "package-number-normal",
                            };
                            
                            let pkg_type_class = match pkg.delivery_type {
                                crate::models::package::DeliveryType::PickupPoint => "type-relais",
                                crate::models::package::DeliveryType::Rcs => "type-rcs",
                                _ => "type-domicile",
                            };
                            
                            html! {
                                <div class={classes!("package-item", pkg_type_class)} key={pkg.tracking.clone()}>
                                    <div class="package-item-header">
                                        <span class={classes!("package-number", pkg_status_color)}>
                                            {idx + 1}
                                        </span>
                                        
                                        {
                                            match pkg.delivery_type {
                                                // RELAIS: Solo tracking
                                                crate::models::package::DeliveryType::PickupPoint => {
                                                    html! {
                                                        <div class="package-item-content">
                                                            <span class="tracking-label">{"📦 "}</span>
                                                            <span class="tracking-value">{&pkg.tracking}</span>
                                                        </div>
                                                    }
                                                },
                                                // NO RELAIS: Nombre del cliente + botón detalles
                                                _ => {
                                                    html! {
                                                        <>
                                                            <strong class="package-customer">{&pkg.customer_name}</strong>
                                                            <button
                                                                class="btn-package-details"
                                                                onclick={{
                                                                    let tracking = pkg.tracking.clone();
                                                                    let on_info = props.on_info.clone();
                                                                    Callback::from(move |e: MouseEvent| {
                                                                        e.stop_propagation();
                                                                        on_info.emit(tracking.clone());
                                                                    })
                                                                }}
                                                            >
                                                                {"i"}
                                                            </button>
                                                        </>
                                                    }
                                                }
                                            }
                                        }
                                    </div>
                                </div>
                            }
                        })}
                    </div>
                }
            }
            
            // EXPAND HANDLE - solo para grupos Y cuando está seleccionado
            // ⬇️ AHORA ESTÁ AL FINAL, después de packages-expanded
            if p.is_group && props.is_selected {
                <div 
                    class={classes!("expand-handle", (!props.is_expanded).then_some("pulse"))}
                    onclick={on_expand_click.clone()}
                >
                    <div class="expand-indicator"></div>
                </div>
            }
        </div>
    }
}


