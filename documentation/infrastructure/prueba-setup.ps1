# prueba-setup.ps1 - Barreras de setup.ps1 y puertas-rust.ps1 en Windows.
#
# Vive aparte de los guiones que vigila. Comprueba:
#   1. puertas-rust.ps1 ejecuta exactamente las cuatro puertas y termina con código 0, y con
#      una puerta en rojo termina con un código distinto de 0.
#   2. setup.ps1 -SinPuertas termina con código 0 y deja portable\ con el ejecutable y datos\.
#   3. Sin cargo en el PATH, setup.ps1 y puertas-rust.ps1 fallan y remiten a https://rustup.rs.
#
# Uso: .\prueba-setup.ps1 [-CarpetaCompilacion <carpeta>]   (código 0 si todo pasa; 1 si no)
# Regenera portable\ en la raíz del proyecto, igual que setup.ps1, y conserva su carpeta datos\.

param(
    # Se pasa tal cual a puertas-rust.ps1 (--target-dir).
    [string]$CarpetaCompilacion = ""
)

$ErrorActionPreference = "Stop"

$RAIZ = Split-Path -Parent (Split-Path -Parent $PSScriptRoot)
$POWERSHELL = Join-Path $env:SystemRoot "System32\WindowsPowerShell\v1.0\powershell.exe"

# Encabezados que escribe puertas-rust.ps1, uno por puerta, en este orden.
$PUERTAS_ESPERADAS = @(
    "=== Puerta: Formato ===",
    "=== Puerta: Linter (clippy sin avisos) ===",
    "=== Puerta: Pruebas ===",
    "=== Puerta: Auditoría de dependencias ==="
)

$script:fallos = 0

function Confirm-Resultado {
    param([string]$Nombre, [bool]$Correcto, [string]$Detalle = "")
    if ($Correcto) {
        Write-Host "OK     $Nombre" -ForegroundColor Green
    } else {
        $script:fallos++
        Write-Host "FALLO  $Nombre $Detalle" -ForegroundColor Red
    }
}

# Tiempo máximo que se espera a cada guion hijo. Solo existe como red de seguridad para que un
# cuelgue imprevisto no deje la barrera esperando indefinidamente. Una hora es una holgura
# amplia frente a lo que tarda una compilación en frío: unos 232 s el workspace de pruebas en
# una máquina Windows de 8 núcleos.
$TIEMPO_MAXIMO_SEGUNDOS = 3600

# Archivos donde cada guion hijo deja su salida, dentro de target\, que git ignora.
$SALIDA_DEL_HIJO = Join-Path $RAIZ "target\prueba-setup-salida.txt"
$ERRORES_DEL_HIJO = Join-Path $RAIZ "target\prueba-setup-errores.txt"

<#
.SYNOPSIS
    Indica si un guion terminó por sí mismo con un código de error. Un tiempo agotado (código
    nulo) no cuenta: un cuelgue no debe pasar nunca por el fallo que se esperaba.
#>
function Test-FalloReal($Resultado) {
    return ($null -ne $Resultado.Codigo) -and ($Resultado.Codigo -ne 0)
}

<#
.SYNOPSIS
    Termina un proceso y todos sus descendientes.
#>
function Stop-Arbol([int]$Id) {
    Get-CimInstance Win32_Process -Filter "ParentProcessId = $Id" | ForEach-Object { Stop-Arbol $_.ProcessId }
    Stop-Process -Id $Id -Force -ErrorAction SilentlyContinue
}

<#
.SYNOPSIS
    Ejecuta un guion en un proceso aparte y devuelve su código y su salida como líneas.
.DESCRIPTION
    El hijo se lanza con Start-Process, con su salida en archivos, y se espera solo a ese proceso
    con WaitForExit. No se lee por una tubería porque, si un proceso que lanza el hijo (como un
    servidor de cargo) hereda la tubería y no la cierra, la lectura no termina nunca. Si se agota
    $TIEMPO_MAXIMO_SEGUNDOS, se termina el hijo con sus descendientes y el código es nulo.
    Start-Process une los argumentos con espacios sin entrecomillar, así que cada uno va entre
    comillas dobles: si no, se partirían las rutas con espacios y se perderían los vacíos.
    La salida redirigida de Windows PowerShell 5.1 sale en la página de códigos OEM de la consola,
    y se lee con esa misma codificación.
