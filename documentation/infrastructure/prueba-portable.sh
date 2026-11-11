#!/usr/bin/env bash
# prueba-portable.sh - Barreras del paquete portable de Linux
# ---------------------------------------------------------------------------
# Proyecto : Gestor Agentes 3.0.0
# Propósito: comprobar mecánicamente que build-portable.sh produce el paquete
#            acordado, o que un paquete ya hecho lo cumple.
#            Vive aparte del guion que vigila.
#
# Qué comprueba:
#   1. La compilación termina con código 0.
#   2. El paquete contiene exactamente lo acordado.
#   3. La referencia y el manual son copia exacta de features.md y usage.md.
#   4. datos/ existe y está vacía en un paquete nuevo.
#   5. El ejecutable es ELF de 64 bits, tiene permiso de ejecución y ldd no
#      da ninguna biblioteca sin encontrar.
#   6. Los textos no hablan de la 2.x.
#   7. Recompilar conserva datos/ byte a byte y regenera todo lo demás.
#   8. Sin cargo, o sin acompañamientos, el guion falla con un mensaje claro.
#
# Uso: ./prueba-portable.sh                      compila un paquete de prueba
#      ./prueba-portable.sh --paquete <carpeta>  comprueba ese paquete sin
#                                                compilar: solo lee la
#                                                carpeta, no la cambia.
# Código 0 si todo pasa; 1 si algo falla.
# Sin --paquete escribe solo en <raíz>/target/prueba-portable, que borra al terminar.
# ---------------------------------------------------------------------------

set -uo pipefail

readonly DIRECTORIO_GUION="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
readonly GUION="$DIRECTORIO_GUION/build-portable.sh"
readonly RAIZ="$(cd "$DIRECTORIO_GUION/../.." && pwd)"
readonly CARPETA_PRUEBA="$RAIZ/target/prueba-portable"
readonly PAQUETE="$CARPETA_PRUEBA/portable"

# Contenido exacto acordado del paquete, con rutas relativas ordenadas
# con LC_ALL=C.
readonly CONTENIDO_ESPERADO="LEEME.txt
datos
documentacion
documentacion/Manual de uso.md
documentacion/Referencia de funcionalidades.md
documentacion/Resolucion de problemas.md
gestor-agentes"

# Documentos que el paquete lleva como copia exacta: origen en el
# proyecto, nombre dentro de documentacion/ y texto de la barrera. La frase de
# la referencia es la de siempre, para que la salida sin --paquete no cambie.
readonly COPIAS_ORIGEN=("documentation/features.md" "documentation/usage.md")
readonly COPIAS_NOMBRE=("Referencia de funcionalidades.md" "Manual de uso.md")
readonly COPIAS_FRASE=("la referencia de funcionalidades es copia exacta de features.md"
    "el manual de uso es copia exacta de usage.md")

# Textos de la 2.x que no deben aparecer en el paquete.
readonly TEXTOS_PROHIBIDOS=("GestorClaude.exe" ".NET" "Guardar configuracion" "Restaurar configuracion"
    "configuracion-portable" "Gestor Claude 2.")

fallos=0
paquete_hecho=""

# Anota el resultado de una comprobación: nombre, 0 si es correcta, detalle.
anotar() {
    if [ "$2" -eq 0 ]; then
        echo "OK     $1"
    else
        fallos=$((fallos + 1))
        echo "FALLO  $1 ${3:-}"
    fi
}

# Ejecuta build-portable.sh y deja su salida en $salida y su código en $codigo.
ejecutar_guion() {
    salida="$(bash "$GUION" "$@" 2>&1)"
    codigo=$?
}

# Suma SHA-256 de cada archivo de una carpeta, con su ruta relativa, ordenadas.
sumas_de_carpeta() {
    (cd "$1" && find . -type f -print0 | LC_ALL=C sort -z | xargs -0 -r sha256sum)
}

terminar() {
    [ -n "$paquete_hecho" ] || rm -rf "$CARPETA_PRUEBA"
    echo
    if [ "$fallos" -eq 0 ]; then
        echo "PRUEBA PORTABLE: OK"
        exit 0
    fi
    echo "PRUEBA PORTABLE: $fallos fallos"
    exit 1
}

