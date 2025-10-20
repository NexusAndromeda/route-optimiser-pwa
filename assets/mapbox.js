// Mapbox initialization and helper functions
let map = null;
let selectedPackageIndex = null;
let pulseAnimationId = null;

// Initialize Mapbox map
window.initMapbox = function(containerId, isDark) {
    console.log('🗺️ Initializing Mapbox...');
    console.log('Container:', containerId);
    console.log('Dark mode:', isDark);
    
    // Clean up existing map instance if any
    if (map) {
        console.log('🧹 Cleaning up existing map instance...');
        try {
            map.remove();
            map = null;
            selectedPackageIndex = null;
        } catch (e) {
            console.log('⚠️ Error removing existing map:', e);
            map = null;
        }
    }
    
    // Check if mapboxgl is available
    if (typeof mapboxgl === 'undefined') {
        console.error('❌ Mapbox GL JS not loaded');
        return;
    }
    
    // Check if container exists
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
        
        // Add navigation controls
        map.addControl(new mapboxgl.NavigationControl(), 'top-right');
        
        // Add geolocate control
        map.addControl(new mapboxgl.GeolocateControl({
            positionOptions: {
                enableHighAccuracy: true
            },
            trackUserLocation: true,
            showUserHeading: true
        }), 'top-right');
        
        // Add packages when map loads
        map.on('load', () => {
            console.log('✅ Map loaded, adding packages...');
            addPackagesToMap();
        });
        
        map.on('error', (e) => {
            console.error('❌ Map error:', e);
        });
        
        map.on('styledata', () => {
            console.log('✅ Map style loaded');
        });
        
        // Listen for theme changes
        if (window.matchMedia) {
            window.matchMedia('(prefers-color-scheme: dark)').addEventListener('change', (e) => {
                const newStyle = e.matches ? 'mapbox://styles/mapbox/dark-v11' : 'mapbox://styles/mapbox/light-v11';
                console.log('🎨 Changing map theme:', e.matches ? 'Dark' : 'Light');
                if (map) {
                    map.setStyle(newStyle);
                    // Re-add packages after style change
                    map.once('styledata', () => {
                        addPackagesToMap();
                    });
                }
            });
        }
        
        // Listen for window resize to ensure map is properly sized
        window.addEventListener('resize', () => {
            if (map) {
                setTimeout(() => {
                    map.resize();
                    console.log('🔄 Map resized');
                }, 100);
            }
        });
        
        // Force resize after a short delay to handle initial render issues
        setTimeout(() => {
            if (map) {
                map.resize();
                console.log('🔄 Map initial resize');
            }
        }, 500);
        
        // Additional resize after DOM is fully loaded
        if (document.readyState === 'loading') {
            document.addEventListener('DOMContentLoaded', () => {
                setTimeout(() => {
                    if (map) {
                        map.resize();
                        console.log('🔄 Map DOM loaded resize');
                    }
                }, 200);
            });
        } else {
            // DOM already loaded
            setTimeout(() => {
                if (map) {
                    map.resize();
                    console.log('🔄 Map DOM ready resize');
                }
            }, 200);
        }
    } catch (error) {
        console.error('❌ Error initializing map:', error);
    }
    };
    
    // Start initialization
    initMap();
};

// Function to reinitialize map if it fails to load
window.reinitializeMap = function() {
    console.log('🔄 Reinitializing map...');
    if (map) {
        try {
            map.remove();
            map = null;
        } catch (e) {
            console.log('Map already removed or not initialized');
        }
    }
    
    // Wait a bit then reinitialize
    setTimeout(() => {
        const container = document.getElementById('map');
        if (container) {
            const isDark = window.matchMedia && window.matchMedia('(prefers-color-scheme: dark)').matches;
            window.initMapbox('map', isDark);
        }
    }, 1000);
};

