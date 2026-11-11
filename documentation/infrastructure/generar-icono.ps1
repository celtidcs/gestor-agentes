# generar-icono.ps1 - Generador del icono de la aplicacion Gestor Agentes
# ---------------------------------------------------------------------------
# Proyecto : Gestor Agentes
# Proposito: Producir, sin dependencias externas, el icono de la aplicacion en
#            los dos formatos que necesita el proyecto:
#              1. gestor-claude.ico - icono del ejecutable (Explorador, barra
#                 de tareas, propiedades del archivo). Multi-resolucion.
#              2. gestor-claude.png - icono de la ventana, en Windows y en
#                 Linux.
#
# Formato del .ico: los tamanos hasta 64 px se codifican como BMP DIB de 32 bpp
# (formato clasico, legible por cualquier version de Windows y por
# System.Drawing) y los de 128 y 256 px como PNG, que es lo habitual desde
# Windows Vista para evitar ficheros de varios megabytes.
#
# Uso:
#   .\generar-icono.ps1                          # escribe en crates\interfaz\assets
#   .\generar-icono.ps1 -RutaSalida C:\tmp\a.ico # escribe solo en la ruta indicada
#
# Idempotente: cada ejecucion regenera los mismos archivos deterministamente.
# ---------------------------------------------------------------------------

param(
    # Ruta de un unico .ico de salida. Si se omite, el script escribe el icono
    # en los Assets de la interfaz Avalonia.
    [string]$RutaSalida
)

$ErrorActionPreference = "Stop"
Add-Type -AssemblyName System.Drawing

# --- Constantes de diseno --------------------------------------------------

# Resoluciones incluidas en el contenedor .ico.
$TAMANOS = @(16, 24, 32, 48, 64, 128, 256)

# A partir de este lado los fotogramas se guardan comprimidos en PNG.
$LADO_MINIMO_PNG = 128

# Paleta terracota, coherente con la identidad visual de Claude.
$COLOR_INICIO = [System.Drawing.Color]::FromArgb(255, 232, 141, 103)
$COLOR_FIN    = [System.Drawing.Color]::FromArgb(255, 193,  86,  55)
$COLOR_LETRA  = [System.Drawing.Color]::FromArgb(255, 255, 252, 249)

# Proporciones del dibujo respecto al lado del icono.
$PROPORCION_RADIO_ESQUINA = 0.22
$PROPORCION_MARGEN        = 0.03
$PROPORCION_FUENTE        = 0.62

<#
.SYNOPSIS
    Dibuja el icono a un tamano concreto.
.PARAMETER Lado
    Lado del icono en pixeles (el icono siempre es cuadrado).
.OUTPUTS
    System.Drawing.Bitmap de 32 bpp con canal alfa. El llamante debe liberarlo.
#>
function New-BitmapIcono {
    param([Parameter(Mandatory = $true)][int]$Lado)

    $bmp = New-Object System.Drawing.Bitmap($Lado, $Lado, [System.Drawing.Imaging.PixelFormat]::Format32bppArgb)
    $g = [System.Drawing.Graphics]::FromImage($bmp)
    $g.SmoothingMode     = [System.Drawing.Drawing2D.SmoothingMode]::AntiAlias
    $g.TextRenderingHint = [System.Drawing.Text.TextRenderingHint]::AntiAliasGridFit
    $g.Clear([System.Drawing.Color]::Transparent)

    # Cuadrado de esquinas redondeadas, construido con cuatro arcos.
    $margen = [Math]::Max(0, [int]($Lado * $PROPORCION_MARGEN))
    $lienzo = $Lado - (2 * $margen)
    $diametro = [Math]::Max(2, [int]($Lado * $PROPORCION_RADIO_ESQUINA)) * 2

    $ruta = New-Object System.Drawing.Drawing2D.GraphicsPath
    $ruta.AddArc($margen, $margen, $diametro, $diametro, 180, 90)
    $ruta.AddArc($margen + $lienzo - $diametro, $margen, $diametro, $diametro, 270, 90)
    $ruta.AddArc($margen + $lienzo - $diametro, $margen + $lienzo - $diametro, $diametro, $diametro, 0, 90)
    $ruta.AddArc($margen, $margen + $lienzo - $diametro, $diametro, $diametro, 90, 90)
    $ruta.CloseFigure()

    $rectFondo = New-Object System.Drawing.Rectangle($margen, $margen, $lienzo, $lienzo)
    $brocha = New-Object System.Drawing.Drawing2D.LinearGradientBrush(
        $rectFondo, $COLOR_INICIO, $COLOR_FIN,
        [System.Drawing.Drawing2D.LinearGradientMode]::ForwardDiagonal)
    $g.FillPath($brocha, $ruta)

    # Inicial "G" de Gestor, centrada opticamente (ligeramente subida).
    $fuente = New-Object System.Drawing.Font(
        "Segoe UI", [single]($Lado * $PROPORCION_FUENTE),
        [System.Drawing.FontStyle]::Bold, [System.Drawing.GraphicsUnit]::Pixel)
    $formato = New-Object System.Drawing.StringFormat
    $formato.Alignment     = [System.Drawing.StringAlignment]::Center
    $formato.LineAlignment = [System.Drawing.StringAlignment]::Center
    $rectTexto = New-Object System.Drawing.RectangleF(0, [single]($Lado * -0.02), [single]$Lado, [single]$Lado)
    $pincel = New-Object System.Drawing.SolidBrush($COLOR_LETRA)
    $g.DrawString("G", $fuente, $pincel, $rectTexto, $formato)

    $pincel.Dispose(); $formato.Dispose(); $fuente.Dispose()
    $brocha.Dispose(); $ruta.Dispose(); $g.Dispose()
    return $bmp
}

