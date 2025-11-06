# 🚚 Route Optimizer App - Frontend

Aplicación PWA (Progressive Web App) para optimización de rutas de entrega. Construida con Rust/WASM usando Yew framework.

## Propósito

El frontend proporciona:
- **Interfaz de usuario** para conductores de entrega
- **Mapa interactivo** con Mapbox GL JS
- **Gestión de paquetes** agrupados por dirección
- **Escáner de códigos de barras** para escaneo rápido
- **Modo offline** con sincronización automática
- **Optimización de rutas** visual en el mapa

## Requisitos

### Software
- **Rust**: 1.70+ (edition 2021)
- **Trunk**: 0.18+ (bundler WASM)
- **Node.js**: 18+ (opcional, solo para desarrollo de assets JS)
- **Navegador moderno**: Chrome 90+, Firefox 88+, Safari 14+ (con soporte WASM)

### Instalación de Trunk

```bash
# Instalar Trunk
cargo install trunk

# Verificar instalación
trunk --version
```

### Variables de Entorno

El frontend usa una URL de backend hardcoded en `src/utils/constants.rs`:

```rust
pub const BACKEND_URL: &str = "https://api.delivery.nexuslabs.one";
```

Para desarrollo local, modificar temporalmente a:
```rust
pub const BACKEND_URL: &str = "http://localhost:3000";
```

**Nota**: En el futuro, esto debería ser configurable vía variable de entorno o build-time.

## Instalación

```bash
# Clonar repositorio
cd app

# Las dependencias se gestionan automáticamente con Cargo
# No requiere npm/yarn

# Verificar que Trunk esté instalado
trunk --version
```

## Comandos

### Desarrollo

```bash
# Servidor de desarrollo (hot reload)
trunk serve

# Servidor en puerto específico
trunk serve --port 8080

# Servidor con dirección específica
trunk serve --address 0.0.0.0 --port 8080

# Con logs detallados
RUST_LOG=debug trunk serve
```

El servidor se inicia en `http://localhost:8080` por defecto.

### Build

```bash
# Build de producción (optimizado)
trunk build --release

# Build de desarrollo (más rápido, menos optimizado)
trunk build

# Los archivos se generan en: dist/
```

### Despliegue

```bash
# Usar script de despliegue
./scripts/deploy.sh

# O manualmente:
trunk build --release
rsync -avz --delete dist/ usuario@servidor:/var/www/html/route-optimizer/
```

## Estructura del Proyecto

```
app/
├── src/
│   ├── main.rs                 # Punto de entrada Yew
│   ├── components/             # SOLO vistas (sin lógica)
│   │   ├── details_modal.rs    # Modal de edición de direcciones
│   │   ├── draggable_package_list.rs  # Lista con drag & drop
│   │   ├── package_card.rs     # Card individual de paquete
│   │   ├── package_list.rs     # Lista de paquetes
│   │   ├── scanner.rs          # Escáner de códigos de barras
│   │   ├── settings_popup.rs   # Popup de configuración
│   │   └── sync_indicator.rs   # Indicador de sincronización
│   ├── viewmodels/             # Estado + Lógica UI
│   │   ├── session_viewmodel.rs
│   │   └── map_viewmodel.rs
│   ├── services/               # SOLO comunicación API
│   │   ├── api_client.rs       # Cliente HTTP
│   │   ├── sync_service.rs     # Sincronización
│   │   ├── offline_service.rs  # Persistencia offline
│   │   └── network_monitor.rs  # Detección de conexión
│   ├── stores/                 # State Management
│   │   ├── session_store.rs    # Estado de sesión
│   │   ├── auth_store.rs       # Estado de autenticación
│   │   └── sync_store.rs       # Estado de sincronización
│   ├── hooks/                  # Custom hooks
│   │   ├── use_session.rs
│   │   ├── use_auth.rs
│   │   └── use_sync_state.rs
│   ├── models/                 # Estructuras compartidas
│   │   ├── session.rs
│   │   ├── package.rs
│   │   └── address.rs
│   ├── views/                  # Vistas principales
│   │   ├── app.rs              # Vista principal
│   │   └── login.rs             # Vista de login
│   └── utils/                  # Utilidades
│       ├── constants.rs        # Constantes (BACKEND_URL)
│       ├── mapbox_ffi.rs       # FFI para Mapbox
│       └── barcode_ffi.rs      # FFI para QuaggaJS
├── assets/
│   ├── mapbox.js               # Integración Mapbox GL JS
│   ├── barcode_scanner.js      # Integración QuaggaJS
│   ├── sw.js                    # Service Worker
│   ├── sw-register.js          # Registro de SW
│   ├── manifest.json           # PWA manifest
│   └── styles/                 # CSS modular
│       ├── base/               # Variables, reset, typography
│       ├── components/         # Estilos de componentes
│       ├── layouts/            # Layouts (app, bottom-sheet)
│       └── utilities/          # Animaciones, helpers
├── dist/                       # Build output (generado)
├── index.html                  # HTML principal
├── Trunk.toml                  # Configuración Trunk
└── Cargo.toml
```

