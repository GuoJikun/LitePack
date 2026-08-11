# LitePack Windows 构建入口脚本
# 用法:
#   .\scripts\build-windows.ps1              # 仅构建所有 crate
#   .\scripts\build-windows.ps1 -Msix        # 构建并打包 MSIX
#   .\scripts\build-windows.ps1 -Msix -Sign  # 构建、打包并签名

param(
    [switch]$Msix,
    [switch]$Sign
)

$ErrorActionPreference = "Stop"
$ProjectRoot = Split-Path -Parent $PSScriptRoot

Write-Host "=== LitePack Windows Build ===" -ForegroundColor Cyan
Write-Host "Project root: $ProjectRoot"
Write-Host ""

# ─── 构建所有 crate ───────────────────────────────────────────────────────────

Write-Host "[Build] cargo build --release (workspace)" -ForegroundColor Yellow
Push-Location $ProjectRoot
cargo build --release
if ($LASTEXITCODE -ne 0) {
    Write-Host "ERROR: cargo build failed" -ForegroundColor Red
    exit 1
}
Pop-Location

Write-Host ""
Write-Host "Build artifacts:" -ForegroundColor Green
Write-Host "  GUI:     target\release\litepack-gui.exe"
Write-Host "  CLI:     target\release\litepack.exe"
Write-Host "  Shell:   target\release\litepack_shell.dll"
Write-Host ""

# ─── 可选：打包 MSIX ──────────────────────────────────────────────────────────

if ($Msix) {
    Write-Host "[MSIX] Building MSIX package..." -ForegroundColor Yellow
    $buildScript = Join-Path $ProjectRoot "packaging\windows\build.ps1"
    $args = @()
    if ($Sign) { $args += "-Sign" }
    & $buildScript @args
    if ($LASTEXITCODE -ne 0) {
        Write-Host "ERROR: MSIX build failed" -ForegroundColor Red
        exit 1
    }
}

Write-Host ""
Write-Host "=== Done ===" -ForegroundColor Green