#>
function Invoke-Guion {
    param([string]$Guion, [string[]]$Argumentos)
    New-Item -ItemType Directory -Path (Split-Path $SALIDA_DEL_HIJO) -Force | Out-Null
    $lista = @("-NoProfile", "-ExecutionPolicy", "Bypass", "-File", (Join-Path $PSScriptRoot $Guion)) + $Argumentos
    $entrecomillados = $lista | ForEach-Object { '"' + ($_ -replace '"', '\"') + '"' }
    $proceso = Start-Process -FilePath $POWERSHELL -ArgumentList $entrecomillados -NoNewWindow -PassThru `
        -RedirectStandardOutput $SALIDA_DEL_HIJO -RedirectStandardError $ERRORES_DEL_HIJO
    # Leer Handle antes de esperar hace que el objeto conserve el código de salida al terminar.
    $null = $proceso.Handle
    $lineas = @()
    if ($proceso.WaitForExit($TIEMPO_MAXIMO_SEGUNDOS * 1000)) {
        $codigo = $proceso.ExitCode
    } else {
        Stop-Arbol $proceso.Id
        $codigo = $null
        $lineas += "TIEMPO AGOTADO: $Guion no terminó en $TIEMPO_MAXIMO_SEGUNDOS s y se ha detenido"
    }
    foreach ($archivo in $SALIDA_DEL_HIJO, $ERRORES_DEL_HIJO) {
        if (Test-Path $archivo) {
            $lineas += @(Get-Content -Path $archivo -Encoding Oem)
            Remove-Item $archivo -Force
        }
    }
    return [pscustomobject]@{ Codigo = $codigo; Lineas = $lineas }
}

try {
    # 1. Las cuatro puertas, exactamente.
    $r = Invoke-Guion "puertas-rust.ps1" @("-CarpetaCompilacion", $CarpetaCompilacion)
    $encabezados = @($r.Lineas | Where-Object { $_ -like "=== Puerta: *" })
    $diferencias = Compare-Object -ReferenceObject $PUERTAS_ESPERADAS -DifferenceObject $encabezados -SyncWindow 0
    Confirm-Resultado "puertas-rust ejecuta exactamente las cuatro puertas" ($null -eq $diferencias) ($encabezados -join "; ")
    Confirm-Resultado "puertas-rust termina con código 0" ($r.Codigo -eq 0) (($r.Lineas | Select-Object -Last 15) -join "`n")

    # 1b. Con una puerta en rojo, puertas-rust falla. Se fuerza pasando como carpeta de
    # compilación una subcarpeta dentro de un archivo, donde cargo no puede crear una carpeta
    # (en Windows, si la ruta es un archivo suelto, cargo lo sustituye por una carpeta y pasa;
    # dentro de un archivo el sistema operativo rechaza la creación con error de ruta).
    # La limpieza usa -Recurse -Force para que nunca pida confirmación interactiva.
    $archivo = Join-Path $RAIZ "target\prueba-setup-no-es-carpeta"
    New-Item -ItemType Directory -Path (Split-Path $archivo) -Force | Out-Null
    Set-Content -Path $archivo -Value "no es una carpeta" -Encoding ASCII
    $subcarpeta = Join-Path $archivo "sub"
    try {
        $r = Invoke-Guion "puertas-rust.ps1" @("-CarpetaCompilacion", $subcarpeta)
    } finally {
        Remove-Item $archivo -Recurse -Force -ErrorAction SilentlyContinue
    }
    Confirm-Resultado "puertas-rust con una puerta en rojo termina con código distinto de 0" (Test-FalloReal $r) ($r.Lineas -join "`n")

    # 2. setup sin puertas. Antes se quita el ejecutable de una ejecución anterior, para que la
    # comprobación no pase por un resto; datos\ se deja, porque es del usuario.
    Remove-Item (Join-Path $RAIZ "portable\gestor-agentes.exe") -Force -ErrorAction SilentlyContinue
    $r = Invoke-Guion "setup.ps1" @("-SinPuertas")
    Confirm-Resultado "setup -SinPuertas termina con código 0" ($r.Codigo -eq 0) (($r.Lineas | Select-Object -Last 15) -join "`n")
    $portable = Join-Path $RAIZ "portable"
    Confirm-Resultado "setup deja el ejecutable en portable\" (Test-Path (Join-Path $portable "gestor-agentes.exe"))
    Confirm-Resultado "setup deja la carpeta portable\datos" (Test-Path (Join-Path $portable "datos") -PathType Container)

    # 3. Sin cargo.
    $pathAnterior = $env:PATH
    try {
        $env:PATH = Join-Path $env:SystemRoot "System32"
        $setup = Invoke-Guion "setup.ps1" @("-SinPuertas")
        $puertas = Invoke-Guion "puertas-rust.ps1" @()
    } finally {
        $env:PATH = $pathAnterior
    }
    Confirm-Resultado "setup sin cargo falla" (Test-FalloReal $setup)
    Confirm-Resultado "setup sin cargo remite a rustup" (($setup.Lineas -join "`n") -match "rustup\.rs")
    Confirm-Resultado "puertas-rust sin cargo falla" (Test-FalloReal $puertas)
    Confirm-Resultado "puertas-rust sin cargo remite a rustup" (($puertas.Lineas -join "`n") -match "rustup\.rs")
}
catch {
    $script:fallos++
    Write-Host "FALLO  $($_.Exception.Message)" -ForegroundColor Red
}

Write-Host ""
if ($script:fallos -eq 0) {
    Write-Host "PRUEBA SETUP: OK" -ForegroundColor Green
    exit 0
}
Write-Host "PRUEBA SETUP: $($script:fallos) fallos" -ForegroundColor Red
exit 1
