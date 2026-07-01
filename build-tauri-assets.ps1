$ErrorActionPreference = "Stop"

$Root = Split-Path -Parent $MyInvocation.MyCommand.Path
$Out = Join-Path $Root "dist\tauri-web"

New-Item -ItemType Directory -Force $Out | Out-Null
Copy-Item -LiteralPath (Join-Path $Root "index.html") -Destination $Out -Force
Copy-Item -LiteralPath (Join-Path $Root "app.js") -Destination $Out -Force
Copy-Item -LiteralPath (Join-Path $Root "styles.css") -Destination $Out -Force
