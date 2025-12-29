# Makefile para Route Optimizer Frontend

.PHONY: dev build deploy clean install-dev-tools

# Instalar herramientas de desarrollo (una vez)
install-dev-tools:
	@echo "📦 Instalando herramientas de desarrollo..."
	cargo install miniserve wasm-pack || true

# Desarrollo (compila una vez y sirve)
dev:
	@echo "🔨 Compilando WASM en modo desarrollo..."
	wasm-pack build --target web --dev
	@echo "📋 Creando symlinks temporales para desarrollo..."
	@ln -sf assets/sw.js sw.js 2>/dev/null || cp assets/sw.js sw.js
	@ln -sf assets/icons/icon-192.png icon-192.png 2>/dev/null || cp assets/icons/icon-192.png icon-192.png
	@ln -sf assets/icons/icon-512.png icon-512.png 2>/dev/null || cp assets/icons/icon-512.png icon-512.png
	@echo "🚀 Iniciando servidor..."
	miniserve . --port 8080 --index index.html

# Build producción
build:
	@echo "🔨 Compilando WASM para producción..."
	wasm-pack build --target web --release
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

# Limpiar
clean:
	@echo "🧹 Limpiando..."
	rm -rf pkg dist target
	@rm -f sw.js icon-*.png  # Archivos temporales de desarrollo
	@echo "✅ Limpieza completada"

