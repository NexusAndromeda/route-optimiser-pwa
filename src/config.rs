use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppConfig {
    pub backend_url_development: String,
    pub backend_url_production: String,
    pub environment: String,
    pub enable_logging: bool,
    pub network_timeout_seconds: u32,
    pub retry_attempts: u32,
    pub map_config: MapConfig,
    pub package_config: PackageConfig,
    pub ui_config: UIConfig,
    pub mapbox_access_token: String,
    pub api_key: Option<String>,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            backend_url_development: "http://192.168.1.9:3000".to_string(),
            backend_url_production: "https://api.delivery.nexuslabs.one".to_string(),
            environment: "development".to_string(),
            enable_logging: true,
            network_timeout_seconds: 30,
            retry_attempts: 3,
            map_config: MapConfig::default(),
            package_config: PackageConfig::default(),
            ui_config: UIConfig::default(),
            mapbox_access_token: String::new(),
            api_key: None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MapConfig {
    pub default_center_lat: f64,
    pub default_center_lng: f64,
    pub default_zoom: f64,
}

impl Default for MapConfig {
    fn default() -> Self {
        Self {
            default_center_lat: 48.8566,
            default_center_lng: 2.3522,
            default_zoom: 12.0,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PackageConfig {
    pub max_packages_for_clustering: u32,
    pub cluster_threshold: u32,
}

impl Default for PackageConfig {
    fn default() -> Self {
        Self {
            max_packages_for_clustering: 50,
            cluster_threshold: 20,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UIConfig {
    pub marker_size: u32,
    pub cluster_size: u32,
    pub route_line_width: u32,
}

impl Default for UIConfig {
    fn default() -> Self {
        Self {
            marker_size: 30,
            cluster_size: 40,
            route_line_width: 4,
        }
    }
}

impl AppConfig {
    /// Carga la configuración desde variables de entorno en tiempo de compilación
    pub fn from_env() -> Self {
        Self {
            backend_url_development: option_env!("BACKEND_URL_DEVELOPMENT")
                .unwrap_or("http://192.168.1.9:3000").to_string(),
            backend_url_production: option_env!("BACKEND_URL_PRODUCTION")
                .unwrap_or("https://api.delivery.nexuslabs.one").to_string(),
            environment: option_env!("ENVIRONMENT")
                .unwrap_or("development").to_string(),
            enable_logging: option_env!("ENABLE_LOGGING")
                .unwrap_or("true").parse().unwrap_or(true),
            network_timeout_seconds: option_env!("NETWORK_TIMEOUT_SECONDS")
                .unwrap_or("30").parse().unwrap_or(30),
            retry_attempts: option_env!("RETRY_ATTEMPTS")
                .unwrap_or("3").parse().unwrap_or(3),
            map_config: MapConfig {
                default_center_lat: option_env!("DEFAULT_MAP_CENTER_LAT")
                    .unwrap_or("48.8566").parse().unwrap_or(48.8566),
                default_center_lng: option_env!("DEFAULT_MAP_CENTER_LNG")
                    .unwrap_or("2.3522").parse().unwrap_or(2.3522),
                default_zoom: option_env!("DEFAULT_MAP_ZOOM")
                    .unwrap_or("12.0").parse().unwrap_or(12.0),
            },
            package_config: PackageConfig {
                max_packages_for_clustering: option_env!("MAX_PACKAGES_FOR_CLUSTERING")
                    .unwrap_or("50").parse().unwrap_or(50),
                cluster_threshold: option_env!("CLUSTER_THRESHOLD")
                    .unwrap_or("20").parse().unwrap_or(20),
            },
            ui_config: UIConfig {
                marker_size: option_env!("MARKER_SIZE")
                    .unwrap_or("30").parse().unwrap_or(30),
                cluster_size: option_env!("CLUSTER_SIZE")
                    .unwrap_or("40").parse().unwrap_or(40),
                route_line_width: option_env!("ROUTE_LINE_WIDTH")
                    .unwrap_or("4").parse().unwrap_or(4),
            },
            mapbox_access_token: option_env!("MAPBOX_ACCESS_TOKEN")
                .unwrap_or("").to_string(),
            api_key: option_env!("API_KEY").map(|s| s.to_string()),
        }
    }

    /// Obtiene la URL del backend según el entorno actual
    pub fn backend_url(&self) -> &str {
        match self.environment.as_str() {
            "production" => &self.backend_url_production,
            _ => &self.backend_url_development,
        }
    }

    /// Verifica si el modo de logging está habilitado
    pub fn is_logging_enabled(&self) -> bool {
        self.enable_logging
    }

    /// Obtiene el token de Mapbox
    pub fn mapbox_token(&self) -> &str {
        &self.mapbox_access_token
    }
}

// Configuración global estática
lazy_static::lazy_static! {
    pub static ref CONFIG: AppConfig = AppConfig::from_env();
}

