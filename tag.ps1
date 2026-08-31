$ErrorActionPreference = 'Stop'
$root = $PSScriptRoot
Set-Location $root

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

$versionFile = Join-Path $root 'VERSION'
if (-not (Test-Path $versionFile)) {
    throw 'VERSION file not found in repository root.'
}
$tag = (Get-Content $versionFile -Raw).Trim()
if ($tag -notmatch '^v\d+\.\d+\.\d+(-[0-9A-Za-z.\-]+)?$') {
    throw "VERSION content '$tag' is not a valid tag like v2026.8.31 or v2026.8.31-beta.1."
}

$localTag = Invoke-Git @('tag', '--list', $tag) | Where-Object { $_ }
$remoteTag = Invoke-Git @('ls-remote', '--tags', 'origin', "refs/tags/$tag") | Where-Object { $_ }
if ($localTag -or $remoteTag) {
    throw "Tag $tag already exists locally or on origin."
}

$commit = (Invoke-Git @('rev-parse', '--short', 'HEAD')).Trim()
Write-Host "Will create tag $tag for commit $commit."
$confirmation = Read-Host 'Enter y to create and push this tag'
if ($confirmation -cne 'y') {
    Write-Host 'Tag creation cancelled.'
    exit 0
}

if (Invoke-Git @('status', '--porcelain')) {
    throw 'Working tree has uncommitted changes. Commit or stash them before tagging.'
}

Invoke-Git @('tag', $tag) | Out-Null
try {
    Invoke-Git @('push', 'origin', "refs/tags/$tag") | Out-Null
} catch {
    Invoke-Git @('tag', '-d', $tag) | Out-Null
    throw
}

Write-Host "Created and pushed tag $tag."
