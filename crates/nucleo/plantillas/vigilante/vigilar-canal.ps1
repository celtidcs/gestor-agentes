# Vigilante del canal de Gestor Agentes. Gestor Agentes reescribe este archivo en cada lanzamiento.
#
# Uso: powershell -NoProfile -ExecutionPolicy Bypass -File .gestor/vigilante/vigilar-canal.ps1 [-UnaVez] [-Agente "Claude 2"]
#
# Sin -UnaVez escribe una linea "CANAL: ha cambiado <archivo>" por cada cambio en .gestor/canal/
# y no termina nunca: es para el Monitor de Claude Code o cualquier vigilante en segundo plano.
# Con -UnaVez compara con la pasada anterior, escribe los cambios o "CANAL: sin cambios" y
# termina. Las dos guardan su referencia en la carpeta propia del agente (canal-visto.txt y la
# carpeta canal-visto, con una copia de cada archivo).
# Etiquetas: [ENCARGO PARA <identidad>] si una linea nueva dice "Para <identidad>", y
# [MUTACION VIVA] si cambia estado.md y su apartado "## Mutaciones vivas" no dice "Ninguna.".
# No se vigilan la bandeja propia, la guia de Programator ni los historicos del canal.
# La identidad sale de GESTOR_AGENTE o de -Agente.
# Solo lleva caracteres ASCII, para que Windows PowerShell 5.1 lo lea igual sin marca BOM.
param(
    [switch]$UnaVez,
    [string]$Agente = $env:GESTOR_AGENTE
)

# Segundos entre dos comprobaciones en modo continuo: un mensaje se ve en menos de ese tiempo
# sin cargar el equipo, porque cada pasada solo calcula la suma de unos pocos archivos pequenos.
$IntervaloSegundos = 10

if ([string]::IsNullOrWhiteSpace($Agente)) {
    [Console]::Error.WriteLine('VIGILANTE: GESTOR_AGENTE esta vacia; pasa tu identidad con -Agente "Claude 2"')
    exit 2
}
$partes = @($Agente.Trim() -split '\s+')
$herramienta = $partes[0].ToLowerInvariant()
$numero = 1
if ($partes.Count -gt 1) {
    if (-not [int]::TryParse($partes[$partes.Count - 1], [ref]$numero) -or $numero -lt 1) {
        [Console]::Error.WriteLine("VIGILANTE: identidad no valida: $Agente")
        exit 2
    }
}
if ($herramienta -notmatch '^[a-z0-9]+$') {
    [Console]::Error.WriteLine("VIGILANTE: identidad no valida: $Agente")
    exit 2
}
$nombre = $herramienta
if ($numero -gt 1) { $nombre = "$herramienta$numero" }
$identidad = "$($partes[0]) $numero"
$patronDeEncargo = '(?i)(^|[^\p{L}\p{N}])Para ' + [regex]::Escape($identidad) + '([^\p{L}\p{N}]|$)'

$gestor = Split-Path -Parent $PSScriptRoot
$canal = Join-Path $gestor 'canal'
$propia = "$nombre.md"
$carpeta = Join-Path $gestor $nombre
$estado = Join-Path $carpeta 'canal-visto.txt'
$copias = Join-Path $carpeta 'canal-visto'

# Una linea por archivo del canal que se vigila: nombre y suma de su contenido. Un archivo que
# otro agente esta escribiendo en ese momento se salta y se ve en la pasada siguiente.
function Get-Foto {
    $lineas = New-Object System.Collections.Generic.List[string]
    foreach ($archivo in @(Get-ChildItem -LiteralPath $canal -Filter '*.md' -File | Sort-Object Name)) {
        if ($archivo.Name -ieq $propia) { continue }
        if ($archivo.Name -ieq 'COMO-ENCARGAR-A-PROGRAMATOR.md' -or $archivo.Name -like 'hist*rico-*') { continue }
        try {
            $suma = (Get-FileHash -LiteralPath $archivo.FullName -Algorithm SHA256 -ErrorAction Stop).Hash
            $lineas.Add("$($archivo.Name) $suma")
        } catch {
            continue
        }
    }
    return , $lineas.ToArray()
}

