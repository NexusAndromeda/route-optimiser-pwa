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

window.stopBarcodeScanner = function() {
    if (window.Quagga) {
        try {
            Quagga.stop();
            // Remove event listeners
            Quagga.offDetected();
            Quagga.offProcessed();
        } catch (e) {
            console.error("Error stopping Quagga:", e);
        }
    }
};

