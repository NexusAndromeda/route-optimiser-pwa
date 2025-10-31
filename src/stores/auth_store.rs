// ============================================================================
// AUTH STORE - SIN YEWDUX (por compatibilidad con Rust 1.90)
// ============================================================================

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AuthStore {
    pub is_logged_in: bool,
    pub username: Option<String>,
    pub token: Option<String>,
    pub company_id: Option<String>,
}

impl Default for AuthStore {
    fn default() -> Self {
        Self {
            is_logged_in: false,
            username: None,
            token: None,
            company_id: None,
        }
    }
}
