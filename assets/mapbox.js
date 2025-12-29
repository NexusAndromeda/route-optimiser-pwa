// ============================================================================
// MAPBOX HELPER FUNCTIONS
// ============================================================================
// Funciones helper para interactuar con Mapbox desde Rust/WASM
// ============================================================================

let map = null;
let geolocateControl = null;
let currentDriverLocation = null;
let selectedPackageIndex = null;

// Initialize Mapbox
window.initMapbox = function(containerId, isDark = true) {
    console.log('🗺️ Initializing Mapbox...');
    console.log('Container:', containerId);
    console.log('Dark mode:', isDark);
    
    const container = document.getElementById(containerId);
    if (!container) {
        console.error('❌ Container not found:', containerId);
        return;
    }
    
    // Wait for container to be properly sized and visible
    const initMap = () => {
        const rect = container.getBoundingClientRect();
        const styles = window.getComputedStyle(container);
        console.log('📏 Container size:', rect.width, 'x', rect.height);
        console.log('👁️ Container visibility:', styles.display, styles.visibility, styles.opacity);
        
        if (rect.width === 0 || rect.height === 0 || 
            styles.display === 'none' || 
            styles.visibility === 'hidden' || 
            styles.opacity === '0') {
            console.log('⏳ Container not ready yet, retrying...');
            setTimeout(initMap, 200);
            return;
        }
        
        // Additional check: ensure container is actually visible in viewport
        if (rect.top < 0 || rect.left < 0) {
            console.log('⏳ Container not in viewport, retrying...');
            setTimeout(initMap, 200);
            return;
        }
    
    // Set token directly (public token for frontend - needs URL restrictions configured in Mapbox dashboard)
    // For development: add localhost:8080 to allowed URLs in Mapbox dashboard
    // Or create a new public token without URL restrictions for development
    const token = 'pk.eyJ1IjoiZGFuaWVsaG5jdCIsImEiOiJjbWZmZ29lcmMwN3l6MnFxeTg2ZHIzanNiIn0.aJPgsMEK-eX90x4zHsJIiw';
    
    // Set token
    mapboxgl.accessToken = token;
    
    // Choose style based on theme
    const mapStyle = isDark ? 'mapbox://styles/mapbox/dark-v11' : 'mapbox://styles/mapbox/light-v11';
    
    try {
        // Create map
        map = new mapboxgl.Map({
            container: containerId,
            style: mapStyle,
            center: [2.3522, 48.8566], // Paris center
            zoom: 12,
            attributionControl: false
        });
        
        console.log('✅ Map created');
        
        // Actualizar tamaño del mapa después de crearlo
        setTimeout(() => {
            if (window.updateMapSizeForBottomSheet) {
                window.updateMapSizeForBottomSheet();
            }
        }, 100);
        
        // Add navigation controls
        map.addControl(new mapboxgl.NavigationControl(), 'top-right');
        
        // Add geolocate control
        geolocateControl = new mapboxgl.GeolocateControl({
            positionOptions: {
                enableHighAccuracy: true
            },
            trackUserLocation: true,
            showUserHeading: true
        });
        
        // Listen for geolocation events
        geolocateControl.on('geolocate', (e) => {
            currentDriverLocation = {
                latitude: e.coords.latitude,
                longitude: e.coords.longitude
            };
            console.log('📍 Ubicación del chofer capturada:', currentDriverLocation);
        });
        
        map.addControl(geolocateControl, 'top-right');
        
        // Cuando el mapa carga, el estilo ya debería estar listo
        map.on('load', () => {
            console.log('✅ Map loaded');
            if (window.pendingPackagesJson) {
                console.log('📦 Found pending packages, adding them now...');
                // Llamar directamente a la función interna para evitar duplicar listeners
                const packages = JSON.parse(window.pendingPackagesJson);
                if (map.isStyleLoaded()) {
                    // Función inline para agregar paquetes (similar a actuallyAddPackages pero sin duplicar código)
                    window.addPackagesToMap(window.pendingPackagesJson);
                }
            }
        });
        
        map.on('error', (e) => {
            console.error('❌ Map error:', e);
        });
        
        // Listen for theme changes
        if (window.matchMedia) {
            window.matchMedia('(prefers-color-scheme: dark)').addEventListener('change', (e) => {
                const newStyle = e.matches ? 'mapbox://styles/mapbox/dark-v11' : 'mapbox://styles/mapbox/light-v11';
                console.log('🎨 Changing map theme:', e.matches ? 'Dark' : 'Light');
                if (map) {
                    map.setStyle(newStyle);
                    // Cuando el estilo cambia, re-agregar paquetes pendientes
                    map.once('style.load', () => {
                        if (window.pendingPackagesJson) {
                            window.addPackagesToMap(window.pendingPackagesJson);
                        }
                    });
                }
            });
        }
        
        // Listen for window resize to ensure map is properly sized
        window.addEventListener('resize', () => {
            if (map) {
                setTimeout(() => {
                    map.resize();
                }, 100);
            }
        });
        
        // Initial resize after a short delay
        setTimeout(() => {
            if (map) {
                console.log('🔄 Map initial resize');
                map.resize();
            }
        }, 200);
        
        // MutationObserver para detectar cambios en el DOM del container
        const observer = new MutationObserver(() => {
            if (map && window.updateMapSizeForBottomSheet) {
                // Debounce para evitar demasiadas llamadas
                if (window.mapResizeTimeout) {
                    clearTimeout(window.mapResizeTimeout);
                }
                window.mapResizeTimeout = setTimeout(() => {
                    window.updateMapSizeForBottomSheet();
                }, 50);
            }
        });
        
        observer.observe(document.body, {
            attributes: true,
            attributeFilter: ['style', 'class'],
            childList: true,
            subtree: true
        });
        
        // Resize cuando el DOM está listo
        if (document.readyState === 'loading') {
            document.addEventListener('DOMContentLoaded', () => {
                if (map) {
                    console.log('🔄 Map DOM ready resize');
                setTimeout(() => {
                        map.resize();
                    }, 100);
                    }
            });
        } else {
            setTimeout(() => {
                if (map) {
                    console.log('🔄 Map DOM ready resize');
                    map.resize();
                }
            }, 100);
        }
    } catch (error) {
        console.error('❌ Error creating map:', error);
    }
    };
    
    // Start initialization
    initMap();
};

