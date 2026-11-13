#!/usr/bin/env bash
# Vigilante del canal de Gestor Agentes. Gestor Agentes reescribe este archivo en cada lanzamiento.
#
# Uso: bash .gestor/vigilante/vigilar-canal.sh [--una-vez] [--agente "Claude 2"]
#
# Sin --una-vez escribe una linea "CANAL: ha cambiado <archivo>" por cada cambio en .gestor/canal/
# y no termina nunca: es para el Monitor de Claude Code o cualquier vigilante en segundo plano.
# Con --una-vez compara con la pasada anterior, escribe los cambios o "CANAL: sin cambios" y
# termina. Las dos guardan su referencia en la carpeta propia del agente (canal-visto.txt y la
# carpeta canal-visto/, con una copia de cada archivo).
# Etiquetas: [ENCARGO PARA <identidad>] si una linea nueva dice "Para <identidad>", y
# [MUTACION VIVA] si cambia estado.md y su apartado "## Mutaciones vivas" no dice "Ninguna.".
# No se vigilan la bandeja propia, la guia de Programator ni los historicos del canal.
# La identidad sale de GESTOR_AGENTE o de --agente.
set -u

# Segundos entre dos comprobaciones en modo continuo: un mensaje se ve en menos de ese tiempo
# sin cargar el equipo, porque cada pasada solo calcula la suma de unos pocos archivos pequenos.
INTERVALO_SEGUNDOS=10

una_vez=0
agente="${GESTOR_AGENTE:-}"
while [ $# -gt 0 ]; do
  case "$1" in
    --una-vez) una_vez=1 ;;
    --agente) shift; agente="${1:-}" ;;
    *) echo "VIGILANTE: opcion desconocida: $1" >&2; exit 2 ;;
  esac
  shift
done

if [ -z "$agente" ]; then
  echo "VIGILANTE: GESTOR_AGENTE esta vacia; pasa tu identidad con --agente \"Claude 2\"" >&2
  exit 2
fi

primera="${agente%% *}"
herramienta=$(printf '%s' "$primera" | tr '[:upper:]' '[:lower:]')
numero="${agente##* }"
[ "$numero" = "$agente" ] && numero=1
case "$numero" in
  '' | *[!0-9]* | 0) echo "VIGILANTE: identidad no valida: $agente" >&2; exit 2 ;;
esac
case "$herramienta" in
  '' | *[!a-z0-9]*) echo "VIGILANTE: identidad no valida: $agente" >&2; exit 2 ;;
esac
nombre="$herramienta"
[ "$numero" -gt 1 ] && nombre="$herramienta$numero"
identidad="$primera $numero"

gestor="$(cd "$(dirname "$0")/.." && pwd)"
canal="$gestor/canal"
propia="$nombre.md"
carpeta="$gestor/$nombre"
estado="$carpeta/canal-visto.txt"
copias="$carpeta/canal-visto"

# Una linea por archivo del canal que se vigila: nombre y suma de su contenido.
foto() {
  for archivo in "$canal"/*.md; do
    [ -f "$archivo" ] || continue
    base=$(basename "$archivo")
    minusculas=$(printf '%s' "$base" | tr '[:upper:]' '[:lower:]')
    [ "$minusculas" = "$propia" ] && continue
    case "$minusculas" in como-encargar-a-programator.md | historico-* | hist*rico-*) continue ;; esac
    printf '%s %s\n' "$base" "$(cksum <"$archivo" | tr ' ' '-')"
  done
}

# Nombres de los archivos de $2 cuya linea no estaba en $1.
cambiados() {
  awk 'NR == FNR { visto[$0] = 1; next } NF && !($0 in visto) { print $1 }' \
    <(printf '%s\n' "$1") <(printf '%s\n' "$2")
}

# Si el apartado "## Mutaciones vivas" de un estado.md dice algo distinto de "Ninguna.".
hay_mutaciones_vivas() {
  awk '{ sub(/\r$/, "") }
       $0 == "## Mutaciones vivas" { dentro = 1; next }
       dentro && /^#/ { dentro = 0 }
       dentro && NF && $0 != "Ninguna." { hay = 1 }
       END { exit hay ? 0 : 1 }' "$1"
}

# Etiquetas de un archivo cambiado, comparado con su copia de la pasada anterior.
etiquetas() {
  local actual="$canal/$1" previo="$copias/$1" nuevas resultado=""
  if [ -f "$previo" ]; then
    nuevas=$(grep -vxF -f "$previo" "$actual")
  else
    nuevas=$(cat "$actual")
  fi
  if printf '%s\n' "$nuevas" | grep -qiE "(^|[^[:alnum:]])Para $identidad([^[:alnum:]]|$)"; then
    resultado=" [ENCARGO PARA $identidad]"
  fi
  if [ "$1" = "estado.md" ] && hay_mutaciones_vivas "$actual"; then
    resultado="$resultado [MUTACION VIVA]"
  fi
  printf '%s' "$resultado"
}

# Avisa de cada archivo cambiado entre $1 y $2, con sus etiquetas, y guarda su copia. Devuelve si
# hubo alguno.
avisar() {
  local hubo=1 base
  while IFS= read -r base; do
    [ -n "$base" ] || continue
    echo "CANAL: ha cambiado .gestor/canal/$base$(etiquetas "$base")"
    cp "$canal/$base" "$copias/$base" 2>/dev/null
    hubo=0
  done < <(cambiados "$1" "$2")
  return $hubo
}

# Copia todos los archivos vigilados, como referencia de la pasada siguiente.
copiar_todo() {
  local linea
  while IFS= read -r linea; do
    [ -n "$linea" ] && cp "$canal/${linea%% *}" "$copias/${linea%% *}" 2>/dev/null
  done <<<"$1"
}

mkdir -p "$copias"

if [ "$una_vez" -eq 1 ]; then
  ahora=$(foto)
  if [ ! -f "$estado" ]; then
    printf '%s\n' "$ahora" >"$estado"
    copiar_todo "$ahora"
    echo "CANAL: primera pasada, sin referencia anterior; lee todas las bandejas de .gestor/canal/"
    exit 0
  fi
  antes=$(cat "$estado")
  avisar "$antes" "$ahora" || echo "CANAL: sin cambios"
  printf '%s\n' "$ahora" >"$estado"
  exit 0
fi

antes=$(foto)
copiar_todo "$antes"
echo "CANAL: vigilando .gestor/canal/ cada $INTERVALO_SEGUNDOS s, sin $propia"
while true; do
  sleep "$INTERVALO_SEGUNDOS"
  ahora=$(foto)
  avisar "$antes" "$ahora"
  antes="$ahora"
done
