$root = $PSScriptRoot

if (-not (Get-Command cargo -ErrorAction SilentlyContinue)) {
    throw '未找到 cargo，请先安装 Rust。'
}

if (-not (Get-Command npm -ErrorAction SilentlyContinue)) {
    throw '未找到 npm，请先安装 Node.js。'
}

if (-not (Test-Path "$root\frontend\node_modules")) {
    npm install --prefix "$root\frontend"
    if ($LASTEXITCODE -ne 0) {
        throw '前端依赖安装失败。'
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

Write-Host '前端: http://127.0.0.1:5173/login'
Write-Host 'API:  http://127.0.0.1:3000/api/health'