// Add packages to map as Style Layers
window.addPackagesToMap = function(packagesJson) {
    console.log('━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━');
    console.log('🗺️ MAPBOX: ADD_PACKAGES_TO_MAP');
    console.log('━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━');
    
    if (!map) {
        console.error('❌ Map not initialized');
        // Guardar para cuando el mapa esté listo
        if (packagesJson) {
            window.pendingPackagesJson = packagesJson;
        }
        return;
    }
    
    try {
        // Require packagesJson to be provided - don't use fallback to avoid clearing packages
        if (!packagesJson) {
            console.warn('⚠️ addPackagesToMap called without packagesJson, ignoring to prevent clearing existing packages');
            return;
        }
        
        // Guardar packagesJson para cuando el mapa esté listo
        window.pendingPackagesJson = packagesJson;
        
        let packages = JSON.parse(packagesJson);
        console.log(`📦 Total paquetes recibidos: ${packages.length}`);
        
        // Función interna para agregar paquetes cuando el mapa esté listo
        const actuallyAddPackages = (packagesData) => {
            if (!map || !map.isStyleLoaded()) {
                console.log('⏳ Map style not loaded yet');
                return false;
            }
            
            console.log(`✅ Map style loaded, adding ${packagesData.length} packages...`);
        
        // Remove existing source and layers
        if (map.getSource('packages')) {
            map.removeLayer('packages-circles');
            map.removeLayer('packages-labels');
            map.removeSource('packages');
        }
        
        // Create GeoJSON data from packages
        let skippedCount = 0;
        const geojsonData = {
            type: 'FeatureCollection',
                features: packagesData.map((pkg, index) => {
                // Skip packages without valid coordinates
                if (!pkg.coords || !Array.isArray(pkg.coords) || pkg.coords.length !== 2) {
                    console.warn(`⚠️ Paquete ${index} sin coordenadas válidas: id=${pkg.id}, coords=${JSON.stringify(pkg.coords)}`);
                    skippedCount++;
                    return null;
                }
                
                const groupIdx = pkg.group_idx !== undefined ? pkg.group_idx : index;
                
                // Log primeros 10 para debugging
                if (index < 10) {
                    console.log(`📍 [${index}] group_idx=${groupIdx}, id=${pkg.id}, address=${pkg.address}, coords=[${pkg.coords[0]}, ${pkg.coords[1]}]`);
                }
                
                return {
                    type: 'Feature',
                    geometry: {
                        type: 'Point',
                        coordinates: [pkg.coords[1], pkg.coords[0]] // Convertir de [lat, lng] a [lng, lat]
                    },
                    properties: {
                        id: pkg.id,
                        index: index, // Índice en el array filtrado (para visualización)
                        group_idx: groupIdx, // ⭐ Índice original del grupo
                        status: pkg.status,
                        code_statut_article: pkg.code_statut_article || null,
                        type_livraison: pkg.type_livraison || 'DOMICILE',
                        recipient: pkg.recipient,
                        address: pkg.address,
                        isSelected: selectedPackageIndex === groupIdx, // ⭐ Comparar con group_idx
                        is_problematic: pkg.is_problematic || false
                    }
                };
            }).filter(feature => feature !== null)
        };
        
        console.log(`✅ Features creados: ${geojsonData.features.length} (saltados: ${skippedCount})`);
        
        // Add source
        map.addSource('packages', {
            type: 'geojson',
            data: geojsonData
        });
        
        // Add circles layer
        map.addLayer({
            id: 'packages-circles',
            type: 'circle',
            source: 'packages',
            paint: {
                'circle-radius': [
                    'case',
                    ['get', 'isSelected'], 12, // Selected: larger
                    8 // Default: smaller
                ],
                'circle-color': [
                    'case',
                    // Por tipo de entrega - FONDO (sin cambiar por selección):
                    ['==', ['get', 'type_livraison'], 'RELAIS'], 'rgb(97, 38, 122)', // RELAIS: morado
                    ['==', ['get', 'type_livraison'], 'RCS'], '#F59E0B', // RCS: dorado
                    // DOMICILE - por estado de entrega:
                    ['has', 'code_statut_article'], [
                        'case',
                        ['in', 'STATUT_LIVRER', ['get', 'code_statut_article']], '#10B981', // Verde - Entregado
                        ['in', 'STATUT_NONLIV', ['get', 'code_statut_article']], '#EF4444', // Rojo - No entregado
                        ['==', ['get', 'code_statut_article'], 'STATUT_RECEPTIONNER'], '#06B6D4', // Cyan - Recepcionado
                        ['==', ['get', 'code_statut_article'], 'STATUT_COLLECTE'], '#EC4899', // Magenta - En recogida
                        '#3B82F6' // Azul - STATUT_CHARGER o default
                    ],
                    '#3B82F6' // Default: blue (si no tiene code_statut_article)
                ],
                'circle-stroke-width': [
                    'case',
                    ['get', 'isSelected'], 3, // Selected: borde más grueso para efecto pulsante
                    ['get', 'is_problematic'], 1.5, // Problemático: borde visible
                    0 // Sin borde para el resto
                ],
                'circle-stroke-color': [
                    'case',
                    // Selected: borde del mismo color del fondo (para efecto pulsante)
                    ['get', 'isSelected'], [
                        'case',
                        ['==', ['get', 'type_livraison'], 'RELAIS'], 'rgb(97, 38, 122)', // RELAIS: morado
                        ['==', ['get', 'type_livraison'], 'RCS'], '#F59E0B', // RCS: dorado
                        ['has', 'code_statut_article'], [
                            'case',
                            ['in', 'STATUT_LIVRER', ['get', 'code_statut_article']], '#10B981', // Verde
                            ['in', 'STATUT_NONLIV', ['get', 'code_statut_article']], '#EF4444', // Rojo
                            ['==', ['get', 'code_statut_article'], 'STATUT_RECEPTIONNER'], '#06B6D4', // Cyan
                            ['==', ['get', 'code_statut_article'], 'STATUT_COLLECTE'], '#EC4899', // Magenta
                            '#3B82F6' // Azul
                        ],
                        '#3B82F6' // Default
                    ],
                    ['get', 'is_problematic'], '#EF4444', // Problemático: rojo pulsante
                    'transparent' // Sin borde para el resto
                ],
                'circle-opacity': [
                    'case',
                    ['get', 'isSelected'], 0.9, // Selected: ligeramente transparente para efecto
                    1 // Default: opaco
                ],
                'circle-stroke-opacity': [
                    'case',
                    ['get', 'isSelected'], 0.6, // Selected: borde semi-transparente
                    1 // Default: opaco
                ]
            }
        });
        
        // Add labels layer (package numbers)
        // ⭐ IMPORTANTE: Usar group_idx (índice original del grupo) en lugar de index (índice filtrado)
        // Esto asegura que el número en el mapa coincida con el índice del grupo en la lista
        map.addLayer({
            id: 'packages-labels',
            type: 'symbol',
            source: 'packages',
            layout: {
                'text-field': ['to-string', ['+', 
                    ['get', 'group_idx'], 
                    1
                ]],
                'text-font': ['Open Sans Bold', 'Arial Unicode MS Bold'],
                'text-size': 11,
                'text-anchor': 'center'
            },
            paint: {
                'text-color': '#FFFFFF',
                'text-halo-color': 'rgba(0,0,0,0.5)',
                'text-halo-width': 1
            }
        });
        
        // Add click event listener
        map.on('click', 'packages-circles', (e) => {
            console.log('━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━');
            console.log('🖱️ MAPBOX: CLICK EN PUNTO DEL MAPA');
            console.log('━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━');
            
            const props = e.features[0].properties;
            // ⭐ Usar group_idx (índice original del grupo) en lugar de index (índice filtrado)
            const groupIdx = props.group_idx !== undefined ? props.group_idx : props.index;
            
            console.log(`   📍 Props del punto:`);
            console.log(`      - id: ${props.id}`);
            console.log(`      - index (filtrado): ${props.index}`);
            console.log(`      - group_idx (original): ${props.group_idx}`);
            console.log(`      - address: ${props.address}`);
            console.log(`      - recipient: ${props.recipient}`);
            console.log(`   ✅ groupIdx seleccionado: ${groupIdx}`);
            
            // Trigger custom event that Yew can listen to
            const event = new CustomEvent('packageSelected', { detail: { index: groupIdx } });
            window.dispatchEvent(event);
            console.log(`   📤 Evento 'packageSelected' disparado con groupIdx: ${groupIdx}`);
            console.log('━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━');
        });
        
        // Change cursor on hover
        map.on('mouseenter', 'packages-circles', () => {
            map.getCanvas().style.cursor = 'pointer';
        });
        
        map.on('mouseleave', 'packages-circles', () => {
            map.getCanvas().style.cursor = '';
        });
        
        console.log(`✅ ${geojsonData.features.length} packages added as Style Layers`);
        console.log('━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━');
            return true;
        };
        
        // Intentar agregar paquetes ahora
        if (!actuallyAddPackages(packages)) {
            // Si el mapa no está listo, esperar a que lo esté
            console.log('⏳ Waiting for map style to load...');
            // Registrar listener para cuando el estilo cargue (si aún no está registrado)
            map.once('style.load', () => {
                console.log('✅ Map style loaded, retrying addPackagesToMap...');
                if (window.pendingPackagesJson) {
                    actuallyAddPackages(JSON.parse(window.pendingPackagesJson));
                }
            });
        }
        
    } catch (error) {
        console.error('❌ Error adding packages to map:', error);
    }
};

