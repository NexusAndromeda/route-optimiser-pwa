# 🚀 Route Optimizer - Frontend (Yew)

PWA construida con Yew (Rust → WebAssembly)

## 🏗️ Stack Tecnológico

- **Framework:** Yew 0.21
- **Lenguaje:** Rust (compilado a WASM)
- **Estilos:** CSS puro (del prototipo)
- **PWA:** Service Worker + Manifest
- **Maps:** Mapbox GL JS (por integrar)

## 🚀 Desarrollo

### Instalar Trunk (build tool para Yew)
```bash
cargo install trunk wasm-bindgen-cli
```

### Ejecutar en desarrollo
```bash
trunk serve
```

Abre: http://localhost:8080

### Build para producción
```bash
trunk build --release
```

## 📁 Estructura

```
app/
├── src/
│   ├── main.rs              # Entry point
│   ├── models.rs            # Data models
│   └── components/          # UI Components
│       ├── app.rs           # Main app
│       ├── header.rs        # Header
│       ├── map.rs           # Map container
│       ├── package_list.rs  # Lista de paquetes
│       ├── package_card.rs  # Card de paquete
│       ├── details_modal.rs # Modal de detalles
│       ├── bal_modal.rs     # Modal BAL (Sí/No)
│       └── settings_popup.rs # Popup de configuración
├── assets/
│   └── style.css            # CSS del prototipo
├── index.html               # HTML template
├── manifest.json            # PWA manifest
└── Cargo.toml
```

## ✨ Funcionalidades

### ✅ Implementado (Réplica del prototipo):
- Header con botón de configuración
- Mapa placeholder
- Bottom Sheet responsive (3 estados)
- Sidebar desktop (fija 320px)
- Cards de paquetes con número
- Selección de paquetes
- Botones de reorden (solo en seleccionado)
- Modal de detalles
- Modal BAL (Sí/No)
- Popup de configuración
- Dark/Light mode automático

### ⏳ Por implementar:
- Integración Mapbox
- API calls al backend
- Service Worker (offline)
- Lógica de reordenamiento real
- Persistencia de datos

## 🎨 Diseño

El diseño es una réplica EXACTA del prototipo HTML/CSS/JS ubicado en `prototype/`

## 🔧 Configuración

Crea `.env` con:
```
MAPBOX_TOKEN=tu_token_aqui
API_BASE_URL=http://localhost:3000
```

## 📱 PWA

La app se instala como nativa en iOS y Android gracias al `manifest.json`

---

*Migrado de Dioxus a Yew para mejor productividad y estabilidad*
