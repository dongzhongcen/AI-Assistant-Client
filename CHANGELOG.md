# Changelog

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
