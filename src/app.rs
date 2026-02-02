// ============================================================================
// APP - Aplicación principal (reemplaza main.rs con Yew)
// ============================================================================

use wasm_bindgen::prelude::*;
use web_sys::{Element, console};
use wasm_bindgen::JsCast;
use crate::dom::{get_element_by_id, set_inner_html, append_child};
use crate::dom::incremental::*;
use crate::state::app_state::{AppState, UpdateType, IncrementalUpdate};
use crate::views::render_app;

/// Aplicación principal
pub struct App {
    state: AppState,
    root: Option<Element>,
}

impl App {
    /// Crear nueva aplicación
    pub fn new() -> Result<Self, JsValue> {
        let root = get_element_by_id("app")
            .ok_or_else(|| JsValue::from_str("No #app element found"))?;
        
        let state = AppState::new();
        
        // Cargar sesión desde storage si existe
        {
            use crate::services::OfflineService;
            let offline_service = OfflineService::new();
            if let Ok(Some(saved_session)) = offline_service.load_session() {
                log::info!("💾 [APP] Sesión encontrada en storage, restaurando...");
                
                // Log de direcciones con mailbox_access al restaurar
                for (addr_id, addr) in &saved_session.addresses {
                    if addr.mailbox_access.is_some() {
                        log::info!("📬 [APP] Dirección {} tiene mailbox_access={:?} al restaurar desde storage", addr_id, addr.mailbox_access);
                    }
                }
                
                state.session.set_session(Some(saved_session.clone()));
                // Establecer auth como logged in si hay sesión
                state.auth.set_logged_in(true);
                state.auth.set_username(Some(saved_session.driver.driver_id.clone()));
                state.auth.set_company_id(Some(saved_session.driver.company_id.clone()));
                log::info!("✅ [APP] Sesión restaurada desde storage");
            }
        }
        
        // Cargar credenciales del admin desde storage si existen
        {
            use crate::services::OfflineService;
            use crate::services::api_client::ApiClient;
            let offline_service = OfflineService::new();
            
            if let Ok(Some((username, password, societe))) = offline_service.load_admin_credentials() {
                log::info!("💾 [APP] Credenciales admin encontradas, restaurando sesión...");
                
                // Guardar credenciales en el estado
                *state.admin_username.borrow_mut() = Some(username.clone());
                *state.admin_password.borrow_mut() = Some(password.clone());
                *state.admin_societe.borrow_mut() = Some(societe.clone());
                
                // Hacer fetch del dashboard para restaurar el estado
                let state_clone = state.clone();
                wasm_bindgen_futures::spawn_local(async move {
                    let api = ApiClient::new();
                    
                    // Formatear fecha de hoy para date_debut
                    let today = js_sys::Date::new_0();
                    let date_debut = format!(
                        "{:04}-{:02}-{:02}T00:00:00.000Z",
                        today.get_full_year(),
                        today.get_month() + 1,
                        today.get_date()
                    );
                    
                    match api.fetch_admin_dashboard(&username, &password, &societe, &date_debut).await {
                        Ok(response) => {
                            log::info!("✅ [APP] Dashboard admin restaurado con {} districts", response.districts.len());
                            
                            // Restaurar estado admin
                            *state_clone.admin_mode.borrow_mut() = true;
                            *state_clone.admin_districts.borrow_mut() = response.districts.clone();
                            *state_clone.admin_total_packages.borrow_mut() = response.total_packages;
                            *state_clone.admin_sso_token.borrow_mut() = Some(response.sso_token.clone());
                            state_clone.auth.set_logged_in(true);
                            state_clone.auth.set_username(Some(username));
                            state_clone.auth.set_company_id(Some(societe));
                            // Cargar demandes pendientes
                            let state_req = state_clone.clone();
                            if let Ok(requests) = api.fetch_status_requests("app_restore").await {
                                *state_req.admin_status_requests.borrow_mut() = requests;
                            }
                            // Re-renderizar para mostrar el dashboard
                            crate::rerender_app();
                        }
                        Err(e) => {
                            log::error!("❌ [APP] Error restaurando dashboard admin: {}", e);
                            // Limpiar credenciales si falla
                            if let Err(clear_err) = offline_service.clear_admin_credentials() {
                                log::error!("❌ Error limpiando credenciales: {}", clear_err);
                            }
                        }
                    }
                });
            }
        }
        
        // Suscribirse a cambios de estado para re-renderizar automáticamente
        state.subscribe_to_changes(move || {
            // Usar gloo_timers para batchear múltiples updates
            use gloo_timers::callback::Timeout;
            Timeout::new(0, move || {
                crate::rerender_app();
            }).forget();
        });
        
        Ok(Self {
            state,
            root: Some(root),
        })
    }
    
