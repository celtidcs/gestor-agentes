# setup.ps1 - Preparación del entorno de desarrollo de Gestor Agentes 3.0.0 en Windows
# Proyecto : Gestor Agentes 3.0.0 (la reescritura en Rust de Gestor Claude)
# Requisitos: Windows 10 o posterior, PowerShell 5.1 o posterior, Rust (cargo) y git.
#
# Uso, desde la raíz del proyecto:
#   .\documentation\infrastructure\setup.ps1              # comprueba, pasa las puertas y genera el portable
#   .\documentation\infrastructure\setup.ps1 -SinPuertas  # omite las cuatro puertas
#   .\documentation\infrastructure\setup.ps1 -CarpetaCompilacion C:\cache\target
#
# Idempotente: puede ejecutarse tantas veces como se quiera; nada se duplica. No toca la
# configuración de la 2.x en %APPDATA%\GestorClaude ni la carpeta datos\ del portable.

param(
    # Omite las cuatro puertas (formato, linter, pruebas y auditoría) antes de generar el portable.
    [switch]$SinPuertas,

    # Carpeta de compilación de las puertas, que se pasa a cargo con --target-dir (no con
    # CARGO_TARGET_DIR, por sccache). Vacía: la de siempre, target\ en la raíz.
    [string]$CarpetaCompilacion = ""
)

$ErrorActionPreference = "Stop"

# Programas de agente que la aplicación abre en una terminal. Ninguno es obligatorio para
# compilar, pero sin al menos uno la aplicación no tiene nada que abrir: claude (Claude Code),
# codex (Codex CLI) y agy (Gemini con la CLI de Antigravity).
$AGENTES_EN_TERMINAL = @("claude", "codex", "agy")

$raizProyecto = Resolve-Path (Join-Path $PSScriptRoot "..\..")

Write-Host ""
Write-Host "=== Gestor Agentes: preparación del entorno ===" -ForegroundColor Cyan
Write-Host ""

# 1. Requisitos: cargo es obligatorio; cargo-audit, para la cuarta puerta; git, para la aplicación.
Write-Host "[1/4] Verificando requisitos..." -ForegroundColor Yellow
if ($null -eq (Get-Command cargo -ErrorAction SilentlyContinue)) {
    Write-Host "      ERROR: no se encuentra 'cargo'. Instala Rust desde https://rustup.rs y vuelve a abrir la terminal." -ForegroundColor Red
    exit 1
}
Write-Host "      OK: $(& cargo --version)" -ForegroundColor Green
if ($null -eq (Get-Command cargo-audit -ErrorAction SilentlyContinue)) {
    Write-Host "      AVISO: falta cargo-audit, que la cuarta puerta necesita. Instálalo con: cargo install cargo-audit --locked" -ForegroundColor Yellow
} else {
    Write-Host "      OK: cargo-audit" -ForegroundColor Green
}
if ($null -eq (Get-Command git -ErrorAction SilentlyContinue)) {
    Write-Host "      AVISO: git no está instalado. La aplicación no podrá crear repositorios en los proyectos." -ForegroundColor Yellow
} else {
    Write-Host "      OK: $(& git --version)" -ForegroundColor Green
}

# rc.exe (Windows SDK): opcional, solo para incrustar el icono en el .exe
$rc = Get-Command rc.exe -ErrorAction SilentlyContinue | Select-Object -First 1
if ($null -eq $rc) {
    $kitsRoot = (Get-ItemProperty -Path "HKLM:\SOFTWARE\Microsoft\Windows Kits\Installed Roots" -Name "KitsRoot10" -ErrorAction SilentlyContinue).KitsRoot10
    if ($null -eq $kitsRoot) {
        $kitsRoot = (Get-ItemProperty -Path "HKLM:\SOFTWARE\WOW6432Node\Microsoft\Windows Kits\Installed Roots" -Name "KitsRoot10" -ErrorAction SilentlyContinue).KitsRoot10
    }
    if ($null -ne $kitsRoot -and (Test-Path (Join-Path $kitsRoot "bin"))) {
        $rc = Get-ChildItem -Path (Join-Path $kitsRoot "bin") -Filter "rc.exe" -Recurse -ErrorAction SilentlyContinue |
            Sort-Object FullName -Descending | Select-Object -First 1
    }
}
if ($null -ne $rc) {
    Write-Host "      OK: rc.exe (Windows SDK)" -ForegroundColor Green
} else {
    Write-Host "      AVISO: no se encuentra rc.exe del Windows SDK; el ejecutable se compilará sin icono incrustado (opcional)." -ForegroundColor Yellow
}

