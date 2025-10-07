use dioxus::prelude::*;

#[cfg(target_arch = "wasm32")]
use wasm_bindgen::{prelude::*, JsCast};

mod models;
mod maps;
mod config;

use models::{PackageData, demo};
// use config::CONFIG; // Commented out - not used yet
// use maps::{MapRenderer, MapConfig, MapStyle}; // TODO: Usar cuando implementemos mapas reales

// Función para detectar si es móvil (usando em units: <48em / 768px)
fn is_mobile() -> bool {
    #[cfg(target_arch = "wasm32")]
    {
        if let Some(window) = web_sys::window() {
            if let Ok(media_query) = window.match_media("(max-width: 47.99em)") {
                if let Some(media_query) = media_query {
                    return media_query.matches();
                }
            }
        }
    }
    false
}

// Paleta de colores para modo oscuro
const DARK_COLORS: ColorPalette = ColorPalette {
    // Colores base
    background: "#121420",      // Rich Black
    surface: "#2C2B3C",         // Raisin Black
    surface_variant: "#403F4C", // Onyx
    surface_container: "#1B2432", // Gunmetal
    
    // Colores de acento
    primary: "#B76D68",         // Indian Red
    primary_variant: "#A05D58", // Indian Red más oscuro
    secondary: "#403F4C",       // Onyx
    secondary_variant: "#2C2B3C", // Raisin Black
    
    // Colores de texto
    on_background: "#ffffff",
    on_surface: "#ffffff",
    on_primary: "#ffffff",
    on_secondary: "#B76D68",
    
    // Colores de estado
    success: "#1B2432",         // Gunmetal
    warning: "#B76D68",         // Indian Red
    error: "#B76D68",           // Indian Red
    info: "#403F4C",            // Onyx
    
    // Colores de borde
    outline: "#403F4C",         // Onyx
    outline_variant: "#2C2B3C", // Raisin Black
};

// Paleta de colores para modo claro (versión clara de la paleta oscura)
const LIGHT_COLORS: ColorPalette = ColorPalette {
    // Colores base (invertidos y aclarados)
    background: "#f8f9fa",      // Casi blanco
    surface: "#ffffff",         // Blanco puro
    surface_variant: "#f1f3f4", // Gris muy claro
    surface_container: "#e8eaed", // Gris claro
    
    // Colores de acento (versiones claras)
    primary: "#B76D68",         // Indian Red (mismo, funciona bien en claro)
    primary_variant: "#8B4A45", // Indian Red más oscuro
    secondary: "#5f6368",       // Gris medio
    secondary_variant: "#80868b", // Gris medio claro
    
    // Colores de texto (invertidos)
    on_background: "#121420",   // Rich Black
    on_surface: "#121420",      // Rich Black
    on_primary: "#ffffff",      // Blanco
    on_secondary: "#B76D68",    // Indian Red
    
    // Colores de estado
    success: "#2e7d32",         // Verde oscuro
    warning: "#f57c00",         // Naranja
    error: "#d32f2f",           // Rojo
    info: "#1976d2",            // Azul
    
    // Colores de borde
    outline: "#dadce0",         // Gris claro
    outline_variant: "#f1f3f4", // Gris muy claro
};

#[derive(Clone, Copy, PartialEq)]
struct ColorPalette {
    background: &'static str,
    surface: &'static str,
    surface_variant: &'static str,
    surface_container: &'static str,
    primary: &'static str,
    primary_variant: &'static str,
    secondary: &'static str,
    secondary_variant: &'static str,
    on_background: &'static str,
    on_surface: &'static str,
    on_primary: &'static str,
    on_secondary: &'static str,
    success: &'static str,
    warning: &'static str,
    error: &'static str,
    info: &'static str,
    outline: &'static str,
    outline_variant: &'static str,
}

fn main() {
    // Initialize logger based on platform
    #[cfg(target_os = "android")]
    android_logger::init_once(
        android_logger::Config::default()
            .with_max_level(log::LevelFilter::Debug)
            .with_tag("RouteOptimizer")
    );
    
    #[cfg(target_os = "ios")]
    oslog::OsLogger::new("com.daniel.routeoptimizer")
        .level_filter(log::LevelFilter::Debug)
        .init()
        .unwrap();
    
    #[cfg(target_arch = "wasm32")]
    {
        wasm_logger::init(wasm_logger::Config::default());
        console_error_panic_hook::set_once();
    }
    
    log::info!("🚀 Route Optimizer iniciando...");
    
    dioxus::launch(App);
}

#[component]
fn App() -> Element {
    log::info!("📱 App component renderizando");
    
    // Estado para el modo del sistema
    let is_dark_mode = use_signal(|| true); // Por defecto modo oscuro
    
    // Detectar modo del sistema al cargar
    use_effect(move || {
        #[cfg(target_arch = "wasm32")]
        {
            let window = web_sys::window().unwrap();

            // Crear media query para detectar modo oscuro
            if let Ok(media_query) = window.match_media("(prefers-color-scheme: dark)") {
                if let Some(media_query) = media_query {
                    let is_dark = media_query.matches();
                    is_dark_mode.set(is_dark);
                    log::info!("🌓 Modo del sistema detectado: {}", if is_dark { "oscuro" } else { "claro" });

                    // Escuchar cambios en el modo del sistema
                    let callback = Closure::wrap(Box::new(move |event: web_sys::MediaQueryListEvent| {
                        let is_dark = event.matches();
                        is_dark_mode.set(is_dark);
                        log::info!("🌓 Modo del sistema cambió a: {}", if is_dark { "oscuro" } else { "claro" });
                        
                        // Cambiar tema del mapa dinámicamente
                        change_map_theme(is_dark);
                    }) as Box<dyn FnMut(_)>);

                    let _ = media_query.set_onchange(Some(callback.as_ref().unchecked_ref()));
                    callback.forget();
                } else {
                    // Fallback si no se puede detectar
                    is_dark_mode.set(true);
                    log::info!("🌓 No se pudo detectar modo del sistema, usando oscuro por defecto");
                }
            } else {
                // Fallback si no se puede crear la media query
                is_dark_mode.set(true);
                log::info!("🌓 No se pudo crear media query, usando oscuro por defecto");
            }
        }
    });

    // Detectar tamaño de pantalla para responsive
    use_effect(move || {
        #[cfg(target_arch = "wasm32")]
        {
            let window = web_sys::window().unwrap();
            
            // Función para verificar si es móvil
            let check_mobile = || {
                if let Some(inner_width) = window.inner_width().ok() {
                    if let Some(width) = inner_width.as_f64() {
                    }
                }
            };
            
            // Verificar inicialmente
            check_mobile();
            
            // Escuchar cambios de tamaño
            let resize_callback = Closure::wrap(Box::new(move |_event: web_sys::Event| {
                check_mobile();
            }) as Box<dyn FnMut(_)>);
            
            let _ = window.add_event_listener_with_callback("resize", resize_callback.as_ref().unchecked_ref());
            resize_callback.forget();
        }
    });
    
    let colors = if *is_dark_mode.read() { DARK_COLORS } else { LIGHT_COLORS };
    
    rsx! {
        div {
            style: "width: 100vw; height: 100vh; background: {colors.background};",
            PackageDemo { colors }
        }
    }
}


