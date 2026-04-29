---
title: 'Desktop (Tauri)'
description: 'Tauri desktop deep dive — architecture, file layout, system tray, per-platform secret storage, logs, updates'
order: 1
---

# Desktop (Tauri)

The Bodhi desktop app is a native shell around the same HTTP server that Docker runs. It's the recommended way to use Bodhi on a personal workstation.

If you only need to install it and start chatting, head to **[Install](/docs/install)** — that walks through the wizard with screenshots. This page picks up where install ends and covers what happens after launch: where data lives, how the tray behaves, and how to inspect logs.

## How it actually works

Most Tauri apps put their UI inside a webview and ship native code through Tauri's IPC commands. Bodhi does **not** do that. Instead, it:

1. Boots a full HTTP server inside the Tauri process (the same server binary you'd run in Docker).
2. Opens your **system browser** to `http://localhost:1135` on first launch.
3. Hides the Tauri window when you close it — the server keeps running in the background, accessible from the system tray.

This means your Bodhi UI runs in the browser you already use (with extensions, devtools, password manager) instead of an embedded webview. The same OpenAI / Anthropic / Gemini / MCP endpoints are reachable on `localhost:1135` exactly as they would be from a Docker deployment.

There is no separate "frontend" and "backend" to wire together — the desktop bundle is just the server plus a tray icon.

## System tray

Once Bodhi is running, look for the icon in your menu bar (macOS) or system tray (Windows / Linux). It exposes two items:

- **Open Homepage** — opens `http://localhost:1135` in your default browser.
- **Quit** — gracefully shuts the server down and exits.

Closing the main window does **not** quit the app; it hides the window. Use **Quit** from the tray to fully exit.

The browser is opened automatically the first time you launch the app. Subsequent launches drop straight to the tray; click **Open Homepage** when you want the UI.

## Where Bodhi stores things

Bodhi keeps its own state under `~/.bodhi/`, and reuses your HuggingFace cache for actual model weights so other tools can share them.

```text
~/.bodhi/
├── bodhi.db                # SQLite — users, tokens, MCP instances, requests
├── session.db              # SQLite — login sessions
├── settings.yaml           # YAML overrides for app settings
├── aliases/                # Local model alias definitions (one file per alias)
└── logs/                   # Rotating log files

~/.cache/huggingface/hub/   # GGUF model files (HF_HOME default)
```

A few notes:

- **`~/.bodhi/`** is the value of `BODHI_HOME`. Back this folder up if you want to preserve your config, users, tokens, and aliases. Coupled with the encryption key (see below), it's everything Bodhi needs to come back up on a new machine.
- **Aliases** are markdown / YAML files in `~/.bodhi/aliases/`. Hand-editing is supported but not recommended — use the UI under `/ui/models`.
- **Model files** (the GGUFs themselves) are _not_ in `~/.bodhi`. They live in the HuggingFace cache so `transformers`, `llama.cpp`, and Bodhi can share them.
- **Logs** live at `~/.bodhi/logs/`. Files rotate automatically; the active file is the most recently modified.

If you want to relocate `~/.bodhi`, set `BODHI_HOME` in your shell environment **before** launching the app. Same for `HF_HOME` to relocate the model cache. See [Reference → Environment Variables](/docs/reference/env-vars).

## Per-platform secret storage

Bodhi uses your operating system's native secret store to protect the **encryption key** that secures sensitive data (API keys for remote providers, OAuth client secrets, MCP credentials).

| Platform | Storage                                       |
| -------- | --------------------------------------------- |
| macOS    | Keychain (`login` keychain by default)        |
| Windows  | DPAPI / Windows Credential Store              |
| Linux    | Secret Service (GNOME Keyring, KWallet, etc.) |

You don't manage this directly — Bodhi creates and reads the entry on your behalf. If you ever reset your OS keychain, you'll need to clear `~/.bodhi/bodhi.db` and re-onboard, since the encryption key is gone.

For Docker, encryption is configured explicitly via `BODHI_ENCRYPTION_KEY`. The desktop app does not require you to set this.

## Logs and how to inspect them

The active log file is at:

- macOS: `~/.bodhi/logs/`
- Windows: `%USERPROFILE%\.bodhi\logs\`
- Linux: `~/.bodhi/logs/`

To follow logs live:

```bash
# macOS / Linux
tail -f ~/.bodhi/logs/*.log

# Windows PowerShell
Get-Content "$env:USERPROFILE\.bodhi\logs\*.log" -Wait
```

To raise log verbosity, set `BODHI_LOG_LEVEL=debug` in your environment before launching the app. To also send logs to stdout (useful when launching from a terminal), set `BODHI_LOG_STDOUT=true`. See [Observability](/docs/advanced/observability) for what the log levels mean.

## Updates

Bodhi notifies you in-app when a new version is available. The flow:

1. Quit the running app from the tray.
2. Download the new installer from your platform's link in [Install](/docs/install).
3. Run the installer (it overwrites the prior install).
4. Launch the new version.

`~/.bodhi/` is **preserved across upgrades** — your users, tokens, aliases, and settings come with you. Database migrations run automatically on first launch.

If something goes badly wrong, you can rename `~/.bodhi` (don't delete it yet) and relaunch — Bodhi creates a fresh home and walks you through the setup wizard. Once you're sure the new install is happy, restore the bits you need from the renamed folder.

## Common issues

- **Tray icon disappears** — On some Linux desktops, system tray support requires an extension (e.g. AppIndicator on GNOME). If you don't see the icon, run Bodhi from a terminal so you can quit cleanly.
- **Browser doesn't open on launch** — Bodhi calls the OS's default browser handler. If nothing happens, click **Open Homepage** from the tray.
- **Port 1135 already in use** — Another Bodhi instance is running, or another service grabbed the port. Quit the old instance from the tray, or set `BODHI_PORT` to another value before launching.
- **OAuth flow fails on first launch** — Bodhi needs network access to reach `id.getbodhi.app`. Check your firewall / VPN.

For anything else, see [Troubleshooting](/docs/support/troubleshooting).

## Where to next

- **[Docker](/docs/deployment/docker)** — if you're outgrowing the desktop and want a shared server.
- **[Reference → Environment Variables](/docs/reference/env-vars)** — the full list of `BODHI_*` and `HF_*` variables you can set before launching.
- **[Reference → Settings](/docs/reference/settings)** — runtime settings (editable in the UI) vs the env-var defaults.
