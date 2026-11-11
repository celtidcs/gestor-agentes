#!/usr/bin/env bash
# prueba-setup.sh - Barreras de setup.sh y puertas-rust.sh en Linux.
#
# Vive aparte de los guiones que vigila. Comprueba:
#   1. puertas-rust.sh ejecuta exactamente las cuatro puertas y termina con código 0, y con
#      una puerta en rojo termina con un código distinto de 0.
#   2. setup.sh --sin-puertas termina con código 0 y deja portable/ con el ejecutable y datos/.
#   3. Sin cargo en el PATH, setup.sh y puertas-rust.sh fallan y remiten a https://rustup.rs.
#
# Uso: ./prueba-setup.sh [carpeta de compilación]   (código 0 si todo pasa; 1 si no)
# Regenera portable/ en la raíz del proyecto, igual que setup.sh, y conserva su carpeta datos/.

set -u

readonly DIRECTORIO="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
readonly RAIZ="$(cd "$DIRECTORIO/../.." && pwd)"
readonly COMPILACION="${1:-}"

# Encabezados que escribe puertas-rust.sh, uno por puerta, en este orden.
readonly PUERTAS_ESPERADAS="=== Puerta: Formato ===
=== Puerta: Linter (clippy sin avisos) ===
=== Puerta: Pruebas ===
=== Puerta: Auditoría de dependencias ==="

fallos=0

anotar() {  # nombre, 0 si es correcta, detalle
    if [ "$2" -eq 0 ]; then
        echo "OK     $1"
    else
        fallos=$((fallos + 1))
        echo "FALLO  $1 ${3:-}"
    fi
}

# 1. Las cuatro puertas, exactamente.
salida="$(bash "$DIRECTORIO/puertas-rust.sh" "$COMPILACION" 2>&1)"; codigo=$?
encabezados="$(echo "$salida" | grep '^=== Puerta: ')"
[ "$encabezados" = "$PUERTAS_ESPERADAS" ]
anotar "puertas-rust ejecuta exactamente las cuatro puertas" $? "$encabezados"
anotar "puertas-rust termina con código 0" "$codigo" "$(echo "$salida" | tail -15)"

# 1b. Con una puerta en rojo, puertas-rust falla. Se fuerza pasando como carpeta de compilación
# una subcarpeta dentro de un archivo, donde cargo no puede crear una carpeta.
archivo="$RAIZ/target/prueba-setup-no-es-carpeta"
mkdir -p "$(dirname "$archivo")"
echo "no es una carpeta" > "$archivo"
bash "$DIRECTORIO/puertas-rust.sh" "$archivo/sub" >/dev/null 2>&1; codigo=$?
rm -rf "$archivo"
[ "$codigo" -ne 0 ]
anotar "puertas-rust con una puerta en rojo termina con código distinto de 0" $?

# 2. setup sin puertas. Antes se quita el ejecutable de una ejecución anterior, para que la
# comprobación no pase por un resto; datos/ se deja, porque es del usuario.
rm -f "$RAIZ/portable/gestor-agentes"
salida="$(bash "$DIRECTORIO/setup.sh" --sin-puertas 2>&1)"; codigo=$?
anotar "setup --sin-puertas termina con código 0" "$codigo" "$(echo "$salida" | tail -15)"
[ -x "$RAIZ/portable/gestor-agentes" ]
anotar "setup deja el ejecutable en portable/" $?
[ -d "$RAIZ/portable/datos" ]
anotar "setup deja la carpeta portable/datos" $?

# 3. Sin cargo: PATH con solo las órdenes básicas del sistema.
if command -v cargo | grep -qE '^/usr/bin/|^/bin/'; then
    anotar "sin cargo: no se puede comprobar, porque cargo está en /usr/bin o /bin" 1
else
    salida="$(PATH=/usr/bin:/bin bash "$DIRECTORIO/setup.sh" --sin-puertas 2>&1)"; codigo=$?
    [ "$codigo" -ne 0 ]; anotar "setup sin cargo falla" $?
    echo "$salida" | grep -q "rustup\.rs"; anotar "setup sin cargo remite a rustup" $?
    salida="$(PATH=/usr/bin:/bin bash "$DIRECTORIO/puertas-rust.sh" 2>&1)"; codigo=$?
    [ "$codigo" -ne 0 ]; anotar "puertas-rust sin cargo falla" $?
    echo "$salida" | grep -q "rustup\.rs"; anotar "puertas-rust sin cargo remite a rustup" $?
fi

echo
if [ "$fallos" -eq 0 ]; then
    echo "PRUEBA SETUP: OK"
    exit 0
fi
echo "PRUEBA SETUP: $fallos fallos"
exit 1