<#
.SYNOPSIS
    Codifica un mapa de bits como PNG.
.OUTPUTS
    Array de bytes con el PNG completo.
#>
function ConvertTo-BytesPng {
    param([Parameter(Mandatory = $true)][System.Drawing.Bitmap]$Bitmap)

    $flujo = New-Object System.IO.MemoryStream
    $Bitmap.Save($flujo, [System.Drawing.Imaging.ImageFormat]::Png)
    $bytes = $flujo.ToArray()
    $flujo.Dispose()
    # La coma impide que PowerShell desenrolle el array al devolverlo.
    return ,([byte[]]$bytes)
}

<#
.SYNOPSIS
    Codifica un mapa de bits como DIB de icono (formato clasico del .ico).
.DESCRIPTION
    Un fotograma DIB dentro de un .ico se compone de:
      - BITMAPINFOHEADER de 40 bytes, con biHeight al doble del alto real
        porque la estructura describe la imagen XOR y la mascara AND juntas.
      - Pixeles BGRA de abajo arriba (bottom-up).
      - Mascara AND de 1 bpp, con las filas alineadas a 4 bytes. Se deja a cero:
        la transparencia real la aporta el canal alfa de 32 bpp.
.OUTPUTS
    Array de bytes con el fotograma DIB.
#>
function ConvertTo-BytesDib {
    param([Parameter(Mandatory = $true)][System.Drawing.Bitmap]$Bitmap)

    $ancho = $Bitmap.Width
    $alto  = $Bitmap.Height

    # Volcado directo de los pixeles en BGRA, evitando GetPixel por rendimiento.
    $rect = New-Object System.Drawing.Rectangle(0, 0, $ancho, $alto)
    $datosBloqueados = $Bitmap.LockBits($rect, [System.Drawing.Imaging.ImageLockMode]::ReadOnly,
        [System.Drawing.Imaging.PixelFormat]::Format32bppArgb)
    $bytesPixeles = New-Object byte[] ($datosBloqueados.Stride * $alto)
    [System.Runtime.InteropServices.Marshal]::Copy($datosBloqueados.Scan0, $bytesPixeles, 0, $bytesPixeles.Length)
    $paso = $datosBloqueados.Stride
    $Bitmap.UnlockBits($datosBloqueados)

    # Cada fila de la mascara AND se alinea a 4 bytes.
    $bytesMascaraPorFila = [int]([Math]::Floor(($ancho + 31) / 32)) * 4

    $flujo = New-Object System.IO.MemoryStream
    $escritor = New-Object System.IO.BinaryWriter($flujo)

    # BITMAPINFOHEADER
    $escritor.Write([UInt32]40)                    # biSize
    $escritor.Write([Int32]$ancho)                 # biWidth
    $escritor.Write([Int32]($alto * 2))            # biHeight (imagen + mascara)
    $escritor.Write([UInt16]1)                     # biPlanes
    $escritor.Write([UInt16]32)                    # biBitCount
    $escritor.Write([UInt32]0)                     # biCompression = BI_RGB
    $escritor.Write([UInt32]($ancho * $alto * 4))  # biSizeImage
    $escritor.Write([Int32]0)                      # biXPelsPerMeter
    $escritor.Write([Int32]0)                      # biYPelsPerMeter
    $escritor.Write([UInt32]0)                     # biClrUsed
    $escritor.Write([UInt32]0)                     # biClrImportant

    # Pixeles, invirtiendo el orden de las filas (el DIB es bottom-up).
    $fila = New-Object byte[] ($ancho * 4)
    for ($y = $alto - 1; $y -ge 0; $y--) {
        [Array]::Copy($bytesPixeles, $y * $paso, $fila, 0, $ancho * 4)
        $escritor.Write($fila, 0, $fila.Length)
    }

    # Mascara AND a ceros (todos los pixeles opacos segun la mascara).
    $mascara = New-Object byte[] ($bytesMascaraPorFila * $alto)
    $escritor.Write($mascara, 0, $mascara.Length)

    $escritor.Flush()
    $bytes = $flujo.ToArray()
    $escritor.Dispose(); $flujo.Dispose()
    return ,([byte[]]$bytes)
}

<#
.SYNOPSIS
    Ensambla los fotogramas en un contenedor .ico y lo escribe en disco.
