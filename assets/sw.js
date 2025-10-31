// ============================================
// SERVICE WORKER - PWA OFFLINE COMPLETA
// ============================================

const CACHE_VERSION = 'v3';
const CACHE_NAME = `delivery-app-${CACHE_VERSION}`;

// Assets a cachear (generados por Trunk)
const ASSETS_TO_CACHE = [
    '/',
    '/index.html',
    '/style.css',
    '/manifest.json',
];

// ============================================
// INSTALL - Cachear assets iniciales
// ============================================
self.addEventListener('install', (event) => {
    console.log('[SW] Installing Service Worker v' + CACHE_VERSION);
    
    event.waitUntil(
        caches.open(CACHE_NAME)
            .then((cache) => {
                console.log('[SW] Caching app shell');
                return cache.addAll(ASSETS_TO_CACHE);
            })
            .then(() => {
                console.log('[SW] Installation complete');
                return self.skipWaiting(); // Activar inmediatamente
            })
            .catch((error) => {
                console.error('[SW] Installation failed:', error);
            })
    );
});

// ============================================
// ACTIVATE - Limpiar cachés antiguos
// ============================================
self.addEventListener('activate', (event) => {
    console.log('[SW] Activating Service Worker v' + CACHE_VERSION);
    
    event.waitUntil(
        caches.keys()
            .then((cacheNames) => {
                return Promise.all(
                    cacheNames.map((cacheName) => {
                        if (cacheName !== CACHE_NAME) {
                            console.log('[SW] Deleting old cache:', cacheName);
                            return caches.delete(cacheName);
                        }
                    })
                );
            })
            .then(() => {
                console.log('[SW] Activation complete');
                return self.clients.claim(); // Tomar control de todas las pestañas
            })
    );
});

// ============================================
// FETCH - Interceptar peticiones
// ============================================
self.addEventListener('fetch', (event) => {
    const { request } = event;
    const url = new URL(request.url);
    
    // Ignorar peticiones de otros orígenes (excepto API backend)
    if (url.origin !== location.origin && !url.hostname.includes('localhost') && !url.hostname.includes('api.delivery.nexuslabs.one')) {
        return;
    }
    
    // Determinar estrategia según tipo de recurso
    if (isApiRequest(url)) {
        // API: Network-First (intentar red, fallback a caché)
        event.respondWith(networkFirstStrategy(request));
    } else if (isAssetRequest(url)) {
        // Assets: Cache-First (caché primero, fallback a red)
        event.respondWith(cacheFirstStrategy(request));
    } else {
        // Otros: Network-First
        event.respondWith(networkFirstStrategy(request));
    }
});

// ============================================
// BACKGROUND SYNC - Sincronización en background
// ============================================
self.addEventListener('sync', (event) => {
    console.log('[SW] Background sync triggered:', event.tag);
    
    if (event.tag === 'sync-pending-changes') {
        event.waitUntil(syncPendingChanges());
    }
});

// ============================================
// PUSH NOTIFICATIONS - Recibir notificaciones
// ============================================
self.addEventListener('push', (event) => {
    console.log('[SW] Push notification received');
    
    const data = event.data ? event.data.json() : {};
    const title = data.title || 'Delivery App';
    const options = {
        body: data.body || 'Nueva actualización disponible',
        icon: '/icon-192.png',
        badge: '/icon-72.png',
        tag: data.tag || 'default',
        data: data,
    };
    
    event.waitUntil(
        self.registration.showNotification(title, options)
    );
});

// ============================================
// NOTIFICATION CLICK - Manejar clicks
// ============================================
self.addEventListener('notificationclick', (event) => {
    console.log('[SW] Notification clicked');
    event.notification.close();
    
    event.waitUntil(
        clients.openWindow('/') // Abrir la app
    );
});

// ============================================
// ESTRATEGIAS DE CACHÉ
// ============================================

/**
 * Cache-First: Buscar en caché primero, si no existe ir a red
 */
async function cacheFirstStrategy(request) {
    try {
        const cachedResponse = await caches.match(request);
        if (cachedResponse) {
            console.log('[SW] Cache hit:', request.url);
            return cachedResponse;
        }
        
        console.log('[SW] Cache miss, fetching:', request.url);
        const networkResponse = await fetch(request);
        
        // Cachear la respuesta para futuras peticiones
        if (networkResponse.ok) {
            const cache = await caches.open(CACHE_NAME);
            cache.put(request, networkResponse.clone());
        }
        
        return networkResponse;
    } catch (error) {
        console.error('[SW] Cache-first strategy failed:', error);
        
        // Fallback: intentar devolver página offline
        const cache = await caches.open(CACHE_NAME);
        return cache.match('/index.html');
    }
}

/**
 * Network-First: Intentar red primero, fallback a caché
 */
async function networkFirstStrategy(request) {
    try {
        console.log('[SW] Fetching from network:', request.url);
        const networkResponse = await fetch(request);
        
        // Cachear respuestas exitosas
        if (networkResponse.ok && request.method === 'GET') {
            const cache = await caches.open(CACHE_NAME);
            cache.put(request, networkResponse.clone());
        }
        
        return networkResponse;
    } catch (error) {
        console.log('[SW] Network failed, trying cache:', request.url);
        
        // Fallback a caché
        const cachedResponse = await caches.match(request);
        if (cachedResponse) {
            return cachedResponse;
        }
        
        // Si tampoco hay caché, devolver error offline
        return new Response('Offline', {
            status: 503,
            statusText: 'Service Unavailable',
            headers: new Headers({
                'Content-Type': 'text/plain'
            })
        });
    }
}

/**
 * Sincronizar cambios pendientes en background
 */
async function syncPendingChanges() {
    console.log('[SW] Syncing pending changes...');
    
    try {
        // Obtener localStorage (no disponible directamente en SW)
        // Necesitamos usar postMessage para comunicarnos con la página
        
        const clients = await self.clients.matchAll();
        if (clients.length > 0) {
            clients[0].postMessage({
                type: 'SYNC_REQUESTED',
                timestamp: Date.now()
            });
        }
        
        return Promise.resolve();
    } catch (error) {
        console.error('[SW] Sync failed:', error);
        return Promise.reject(error);
    }
}

// ============================================
// UTILIDADES
// ============================================

function isApiRequest(url) {
    // Detectar si es una llamada a la API
    return url.pathname.startsWith('/api/') || 
           url.pathname.startsWith('/session/') ||
           url.hostname === 'api.delivery.nexuslabs.one' ||
           (url.hostname === 'localhost' && url.port === '8000');
}

function isAssetRequest(url) {
    // Detectar si es un asset estático
    const assetExtensions = ['.js', '.wasm', '.css', '.html', '.png', '.jpg', '.svg', '.json'];
    return assetExtensions.some(ext => url.pathname.endsWith(ext));
}

// ============================================
// LOGGING
// ============================================
console.log('[SW] Service Worker loaded - Version:', CACHE_VERSION);
