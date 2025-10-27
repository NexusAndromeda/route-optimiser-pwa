use yew::prelude::*;
use crate::models::LegacyPackage as Package;
use crate::context::get_text;
use super::PackageCard;
use std::collections::HashMap;

#[derive(Properties, PartialEq)]
pub struct PackageListProps {
    pub packages: Vec<Package>,
    pub selected_index: Option<usize>,
    pub delivered: usize,
    pub total: usize,
    pub percentage: usize,
    pub sheet_state: &'static str,
    pub on_toggle: Callback<MouseEvent>,
    pub on_select: Callback<usize>,
    pub on_show_details: Callback<usize>,
    pub on_navigate: Callback<usize>,
    pub on_reorder: Callback<(usize, String)>,
    #[prop_or_default]
    pub animations: HashMap<usize, String>,
    #[prop_or_default]
    pub loading: bool,
    #[prop_or_default]
    pub expanded_groups: Vec<String>,
    #[prop_or_default]
    pub on_toggle_group: Option<Callback<String>>,
    #[prop_or_default]
    pub on_show_package_details: Option<Callback<Package>>,
    #[prop_or_default]
    pub reorder_mode: bool,
    #[prop_or_default]
    pub reorder_origin: Option<usize>,
    #[prop_or_default]
    pub on_optimize: Option<Callback<MouseEvent>>,
    #[prop_or_default]
    pub optimizing: bool,
}

#[function_component(PackageList)]
pub fn package_list(props: &PackageListProps) -> Html {
    let show_progress_bar = props.sheet_state != "collapsed";
    
    html! {
        <>
            // Drag Handle Container
            <div class="drag-handle-container" onclick={props.on_toggle.clone()}>
                <div class="drag-handle"></div>
                
                // Progress Info
                <div class="progress-info">
                    <div class="progress-text">
                        <span class="progress-count">
                            {format!("✓ {}/{} {}", props.delivered, props.total, get_text("delivered"))}
                        </span>
                    </div>
                    <div class="progress-percentage">
                        <span>{format!("{}%", props.percentage)}</span>
                    </div>
                </div>
                
                // Progress Bar
                if show_progress_bar {
                    <div class="progress-bar-container">
                        <div class="progress-bar" style={format!("width: {}%", props.percentage)}></div>
                    </div>
                }
            </div>
            
            // Package List
            <div class="package-list">
                {
                    if props.loading {
                        html! {
                            <div class="no-packages">
                                <div class="no-packages-icon">{"⏳"}</div>
                                <div class="no-packages-text">{format!("{}...", get_text("loading"))}</div>
                                <div class="no-packages-subtitle">{get_text("please_wait")}</div>
                            </div>
                        }
                    } else if props.packages.is_empty() {
                        html! {
                            <div class="no-packages">
                                <div class="no-packages-icon">{"📦"}</div>
                                <div class="no-packages-text">{get_text("no_packages")}</div>
                                <div class="no-packages-subtitle">{get_text("packages_after_login")}</div>
                            </div>
                        }
                    } else {
                        html! {
                            <>
                                { for props.packages.iter().enumerate().map(|(index, package)| {
                                    let is_selected = props.selected_index == Some(index);
                                    let animation_class = props.animations.get(&index).cloned();
                                    let is_expanded = props.expanded_groups.contains(&package.id);
                                    let is_reorder_origin = props.reorder_origin == Some(index);
                                    
                                    html! {
                                        <PackageCard
                                            key={package.id.clone()}
                                            index={index}
                                            package={package.clone()}
                                            is_selected={is_selected}
                                            is_expanded={is_expanded}
                                            on_select={props.on_select.clone()}
                                            on_show_details={props.on_show_details.clone()}
                                            on_navigate={props.on_navigate.clone()}
                                            on_reorder={props.on_reorder.clone()}
                                            on_toggle_group={props.on_toggle_group.clone()}
                                            on_show_package_details={props.on_show_package_details.clone()}
                                            total_packages={props.packages.len()}
                                            animation_class={animation_class}
                                            reorder_mode={props.reorder_mode}
                                            is_reorder_origin={is_reorder_origin}
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

