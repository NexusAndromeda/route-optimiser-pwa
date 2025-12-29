// ============================================
// SERVICE WORKER REGISTRATION
// ============================================

if ('serviceWorker' in navigator) {
    window.addEventListener('load', async () => {
        // En desarrollo: assets/sw.js, en producción: /sw.js (copiado a raíz por Makefile)
        const swPath = '/sw.js'; // Deploy copia a raíz
        const swPathDev = '/assets/sw.js'; // Desarrollo
        
        try {
            // Intentar primero desde la raíz (producción)
            const registration = await navigator.serviceWorker.register(swPath);
            handleRegistration(registration);
        } catch (error1) {
            // Si falla (desarrollo), intentar desde assets/
            console.log('🔄 Intentando cargar Service Worker desde assets/...');
            try {
                const registration = await navigator.serviceWorker.register(swPathDev);
                handleRegistration(registration);
            } catch (error2) {
                console.error('❌ Error registrando Service Worker:', error2);
            }
        }
        
        function handleRegistration(registration) {
                console.log('✅ Service Worker registrado:', registration.scope);
                
                // Escuchar actualizaciones
                registration.addEventListener('updatefound', () => {
                    const newWorker = registration.installing;
                    console.log('🔄 Nueva versión del Service Worker encontrada');
                    
                    newWorker.addEventListener('statechange', () => {
                        if (newWorker.state === 'installed' && navigator.serviceWorker.controller) {
                            // Nueva versión disponible
                            showUpdateNotification();
                        }
                    });
                });
                
                // Verificar actualizaciones cada 5 minutos
                setInterval(() => {
                    registration.update();
                }, 5 * 60 * 1000);
        }
        
        // Escuchar mensajes del Service Worker
        navigator.serviceWorker.addEventListener('message', (event) => {
            console.log('📨 Mensaje del SW:', event.data);
            
            if (event.data.type === 'SYNC_REQUESTED') {
                // Disparar sincronización cuando el SW lo solicite
                window.dispatchEvent(new CustomEvent('sw-sync-request'));
            }
        });
    });
}

// Notificar al usuario sobre actualización disponible
function showUpdateNotification() {
    const notification = document.createElement('div');
    notification.className = 'update-notification';
    notification.innerHTML = `
        <div class="update-content">
            <span>🔄 Nueva versión disponible</span>
            <button onclick="location.reload()">Actualizar</button>
        </div>
    `;
    document.body.appendChild(notification);
    
    // Auto-ocultar después de 10 segundos
    setTimeout(() => {
        notification.remove();
    }, 10000);
}
