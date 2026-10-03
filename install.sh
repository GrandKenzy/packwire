#!/usr/bin/env bash
# ==========================================================
#              PACKWIRE POSIX INSTALLER (Linux & macOS)
# ==========================================================

set -e

# ANSI Color codes
CYAN='\033[0;36m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
RED='\033[0;31m'
NC='\033[0m' # No Color

echo ""
echo -e "${CYAN}==========================================================${NC}"
echo -e "${CYAN}       PACKWIRE INSTALLER (Linux & macOS / POSIX)         ${NC}"
echo -e "${CYAN}==========================================================${NC}"
echo ""

# 1. Detect Python 3.9+
PYTHON_CMD=""
for cmd in python3 python; do
    if command -v "$cmd" >/dev/null 2>&1; then
        ver=$("$cmd" -c 'import sys; print(f"{sys.version_info.major}.{sys.version_info.minor}")' 2>/dev/null || echo "0.0")
        major=$(echo "$ver" | cut -d. -f1)
        minor=$(echo "$ver" | cut -d. -f2)
        if [ "$major" -ge 3 ] && [ "$minor" -ge 9 ]; then
            PYTHON_CMD="$cmd"
            break
        fi
    fi
done

if [ -z "$PYTHON_CMD" ]; then
    echo -e "${RED}[-] Error: Python 3.9 o superior no fue encontrado.${NC}"
    echo -e "${YELLOW}[!] Por favor instala Python 3.9+ mediante tu gestor de paquetes (apt, pacman, brew, dnf).${NC}"
    exit 1
fi

PY_VERSION=$("$PYTHON_CMD" --version 2>&1)
echo -e "${GREEN}[+] Entorno Python detectado: $PY_VERSION ($($PYTHON_CMD -c 'import sys; print(sys.executable)'))${NC}"

# 2. Check if running from repository or remote
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"

if [ -f "$SCRIPT_DIR/packwire/__init__.py" ]; then
    echo -e "${CYAN}[*] Instalando dependencias de Packwire en modo editable...${NC}"
    "$PYTHON_CMD" -m pip install -e "$SCRIPT_DIR" --no-warn-script-location
    
    echo -e "${CYAN}[*] Configurando lanzador shims, variables PATH y accesos directos...${NC}"
    "$PYTHON_CMD" -m packwire setup "$@"
else
    echo -e "${CYAN}[*] Instalando paquete Packwire...${NC}"
    "$PYTHON_CMD" -m pip install packwire --no-warn-script-location
    "$PYTHON_CMD" -m packwire setup "$@"
fi

echo ""
echo -e "${GREEN}==========================================================${NC}"
echo -e "${GREEN}  ¡PACKWIRE HA SIDO INSTALADO Y CONFIGURADO CON ÉXITO!    ${NC}"
echo -e "${GREEN}==========================================================${NC}"
echo ""
echo -e "Para comenzar a usar Packwire:"
echo -e "  • Terminal : ${YELLOW}packwire --help${NC}"
echo -e "  • Catálogo : ${YELLOW}packwire available${NC}"
echo -e "  • Gráfico  : ${YELLOW}packwire ui${NC}"
echo ""
echo -e "${YELLOW}[!] Si tu terminal no reconoce 'packwire' de inmediato, recarga tu shell:${NC}"
echo -e "    source ~/.bashrc  (o source ~/.zshrc en macOS / zsh)"
echo ""
