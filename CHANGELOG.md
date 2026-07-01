# Changelog

## 1.0.6 - 2026-07-01

- Added multi-image upload for vision-capable OpenAI-compatible models.
- Added image thumbnails in chat while avoiding permanent image data in local storage.
- Added long-message TXT preview for very large pasted text.
- Added per-message delete buttons for removing a single chat record.
- Reworked the Windows setup into an interactive GUI panel with install, launch, uninstall, and clean uninstall actions.
- Removed CMD-based uninstall helper generation so shortcuts never point back to setup.

## 1.0.5 - 2026-07-01

- Fixed desktop chat requests by routing model calls through Tauri native Rust commands.
- Avoided WebView CORS failures that caused `Failed to fetch` after filling API settings.
- Fixed the local installer to embed the correct Tauri release executable.
- Hid the installer console window for release builds.
- Replaced PowerShell shortcut creation with native Windows COM shortcuts to stop flashing setup helper windows.
- Installer shortcuts now point directly to the app executable and never rerun setup.

## 1.0.4 - 2026-07-01

- Added Tauri desktop app scaffold for installable Windows UI.
- Added NSIS/MSI bundle configuration for standalone desktop installation.
- Kept desktop data under the app-specific data directory for predictable cleanup.
- Added a local Rust setup installer fallback when the Tauri bundler cannot download NSIS/MSI tooling.
- Fixed Windows release builds to use the GUI subsystem so no extra console window opens.

## 1.0.3 - 2026-07-01

- Added Windows desktop EXE launcher built with Rust.
- Added a fixed desktop data directory under `%LOCALAPPDATA%\AI-Assistant-Client`.
- Added `--clear-data` and a cleanup command file for easy residue cleanup.
- Added desktop documentation inspired by Chatbox's desktop-first structure.

## 1.0.2 - 2026-07-01

- Fixed Android export by writing conversation JSON to Downloads through the native bridge.
- Fixed clear conversation flow with a native Android confirmation dialog.
- Added clipboard paste support for API keys in WebView.
- Improved edge-to-edge display to reduce black borders on modern Android screens.
- Added smoother transform-based UI transitions without adding heavy runtime dependencies.

## 1.0.1 - 2026-07-01

- Improved Android WebView performance.
- Added responsive layout for phone, tablet, narrow screen, and landscape mode.
- Reduced temporary cache residue while preserving local conversations and settings.
- Rebuilt debug APK.

## 1.0.0 - 2026-07-01

- Initial AI assistant client.
- Added local chat storage, prompt shortcuts, streaming responses, and Android APK packaging.
