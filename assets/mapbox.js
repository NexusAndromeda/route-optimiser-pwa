// Mapbox initialization and helper functions
let map = null;
let selectedPackageIndex = null;

// Initialize Mapbox map
window.initMapbox = function(containerId, isDark) {
    console.log('🗺️ Initializing Mapbox...');
    console.log('Container:', containerId);
    console.log('Dark mode:', isDark);
    
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
    } catch (error) {
        console.error('❌ Error initializing map:', error);
    }
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
                        recipient: pkg.recipient,
                        address: pkg.address,
                        isSelected: selectedPackageIndex === index
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
                    ['==', ['get', 'status'], 'delivered'], 8, // Delivered: smaller
                    8 // Pending: smaller
                ],
                'circle-color': [
                    'case',
                    ['get', 'isSelected'], '#FFD700', // Selected: gold
                    ['==', ['get', 'status'], 'delivered'], '#10B981', // Delivered: green
                    '#3B82F6' // Pending: blue
                ],
                'circle-stroke-width': [
                    'case',
                    ['get', 'isSelected'], 2, // Selected: thick border
                    1.5 // Normal: thin border
                ],
                'circle-stroke-color': [
                    'case',
                    ['get', 'isSelected'], '#FF6B35', // Selected: orange border
                    '#FFFFFF' // Normal: white border
                ]
            }
        });
        
        // Add labels layer (package numbers)
        map.addLayer({
            id: 'packages-labels',
            type: 'symbol',
            source: 'packages',
            layout: {
                'text-field': ['get', 'index'],
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
    }
};

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

console.log('📍 Mapbox helper functions loaded');

