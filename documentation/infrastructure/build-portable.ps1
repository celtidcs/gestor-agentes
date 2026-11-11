# build-portable.ps1 - Compila Gestor Agentes como paquete portable para Windows
# ---------------------------------------------------------------------------
# Proyecto : Gestor Agentes 3.0.0 (antes Gestor Claude)
# Proposito: Producir una carpeta autosuficiente que se pueda copiar a una
#            memoria USB o a otro equipo con Windows y funcionar alli sin
#            instalar nada, con toda la documentacion que el usuario necesita.
#
# Contenido del paquete resultante:
#   gestor-agentes.exe   Ejecutable unico en Rust, sin dependencias de .NET.
#   datos\               Configuracion del usuario (lista de proyectos y
#                        perfiles). Se crea vacia y la rellena la aplicacion.
#   LEEME.txt            Primeros pasos.
#   documentacion\       Manual de uso, resolucion de problemas y referencia
#                        de funcionalidades.
#
# La version para Linux la genera build-portable.sh, en Linux, con los mismos
# acompanamientos de portable-extras\. Las comprobaciones del paquete (barreras)
# estan aparte, en prueba-portable.ps1.
#
# Uso:
#   .\build-portable.ps1                        # salida en <raiz>\portable
#   .\build-portable.ps1 -Destino D:\USB\App    # carpeta de salida a medida
#   .\build-portable.ps1 -Objetivo aarch64-pc-windows-msvc
#
# Idempotente: cada ejecucion regenera el paquete y conserva datos\, que son
# los datos del usuario y no un artefacto de la compilacion.
# ---------------------------------------------------------------------------

param(
    # Carpeta de salida. Por defecto, "portable" en la raiz del proyecto.
    [string]$Destino,

    # Triple de destino de Rust. Vacio: el del propio equipo.
    [string]$Objetivo = "",

    # Carpeta de los acompanamientos. Por defecto, portable-extras junto a este
    # guion; se puede cambiar para probar el guion sin tocar el repositorio.
    [string]$CarpetaExtras
)

$ErrorActionPreference = "Stop"

# --- Constantes ------------------------------------------------------------

# Paquete de cargo que produce el ejecutable (crates/interfaz/Cargo.toml).
$PAQUETE = "gestor-agentes"

# Nombre del ejecutable que genera cargo en Windows.
$EJECUTABLE = "$PAQUETE.exe"

# Carpeta de datos junto al ejecutable; debe coincidir con CARPETA_DE_DATOS de
# crates/interfaz/src/rutas.rs (la configuracion es portable).
$CARPETA_DATOS = "datos"

# Nombre de la carpeta de documentacion dentro del paquete portable.
$CARPETA_DOCUMENTACION = "documentacion"

# Carpetas del paquete que no se borran al recompilar, porque son del usuario.
$CARPETAS_PRESERVADAS = @($CARPETA_DATOS)

# Documentos del proyecto que se copian al paquete, con el nombre que tendran
# dentro de el. Se copian en cada compilacion para que nunca se desincronicen.
# El manual sale de usage.md, para que tenga un solo
# original; como se copia despues de portable-extras, manda sobre una copia alli.
$DOCUMENTOS_DEL_PROYECTO = @{
    "documentation\features.md" = "Referencia de funcionalidades.md"
    "documentation\usage.md"    = "Manual de uso.md"
}

# El guion vive en documentation/infrastructure/, dos niveles bajo la raiz.
$raizProyecto = Split-Path -Parent (Split-Path -Parent $PSScriptRoot)

if ([string]::IsNullOrWhiteSpace($CarpetaExtras)) {
    $CarpetaExtras = Join-Path $PSScriptRoot "portable-extras"
}
if ([string]::IsNullOrWhiteSpace($Destino)) {
    $Destino = Join-Path $raizProyecto "portable"
}

# --- Funciones -------------------------------------------------------------

<#
.SYNOPSIS
    Comprueba que estan disponibles cargo, el workspace y los acompanamientos.
.DESCRIPTION
    Lanza una excepcion con un mensaje accionable si falta algun requisito.
