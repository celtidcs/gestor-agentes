# puertas-rust.ps1 — Las cuatro puertas de aceptación de Gestor Agentes 3.0.0 (Rust) en Windows.
#
# Un cambio solo se acepta si pasan las cuatro, sobre todo el workspace:
#   1. Formato: cargo fmt sin ninguna diferencia.
#   2. Linter: cargo clippy en todos los destinos, con los avisos tratados como errores.
#   3. Pruebas: toda la suite en verde, con las dependencias exactas del Cargo.lock.
#   4. Auditoría: cargo audit sin vulnerabilidades conocidas.
#
# Requisitos: Rust (cargo) y cargo-audit en el PATH, PowerShell 5.1 o posterior.
#
# Uso, desde cualquier carpeta:
#   .\puertas-rust.ps1
#   .\puertas-rust.ps1 -CarpetaCompilacion C:\cache\target
# La carpeta de compilación se pasa a cargo con --target-dir. No se usa CARGO_TARGET_DIR porque
# sccache mete las variables CARGO_* en su clave y entonces no comparte la caché entre
# compilaciones que usen carpetas distintas.
#
# Devuelve 0 si pasan las cuatro y 1 si falla alguna; siempre las ejecuta todas.

param(
    # Carpeta de compilación de cargo. Vacía: la de siempre, target\ en la raíz.
    [string]$CarpetaCompilacion = ""
)

# Continue y no Stop: cargo escribe su progreso en la salida de errores, y en Windows
# PowerShell 5.1, si esa salida está redirigida, cada línea llegaría como un error.
$ErrorActionPreference = 'Continue'
Set-Location (Join-Path $PSScriptRoot '..\..')

$script:Fallos = 0

# Argumentos de cargo para la carpeta de compilación, si se ha indicado.
$destino = @()
if ($CarpetaCompilacion -ne "") { $destino = @("--target-dir", $CarpetaCompilacion) }

# Ejecuta una puerta, informa del resultado y acumula el fallo sin detener las demás.
function Invoke-Puerta([string]$Nombre, [scriptblock]$Accion) {
    Write-Host "=== Puerta: $Nombre ===" -ForegroundColor Cyan
    & $Accion
    if ($LASTEXITCODE -eq 0) {
        Write-Host "--- ${Nombre}: CORRECTA" -ForegroundColor Green
    } else {
        Write-Host "--- ${Nombre}: FALLIDA" -ForegroundColor Red
        $script:Fallos++
    }
}

if ($null -eq (Get-Command cargo -ErrorAction SilentlyContinue)) {
    Write-Host "No se encuentra 'cargo'. Instala Rust desde https://rustup.rs y vuelve a abrir la terminal." -ForegroundColor Red
    exit 1
}

Invoke-Puerta 'Formato' { cargo fmt --all --check 2>&1 | ForEach-Object { "$_" } }
Invoke-Puerta 'Linter (clippy sin avisos)' {
    cargo clippy --workspace --all-targets --locked @destino -- -D warnings 2>&1 | ForEach-Object { "$_" }
}
Invoke-Puerta 'Pruebas' { cargo test --workspace --locked @destino 2>&1 | ForEach-Object { "$_" } }

# cargo audit devuelve un código distinto de 0 cuando encuentra vulnerabilidades.
Invoke-Puerta 'Auditoría de dependencias' {
    if ($null -eq (Get-Command cargo-audit -ErrorAction SilentlyContinue)) {
        Write-Host "No se encuentra cargo-audit. Instálalo con: cargo install cargo-audit --locked"
        $global:LASTEXITCODE = 1
        return
    }
    cargo audit 2>&1 | ForEach-Object { "$_" }
}

Write-Host ''
if ($script:Fallos -eq 0) {
    Write-Host '=== Las cuatro puertas están en verde ===' -ForegroundColor Green
    exit 0
}
Write-Host "=== $($script:Fallos) puerta(s) fallida(s) ===" -ForegroundColor Red
exit 1