# Barreras del contenido de un paquete, comunes al paquete compilado aquí y a
# uno ya hecho. Argumentos: la carpeta y «nuevo» si se acaba de compilar
# (entonces datos/ tiene que estar vacía; en un paquete ya hecho lleva la
# configuración del usuario y solo se exige que exista).
barreras_del_paquete() {
    local ruta="$1" nuevo="$2"
    local contenido resultado indice encontrados archivo prohibido sin_encontrar

    # Contenido exacto. En un paquete ya hecho no cuenta lo que hay dentro de datos/.
    contenido="$(cd "$ruta" && find . -mindepth 1 | sed 's|^\./||' | LC_ALL=C sort)"
    [ "$nuevo" = "nuevo" ] || contenido="$(echo "$contenido" | grep -v '^datos/')"
    [ "$contenido" = "$CONTENIDO_ESPERADO" ]
    resultado=$?
    anotar "el paquete tiene exactamente el contenido acordado" "$resultado" \
        "$(diff <(echo "$CONTENIDO_ESPERADO") <(echo "$contenido"))"
    for indice in "${!COPIAS_ORIGEN[@]}"; do
        cmp -s "$RAIZ/${COPIAS_ORIGEN[$indice]}" "$ruta/documentacion/${COPIAS_NOMBRE[$indice]}"
        anotar "${COPIAS_FRASE[$indice]}" $?
    done

    # datos/.
    if [ "$nuevo" = "nuevo" ]; then
        [ -d "$ruta/datos" ] && [ -z "$(ls -A "$ruta/datos")" ]
        anotar "datos/ existe y está vacía" $?
    else
        [ -d "$ruta/datos" ]
        anotar "datos/ existe" $?
    fi

    # ELF de 64 bits, ejecutable y con sus bibliotecas.
    # Los cinco primeros bytes de un ELF de 64 bits son 7f 45 4c 46 02 (especificación ELF, e_ident).
    [ "$(head -c 5 "$ruta/gestor-agentes" | od -An -tx1 | tr -d ' \n')" = "7f454c4602" ]
    anotar "el ejecutable es ELF de 64 bits" $?
    [ -x "$ruta/gestor-agentes" ]
    anotar "el ejecutable tiene permiso de ejecución" $?
    sin_encontrar="$(ldd "$ruta/gestor-agentes" 2>&1 | grep 'not found')"
    [ -z "$sin_encontrar" ]
    anotar "ldd no da bibliotecas sin encontrar" $? "$sin_encontrar"

    # Textos. La referencia puede citar la 2.x en su historia; el manual no.
    encontrados=""
    while IFS= read -r -d '' archivo; do
        for prohibido in "${TEXTOS_PROHIBIDOS[@]}"; do
            grep -qF -- "$prohibido" "$archivo" && encontrados+="$(basename "$archivo"): $prohibido; "
        done
    done < <(find "$ruta" -type f \( -name '*.txt' -o -name '*.md' \) ! -name 'Referencia de funcionalidades.md' \
        ! -path "$ruta/datos/*" -print0)
    [ -z "$encontrados" ]
    anotar "los textos del paquete no hablan de la 2.x" $? "$encontrados"
}

# Modo paquete: comprueba un paquete ya hecho sin compilar y
# sin escribir en él.
comprobar_paquete_hecho() {
    local ruta="$1" commit sumas_antes sumas_despues
    [ -d "$ruta" ] || { anotar "existe el paquete $ruta" 1; terminar; }
    ruta="$(cd "$ruta" && pwd)"
    commit="(sin git)"
    command -v git > /dev/null && commit="$(git -C "$RAIZ" -c safe.directory='*' rev-parse HEAD 2> /dev/null)"
    echo "MODO PAQUETE: $ruta"
    echo "MAQUINA: $(hostname) | $(uname -sr) | $(date '+%F %T')"
    echo "COMMIT DEL ARBOL: $commit"
    sumas_antes="$(sumas_de_carpeta "$ruta")"
    echo "$sumas_antes" | sed 's|  \./|  |; s|^|SHA256 |'

    barreras_del_paquete "$ruta" "hecho"

    for omitida in "la compilación termina con código 0" "la recompilación conserva datos/" \
        "sin cargo falla" "sin acompañamientos falla"; do
        echo "OMITIDA en modo paquete: $omitida"
    done

    # La carpeta solo se ha leído.
    sumas_despues="$(sumas_de_carpeta "$ruta")"
    [ "$sumas_antes" = "$sumas_despues" ]
    anotar "el paquete no ha cambiado al comprobarlo" $?
}

if [ "${1:-}" = "--paquete" ]; then
    [ -n "${2:-}" ] || { echo "Uso: $0 --paquete <carpeta>"; exit 2; }
    paquete_hecho="$2"
    comprobar_paquete_hecho "$paquete_hecho"
    terminar
fi

rm -rf "$CARPETA_PRUEBA"
mkdir -p "$CARPETA_PRUEBA"

# 1. Compilación.
ejecutar_guion --destino "$PAQUETE"
anotar "la compilación termina con código 0" "$codigo" "$salida"
[ "$codigo" -eq 0 ] || terminar

# 2 a 6. Barreras del contenido del paquete.
barreras_del_paquete "$PAQUETE" "nuevo"

# 7. Recompilar conserva datos/.
printf '{"prueba": "conservar á"}\n' > "$PAQUETE/datos/config.json"
huella_antes="$(sha256sum "$PAQUETE/datos/config.json" | cut -d' ' -f1)"
echo "no debe sobrevivir" > "$PAQUETE/residuo.txt"
ejecutar_guion --destino "$PAQUETE"
anotar "la recompilación termina con código 0" "$codigo" "$salida"
[ -f "$PAQUETE/datos/config.json" ] && [ "$(sha256sum "$PAQUETE/datos/config.json" | cut -d' ' -f1)" = "$huella_antes" ]
anotar "recompilar conserva datos/ byte a byte" $?
[ ! -e "$PAQUETE/residuo.txt" ]
anotar "recompilar borra lo que no es del paquete" $?

# 8a. Sin cargo: PATH con solo las órdenes básicas del sistema.
salida="$(PATH=/usr/bin:/bin bash "$GUION" --destino "$CARPETA_PRUEBA/sin-cargo" 2>&1)"
codigo=$?
if command -v cargo | grep -qE '^/usr/bin/|^/bin/'; then
    anotar "sin cargo: no se puede comprobar, porque cargo está en /usr/bin o /bin" 1
else
    [ "$codigo" -ne 0 ]
    anotar "sin cargo falla con código distinto de 0" $?
    echo "$salida" | grep -q "no se encuentra 'cargo'"
    anotar "sin cargo el mensaje lo dice" $? "$salida"
fi

# 8b. Sin acompañamientos.
ejecutar_guion --destino "$CARPETA_PRUEBA/sin-extras" --extras "$CARPETA_PRUEBA/no-existe"
[ "$codigo" -ne 0 ]
anotar "sin acompañamientos falla con código distinto de 0" $?
echo "$salida" | grep -q "acompañamientos"
anotar "sin acompañamientos el mensaje lo dice" $? "$salida"

terminar
