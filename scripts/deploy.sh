#!/bin/bash

# Script de despliegue para Route Optimizer PWA (NUEVA VERSIÓN)
# Uso: ./deploy.sh

set -e  # Salir si hay algún error

echo "🚀 Iniciando despliegue de Route Optimizer (app-new)..."
echo ""

# Colores
GREEN='\033[0;32m'
BLUE='\033[0;34m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# Directorio del proyecto
PROJECT_ROOT="/Users/nexus/projects/route-optimizer"
APP_DIR="${PROJECT_ROOT}/app"

# 1. Build del frontend (NUEVA VERSIÓN)
echo -e "${BLUE}📦 Compilando frontend (app)...${NC}"
cd "${APP_DIR}"

# Verificar que estamos en el directorio correcto
if [ ! -f "Trunk.toml" ]; then
    echo -e "${YELLOW}⚠️  Error: No se encontró Trunk.toml en ${APP_DIR}${NC}"
    exit 1
fi

# Compilar con trunk
echo -e "${BLUE}   → Configurando BACKEND_URL para producción...${NC}"
export BACKEND_URL="https://api.delivery.nexuslabs.one"
echo -e "${BLUE}   → Ejecutando trunk build --release...${NC}"
trunk build --release

# 2. Verificar que el build fue exitoso
if [ ! -d "dist" ]; then
    echo -e "${YELLOW}⚠️  Error: No se generó el directorio dist${NC}"
    exit 1
fi

# 3. Verificar archivos críticos
echo -e "${BLUE}📋 Verificando archivos críticos...${NC}"
if [ ! -f "dist/index.html" ]; then
    echo -e "${YELLOW}⚠️  Error: No se encontró dist/index.html${NC}"
    exit 1
fi

if [ ! -f "dist/sw.js" ]; then
    echo -e "${YELLOW}⚠️  Advertencia: sw.js no encontrado, copiando manualmente...${NC}"
    cp assets/sw.js dist/sw.js
fi

if [ ! -f "dist/manifest.json" ]; then
    echo -e "${YELLOW}⚠️  Advertencia: manifest.json no encontrado, copiando manualmente...${NC}"
    cp assets/manifest.json dist/manifest.json
fi

echo -e "${GREEN}✅ Build completado${NC}"
ls -lh dist/*.wasm dist/*.js | head -5

# 4. Desplegar a RPi
echo ""
echo -e "${BLUE}🌐 Desplegando a RPi (scorpius)...${NC}"
rsync -avz --delete dist/ scorpius:/var/www/html/route-optimizer/

# 5. Verificar despliegue
echo ""
echo -e "${BLUE}🔍 Verificando despliegue en servidor...${NC}"
ssh scorpius "ls -lh /var/www/html/route-optimizer/ | head -10"

echo ""
echo -e "${GREEN}✅ Despliegue completado exitosamente!${NC}"
echo ""
echo "🌍 Aplicación disponible en: https://delivery.nexuslabs.one"
echo "📱 PWA lista para instalar desde el navegador"
echo ""

