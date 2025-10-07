# 🔧 Configuración de Variables de Entorno

## 📋 Descripción

La aplicación web de Route Optimizer utiliza variables de entorno para gestionar la configuración de manera segura, evitando hardcodear información sensible como tokens de API.

## 🚀 Configuración Inicial

### 1. Crear el archivo `.env`

Copia el archivo de ejemplo `.env.example` a `.env`:

```bash
cd app
cp .env.example .env
```

### 2. Configurar Variables

Edita el archivo `.env` con tus valores reales:

```env
# Backend Configuration
BACKEND_URL_DEVELOPMENT=http://192.168.1.9:3000
BACKEND_URL_PRODUCTION=https://api.delivery.nexuslabs.one
ENVIRONMENT=development

# App Settings
ENABLE_LOGGING=true
NETWORK_TIMEOUT_SECONDS=30
RETRY_ATTEMPTS=3

# Map Configuration
DEFAULT_MAP_CENTER_LAT=48.8566
DEFAULT_MAP_CENTER_LNG=2.3522
DEFAULT_MAP_ZOOM=12.0

# Package Configuration
MAX_PACKAGES_FOR_CLUSTERING=50
CLUSTER_THRESHOLD=20

# UI Configuration
MARKER_SIZE=30
CLUSTER_SIZE=40
ROUTE_LINE_WIDTH=4

# Mapbox Configuration (IMPORTANTE: Agrega tu token aquí)
MAPBOX_ACCESS_TOKEN=tu_token_de_mapbox_aqui

# API Keys (if needed)
API_KEY=tu_api_key_aqui
```

### 3. Obtener Token de Mapbox

1. Ve a [Mapbox](https://account.mapbox.com/)
2. Crea una cuenta o inicia sesión
3. Ve a la sección "Access tokens"
4. Copia tu token de acceso público
5. Pégalo en el archivo `.env` en la variable `MAPBOX_ACCESS_TOKEN`

## 🔒 Seguridad

### ⚠️ **IMPORTANTE:**

- **NUNCA** commitees el archivo `.env` al repositorio
- El archivo `.env` está incluido en `.gitignore`
- Solo commitea el archivo `.env.example` con valores de ejemplo
- No compartas tu token de Mapbox públicamente

### ✅ Archivos Seguros

- ✅ `.env.example` - Commitear (solo ejemplos)
- ❌ `.env` - NO commitear (valores reales)
- ❌ `.env.local` - NO commitear (valores locales)
- ❌ `*.env` - NO commitear (cualquier archivo .env)

## 🏗️ Compilación

Las variables de entorno se cargan en **tiempo de compilación** mediante el archivo `build.rs`:

```bash
# Desarrollo (web)
dx serve --web --port 8080

# Producción (web)
dx build --release --web

# Android
dx serve --android

# iOS
dx serve --ios
```

## 🔄 Cambio de Entorno

Para cambiar entre desarrollo y producción, modifica la variable `ENVIRONMENT`:

```env
# Desarrollo
ENVIRONMENT=development  # Usa BACKEND_URL_DEVELOPMENT

# Producción
ENVIRONMENT=production   # Usa BACKEND_URL_PRODUCTION
```

## 🐛 Troubleshooting

### El mapa no carga

1. Verifica que `MAPBOX_ACCESS_TOKEN` esté configurado en `.env`
2. Asegúrate de que el token sea válido
3. Revisa la consola del navegador para errores

### No se conecta al backend

1. Verifica que `BACKEND_URL_DEVELOPMENT` apunte a la IP correcta
2. Asegúrate de que el backend esté corriendo
3. Verifica que el puerto sea el correcto (3000)

### Variables no se cargan

1. Asegúrate de que el archivo `.env` exista en `app/`
2. Verifica que el formato sea correcto: `KEY=VALUE`
3. Recompila la aplicación: `cargo clean && dx serve --web`

## 📚 Estructura de Archivos

```
app/
├── .env                  # ❌ NO commitear - Valores reales
├── .env.example          # ✅ Commitear - Valores de ejemplo
├── .gitignore            # Ignora archivos sensibles
├── build.rs              # Carga variables de entorno
├── src/
│   ├── config.rs         # Configuración de la app
│   └── main.rs           # Punto de entrada
└── CONFIG_SETUP.md       # Esta documentación
```

## 🌐 URLs por Defecto

### Desarrollo
- Backend: `http://192.168.1.9:3000`
- Frontend: `http://localhost:8080`

### Producción
- Backend: `https://api.delivery.nexuslabs.one`
- Frontend: Según el hosting configurado

## 📖 Más Información

Para más detalles sobre la configuración, consulta:
- `app/src/config.rs` - Implementación de la configuración
- `app/build.rs` - Script de compilación
- `app/.env.example` - Ejemplo de configuración