.PARAMETER Fotogramas
    Lista de objetos con dos propiedades: Lado (pixeles) y Datos (bytes ya
    codificados del fotograma). Se usa una lista y no un diccionario porque
    OrderedDictionary interpreta un indice entero como posicion, no como clave.
.PARAMETER Ruta
    Ruta completa del archivo .ico a escribir.
#>
function Write-ArchivoIco {
    param(
        [Parameter(Mandatory = $true)][object[]]$Fotogramas,
        [Parameter(Mandatory = $true)][string]$Ruta
    )

    $flujo = New-Object System.IO.MemoryStream
    $escritor = New-Object System.IO.BinaryWriter($flujo)

    # ICONDIR
    $escritor.Write([UInt16]0)                       # Reservado, siempre 0
    $escritor.Write([UInt16]1)                       # Tipo: 1 = icono
    $escritor.Write([UInt16]$Fotogramas.Count)       # Numero de imagenes

    # Una ICONDIRENTRY de 16 bytes por fotograma.
    $desplazamiento = 6 + (16 * $Fotogramas.Count)
    foreach ($fotograma in $Fotogramas) {
        $datos = [byte[]]$fotograma.Datos
        # El ancho y el alto ocupan un byte: 256 se codifica como 0.
        $dimension = 0
        if ([int]$fotograma.Lado -lt 256) { $dimension = [int]$fotograma.Lado }
        $escritor.Write([Byte]$dimension)            # bWidth
        $escritor.Write([Byte]$dimension)            # bHeight
        $escritor.Write([Byte]0)                     # bColorCount (0 = sin paleta)
        $escritor.Write([Byte]0)                     # bReserved
        $escritor.Write([UInt16]1)                   # wPlanes
        $escritor.Write([UInt16]32)                  # wBitCount
        $escritor.Write([UInt32]$datos.Length)       # dwBytesInRes
        $escritor.Write([UInt32]$desplazamiento)     # dwImageOffset
        $desplazamiento += $datos.Length
    }

    foreach ($fotograma in $Fotogramas) {
        $datos = [byte[]]$fotograma.Datos
        # Sobrecarga explicita (buffer, indice, cuenta): Write($array) a secas
        # resolveria a Write(Byte) y escribiria un unico byte.
        $escritor.Write($datos, 0, $datos.Length)
    }

    $escritor.Flush()
    [System.IO.File]::WriteAllBytes($Ruta, $flujo.ToArray())
    $escritor.Dispose(); $flujo.Dispose()
}

# --- Programa principal ----------------------------------------------------

# El script vive en documentation/infrastructure/, dos niveles bajo la raiz.
$raizProyecto = Split-Path -Parent (Split-Path -Parent $PSScriptRoot)

# Destinos por defecto: los recursos del crate interfaz.
if ([string]::IsNullOrWhiteSpace($RutaSalida)) {
    $destinosIco = @(
        (Join-Path $raizProyecto "crates\interfaz\assets\gestor-claude.ico")
    )
    $destinosPng = @(
        (Join-Path $raizProyecto "crates\interfaz\assets\gestor-claude.png")
    )
} else {
    $destinosIco = @($RutaSalida)
    $destinosPng = @()
}

Write-Host "Generando fotogramas del icono..." -ForegroundColor Cyan

$fotogramas = New-Object System.Collections.Generic.List[object]
$bitmapGrande = $null
foreach ($lado in $TAMANOS) {
    $bmp = New-BitmapIcono -Lado $lado
    if ($lado -ge $LADO_MINIMO_PNG) {
        $datos = ConvertTo-BytesPng -Bitmap $bmp
    } else {
        $datos = ConvertTo-BytesDib -Bitmap $bmp
    }
    $fotogramas.Add([pscustomobject]@{ Lado = $lado; Datos = [byte[]]$datos })
    # El fotograma de 256 px se reutiliza como PNG del icono de ventana.
    if ($lado -eq 256) { $bitmapGrande = $bmp } else { $bmp.Dispose() }
    Write-Host ("  {0,3}x{1,-3} -> {2,7} bytes" -f $lado, $lado, $datos.Length)
}

foreach ($destino in $destinosIco) {
    $carpeta = Split-Path -Parent $destino
    if (-not (Test-Path $carpeta)) { New-Item -ItemType Directory -Path $carpeta -Force | Out-Null }
    Write-ArchivoIco -Fotogramas $fotogramas -Ruta $destino
    Write-Host "  ICO escrito: $destino ($((Get-Item $destino).Length) bytes)" -ForegroundColor Green
}

foreach ($destino in $destinosPng) {
    $carpeta = Split-Path -Parent $destino
    if (-not (Test-Path $carpeta)) { New-Item -ItemType Directory -Path $carpeta -Force | Out-Null }
    $bitmapGrande.Save($destino, [System.Drawing.Imaging.ImageFormat]::Png)
    Write-Host "  PNG escrito: $destino ($((Get-Item $destino).Length) bytes)" -ForegroundColor Green
}

if ($null -ne $bitmapGrande) { $bitmapGrande.Dispose() }
Write-Host "Icono generado correctamente." -ForegroundColor Green
