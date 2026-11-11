# prueba-portable.ps1 - Barreras del paquete portable de Windows
# ---------------------------------------------------------------------------
# Proyecto : Gestor Agentes 3.0.0
# Proposito: Comprobar mecanicamente que build-portable.ps1 produce el paquete
#            acordado, o que un paquete ya hecho lo cumple.
#            Vive aparte del guion que vigila.
#
# Que comprueba:
#   1. La compilacion termina con codigo 0.
#   2. El paquete contiene exactamente lo acordado, sin nada de .NET.
#   3. La referencia y el manual son copia exacta de features.md y usage.md.
#   4. datos\ existe y esta vacia en un paquete nuevo.
#   5. El ejecutable es PE de 64 bits con subsistema de ventana.
#   6. Los textos no hablan de la 2.x (GestorClaude.exe, .NET, guiones retirados).
#   7. Recompilar conserva datos\ byte a byte y regenera todo lo demas.
#   8. Sin cargo, o sin acompanamientos, el guion falla con un mensaje claro.
#
# Uso: .\prueba-portable.ps1                    compila un paquete de prueba
#      .\prueba-portable.ps1 -Paquete <carpeta>  comprueba ese paquete sin
#                                                compilar: solo lee la
#                                                carpeta, no la cambia.
# Codigo 0 si todo pasa; 1 si algo falla.
# Sin -Paquete escribe solo en <raiz>\target\prueba-portable, que borra al terminar.
# ---------------------------------------------------------------------------

param(
    # Paquete ya hecho que se comprueba sin compilar. Vacio: se compila uno.
    [string]$Paquete = ""
)

$ErrorActionPreference = "Stop"

# --- Constantes ------------------------------------------------------------

$GUION = Join-Path $PSScriptRoot "build-portable.ps1"
$RAIZ = Split-Path -Parent (Split-Path -Parent $PSScriptRoot)
$CARPETA_PRUEBA = Join-Path $RAIZ "target\prueba-portable"

# Contenido exacto acordado del paquete, con rutas relativas.
$CONTENIDO_ESPERADO = @(
    "datos",
    "documentacion",
    "documentacion\Manual de uso.md",
    "documentacion\Referencia de funcionalidades.md",
    "documentacion\Resolucion de problemas.md",
    "gestor-agentes.exe",
    "LEEME.txt"
)

# Documentos del proyecto que el paquete lleva como copia exacta:
# nombre dentro de documentacion\, origen en el proyecto y texto de la barrera.
# La frase de la referencia es la de siempre, para que la salida sin -Paquete
# no cambie.
$COPIAS_EXACTAS = @(
    @{ Nombre = "Referencia de funcionalidades.md"; Origen = "documentation\features.md"
       Frase = "la referencia de funcionalidades es copia exacta de features.md" },
    @{ Nombre = "Manual de uso.md"; Origen = "documentation\usage.md"
       Frase = "el manual de uso es copia exacta de usage.md" }
)

# Carpeta del paquete con los datos del usuario.
$CARPETA_DATOS = "datos"

# Textos de la 2.x que no deben aparecer en el paquete.
$TEXTOS_PROHIBIDOS = @("GestorClaude.exe", ".NET", "Guardar configuracion", "Restaurar configuracion",
    "configuracion-portable", "Gestor Claude 2.")

# Valor del campo Subsystem de la cabecera PE para una aplicacion de ventana
# (IMAGE_SUBSYSTEM_WINDOWS_GUI, especificacion PE/COFF de Microsoft).
$SUBSISTEMA_VENTANA = 2

# Valor del campo Magic de la cabecera opcional en un PE32+ (64 bits).
$MAGIA_PE32_MAS = 0x20B

$script:fallos = 0

# --- Funciones -------------------------------------------------------------

<#
.SYNOPSIS
    Anota el resultado de una comprobacion y lo muestra.
#>
function Confirm-Resultado {
    param([string]$Nombre, [bool]$Correcto, [string]$Detalle = "")
    if ($Correcto) {
        Write-Host "OK     $Nombre" -ForegroundColor Green
    } else {
        $script:fallos++
        Write-Host "FALLO  $Nombre $Detalle" -ForegroundColor Red
    }
}

