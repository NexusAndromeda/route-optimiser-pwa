mod components;
mod models;

use components::App;

fn main() {
        wasm_logger::init(wasm_logger::Config::default());
    log::info!("🚀 Route Optimizer starting...");
    
    yew::Renderer::<App>::new().render();
}

