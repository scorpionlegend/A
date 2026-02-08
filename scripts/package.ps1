param(
    [string]$Target = "",
    [string]$OutDir = "dist"
)

$ErrorActionPreference = "Stop"

$ScriptDir = Split-Path -Parent $MyInvocation.MyCommand.Path
$Root = Resolve-Path (Join-Path $ScriptDir "..")

Push-Location $Root

if ($Target.Trim().Length -gt 0) {
    $targetArg = @("--target", $Target)
    $pkgName = "a-$Target"
    $binPath = Join-Path $Root "target\$Target\release\a.exe"
} else {
    $targetArg = @()
    $arch = if ([Environment]::Is64BitOperatingSystem) { "x86_64" } else { "x86" }
    $pkgName = "a-windows-$arch"
    $binPath = Join-Path $Root "target\release\a.exe"
}

Write-Host "Building release..."
cargo build --release @targetArg

if (-not (Test-Path $binPath)) {
    throw "Expected binary not found: $binPath"
}

$distDir = Join-Path $Root $OutDir
$stageDir = Join-Path $distDir $pkgName
$zipPath = Join-Path $distDir "$pkgName.zip"
$rawPath = Join-Path $distDir "$pkgName.exe"

if (Test-Path $stageDir) { Remove-Item -Recurse -Force $stageDir }
if (Test-Path $zipPath) { Remove-Item -Force $zipPath }
if (Test-Path $rawPath) { Remove-Item -Force $rawPath }

New-Item -ItemType Directory -Force -Path (Join-Path $stageDir "bin") | Out-Null
Copy-Item $binPath -Destination (Join-Path $stageDir "bin\a.exe")
Copy-Item (Join-Path $Root "README.md") -Destination (Join-Path $stageDir "README.md")
Copy-Item (Join-Path $Root "syntax.md") -Destination (Join-Path $stageDir "syntax.md")
Copy-Item (Join-Path $Root "scripts\\install.ps1") -Destination (Join-Path $stageDir "install.ps1")

Compress-Archive -Path (Join-Path $stageDir "*") -DestinationPath $zipPath
Copy-Item $binPath -Destination $rawPath

Pop-Location

Write-Host "Wrote $zipPath"
Write-Host "Wrote $rawPath"