// Update selected package
window.updateSelectedPackage = function(groupIdx) {
    console.log('━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━');
    console.log('🗺️ MAPBOX: UPDATE_SELECTED_PACKAGE');
    console.log('━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━');
    console.log(`   📍 groupIdx recibido: ${groupIdx}`);
    
    if (!map || !map.getSource('packages')) {
        console.warn('   ⚠️  Mapa o source no disponible');
        return;
    }
    
    selectedPackageIndex = groupIdx;
    
    // Update source data to mark selected package
    const source = map.getSource('packages');
    if (source && source._data) {
        source._data.features.forEach((feature) => {
            feature.properties.isSelected = feature.properties.group_idx === groupIdx;
        });
        source.setData(source._data);
    }
};

// Center map on a specific package
window.centerMapOnPackage = function(groupIdx) {
    console.log('━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━');
    console.log('🗺️ MAPBOX: CENTER_MAP_ON_PACKAGE');
    console.log('━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━');
    console.log(`   📍 groupIdx: ${groupIdx}`);
    
    if (!map || !map.getSource('packages')) {
        console.warn('   ⚠️  Mapa o source no disponible');
        return;
    }
    
    const source = map.getSource('packages');
    if (!source || !source._data) {
        console.error('❌ No package data found');
        return;
    }
    
    // Find package by group_idx
    const feature = source._data.features.find(f => f.properties.group_idx === groupIdx);
    if (!feature) {
        console.error(`❌ Package with group_idx ${groupIdx} not found`);
        return;
    }
    
    const [lng, lat] = feature.geometry.coordinates;
    console.log(`   📍 Centrando en: [${lat}, ${lng}]`);
        
            map.flyTo({
            center: [lng, lat],
        zoom: 15,
        duration: 1000
    });
};

