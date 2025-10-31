# 🚚 Route Optimizer App - Frontend MVVM Estricto

Arquitectura MVVM estricta re-implementada según `ANALISIS_EXHAUSTIVO_NIVEL_2.md`.

## Estructura

```
src/
├── components/    # SOLO vistas (sin lógica)
├── viewmodels/    # Estado + Lógica UI
├── services/      # SOLO comunicación API
├── stores/        # State Management (Yewdux)
├── models/        # Estructuras compartidas
└── hooks/         # Custom hooks (acceso a stores)
```

## Reglas MVVM Estrictas

1. ✅ Components NUNCA acceden a Services (solo ViewModels)
2. ✅ ViewModels NUNCA acceden a localStorage (solo Stores)
3. ✅ Services NUNCA contienen estado (stateless)
4. ✅ Stores son el ÚNICO source of truth

## Características

- ✅ State Management centralizado (Yewdux)
- ✅ Separación estricta de capas
- ✅ Background Sync API
- ✅ IndexedDB (reemplaza localStorage)
- ✅ Optimistic UI

## Desarrollo

```bash
# Ejecutar con Trunk
trunk serve

# Build para producción
trunk build --release
```