    /// Renderizar aplicación
    pub fn render(&mut self) -> Result<(), JsValue> {
        console::log_1(&JsValue::from_str("🎬 [APP] App::render() llamado"));
        
        if let Some(root) = &self.root {
            // PRESERVAR scroll ANTES de limpiar el contenido para evitar el "salto" visual
            // Guardar scroll position en una variable JavaScript global antes de destruir el DOM
            let preserve_scroll_js = r#"
                (function() {
                    const container = document.querySelector('.package-list');
                    if (container) {
                        window._preservedScrollPosition = container.scrollTop;
                        console.log('💾 [JS] Scroll preservado ANTES de limpiar DOM:', window._preservedScrollPosition);
                    } else {
                        window._preservedScrollPosition = null;
                    }
                })();
            "#;
            let _ = js_sys::eval(preserve_scroll_js);
            
            // Limpiar contenido anterior
            set_inner_html(root, "");
            
            // Renderizar aplicación completa
            console::log_1(&JsValue::from_str("🔄 [APP] Llamando a render_app()..."));
            let app_view = render_app(&self.state)?;
            console::log_1(&JsValue::from_str("✅ [APP] render_app() completado, agregando al DOM"));
            append_child(root, &app_view)?;
            
            // Después del render, asegurar que la variable CSS del bottom sheet esté sincronizada
            use crate::dom::incremental::update_bottom_sheet_incremental;
            if let Err(e) = update_bottom_sheet_incremental(&self.state) {
                log::warn!("⚠️ Error sincronizando bottom sheet después del render: {:?}", e);
            }
            
            // Restaurar scroll position INMEDIATAMENTE después de agregar al DOM
            // Usar requestAnimationFrame con delay mínimo para restaurar ANTES del próximo frame
            // Esto evita el "salto" visual porque restauramos antes de que el navegador renderice
            let restore_scroll_immediate_js = r#"
                (function() {
                    if (window._preservedScrollPosition !== null && window._preservedScrollPosition !== undefined) {
                        const preservedPos = window._preservedScrollPosition;
                        console.log('🔄 [JS] Restaurando scroll preservado INMEDIATAMENTE:', preservedPos);
                        
                        // Usar requestAnimationFrame para restaurar en el próximo frame (antes del render visual)
                        requestAnimationFrame(function() {
                            const container = document.querySelector('.package-list');
                            if (container) {
                                container.scrollTop = preservedPos;
                                console.log('✅ [JS] Scroll restaurado INMEDIATAMENTE a:', preservedPos, 'Actual:', container.scrollTop);
                            }
                            delete window._preservedScrollPosition;
                        });
                    }
                })();
            "#;
            let _ = js_sys::eval(restore_scroll_immediate_js);
            
            // También restaurar usando el sistema de estado (para cuando se cierra el modal)
            // Usar delay más largo para asegurar que el DOM esté completamente renderizado
            // NO limpiar la posición guardada aquí, mantenerla para cuando se cierre el modal
            use gloo_timers::callback::Timeout;
            let state_clone = self.state.clone();
            web_sys::console::log_1(&wasm_bindgen::JsValue::from_str("🔄 [SCROLL] Re-render completo completado, programando restauración de scroll (manteniendo posición guardada)"));
            Timeout::new(200, move || {
                web_sys::console::log_1(&wasm_bindgen::JsValue::from_str("⏰ [SCROLL] Timeout después de re-render completado, restaurando scroll ahora (sin limpiar)"));
                state_clone.restore_package_list_scroll_position(false); // false = no limpiar después de restaurar
            }).forget();
        }
        Ok(())
    }
    