#>
function Test-RequisitosCompilacion {
    if ($null -eq (Get-Command cargo -ErrorAction SilentlyContinue)) {
        throw "No se encuentra 'cargo'. Instala Rust desde https://rustup.rs y vuelve a abrir la terminal."
    }
    if (-not (Test-Path (Join-Path $raizProyecto "Cargo.toml"))) {
        throw "No se encuentra el workspace de Rust: falta $(Join-Path $raizProyecto 'Cargo.toml')"
    }
    if (-not (Test-Path (Join-Path $CarpetaExtras "LEEME.txt"))) {
        throw "No se encuentra la carpeta de acompanamientos del portable, o le falta LEEME.txt: $CarpetaExtras"
    }
    Write-Host "      $(& cargo --version)" -ForegroundColor Green
}

<#
.SYNOPSIS
    Devuelve la carpeta en la que cargo deja el ejecutable de release.
.DESCRIPTION
    Respeta CARGO_TARGET_DIR si esta fijada, como hace el propio cargo, y
    anade el triple cuando se compila para un destino explicito.
#>
function Get-CarpetaRelease {
    $carpetaTarget = $env:CARGO_TARGET_DIR
    if ([string]::IsNullOrWhiteSpace($carpetaTarget)) {
        $carpetaTarget = Join-Path $raizProyecto "target"
    }
    if ($Objetivo -ne "") {
        return Join-Path (Join-Path $carpetaTarget $Objetivo) "release"
    }
    return Join-Path $carpetaTarget "release"
}

<#
.SYNOPSIS
    Compila el ejecutable en modo release con el Cargo.lock confirmado.
.DESCRIPTION
    --locked impide que cargo cambie el Cargo.lock: el paquete se construye
    exactamente con las dependencias auditadas. El perfil release del
    workspace ya activa lto y strip.
#>
function Invoke-Compilacion {
    $argumentos = @("build", "--release", "--locked", "-p", $PAQUETE,
        "--manifest-path", (Join-Path $raizProyecto "Cargo.toml"))
    if ($Objetivo -ne "") { $argumentos += @("--target", $Objetivo) }

    # cargo escribe su progreso en la salida de errores. En Windows PowerShell 5.1,
    # si esa salida esta redirigida, cada linea llega como un error y, con
    # ErrorActionPreference en Stop, detendria el guion aunque cargo vaya bien.
    # Por eso aqui solo manda el codigo de salida de cargo.
    $preferenciaAnterior = $ErrorActionPreference
    $ErrorActionPreference = "Continue"
    try {
        & cargo @argumentos 2>&1 | ForEach-Object { Write-Host "      $_" }
        $codigo = $LASTEXITCODE
    } finally {
        $ErrorActionPreference = $preferenciaAnterior
    }
    if ($codigo -ne 0) { throw "La compilacion ha fallado (codigo $codigo)." }
}

<#
.SYNOPSIS
    Deja la carpeta de destino lista para recibir un paquete nuevo.
.DESCRIPTION
    Borra el contenido anterior salvo $CARPETAS_PRESERVADAS. Se vacia la
    carpeta en lugar de borrarla para no invalidar accesos directos que el
    usuario pueda tener apuntando a ella.
.PARAMETER Ruta
    Carpeta de salida del paquete portable.
#>
function Clear-CarpetaDestino {
    param([Parameter(Mandatory = $true)][string]$Ruta)

    if (-not (Test-Path $Ruta)) {
        New-Item -ItemType Directory -Path $Ruta -Force | Out-Null
        return
    }

    Get-ChildItem -Path $Ruta -Force |
        Where-Object { $_.Name -notin $CARPETAS_PRESERVADAS } |
        Remove-Item -Recurse -Force

    foreach ($preservada in $CARPETAS_PRESERVADAS) {
        if (Test-Path (Join-Path $Ruta $preservada)) {
            Write-Host "      Se conserva la carpeta '$preservada' (datos del usuario)" -ForegroundColor DarkGray
        }
    }
}

<#
.SYNOPSIS
    Copia al paquete el ejecutable, los acompanamientos y la carpeta de datos.
