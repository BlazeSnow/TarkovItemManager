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

$baseTag = "v$(Get-Date -Format 'yyyy.M.d')"
$pattern = "^$([regex]::Escape($baseTag))\.(\d+)$"
$existingTags = @(
    Invoke-Git @('tag', '--list', "$baseTag.*")
    Invoke-Git @('ls-remote', '--tags', 'origin', "refs/tags/$baseTag.*") |
        ForEach-Object { ($_ -split "`t")[-1] -replace '^refs/tags/', '' -replace '\^\{\}$', '' }
) | Where-Object { $_ -match $pattern } | Select-Object -Unique

$nextSequence = 0
foreach ($existingTag in $existingTags) {
    if ($existingTag -match $pattern) {
        $nextSequence = [Math]::Max($nextSequence, [int]$Matches[1] + 1)
    }
}

$tag = "$baseTag.$nextSequence"
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