    /// Obtener referencia al estado
    pub fn state(&self) -> &AppState {
        &self.state
    }
    
    /// Obtener referencia mutable al estado
    pub fn state_mut(&mut self) -> &mut AppState {
        &mut self.state
    }
    
    /// Actualizar UI cuando cambia el estado (re-render completo)
    pub fn update(&mut self) -> Result<(), JsValue> {
        self.render()
    }
    
    /// Actualización incremental del DOM (solo elementos específicos)
    pub fn update_incremental(&self, update_type: IncrementalUpdate) -> Result<(), JsValue> {
        match update_type {
            IncrementalUpdate::BottomSheet => {
                update_bottom_sheet_incremental(&self.state)?;
            }
            IncrementalUpdate::PackageSelection => {
                update_package_selection(&self.state)?;
            }
            IncrementalUpdate::Modal(modal_type) => {
                match modal_type {
                    ModalType::Details => {
                        // Usar manipulación directa del DOM para el modal de detalles
                        // Esto evita re-render completo y preserva el scroll
                        use crate::dom::incremental::update_details_modal_direct;
                        update_details_modal_direct(&self.state)?;
                    }
                    _ => {
                        // Otros modales usan el método tradicional
                        let show = match modal_type {
                            ModalType::Settings => *self.state.show_settings.borrow(),
                            ModalType::Scanner => *self.state.show_scanner.borrow(),
                            ModalType::Tracking => *self.state.show_tracking_modal.borrow(),
                            ModalType::Details => unreachable!(), // Ya manejado arriba
                        };
                        // Si el modal no existe y queremos mostrarlo, retornar error para hacer re-render completo
                        if let Err(_) = update_modal_visibility(modal_type, show) {
                            if show {
                                // Modal no existe pero queremos mostrarlo - necesita re-render completo
                                log::warn!("⚠️ Modal no existe, necesita re-render completo");
                                return Err(JsValue::from_str("Modal not found, needs full render"));
                            }
                            // Si show es false y el modal no existe, no hay nada que hacer (OK)
                        }
                    }
                }
            }
            IncrementalUpdate::Header => {
                let has_session = self.state.session.get_session().is_some();
                update_header(&self.state, has_session)?;
            }
            IncrementalUpdate::PackageList => {
                if let Some(session) = self.state.session.get_session() {
                    // Calcular grupos
                    use crate::views::group_packages_by_address;
                    use crate::models::package::Package;
                    let mut packages: Vec<Package> = session.packages.values().cloned().collect();
                    if *self.state.filter_mode.borrow() {
                        packages.retain(|p| p.status.starts_with("STATUT_CHARGER"));
                    }
                    let groups = group_packages_by_address(packages);
                    update_package_list(&self.state, &groups, &session)?;
                }
            }
            IncrementalUpdate::SyncIndicator => {
                update_sync_indicator(&self.state)?;
            }
            IncrementalUpdate::MapPackages => {
                if let Some(session) = self.state.session.get_session() {
                    update_map_packages(&self.state, &session)?;
                }
            }
            IncrementalUpdate::AdminBottomSheetContent => {
                use crate::views::admin_dashboard::update_admin_bottom_sheet_content;
                update_admin_bottom_sheet_content(&self.state)?;
            }
        }
        Ok(())
    }
}

