#!/usr/bin/env bash
# build-portable.sh - Compila Gestor Agentes como paquete portable para Linux
# ---------------------------------------------------------------------------
# Proyecto : Gestor Agentes 3.0.0 (antes Gestor Claude)
# Propósito: producir una carpeta autosuficiente que se pueda copiar a otro
#            equipo con Linux y usar sin instalar el programa, con toda la
#            documentación que necesita el usuario. Es la pareja de
#            build-portable.ps1 y comparte con él portable-extras/.
#
# Contenido del paquete resultante:
#   gestor-agentes   Ejecutable único en Rust.
#   datos/           Configuración del usuario; se crea vacía.
#   LEEME.txt        Primeros pasos.
#   documentacion/   Manual de uso, resolución de problemas y referencia de
#                    funcionalidades.
#
# Las comprobaciones del paquete (barreras) están aparte, en prueba-portable.sh.
#
# Uso:
#   ./build-portable.sh                          # salida en <raíz>/portable
#   ./build-portable.sh --destino /media/usb/App
#   ./build-portable.sh --objetivo x86_64-unknown-linux-gnu
#   ./build-portable.sh --extras <carpeta>       # acompañamientos a medida
#
# Idempotente: cada ejecución regenera el paquete y conserva datos/, que son
# los datos del usuario y no un artefacto de la compilación.
# ---------------------------------------------------------------------------

set -euo pipefail

# --- Constantes ------------------------------------------------------------

# Paquete de cargo que produce el ejecutable (crates/interfaz/Cargo.toml).
readonly PAQUETE="gestor-agentes"

# Carpeta de datos junto al ejecutable; debe coincidir con CARPETA_DE_DATOS de
# crates/interfaz/src/rutas.rs (la configuración es portable).
readonly CARPETA_DATOS="datos"

# Nombre de la carpeta de documentación dentro del paquete.
readonly CARPETA_DOCUMENTACION="documentacion"

# Documentos del proyecto que se copian al paquete, en parejas de origen y
# nombre dentro de él. Se copian en cada compilación para que nunca se
# desincronicen. El manual sale de usage.md, para que tenga un solo original.
readonly DOCUMENTOS_ORIGEN=("documentation/features.md" "documentation/usage.md")
readonly DOCUMENTOS_DESTINO=("Referencia de funcionalidades.md" "Manual de uso.md")

# El guion vive en documentation/infrastructure/, dos niveles bajo la raíz.
readonly DIRECTORIO_GUION="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
readonly RAIZ_PROYECTO="$(cd "$DIRECTORIO_GUION/../.." && pwd)"

destino="$RAIZ_PROYECTO/portable"
objetivo=""
carpeta_extras="$DIRECTORIO_GUION/portable-extras"

# --- Funciones -------------------------------------------------------------

# Muestra un error accionable y termina con código 1.
fallar() {
    echo "ERROR: $1" >&2
    exit 1
}

# Lee las opciones de la línea de órdenes.
leer_opciones() {
    while [ $# -gt 0 ]; do
        case "$1" in
            --destino) [ $# -ge 2 ] || fallar "--destino necesita una carpeta"; destino="$2"; shift 2 ;;
            --objetivo) [ $# -ge 2 ] || fallar "--objetivo necesita un triple de Rust"; objetivo="$2"; shift 2 ;;
            --extras) [ $# -ge 2 ] || fallar "--extras necesita una carpeta"; carpeta_extras="$2"; shift 2 ;;
            *) fallar "opción desconocida: $1" ;;
        esac
    done
}

# Comprueba que están cargo, el workspace y los acompañamientos.
comprobar_requisitos() {
    command -v cargo >/dev/null 2>&1 \
        || fallar "no se encuentra 'cargo'. Instala Rust desde https://rustup.rs y vuelve a abrir la terminal."
    [ -f "$RAIZ_PROYECTO/Cargo.toml" ] || fallar "no se encuentra el workspace de Rust: falta $RAIZ_PROYECTO/Cargo.toml"
    [ -f "$carpeta_extras/LEEME.txt" ] \
        || fallar "no se encuentra la carpeta de acompañamientos del portable, o le falta LEEME.txt: $carpeta_extras"
    echo "      $(cargo --version)"
}

