# LitePack MSIX 构建脚本
# 用法: .\build.ps1 [-Configuration Release] [-Sign]
#
# 前置条件:
# - Rust toolchain (cargo)
# - Windows SDK (提供 makeappx.exe)
# - Node.js + pnpm (前端构建)
# - (可选) signtool.exe 用于签名

param(
    [string]$Configuration = "Release",
    [switch]$Sign
)

$ErrorActionPreference = "Stop"

# ─── 路径定义 ─────────────────────────────────────────────────────────────────

$ProjectRoot = Split-Path -Parent (Split-Path -Parent $PSScriptRoot)
$PackagingDir = $PSScriptRoot
$StagingDir = Join-Path $PackagingDir "staging"
$OutputDir = Join-Path $PackagingDir "output"

Write-Host "=== LitePack MSIX Build ===" -ForegroundColor Cyan
Write-Host "Project root: $ProjectRoot"
Write-Host "Configuration: $Configuration"
Write-Host ""

# ─── Step 1: 构建 Shell Extension DLL ─────────────────────────────────────────

Write-Host "[1/5] Building litepack_shell.dll..." -ForegroundColor Yellow
Push-Location $ProjectRoot
cargo build --release -p litepack-shell
if ($LASTEXITCODE -ne 0) {
    Write-Host "ERROR: Failed to build litepack-shell" -ForegroundColor Red
    exit 1
}
Pop-Location

$ShellDll = Join-Path $ProjectRoot "target\release\litepack_shell.dll"
if (-not (Test-Path $ShellDll)) {
    Write-Host "ERROR: litepack_shell.dll not found at $ShellDll" -ForegroundColor Red
    exit 1
}
Write-Host "  OK: $ShellDll"

# ─── Step 2: 构建 Tauri GUI ──────────────────────────────────────────────────

Write-Host "[2/5] Building LitePack GUI (Tauri)..." -ForegroundColor Yellow
Push-Location (Join-Path $ProjectRoot "crates\litepack-gui")
pnpm tauri build
if ($LASTEXITCODE -ne 0) {
    Write-Host "ERROR: Tauri build failed" -ForegroundColor Red
    exit 1
}
Pop-Location

$GuiExe = Join-Path $ProjectRoot "target\release\litepack-gui.exe"
if (-not (Test-Path $GuiExe)) {
    Write-Host "ERROR: litepack-gui.exe not found at $GuiExe" -ForegroundColor Red
    exit 1
}
Write-Host "  OK: $GuiExe"

# ─── Step 3: 准备 staging 目录 ───────────────────────────────────────────────

Write-Host "[3/5] Preparing staging directory..." -ForegroundColor Yellow

if (Test-Path $StagingDir) {
    Remove-Item -Recurse -Force $StagingDir
}
New-Item -ItemType Directory -Path $StagingDir | Out-Null
New-Item -ItemType Directory -Path (Join-Path $StagingDir "Assets") | Out-Null

# 复制 manifest
Copy-Item (Join-Path $PackagingDir "Package.appxmanifest") (Join-Path $StagingDir "AppxManifest.xml")

# 复制 DLL 和 EXE（重命名为 manifest 中声明的名称）
Copy-Item $ShellDll (Join-Path $StagingDir "litepack_shell.dll")
Copy-Item $GuiExe (Join-Path $StagingDir "LitePack.exe")

# 复制图标资源（如果存在）
$IconDir = Join-Path $ProjectRoot "crates\litepack-gui\src-tauri\icons"
if (Test-Path $IconDir) {
    # StoreLogo.png (50x50)
    if (Test-Path (Join-Path $IconDir "32x32.png")) {
        Copy-Item (Join-Path $IconDir "32x32.png") (Join-Path $StagingDir "Assets\StoreLogo.png")
    }
    # Square150x150Logo.png
    if (Test-Path (Join-Path $IconDir "128x128.png")) {
        Copy-Item (Join-Path $IconDir "128x128.png") (Join-Path $StagingDir "Assets\Square150x150Logo.png")
    }
    # Square44x44Logo.png
    if (Test-Path (Join-Path $IconDir "32x32.png")) {
        Copy-Item (Join-Path $IconDir "32x32.png") (Join-Path $StagingDir "Assets\Square44x44Logo.png")
    }
}

