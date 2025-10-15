mod models;
mod utils;
mod services;
mod hooks;
mod views;
mod context;

use views::App;

fn main() {
    wasm_logger::init(wasm_logger::Config::default());
    log::info!("🚀 Route Optimizer starting...");
    
    yew::Renderer::<App>::new().render();
}