# Escribe la carpeta en la que cargo deja el binario de release, respetando
# CARGO_TARGET_DIR como hace el propio cargo.
carpeta_release() {
    local carpeta_target="${CARGO_TARGET_DIR:-$RAIZ_PROYECTO/target}"
    if [ -n "$objetivo" ]; then
        echo "$carpeta_target/$objetivo/release"
    else
        echo "$carpeta_target/release"
    fi
}

# Compila en release con el Cargo.lock confirmado (--locked).
compilar() {
    local argumentos=(build --release --locked -p "$PAQUETE" --manifest-path "$RAIZ_PROYECTO/Cargo.toml")
    [ -n "$objetivo" ] && argumentos+=(--target "$objetivo")
    cargo "${argumentos[@]}" || fallar "la compilación ha fallado."
}

# Vacía el destino salvo datos/. Se vacía en lugar de borrarlo para no
# invalidar accesos directos que apunten a la carpeta.
limpiar_destino() {
    mkdir -p "$destino"
    find "$destino" -mindepth 1 -maxdepth 1 ! -name "$CARPETA_DATOS" -exec rm -rf {} +
    [ -d "$destino/$CARPETA_DATOS" ] && echo "      Se conserva la carpeta '$CARPETA_DATOS' (datos del usuario)"
    return 0
}

# Copia el ejecutable, los acompañamientos y los documentos del proyecto, y
# crea datos/ si no existe. Los documentos van después de los acompañamientos,
# así que mandan sobre una copia con el mismo nombre en portable-extras/.
montar_paquete() {
    local ejecutable="$1"
    local indice origen
    install -m 0755 "$ejecutable" "$destino/$PAQUETE"
    cp -R "$carpeta_extras"/. "$destino"/
    mkdir -p "$destino/$CARPETA_DOCUMENTACION"
    for indice in "${!DOCUMENTOS_ORIGEN[@]}"; do
        origen="${DOCUMENTOS_ORIGEN[$indice]}"
        [ -f "$RAIZ_PROYECTO/$origen" ] || fallar "no se encuentra $origen, que debe ir en el paquete."
        cp "$RAIZ_PROYECTO/$origen" "$destino/$CARPETA_DOCUMENTACION/${DOCUMENTOS_DESTINO[$indice]}"
    done
    mkdir -p "$destino/$CARPETA_DATOS"
}

# Comprueba que el paquete tiene todo lo imprescindible.
comprobar_paquete() {
    local imprescindibles=(
        "$PAQUETE"
        "LEEME.txt"
        "$CARPETA_DATOS"
        "$CARPETA_DOCUMENTACION/Manual de uso.md"
        "$CARPETA_DOCUMENTACION/Resolucion de problemas.md"
        "$CARPETA_DOCUMENTACION/Referencia de funcionalidades.md"
    )
    local faltan=()
    local elemento
    for elemento in "${imprescindibles[@]}"; do
        [ -e "$destino/$elemento" ] || faltan+=("$elemento")
    done
    [ ${#faltan[@]} -eq 0 ] || fallar "el paquete está incompleto. Faltan: ${faltan[*]}"
    echo "      ${#imprescindibles[@]} elementos imprescindibles presentes"
}

# --- Programa principal ----------------------------------------------------

leer_opciones "$@"

echo "=== Compilación portable de Gestor Agentes (Linux) ==="
echo "  Destino  : $destino"
[ -n "$objetivo" ] && echo "  Objetivo : $objetivo"
echo

echo "[1/4] Verificando requisitos..."
comprobar_requisitos

echo "[2/4] Compilando en release (la primera vez tarda unos minutos)..."
compilar
ejecutable_compilado="$(carpeta_release)/$PAQUETE"
[ -f "$ejecutable_compilado" ] || fallar "no se ha generado el ejecutable en $ejecutable_compilado"

echo "[3/4] Montando el paquete..."
limpiar_destino
montar_paquete "$ejecutable_compilado"

echo "[4/4] Comprobando el paquete..."
comprobar_paquete

echo
echo "=== Compilación completada ==="
echo "  Paquete   : $destino"
echo "  Ejecutable: $PAQUETE ($(du -h "$destino/$PAQUETE" | cut -f1))"
echo "  Para arrancar: ./$PAQUETE desde esa carpeta, o doble clic en el gestor de archivos"
