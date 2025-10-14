use yew::prelude::*;
use crate::models::Package;
use crate::context::get_text;

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
    
    let status_class = match props.package.status.as_str() {
        "delivered" => "status-delivered",
        "pending" => "status-pending",
        _ => "status-pending",
    };
    
    let status_text = match props.package.status.as_str() {
        "delivered" => get_text("delivered"),
        "pending" => get_text("pending"),
        _ => get_text("pending"),
    };
    
    let card_class = classes!(
        "package-card",
        props.is_selected.then_some("selected"),
        props.animation_class.as_ref()
    );
    
    html! {
        <div class={card_class} {onclick}>
            <div class="package-main">
                <div class="package-header">
                    <div class="package-number">{index + 1}</div>
                    <div class={classes!("package-status", status_class)}>
                        <span class="status-icon">
                            {if props.package.status == "delivered" { "✓" } else { "⏳" }}
                        </span>
                        <span class="status-text">{status_text}</span>
                    </div>
                </div>
                <div class="package-info">
                    <div class="package-recipient">
                        {&props.package.recipient}
                    </div>
                    <div class="package-address">
                        {&props.package.address}
                    </div>
                </div>
                
                // Reorder Actions (solo cuando está seleccionado)
                if props.is_selected {
                    <div class="reorder-actions" style="animation: slideInUp 0.2s ease;">
                        // Reorder buttons
                        <div class="reorder-buttons">
                            <button
                                class="btn-reorder btn-up"
                                disabled={is_first}
                                onclick={{
                                    let on_reorder = props.on_reorder.clone();
                                    Callback::from(move |e: MouseEvent| {
                                        e.stop_propagation();
                                        if !is_first {
                                            on_reorder.emit((index, "up".to_string()));
                                        }
                                    })
                                }}
                            >
                                {"↑"}
                            </button>
                            <button
                                class="btn-reorder btn-down"
                                disabled={is_last}
                                onclick={{
                                    let on_reorder = props.on_reorder.clone();
                                    Callback::from(move |e: MouseEvent| {
                                        e.stop_propagation();
                                        if !is_last {
                                            on_reorder.emit((index, "down".to_string()));
                                        }
                                    })
                                }}
                            >
                                {"↓"}
                            </button>
                        </div>
                        
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
                        
                        // Details button
                        <button
                            class="btn-details"
                            onclick={{
                                let on_show_details = props.on_show_details.clone();
                                Callback::from(move |e: MouseEvent| {
                                    e.stop_propagation();
                                    on_show_details.emit(index);
                                })
                            }}
                        >
                            {get_text("details")}
                        </button>
                    </div>
                }
            </div>
        </div>
    }
}