.DESCRIPTION
    Los acompanamientos vienen de portable-extras (LEEME y documentacion para
    el usuario final) y de los documentos del proyecto listados en
    $DOCUMENTOS_DEL_PROYECTO, que se copian con un nombre legible.
.PARAMETER Ruta
    Carpeta de salida.
.PARAMETER RutaEjecutable
    Ruta del ejecutable recien compilado.
#>
function Copy-ContenidoPortable {
    param(
        [Parameter(Mandatory = $true)][string]$Ruta,
        [Parameter(Mandatory = $true)][string]$RutaEjecutable
    )

    Copy-Item -Path $RutaEjecutable -Destination (Join-Path $Ruta $EJECUTABLE) -Force
    Copy-Item -Path (Join-Path $CarpetaExtras "*") -Destination $Ruta -Recurse -Force

    $carpetaDocs = Join-Path $Ruta $CARPETA_DOCUMENTACION
    if (-not (Test-Path $carpetaDocs)) {
        New-Item -ItemType Directory -Path $carpetaDocs -Force | Out-Null
    }
    foreach ($origenRelativo in $DOCUMENTOS_DEL_PROYECTO.Keys) {
        $origen = Join-Path $raizProyecto $origenRelativo
        if (-not (Test-Path $origen)) { throw "No se encuentra $origenRelativo, que debe ir en el paquete." }
        Copy-Item -Path $origen -Destination (Join-Path $carpetaDocs $DOCUMENTOS_DEL_PROYECTO[$origenRelativo]) -Force
    }

    $carpetaDatos = Join-Path $Ruta $CARPETA_DATOS
    if (-not (Test-Path $carpetaDatos)) {
        New-Item -ItemType Directory -Path $carpetaDatos -Force | Out-Null
    }
}

# --- Programa principal ----------------------------------------------------

Write-Host "=== Compilacion portable de Gestor Agentes (Windows) ===" -ForegroundColor Cyan
Write-Host "  Destino  : $Destino"
if ($Objetivo -ne "") { Write-Host "  Objetivo : $Objetivo" }
Write-Host ""

Write-Host "[1/4] Verificando requisitos..." -ForegroundColor Yellow
Test-RequisitosCompilacion

Write-Host "[2/4] Compilando en release (la primera vez tarda unos minutos)..." -ForegroundColor Yellow
Invoke-Compilacion
$ejecutableCompilado = Join-Path (Get-CarpetaRelease) $EJECUTABLE
if (-not (Test-Path $ejecutableCompilado)) { throw "No se ha generado el ejecutable en $ejecutableCompilado" }

Write-Host "[3/4] Montando el paquete..." -ForegroundColor Yellow
Clear-CarpetaDestino -Ruta $Destino
Copy-ContenidoPortable -Ruta $Destino -RutaEjecutable $ejecutableCompilado

Write-Host "[4/4] Comprobando el paquete..." -ForegroundColor Yellow
$imprescindibles = @(
    $EJECUTABLE,
    "LEEME.txt",
    $CARPETA_DATOS,
    "$CARPETA_DOCUMENTACION\Manual de uso.md",
    "$CARPETA_DOCUMENTACION\Resolucion de problemas.md",
    "$CARPETA_DOCUMENTACION\Referencia de funcionalidades.md"
)
$faltan = $imprescindibles | Where-Object { -not (Test-Path (Join-Path $Destino $_)) }
if ($faltan.Count -gt 0) {
    throw "El paquete esta incompleto. Faltan: $($faltan -join ', ')"
}
Write-Host "      $($imprescindibles.Count) elementos imprescindibles presentes" -ForegroundColor Green

$tamanoMb = [math]::Round((Get-Item (Join-Path $Destino $EJECUTABLE)).Length / 1MB, 1)
Write-Host ""
Write-Host "=== Compilacion completada ===" -ForegroundColor Green
Write-Host "  Paquete   : $Destino"
Write-Host "  Ejecutable: $EJECUTABLE ($tamanoMb MB)"
Write-Host "  Para arrancar: doble clic en $EJECUTABLE"
