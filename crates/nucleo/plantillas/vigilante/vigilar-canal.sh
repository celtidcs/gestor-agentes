#!/usr/bin/env bash
# Vigilante del canal de Gestor Agentes. Gestor Agentes reescribe este archivo en cada lanzamiento.
#
# Uso: bash .gestor/vigilante/vigilar-canal.sh [--una-vez | --hasta-cambio] [--agente "Claude 2"]
#        [--vida-segundos N]
#
# Sin opciones escribe una linea "CANAL: ha cambiado <archivo>" por cada cambio en .gestor/canal/
# hasta cumplir su vida maxima (--vida-segundos): es para el Monitor de Claude Code o cualquier
# vigilante en segundo plano, que lo rearma al terminar.
# Con --una-vez compara con la pasada anterior, escribe los cambios o "CANAL: sin cambios" y
# termina. Con --hasta-cambio compara igual, en el momento y despues cada pocos segundos, y termina
# en cuanto un cambio lleva etiqueta, tras escribir todos: es para las herramientas que despiertan
# al agente cuando termina una tarea en segundo plano. Todas guardan su referencia en la carpeta propia del
# agente (canal-visto.txt y la carpeta canal-visto/, con una copia de cada archivo).
# Etiquetas: [ENCARGO PARA <identidad>] si una linea nueva dice "Para <identidad>", y
# [MUTACION VIVA] si cambia estado.md y su apartado "## Mutaciones vivas" no dice "Ninguna.".
# No se vigilan la bandeja propia, la guia de Programator ni los historicos del canal.
# La identidad sale de GESTOR_AGENTE o de --agente.
# En los dos modos que esperan (continuo y --hasta-cambio) hay un solo vigilante por agente: el
# ultimo que arranca anota su marca en vigilante.pid, en la carpeta del agente, y el anterior
# termina al verlo. Tambien terminan si muere alguno de los procesos que los lanzaron, para no
# quedar huerfanos, y al cumplir su vida maxima (--vida-segundos), para no quedar en bucle sin fin.
set -u

# Segundos entre dos comprobaciones en modo continuo y en --hasta-cambio: un mensaje se ve en
# menos de ese tiempo sin cargar el equipo, porque cada pasada solo calcula la suma de unos pocos
# archivos pequenos.
INTERVALO_SEGUNDOS=10

# Vida maxima del modo continuo, en segundos. El Monitor de Claude Code caduca como mucho a los
# 1800000 ms (30 minutos) y en Windows no mata el guion al caducar; con 29 minutos el guion
# termina antes que el Monitor mas largo y nunca sigue en bucle sin nadie que lo escuche.
VIDA_CONTINUO_SEGUNDOS=1740

# Vida maxima de --hasta-cambio, en segundos: una jornada de 8 horas. Es solo un tope de
# seguridad; al cumplirlo despierta al agente una vez, sin nada que atender, y este lo relanza.
VIDA_HASTA_CAMBIO_SEGUNDOS=28800

# Cuantos procesos ascendientes se vigilan: el que lanzo el guion (la envoltura de la orden), la
# herramienta del agente y uno mas, porque algunas herramientas ponen un proceso intermedio.
ASCENDIENTES_VIGILADOS=3

una_vez=0
hasta_cambio=0
vida_segundos=""
agente="${GESTOR_AGENTE:-}"
while [ $# -gt 0 ]; do
  case "$1" in
    --una-vez) una_vez=1 ;;
    --hasta-cambio) hasta_cambio=1 ;;
    --agente) shift; agente="${1:-}" ;;
    --vida-segundos) shift; vida_segundos="${1:-}" ;;
    *) echo "VIGILANTE: opcion desconocida: $1" >&2; exit 2 ;;
  esac
  shift
done

if [ "$una_vez" -eq 1 ] && [ "$hasta_cambio" -eq 1 ]; then
  echo "VIGILANTE: --una-vez y --hasta-cambio no se pueden usar a la vez" >&2
  exit 2
