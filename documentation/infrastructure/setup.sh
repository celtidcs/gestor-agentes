#!/bin/bash
# setup.sh - Preparación del entorno de desarrollo de Gestor Agentes 3.0.0 en Linux
# Proyecto : Gestor Agentes 3.0.0 (la reescritura en Rust de Gestor Claude)
# Requisitos: distribución con bash, Rust (cargo), git y un escritorio gráfico (X11 o Wayland).
#
# Uso, desde la raíz del proyecto:
#   ./documentation/infrastructure/setup.sh                    # comprueba, pasa las puertas y genera el portable
#   ./documentation/infrastructure/setup.sh --sin-puertas      # omite las cuatro puertas
#   ./documentation/infrastructure/setup.sh --compilacion <carpeta>
#
# Idempotente: puede ejecutarse tantas veces como se quiera; nada se duplica. No toca la
# configuración de la 2.x en ~/.config/GestorClaude ni la carpeta datos/ del portable.

set -u

# Programas de agente que la aplicación abre en una terminal. Ninguno es obligatorio para
# compilar, pero sin al menos uno la aplicación no tiene nada que abrir: claude (Claude Code),
# codex (Codex CLI) y agy (Gemini con la CLI de Antigravity). Programator se fija en su perfil.
AGENTES_EN_TERMINAL="claude codex agy"

AMARILLO='\033[1;33m'; VERDE='\033[0;32m'; ROJO='\033[0;31m'; CIAN='\033[0;36m'; NC='\033[0m'

DIR_SCRIPT="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
RAIZ_PROYECTO="$(cd "$DIR_SCRIPT/../.." && pwd)"
SIN_PUERTAS=0
CARPETA_COMPILACION=""

while [ $# -gt 0 ]; do
    case "$1" in
        --sin-puertas) SIN_PUERTAS=1; shift ;;
        --compilacion)
            [ $# -ge 2 ] || { echo "--compilacion necesita una carpeta" >&2; exit 1; }
            CARPETA_COMPILACION="$2"; shift 2 ;;
        *) echo "opción desconocida: $1" >&2; exit 1 ;;
    esac
done

echo -e "\n${CIAN}=== Gestor Agentes: preparación del entorno ===${NC}\n"

# 1. Requisitos: cargo es obligatorio; cargo-audit, para la cuarta puerta; git, para la aplicación.
echo -e "${AMARILLO}[1/4] Verificando requisitos...${NC}"
if ! command -v cargo >/dev/null 2>&1; then
    echo -e "      ${ROJO}ERROR: no se encuentra 'cargo'. Instala Rust desde https://rustup.rs y vuelve a abrir la terminal.${NC}"
    exit 1
fi
echo -e "      ${VERDE}OK: $(cargo --version)${NC}"
if command -v cargo-audit >/dev/null 2>&1; then
    echo -e "      ${VERDE}OK: cargo-audit${NC}"
else
    echo -e "      ${AMARILLO}AVISO: falta cargo-audit, que la cuarta puerta necesita. Instálalo con: cargo install cargo-audit --locked${NC}"
fi
if command -v git >/dev/null 2>&1; then
    echo -e "      ${VERDE}OK: $(git --version)${NC}"
else
    echo -e "      ${AMARILLO}AVISO: git no está instalado; la aplicación no podrá crear repositorios en los proyectos.${NC}"
fi

# 2. Agentes disponibles: solo se informa, porque cada equipo puede usar unos u otros.
echo -e "${AMARILLO}[2/4] Buscando programas de agente...${NC}"
for agente in $AGENTES_EN_TERMINAL; do
    encontrado="$(command -v "$agente" || true)"
    [ -z "$encontrado" ] && [ -x "$HOME/.local/bin/$agente" ] && encontrado="$HOME/.local/bin/$agente"
    if [ -n "$encontrado" ]; then
        echo -e "      ${VERDE}OK: $agente en $encontrado${NC}"
    else
        echo -e "      ${AMARILLO}AVISO: $agente no está instalado; su botón de terminal no podrá abrirlo.${NC}"
    fi
done

# La aplicación de Antigravity se busca en las mismas rutas que usa la aplicación y, por último,
# en el PATH.
ANTIGRAVITY=""
for candidato in "$HOME/.local/bin/antigravity" /usr/local/bin/antigravity /opt/antigravity/bin/antigravity; do
    if [ -x "$candidato" ]; then ANTIGRAVITY="$candidato"; break; fi
done
[ -z "$ANTIGRAVITY" ] && ANTIGRAVITY="$(command -v antigravity || true)"
if [ -n "$ANTIGRAVITY" ]; then
    echo -e "      ${VERDE}OK: Antigravity en $ANTIGRAVITY${NC}"
else
    echo -e "      ${AMARILLO}AVISO: Antigravity no está instalado; el botón de Antigravity no podrá abrirlo.${NC}"
fi

# 3. Las cuatro puertas: formato, linter, pruebas y auditoría de dependencias.
if [ "$SIN_PUERTAS" -eq 1 ]; then
    echo -e "${AMARILLO}[3/4] Puertas omitidas a petición.${NC}"
else
    echo -e "${AMARILLO}[3/4] Pasando las cuatro puertas...${NC}"
    if ! bash "$DIR_SCRIPT/puertas-rust.sh" "$CARPETA_COMPILACION"; then
        echo -e "      ${ROJO}ERROR: alguna puerta ha fallado; revisa la salida anterior.${NC}"
        exit 1
    fi
fi

# 4. Paquete portable: ejecutable, carpeta datos/ y documentación.
echo -e "${AMARILLO}[4/4] Generando el paquete portable...${NC}"
if ! bash "$DIR_SCRIPT/build-portable.sh"; then
    echo -e "      ${ROJO}ERROR: la generación del portable ha fallado.${NC}"
    exit 1
fi

echo -e "\n${CIAN}=== Entorno preparado ===${NC}"
echo "Ejecutable: $RAIZ_PROYECTO/portable/gestor-agentes"
echo
echo "La configuración (lista de proyectos y perfiles) vive en $RAIZ_PROYECTO/portable/datos."
echo "La primera vez, si existe la de la 2.x en ~/.config/GestorClaude, la aplicación la copia allí."
exit 0
