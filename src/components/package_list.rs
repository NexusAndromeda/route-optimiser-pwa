use yew::prelude::*;
use crate::models::Package;
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
                            {format!("✓ {}/{} livrés", props.delivered, props.total)}
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
                    props.packages.iter().enumerate().map(|(index, package)| {
                        let is_selected = props.selected_index == Some(index);
                        let animation_class = props.animations.get(&index).cloned();
                        
                        html! {
                            <PackageCard
                                key={package.id.clone()}
                                index={index}
                                package={package.clone()}
                                is_selected={is_selected}
                                on_select={props.on_select.clone()}
                                on_show_details={props.on_show_details.clone()}
                                on_navigate={props.on_navigate.clone()}
                                on_reorder={props.on_reorder.clone()}
                                total_packages={props.packages.len()}
                                animation_class={animation_class}
                            />
                        }
                    }).collect::<Html>()
                }
            </div>
        </>
    }
}

