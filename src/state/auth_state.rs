// ============================================================================
// AUTH STATE - Estado de autenticación (reemplaza AuthStore)
// ============================================================================

use std::cell::RefCell;
use std::rc::Rc;

/// Estado de autenticación
#[derive(Clone)]
pub struct AuthState {
    pub is_logged_in: Rc<RefCell<bool>>,
    pub username: Rc<RefCell<Option<String>>>,
    pub token: Rc<RefCell<Option<String>>>,
    pub company_id: Rc<RefCell<Option<String>>>,
}

impl AuthState {
    /// Crear nuevo estado de autenticación
    pub fn new() -> Self {
        Self {
            is_logged_in: Rc::new(RefCell::new(false)),
            username: Rc::new(RefCell::new(None)),
            token: Rc::new(RefCell::new(None)),
            company_id: Rc::new(RefCell::new(None)),
        }
    }
    
    /// Establecer logged in
    pub fn set_logged_in(&self, logged_in: bool) {
        *self.is_logged_in.borrow_mut() = logged_in;
    }
    
    /// Obtener logged in
    pub fn get_logged_in(&self) -> bool {
        *self.is_logged_in.borrow()
    }
    
    /// Establecer username
    pub fn set_username(&self, username: Option<String>) {
        *self.username.borrow_mut() = username;
    }
    
    /// Obtener username
    pub fn get_username(&self) -> Option<String> {
        self.username.borrow().clone()
    }
    
    /// Establecer token
    pub fn set_token(&self, token: Option<String>) {
        *self.token.borrow_mut() = token;
    }
    
    /// Obtener token
    pub fn get_token(&self) -> Option<String> {
        self.token.borrow().clone()
    }
    
    /// Establecer company_id
    pub fn set_company_id(&self, company_id: Option<String>) {
        *self.company_id.borrow_mut() = company_id;
    }
    
    /// Obtener company_id
    pub fn get_company_id(&self) -> Option<String> {
        self.company_id.borrow().clone()
    }
    
    /// Logout - limpiar todo
    pub fn logout(&self) {
        self.set_logged_in(false);
        self.set_username(None);
        self.set_token(None);
        self.set_company_id(None);
    }
}

impl Default for AuthState {
    fn default() -> Self {
        Self::new()
    }
}

