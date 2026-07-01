# Desktop Build

This desktop edition follows the same broad idea as Chatbox:

- a small desktop host process
- a web renderer for the chat UI
- local-first settings and conversations
- clean, explicit user-data management

It intentionally avoids bundling Electron for now. The Windows host is a Rust EXE that serves the embedded UI locally and opens it in an app-style Edge window when available.

## Build

```powershell
powershell -ExecutionPolicy Bypass -File .\build-desktop-exe.ps1
```

## Output

```text
dist/windows/AI-Assistant-Client.exe
dist/windows/Clean-AI-Assistant-Client-Data.cmd
```

## Data And Cleanup

Default data directory:

```text
%LOCALAPPDATA%\AI-Assistant-Client
```

Show the active data directory:

```powershell
dist/windows/AI-Assistant-Client.exe --data-dir
```

Clean all desktop residue:

```powershell
dist/windows/AI-Assistant-Client.exe --clear-data
```

or double-click:

```text
dist/windows/Clean-AI-Assistant-Client-Data.cmd
```

Override data directory:

```powershell
$env:AI_ASSISTANT_CLIENT_DATA="D:\AI-Assistant-Data"
dist/windows/AI-Assistant-Client.exe
```
