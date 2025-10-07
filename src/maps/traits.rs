use crate::models::{PackageData, Coordinates};

/// Trait común para renderizadores de mapas en todas las plataformas
pub trait MapRenderer {
    /// Agregar un marcador de paquete al mapa
    fn add_package_marker(&mut self, package: &PackageData) -> Result<(), String>;
    
    /// Remover un marcador de paquete del mapa
    fn remove_package_marker(&mut self, package_id: &str) -> Result<(), String>;
    
    /// Limpiar todos los marcadores de paquetes
    fn clear_package_markers(&mut self) -> Result<(), String>;
    
    /// Centrar el mapa en una ubicación específica
    fn set_center(&mut self, coordinates: Coordinates, zoom: f64) -> Result<(), String>;
    
    /// Centrar el mapa en la ubicación del usuario
    fn center_on_user_location(&mut self) -> Result<(), String>;
    
    /// Ajustar la vista para mostrar todos los paquetes
    fn fit_to_packages(&mut self, packages: &[PackageData]) -> Result<(), String>;
    
    /// Establecer el callback para cuando se hace click en un paquete
    fn set_package_click_callback(&mut self, callback: Box<dyn Fn(PackageData) + Send + Sync>) -> Result<(), String>;
    
    /// Verificar si el mapa está listo
    fn is_ready(&self) -> bool;
}

/// Configuración del mapa
#[derive(Debug, Clone)]
pub struct MapConfig {
    pub center: Coordinates,
    pub zoom: f64,
    pub style: MapStyle,
    pub enable_location: bool,
}

#[derive(Debug, Clone, PartialEq)]
pub enum MapStyle {
    Streets,
    Satellite,
    Dark,
    Light,
}

impl Default for MapConfig {
    fn default() -> Self {
        Self {
            center: Coordinates {
                latitude: 48.8566,  // París
                longitude: 2.3522,
            },
            zoom: 12.0,
            style: MapStyle::Streets,
            enable_location: true,
        }
    }
}

/// Error del mapa
#[derive(Debug, Clone)]
pub enum MapError {
    NotReady,
    InvalidCoordinates,
    LocationPermissionDenied,
    NetworkError(String),
    Unknown(String),
}

impl std::fmt::Display for MapError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            MapError::NotReady => write!(f, "Map is not ready"),
            MapError::InvalidCoordinates => write!(f, "Invalid coordinates"),
            MapError::LocationPermissionDenied => write!(f, "Location permission denied"),
            MapError::NetworkError(msg) => write!(f, "Network error: {}", msg),
            MapError::Unknown(msg) => write!(f, "Unknown error: {}", msg),
        }
    }
}

impl std::error::Error for MapError {}
