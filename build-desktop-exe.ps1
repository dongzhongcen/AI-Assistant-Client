$ErrorActionPreference = "Stop"

$Root = Split-Path -Parent $MyInvocation.MyCommand.Path
$DesktopDir = Join-Path $Root "desktop"
$OutDir = Join-Path $Root "dist\windows"
$ExeSource = Join-Path $DesktopDir "target\x86_64-pc-windows-msvc\release\AI-Assistant-Client.exe"
$ExeDest = Join-Path $OutDir "AI-Assistant-Client.exe"
$CleanScript = Join-Path $OutDir "Clean-AI-Assistant-Client-Data.cmd"

New-Item -ItemType Directory -Force $OutDir | Out-Null

Push-Location $DesktopDir
try {
    cargo build --release --target x86_64-pc-windows-msvc
} finally {
    Pop-Location
}

if (!(Test-Path $ExeSource)) {
    throw "Desktop EXE was not produced: $ExeSource"
}

Copy-Item -LiteralPath $ExeSource -Destination $ExeDest -Force

@'
@echo off
set EXE_DIR=%~dp0
"%EXE_DIR%AI-Assistant-Client.exe" --clear-data
pause
'@ | Set-Content -LiteralPath $CleanScript -Encoding ASCII

Get-Item -LiteralPath $ExeDest | Select-Object FullName,Length,LastWriteTime
Get-Item -LiteralPath $CleanScript | Select-Object FullName,Length,LastWriteTime
