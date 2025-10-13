// Register Service Worker for PWA
if ('serviceWorker' in navigator) {
    window.addEventListener('load', () => {
        // Intentar registrar SW, pero no es crítico si falla (en desarrollo)
        navigator.serviceWorker.register('/sw.js')
            .then(registration => {
                console.log('✅ Service Worker registered:', registration);
            })
            .catch(error => {
                // No es crítico en desarrollo, solo mostrar warning
                console.warn('⚠️ Service Worker not available (normal en trunk serve):', error.message);
            });
    });
}

