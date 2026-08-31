$ErrorActionPreference = 'Stop'
$root = $PSScriptRoot
Set-Location $root

$versionFile = Join-Path $root 'VERSION'
if (-not (Test-Path $versionFile)) {
    throw 'VERSION file not found in repository root.'
}
$version = (Get-Content $versionFile -Raw).Trim()
if ($version -notmatch '^v(?<semver>\d+\.\d+\.\d+(-[0-9A-Za-z.\-]+)?)$') {
    throw "VERSION content '$version' is not a valid version like v2026.8.31-beta.1."
}
$semver = $Matches['semver']
Write-Host "Syncing version $semver from VERSION."

function Update-TextFile([string]$Path, [scriptblock]$Transform) {
    $content = [System.IO.File]::ReadAllText($Path)
    $updated = & $Transform $content
    if ($updated -ne $content) {
        [System.IO.File]::WriteAllText($Path, $updated)
        Write-Host "Updated $Path to $semver."
    } else {
        Write-Host "$Path already at $semver."
    }
}

Update-TextFile (Join-Path $root 'backend\Cargo.toml') {
    param($content)
    $regex = [regex]'(?m)^(version = )"[^"]*"$'
    if (-not $regex.IsMatch($content)) {
        throw 'Package version field not found in backend/Cargo.toml.'
    }
    return $regex.Replace($content, '${1}"' + $semver + '"', 1)
}

Update-TextFile (Join-Path $root 'frontend\package.json') {
    param($content)
    $versionField = [regex]'(?m)^(\s*"version"\s*:\s*)"[^"]*"'
    if ($versionField.IsMatch($content)) {
        return $versionField.Replace($content, '${1}"' + $semver + '"', 1)
    }
    $nameField = [regex]'(?m)^(\s*"name"\s*:\s*"tarkov-item-manager",\r?\n)'
    if (-not $nameField.IsMatch($content)) {
        throw 'name field not found in frontend/package.json.'
    }
    $newline = if ($content -match "`r`n") { "`r`n" } else { "`n" }
    $insertion = '${1}  "version": "' + $semver + '",' + $newline
    return $nameField.Replace($content, $insertion, 1)
}

if (Get-Command cargo -ErrorAction SilentlyContinue) {
    & cargo update --workspace --quiet --manifest-path (Join-Path $root 'backend\Cargo.toml')
    if ($LASTEXITCODE -ne 0) {
        throw 'cargo update --workspace failed.'
    }
    Write-Host 'Refreshed backend/Cargo.lock workspace version.'
} else {
    Write-Warning 'cargo not found in PATH; backend/Cargo.lock was not refreshed.'
}
