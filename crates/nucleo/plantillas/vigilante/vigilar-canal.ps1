# Vigilante del canal de Gestor Agentes. Gestor Agentes reescribe este archivo en cada lanzamiento.
#
# Uso: powershell -NoProfile -ExecutionPolicy Bypass -File .gestor/vigilante/vigilar-canal.ps1 [-UnaVez | -HastaCambio] [-Agente "Claude 2"]
#        [-VidaSegundos N]
#
# Sin opciones escribe una linea "CANAL: ha cambiado <archivo>" por cada cambio en .gestor/canal/
# hasta cumplir su vida maxima (-VidaSegundos): es para el Monitor de Claude Code o cualquier
# vigilante en segundo plano, que lo rearma al terminar.
# Con -UnaVez compara con la pasada anterior, escribe los cambios o "CANAL: sin cambios" y
# termina. Con -HastaCambio compara igual, en el momento y despues cada pocos segundos, y termina
# en cuanto un cambio lleva etiqueta, tras escribir todos: es para las herramientas que despiertan
# al agente cuando termina una tarea en segundo plano. Todas guardan su referencia en la carpeta propia del
# agente (canal-visto.txt y la carpeta canal-visto, con una copia de cada archivo).
# Etiquetas: [ENCARGO PARA <identidad>] si una linea nueva dice "Para <identidad>", y
# [MUTACION VIVA] si cambia estado.md y su apartado "## Mutaciones vivas" no dice "Ninguna.".
# No se vigilan la bandeja propia, la guia de Programator ni los historicos del canal.
# La identidad sale de GESTOR_AGENTE o de -Agente.
# En los dos modos que esperan (continuo y -HastaCambio) hay un solo vigilante por agente: el
# ultimo que arranca anota su marca en vigilante.pid, en la carpeta del agente, y el anterior
# termina al verlo. Tambien terminan si muere alguno de los procesos que los lanzaron, para no
# quedar huerfanos, y al cumplir su vida maxima (-VidaSegundos), para no quedar en bucle sin fin.
# Todo se hace dentro de este proceso, sin lanzar programas en cada pasada.
# Solo lleva caracteres ASCII, para que Windows PowerShell 5.1 lo lea igual sin marca BOM.
param(
    [switch]$UnaVez,
    [switch]$HastaCambio,
    [string]$Agente = $env:GESTOR_AGENTE,
    [string]$VidaSegundos = ''
)

# Segundos entre dos comprobaciones en modo continuo y en -HastaCambio: un mensaje se ve en menos
# de ese tiempo sin cargar el equipo, porque cada pasada solo calcula la suma de unos pocos
# archivos pequenos.
$IntervaloSegundos = 10

# Vida maxima del modo continuo, en segundos. El Monitor de Claude Code caduca como mucho a los
# 1800000 ms (30 minutos) y en Windows no mata el guion al caducar; con 29 minutos el guion
# termina antes que el Monitor mas largo y nunca sigue en bucle sin nadie que lo escuche.
$VidaContinuoSegundos = 1740

# Vida maxima de -HastaCambio, en segundos: una jornada de 8 horas. Es solo un tope de
# seguridad; al cumplirlo despierta al agente una vez, sin nada que atender, y este lo relanza.
$VidaHastaCambioSegundos = 28800

# Cuantos procesos ascendientes se vigilan: el que lanzo el guion (la envoltura de la orden), la
# herramienta del agente y uno mas, porque algunas herramientas ponen un proceso intermedio.
$AscendientesVigilados = 3

if ($UnaVez -and $HastaCambio) {
    [Console]::Error.WriteLine('VIGILANTE: -UnaVez y -HastaCambio no se pueden usar a la vez')
    exit 2
}

if ($VidaSegundos -eq '') {
    $VidaSegundos = if ($HastaCambio) { "$VidaHastaCambioSegundos" } else { "$VidaContinuoSegundos" }
}
$vida = 0
if ($VidaSegundos -notmatch '^[1-9][0-9]*$' -or -not [int]::TryParse($VidaSegundos, [ref]$vida)) {
    [Console]::Error.WriteLine("VIGILANTE: -VidaSegundos necesita un entero positivo: $VidaSegundos")
    exit 2
}

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

# Una pasada contra la referencia guardada: escribe los avisos si hay cambios, o el de primera
# pasada si no habia referencia, y guarda la referencia nueva. Deja en $script:HuboAviso si ha
# escrito algo; no devuelve nada, porque en PowerShell lo que escribe una funcion es su resultado.
function Invoke-Pasada {
    $ahora = Get-Foto
    if (-not (Test-Path -LiteralPath $estado)) {
        Set-Content -LiteralPath $estado -Value $ahora -Encoding UTF8
        foreach ($linea in $ahora) { Save-Copia (($linea -split ' ')[0]) }
        Write-Output 'CANAL: primera pasada, sin referencia anterior; lee todas las bandejas de .gestor/canal/'
        $script:HuboAviso = $true
        return
    }
    $antes = @(Get-Content -LiteralPath $estado -Encoding UTF8)
    $script:HuboAviso = (Get-Cambios $antes $ahora).Count -gt 0
    if ($script:HuboAviso) { Write-Avisos $antes $ahora }
    # Solo se reescribe si la foto ha cambiado (tambien si falta un archivo).
    if (($antes -join "`n") -cne ($ahora -join "`n")) {
        Set-Content -LiteralPath $estado -Value $ahora -Encoding UTF8
    }
}

