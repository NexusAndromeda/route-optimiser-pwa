// ============================================================================
// SESSION STORE - SIN YEWDUX (por compatibilidad con Rust 1.90)
// ============================================================================
// Usando use_state_handle directamente en lugar de Yewdux
// TODO: Migrar a Yewdux cuando esté disponible para Rust 1.90
// ============================================================================

use crate::models::session::DeliverySession;

/// Estado de sesión - Compatible con use_state_handle
#[derive(Clone, Debug, PartialEq)]
pub struct SessionStore {
    pub session: Option<DeliverySession>,
    pub loading: bool,
    pub error: Option<String>,
    pub last_fetch_time: Option<i64>,
}

impl Default for SessionStore {
    fn default() -> Self {
        Self {
            session: None,
            loading: false,
            error: None,
            last_fetch_time: None,
        }
    }
}