// Scroll to selected package in the bottom sheet
// MEJORADO: Hace scroll dentro del contenedor .package-list en lugar del documento completo
window.scrollToSelectedPackage = function(groupIdx) {
    console.log('━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━');
    console.log('📜 MAPBOX: SCROLL_TO_SELECTED_PACKAGE (MEJORADO)');
    console.log('━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━');
    console.log(`   📍 groupIdx: ${groupIdx}`);
    
    // Find the package card in the DOM
    const cards = document.querySelectorAll('.package-card[data-index]');
    console.log(`   📦 package-cards encontrados: ${cards.length}`);
    
    if (cards.length === 0) {
        console.warn('   ⚠️  No se encontraron package-cards');
        return;
    }
    
    // Find card with matching data-index
    let targetCard = null;
    for (let card of cards) {
        const cardIndex = parseInt(card.getAttribute('data-index'), 10);
        if (cardIndex === groupIdx) {
            targetCard = card;
            break;
        }
    }
    
    if (!targetCard) {
        console.warn(`   ⚠️  Package card con groupIdx ${groupIdx} no encontrado`);
        return;
    }
    
    console.log(`   ✅ package-card encontrado en índice ${groupIdx}`);
    
    // Buscar el contenedor .package-list (el que tiene overflow-y: auto)
    const packageListContainer = document.querySelector('.package-list');
    
    if (packageListContainer) {
        console.log(`   📜 Haciendo scroll dentro del contenedor .package-list...`);
        
        // Obtener posiciones relativas
        const cardRect = targetCard.getBoundingClientRect();
        const containerRect = packageListContainer.getBoundingClientRect();
        
        // Calcular posición del card dentro del contenedor
        const cardTop = cardRect.top - containerRect.top + packageListContainer.scrollTop;
        const cardHeight = cardRect.height;
        const containerHeight = packageListContainer.clientHeight;
        
        // Calcular scrollTop para centrar el card
        const targetScrollTop = cardTop - (containerHeight / 2) + (cardHeight / 2);
        
        // Hacer scroll suave usando requestAnimationFrame con easing
        const startScrollTop = packageListContainer.scrollTop;
        const distance = targetScrollTop - startScrollTop;
        const duration = 300; // ms
        const startTime = performance.now();
        
        function animateScroll(currentTime) {
            const elapsed = currentTime - startTime;
            const progress = Math.min(elapsed / duration, 1);
            
            // Easing function: ease-out cubic (suave y natural)
            const easeOutCubic = 1 - Math.pow(1 - progress, 3);
            
            packageListContainer.scrollTop = startScrollTop + (distance * easeOutCubic);
            
            if (progress < 1) {
                requestAnimationFrame(animateScroll);
            } else {
                // Agregar clase flash al card después del scroll
                targetCard.classList.add('flash');
                setTimeout(() => {
                    targetCard.classList.remove('flash');
                }, 500);
                
                console.log(`   ✅ Scroll completado dentro del contenedor y animación flash iniciada`);
            }
        }
        
        requestAnimationFrame(animateScroll);
    } else {
        // Fallback: si no se encuentra .package-list, usar scrollIntoView normal
        console.warn(`   ⚠️  Contenedor .package-list no encontrado, usando scrollIntoView como fallback`);
        console.log(`   📜 Haciendo scroll a grupo ${groupIdx}...`);
        
        targetCard.scrollIntoView({
            behavior: 'smooth',
            block: 'center'
        });
        
        // Add flash animation class
        targetCard.classList.add('flash');
        setTimeout(() => {
            targetCard.classList.remove('flash');
        }, 500);
        
        console.log(`   ✅ Scroll completado (fallback) y animación flash iniciada`);
    }
};

