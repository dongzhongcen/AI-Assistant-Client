param(
    [string]$SdkPath = "C:\Users\10772\AppData\Local\Android\Sdk",
    [string]$BuildToolsVersion = "36.1.0",
    [string]$PlatformVersion = "android-36",
    [string]$OutputApk = "AI-Assistant-Client-debug.apk"
)

$ErrorActionPreference = "Stop"

$Root = Split-Path -Parent $MyInvocation.MyCommand.Path
$BuildDir = Join-Path $Root "build\android-manual"
$AssetsDir = Join-Path $Root "app\src\main\assets"
$BuildTools = Join-Path $SdkPath "build-tools\$BuildToolsVersion"
$AndroidJar = Join-Path $SdkPath "platforms\$PlatformVersion\android.jar"
$Aapt2 = Join-Path $BuildTools "aapt2.exe"
$D8 = Join-Path $BuildTools "d8.bat"
$Zipalign = Join-Path $BuildTools "zipalign.exe"
$Apksigner = Join-Path $BuildTools "apksigner.bat"
$Manifest = Join-Path $Root "app\src\main\AndroidManifest.xml"
$ResDir = Join-Path $Root "app\src\main\res"
$JavaDir = Join-Path $Root "app\src\main\java"
$KeyStore = Join-Path $BuildDir "debug.keystore"
$UnsignedApk = Join-Path $BuildDir "unsigned.apk"
$AlignedApk = Join-Path $BuildDir "aligned.apk"
$FinalApk = Join-Path $Root $OutputApk

foreach ($Path in @($Aapt2, $D8, $Zipalign, $Apksigner, $AndroidJar)) {
    if (!(Test-Path $Path)) {
        throw "Missing Android build tool: $Path"
    }
}

if (Test-Path $BuildDir) {
    Remove-Item -LiteralPath $BuildDir -Recurse -Force
}
New-Item -ItemType Directory -Force $BuildDir, $AssetsDir | Out-Null

Copy-Item -LiteralPath (Join-Path $Root "index.html") -Destination $AssetsDir -Force
Copy-Item -LiteralPath (Join-Path $Root "styles.css") -Destination $AssetsDir -Force
Copy-Item -LiteralPath (Join-Path $Root "app.js") -Destination $AssetsDir -Force

$CompiledResDir = Join-Path $BuildDir "compiled-res"
$ClassesDir = Join-Path $BuildDir "classes"
$DexDir = Join-Path $BuildDir "dex"
New-Item -ItemType Directory -Force $CompiledResDir, $ClassesDir, $DexDir | Out-Null

& $Aapt2 compile --dir $ResDir -o $CompiledResDir
if ($LASTEXITCODE -ne 0) { throw "aapt2 compile failed" }

$FlatFiles = Get-ChildItem -Path $CompiledResDir -Filter *.flat | ForEach-Object { $_.FullName }
& $Aapt2 link `
    -o $UnsignedApk `
    -I $AndroidJar `
    --manifest $Manifest `
    -A $AssetsDir `
    --java (Join-Path $BuildDir "generated") `
    --auto-add-overlay `
    $FlatFiles
if ($LASTEXITCODE -ne 0) { throw "aapt2 link failed" }

$JavaFiles = Get-ChildItem -Path $JavaDir,(Join-Path $BuildDir "generated") -Recurse -Filter *.java | ForEach-Object { $_.FullName }
& javac -encoding UTF-8 -source 1.8 -target 1.8 -bootclasspath $AndroidJar -d $ClassesDir $JavaFiles
if ($LASTEXITCODE -ne 0) { throw "javac failed" }

$ClassFiles = Get-ChildItem -Path $ClassesDir -Recurse -Filter *.class | ForEach-Object { $_.FullName }
& $D8 --lib $AndroidJar --output $DexDir $ClassFiles
if ($LASTEXITCODE -ne 0) { throw "d8 failed" }

Push-Location $DexDir
try {
    & jar uf $UnsignedApk "classes.dex"
    if ($LASTEXITCODE -ne 0) { throw "jar failed while adding classes.dex" }
} finally {
    Pop-Location
}

if (!(Test-Path $KeyStore)) {
    & keytool -genkeypair `
        -keystore $KeyStore `
        -storepass android `
        -keypass android `
        -alias androiddebugkey `
        -keyalg RSA `
        -keysize 2048 `
        -validity 10000 `
        -dname "CN=Android Debug,O=Android,C=US"
    if ($LASTEXITCODE -ne 0) { throw "keytool failed" }
}

& $Zipalign -f 4 $UnsignedApk $AlignedApk
if ($LASTEXITCODE -ne 0) { throw "zipalign failed" }

& $Apksigner sign `
    --ks $KeyStore `
    --ks-pass pass:android `
    --key-pass pass:android `
    --out $FinalApk `
    $AlignedApk
if ($LASTEXITCODE -ne 0) { throw "apksigner sign failed" }

& $Apksigner verify --verbose $FinalApk
if ($LASTEXITCODE -ne 0) { throw "apksigner verify failed" }

Get-Item -LiteralPath $FinalApk | Select-Object FullName,Length,LastWriteTime
