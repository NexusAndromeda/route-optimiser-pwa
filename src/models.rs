use serde::{Deserialize, Serialize};

/// Modelo de datos de un paquete (compatible con Android)
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PackageData {
    pub id: String,
    #[serde(rename = "tracking_number")]
    pub tracking_number: String,
    #[serde(rename = "recipient_name")]
    pub recipient_name: String,
    pub address: String,
    pub status: String,
    pub instructions: String,
    pub phone: Option<String>,
    #[serde(rename = "delivery_date")]
    pub delivery_date: Option<String>,
    pub priority: String,
    pub latitude: Option<f64>,
    pub longitude: Option<f64>,
    #[serde(rename = "num_ordre_passage_prevu")]
    pub num_ordre_passage_prevu: Option<i32>,
}

impl PackageData {
    /// Crear un paquete de demo
    pub fn demo(id: &str, tracking_number: &str, recipient_name: &str, address: &str, 
                status: &str, latitude: f64, longitude: f64) -> Self {
        Self {
            id: id.to_string(),
            tracking_number: tracking_number.to_string(),
            recipient_name: recipient_name.to_string(),
            address: address.to_string(),
            status: status.to_string(),
            instructions: String::new(),
            phone: None,
            delivery_date: None,
            priority: "Normal".to_string(),
            latitude: Some(latitude),
            longitude: Some(longitude),
            num_ordre_passage_prevu: None,
        }
    }

    /// Verificar si el paquete tiene coordenadas válidas
    pub fn has_coordinates(&self) -> bool {
        self.latitude.is_some() && self.longitude.is_some()
    }

    /// Obtener coordenadas como tupla (lat, lng)
    pub fn coordinates(&self) -> Option<(f64, f64)> {
        match (self.latitude, self.longitude) {
            (Some(lat), Some(lng)) => Some((lat, lng)),
            _ => None,
        }
    }
}

/// Coordenadas geográficas
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Coordinates {
    pub latitude: f64,
    pub longitude: f64,
}

impl From<(f64, f64)> for Coordinates {
    fn from((lat, lng): (f64, f64)) -> Self {
        Self {
            latitude: lat,
            longitude: lng,
        }
    }
}

/// Datos de ubicación del usuario
#[derive(Debug, Clone, PartialEq)]
pub struct UserLocation {
    pub latitude: f64,
    pub longitude: f64,
    pub accuracy: Option<f64>,
}

/// Datos de demo para testing
pub mod demo {
    use super::PackageData;

    pub fn get_demo_packages() -> Vec<PackageData> {
        vec![
        PackageData::demo(
            "demo-001",
            "PU0000867901",
            "Marie Dubois",
            "15 Rue de la Paix, 75001 Paris",
            "Pendiente",
            48.8667,
            2.3333,
        ),
        PackageData::demo(
            "demo-002",
            "2E0000153827",
            "Jean Martin",
            "42 Avenue des Champs-Élysées, 75008 Paris",
            "En Ruta",
            48.8698,
            2.3076,
        ),
        PackageData::demo(
            "demo-003",
            "S79401757791",
            "Sophie Leroy",
            "8 Rue de Rivoli, 75004 Paris",
            "Pendiente",
            48.8566,
            2.3522,
        ),
        PackageData::demo(
            "demo-004",
            "PU0000867902",
            "Pierre Moreau",
            "25 Boulevard Saint-Germain, 75005 Paris",
            "Entregado",
            48.8500,
            2.3400,
        ),
        PackageData::demo(
            "demo-005",
            "2E0000153828",
            "Claire Bernard",
            "12 Place de la Bastille, 75011 Paris",
            "En Ruta",
            48.8532,
            2.3694,
        ),
        ]
    }
}
