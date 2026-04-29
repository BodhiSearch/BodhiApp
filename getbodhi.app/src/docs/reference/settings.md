---
title: 'Settings Precedence'
description: 'How Bodhi merges settings from CLI flags, environment, database, settings.yaml, and built-in defaults — and which two settings the UI can edit'
order: 1
---

# Settings Precedence

Bodhi resolves every configuration key from multiple sources and picks the _highest-priority_ one that has a value. This page covers the resolution order, which sources are writable, and the most common footgun: saving a setting in the UI that a higher-priority source then silently overrides.

For the alphabetical list of every variable Bodhi reads, see [Environment Variables](/docs/reference/env-vars).

## The six sources, in priority order

When you ask for a setting (e.g. `BODHI_PORT`), Bodhi walks these sources from top to bottom and returns the first one that has a value.

| #   | Source           | Where it comes from                                                                                                                                                              |
| --- | ---------------- | -------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| 1   | **System**       | Build-time constants and platform-derived values. Highest priority — cannot be overridden. Examples: `BODHI_VERSION`, `BODHI_COMMIT_SHA`, `BODHI_HOME` once resolved at startup. |
| 2   | **CommandLine**  | Flags passed to the Bodhi CLI. Used by the Tauri launcher and Docker entrypoint to inject derived values.                                                                        |
| 3   | **Environment**  | The process environment — `BODHI_PORT=2000`, `HF_TOKEN=hf_...`, etc. This is where you'll set most things.                                                                       |
| 4   | **Database**     | Values written to the SQLite (or PostgreSQL) settings table at runtime — the only writable source available to the running app.                                                  |
| 5   | **SettingsFile** | The YAML file at `$BODHI_HOME/settings.yaml`. Useful for desktop installs that want to keep config out of shell environment.                                                     |
| 6   | **Default**      | Built-in fallbacks. Lowest priority.                                                                                                                                             |

So a value set in `Environment` beats one stored in the `Database`, which beats `SettingsFile`, which beats `Default`. `System` and `CommandLine` win over everything.

## Editable at runtime

Only two settings can be changed via the running app's API or `/ui/settings/` page:

- `BODHI_EXEC_VARIANT` — switch between bundled inference variants (e.g. `cpu` ↔ `cuda`)
- `BODHI_KEEP_ALIVE_SECS` — seconds a loaded model stays warm before being unloaded

Attempting to PUT or DELETE any other key returns an error. Edits go to the `Database` source — they survive process restarts but, per the priority list above, **lose to anything set in `Environment`, on the `CommandLine`, or by the `System`**.

To change anything else, set the corresponding environment variable (or write `settings.yaml`) and restart the process.

## The "saved but no effect" footgun

A common surprise:

1. You set `BODHI_KEEP_ALIVE_SECS=120` in your `docker-compose.yml`.
2. Later, an admin opens `/ui/settings/`, changes Keep-Alive to `600`, and clicks Save.
3. The save succeeds, the database now holds `600`, the UI shows `600`. But the server still uses `120`.

This is working as designed — `Environment` (priority 3) outranks `Database` (priority 4). The UI hides this from you up to a point: the active source is shown next to each setting on `/ui/settings/`. If the source for Keep-Alive reads `Environment` after you save, the saved value is ignored until the env var is removed.

To fix:

- **Preferred** — remove the env var (or YAML entry) and restart. Database value then becomes active.
- Or set the new value via the env var/YAML directly and skip the UI save.

This applies to both editable-at-runtime keys. It does not apply to the desktop install when nothing is set in the environment, since `Default` always loses to `Database`.

## Where each setting _can_ be set

| Source                                     | Read by Bodhi | Writable from a running Bodhi                                       |
| ------------------------------------------ | ------------- | ------------------------------------------------------------------- |
| System                                     | always        | no                                                                  |
| CommandLine                                | always        | no                                                                  |
| Environment                                | always        | no — fixed at process start                                         |
| Database                                   | always        | yes — but only for `BODHI_EXEC_VARIANT` and `BODHI_KEEP_ALIVE_SECS` |
| SettingsFile (`$BODHI_HOME/settings.yaml`) | always        | no — edit the file and restart                                      |
| Default                                    | always        | no                                                                  |

## Inspecting what's active

The `/ui/settings/` page renders one row per setting with three columns: the **current value**, the **default value**, and the **source** that produced the current value. If a setting reads its value from a source higher than `Database`, editing it in the UI is a no-op for the runtime — only the stored value changes.

Three forms you'll see in that source column most often:

- `default` — nothing is overriding this; the built-in fallback is in effect.
- `environment` — set via env var; UI saves are stored but ignored at runtime.
- `database` — set via the UI or directly in the DB; this value is active.

## When values change

For settings the runtime reads on every request (e.g. `BODHI_KEEP_ALIVE_SECS`, `BODHI_LOG_LEVEL`), changes apply on the next read. For settings consumed during startup only (`BODHI_HOME`, `BODHI_PORT`, network settings, database URLs), you have to restart the process. The two editable-at-runtime keys are deliberately scoped to "safe to change live."

## Related

- [Environment Variables](/docs/reference/env-vars) — full alphabetical reference.
- [App settings page](/docs/features/settings/app-settings) — the in-app UI walkthrough.
- [Observability](/docs/advanced/observability) — how to verify which value is currently active by reading logs.