# 2. Agentes disponibles: solo se informa, porque cada equipo puede usar unos u otros.
Write-Host "[2/4] Buscando programas de agente..." -ForegroundColor Yellow
foreach ($agente in $AGENTES_EN_TERMINAL) {
    $comando = Get-Command $agente -ErrorAction SilentlyContinue | Select-Object -First 1
    if ($null -ne $comando) {
        Write-Host "      OK: $agente en $($comando.Source)" -ForegroundColor Green
    } else {
        Write-Host "      AVISO: $agente no está en el PATH; su botón de terminal no podrá abrirlo." -ForegroundColor Yellow
    }
}

# agy: su instalador oficial lo deja en %LOCALAPPDATA%\agy\bin, donde lo busca también la
# aplicación aunque no esté en el PATH.
$agyInstalado = Join-Path $env:LOCALAPPDATA "agy\bin\agy.exe"
if (Test-Path $agyInstalado) {
    Write-Host "      OK: agy en $agyInstalado" -ForegroundColor Green
}

# La aplicación de escritorio de Codex se abre con el codex.exe de la aplicación de la Microsoft
# Store (la que aparece como «ChatGPT»). Es la misma búsqueda que hace la aplicación.
$paqueteCodex = Get-AppxPackage -Name "OpenAI.Codex" -ErrorAction SilentlyContinue |
    Where-Object { $_.PublisherId -eq "2p2nqsd0c76g0" } |
    Sort-Object { [version]$_.Version } -Descending | Select-Object -First 1
$codexDeLaTienda = $null
if ($null -ne $paqueteCodex) { $codexDeLaTienda = Join-Path $paqueteCodex.InstallLocation "app\resources\codex.exe" }
if ($null -ne $codexDeLaTienda -and (Test-Path $codexDeLaTienda)) {
    Write-Host "      OK: aplicación de Codex en $codexDeLaTienda" -ForegroundColor Green
} else {
    Write-Host "      AVISO: la aplicación de Codex no está instalada; el botón de la aplicación de Codex no podrá abrirla." -ForegroundColor Yellow
}

# La aplicación de Antigravity se busca en su ruta de instalación por usuario, la misma que usa
# la aplicación.
$antigravity = Join-Path $env:LOCALAPPDATA "Programs\Antigravity\Antigravity.exe"
if (Test-Path $antigravity) {
    Write-Host "      OK: Antigravity en $antigravity" -ForegroundColor Green
} else {
    Write-Host "      AVISO: Antigravity no está instalado; el botón de Antigravity no podrá abrirlo." -ForegroundColor Yellow
}

# 3. Las cuatro puertas: formato, linter, pruebas y auditoría de dependencias.
if ($SinPuertas) {
    Write-Host "[3/4] Puertas omitidas a petición." -ForegroundColor Yellow
} else {
    Write-Host "[3/4] Pasando las cuatro puertas..." -ForegroundColor Yellow
    & (Join-Path $PSScriptRoot "puertas-rust.ps1") -CarpetaCompilacion $CarpetaCompilacion
    if ($LASTEXITCODE -ne 0) {
        Write-Host "      ERROR: alguna puerta ha fallado; revisa la salida anterior." -ForegroundColor Red
        exit 1
    }
}

# 4. Paquete portable: ejecutable, carpeta datos\ y documentación.
Write-Host "[4/4] Generando el paquete portable..." -ForegroundColor Yellow
& (Join-Path $PSScriptRoot "build-portable.ps1")
if (-not $?) {
    Write-Host "      ERROR: la generación del portable ha fallado." -ForegroundColor Red
    exit 1
}

Write-Host ""
Write-Host "=== Entorno preparado ===" -ForegroundColor Cyan
Write-Host "Ejecutable: $raizProyecto\portable\gestor-agentes.exe"
Write-Host ""
Write-Host "La configuración (lista de proyectos y perfiles) vive en $raizProyecto\portable\datos."
Write-Host "La primera vez, si existe la de la 2.x en $env:APPDATA\GestorClaude, la aplicación la copia allí."
Write-Host ""
exit 0