// Update map size for bottom sheet
window.updateMapSizeForBottomSheet = function() {
    if (!map) return;
    
    // Obtener altura del bottom sheet desde CSS variable
    const root = document.documentElement;
    const computedStyle = window.getComputedStyle(root);
    const sheetHeight = computedStyle.getPropertyValue('--bottom-sheet-height').trim();
    
    if (!sheetHeight) {
        console.warn('⚠️ No se pudo obtener --bottom-sheet-height del CSS');
        return;
    }
    
    // Limpiar cualquier intervalo activo
    if (window.activeResizeInterval) {
        clearInterval(window.activeResizeInterval);
        window.activeResizeInterval = null;
    }
    
    // Estrategia de resize progresivo para fluidez durante transiciones CSS
    // 1. Resize inmediato en el próximo frame para respuesta instantánea
    requestAnimationFrame(() => {
        map.resize();
    });
    
    // 2. Resize múltiple durante la transición (300ms) cada 40ms para fluidez
    let resizeCount = 0;
    const maxResizes = Math.ceil(300 / 40); // ~7-8 resizes durante la transición
    
    window.activeResizeInterval = setInterval(() => {
        if (resizeCount >= maxResizes) {
            clearInterval(window.activeResizeInterval);
            window.activeResizeInterval = null;
            return;
        }
        map.resize();
        resizeCount++;
    }, 40);
    
    // 3. Resize final después de la transición para precisión
    setTimeout(() => {
        if (window.activeResizeInterval) {
            clearInterval(window.activeResizeInterval);
            window.activeResizeInterval = null;
        }
            map.resize();
    }, 300 + 20); // 20ms de margen después de la transición
};

