// ============================================================================
// REACTIVITY - Sistema de notificaciones/subscribers para reactividad
// ============================================================================

use std::cell::RefCell;
use std::rc::Rc;

type Callback = Box<dyn Fn()>;

/// Estado reactivo con sistema de notificaciones
pub struct ReactiveState<T> {
    value: Rc<RefCell<T>>,
    subscribers: RefCell<Vec<Callback>>,
}

impl<T> ReactiveState<T> {
    /// Crear nuevo estado reactivo
    pub fn new(value: T) -> Self {
        Self {
            value: Rc::new(RefCell::new(value)),
            subscribers: RefCell::new(Vec::new()),
        }
    }
    
    /// Obtener referencia al valor interno
    pub fn get(&self) -> Rc<RefCell<T>> {
        self.value.clone()
    }
    
    /// Establecer nuevo valor y notificar subscribers
    pub fn set(&self, new_value: T) {
        *self.value.borrow_mut() = new_value;
        self.notify();
    }
    
    /// Actualizar valor usando closure y notificar
    pub fn update<F>(&self, updater: F)
    where
        F: FnOnce(&mut T),
    {
        updater(&mut *self.value.borrow_mut());
        self.notify();
    }
    
    /// Suscribirse a cambios
    pub fn subscribe<F>(&self, callback: F)
    where
        F: Fn() + 'static,
    {
        self.subscribers.borrow_mut().push(Box::new(callback));
    }
    
    /// Notificar a todos los subscribers
    fn notify(&self) {
        for callback in self.subscribers.borrow().iter() {
            callback();
        }
    }
}

impl<T> Clone for ReactiveState<T> {
    fn clone(&self) -> Self {
        Self {
            value: self.value.clone(),
            subscribers: RefCell::new(Vec::new()), // Nuevos subscribers
        }
    }
}

