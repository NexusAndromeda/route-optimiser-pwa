// ============================================================================
// SESSION STATE - Estado de sesión (reemplaza SessionStore)
// ============================================================================

use std::cell::RefCell;
use std::rc::Rc;
use crate::models::session::DeliverySession;

/// Estado de sesión
#[derive(Clone)]
pub struct SessionState {
    pub session: Rc<RefCell<Option<DeliverySession>>>,
    pub loading: Rc<RefCell<bool>>,
    pub error: Rc<RefCell<Option<String>>>,
    pub last_fetch_time: Rc<RefCell<Option<i64>>>,
}

impl SessionState {
    /// Crear nuevo estado de sesión
    pub fn new() -> Self {
        Self {
            session: Rc::new(RefCell::new(None)),
            loading: Rc::new(RefCell::new(false)),
            error: Rc::new(RefCell::new(None)),
            last_fetch_time: Rc::new(RefCell::new(None)),
        }
    }
    
    /// Establecer sesión
    pub fn set_session(&self, session: Option<DeliverySession>) {
        // Guardar en storage cuando se actualiza
        if let Some(ref sess) = session {
            use crate::services::OfflineService;
            let offline_service = OfflineService::new();
            if let Err(e) = offline_service.save_session(sess) {
                log::error!("❌ Error guardando sesión en storage: {}", e);
            }
        }
        *self.session.borrow_mut() = session;
    }
    
    /// Obtener sesión
    pub fn get_session(&self) -> Option<DeliverySession> {
        self.session.borrow().clone()
    }
    
    /// Establecer loading
    pub fn set_loading(&self, loading: bool) {
        *self.loading.borrow_mut() = loading;
    }
    
    /// Obtener loading
    pub fn get_loading(&self) -> bool {
        *self.loading.borrow()
    }
    
    /// Establecer error
    pub fn set_error(&self, error: Option<String>) {
        *self.error.borrow_mut() = error;
    }
    
    /// Obtener error
    pub fn get_error(&self) -> Option<String> {
        self.error.borrow().clone()
    }
    
    /// Establecer último fetch time
    pub fn set_last_fetch_time(&self, time: Option<i64>) {
        *self.last_fetch_time.borrow_mut() = time;
    }
    
    /// Obtener último fetch time
    pub fn get_last_fetch_time(&self) -> Option<i64> {
        *self.last_fetch_time.borrow()
    }
}

impl Default for SessionState {
    fn default() -> Self {
        Self::new()
    }
}

