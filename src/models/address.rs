use serde::{Deserialize, Serialize, Deserializer, de::Visitor};
use std::fmt;

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
    #[serde(deserialize_with = "deserialize_mailbox_access", alias = "has_mailbox_access")]
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

/// Deserializador personalizado para convertir has_mailbox_access (bool) a mailbox_access (Option<String>)
/// El backend devuelve `has_mailbox_access: bool` pero el frontend usa `mailbox_access: Option<String>`
fn deserialize_mailbox_access<'de, D>(deserializer: D) -> Result<Option<String>, D::Error>
where
    D: Deserializer<'de>,
{
    struct MailboxAccessVisitor;

    impl<'de> Visitor<'de> for MailboxAccessVisitor {
        type Value = Option<String>;

        fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
            formatter.write_str("bool or string or null")
        }

        fn visit_bool<E>(self, value: bool) -> Result<Self::Value, E>
        where
            E: serde::de::Error,
        {
            // Convertir bool a Option<String>: true → Some("true"), false → None
            // IMPORTANTE: Solo loguear si es true para evitar spam de logs
            let result = if value { 
                Some("true".to_string()) 
            } else { 
                None 
            };
            // Solo loguear si es true para evitar spam
            if value {
                log::info!("📬 [DESERIALIZE] Convertido has_mailbox_access: {} → mailbox_access: {:?}", value, result);
            }
            Ok(result)
        }

        fn visit_str<E>(self, value: &str) -> Result<Self::Value, E>
        where
            E: serde::de::Error,
        {
            // Mantener compatibilidad con Option<String> existente
            Ok(Some(value.to_string()))
        }

        fn visit_unit<E>(self) -> Result<Self::Value, E>
        where
            E: serde::de::Error,
        {
            // Manejar null explícitamente
            Ok(None)
        }

        fn visit_none<E>(self) -> Result<Self::Value, E>
        where
            E: serde::de::Error,
        {
            Ok(None)
        }

        fn visit_some<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
        where
            D: Deserializer<'de>,
        {
            deserializer.deserialize_any(self)
        }
    }

    // Usar deserialize_option para manejar correctamente null
    deserializer.deserialize_option(MailboxAccessVisitor)
}
