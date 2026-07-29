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

function Invoke-Git([string[]]$Arguments) {
    $output = & git @Arguments
    if ($LASTEXITCODE -ne 0) {
        throw "git $($Arguments -join ' ') failed."
    }
    return $output
}

if (-not (Get-Command git -ErrorAction SilentlyContinue)) {
    throw 'Git is not available in PATH.'
}

$tagOutput = & git describe --tags --abbrev=0
$version = 'dev'
if ($LASTEXITCODE -eq 0 -and $tagOutput) {
    $tag = $tagOutput.Trim()
    Write-Host "Candidate release version: $tag"
    if ((Read-Host 'Enter y to use this version') -ceq 'y') {
        $version = $tag
    } else {
        Write-Host 'Using development version.'
    }
} else {
    Write-Host 'No Git tag is available. Using development version.'
}

$archive = Join-Path $root "TarkovItemManager-$version.zip"
if ($version -ne 'dev' -and (Test-Path $archive)) {
    throw "Release archive already exists: $archive"
}
if ($version -eq 'dev' -and (Test-Path $archive)) {
    Remove-Item -Force $archive
}
if (-not (Get-Command cargo -ErrorAction SilentlyContinue)) {
    throw 'Cargo is not available in PATH. Install Rust first.'
}

if (-not (Get-Command pnpm -ErrorAction SilentlyContinue)) {
    throw 'pnpm is not available in PATH. Install pnpm first.'
}

Invoke-CheckedCommand 'Frontend dependency installation' { pnpm --dir $frontendRoot install --frozen-lockfile }
Invoke-CheckedCommand 'Frontend production build' { pnpm --dir $frontendRoot run build }
Invoke-CheckedCommand 'Backend release build' { cargo build --release --manifest-path (Join-Path $backendRoot 'Cargo.toml') }

New-Item -ItemType Directory -Force -Path $publishRoot | Out-Null
New-Item -ItemType Directory -Force -Path (Join-Path $publishRoot 'data') | Out-Null

$generatedPaths = @(
    (Join-Path $publishRoot 'tarkov-item-manager.exe'),
    (Join-Path $publishRoot 'start.cmd')
)
foreach ($path in $generatedPaths) {
    if (Test-Path $path) {
        Remove-Item -Recurse -Force $path
    }
}

Copy-Item (Join-Path $backendRoot 'target\release\tarkov-item-manager.exe') (Join-Path $publishRoot 'tarkov-item-manager.exe')

$envFile = Join-Path $publishRoot '.env'
if (-not (Test-Path $envFile)) {
    $randomBytes = New-Object byte[] 32
    [System.Security.Cryptography.RandomNumberGenerator]::Create().GetBytes($randomBytes)
    $sessionSecret = [Convert]::ToBase64String($randomBytes)

    @"
DATABASE_URL=sqlite:data/tarkov-item-manager.db?mode=rwc
APP_ORIGIN=http://127.0.0.1:3000
LISTEN_ADDR=127.0.0.1:3000
AUTO_OPEN_BROWSER=true
SESSION_SECRET=$sessionSecret
SECURE_COOKIES=false
"@ | Set-Content -Encoding utf8 $envFile
}

Compress-Archive -LiteralPath (Join-Path $publishRoot 'tarkov-item-manager.exe') -DestinationPath $archive -CompressionLevel Optimal -ErrorAction Stop

if (-not (Test-Path $archive)) {
    throw "Release archive was not created: $archive"
}

Write-Host "Test release created at: $publishRoot"
Write-Host "Release archive created at: $archive"
