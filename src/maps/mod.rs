// Módulo de mapas con implementaciones específicas por plataforma

#[cfg(target_arch = "wasm32")]
pub mod web;

#[cfg(target_os = "android")]
pub mod android;

#[cfg(target_os = "ios")]
pub mod ios;

// Traits comunes para todas las plataformas
pub mod traits;

// Re-exportar el trait principal
// pub use traits::*; // Commented out - not used yet
