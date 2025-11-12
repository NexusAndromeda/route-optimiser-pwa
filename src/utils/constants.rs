/// URL base del backend
/// Configurada en tiempo de compilación:
/// - Desarrollo: http://localhost:3000 (por defecto)
/// - Producción: https://api.delivery.nexuslabs.one (via BACKEND_URL env var)
pub const BACKEND_URL: &str = match option_env!("BACKEND_URL") {
    Some(url) => url,
    None => "http://localhost:3000",
};

