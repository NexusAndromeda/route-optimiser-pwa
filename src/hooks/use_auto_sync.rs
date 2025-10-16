use yew::prelude::*;
use gloo_timers::callback::Interval;
use crate::models::{Package, LoginData};
use crate::services::{fetch_packages, CacheService};

const SYNC_INTERVAL_MS: u32 = 5 * 60 * 1000; // 5 minutos
const SYNC_INTERVAL_ACTIVE_MS: u32 = 2 * 60 * 1000; // 2 minutos cuando hay cambios

pub struct UseAutoSyncHandle {
    pub is_syncing: UseStateHandle<bool>,
    pub last_sync: UseStateHandle<Option<chrono::DateTime<chrono::Utc>>>,
    pub sync_error: UseStateHandle<Option<String>>,
    pub force_sync: Callback<()>,
}

#[hook]
pub fn use_auto_sync(
    login_data: Option<LoginData>,
    packages: UseStateHandle<Vec<Package>>,
    active: bool, // Si hay actividad reciente del usuario
) -> UseAutoSyncHandle {
    let is_syncing = use_state(|| false);
    let last_sync = use_state(|| None::<chrono::DateTime<chrono::Utc>>);
    let sync_error = use_state(|| None::<String>);
    let interval_handle = use_mut_ref(|| None::<Interval>);

    // Función de sincronización
    let sync_fn = {
        let packages = packages.clone();
        let is_syncing = is_syncing.clone();
        let last_sync = last_sync.clone();
        let sync_error = sync_error.clone();
        let login_data = login_data.clone();

        Callback::from(move |_| {
            if let Some(login) = &login_data {
                let packages = packages.clone();
                let is_syncing = is_syncing.clone();
                let last_sync = last_sync.clone();
                let sync_error = sync_error.clone();
                let username = login.username.clone();
                let company_code = login.company.code.clone();

                // No sincronizar si ya está en proceso
                if *is_syncing {
                    log::info!("🔄 Sincronización ya en progreso, saltando...");
                    return;
                }

                wasm_bindgen_futures::spawn_local(async move {
                    is_syncing.set(true);
                    sync_error.set(None);
                    log::info!("🔄 Iniciando sincronización automática...");

                    match fetch_packages(&username, &company_code, false).await {
                        Ok(fetched_packages) => {
                            let current_packages = (*packages).clone();
                            
                            // Verificar si hay cambios
                            let has_changes = packages_differ(&current_packages, &fetched_packages);
                            
                            if has_changes {
                                log::info!("✅ Sincronización completada: {} paquetes (CAMBIOS DETECTADOS)", fetched_packages.len());
                                
                                // Actualizar caché
                                if let Err(e) = CacheService::update_packages(fetched_packages.clone()) {
                                    log::error!("❌ Error actualizando caché: {}", e);
                                }
                                
                                packages.set(fetched_packages);
                            } else {
                                log::info!("✅ Sincronización completada: {} paquetes (sin cambios)", fetched_packages.len());
                            }
                            
                            last_sync.set(Some(chrono::Utc::now()));
                            is_syncing.set(false);
                        }
                        Err(e) => {
                            log::error!("❌ Error en sincronización automática: {}", e);
                            sync_error.set(Some(format!("Error de sincronización: {}", e)));
                            is_syncing.set(false);
                        }
                    }
                });
            }
        })
    };

    // Configurar intervalo automático
    {
        let sync_fn = sync_fn.clone();
        let interval_handle = interval_handle.clone();
        let login_data = login_data.clone();
        
        use_effect_with((login_data, active), move |(login_opt, is_active)| {
            // Limpiar intervalo anterior
            *interval_handle.borrow_mut() = None;

            if login_opt.is_some() {
                let interval_ms = if *is_active {
                    SYNC_INTERVAL_ACTIVE_MS
                } else {
                    SYNC_INTERVAL_MS
                };

                log::info!("⏰ Configurando sincronización automática cada {} segundos", interval_ms / 1000);

                let sync_fn = sync_fn.clone();
                let interval = Interval::new(interval_ms, move || {
                    sync_fn.emit(());
                });

                *interval_handle.borrow_mut() = Some(interval);
            }

            move || {
                // Cleanup
                *interval_handle.borrow_mut() = None;
            }
        });
    }

    // Force sync callback
    let force_sync = sync_fn.clone();

    UseAutoSyncHandle {
        is_syncing,
        last_sync,
        sync_error,
        force_sync,
    }
}

/// Compara dos listas de paquetes para detectar cambios
fn packages_differ(current: &[Package], new: &[Package]) -> bool {
    // Verificar si la cantidad cambió
    if current.len() != new.len() {
        return true;
    }

    // Verificar si algún paquete cambió su estado
    for (curr, new_pkg) in current.iter().zip(new.iter()) {
        if curr.id != new_pkg.id {
            return true; // Orden cambió
        }
        
        if curr.code_statut_article != new_pkg.code_statut_article {
            return true; // Estado cambió
        }
        
        if curr.coords != new_pkg.coords {
            return true; // Coordenadas cambiaron
        }
        
        if curr.is_problematic != new_pkg.is_problematic {
            return true; // Marcado problemático cambió
        }
    }

    false
}

