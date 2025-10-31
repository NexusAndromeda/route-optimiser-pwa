use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use crate::models::{Package, Address};

// ============================================================================
// MODELO PRINCIPAL - IDÉNTICO AL ORIGINAL
// ============================================================================

/// Estructura PRINCIPAL que contiene TODO
/// ✅ IDÉNTICA al original (app/src/models/delivery_session.rs)
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct DeliverySession {
    /// ID único de la sesión (generado al login)
    pub session_id: String,
    
    /// Timestamp del último fetch de Colis Privé
    pub last_fetch: i64,
    
    /// Timestamp de última sincronización
    pub last_sync: i64,
    
    /// Timestamp de última optimización
    pub last_optimization: Option<i64>,
    
    /// ¿La ruta está optimizada?
    pub is_optimized: bool,
    
    /// Todos los paquetes indexados por tracking
    pub packages: HashMap<String, Package>,
    
    /// Todas las direcciones indexados por address_id
    pub addresses: HashMap<String, Address>,
    
    /// Índices para búsquedas rápidas
    pub indices: Indices,
    
    /// Stats generales
    pub stats: Stats,
    
    /// Info del conductor
    pub driver: DriverInfo,
}

// ============================================================================
// ÍNDICES PARA BÚSQUEDAS RÁPIDAS
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
pub struct Indices {
    /// delivery_type -> [trackings]
    pub by_type: HashMap<String, Vec<String>>,
    
    /// status -> [trackings]
    pub by_status: HashMap<String, Vec<String>>,
    
    /// address_id -> [trackings]
    pub by_address: HashMap<String, Vec<String>>,
    
    /// problematic -> [trackings]
    pub problematic_packages: Vec<String>,
    
    /// route_order -> tracking (solo después de optimizar)
    pub by_route_order: HashMap<usize, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
pub struct Stats {
    pub total_packages: usize,
    pub total_addresses: usize,
    pub problematic_count: usize,
    pub by_type: HashMap<String, usize>,
    pub by_status: HashMap<String, usize>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct DriverInfo {
    pub driver_id: String,
    pub name: String,
    pub company_id: String,
    pub vehicle_id: Option<String>,
}

// ============================================================================
// MÉTODOS CRÍTICOS PRESERVADOS DEL ORIGINAL
// ============================================================================

impl DeliverySession {
    /// Buscar paquete por tracking (acceso directo)
    pub fn find_by_tracking(&self, tracking: &str) -> Option<&Package> {
        self.packages.get(tracking)
    }
    
    /// Buscar paquete mutable por tracking
    pub fn find_by_tracking_mut(&mut self, tracking: &str) -> Option<&mut Package> {
        self.packages.get_mut(tracking)
    }
    
    /// Actualizar status de un paquete (genérico - Optimistic UI)
    pub fn update_status(&mut self, tracking: &str, new_status: String) -> Result<(), String> {
        let package = self.packages.get_mut(tracking)
            .ok_or_else(|| format!("Package with tracking {} not found", tracking))?;
        
            let old_status = package.status.clone();
        package.status = new_status.clone();
            package.modified_by_driver = true;
        package.last_modified_at = js_sys::Date::now() as i64;
            
            // Actualizar índice by_status
        if let Some(trackings) = self.indices.by_status.get_mut(&old_status) {
            trackings.retain(|t| t != tracking);
        }
            self.indices.by_status
            .entry(new_status.clone())
                .or_default()
            .push(tracking.to_string());
            
            // Actualizar stats
            if let Some(count) = self.stats.by_status.get_mut(&old_status) {
                *count = count.saturating_sub(1);
            }
        *self.stats.by_status.entry(new_status).or_insert(0) += 1;
            
            Ok(())
        }
    
    /// Marcar como escaneado (wrapper por compatibilidad)
    pub fn mark_scanned(&mut self, tracking: &str) -> Result<(), String> {
        self.update_status(tracking, "STATUT_SCANNED".to_string())
    }
    
    /// Obtener posición en ruta optimizada por tracking
    pub fn get_route_position(&self, tracking: &str) -> Option<usize> {
        self.find_by_tracking(tracking)
            .and_then(|pkg| pkg.route_order)
    }
    
    /// Reconstruir todos los índices desde packages y addresses
    pub fn rebuild_indices(&mut self) {
        log::info!("🔨 Reconstruyendo índices...");
        
        // Resetear índices
        self.indices = Indices::default();
        
        // Reconstruir desde packages (tracking es el key directo)
        for (tracking, package) in &self.packages {
            // by_type
            let type_key = format!("{:?}", package.delivery_type);
            self.indices.by_type.entry(type_key.clone())
                .or_default()
                .push(tracking.clone());
            
            // by_status
            self.indices.by_status.entry(package.status.clone())
                .or_default()
                .push(tracking.clone());
            
            // by_address
            self.indices.by_address.entry(package.address_id.clone())
                .or_default()
                .push(tracking.clone());
            
            // problematic
            if package.is_problematic {
                self.indices.problematic_packages.push(tracking.clone());
            }
            
            // by_route_order (si está optimizado)
            if let Some(order) = package.route_order {
                self.indices.by_route_order.insert(order, tracking.clone());
            }
        }
        
        log::info!("✅ Índices reconstruidos: {} packages, {} por status", 
                   self.packages.len(),
                   self.indices.by_status.len());
    }
    
    /// Validar integridad de la sesión
    pub fn validate(&self) -> Result<(), Vec<String>> {
        let mut errors = Vec::new();
        
        // Verificar que todos los paquetes tengan address_id válido
        for (tracking, package) in &self.packages {
            if !self.addresses.contains_key(&package.address_id) {
                errors.push(format!("Package {} apunta a address_id inexistente: {}", 
                                   tracking, package.address_id));
            }
        }
        
        if errors.is_empty() {
            Ok(())
        } else {
            Err(errors)
        }
    }
}
