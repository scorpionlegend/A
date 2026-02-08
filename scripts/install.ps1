param(
    [ValidateSet("User", "AllUsers")]
    [string]$Scope = "User",
    [string]$Dest = ""
)

$ErrorActionPreference = "Stop"

$Root = Split-Path -Parent $MyInvocation.MyCommand.Path
$Src = Join-Path $Root "bin\\a.exe"

if (-not (Test-Path $Src)) {
    throw "Could not find $Src. Run this script from the extracted package root."
}

if ($Dest.Trim().Length -eq 0) {
    if ($Scope -eq "AllUsers") {
        $Dest = Join-Path $env:ProgramFiles "A"
    } else {
        $Dest = Join-Path $env:LOCALAPPDATA "A"
    }
}

$BinDir = Join-Path $Dest "bin"
New-Item -ItemType Directory -Force -Path $BinDir | Out-Null

Copy-Item $Src -Destination (Join-Path $BinDir "a.exe") -Force

if ($Scope -eq "AllUsers") {
    $isAdmin = ([Security.Principal.WindowsPrincipal] [Security.Principal.WindowsIdentity]::GetCurrent()).IsInRole([Security.Principal.WindowsBuiltInRole]::Administrator)
    if (-not $isAdmin) {
        throw "AllUsers install requires admin privileges. Re-run this script as Administrator."
    }
}

$envScope = if ($Scope -eq "AllUsers") { "Machine" } else { "User" }
$current = [Environment]::GetEnvironmentVariable("Path", $envScope)
if ($null -eq $current) { $current = "" }

if ($current -notmatch [Regex]::Escape($BinDir)) {
    $newPath = if ($current.Trim().Length -eq 0) { $BinDir } else { "$current;$BinDir" }
    [Environment]::SetEnvironmentVariable("Path", $newPath, $envScope)
    $env:Path = $newPath
    $scopeLabel = if ($Scope -eq "AllUsers") { "system" } else { "user" }
    Write-Host "Added $BinDir to the $scopeLabel PATH."
} else {
    Write-Host "PATH already contains $BinDir."
}

Write-Host "Installed A to $BinDir"
Write-Host "Open a new terminal and run: a --help"
