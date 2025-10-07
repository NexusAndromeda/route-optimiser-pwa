# Route Optimizer App

Frontend multiplataforma construido con Dioxus (Rust).

## 🎯 Plataformas soportadas

- 🌐 **Web (PWA)** - Progressive Web App
- 🤖 **Android** - App nativa (API 29+)
- 🍎 **iOS** - App nativa (iOS 14+)

## 🚀 Quick Start

### Prerequisitos

```bash
# Instalar Dioxus CLI
cargo install dioxus-cli

# Verificar instalación
dx --version
```

### Desarrollo

#### Web (más rápido para desarrollo)
```bash
dx serve --web
# Abre http://localhost:8080
```

#### Android
```bash
# Con emulador corriendo
dx serve --android
```

#### iOS (solo macOS)
```bash
# Con simulador corriendo
dx serve --ios
```

## 📦 Build para producción

```bash
# Web
dx build --web --release
# Output: dist/

# Android (APK)
dx build --android --release

# iOS
dx build --ios --release

# Build específico
dx build --release
```

## 🏗️ Estructura del proyecto

```
app/
├── src/
│   ├── main.rs              # Entry point
│   ├── components/          # Componentes UI (próximamente)
│   ├── models/              # Modelos de datos (próximamente)
│   └── services/            # HTTP client, etc (próximamente)
├── assets/                  # Imágenes, iconos (próximamente)
├── platforms/
│   ├── android/            # Generado por Dioxus
│   └── ios/                # Generado por Dioxus
├── Cargo.toml
├── Dioxus.toml
└── README.md
```

## 🔧 Configuración

### Backend URL

El backend está en: `http://localhost:8000` (desarrollo)

Para producción, actualizar en `src/config.rs` (próximamente)

## 📱 Renderer

Usando **Skia renderer** para tener UI idéntica en todas las plataformas.

## 🐛 Debugging

### Web
```bash
# Chrome DevTools funciona perfectamente
dx serve
# F12 en navegador
```

### Android
```bash
# Terminal 1
dx serve --platform android

# Terminal 2
adb logcat | grep RouteOptimizer
```

### iOS
```bash
# Ver logs en Xcode Console
# Window → Devices and Simulators → Open Console
```

## 📝 Notas

- Este es un proyecto de prueba para evaluar Dioxus
- Comparar con implementaciones actuales en Swift (iOS) y Kotlin (Android)
- Evaluar viabilidad de migración completa

