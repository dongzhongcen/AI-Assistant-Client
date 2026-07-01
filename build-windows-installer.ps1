$ErrorActionPreference = "Stop"

$Root = Split-Path -Parent $MyInvocation.MyCommand.Path
$OutDir = Join-Path $Root "dist\installer"
$TauriExe = Join-Path $Root "src-tauri\target\x86_64-pc-windows-msvc\release\ai_assistant_client.exe"
$SetupSource = Join-Path $Root "installer\target\x86_64-pc-windows-msvc\release\AI-Assistant-Client-Setup.exe"
$SetupDest = Join-Path $OutDir "AI-Assistant-Client-Setup.exe"

if (!(Test-Path $TauriExe)) {
    npm.cmd run desktop:build
}

New-Item -ItemType Directory -Force $OutDir | Out-Null

Push-Location (Join-Path $Root "installer")
try {
    cargo build --release --target x86_64-pc-windows-msvc
} finally {
    Pop-Location
}

if (!(Test-Path $SetupSource)) {
    throw "Installer EXE was not produced: $SetupSource"
}

Copy-Item -LiteralPath $SetupSource -Destination $SetupDest -Force

Get-Item -LiteralPath $SetupDest | Select-Object FullName,Length,LastWriteTime
