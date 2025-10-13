// Mapbox initialization and helper functions
let map = null;
let markers = [];

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
    
    // Set token directly (now secured with URL restrictions)
    const token = 'pk.eyJ1IjoiZGFuaWVsaG5jdCIsImEiOiJjbWdwNHVva3oyMmR5MmtzZzBuMzFmbWh2In0.1GtUgUs47OpLePKt3gH4dg'; // TODO: Move to env variable
    
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
        
        map.on('load', () => {
            console.log('✅ Map loaded');
        });
        
        map.on('error', (e) => {
            console.error('❌ Map error:', e);
        });
        
        // Listen for theme changes
        if (window.matchMedia) {
            window.matchMedia('(prefers-color-scheme: dark)').addEventListener('change', (e) => {
                const newStyle = e.matches ? 'mapbox://styles/mapbox/dark-v11' : 'mapbox://styles/mapbox/light-v11';
                console.log('🎨 Theme changed:', e.matches ? 'Dark' : 'Light');
                if (map) {
                    map.setStyle(newStyle);
                }
            });
        }
    } catch (error) {
        console.error('❌ Error initializing map:', error);
    }
};

// Add a marker to the map
window.addMapMarker = function(index, lat, lng, isDelivered) {
    if (!map) {
        console.error('❌ Map not initialized');
        return;
    }
    
    // Create marker element
    const el = document.createElement('div');
    el.className = 'map-marker';
    el.style.backgroundColor = isDelivered ? '#10B981' : '#3B82F6';
    el.style.width = '30px';
    el.style.height = '30px';
    el.style.borderRadius = '50%';
    el.style.border = '3px solid white';
    el.style.boxShadow = '0 2px 4px rgba(0,0,0,0.3)';
    el.style.cursor = 'pointer';
    el.style.display = 'flex';
    el.style.alignItems = 'center';
    el.style.justifyContent = 'center';
    el.style.color = 'white';
    el.style.fontWeight = 'bold';
    el.style.fontSize = '14px';
    el.textContent = index;
    
    // Add click handler
    el.addEventListener('click', () => {
        console.log('📍 Marker clicked:', index);
        // TODO: Select package when marker is clicked
    });
    
    // Create and add marker
    const marker = new mapboxgl.Marker(el)
        .setLngLat([lng, lat])
        .addTo(map);
    
    markers.push(marker);
    
    console.log(`✅ Marker ${index} added at [${lat}, ${lng}]`);
};

// Clear all markers
window.clearMapMarkers = function() {
    markers.forEach(marker => marker.remove());
    markers = [];
    console.log('🧹 All markers cleared');
};

// Get map instance
window.getMapInstance = function() {
    return map;
};

console.log('📍 Mapbox helper functions loaded');

