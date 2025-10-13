use yew::prelude::*;
use wasm_bindgen::prelude::*;
use web_sys::window;
use gloo_timers::callback::Timeout;
use crate::models::Package;

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = console)]
    fn log(s: &str);
    
    #[wasm_bindgen(js_name = initMapbox)]
    fn init_mapbox(container_id: &str, is_dark: bool);
    
    #[wasm_bindgen(js_name = addPackagesToMap)]
    fn add_packages_to_map(packages_json: &str);
    
    #[wasm_bindgen(js_name = updateSelectedPackage)]
    fn update_selected_package(selected_index: i32);
}

#[derive(Properties, PartialEq)]
pub struct MapContainerProps {
    pub packages: Vec<Package>,
    pub selected_index: Option<usize>,
}

#[function_component(MapContainer)]
pub fn map_container(props: &MapContainerProps) -> Html {
    // Initialize map on mount
    {
        let packages = props.packages.clone();
        
        use_effect_with((), move |_| {
            // Detect dark mode
            let is_dark = window()
                .and_then(|w| w.match_media("(prefers-color-scheme: dark)").ok())
                .flatten()
                .map(|mq| mq.matches())
                .unwrap_or(false);
            
            // Initialize map after a short delay to ensure DOM is ready
            Timeout::new(100, move || {
                log("🗺️ Initializing Mapbox from Rust/WASM");
                
                init_mapbox("map", is_dark);
                
                // Add packages to map after initialization
                let packages_json = serde_json::to_string(&packages).unwrap_or_default();
                add_packages_to_map(&packages_json);
            }).forget();
            
            || ()
        });
    }
    
    // Update selected package when selection changes
    {
        let selected_index = props.selected_index;
        
        use_effect_with(selected_index, move |sel_idx| {
            if let Some(idx) = sel_idx {
                update_selected_package(*idx as i32);
            }
            || ()
        });
    }
    
    html! {
        <></>
    }
}

