$root = $PSScriptRoot

function Test-ListeningPort([int]$Port) {
    return $null -ne (Get-NetTCPConnection -LocalPort $Port -State Listen -ErrorAction SilentlyContinue)
}

if (-not (Get-Command cargo -ErrorAction SilentlyContinue)) {
    throw 'Cargo is not available in PATH. Install Rust first.'
}

if (-not (Get-Command pnpm -ErrorAction SilentlyContinue)) {
    throw 'pnpm is not available in PATH. Install pnpm first.'
}

if (Test-ListeningPort 3000) {
    throw 'Port 3000 is already in use. Stop the existing API process before running this script.'
}

if (Test-ListeningPort 5173) {
    throw 'Port 5173 is already in use. Stop the existing Vite process before running this script.'
}

pnpm install --dir "$root\frontend" --frozen-lockfile
if ($LASTEXITCODE -ne 0) {
    throw 'Frontend dependency installation failed.'
}

$envFile = "$root\backend\.env"
if (-not (Test-Path $envFile)) {
    @'
DATABASE_URL=sqlite:data/tarkov-item-manager.db?mode=rwc
APP_ORIGIN=http://localhost:5173
LISTEN_ADDR=0.0.0.0:3000
SESSION_SECRET=local-development-secret-change-me
SECURE_COOKIES=false
'@ | Set-Content -Encoding utf8 $envFile
}

Start-Process powershell -ArgumentList '-NoExit', '-Command', "Set-Location '$root\backend'; cargo run"

$apiReady = $false
for ($attempt = 0; $attempt -lt 30; $attempt++) {
    Start-Sleep -Seconds 1
    try {
        if ((Invoke-WebRequest -UseBasicParsing 'http://127.0.0.1:3000/api/health').Content -eq 'ok') {
            $apiReady = $true
            break
        }
    } catch {
    }
}

if (-not $apiReady) {
    throw 'The API did not become ready on port 3000. Check the API window for errors.'
}

Start-Process powershell -ArgumentList '-NoExit', '-Command', "Set-Location '$root\frontend'; pnpm dev -- --host 127.0.0.1 --strictPort"

Write-Host 'Frontend: http://127.0.0.1:5173/login'
Write-Host 'API:      http://127.0.0.1:3000/api/health'
