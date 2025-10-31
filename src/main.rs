// ============================================================================
// ROUTE OPTIMIZER APP - FRONTEND MVVM ESTRICTO
// ============================================================================
// Arquitectura MVVM estricta:
// - Components: SOLO vistas (sin lógica)
// - ViewModels: Estado + Lógica UI
// - Services: SOLO comunicación API
// - Stores: State Management centralizado (Yewdux)
// - Models: Estructuras compartidas con backend
// ============================================================================

mod models;
mod stores;
mod services;
mod viewmodels;
mod components;
mod hooks;
mod views;
mod utils;

use wasm_logger::Config;
use yew::Renderer;
use views::App;

fn main() {
    wasm_logger::init(Config::default());
    log::info!("🚀 Route Optimizer App - MVVM Estricto");
    
    Renderer::<App>::new().render();
}

