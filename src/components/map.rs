use yew::prelude::*;
use wasm_bindgen::prelude::*;
use web_sys::window;
use gloo_timers::callback::Timeout;

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = console)]
    fn log(s: &str);
    
    #[wasm_bindgen(js_name = initMapbox)]
    fn init_mapbox(container_id: &str, is_dark: bool);
    
    #[wasm_bindgen(js_name = addMapMarker)]
    fn add_map_marker(index: i32, lat: f64, lng: f64, is_delivered: bool);
}

#[function_component(MapContainer)]
pub fn map_container() -> Html {
    use_effect(|| {
        // Detect dark mode
        let is_dark = window()
            .and_then(|w| w.match_media("(prefers-color-scheme: dark)").ok())
            .flatten()
            .map(|mq| mq.matches())
            .unwrap_or(false);
        
        // Initialize map after a short delay to ensure DOM is ready
        Timeout::new(100, move || {
            log("🗺️ Initializing Mapbox from Rust/WASM");
            log("🔑 Token loaded from APP_CONFIG");
            
            init_mapbox("map", is_dark);
            
            // Add demo markers
            let markers = vec![
                (2.3316, 48.8698, false), // Rue de la Paix - pending
                (2.3069, 48.8698, true),  // Champs-Élysées - delivered
                (2.3522, 48.8566, false), // Rivoli - pending
                (2.3488, 48.8534, true),  // Saint-Germain - delivered
                (2.3691, 48.8530, false), // Bastille - pending
            ];
            
            for (i, (lng, lat, delivered)) in markers.iter().enumerate() {
                add_map_marker(i as i32 + 1, *lat, *lng, *delivered);
            }
        }).forget();
        
        || ()
    });
    
    html! {
        <div id="map" class="map-container"></div>
    }
}