fi

if [ -z "$vida_segundos" ]; then
  vida_segundos=$VIDA_CONTINUO_SEGUNDOS
  [ "$hasta_cambio" -eq 1 ] && vida_segundos=$VIDA_HASTA_CAMBIO_SEGUNDOS
fi
case "$vida_segundos" in
  '' | *[!0-9]* | 0*) echo "VIGILANTE: --vida-segundos necesita un entero positivo: $vida_segundos" >&2; exit 2 ;;
esac

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
marcador="$carpeta/vigilante.pid"

# Una linea por archivo del canal que se vigila: nombre y suma de su contenido ("<nombre>
# <suma>-<bytes>"). Deja el resultado en FOTO en lugar de escribirlo, y calcula todas las sumas con
# una sola llamada a cksum: cada subshell o programa que se lanza es un proceso (y en Windows una
# consola nueva), y el modo continuo hace esto cada pocos segundos.
foto() {
  local archivo base minusculas suma bytes ruta
  local -a vigilados=()
  FOTO=""
  for archivo in "$canal"/*.md; do
    [ -f "$archivo" ] || continue
    base="${archivo##*/}"
    minusculas="${base,,}"
    [ "$minusculas" = "$propia" ] && continue
    case "$minusculas" in como-encargar-a-programator.md | historico-* | hist*rico-*) continue ;; esac
    vigilados+=("$archivo")
  done
  [ ${#vigilados[@]} -gt 0 ] || return 0
  while read -r suma bytes ruta; do
    FOTO+="${ruta##*/} $suma-$bytes"$'\n'
  done < <(cksum -- "${vigilados[@]}" 2>/dev/null)
  FOTO="${FOTO%$'\n'}"
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

# Una pasada contra la referencia guardada: escribe los avisos si hay cambios, o el de primera
# pasada si no habia referencia, y guarda la referencia nueva. Devuelve si ha escrito algo.
pasada() {
  local ahora antes hubo=1
  foto
  ahora="$FOTO"
  if [ ! -f "$estado" ]; then
    printf '%s\n' "$ahora" >"$estado"
    copiar_todo "$ahora"
    echo "CANAL: primera pasada, sin referencia anterior; lee todas las bandejas de .gestor/canal/"
    return 0
  fi
  antes=$(cat "$estado")
  avisar "$antes" "$ahora" && hubo=0
  # Solo se reescribe si la foto ha cambiado (tambien si falta un archivo).
  [ "$antes" = "$ahora" ] || printf '%s\n' "$ahora" >"$estado"
  return $hubo
}

mkdir -p "$copias"

if [ "$una_vez" -eq 1 ]; then
  pasada || echo "CANAL: sin cambios"
  exit 0
fi

# Si algun archivo cambiado entre $1 y $2 lleva etiqueta: un encargo para el agente o una
# mutacion viva.
etiquetado() {
  local base
  while IFS= read -r base; do
    [ -n "$base" ] || continue
    [ -n "$(etiquetas "$base")" ] && return 0
  done < <(cambiados "$1" "$2")
  return 1
}

# Anota la marca de este vigilante, que releva a cualquier otro del mismo agente, y la borra al
# terminar si sigue siendo la suya. La marca lleva un numero al azar porque el PID de bash en
# Windows puede repetir el de un vigilante de PowerShell del mismo agente.
marca="bash $$ $RANDOM"
printf '%s\n' "$marca" >"$marcador"
soltar_marca() {
  local leida=""
  [ -f "$marcador" ] && read -r leida <"$marcador"
  [ "$leida" = "$marca" ] && rm -f "$marcador"
}
trap soltar_marca EXIT

# Hora de arranque de un proceso, tal como la da /proc (campo 22 de stat, contado tras el nombre
# entre parentesis, que puede llevar espacios). Deja el resultado en ARRANQUE y PADRE_DE; falla si
# el proceso no existe. Solo usa ordenes internas de bash.
leer_proceso() {
  local linea campos
  ARRANQUE=""
  PADRE_DE=""
  [ -r "/proc/$1/stat" ] || return 1
  read -r linea <"/proc/$1/stat" || return 1
  read -r -a campos <<<"${linea##*) }"
  PADRE_DE="${campos[1]:-}"
  ARRANQUE="${campos[19]:-}"
  [ -n "$ARRANQUE" ]
}

# Ascendientes vigilados, con su hora de arranque para no confundirlos con otro proceso que
# reciba el mismo numero. Se para en el proceso 1: por encima no hay nada que vigilar, y en Windows
# los programas que no son de bash aparecen como hijos del 1.
ascendientes=()
arranques=()
proceso=$PPID
while [ ${#ascendientes[@]} -lt "$ASCENDIENTES_VIGILADOS" ] && [ "$proceso" -gt 1 ] 2>/dev/null; do
  leer_proceso "$proceso" || break
  ascendientes+=("$proceso")
  arranques+=("$ARRANQUE")
  proceso="$PADRE_DE"
done
# Sin /proc solo se puede comprobar el padre directo.
if [ ${#ascendientes[@]} -eq 0 ] && [ "$PPID" -gt 1 ]; then
  ascendientes=("$PPID")
  arranques=("")
fi

# Si sigue vivo cada ascendiente anotado al arrancar.
ascendientes_vivos() {
  local i
  for i in "${!ascendientes[@]}"; do
    if [ -n "${arranques[$i]}" ]; then
      leer_proceso "${ascendientes[$i]}" || return 1
      [ "$ARRANQUE" = "${arranques[$i]}" ] || return 1
    else
      kill -0 "${ascendientes[$i]}" 2>/dev/null || return 1
    fi
  done
}

# Termina si otro vigilante del agente lo ha relevado, si ha muerto quien lo lanzo o si ha
# cumplido su vida maxima. Solo usa ordenes internas de bash, que no lanzan procesos.
seguir_o_terminar() {
  local leida=""
  [ -f "$marcador" ] && read -r leida <"$marcador"
  if [ "$leida" != "$marca" ]; then
    echo "VIGILANTE: relevado por otro vigilante de $identidad; no lo rearmes"
    exit 0
  fi
  if ! ascendientes_vivos; then
    echo "VIGILANTE: fin porque se cerro quien lo lanzo; relanzalo si sigues trabajando"
    exit 0
  fi
  if [ "$SECONDS" -ge "$vida_segundos" ]; then
    echo "VIGILANTE: fin por tiempo; rearma"
    exit 0
  fi
}

# --hasta-cambio solo termina con algo para el agente, para no despertarlo por cambios que no le
# tocan. Lo demas no se da por visto: sale en el aviso que lo despierte o en su siguiente pasada.
if [ "$hasta_cambio" -eq 1 ]; then
  if [ ! -f "$estado" ]; then
    pasada
    exit 0
  fi
  while true; do
    seguir_o_terminar
    read -r -d '' antes <"$estado" || true
    foto
    ahora="$FOTO"
    if [ "$antes" != "$ahora" ] && etiquetado "$antes" "$ahora"; then
      avisar "$antes" "$ahora"
      printf '%s\n' "$ahora" >"$estado"
      exit 0
    fi
    sleep "$INTERVALO_SEGUNDOS"
  done
fi

foto
antes="$FOTO"
copiar_todo "$antes"
echo "CANAL: vigilando .gestor/canal/ cada $INTERVALO_SEGUNDOS s, sin $propia, durante $vida_segundos s"
while true; do
  sleep "$INTERVALO_SEGUNDOS"
  seguir_o_terminar
  foto
  ahora="$FOTO"
  [ "$antes" = "$ahora" ] || avisar "$antes" "$ahora"
  antes="$ahora"
done