#[component]
fn PackageDemo(colors: ColorPalette) -> Element {
    let mut packages = use_signal(|| demo::get_demo_packages());
    let mut show_sidebar = use_signal(|| false);
    let mut selected_tab = use_signal(|| "Paquetes");
    let mut selected_package = use_signal(|| None::<PackageData>);
    let mut isLoading = use_signal(|| false);
    let mut search_text = use_signal(|| String::new());
    let mut is_search_focused = use_signal(|| false);
    let mut dragging_package = use_signal(|| None::<PackageData>);
    let mut show_popup = use_signal(|| false);
    let mut popup_package = use_signal(|| None::<PackageData>);
    
    // Clonar packages para usar en los efectos
    let packages_for_update = packages.clone();
    
    // Inicializar mapa cuando se monta el componente
    use_effect(move || {
        // Esperar un poco para que el DOM esté listo
        let timeout = gloo_timers::callback::Timeout::new(1000, move || {
            init_mapbox_map();
        });
        timeout.forget();
    });
    
    // Actualizar paquetes cuando cambien
    use_effect(move || {
        update_map_packages(&packages_for_update.read());
    });
    
    // Limpiar highlight cuando se cierre el popup
    use_effect(move || {
        if !*show_popup.read() {
            clear_map_highlight();
        }
    });
    

    // Manejar click en marcadores del mapa
    use_effect(move || {
        #[cfg(target_arch = "wasm32")]
        {
            let window = web_sys::window().unwrap();
            let document = window.document().unwrap();
            
            // Clonar signals para el callback
            let mut show_sidebar_clone = show_sidebar.clone();
            let mut selected_tab_clone = selected_tab.clone();
            let mut selected_package_clone = selected_package.clone();
            let packages_clone = packages.clone();
            
            // Crear callback para el evento personalizado
            let callback = Closure::wrap(Box::new(move |event: web_sys::CustomEvent| {
                log::info!("🎯 Evento personalizado recibido desde el mapa");
                if let Some(package_data) = event.detail().dyn_into::<js_sys::Object>().ok() {
                    if let Ok(package_id) = js_sys::Reflect::get(&package_data, &"id".into()) {
                        if let Some(id_str) = package_id.as_string() {
                            log::info!("🖱️ Paquete seleccionado desde el mapa: {}", id_str);
                            
                            // Abrir sidebar
                            show_sidebar_clone.set(true);
                            
                            // Cambiar a la pestaña "Paquetes"
                            selected_tab_clone.set("Paquetes");
                            
                            // Encontrar el paquete en la lista
                            let packages_read = packages_clone.read();
                            if let Some(pkg) = packages_read.iter().find(|p| p.id == id_str) {
                                // Seleccionar el paquete
                                selected_package_clone.set(Some(pkg.clone()));
                                log::info!("✅ Paquete seleccionado en sidebar: {}", pkg.tracking_number);
                                
                                // Hacer scroll hasta el paquete
                                let pkg_id = pkg.id.clone();
                                gloo_timers::callback::Timeout::new(100, move || {
                                    if let Some(window) = web_sys::window() {
                                        if let Some(document) = window.document() {
                                            if let Some(element) = document.get_element_by_id(&format!("package-{}", pkg_id)) {
                                                let mut options = web_sys::ScrollIntoViewOptions::new();
                                                options.set_behavior(web_sys::ScrollBehavior::Smooth);
                                                options.set_block(web_sys::ScrollLogicalPosition::Center);
                                                element.scroll_into_view_with_scroll_into_view_options(&options);
                                            }
                                        }
                                    }
                                }).forget();
                            } else {
                                log::warn!("❌ No se encontró paquete con ID: {}", id_str);
                            }
                        }
                    }
                }
            }) as Box<dyn FnMut(_)>);
            
            // Agregar event listener para el evento personalizado
            let _ = document.add_event_listener_with_callback("packageSelectedFromMap", callback.as_ref().unchecked_ref());
            callback.forget();
        }
    });

    // Manejar cierre del sidebar por swipe
    use_effect(move || {
        #[cfg(target_arch = "wasm32")]
        {
            let window = web_sys::window().unwrap();
            let document = window.document().unwrap();
            
            // Clonar signal para el callback
            let mut show_sidebar_clone = show_sidebar.clone();
            
            // Crear callback para el evento de cierre
            let callback = Closure::wrap(Box::new(move |_event: web_sys::CustomEvent| {
                log::info!("👋 Sidebar cerrado por swipe");
                show_sidebar_clone.set(false);
            }) as Box<dyn FnMut(_)>);
            
            // Agregar event listener
            let _ = document.add_event_listener_with_callback("sidebarClosed", callback.as_ref().unchecked_ref());
            callback.forget();
        }
    });
    
    // Estado para los filtros de status
    let status_filter = use_signal(|| "Todos".to_string());
    
    // Filtrar paquetes por búsqueda y filtro de status
    let filtered_packages = packages.read().iter()
        .filter(|pkg| {
            // Solo mostrar paquetes en la pestaña "Paquetes"
            if *selected_tab.read() == "Configuración" {
                return false;
            }
            
            // Filtrar por status (desde configuración)
            let matches_status = match status_filter.read().as_ref() {
                "Pendientes" => pkg.status == "Pendiente",
                "En Ruta" => pkg.status == "En Ruta", 
                "Entregados" => pkg.status == "Entregado",
                _ => true, // "Todos"
            };
            
            // Filtrar por búsqueda si hay texto
            let matches_search = if search_text.read().is_empty() {
                true
            } else {
                let search = search_text.read().to_lowercase();
                pkg.tracking_number.to_lowercase().contains(&search) ||
                pkg.recipient_name.to_lowercase().contains(&search) ||
                pkg.address.to_lowercase().contains(&search)
            };
            
            matches_status && matches_search
        })
        .cloned()
        .collect::<Vec<_>>();
    
    // Calcular estadísticas
    let total_packages = packages.read().len();
    let packages_with_coords = packages.read().iter().filter(|p| p.has_coordinates()).count();
    let packages_without_coords = total_packages - packages_with_coords;
    
    rsx! {
        div {
            style: "
                position: fixed !important;
                top: 0 !important;
                left: 0 !important;
                width: 100vw !important;
                height: 100vh !important;
                overflow: hidden !important;
                display: flex !important;
                background: {colors.background};
                margin: 0 !important;
                padding: 0 !important;
                z-index: 1 !important;
            ",
            
            // Sidebar deslizable desde la izquierda (responsive)
            div {
                class: if *show_sidebar.read() { "sidebar open" } else { "sidebar" },
                style: if *show_sidebar.read() {
                    "width: 400px; overflow: hidden; transition: width 0.3s ease;"
                } else {
                    "width: 0; overflow: hidden; transition: width 0.3s ease;"
                },
                
                div {
                    class: "sidebar-content",
                    style: format!(
                        "
                        width: 400px;
                        height: 100vh;
                        background: {};
                        border-right: 1px solid {};
                        box-shadow: 2px 0 10px rgba(0,0,0,0.2);
                        display: flex;
                        flex-direction: column;
                    ",
                        colors.surface,
                        colors.outline
                    ),
                
                    // Tabs de selección en la parte superior
                    div {
                        class: "sidebar-header",
                        style: "
                            padding: 16px 20px 0 20px;
                            background: {colors.surface_variant};
                        ",
                        
                        // Tabs simplificadas: Paquetes y Configuración
                        div {
                            class: "sidebar-tabs",
                            style: "display: flex; gap: 4px; background: {colors.surface_container}; padding: 4px; border-radius: 12px;",
                            
                            for tab in ["Paquetes", "Configuración"] {
                                button {
                                    class: "sidebar-tab",
                                    style: if *selected_tab.read() == tab {
                                        "flex: 1; padding: 12px 16px; border: none; border-radius: 8px; font-size: 14px; font-weight: 600; cursor: pointer; transition: all 0.2s; background: {colors.primary}; color: {colors.on_primary};"
                                    } else {
                                        "flex: 1; padding: 12px 16px; border: none; border-radius: 8px; font-size: 14px; font-weight: 500; cursor: pointer; transition: all 0.2s; background: transparent; color: {colors.on_surface};"
                                    },
                                    onclick: move |_| {
                                        selected_tab.set(tab);
                                    },
                                    "{tab}"
                                }
                            }
                        }
                    }
                    
                    // Header del sidebar con búsqueda (solo en pestaña Paquetes)
                    if *selected_tab.read() == "Paquetes" {
                        div {
                            class: "search-header",
                            style: "
                                padding: 20px;
                                border-bottom: 1px solid {colors.outline};
                                background: {colors.surface_variant};
                                display: flex;
                                flex-direction: column;
                                gap: 16px;
                            ",
                        
                            // Título con contador
                            h2 {
                                class: "sidebar-title",
                                style: "margin: 0; color: {colors.on_surface}; font-size: 20px; font-weight: 600;",
                                "📦 Paquetes ({filtered_packages.len()})"
                            }
                        
                        // Barra de búsqueda elegante (como iOS)
                        div {
                            style: "display: flex; align-items: center; gap: 12px;",
                            
                            div {
                                style: "
                                    flex: 1;
                                    position: relative;
                                    display: flex;
                                    align-items: center;
                                ",
                                
                                input {
                                    class: "search-input",
                                    r#type: "text",
                                    placeholder: "Buscar paquetes...",
                                    value: "{search_text.read()}",
                                    style: "
                                        width: 100%;
                                        padding: 12px 16px;
                                        padding-right: 48px;
                                        border: 1px solid {colors.outline};
                                        border-radius: 24px;
                                        background: {colors.surface};
                                        color: {colors.on_surface};
                                        font-size: 14px;
                                        outline: none;
                                        transition: all 0.2s;
                                    ",
                                    oninput: move |evt| {
                                        search_text.set(evt.value());
                                    },
                                    onfocus: move |_| {
                                        is_search_focused.set(true);
                                    },
                                    onblur: move |_| {
                                        is_search_focused.set(false);
                                    }
                                }
                                
                                // Ícono de búsqueda o X
                                button {
                                    style: "
                                        position: absolute;
                                        right: 8px;
                                        width: 32px;
                                        height: 32px;
                                        border: none;
                                        border-radius: 16px;
                                        background: {colors.surface_container};
                                        color: {colors.primary};
                                        cursor: pointer;
                                        display: flex;
                                        align-items: center;
                                        justify-content: center;
                                        font-size: 16px;
                                        transition: all 0.2s;
                                    ",
                                    onclick: move |_| {
                                        if *is_search_focused.read() {
                                            search_text.set(String::new());
                                            is_search_focused.set(false);
                                        }
                                    },
                                    if *is_search_focused.read() { "✕" } else { "🔍" }
                                }
                            }
                        }
                    }
                    
                    // Botón Refrescar dentro del sidebar (solo en pestaña Paquetes)
                    if *selected_tab.read() == "Paquetes" {
                        div {
                            style: "padding: 0 20px 16px 20px; display: flex; justify-content: flex-end;",
                            
                            button {
                                class: "sidebar-button",
                                style: "
                                    padding: 8px 12px;
                                    background: {colors.surface};
                                    color: {colors.primary};
                                    border: 1px solid {colors.outline};
                                    border-radius: 6px;
                                    font-size: 12px;
                                    font-weight: 500;
                                    cursor: pointer;
                                    display: flex;
                                    align-items: center;
                                    justify-content: center;
                                    gap: 6px;
                                    transition: all 0.2s ease;
                                    box-shadow: 0 2px 4px rgba(0,0,0,0.1);
                                ",
                                onclick: move |_| {
                                    log::info!("🔄 Refrescando paquetes");
                                    isLoading.set(true);
                                    // Simular refresh
                                    let timeout = gloo_timers::callback::Timeout::new(2000, move || {
                                        isLoading.set(false);
                                    });
                                    timeout.forget();
                                },
                                "🔄 Refrescar"
                            }
                        }
                    }
                    }
                    // Contenido de paquetes
                    div {
                        class: "packages-container",
                        style: "
                            flex: 1;
                            overflow-y: auto;
                            padding: 16px 20px;
                        ",
                        
                        if *selected_tab.read() == "Configuración" {
                            SettingsTabContent { 
                                colors,
                                status_filter: status_filter.clone(),
                                total_packages,
                                packages_with_coords,
                                packages_without_coords
                            }
                        } else if *isLoading.read() {
                            div {
                                style: "
                                    display: flex;
                                    justify-content: center;
                                    align-items: center;
                                    height: 200px;
                                ",
                                "🔄 Cargando paquetes..."
                            }
                        } else if filtered_packages.is_empty() {
                            div {
                                style: "
                                    text-align: center;
                                    padding: 40px 20px;
                                    color: {colors.on_surface};
                                ",
                                "📦 No hay paquetes en esta categoría"
                            }
                        } else {
                            div {
                                style: "display: flex; flex-direction: column; gap: 12px;",
                                
                                for package in filtered_packages.iter() {
                                    div {
                                        id: "package-{package.id}",
                                        PackageCard { 
                                            package: package.clone(),
                                            is_selected: selected_package.read().as_ref().map_or(false, |p| p.id == package.id),
                                        on_select: move |pkg: PackageData| {
                                            selected_package.set(Some(pkg.clone()));
                                            log::info!("📍 Paquete seleccionado: {}", pkg.tracking_number);
                                            
                                            // Resaltar en el mapa
                                            log::info!("🎯 Llamando highlight_package_on_map con ID: {}", pkg.id);
                                            highlight_package_on_map(&pkg.id);
                                        },
                                        on_show_details: move |pkg: PackageData| {
                                            log::info!("📋 Mostrando detalles de: {}", pkg.tracking_number);
                                            popup_package.set(Some(pkg.clone()));
                                            show_popup.set(true);
                                        },
                                        on_drag_start: move |pkg: PackageData| {
                                            log::info!("🔄 Iniciando drag de: {}", pkg.tracking_number);
                                            dragging_package.set(Some(pkg.clone()));
                                        },
                                        on_drop: move |pkg: PackageData| {
                                            log::info!("🔄 Drop en: {}", pkg.tracking_number);
                                            
                                            // Obtener el paquete que se está arrastrando
                                            if let Some(dragging) = dragging_package.read().clone() {
                                                // Solo reordenar si es un paquete diferente
                                                if dragging.tracking_number != pkg.tracking_number {
                                                    let mut packages_list = packages.write();
                                                    
                                                    // Encontrar índices de ambos paquetes
                                                    let dragging_index = packages_list.iter().position(|p| p.tracking_number == dragging.tracking_number);
                                                    let drop_index = packages_list.iter().position(|p| p.tracking_number == pkg.tracking_number);
                                                    
                                                    if let (Some(drag_idx), Some(drop_idx)) = (dragging_index, drop_index) {
                                                        // Remover el paquete que se está arrastrando
                                                        let dragged_package = packages_list.remove(drag_idx);
                                                        
                                                        // Insertar en la nueva posición
                                                        let new_index = if drop_idx > drag_idx { drop_idx - 1 } else { drop_idx };
                                                        packages_list.insert(new_index, dragged_package);
                                                        
                                                        log::info!("✅ Paquete reordenado: {} movido a posición {}", dragging.tracking_number, new_index + 1);
                                                    }
                                                }
                                            }
                                            
                                            // Limpiar el paquete que se está arrastrando
                                            dragging_package.set(None);
                                        },
                                        colors
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
            
            // Mapa que se ajusta al ancho disponible
            div {
                style: "
                    flex: 1;
                    position: relative;
                    width: 100%;
                    height: 100vh;
                    overflow: hidden;
                ",
                
                MapView { packages: packages.read().clone() }
                
                  // Botón de flecha para toggle sidebar (siempre visible, cambia de dirección)
                  button {
                      class: "sidebar-toggle",
                      style: if *show_sidebar.read() {
                          format!("
                              position: fixed;
                              top: 50%;
                              left: 400px;
                              transform: translateY(-50%);
                              width: 40px;
                              height: 80px;
                              border: none;
                              border-radius: 0 12px 12px 0;
                              background: {};
                              color: {};
                              font-size: 24px;
                              cursor: pointer;
                              display: flex;
                              align-items: center;
                              justify-content: center;
                              box-shadow: 2px 0 10px rgba(0,0,0,0.2);
                              z-index: 9999;
                              transition: all 0.3s ease;
                              border: 1px solid {};
                              border-left: none;
                          ", colors.surface, colors.primary, colors.outline)
                      } else {
                          format!("
                              position: fixed;
                              top: 50%;
                              left: 0;
                              transform: translateY(-50%);
                              width: 40px;
                              height: 80px;
                              border: none;
                              border-radius: 0 12px 12px 0;
                              background: {};
                              color: {};
                              font-size: 24px;
                              cursor: pointer;
                              display: flex;
                              align-items: center;
                              justify-content: center;
                              box-shadow: 2px 0 10px rgba(0,0,0,0.2);
                              z-index: 9999;
                              transition: all 0.3s ease;
                              border: 1px solid {};
                              border-left: none;
                          ", colors.surface, colors.primary, colors.outline)
                      },
                    onclick: move |_| {
                        let new_state = !*show_sidebar.read();
                        show_sidebar.set(new_state);
                        log::info!("📋 {} sidebar", if new_state { "Abriendo" } else { "Cerrando" });
                    },
                    if *show_sidebar.read() {
                        "←"
                    } else {
                        "→"
                    }
                }
            }
            
            // Popup de detalles del paquete
            if *show_popup.read() {
                if let Some(pkg) = popup_package.read().as_ref() {
                    PackageDetailsPopup {
                        package: pkg.clone(),
                        on_close: move || {
                            show_popup.set(false);
                            popup_package.set(None);
                        },
                        colors
                    }
                }
            }
        }
    }
}


#[component]
fn PackageCard(
    package: PackageData,
    is_selected: bool,
    on_select: EventHandler<PackageData>,
    on_show_details: EventHandler<PackageData>,
    on_drag_start: EventHandler<PackageData>,
    on_drop: EventHandler<PackageData>,
    colors: ColorPalette,
) -> Element {
    let status_color = match package.status.as_str() {
        "Pendiente" => colors.warning,
        "En Ruta" => colors.info,
        "Entregado" => colors.success,
        _ => colors.secondary,
    };

    let status_bg = match package.status.as_str() {
        "Pendiente" => format!("{}20", colors.warning),
        "En Ruta" => format!("{}20", colors.info),
        "Entregado" => format!("{}20", colors.success),
        _ => format!("{}20", colors.secondary),
    };

    // Clone package for use in closures
    let pkg_for_select = package.clone();
    let pkg_for_details = package.clone();
    let pkg_for_drag_start = package.clone();
    let pkg_for_drop = package.clone();

    rsx! {
            div {
                class: "package-card",
                draggable: if is_mobile() { "false" } else { "true" },
            style: if is_selected {
                format!(
                    "
                    background: {};
                    border: 2px solid {};
                    border-radius: 12px;
                    box-shadow: 0 4px 12px rgba(0,0,0,0.2);
                    margin-bottom: 8px;
                    transition: all 0.2s ease;
                    cursor: pointer;
                    transform: scale(1.02);
                ",
                    colors.surface,
                    colors.primary
                )
            } else {
                format!(
                    "
                    background: {};
                    border: 1px solid {};
                    border-radius: 12px;
                    box-shadow: 0 2px 8px rgba(0,0,0,0.1);
                    margin-bottom: 8px;
                    transition: all 0.2s ease;
                    cursor: pointer;
                ",
                    colors.surface,
                    colors.outline
                )
            },
                onmouseenter: move |_| {
                    // Hover effect
                },
                onclick: move |_| {
                    // Si ya está seleccionado, mostrar popup
                    if is_selected {
                        on_show_details.call(pkg_for_details.clone());
                    } else {
                        // Si no está seleccionado, seleccionar
                        on_select.call(pkg_for_select.clone());
                    }
                },
                ondragstart: move |_evt| {
                    // NO prevenir default en dragstart para permitir el drag
                    on_drag_start.call(pkg_for_drag_start.clone());
                },
                ondragover: move |evt| {
                    evt.prevent_default(); // Sí prevenir default en dragover
                },
                ondrop: move |evt| {
                    evt.prevent_default(); // Sí prevenir default en drop
                    on_drop.call(pkg_for_drop.clone());
                },

            div {
                style: "display: flex; align-items: stretch; min-height: 60px;",

                    // Número de orden con fondo (como iOS)
                    if let Some(order) = package.num_ordre_passage_prevu {
                        div {
                            class: "order-number",
                            style: format!(
                                "
                                width: 50px;
                                background: {};
                                border-radius: 8px 0 0 8px;
                                display: flex;
                                align-items: center;
                                justify-content: center;
                                color: white;
                                font-weight: bold;
                                font-size: 18px;
                            ",
                                status_color
                            ),
                            "{order}"
                        }
                    }

                // Contenido principal
                div {
                    style: "flex: 1; padding: 12px; display: flex; align-items: center; gap: 12px;",

                        // Indicador de estado (punto pequeño)
                        div {
                            class: "status-indicator",
                            style: format!(
                                "
                                width: 12px;
                                height: 12px;
                                border-radius: 50%;
                                background: {};
                                flex-shrink: 0;
                            ",
                                status_color
                            )
                        }

                        // Información del paquete (más compacta)
                        div {
                            style: "flex: 1; min-width: 0;",

                            // Nombre del cliente (primero, como iOS)
                            h3 {
                                style: format!(
                                    "
                                    margin: 0 0 2px 0; 
                                    color: {}; 
                                    font-size: 14px; 
                                    font-weight: 600; 
                                    line-height: 1.2;
                                    white-space: nowrap;
                                    overflow: hidden;
                                    text-overflow: ellipsis;
                                ",
                                    colors.on_surface
                                ),
                                "{package.recipient_name}"
                            }

                            // Dirección (sin código postal para ahorrar espacio)
                            p {
                                style: format!(
                                    "
                                    margin: 0 0 2px 0; 
                                    color: {}; 
                                    font-size: 12px; 
                                    line-height: 1.2;
                                    white-space: nowrap;
                                    overflow: hidden;
                                    text-overflow: ellipsis;
                                ",
                                    colors.primary
                                ),
                                "{package.address.split(',').next().unwrap_or(&package.address)}"
                            }

                            // Número de seguimiento (solo si está seleccionado)
                            if is_selected {
                                p {
                                    style: format!(
                                        "
                                        margin: 0; 
                                        color: {}; 
                                        font-size: 10px; 
                                        font-weight: 500;
                                        white-space: nowrap;
                                        overflow: hidden;
                                        text-overflow: ellipsis;
                                    ",
                                        colors.info
                                    ),
                                    "{package.tracking_number}"
                                }
                            }
                        }

                    // Estado y botones de acción
                    div {
                        style: "display: flex; flex-direction: column; align-items: flex-end; gap: 4px;",

                        // Estado
                        span {
                            class: "status-badge",
                            style: format!(
                                "
                                color: {}; 
                                font-size: 12px; 
                                font-weight: 500;
                                padding: 2px 8px;
                                border-radius: 12px;
                                background: {};
                            ",
                                status_color,
                                status_bg
                            ),
                            "{package.status}"
                        }

                    }
                }
            }
        }
    }
}

#[component]
fn PackageDetailsPopup(
    package: PackageData,
    on_close: EventHandler<()>,
    colors: ColorPalette,
) -> Element {
    let status_color = match package.status.as_str() {
        "Pendiente" => colors.warning,
        "En Ruta" => colors.info,
        "Entregado" => colors.success,
        _ => colors.secondary,
    };

    rsx! {
        // Overlay de fondo
        div {
            style: "
                position: fixed;
                top: 0;
                left: 0;
                width: 100vw;
                height: 100vh;
                background: rgba(0,0,0,0.5);
                display: flex;
                align-items: center;
                justify-content: center;
                z-index: 10000;
            ",
            onclick: move |_| {
                on_close.call(());
            },
            
            // Popup principal
            div {
                style: "
                    background: {colors.surface};
                    border-radius: 16px;
                    box-shadow: 0 20px 40px rgba(0,0,0,0.3);
                    max-width: 500px;
                    width: 90%;
                    max-height: 80vh;
                    overflow-y: auto;
                    position: relative;
                ",
                onclick: move |evt| {
                    evt.stop_propagation();
                },
                
                // Header del popup
                div {
                    style: "
                        padding: 24px 24px 16px 24px;
                        border-bottom: 1px solid {colors.outline};
                        display: flex;
                        align-items: center;
                        justify-content: space-between;
                    ",
                    
                    h2 {
                        style: "
                            margin: 0;
                            color: {colors.on_surface};
                            font-size: 24px;
                            font-weight: 600;
                        ",
                        "📦 Detalles del Paquete"
                    }
                    
                    button {
                        style: "
                            width: 32px;
                            height: 32px;
                            border: none;
                            border-radius: 16px;
                            background: {colors.surface_container};
                            color: {colors.on_surface};
                            cursor: pointer;
                            display: flex;
                            align-items: center;
                            justify-content: center;
                            font-size: 18px;
                            transition: all 0.2s;
                        ",
                        onclick: move |_| {
                            on_close.call(());
                        },
                        "✕"
                    }
                }
                
                // Contenido del popup
                div {
                    style: "padding: 24px;",
                    
                    // Información principal
                    div {
                        style: "
                            display: flex;
                            align-items: center;
                            gap: 16px;
                            margin-bottom: 24px;
                            padding: 16px;
                            background: {colors.surface_variant};
                            border-radius: 12px;
                        ",
                        
                        // Número de orden
                        if let Some(order) = package.num_ordre_passage_prevu {
                            div {
                                style: format!(
                                    "
                                    width: 60px;
                                    height: 60px;
                                    background: {};
                                    border-radius: 12px;
                                    display: flex;
                                    align-items: center;
                                    justify-content: center;
                                    color: white;
                                    font-weight: bold;
                                    font-size: 24px;
                                ",
                                    status_color
                                ),
                                "{order}"
                            }
                        }
                        
                        // Información del paquete
                        div {
                            style: "flex: 1;",
                            
                            h3 {
                                style: "
                                    margin: 0 0 8px 0;
                                    color: {colors.on_surface};
                                    font-size: 20px;
                                    font-weight: 600;
                                ",
                                "{package.recipient_name}"
                            }
                            
                            p {
                                style: "
                                    margin: 0 0 4px 0;
                                    color: {colors.primary};
                                    font-size: 16px;
                                ",
                                "{package.address}"
                            }
                            
                            span {
                                style: format!(
                                    "
                                    color: {};
                                    font-size: 14px;
                                    font-weight: 500;
                                    padding: 4px 12px;
                                    border-radius: 12px;
                                    background: {}20;
                                ",
                                    status_color,
                                    status_color
                                ),
                                "{package.status}"
                            }
                        }
                    }
                    
                    // Detalles adicionales
                    div {
                        style: "display: flex; flex-direction: column; gap: 16px;",
                        
                        // Número de seguimiento
                        div {
                            style: "
                                display: flex;
                                justify-content: space-between;
                                align-items: center;
                                padding: 12px 0;
                                border-bottom: 1px solid {colors.outline};
                            ",
                            
                            span {
                                style: "
                                    color: {colors.on_surface};
                                    font-weight: 500;
                                ",
                                "Número de seguimiento:"
                            }
                            
                            span {
                                style: "
                                    color: {colors.primary};
                                    font-family: monospace;
                                    font-size: 14px;
                                ",
                                "{package.tracking_number}"
                            }
                        }
                        
                        // Fecha de entrega
                        if let Some(delivery_date) = &package.delivery_date {
                            div {
                                style: "
                                    display: flex;
                                    justify-content: space-between;
                                    align-items: center;
                                    padding: 12px 0;
                                    border-bottom: 1px solid {colors.outline};
                                ",
                                
                                span {
                                    style: "
                                        color: {colors.on_surface};
                                        font-weight: 500;
                                    ",
                                    "Fecha de entrega:"
                                }
                                
                                span {
                                    style: "
                                        color: {colors.primary};
                                        font-size: 14px;
                                    ",
                                    "{delivery_date}"
                                }
                            }
                        }
                        
                        // Instrucciones
                        if !package.instructions.is_empty() {
                            div {
                                style: "
                                    padding: 12px 0;
                                    border-bottom: 1px solid {colors.outline};
                                ",
                                
                                span {
                                    style: "
                                        color: {colors.on_surface};
                                        font-weight: 500;
                                        display: block;
                                        margin-bottom: 8px;
                                    ",
                                    "Instrucciones:"
                                }
                                
                                p {
                                    style: "
                                        margin: 0;
                                        color: {colors.primary};
                                        font-size: 14px;
                                        line-height: 1.4;
                                    ",
                                    "{package.instructions}"
                                }
                            }
                        }
                        
                        // Teléfono
                        if let Some(phone) = &package.phone {
                            div {
                                style: "
                                    display: flex;
                                    justify-content: space-between;
                                    align-items: center;
                                    padding: 12px 0;
                                    border-bottom: 1px solid {colors.outline};
                                ",
                                
                                span {
                                    style: "
                                        color: {colors.on_surface};
                                        font-weight: 500;
                                    ",
                                    "Teléfono:"
                                }
                                
                                span {
                                    style: "
                                        color: {colors.primary};
                                        font-size: 14px;
                                    ",
                                    "{phone}"
                                }
                            }
                        }
                        
                        // Prioridad
                        div {
                            style: "
                                display: flex;
                                justify-content: space-between;
                                align-items: center;
                                padding: 12px 0;
                            ",
                            
                            span {
                                style: "
                                    color: {colors.on_surface};
                                    font-weight: 500;
                                ",
                                "Prioridad:"
                            }
                            
                            span {
                                style: format!(
                                    "
                                    color: {};
                                    font-size: 14px;
                                    font-weight: 500;
                                    padding: 4px 8px;
                                    border-radius: 8px;
                                    background: {}20;
                                ",
                                    if package.priority == "Alta" { colors.warning } else { colors.info },
                                    if package.priority == "Alta" { colors.warning } else { colors.info }
                                ),
                                "{package.priority}"
                            }
                        }
                    }
                    
                    // Botones de acción
                    div {
                        style: "
                            display: flex;
                            gap: 12px;
                            margin-top: 24px;
                            padding-top: 16px;
                            border-top: 1px solid {colors.outline};
                        ",
                        
                        button {
                            style: "
                                flex: 1;
                                padding: 12px 24px;
                                border: 1px solid {colors.outline};
                                border-radius: 8px;
                                background: {colors.surface};
                                color: {colors.on_surface};
                                cursor: pointer;
                                font-size: 14px;
                                font-weight: 500;
                                transition: all 0.2s;
                            ",
                            onclick: move |_| {
                                log::info!("📍 Centrando en paquete: {}", package.tracking_number);
                                // Aquí implementarías la lógica para centrar en el mapa
                            },
                            "📍 Ver en Mapa"
                        }
                        
                        button {
                            style: "
                                flex: 1;
                                padding: 12px 24px;
                                border: none;
                                border-radius: 8px;
                                background: {colors.primary};
                                color: {colors.on_primary};
                                cursor: pointer;
                                font-size: 14px;
                                font-weight: 500;
                                transition: all 0.2s;
                            ",
                            onclick: move |_| {
                                log::info!("📞 Llamando a: {}", package.recipient_name);
                                // Aquí implementarías la lógica para llamar
                            },
                            "📞 Llamar"
                        }
                    }
                }
            }
        }
    }
}

#[component]
fn SettingsTabContent(
    colors: ColorPalette,
    mut status_filter: Signal<String>,
    total_packages: usize,
    packages_with_coords: usize,
    packages_without_coords: usize,
) -> Element {
    rsx! {
        div {
            style: "display: flex; flex-direction: column; gap: 20px;",
            
            h3 {
                style: "margin: 0 0 16px 0; color: {colors.on_surface}; font-size: 18px; font-weight: 600;",
                "⚙️ Configuración"
            }
            
            // Filtros de estado
            div {
                style: "
                    background: {colors.surface_variant};
                    border: 1px solid {colors.outline};
                    border-radius: 12px;
                    padding: 16px;
                ",
                
                h4 {
                    style: "margin: 0 0 12px 0; color: {colors.on_surface}; font-size: 16px; font-weight: 600;",
                    "🔍 Filtros de Estado"
                }
                
                div {
                    style: "display: flex; flex-wrap: wrap; gap: 8px;",
                    
                    for filter in ["Todos", "Pendientes", "En Ruta", "Entregados"] {
                        button {
                            style: if *status_filter.read() == filter {
                                format!(
                                    "
                                    padding: 10px 16px;
                                    border: none;
                                    border-radius: 20px;
                                    font-size: 14px;
                                    font-weight: 500;
                                    cursor: pointer;
                                    transition: all 0.2s;
                                    background: {};
                                    color: {};
                                ",
                                    colors.primary,
                                    colors.on_primary
                                )
                            } else {
                                format!(
                                    "
                                    padding: 10px 16px;
                                    border: 1px solid {};
                                    border-radius: 20px;
                                    font-size: 14px;
                                    font-weight: 500;
                                    cursor: pointer;
                                    transition: all 0.2s;
                                    background: {};
                                    color: {};
                                ",
                                    colors.outline,
                                    colors.surface,
                                    colors.on_surface
                                )
                            },
                            onclick: move |_| {
                                status_filter.set(filter.to_string());
                                log::info!("🔍 Filtro cambiado a: {}", filter);
                            },
                            "{filter}"
                        }
                    }
                }
            }
            
        }
    }
}

#[component]
fn MapView(packages: Vec<PackageData>) -> Element {
    rsx! {
        div {
            style: "
                width: 100%;
                height: 100%;
                position: absolute;
                top: 0;
                left: 0;
                z-index: 1;
            ",
            
            // Contenedor del mapa
            div {
                id: "map",
                style: "
                    width: 100%;
                    height: 100%;
                    position: absolute;
                    top: 0;
                    left: 0;
                    margin: 0;
                    padding: 0;
                    border: none;
                    outline: none;
                "
            }
            
        }
    }
}

// Funciones JavaScript para interactuar con Mapbox
#[cfg(target_arch = "wasm32")]
fn init_mapbox_map() {
    log::info!("🗺️ Inicializando Mapbox GL JS...");
    
    // Cargar Mapbox GL JS desde CDN
    load_mapbox_resources();
    
    // Esperar a que se cargue y crear el mapa
    let timeout = gloo_timers::callback::Timeout::new(2000, move || {
        create_mapbox_map_with_theme(true); // Por defecto modo oscuro
    });
    timeout.forget();
}

#[cfg(target_arch = "wasm32")]
fn load_mapbox_resources() {
    use web_sys::window;
    
    if let Some(window) = window() {
        if let Some(document) = window.document() {
            // Verificar si ya están cargados
            if document.get_element_by_id("mapbox-css").is_some() {
                return;
            }
            
            // Cargar CSS
            if let Some(head) = document.head() {
                let link = document.create_element("link").unwrap();
                link.set_attribute("id", "mapbox-css").unwrap();
                link.set_attribute("href", "https://api.mapbox.com/mapbox-gl-js/v3.15.0/mapbox-gl.css").unwrap();
                link.set_attribute("rel", "stylesheet").unwrap();
                let _ = head.append_child(&link);
            }
            
            // Cargar JS
            if let Some(head) = document.head() {
                let script = document.create_element("script").unwrap();
                script.set_attribute("id", "mapbox-js").unwrap();
                script.set_attribute("src", "https://api.mapbox.com/mapbox-gl-js/v3.15.0/mapbox-gl.js").unwrap();
                script.set_attribute("type", "text/javascript").unwrap();
                let _ = head.append_child(&script);
            }
        }
    }
}

#[cfg(target_arch = "wasm32")]
fn change_map_theme(is_dark_mode: bool) {
    use web_sys::window;
    
    if let Some(window) = window() {
        let style = if is_dark_mode {
            "mapbox://styles/mapbox/dark-v11"
        } else {
            "mapbox://styles/mapbox/light-v11"
        };
        
        let theme_script = format!(r#"
            if (window.mapboxMap) {{
                console.log('🌓 Cambiando tema del mapa a:', '{}');
                window.mapboxMap.setStyle('{}');
                console.log('✅ Tema del mapa cambiado correctamente');
            }}
        "#, 
        if is_dark_mode { "oscuro" } else { "claro" },
        style
        );
        
        if let Some(document) = window.document() {
            if let Ok(script) = document.create_element("script") {
                script.set_inner_html(&theme_script);
                if let Some(head) = document.head() {
                    let _ = head.append_child(&script);
                }
            }
        }
    }
}

#[cfg(target_arch = "wasm32")]
fn create_mapbox_map_with_theme(is_dark_mode: bool) {
    use web_sys::window;
    
    if let Some(window) = window() {
        // Verificar si mapboxgl está disponible
        if let Ok(mapboxgl) = js_sys::Reflect::get(&window, &JsValue::from_str("mapboxgl")) {
            log::info!("✅ Mapbox GL JS cargado correctamente");
            
            // Configurar token de acceso desde la configuración
            let mapbox_token = CONFIG.mapbox_token();
            log::info!("🗺️ Configurando Mapbox con token desde .env");
            
            let _ = js_sys::Reflect::set(&mapboxgl, &JsValue::from_str("accessToken"), 
                &JsValue::from_str(mapbox_token));
            
            // Crear el mapa usando eval para evitar problemas con constructores
            // Usar estilos básicos según el modo del sistema
            let style = if is_dark_mode {
                "mapbox://styles/mapbox/dark-v11"
            } else {
                "mapbox://styles/mapbox/light-v11"
            };
            
            let map_script = format!(r#"
                console.log('🗺️ Iniciando creación del mapa...');
                const map = new mapboxgl.Map({{
                    container: 'map',
                    style: '{}',
                    center: [2.3522, 48.8566], // París
                    zoom: 12
                }});
                
                // Agregar controles
                map.addControl(new mapboxgl.NavigationControl(), 'top-left');
                map.addControl(new mapboxgl.GeolocateControl({{
                    positionOptions: {{
                        enableHighAccuracy: true
                    }},
                    trackUserLocation: true,
                    showUserHeading: true
                }}), 'top-left');
                
                map.on('load', () => {{
                    console.log('🗺️ Mapa cargado correctamente');
                }});
                
                // Guardar referencia global
                window.mapboxMap = map;
                
                console.log('✅ Mapa Mapbox creado correctamente');
            "#, style);
            
            if let Some(document) = window.document() {
                if let Ok(script) = document.create_element("script") {
                    script.set_inner_html(&map_script);
                    if let Some(head) = document.head() {
                        let _ = head.append_child(&script);
                        log::info!("🗺️ Script de inicialización del mapa ejecutado");
                        
                        // Agregar marcadores después de un delay
                        let timeout = gloo_timers::callback::Timeout::new(1000, move || {
                            add_package_markers_simple();
                        });
                        timeout.forget();
                    }
                }
            }
        } else {
            log::warn!("⚠️ Mapbox GL JS no está cargado, reintentando...");
            // Reintentar después de un delay
            let timeout = gloo_timers::callback::Timeout::new(2000, move || {
                create_mapbox_map_with_theme(true); // Por defecto modo oscuro
            });
            timeout.forget();
        }
    }
}

#[cfg(target_arch = "wasm32")]
fn add_package_markers_simple() {
    use web_sys::window;
    
    if let Some(window) = window() {
        // Obtener paquetes de demo
        let packages = demo::get_demo_packages();
        
        // Crear script JavaScript para agregar marcadores
        let mut markers_script = String::from("if (window.mapboxMap) {");
        
        for package in packages.iter() {
            if let Some((lat, lng)) = package.coordinates() {
                markers_script.push_str(&format!(
                    r#"
                    const marker_{} = document.createElement('div');
                    marker_{}.className = 'package-marker';
                    marker_{}.style.cssText = `
                        background: #6B46C1;
                        border: 3px solid white;
                        border-radius: 50%;
                        width: 20px;
                        height: 20px;
                        cursor: pointer;
                        box-shadow: 0 2px 4px rgba(0,0,0,0.2);
                        display: flex;
                        align-items: center;
                        justify-content: center;
                        color: white;
                        font-size: 12px;
                        font-weight: bold;
                    `;
                    marker_{}.innerHTML = '📦';
                    
                    const mapboxMarker_{} = new mapboxgl.Marker(marker_{})
                        .setLngLat([{}, {}])
                        .addTo(window.mapboxMap);
                    
                    marker_{}.addEventListener('click', () => {{
                        console.log('🖱️ Click en paquete: {}');
                    }});
                    "#,
                    package.id.replace("-", "_"),
                    package.id.replace("-", "_"),
                    package.id.replace("-", "_"),
                    package.id.replace("-", "_"),
                    package.id.replace("-", "_"),
                    package.id.replace("-", "_"),
                    lng, lat,
                    package.id.replace("-", "_"),
                    package.tracking_number
                ));
            }
        }
        
        markers_script.push_str("console.log('📍 Marcadores agregados al mapa'); }");
        
        if let Some(document) = window.document() {
            if let Ok(script) = document.create_element("script") {
                script.set_inner_html(&markers_script);
                if let Some(head) = document.head() {
                    let _ = head.append_child(&script);
                    log::info!("📍 Script de marcadores ejecutado");
                }
            }
        }
    }
}

#[cfg(target_arch = "wasm32")]
fn update_map_packages(packages: &[PackageData]) {
    #[cfg(target_arch = "wasm32")]
    {
        log::info!("📦 Actualizando {} paquetes en el mapa", packages.len());
        
        let window = web_sys::window().unwrap();
        
        // Llamar a la función JavaScript para actualizar paquetes
        if let Ok(update_fn) = js_sys::Reflect::get(&window, &"mapboxUtils".into()) {
            if let Ok(update_packages_fn) = js_sys::Reflect::get(&update_fn, &"updatePackages".into()) {
                if let Ok(update_packages) = update_packages_fn.dyn_into::<js_sys::Function>() {
                    // Convertir paquetes a JavaScript
                    let packages_js = packages.iter().map(|pkg| {
                        let obj = js_sys::Object::new();
                        js_sys::Reflect::set(&obj, &"id".into(), &pkg.id.clone().into()).unwrap();
                        js_sys::Reflect::set(&obj, &"trackingNumber".into(), &pkg.tracking_number.clone().into()).unwrap();
                        js_sys::Reflect::set(&obj, &"recipientName".into(), &pkg.recipient_name.clone().into()).unwrap();
                        js_sys::Reflect::set(&obj, &"address".into(), &pkg.address.clone().into()).unwrap();
                        js_sys::Reflect::set(&obj, &"status".into(), &pkg.status.clone().into()).unwrap();
                        js_sys::Reflect::set(&obj, &"latitude".into(), &pkg.latitude.unwrap_or(0.0).into()).unwrap();
                        js_sys::Reflect::set(&obj, &"longitude".into(), &pkg.longitude.unwrap_or(0.0).into()).unwrap();
                        obj
                    }).collect::<js_sys::Array>();
                    
                    let _ = update_packages.call1(&update_fn, &packages_js);
                }
            }
        }
    }
    
    #[cfg(not(target_arch = "wasm32"))]
    {
        log::info!("📦 Actualizando {} paquetes en el mapa (solo web)", packages.len());
    }
}

#[cfg(target_arch = "wasm32")]
fn center_on_user_location() {
    log::info!("📍 Centrando en ubicación del usuario");
    // Implementación simplificada por ahora
}

#[cfg(target_arch = "wasm32")]
fn fit_to_packages(packages: &[PackageData]) {
    log::info!("📦 Ajustando vista a {} paquetes", packages.len());
    // Implementación simplificada por ahora
}

// Stubs para compilación en otras plataformas
#[cfg(not(target_arch = "wasm32"))]
fn init_mapbox_map() {
    log::info!("🗺️ Mapbox solo disponible en web");
}

#[cfg(not(target_arch = "wasm32"))]
fn update_map_packages(_packages: &[PackageData]) {
    log::info!("📦 Actualización de paquetes solo disponible en web");
}

#[cfg(not(target_arch = "wasm32"))]
fn center_on_user_location() {
    log::info!("📍 Geolocalización solo disponible en web");
}

#[cfg(not(target_arch = "wasm32"))]
fn fit_to_packages(_packages: &[PackageData]) {
    log::info!("📦 Ajuste de vista solo disponible en web");
}

#[cfg(target_arch = "wasm32")]
fn highlight_package_on_map(package_id: &str) {
    use web_sys::window;
    
    if let Some(window) = window() {
        if let Ok(mapbox_utils) = js_sys::Reflect::get(&window, &"mapboxUtils".into()) {
            if let Ok(highlight_fn) = js_sys::Reflect::get(&mapbox_utils, &"highlightPackage".into()) {
                if let Ok(highlight) = highlight_fn.dyn_into::<js_sys::Function>() {
                    let _ = highlight.call1(&mapbox_utils, &package_id.into());
                    log::info!("✨ Resaltando paquete en el mapa: {}", package_id);
                }
            }
        }
    }
}

#[cfg(target_arch = "wasm32")]
fn clear_map_highlight() {
    use web_sys::window;
    
    if let Some(window) = window() {
        if let Ok(mapbox_utils) = js_sys::Reflect::get(&window, &"mapboxUtils".into()) {
            if let Ok(clear_fn) = js_sys::Reflect::get(&mapbox_utils, &"clearHighlight".into()) {
                if let Ok(clear) = clear_fn.dyn_into::<js_sys::Function>() {
                    let _ = clear.call0(&mapbox_utils);
                    log::info!("✨ Limpiando highlights del mapa");
                }
            }
        }
    }
}

#[cfg(not(target_arch = "wasm32"))]
fn highlight_package_on_map(_package_id: &str) {
    log::info!("✨ Highlight solo disponible en web");
}

#[cfg(not(target_arch = "wasm32"))]
fn clear_map_highlight() {
    log::info!("✨ Clear highlight solo disponible en web");
}