Write-Host "  Staging directory ready: $StagingDir"
Get-ChildItem $StagingDir -Recurse | ForEach-Object {
    Write-Host "    $($_.FullName.Replace($StagingDir, '.'))"
}

# ─── Step 4: 打包 MSIX ───────────────────────────────────────────────────────

Write-Host "[4/5] Creating MSIX package..." -ForegroundColor Yellow

# 查找 makeappx.exe
$MakeAppx = Get-ChildItem "C:\Program Files (x86)\Windows Kits\10\bin\*\x64\makeappx.exe" -ErrorAction SilentlyContinue |
    Sort-Object FullName -Descending | Select-Object -First 1

if (-not $MakeAppx) {
    Write-Host "ERROR: makeappx.exe not found. Install Windows SDK." -ForegroundColor Red
    exit 1
}
Write-Host "  Using: $($MakeAppx.FullName)"

if (-not (Test-Path $OutputDir)) {
    New-Item -ItemType Directory -Path $OutputDir | Out-Null
}

$MsixPath = Join-Path $OutputDir "LitePack.msix"
if (Test-Path $MsixPath) {
    Remove-Item $MsixPath
}

& $MakeAppx.FullName pack /d $StagingDir /p $MsixPath /o
if ($LASTEXITCODE -ne 0) {
    Write-Host "ERROR: makeappx pack failed" -ForegroundColor Red
    exit 1
}
Write-Host "  OK: $MsixPath"

# ─── Step 5: 签名（可选）─────────────────────────────────────────────────────

if ($Sign) {
    Write-Host "[5/5] Signing MSIX package..." -ForegroundColor Yellow

    $SignTool = Get-ChildItem "C:\Program Files (x86)\Windows Kits\10\bin\*\x64\signtool.exe" -ErrorAction SilentlyContinue |
        Sort-Object FullName -Descending | Select-Object -First 1

    if (-not $SignTool) {
        Write-Host "WARNING: signtool.exe not found, skipping signing" -ForegroundColor Yellow
    } else {
        # 使用自签名证书（开发用）
        $CertPath = Join-Path $PackagingDir "LitePack_Dev.pfx"
        if (-not (Test-Path $CertPath)) {
            Write-Host "  Creating self-signed certificate..."
            $Cert = New-SelfSignedCertificate -Type Custom -Subject "CN=LitePack" -KeyUsage DigitalSignature -FriendlyName "LitePack Dev" -CertStoreLocation "Cert:\CurrentUser\My" -TextExtension @("2.5.29.37={text}1.3.6.1.5.5.7.3.3", "2.5.29.19={text}")
            $PfxPasswordPlain = $env:LITEPACK_PFX_PASSWORD
            if (-not $PfxPasswordPlain) { $PfxPasswordPlain = "litepack" }
            $Password = ConvertTo-SecureString -String $PfxPasswordPlain -Force -AsPlainText
            Export-PfxCertificate -cert "Cert:\CurrentUser\My\$($Cert.Thumbprint)" -FilePath $CertPath -Password $Password | Out-Null

        & $SignTool.FullName sign /fd SHA256 /a /f $CertPath /p $PfxPasswordPlain $MsixPath
        if ($LASTEXITCODE -ne 0) {
            Write-Host "WARNING: Signing failed" -ForegroundColor Yellow
        } else {
            Write-Host "  Signed: $MsixPath"
        }
    }
} else {
    Write-Host "[5/5] Signing skipped (use -Sign to enable)" -ForegroundColor Gray
}

# ─── 完成 ─────────────────────────────────────────────────────────────────────

Write-Host ""
Write-Host "=== Build Complete ===" -ForegroundColor Green
Write-Host "MSIX: $MsixPath"
Write-Host ""
Write-Host "安装测试: Add-AppxPackage -Path '$MsixPath'" -ForegroundColor Cyan
Write-Host "卸载测试: Get-AppxPackage LitePack | Remove-AppxPackage" -ForegroundColor Cyan
