$root = $PSScriptRoot

if (-not (Get-Command cargo -ErrorAction SilentlyContinue)) {
    throw 'Cargo is not available in PATH. Install Rust first.'
}

if (-not (Get-Command npm -ErrorAction SilentlyContinue)) {
    throw 'npm is not available in PATH. Install Node.js first.'
}

if (-not (Test-Path "$root\frontend\node_modules")) {
    npm install --prefix "$root\frontend"
    if ($LASTEXITCODE -ne 0) {
        throw 'Frontend dependency installation failed.'
    }
}

$envFile = "$root\backend\.env"
if (-not (Test-Path $envFile)) {
    @'
DATABASE_URL=sqlite:data/tarkov-item-manager.db?mode=rwc
DATASET_DIR=../dataset
APP_ORIGIN=http://localhost:5173
LISTEN_ADDR=0.0.0.0:3000
SESSION_SECRET=local-development-secret-change-me
SECURE_COOKIES=false
'@ | Set-Content -Encoding utf8 $envFile
}

Start-Process powershell -ArgumentList '-NoExit', '-Command', "Set-Location '$root\backend'; cargo run"
Start-Process powershell -ArgumentList '-NoExit', '-Command', "Set-Location '$root\frontend'; npm run dev -- --host 127.0.0.1"

Write-Host 'Frontend: http://127.0.0.1:5173/login'
Write-Host 'API:      http://127.0.0.1:3000/api/health'