## Arquitectura MVVM

### Reglas Estrictas

1. ✅ **Components NUNCA acceden a Services** (solo ViewModels)
2. ✅ **ViewModels NUNCA acceden a localStorage** (solo Stores)
3. ✅ **Services NUNCA contienen estado** (stateless)
4. ✅ **Stores son el ÚNICO source of truth**

### Flujo de Datos

```
Usuario → Component → ViewModel → Store → Service → API Backend
                ↓
            localStorage/IndexedDB (persistencia)
```

## Componentes Principales

### MapView
- Integración con Mapbox GL JS vía FFI (`assets/mapbox.js`)
- Muestra paquetes como puntos en el mapa
- Interacción: click en punto → seleccionar paquete
- Sincronización con lista de paquetes (scroll automático)

### DraggablePackageList
- Lista de paquetes agrupados por dirección
- Drag & drop para reordenar manualmente
- Bottom sheet responsive (deslizable)
- Agrupamiento por calle (`use_grouped_packages`)

### DetailsModal
- Edición de direcciones:
  - `door_code`: Código de puerta
  - `has_mailbox_access`: Acceso a buzón
  - `driver_notes`: Notas del conductor
- Validación de campos
- Guardado optimista (UI actualiza antes de sync)

### Scanner
- Escáner de códigos de barras usando QuaggaJS
- FFI vía `assets/barcode_scanner.js`
- Busca paquete por tracking code
- Scroll automático al paquete encontrado

### SyncIndicator
- Muestra estado de sincronización:
  - ✅ Sincronizado
  - 🔄 Sincronizando
  - ⚠️ Error
  - 📴 Offline
- Contador de cambios pendientes

## State Management

**Nota**: Yewdux está comentado por compatibilidad con Rust 1.90. Actualmente se usa `use_state_handle` directamente.

### Stores

- **SessionStore**: Sesión actual, paquetes, direcciones
- **AuthStore**: Estado de autenticación, usuario logueado
- **SyncStore**: Estado de sincronización, cambios pendientes

### Persistencia

- **IndexedDB**: Placeholder (no completamente implementado)
- **localStorage**: Fallback actual (usado en producción)
- **Queue persistente**: Cambios pendientes de sincronización

## Modo Offline

### Comportamiento

1. **Cambios locales**: Se guardan en sesión local + queue persistente
2. **Reintentos automáticos**: Backoff exponencial (1s, 2s, 4s, 8s...)
3. **Auto-sync**: Cuando vuelve conexión (NetworkMonitor)
4. **Polling remoto**: Cada 30s verifica cambios remotos

### Queue Persistente

- Guardada en localStorage/IndexedDB
- Incluye: tipo de cambio, timestamp, datos
- Se limpia automáticamente tras sync exitoso
- Máximo de reintentos: configurable (default: 5)

Ver documentación: [OFFLINE_STRATEGY.md](../docs/OFFLINE_STRATEGY.md)