// Add packages to map as Style Layers
window.addPackagesToMap = function(packagesJson) {
    if (!map) {
        console.error('❌ Map not initialized');
        return;
    }
    
    try {
        // Use provided packages or fall back to window.currentPackages
        let packages;
        if (packagesJson) {
            packages = JSON.parse(packagesJson);
        } else {
            packages = window.currentPackages || [];
        }
        console.log('📦 Adding packages to map:', packages.length);
        
        // Wait for style to load before adding layers
        if (!map.isStyleLoaded()) {
            console.log('⏳ Waiting for map style to load...');
            map.once('style.load', () => {
                window.addPackagesToMap(packagesJson);
            });
            return;
        }
        
        // Remove existing source and layers
        if (map.getSource('packages')) {
            map.removeLayer('packages-circles');
            map.removeLayer('packages-labels');
            map.removeSource('packages');
        }
        
        // Create GeoJSON data from packages
        const geojsonData = {
            type: 'FeatureCollection',
            features: packages.map((pkg, index) => {
                // Skip packages without valid coordinates
                if (!pkg.coords || !Array.isArray(pkg.coords) || pkg.coords.length !== 2) {
                    return null;
                }
                
                return {
                    type: 'Feature',
                    geometry: {
                        type: 'Point',
                        coordinates: pkg.coords
                    },
                    properties: {
                        id: pkg.id,
                        index: index,
                        status: pkg.status,
                        code_statut_article: pkg.code_statut_article || null,
                        type_livraison: pkg.type_livraison || 'DOMICILE',
                        recipient: pkg.recipient,
                        address: pkg.address,
                        isSelected: selectedPackageIndex === index,
                        is_problematic: pkg.is_problematic || false
                    }
                };
            }).filter(feature => feature !== null)
        };
        
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
        map.addLayer({
            id: 'packages-labels',
            type: 'symbol',
            source: 'packages',
            layout: {
                'text-field': ['to-string', ['+', ['get', 'index'], 1]],
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
            const index = e.features[0].properties.index;
            console.log('📍 Package clicked on map:', index);
            
            // Trigger custom event that Yew can listen to
            const event = new CustomEvent('packageSelected', { detail: { index } });
            window.dispatchEvent(event);
        });
        
        // Change cursor on hover
        map.on('mouseenter', 'packages-circles', () => {
            map.getCanvas().style.cursor = 'pointer';
        });
        
        map.on('mouseleave', 'packages-circles', () => {
            map.getCanvas().style.cursor = '';
        });
        
        console.log(`✅ ${geojsonData.features.length} packages added as Style Layers`);
        
    } catch (error) {
        console.error('❌ Error adding packages to map:', error);
    }
};

// Update selected package
window.updateSelectedPackage = function(selectedIndex) {
    if (!map || !map.getSource('packages')) {
        return;
    }
    
    selectedPackageIndex = selectedIndex;
    
    // Create new GeoJSON data with updated selection
    const source = map.getSource('packages');
    if (source && source._data) {
        const geojsonData = {
            type: 'FeatureCollection',
            features: source._data.features.map(feature => ({
                ...feature,
                properties: {
                    ...feature.properties,
                    isSelected: feature.properties.index === selectedIndex
                }
            }))
        };
        
        // Update the source data
        source.setData(geojsonData);
        console.log(`✅ Package ${selectedIndex} updated as selected`);
        
        // Start pulse animation for selected package
        startPulseAnimation();
    }
};

// Pulse animation for selected package
function startPulseAnimation() {
    // Clear existing animation
    if (pulseAnimationId) {
        cancelAnimationFrame(pulseAnimationId);
    }
    
    let phase = 0;
    
    function animate() {
        if (!map || selectedPackageIndex === null) {
            return;
        }
        
        phase += 0.05; // Speed of animation
        
        // Calculate pulsating stroke width (between 2 and 5)
        const strokeWidth = 3 + Math.sin(phase) * 1.5;
        
        // Calculate pulsating opacity (between 0.4 and 0.8)
        const strokeOpacity = 0.6 + Math.sin(phase) * 0.2;
        
        // Update paint properties
        map.setPaintProperty('packages-circles', 'circle-stroke-width', [
            'case',
            ['get', 'isSelected'], strokeWidth, // Selected: pulsating
            ['get', 'is_problematic'], 1.5, // Problemático: borde visible
            0 // Sin borde para el resto
        ]);
        
        map.setPaintProperty('packages-circles', 'circle-stroke-opacity', [
            'case',
            ['get', 'isSelected'], strokeOpacity, // Selected: pulsating opacity
            1 // Default: opaco
        ]);
        
        pulseAnimationId = requestAnimationFrame(animate);
    }
    
    animate();
}

// Center map on package
window.centerMapOnPackage = function(index) {
    // Get package from window (will be set by Yew)
    const packages = window.currentPackages || [];
    const pkg = packages[index];
    
    if (pkg && pkg.coords && Array.isArray(pkg.coords) && pkg.coords.length === 2) {
        console.log(`🗺️ Centering map on package ${index}:`, pkg.coords);
        
        if (map) {
            map.flyTo({
                center: pkg.coords,
                zoom: 15,
                duration: 1000,
                essential: true
            });
        }
    } else {
        console.log(`⚠️ Package ${index} has no valid coords`);
    }
};

// Scroll to selected package in bottom sheet
window.scrollToSelectedPackage = function(index) {
    const packageCards = document.querySelectorAll('.package-card');
    const selectedCard = packageCards[index];
    
    if (selectedCard) {
        console.log(`📜 Scrolling to package ${index} in bottom sheet`);
        
        selectedCard.scrollIntoView({
            behavior: 'smooth',
            block: 'center'
        });
        
        // Add flash animation
        selectedCard.style.animation = 'none';
        setTimeout(() => {
            selectedCard.style.animation = 'flash 0.8s ease';
        }, 100);
    }
};

// Get map instance
window.getMapInstance = function() {
    return map;
};

// Update package coordinates on the map
window.updatePackageCoordinates = function(packageId, latitude, longitude) {
    if (!map) {
        console.error('❌ Map not initialized');
        return false;
    }
    
    const source = map.getSource('packages');
    if (!source) {
        console.error('❌ Packages source not found');
        return false;
    }
    
    // Get current data
    const data = source._data;
    if (!data || !data.features) {
        console.error('❌ No package data found');
        return false;
    }
    
    // Find and update the package
    const feature = data.features.find(f => f.properties.id === packageId);
    if (!feature) {
        console.error('❌ Package not found:', packageId);
        return false;
    }
    
    // Update coordinates [lng, lat] format for GeoJSON
    feature.geometry.coordinates = [longitude, latitude];
    
    // Update the source
    source.setData(data);
    
    // Fly to the new location
    map.flyTo({
        center: [longitude, latitude],
        zoom: 15,
        duration: 1500
    });
    
    console.log('✅ Package coordinates updated:', packageId, 'to', [latitude, longitude]);
    return true;
};

// Add single package to map (when geocoded from problematic)
window.addPackageToMap = function(packageId, latitude, longitude, address, code_statut_article) {
    if (!map) {
        console.error('❌ Map not initialized');
        return false;
    }
    
    const source = map.getSource('packages');
    if (!source) {
        console.error('❌ Packages source not found');
        return false;
    }
    
    const data = source._data;
    if (!data || !data.features) {
        console.error('❌ No package data found');
        return false;
    }
    
    // Check if package already exists
    const existingIndex = data.features.findIndex(f => f.properties.id === packageId);
    if (existingIndex !== -1) {
        // Update existing package
        data.features[existingIndex].geometry.coordinates = [longitude, latitude];
        data.features[existingIndex].properties.address = address;
        data.features[existingIndex].properties.code_statut_article = code_statut_article || null;
        console.log('🔄 Package updated on map:', packageId);
    } else {
        // Add new package
        const newFeature = {
            type: 'Feature',
            geometry: {
                type: 'Point',
                coordinates: [longitude, latitude]
            },
            properties: {
                id: packageId,
                address: address,
                code_statut_article: code_statut_article || null
            }
        };
        data.features.push(newFeature);
        console.log('➕ Package added to map:', packageId);
    }
    
    // Update the source
    source.setData(data);
    
    // Fly to the package
    map.flyTo({
        center: [longitude, latitude],
        zoom: 15,
        duration: 1500
    });
    
    return true;
};

// Remove package from map
window.removePackageFromMap = function(packageId) {
    if (!map) {
        console.error('❌ Map not initialized');
        return false;
    }
    
    const source = map.getSource('packages');
    if (!source) {
        console.error('❌ Packages source not found');
        return false;
    }
    
    // Get current data
    const data = source._data;
    if (!data || !data.features) {
        console.error('❌ No package data found');
        return false;
    }
    
    // Find and remove the package
    const featureIndex = data.features.findIndex(f => f.properties.id === packageId);
    if (featureIndex === -1) {
        console.error('❌ Package not found:', packageId);
        return false;
    }
    
    // Remove the feature
    data.features.splice(featureIndex, 1);
    
    // Update the source
    source.setData(data);
    
    console.log('🗑️ Package removed from map:', packageId);
    return true;
};

console.log('📍 Mapbox helper functions loaded');