<#
.SYNOPSIS
    Ejecuta build-portable.ps1 en un proceso aparte y devuelve codigo y salida.
#>
function Invoke-Guion {
    param([string[]]$Argumentos)
    $powershell = Join-Path $env:SystemRoot "System32\WindowsPowerShell\v1.0\powershell.exe"
    # En Windows PowerShell 5.1, la salida de errores redirigida de un proceso
    # hijo llega como errores; con Stop detendria la prueba justo en los casos
    # que esperan un fallo del guion. Aqui solo cuentan el codigo y el texto.
    $preferenciaAnterior = $ErrorActionPreference
    $ErrorActionPreference = "Continue"
    try {
        $salida = & $powershell -NoProfile -ExecutionPolicy Bypass -File $GUION @Argumentos 2>&1 | Out-String
        $codigo = $LASTEXITCODE
    } finally {
        $ErrorActionPreference = $preferenciaAnterior
    }
    return [pscustomobject]@{ Codigo = $codigo; Salida = $salida }
}

<#
.SYNOPSIS
    Lista el contenido de una carpeta con rutas relativas, ordenado.
#>
function Get-ContenidoRelativo {
    param([string]$Carpeta)
    $base = (Resolve-Path $Carpeta).Path.TrimEnd('\') + '\'
    return Get-ChildItem -Path $Carpeta -Recurse -Force |
        ForEach-Object { $_.FullName.Substring($base.Length) } |
        Sort-Object
}

<#
.SYNOPSIS
    Lee de un ejecutable PE el campo Magic y el Subsystem de la cabecera opcional.
.DESCRIPTION
    e_lfanew esta en el desplazamiento 0x3C; la cabecera opcional empieza 24
    bytes despues (firma de 4 y cabecera COFF de 20); Magic esta en su byte 0
    y Subsystem en su byte 68.
#>
function Get-CabeceraPe {
    param([string]$Ruta)
    $bytes = [System.IO.File]::ReadAllBytes($Ruta)
    $inicioPe = [BitConverter]::ToInt32($bytes, 0x3C)
    $opcional = $inicioPe + 24
    return [pscustomobject]@{
        Firma      = [System.Text.Encoding]::ASCII.GetString($bytes, $inicioPe, 4)
        Magia      = [BitConverter]::ToUInt16($bytes, $opcional)
        Subsistema = [BitConverter]::ToUInt16($bytes, $opcional + 68)
    }
}

<#
.SYNOPSIS
    Suma SHA-256 de cada archivo de una carpeta, con su ruta relativa.
#>
function Get-SumasDeCarpeta {
    param([string]$Carpeta)
    $base = (Resolve-Path $Carpeta).Path.TrimEnd('\') + '\'
    $sumas = [ordered]@{}
    Get-ChildItem -Path $Carpeta -Recurse -File -Force | Sort-Object FullName | ForEach-Object {
        $sumas[$_.FullName.Substring($base.Length)] = (Get-FileHash $_.FullName -Algorithm SHA256).Hash
    }
    return $sumas
}

<#
.SYNOPSIS
    Barreras del contenido de un paquete, comunes al paquete compilado aqui y a
    uno ya hecho.
.PARAMETER EsNuevo
    Verdadero si el paquete se acaba de compilar: entonces datos\ tiene que
    estar vacia. En un paquete ya hecho, datos\ lleva la configuracion del
    usuario y solo se exige que exista.
#>
function Test-BarrerasDelPaquete {
    param([string]$Ruta, [bool]$EsNuevo)

    # Contenido exacto. En un paquete ya hecho no cuenta lo que hay dentro de datos\.
    $contenido = @(Get-ContenidoRelativo $Ruta | Where-Object { $EsNuevo -or ($_ -notlike "$CARPETA_DATOS\*") })
    $diferencias = Compare-Object -ReferenceObject $CONTENIDO_ESPERADO -DifferenceObject $contenido
    Confirm-Resultado "el paquete tiene exactamente el contenido acordado" ($null -eq $diferencias) ($diferencias | Out-String)

    foreach ($documento in $COPIAS_EXACTAS) {
        $copia = Join-Path $Ruta "documentacion\$($documento.Nombre)"
        $original = Join-Path $RAIZ $documento.Origen
        $igual = (Test-Path $copia) -and ((Get-FileHash $copia).Hash -eq (Get-FileHash $original).Hash)
        Confirm-Resultado $documento.Frase $igual
    }

    $datos = Join-Path $Ruta $CARPETA_DATOS
    if ($EsNuevo) {
        $vacia = (Test-Path $datos -PathType Container) -and (@(Get-ChildItem $datos -Force).Count -eq 0)
        Confirm-Resultado "datos\ existe y esta vacia" $vacia
    } else {
        Confirm-Resultado "datos\ existe" (Test-Path $datos -PathType Container)
    }

    # Un archivo que no es PE puede no tener cabecera que leer: se cuenta como
    # fallo de esta barrera, y las demas siguen.
    try {
        $cabecera = Get-CabeceraPe (Join-Path $Ruta "gestor-agentes.exe")
        $peCorrecto = ($cabecera.Firma -eq "PE`0`0") -and ($cabecera.Magia -eq $MAGIA_PE32_MAS) -and
            ($cabecera.Subsistema -eq $SUBSISTEMA_VENTANA)
        $detallePe = $cabecera | Out-String
    } catch {
        $peCorrecto = $false
        $detallePe = "no se puede leer la cabecera PE: $($_.Exception.Message)"
    }
    Confirm-Resultado "el exe es PE32+ con subsistema de ventana" $peCorrecto $detallePe

    # La referencia puede citar la 2.x en su historia; el manual no.
    $textos = Get-ChildItem $Ruta -Recurse -File -Include *.txt, *.md |
        Where-Object { $_.Name -ne "Referencia de funcionalidades.md" -and $_.FullName -notlike "*\$CARPETA_DATOS\*" }
    $encontrados = foreach ($archivo in $textos) {
        foreach ($prohibido in $TEXTOS_PROHIBIDOS) {
            if ((Get-Content $archivo.FullName -Raw -Encoding UTF8).Contains($prohibido)) { "$($archivo.Name): $prohibido" }
        }
    }
    Confirm-Resultado "los textos del paquete no hablan de la 2.x" ($null -eq $encontrados) ($encontrados -join "; ")
}

<#
.SYNOPSIS
    Modo paquete: comprueba un paquete ya hecho sin compilar
    y sin escribir en el.
#>
function Test-PaqueteHecho {
    param([string]$Ruta)
    if (-not (Test-Path $Ruta -PathType Container)) { throw "No existe el paquete $Ruta" }

    # Que se ha comprobado, y donde.
    $commit = "(sin git)"
    if (Get-Command git -ErrorAction SilentlyContinue) {
        $commit = (& git -C $RAIZ -c "safe.directory=*" rev-parse HEAD 2>$null | Out-String).Trim()
    }
    Write-Host "MODO PAQUETE: $((Resolve-Path $Ruta).Path)"
    Write-Host "MAQUINA: $env:COMPUTERNAME | $([Environment]::OSVersion.VersionString) | $(Get-Date -Format 'yyyy-MM-dd HH:mm:ss')"
    Write-Host "COMMIT DEL ARBOL: $commit"
    $sumasAntes = Get-SumasDeCarpeta $Ruta
    foreach ($archivo in $sumasAntes.Keys) { Write-Host "SHA256 $($sumasAntes[$archivo])  $archivo" }

    Test-BarrerasDelPaquete -Ruta $Ruta -EsNuevo $false

    foreach ($omitida in @("la compilacion termina con codigo 0", "la recompilacion conserva datos\",
            "sin cargo falla", "sin acompanamientos falla")) {
        Write-Host "OMITIDA en modo paquete: $omitida"
    }

    # La carpeta solo se ha leido.
    $sumasDespues = Get-SumasDeCarpeta $Ruta
    $iguales = ($sumasAntes.Count -eq $sumasDespues.Count) -and
        (@($sumasAntes.Keys | Where-Object { $sumasDespues[$_] -ne $sumasAntes[$_] }).Count -eq 0)
    Confirm-Resultado "el paquete no ha cambiado al comprobarlo" $iguales
}

# --- Pruebas ---------------------------------------------------------------

if ($Paquete -ne "") {
    try {
        Test-PaqueteHecho -Ruta $Paquete
    }
    catch {
        $script:fallos++
        Write-Host "FALLO  $($_.Exception.Message)" -ForegroundColor Red
    }
    Write-Host ""
    if ($script:fallos -eq 0) {
        Write-Host "PRUEBA PORTABLE: OK" -ForegroundColor Green
        exit 0
    }
    Write-Host "PRUEBA PORTABLE: $($script:fallos) fallos" -ForegroundColor Red
    exit 1
}

if (Test-Path $CARPETA_PRUEBA) { Remove-Item $CARPETA_PRUEBA -Recurse -Force }
New-Item -ItemType Directory -Path $CARPETA_PRUEBA -Force | Out-Null
$paquete = Join-Path $CARPETA_PRUEBA "portable"

try {
    # 1. Compilacion.
    $resultado = Invoke-Guion @("-Destino", $paquete)
    Confirm-Resultado "la compilacion termina con codigo 0" ($resultado.Codigo -eq 0) $resultado.Salida
    if ($resultado.Codigo -ne 0) { throw "Sin paquete no se pueden hacer las demas comprobaciones." }

    # 2 a 6. Barreras del contenido del paquete.
    Test-BarrerasDelPaquete -Ruta $paquete -EsNuevo $true
    $datos = Join-Path $paquete $CARPETA_DATOS

    # 7. Recompilar conserva datos\.
    $archivoDatos = Join-Path $datos "config.json"
    Set-Content -Path $archivoDatos -Value '{"prueba": "conservar"}' -Encoding UTF8
    $huellaAntes = (Get-FileHash $archivoDatos).Hash
    Set-Content -Path (Join-Path $paquete "residuo.txt") -Value "no debe sobrevivir" -Encoding UTF8
    $resultado = Invoke-Guion @("-Destino", $paquete)
    Confirm-Resultado "la recompilacion termina con codigo 0" ($resultado.Codigo -eq 0) $resultado.Salida
    $conservado = (Test-Path $archivoDatos) -and ((Get-FileHash $archivoDatos).Hash -eq $huellaAntes)
    Confirm-Resultado "recompilar conserva datos\ byte a byte" $conservado
    Confirm-Resultado "recompilar borra lo que no es del paquete" (-not (Test-Path (Join-Path $paquete "residuo.txt")))

    # 8a. Sin cargo.
    $pathAnterior = $env:PATH
    try {
        $env:PATH = Join-Path $env:SystemRoot "System32"
        $resultado = Invoke-Guion @("-Destino", (Join-Path $CARPETA_PRUEBA "sin-cargo"))
    } finally {
        $env:PATH = $pathAnterior
    }
    Confirm-Resultado "sin cargo falla con codigo distinto de 0" ($resultado.Codigo -ne 0)
    Confirm-Resultado "sin cargo el mensaje lo dice" ($resultado.Salida -match "No se encuentra 'cargo'") $resultado.Salida

    # 8b. Sin acompanamientos.
    $resultado = Invoke-Guion @("-Destino", (Join-Path $CARPETA_PRUEBA "sin-extras"),
        "-CarpetaExtras", (Join-Path $CARPETA_PRUEBA "no-existe"))
    Confirm-Resultado "sin acompanamientos falla con codigo distinto de 0" ($resultado.Codigo -ne 0)
    Confirm-Resultado "sin acompanamientos el mensaje lo dice" ($resultado.Salida -match "acompanamientos") $resultado.Salida
}
catch {
    $script:fallos++
    Write-Host "FALLO  $($_.Exception.Message)" -ForegroundColor Red
}
finally {
    if (Test-Path $CARPETA_PRUEBA) { Remove-Item $CARPETA_PRUEBA -Recurse -Force }
}

Write-Host ""
if ($script:fallos -eq 0) {
    Write-Host "PRUEBA PORTABLE: OK" -ForegroundColor Green
    exit 0
}
Write-Host "PRUEBA PORTABLE: $($script:fallos) fallos" -ForegroundColor Red
exit 1