// MutationObserver para detectar cambios en bottom-sheet y actualizar tamaño del mapa
(function() {
    const bottomSheet = document.getElementById('bottom-sheet');
    if (!bottomSheet) {
        // Si el bottom-sheet no existe aún, crear observer después de un delay
        setTimeout(arguments.callee, 100);
        return;
    }
    
    const observer = new MutationObserver((mutations) => {
        mutations.forEach((mutation) => {
            if (mutation.type === 'attributes' && mutation.attributeName === 'class') {
                const target = mutation.target;
                if (target.id === 'bottom-sheet') {
                    console.log('🔍 [OBSERVER] Detectado cambio de clase en bottom-sheet');
                    // Debounce para evitar demasiadas llamadas
                    if (window.mapResizeDebounce) {
                        clearTimeout(window.mapResizeDebounce);
                    }
                    window.mapResizeDebounce = setTimeout(() => {
                        if (window.updateMapSizeForBottomSheet) {
                window.updateMapSizeForBottomSheet();
                        }
                    }, 50);
                }
            }
        });
    });
    
    observer.observe(bottomSheet, {
        attributes: true,
        attributeFilter: ['class']
    });
})();

// Initialize update when DOM is ready
(function() {
    if (document.readyState === 'loading') {
        document.addEventListener('DOMContentLoaded', () => {
            setTimeout(() => {
                if (window.updateMapSizeForBottomSheet) {
        window.updateMapSizeForBottomSheet();
                }
            }, 100);
    });
    } else {
        setTimeout(() => {
            if (window.updateMapSizeForBottomSheet) {
    window.updateMapSizeForBottomSheet();
            }
        }, 100);
    }
})();

console.log('📍 Mapbox helper functions loaded');
