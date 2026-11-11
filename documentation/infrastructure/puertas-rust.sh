#!/usr/bin/env bash
# puertas-rust.sh — Las cuatro puertas de aceptación de Gestor Agentes 3.0.0 (Rust) en Linux.
#
# Un cambio solo se acepta si pasan las cuatro, sobre todo el workspace:
#   1. Formato: cargo fmt sin ninguna diferencia.
#   2. Linter: cargo clippy en todos los destinos, con los avisos tratados como errores.
#   3. Pruebas: toda la suite en verde, con las dependencias exactas del Cargo.lock.
#   4. Auditoría: cargo audit sin vulnerabilidades conocidas.
#
# Requisitos: Rust (cargo) y cargo-audit en el PATH.
#
# Uso, desde cualquier carpeta:
#   ./puertas-rust.sh
#   ./puertas-rust.sh /ruta/de/la/cache/target
# La carpeta de compilación se pasa a cargo con --target-dir. No se usa CARGO_TARGET_DIR porque
# sccache mete las variables CARGO_* en su clave y entonces no comparte la caché entre
# compilaciones que usen carpetas distintas.
#
# Devuelve 0 si pasan las cuatro y 1 si falla alguna; siempre las ejecuta todas.

set -u
cd "$(dirname "$0")/../.."

FALLOS=0
DESTINO=()
[ $# -ge 1 ] && [ -n "$1" ] && DESTINO=(--target-dir "$1")

# Ejecuta una puerta, informa del resultado y acumula el fallo sin detener las demás.
puerta() {
    local nombre="$1"; shift
    echo "=== Puerta: $nombre ==="
    if "$@"; then
        echo "--- $nombre: CORRECTA"
    else
        echo "--- $nombre: FALLIDA"
        FALLOS=$((FALLOS + 1))
    fi
}

# cargo audit devuelve un código distinto de 0 cuando encuentra vulnerabilidades.
auditar() {
    if ! command -v cargo-audit >/dev/null 2>&1; then
        echo "No se encuentra cargo-audit. Instálalo con: cargo install cargo-audit --locked"
        return 1
    fi
    cargo audit
}

if ! command -v cargo >/dev/null 2>&1; then
    echo "No se encuentra 'cargo'. Instala Rust desde https://rustup.rs y vuelve a abrir la terminal."
    exit 1
fi

puerta "Formato" cargo fmt --all --check
puerta "Linter (clippy sin avisos)" cargo clippy --workspace --all-targets --locked "${DESTINO[@]}" -- -D warnings
puerta "Pruebas" cargo test --workspace --locked "${DESTINO[@]}"
puerta "Auditoría de dependencias" auditar

echo
if [ "$FALLOS" -eq 0 ]; then
    echo "=== Las cuatro puertas están en verde ==="
    exit 0
fi
echo "=== $FALLOS puerta(s) fallida(s) ==="
exit 1
