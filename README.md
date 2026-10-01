# AI Assistant Client

<p align="center">
  <img alt="Version" src="https://img.shields.io/badge/version-1.0.6-blue">
  <img alt="JavaScript" src="https://img.shields.io/badge/javascript-vanilla-yellow">
  <img alt="Tauri" src="https://img.shields.io/badge/tauri-2.x-24c8db">
  <img alt="Rust" src="https://img.shields.io/badge/rust-2021%20edition-dea584">
  <img alt="Android" src="https://img.shields.io/badge/android-minSdk%2024-3ddc84">
  <img alt="Windows" src="https://img.shields.io/badge/platform-Windows-0078d6">
  <img alt="OpenAI Compatible" src="https://img.shields.io/badge/api-OpenAI--compatible-412991">
</p>

AI Assistant Client 是一个轻量、干净、可安装的 AI 助手客户端，支持 OpenAI-compatible 接口，数据默认保存在本地。前端界面由原生 HTML / CSS / JavaScript 编写，同一套界面分别打包为 Tauri 2 Windows 桌面应用、Rust 编写的 Windows 安装程序、基于 Edge 应用窗口的轻量桌面 EXE，以及基于 WebView 的 Android 应用。项目目前实现了多会话聊天、本地保存、导出/清空、图片识别、生图、对话转接、长文本 TXT 预览和可清理的安装/卸载流程。

## 功能特性

- **多会话聊天**：通过 OpenAI-compatible `/chat/completions` 接口对话，会话保存在本地 `localStorage`，支持导出、清空和删除单条消息。
- **图片识别**：支持多图上传给具备视觉能力的模型；图片只用于当次识别，不永久写入本地历史。
- **生图模式**：调用 `/images/generations` 接口生成图片，可单独配置 Image Base URL、Image API Key 和 Image Model，并支持保存生成的图片。
- **对话转接**：把当前对话上下文复制到新对话中继续提问。
- **长文本预览**：超过 12000 字符的消息会折叠为可打开的 `.txt` 预览。
- **Tauri 桌面应用**：模型请求通过 Rust 原生命令（`chat_completions`、`image_generations`）发出，避免 WebView 的 CORS 问题。
- **Windows 安装程序**：图形化安装面板，支持修改安装路径、安装、启动、卸载和清理卸载；清理卸载会同时删除 `com.dzc.aiassistant` 的 WebView 数据目录。
- **轻量桌面 EXE**：Rust 程序内嵌界面并在本地提供服务，优先以 Edge 应用窗口打开，支持 `--data-dir` 和 `--clear-data`。
- **Android 应用**：WebView 加载内置的 `index.html`，包名 `com.dzc.aiassistant`。

## 项目结构

```text
.
├── index.html / app.js / styles.css   # 聊天界面（各平台共用）
├── server.js                          # 本地静态服务和 /api/chat/completions 代理（端口 4173）
├── src-tauri/                         # Tauri 2 桌面应用（Rust 命令、打包配置）
├── installer/                         # Rust 编写的 Windows 图形化安装程序
├── desktop/                           # Rust 轻量桌面 EXE（Edge 应用窗口）
├── app/                               # Android 应用（MainActivity + WebView）
├── dist/windows/                      # 数据清理脚本 Clean-AI-Assistant-Client-Data.cmd
├── build-tauri-assets.ps1             # 复制前端文件到 dist/tauri-web
├── build-tauri-desktop.ps1            # 安装依赖并构建 Tauri 桌面应用
├── build-windows-installer.ps1        # 构建 Windows 安装程序
├── build-desktop-exe.ps1              # 构建轻量桌面 EXE
├── build-android-apk.ps1              # 手动构建 Android debug APK
├── DESKTOP.md                         # 轻量桌面 EXE 说明
└── CHANGELOG.md                       # 版本记录
```

## 快速开始

### 环境要求

- Windows（桌面构建脚本均为 PowerShell，并使用 `x86_64-pc-windows-msvc` 目标）
- Node.js 与 npm（Tauri CLI `2.11.4`）
- Rust 工具链与 Cargo
- Android SDK（build-tools、platform）和 JDK（仅构建 Android 时需要）

### 开发调试（Tauri）

```powershell
npm install
npm run desktop:dev
```

### 构建 Tauri 桌面应用

```powershell
powershell -ExecutionPolicy Bypass -File .\build-tauri-desktop.ps1
```

脚本会执行 `npm install` 和 `npm run desktop:build`，在 `src-tauri\target\release\bundle` 下生成 NSIS / MSI 安装包。

### 构建 Windows 安装程序

```powershell
powershell -ExecutionPolicy Bypass -File .\build-windows-installer.ps1
```

安装程序会内嵌 `src-tauri\target\x86_64-pc-windows-msvc\release\ai_assistant_client.exe`，需要先以该 target 构建 Tauri 程序（例如 `npm run tauri -- build --target x86_64-pc-windows-msvc`）。输出：

```text
dist\installer\AI-Assistant-Client-Setup.exe
```

安装后的程序位于 `%LOCALAPPDATA%\Programs\AI-Assistant-Client\AI-Assistant-Client.exe`，快捷方式直接指向程序本身，不会再次打开安装程序。

### 构建轻量桌面 EXE

```powershell
powershell -ExecutionPolicy Bypass -File .\build-desktop-exe.ps1
```

输出 `dist\windows\AI-Assistant-Client.exe` 和 `dist\windows\Clean-AI-Assistant-Client-Data.cmd`，详见 [DESKTOP.md](DESKTOP.md)。

### 构建 Android APK

```powershell
.\build-android-apk.ps1 -SdkPath "你的 Android SDK 路径"
```

输出 `AI-Assistant-Client-debug.apk`（使用自动生成的 debug keystore 签名）。

### 模型配置

打开「设置」，填写：

- `Base URL`：例如 `https://api.openai.com/v1`
- `API Key`：模型服务的密钥
- `Model`：图片识别需使用支持视觉的模型，生图模式需使用图像模型
- `Image Base URL` / `Image API Key` / `Image Model`：可选，为生图单独配置服务

## 当前状态

项目当前版本为 1.0.6（2026-07-01），已完成多平台打包和主要聊天功能，版本记录见 [CHANGELOG.md](CHANGELOG.md)。后续可继续完善：

- 修复 `server.js`：仓库 `package.json` 声明了 `"type": "module"`，而 `server.js` 使用 `require`，直接运行会报错
- 统一 `build-windows-installer.ps1` 检查的 Tauri 输出路径与 `npm run desktop:build` 的默认输出路径
- 同步 Android `versionName`（当前为 1.0.0）与应用版本 1.0.6
- 去掉 `build-android-apk.ps1` 中写死的本机 SDK 路径默认值
- 在 GitHub Releases 提供安装包和 APK

## 数据和敏感信息

桌面版本地数据默认位于 `%LOCALAPPDATA%\AI-Assistant-Client`，可通过环境变量 `AI_ASSISTANT_CLIENT_DATA` 修改（轻量桌面 EXE），并可通过 `--clear-data` 或清理卸载彻底删除。构建产物（`build/`、`dist/`、各 `target/`）、`*.apk`、keystore、`node_modules/`、本地 SDK 配置和 `.env` 系列文件已通过 `.gitignore` 排除，不应提交到仓库。
