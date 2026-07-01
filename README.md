# AI Assistant Client

轻量、干净、可打包的 AI 助手客户端。  
Lightweight AI chat client, ready for Android packaging.  
Client IA leger, simple et portable.

## Version

- App: `1.0.2`
- Android package: `com.dzc.aiassistant`
- Build type: debug APK
- Updated: `2026-07-01`

## What It Does

- 中文：多会话、本地保存、提示词库、流式回复。
- English: OpenAI-compatible chat client with local storage.
- Francais: interface simple, donnees locales, APK Android possible.

## Run On Desktop

```bash
node server.js
```

Open:

```text
http://localhost:4173
```

## Android APK

Build:

```powershell
.\build-android-apk.ps1
```

Output:

```text
AI-Assistant-Client-debug.apk
```

The Android app uses a local WebView shell and a native bridge for streaming chat requests.  
No extra Gradle download is required; the script uses the local Android SDK tools.

## Windows Desktop EXE

Inspired by Chatbox's desktop-first structure, this project keeps a small host process and a web chat renderer, while making data cleanup explicit.

Build:

```powershell
.\build-desktop-exe.ps1
```

Output:

```text
dist/windows/AI-Assistant-Client.exe
dist/windows/Clean-AI-Assistant-Client-Data.cmd
```

Desktop data is kept in one easy-to-clean folder:

```text
%LOCALAPPDATA%\AI-Assistant-Client
```

Clean it:

```powershell
dist/windows/AI-Assistant-Client.exe --clear-data
```

or run:

```text
dist/windows/Clean-AI-Assistant-Client-Data.cmd
```

## Model Setup

In the app, open `设置 / Settings / Reglages` and fill:

- `Base URL`: OpenAI-compatible endpoint, for example `https://api.openai.com/v1`
- `API Key`: your provider key
- `Model`: for example `gpt-4.1-mini`

## Notes

- 会话和设置默认保存在本地浏览器/WebView 存储里。
- Cache is kept minimal; temporary WebView cache is cleared on app exit.
- Les donnees importantes restent locales, les fichiers temporaires sont limites.

## Latest Fix

`1.0.2` fixes Android export, clear confirmation, API-key paste, edge-to-edge display, and smoother motion.