## PWA (Progressive Web App)

### Service Worker

- Cachea assets estáticos (HTML, CSS, JS, WASM)
- Versión de cache: `v3` (actualizar en `assets/sw.js`)
- Activación automática al cargar app

### Manifest

- Configurado en `assets/manifest.json`
- Instalable en dispositivos móviles
- Iconos y tema definidos

## Troubleshooting

### Error: "trunk: command not found"
- Instalar Trunk: `cargo install trunk`
- Verificar PATH: `which trunk`

### Error: "Failed to fetch" al llamar API
- Verificar `BACKEND_URL` en `src/utils/constants.rs`
- Verificar que backend esté corriendo
- Verificar CORS en backend

### Mapa no se muestra
- Verificar token Mapbox en `assets/mapbox.js`
- Verificar que Mapbox GL JS se cargue: `console.log(mapboxgl)`
- Verificar consola del navegador para errores

### Escáner no funciona
- Verificar permisos de cámara en navegador
- Verificar que QuaggaJS se cargue: `console.log(Quagga)`
- Verificar `assets/barcode_scanner.js` está incluido

### Sincronización falla
- Verificar queue persistente: `localStorage.getItem('pending_changes_queue')`
- Verificar logs: `RUST_LOG=debug trunk serve`
- Ver documentación: [TROUBLESHOOTING.md](../docs/TROUBLESHOOTING.md)

### Build falla
- Limpiar build anterior: `rm -rf dist/ target/`
- Verificar Rust version: `rustc --version` (debe ser 1.70+)
- Verificar Trunk version: `trunk --version` (debe ser 0.18+)

### WASM muy grande
- Usar build release: `trunk build --release`
- Verificar optimizaciones en `Cargo.toml`:
  ```toml
  [profile.release]
  opt-level = "z"
  lto = true
  codegen-units = 1
  panic = "abort"
  ```

## Desarrollo

### Agregar Nuevo Componente

1. Crear archivo en `src/components/`
2. Implementar `FunctionComponent` o `Component`
3. Agregar CSS en `assets/styles/components/`
4. Importar en `src/components/mod.rs`
5. Usar en vista correspondiente

### Agregar Nuevo Hook

1. Crear archivo en `src/hooks/`
2. Implementar función que retorna hook handle
3. Agregar en `src/hooks/mod.rs`
4. Usar en componentes/viewmodels

### Modificar Estilos

- **Base**: Variables, reset, typography → `assets/styles/base/`
- **Componentes**: Estilos específicos → `assets/styles/components/`
- **Layouts**: Layouts generales → `assets/styles/layouts/`
- **Utilidades**: Helpers, animaciones → `assets/styles/utilities/`

**Orden de carga**: Base → Utilities → Components → Layouts (ver `index.html`)

## Documentación Adicional

- **[FRONTEND_GUIDE.md](../docs/FRONTEND_GUIDE.md)**: Guía detallada del frontend
- **[OFFLINE_STRATEGY.md](../docs/OFFLINE_STRATEGY.md)**: Estrategia offline completa
- **[ARCHITECTURE.md](../docs/ARCHITECTURE.md)**: Arquitectura general
- **[TROUBLESHOOTING.md](../docs/TROUBLESHOOTING.md)**: Errores comunes y soluciones

## Notas Importantes

### Inconsistencias Conocidas

1. **IndexedDB**: Mencionado pero no completamente implementado. Actualmente usa localStorage.
2. **Yewdux**: Comentado por compatibilidad Rust 1.90. Usa `use_state_handle` directamente.
3. **BACKEND_URL**: Hardcoded en código. Debería ser configurable.

### Mejoras Futuras

- Migrar completamente a IndexedDB
- Implementar Yewdux cuando esté disponible para Rust 1.90+
- Hacer BACKEND_URL configurable vía build-time o runtime
- Agregar tests unitarios para componentes
- Implementar error boundaries

## Licencia

Propietario - Nexus Labs
