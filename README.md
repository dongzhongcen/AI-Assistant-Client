# AI Assistant Client

轻量、干净、可安装的 AI 助手客户端。  
OpenAI-compatible desktop chat client with local-first data.  
Client IA simple, local, installable sous Windows.

## Version

- App: `1.0.6`
- Windows installer: `dist/installer/AI-Assistant-Client-Setup.exe`
- Android package: `com.dzc.aiassistant`
- Updated: `2026-07-01`

## Highlights

- 中文：独立 Windows 窗口、多会话、本地保存、导出/清空、图片识别、生图、长文本 TXT preview。
- English: installable Tauri desktop app, OpenAI-compatible API, vision/image generation, clean uninstall path.
- Francais: interface simple, donnees locales, installation et suppression faciles.

## Windows Install

Build the desktop app and setup program:

```powershell
npm run build
powershell -ExecutionPolicy Bypass -File .\build-windows-installer.ps1
```

Installer output:

```text
D:\AI-Client_dongzhongcen\dist\installer\AI-Assistant-Client-Setup.exe
```

Installed app:

```text
%LOCALAPPDATA%\Programs\AI-Assistant-Client\AI-Assistant-Client.exe
```

Local data:

```text
%LOCALAPPDATA%\AI-Assistant-Client
```

The setup opens a Windows GUI panel with an editable install path plus install, launch, uninstall, and clean uninstall actions. Clean uninstall also removes the Tauri/WebView `com.dzc.aiassistant` localStorage folders. Shortcuts point directly to the app exe, not back to setup.

## Model Setup

Open `设置 / Settings / Reglages` and fill:

- `Base URL`: for example `https://api.openai.com/v1`
- `API Key`: your provider key
- `Model`: use a vision-capable model for image recognition, or an image model when using 生图 mode

## Android

```powershell
.\build-android-apk.ps1
```

Output:

```text
AI-Assistant-Client-debug.apk
```

## Notes

- 图片用于当次识别，不永久写入本地历史，减少残留。
- Very long messages are collapsed into an openable `.txt` preview in the chat.
- Des donnees importantes restent locales; le nettoyage reste previsible.
