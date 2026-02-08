# Makefile para Route Optimizer Frontend

.PHONY: dev build deploy deploy-local clean install-dev-tools

# Instalar herramientas de desarrollo (una vez)
install-dev-tools:
	@echo "📦 Instalando herramientas de desarrollo..."
	cargo install miniserve wasm-pack || true

# Desarrollo (compila una vez y sirve)
dev:
	@echo "🔨 Compilando WASM en modo desarrollo..."
	@echo "🌐 Usando BACKEND_URL=http://localhost:3000 (forzado para desarrollo)"
	@BACKEND_URL=http://localhost:3000 wasm-pack build --target web --dev
	@echo "✅ Build desarrollo completado"
	@echo "📋 Creando symlinks temporales para desarrollo..."
	@ln -sf assets/sw.js sw.js 2>/dev/null || cp assets/sw.js sw.js
	@ln -sf assets/icons/icon-192.png icon-192.png 2>/dev/null || cp assets/icons/icon-192.png icon-192.png
	@ln -sf assets/icons/icon-512.png icon-512.png 2>/dev/null || cp assets/icons/icon-512.png icon-512.png
	@echo "🚀 Iniciando servidor..."
	miniserve . --port 8080 --index index.html

# Build producción
build:
	@echo "🔨 Compilando WASM para producción..."
	@if [ -f .env ]; then \
		echo "📋 Cargando variables de entorno desde .env..."; \
		export $$(grep -v '^#' .env | xargs); \
		echo "🌐 Usando BACKEND_URL=$${BACKEND_URL:-https://api.delivery.nexuslabs.one}"; \
		BACKEND_URL=$${BACKEND_URL:-https://api.delivery.nexuslabs.one} wasm-pack build --target web --release; \
	else \
		echo "🌐 Usando BACKEND_URL=https://api.delivery.nexuslabs.one (default producción)"; \
		BACKEND_URL=https://api.delivery.nexuslabs.one wasm-pack build --target web --release; \
	fi
	@echo "✅ Build completado en pkg/"

# Deploy a Raspberry Pi (build + preparar + rsync)
deploy: build
	@echo "📦 Preparando dist/..."
	@mkdir -p dist
	@cp -r pkg dist/
	@cp -r assets dist/
	@cp index.html dist/
	@cp assets/sw.js dist/sw.js
	@cp assets/manifest.json dist/manifest.json
	@cp assets/icons/*.png dist/ 2>/dev/null || true
	@echo "📋 Verificando archivos críticos..."
	@test -f dist/index.html || (echo "❌ Error: dist/index.html no encontrado" && exit 1)
	@test -f dist/sw.js || (echo "❌ Error: dist/sw.js no encontrado" && exit 1)
	@test -f dist/manifest.json || (echo "❌ Error: dist/manifest.json no encontrado" && exit 1)
	@echo "✅ Archivos verificados"
	@echo "📊 Archivos WASM generados:"
	@ls -lh dist/pkg/*.wasm dist/pkg/*.js 2>/dev/null | head -5 || true
	@echo ""
	@echo "🌐 Desplegando a RPi (scorpius)..."
	@rsync -avz --delete dist/ scorpius:/var/www/html/route-optimizer/
	@echo ""
	@echo "🔍 Verificando despliegue en servidor..."
	@ssh scorpius "ls -lh /var/www/html/route-optimizer/ | head -10"
	@echo ""
	@echo "✅ Despliegue completado exitosamente!"
	@echo ""
	@echo "🌍 Aplicación disponible en: https://delivery.nexuslabs.one"
	@echo "📱 PWA lista para instalar desde el navegador"

# Deploy local (cuando el RPi es tanto dev como servidor - mismo build que deploy pero copia local)
# Uso: make deploy-local (desde el RPi, requiere sudo para /var/www/html/)
deploy-local: build
	@echo "📦 Preparando dist/..."
	@mkdir -p dist
	@cp -r pkg dist/
	@cp -r assets dist/
	@cp index.html dist/
	@cp assets/sw.js dist/sw.js
	@cp assets/manifest.json dist/manifest.json
	@cp assets/icons/*.png dist/ 2>/dev/null || true
	@echo "📋 Verificando archivos críticos..."
	@test -f dist/index.html || (echo "❌ Error: dist/index.html no encontrado" && exit 1)
	@test -f dist/sw.js || (echo "❌ Error: dist/sw.js no encontrado" && exit 1)
	@test -f dist/manifest.json || (echo "❌ Error: dist/manifest.json no encontrado" && exit 1)
	@echo "✅ Archivos verificados"
	@echo ""
	@echo "🏠 Desplegando localmente en /var/www/html/route-optimizer/..."
	@sudo rsync -av --delete dist/ /var/www/html/route-optimizer/
	@echo ""
	@echo "✅ Deploy local completado!"
	@echo "🌍 Aplicación disponible en: https://delivery.nexuslabs.one"

# Limpiar
clean:
	@echo "🧹 Limpiando..."
	rm -rf pkg dist target
	@rm -f sw.js icon-*.png  # Archivos temporales de desarrollo
	@echo "✅ Limpieza completada"