# Nombres de los archivos de $Ahora cuya linea no estaba en $Antes.
function Get-Cambios([string[]]$Antes, [string[]]$Ahora) {
    $vistos = @{}
    foreach ($linea in $Antes) { $vistos[$linea] = $true }
    $cambios = New-Object System.Collections.Generic.List[string]
    foreach ($linea in $Ahora) {
        if (-not $vistos.ContainsKey($linea)) { $cambios.Add(($linea -split ' ')[0]) }
    }
    return , $cambios.ToArray()
}

# Lineas de un archivo, o ninguna si no se puede leer.
function Get-Lineas([string]$Ruta) {
    if (-not (Test-Path -LiteralPath $Ruta)) { return , @() }
    try {
        return , @(Get-Content -LiteralPath $Ruta -Encoding UTF8 -ErrorAction Stop)
    } catch {
        return , @()
    }
}

# Si el apartado "## Mutaciones vivas" dice algo distinto de "Ninguna.".
function Test-MutacionesVivas([string[]]$Lineas) {
    $dentro = $false
    foreach ($linea in $Lineas) {
        if ($linea -ceq '## Mutaciones vivas') { $dentro = $true; continue }
        if ($dentro -and $linea.StartsWith('#')) { $dentro = $false }
        if ($dentro -and $linea.Trim() -ne '' -and $linea -cne 'Ninguna.') { return $true }
    }
    return $false
}

# Etiquetas de un archivo cambiado, comparado con su copia de la pasada anterior.
function Get-Etiquetas([string]$Nombre) {
    $actual = Get-Lineas (Join-Path $canal $Nombre)
    $viejas = New-Object 'System.Collections.Generic.HashSet[string]'
    foreach ($linea in (Get-Lineas (Join-Path $copias $Nombre))) { [void]$viejas.Add($linea) }
    $etiquetas = ''
    foreach ($linea in $actual) {
        if (-not $viejas.Contains($linea) -and $linea -match $patronDeEncargo) {
            $etiquetas = " [ENCARGO PARA $identidad]"
            break
        }
    }
    if ($Nombre -ieq 'estado.md' -and (Test-MutacionesVivas $actual)) { $etiquetas += ' [MUTACION VIVA]' }
    return $etiquetas
}

# Copia un archivo del canal como referencia de la pasada siguiente.
function Save-Copia([string]$Nombre) {
    try {
        Copy-Item -LiteralPath (Join-Path $canal $Nombre) -Destination (Join-Path $copias $Nombre) -Force -ErrorAction Stop
    } catch {
        return
    }
}

# Avisa de cada archivo cambiado, con sus etiquetas, y guarda su copia.
function Write-Avisos([string[]]$Antes, [string[]]$Ahora) {
    $cambios = Get-Cambios $Antes $Ahora
    foreach ($cambio in $cambios) {
        Write-Output "CANAL: ha cambiado .gestor/canal/$cambio$(Get-Etiquetas $cambio)"
        Save-Copia $cambio
    }
    [Console]::Out.Flush()
}

New-Item -ItemType Directory -Force -Path $copias | Out-Null

if ($UnaVez) {
    $ahora = Get-Foto
    if (-not (Test-Path -LiteralPath $estado)) {
        Set-Content -LiteralPath $estado -Value $ahora -Encoding UTF8
        foreach ($linea in $ahora) { Save-Copia (($linea -split ' ')[0]) }
        Write-Output 'CANAL: primera pasada, sin referencia anterior; lee todas las bandejas de .gestor/canal/'
        exit 0
    }
    $antes = @(Get-Content -LiteralPath $estado -Encoding UTF8)
    if ((Get-Cambios $antes $ahora).Count -eq 0) {
        Write-Output 'CANAL: sin cambios'
    } else {
        Write-Avisos $antes $ahora
    }
    Set-Content -LiteralPath $estado -Value $ahora -Encoding UTF8
    exit 0
}

$antes = Get-Foto
foreach ($linea in $antes) { Save-Copia (($linea -split ' ')[0]) }
Write-Output "CANAL: vigilando .gestor/canal/ cada $IntervaloSegundos s, sin $propia"
[Console]::Out.Flush()
while ($true) {
    Start-Sleep -Seconds $IntervaloSegundos
    $ahora = Get-Foto
    Write-Avisos $antes $ahora
    $antes = $ahora
}
