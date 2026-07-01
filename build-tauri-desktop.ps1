$ErrorActionPreference = "Stop"

$Root = Split-Path -Parent $MyInvocation.MyCommand.Path

Push-Location $Root
try {
    npm.cmd install
    npm.cmd run desktop:build
} finally {
    Pop-Location
}

Get-ChildItem -Recurse -Force src-tauri\target\release\bundle |
    Where-Object { $_.Extension -in ".exe", ".msi" } |
    Select-Object FullName,Length,LastWriteTime
