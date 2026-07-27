$ErrorActionPreference = 'Stop'
$root = $PSScriptRoot
$publishRoot = Join-Path $root 'pubdev'
$frontendRoot = Join-Path $root 'frontend'
$backendRoot = Join-Path $root 'backend'

function Invoke-CheckedCommand([string]$Description, [scriptblock]$Command) {
    & $Command
    if ($LASTEXITCODE -ne 0) {
        throw "$Description failed."
    }
}

if (-not (Get-Command cargo -ErrorAction SilentlyContinue)) {
    throw 'Cargo is not available in PATH. Install Rust first.'
}

if (-not (Get-Command npm -ErrorAction SilentlyContinue)) {
    throw 'npm is not available in PATH. Install Node.js first.'
}

if (-not (Test-Path (Join-Path $frontendRoot 'node_modules'))) {
    Invoke-CheckedCommand 'Frontend dependency installation' { npm install --prefix $frontendRoot }
}

Invoke-CheckedCommand 'Frontend production build' { npm run build --prefix $frontendRoot }
Invoke-CheckedCommand 'Backend release build' { cargo build --release --manifest-path (Join-Path $backendRoot 'Cargo.toml') }

New-Item -ItemType Directory -Force -Path $publishRoot | Out-Null
New-Item -ItemType Directory -Force -Path (Join-Path $publishRoot 'data') | Out-Null

$generatedPaths = @(
    (Join-Path $publishRoot 'tarkov-item-manager.exe'),
    (Join-Path $publishRoot 'frontend'),
    (Join-Path $publishRoot 'dataset')
)
foreach ($path in $generatedPaths) {
    if (Test-Path $path) {
        Remove-Item -Recurse -Force $path
    }
}

Copy-Item (Join-Path $backendRoot 'target\release\tarkov-item-manager.exe') (Join-Path $publishRoot 'tarkov-item-manager.exe')
New-Item -ItemType Directory -Force -Path (Join-Path $publishRoot 'frontend') | Out-Null
Copy-Item (Join-Path $frontendRoot 'dist') (Join-Path $publishRoot 'frontend\dist') -Recurse
Copy-Item (Join-Path $root 'dataset') (Join-Path $publishRoot 'dataset') -Recurse

$envFile = Join-Path $publishRoot '.env'
if (-not (Test-Path $envFile)) {
    $randomBytes = New-Object byte[] 32
    [System.Security.Cryptography.RandomNumberGenerator]::Create().GetBytes($randomBytes)
    $sessionSecret = [Convert]::ToBase64String($randomBytes)

    @"
DATABASE_URL=sqlite:data/tarkov-item-manager.db?mode=rwc
DATASET_DIR=dataset
APP_ORIGIN=http://127.0.0.1:3000
LISTEN_ADDR=127.0.0.1:3000
SESSION_SECRET=$sessionSecret
SECURE_COOKIES=false
"@ | Set-Content -Encoding utf8 $envFile
}

@'
@echo off
setlocal
cd /d "%~dp0"
echo Tarkov Item Manager: http://127.0.0.1:3000/login
"%~dp0tarkov-item-manager.exe"
'@ | Set-Content -Encoding ascii (Join-Path $publishRoot 'start.cmd')

Write-Host "Test release created at: $publishRoot"
Write-Host 'Start it with pubdev\start.cmd, then open http://127.0.0.1:3000/login'
