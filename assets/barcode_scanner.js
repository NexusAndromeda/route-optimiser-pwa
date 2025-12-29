// Barcode Scanner using QuaggaJS
window.initBarcodeScanner = function(containerId, onBarcodeDetected, onError) {
    const container = document.getElementById(containerId);
    if (!container) {
        console.error("Container not found:", containerId);
        if (onError) onError("Container not found");
        return;
    }
    
    // Check if Quagga is available
    if (!window.Quagga) {
        console.error("QuaggaJS not loaded");
        if (onError) onError("QuaggaJS not loaded");
        return;
    }
    
    const config = {
        inputStream: {
            name: "Live",
            type: "LiveStream",
            target: container,
            constraints: {
                width: 640,
                height: 480,
                facingMode: "environment" // Use back camera by default
            }
        },
        locator: {
            patchSize: "medium",
            halfSample: false
        },
        numOfWorkers: 4,
        decoder: {
            readers: [
                "code_128_reader",
                "ean_reader",
                "ean_8_reader",
                "code_39_reader",
                "code_39_vin_reader",
                "codabar_reader",
                "upc_reader",
                "upc_e_reader",
                "i2of5_reader"
            ]
        },
        locate: true
    };
    
    // Clear previous listeners
    Quagga.offDetected();
    Quagga.offProcessed();
    
    Quagga.onDetected(function(result) {
        const code = result.codeResult.code;
        console.log("Barcode detected:", code);
        
        // Stop the scanner
        Quagga.stop();
        
        // Notify the callback
        if (onBarcodeDetected) {
            onBarcodeDetected(code);
        }
    });
    
    Quagga.init(config, function(err) {
        if (err) {
            console.error("Error initializing Quagga:", err);
            if (onError) onError(err.toString());
            return;
        }
        console.log("Quagga initialized successfully");
        Quagga.start();
    });
};

// Versión con callback de ready para feedback visual
window.initBarcodeScannerWithReady = function(containerId, onBarcodeDetected, onError, onReady) {
    const container = document.getElementById(containerId);
    if (!container) {
        console.error("Container not found:", containerId);
        if (onError) onError("Container not found");
        return;
    }
    
    // Check if Quagga is available
    if (!window.Quagga) {
        console.error("QuaggaJS not loaded");
        if (onError) onError("QuaggaJS not loaded");
        return;
    }
    
    const config = {
        inputStream: {
            name: "Live",
            type: "LiveStream",
            target: container,
            constraints: {
                width: 640,
                height: 480,
                facingMode: "environment" // Use back camera by default
            }
        },
        locator: {
            patchSize: "medium",
            halfSample: false
        },
        numOfWorkers: 4,
        decoder: {
            readers: [
                "code_128_reader",
                "ean_reader",
                "ean_8_reader",
                "code_39_reader",
                "code_39_vin_reader",
                "codabar_reader",
                "upc_reader",
                "upc_e_reader",
                "i2of5_reader"
            ]
        },
        locate: true
    };
    
    // Clear previous listeners
    Quagga.offDetected();
    Quagga.offProcessed();
    
    // NO agregar Quagga.onProcessed - removido para evitar movimientos pulsantes
    
    Quagga.onDetected(function(result) {
        const code = result.codeResult.code;
        console.log("Barcode detected:", code);
        
        // NO detener el scanner aquí - Rust decidirá si detener o no
        // Solo notificar el callback
        if (onBarcodeDetected) {
            onBarcodeDetected(code);
        }
    });
    
    Quagga.init(config, function(err) {
        if (err) {
            console.error("Error initializing Quagga:", err);
            if (onError) onError(err.toString());
            return;
        }
        console.log("Quagga initialized successfully");
        Quagga.start();
        
        // Notificar que está listo y escaneando
        if (onReady) {
            onReady(true);
        }
    });
};

window.stopBarcodeScanner = function() {
    if (window.Quagga) {
        try {
            // Intentar detener Quagga solo si está inicializado
            // Usar try-catch para manejar el caso donde no está inicializado
            if (typeof Quagga.stop === 'function') {
            Quagga.stop();
            }
        } catch (e) {
            // Si falla, probablemente no está inicializado - esto es OK
            console.log("Quagga no estaba inicializado o ya estaba detenido:", e.message);
        }
        
        // Siempre limpiar listeners (es seguro incluso si no está inicializado)
        try {
            Quagga.offDetected();
            Quagga.offProcessed();
        } catch (e) {
            // Ignorar errores al limpiar listeners
            console.log("Error limpiando listeners (puede ser normal):", e.message);
        }
        
            // Limpiar error visual si existe
            const viewport = document.getElementById('scanner-viewport');
            if (viewport) {
                viewport.classList.remove('scanner-error');
        }
    }
};

// Función para mostrar error visual (borde rojo) sin detener escaneo
window.showScannerError = function() {
    const viewport = document.getElementById('scanner-viewport');
    if (viewport) {
        viewport.classList.add('scanner-error');
        // Remover error después de 2.5 segundos
        setTimeout(() => {
            viewport.classList.remove('scanner-error');
        }, 2500);
    }
};

// Función para ocultar error visual
window.hideScannerError = function() {
    const viewport = document.getElementById('scanner-viewport');
    if (viewport) {
        viewport.classList.remove('scanner-error');
    }
};

