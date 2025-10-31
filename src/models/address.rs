use serde::{Deserialize, Serialize};

/// Dirección de entrega
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
pub struct Address {
    pub address_id: String,
    #[serde(default, alias = "official_label")]
    pub label: String,
    #[serde(default)]
    pub latitude: f64,
    #[serde(default)]
    pub longitude: f64,
    pub mailbox_access: Option<String>,
    pub door_code: Option<String>,
    pub driver_notes: Option<String>,
    #[serde(default)]
    pub package_ids: Vec<String>,
    pub visit_order: Option<usize>,
    #[serde(default)]
    pub corrected_by_driver: bool,
    pub original_label: Option<String>,
}
