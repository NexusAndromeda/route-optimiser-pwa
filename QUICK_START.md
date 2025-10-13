# 🚀 Quick Start - Route Optimizer (Yew)

## 📦 Instalación (Solo primera vez)

### 1. Instalar Trunk (build tool para Yew)
```bash
cargo install trunk wasm-bindgen-cli
```

### 2. Agregar target WASM
```bash
rustup target add wasm32-unknown-unknown
```

## 🏃 Ejecutar la app

```bash
cd /Users/nexus/projects/route-optimizer/app
trunk serve
```

Luego abre: **http://localhost:8080**

## 🎨 Lo que verás

**Réplica EXACTA del prototipo:**
- ✅ Header con botón ⚙️
- ✅ Mapa placeholder
- ✅ Bottom Sheet (mobile) / Sidebar (desktop)
- ✅ Cards con selección
- ✅ Botones [↑] [↓] [Aller] [Détails] en seleccionado
- ✅ Modal de detalles
- ✅ Modal BAL (Sí/No)
- ✅ Dark/Light mode automático

## 🔧 Desarrollo

### Hot reload
Trunk tiene hot reload automático. Cambia el código y se recarga solo.

### Compilar para producción
```bash
trunk build --release
```

Output en: `dist/`

## ⚡ Velocidad

**Yew compila a WASM = Ultra rápido** 🚀

---

¡Todo listo hermano! Solo ejecuta `trunk serve` 🎯