New-Item -ItemType Directory -Force -Path $copias | Out-Null

if ($UnaVez) {
    Invoke-Pasada
    if (-not $script:HuboAviso) { Write-Output 'CANAL: sin cambios' }
    exit 0
}

# Si algun archivo cambiado lleva etiqueta: un encargo para el agente o una mutacion viva.
function Test-Etiquetado([string[]]$Cambios) {
    foreach ($cambio in $Cambios) {
        if (Get-Etiquetas $cambio) { return $true }
    }
    return $false
}

# Marca de este vigilante: releva a cualquier otro del mismo agente. Lleva un numero al azar
# porque el PID de PowerShell puede repetir el de un vigilante de bash del mismo agente.
$marcador = Join-Path $carpeta 'vigilante.pid'
$marca = "powershell $PID $(Get-Random)"

# Contenido del marcador, o nada si no se puede leer en este momento.
function Get-Marcador {
    try {
        return ([IO.File]::ReadAllText($marcador)).Trim()
    } catch {
        return ''
    }
}

# Borra el marcador si sigue siendo el de este vigilante y termina con codigo 0, tras escribir el
# motivo si lo hay.
function Exit-Vigilante([string]$Motivo) {
    if ($Motivo) { Write-Output $Motivo; [Console]::Out.Flush() }
    if ((Get-Marcador) -ceq $marca) {
        try { [IO.File]::Delete($marcador) } catch { }
    }
    exit 0
}

# Ascendientes vigilados, con su hora de arranque para no confundirlos con otro proceso que
# reciba el mismo numero. Se buscan una sola vez, al arrancar.
$ascendientes = New-Object System.Collections.Generic.List[object]
$proceso = $PID
while ($ascendientes.Count -lt $AscendientesVigilados) {
    $datos = Get-CimInstance Win32_Process -Filter "ProcessId=$proceso" -ErrorAction SilentlyContinue
    if (-not $datos -or $datos.ParentProcessId -le 0) { break }
    $proceso = [int]$datos.ParentProcessId
    try {
        $padre = [Diagnostics.Process]::GetProcessById($proceso)
    } catch {
        break
    }
    $arranque = $null
    try { $arranque = $padre.StartTime } catch { }
    $ascendientes.Add([pscustomobject]@{ Id = $proceso; Arranque = $arranque })
}

# Si sigue vivo cada ascendiente anotado al arrancar. Si no se pudo leer su hora de arranque (un
# proceso de otro usuario o elevado), basta con que exista.
function Test-AscendientesVivos {
    foreach ($ascendiente in $ascendientes) {
        try {
            $vivo = [Diagnostics.Process]::GetProcessById($ascendiente.Id)
        } catch {
            return $false
        }
        $terminado = $false
        try { $terminado = $vivo.HasExited } catch { }
        if ($terminado) { return $false }
        if ($null -ne $ascendiente.Arranque) {
            $arranque = $null
            try { $arranque = $vivo.StartTime } catch { }
            if ($arranque -ne $ascendiente.Arranque) { return $false }
        }
    }
    return $true
}

$finDeVida = (Get-Date).AddSeconds($vida)

# Termina si otro vigilante del agente lo ha relevado, si ha muerto quien lo lanzo o si ha
# cumplido su vida maxima.
function Assert-Seguir {
    if ((Get-Marcador) -cne $marca) {
        Write-Output "VIGILANTE: relevado por otro vigilante de $identidad; no lo rearmes"
        [Console]::Out.Flush()
        exit 0
    }
    if (-not (Test-AscendientesVivos)) {
        Exit-Vigilante 'VIGILANTE: fin porque se cerro quien lo lanzo; relanzalo si sigues trabajando'
    }
    if ((Get-Date) -ge $finDeVida) {
        Exit-Vigilante 'VIGILANTE: fin por tiempo; rearma'
    }
}

# -HastaCambio solo termina con algo para el agente, para no despertarlo por cambios que no le
# tocan. Lo demas no se da por visto: sale en el aviso que lo despierte o en su siguiente pasada.
if ($HastaCambio) {
    if (-not (Test-Path -LiteralPath $estado)) {
        Invoke-Pasada
        exit 0
    }
    [IO.File]::WriteAllText($marcador, "$marca`n")
    while ($true) {
        Assert-Seguir
        $antes = @(Get-Content -LiteralPath $estado -Encoding UTF8)
        $ahora = Get-Foto
        if (Test-Etiquetado (Get-Cambios $antes $ahora)) {
            Write-Avisos $antes $ahora
            Set-Content -LiteralPath $estado -Value $ahora -Encoding UTF8
            Exit-Vigilante ''
        }
        Start-Sleep -Seconds $IntervaloSegundos
    }
}

[IO.File]::WriteAllText($marcador, "$marca`n")
$antes = Get-Foto
foreach ($linea in $antes) { Save-Copia (($linea -split ' ')[0]) }
Write-Output "CANAL: vigilando .gestor/canal/ cada $IntervaloSegundos s, sin $propia, durante $vida s"
[Console]::Out.Flush()
while ($true) {
    Start-Sleep -Seconds $IntervaloSegundos
    Assert-Seguir
    $ahora = Get-Foto
    Write-Avisos $antes $ahora
    $antes = $ahora
}
