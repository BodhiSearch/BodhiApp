---
title: 'Environment Variables'
description: 'Alphabetical reference for every BODHI_* / HF_* environment variable Bodhi App reads at startup or runtime'
order: 0
---

# Environment Variables

Bodhi App reads its configuration from environment variables, a `settings.yaml` file under `$BODHI_HOME`, the SQLite database (`bodhi.sqlite`), and built-in defaults. This page is the alphabetical lookup table — for the _order_ in which those sources are merged, see [Settings precedence](/docs/reference/settings).

A handful of variables can be edited from the running app's `/ui/settings/` page. The vast majority must be set before the process starts (typically via `docker run -e ...`, a Docker Compose file, or a shell export). Variables marked as Editable at runtime survive process restarts via the database; everything else is re-read from the environment on every boot.

> Multi-tenant deployment is out of scope for these docs — the `BODHI_MULTITENANT_*` entries are listed for completeness only.

## Quick jump

- [Core](#core) — home dir, env type, app type, deployment mode
- [Network](#network) — host, port, scheme + their `PUBLIC_*` siblings
- [Auth](#auth) — OAuth provider, encryption, multitenant
- [Database](#database) — session and app DB URLs
- [llama.cpp / inference](#llamacpp--inference) — variant, args, keep-alive
- [Logging](#logging) — log level, log dir, stdout flag
- [HuggingFace](#huggingface) — token, cache home
- [Platform](#platform) — RunPod helpers
- [Editable at runtime](#editable-at-runtime) — the two settings the UI can change

## Core

| Variable           | Type   | Default                                             | Description                                                                                                                                          | Editable at runtime |
| ------------------ | ------ | --------------------------------------------------- | ---------------------------------------------------------------------------------------------------------------------------------------------------- | ------------------- |
| `BODHI_APP_TYPE`   | string | set by build                                        | Application form factor — `native` (Tauri desktop) or `container` (Docker / standalone server). Resolved at build time, not user-editable.           | —                   |
| `BODHI_COMMIT_SHA` | string | set by build                                        | Git commit the binary was built from. Surfaced in the UI footer for support.                                                                         | —                   |
| `BODHI_DEPLOYMENT` | string | `standalone`                                        | Deployment mode. `standalone` (default) for desktop and single-tenant Docker. `multi_tenant` exists for hosted deployments and is out of scope here. | —                   |
| `BODHI_ENV_TYPE`   | string | set by build                                        | `production` or `development`. Affects log-source selection and debug helpers.                                                                       | —                   |
| `BODHI_HOME`       | path   | `~/.bodhi` (desktop), `/data/bodhi` (Docker images) | Root directory for all Bodhi state — settings, DBs, logs, secrets. Must be writable.                                                                 | —                   |
| `BODHI_VERSION`    | string | set by build                                        | App version string.                                                                                                                                  | —                   |

## Network

The `BODHI_HOST` / `BODHI_PORT` / `BODHI_SCHEME` triple controls **where the server binds**. The `BODHI_PUBLIC_*` triple controls **how the server identifies itself** in OAuth redirect URLs, generated callback links, and the embedded Swagger spec — useful when Bodhi sits behind a reverse proxy.

| Variable                   | Type   | Default                      | Description                                                                                                                                                              | Editable at runtime |
| -------------------------- | ------ | ---------------------------- | ------------------------------------------------------------------------------------------------------------------------------------------------------------------------ | ------------------- |
| `BODHI_CANONICAL_REDIRECT` | bool   | `true`                       | When `true`, requests that arrive on a non-canonical host are redirected to the public URL. Disable when fronting Bodhi with a load balancer that already canonicalizes. | —                   |
| `BODHI_HOST`               | string | `0.0.0.0`                    | Bind address for the HTTP listener.                                                                                                                                      | —                   |
| `BODHI_PORT`               | u16    | `1135`                       | Port the HTTP listener binds to.                                                                                                                                         | —                   |
| `BODHI_PUBLIC_HOST`        | string | falls back to `BODHI_HOST`   | The hostname clients use to reach Bodhi (e.g. `bodhi.example.com` behind a proxy). Used to build OAuth callback URLs.                                                    | —                   |
| `BODHI_PUBLIC_PORT`        | u16    | falls back to `BODHI_PORT`   | Public-facing port. Set to `443` when terminating TLS at a proxy.                                                                                                        | —                   |
| `BODHI_PUBLIC_SCHEME`      | string | falls back to `BODHI_SCHEME` | Public-facing scheme — set to `https` when behind a TLS-terminating proxy.                                                                                               | —                   |
| `BODHI_SCHEME`             | string | `http`                       | Bind scheme. The server itself currently serves plain HTTP and expects TLS to be terminated upstream.                                                                    | —                   |

## Auth

| Variable                          | Type   | Default                                         | Description                                                                                                                                                       | Editable at runtime |
| --------------------------------- | ------ | ----------------------------------------------- | ----------------------------------------------------------------------------------------------------------------------------------------------------------------- | ------------------- |
| `BODHI_AUTH_REALM`                | string | `bodhi` (production), `bodhi-dev` (development) | Keycloak realm under the auth provider hosting Bodhi's identity.                                                                                                  | —                   |
| `BODHI_AUTH_URL`                  | string | `https://id.getbodhi.app`                       | Base URL of the OAuth2 / OIDC provider. Bodhi composes the issuer, login, and token URLs from this plus `BODHI_AUTH_REALM`.                                       | —                   |
| `BODHI_ENCRYPTION_KEY`            | string | derived from OS keychain (desktop)              | Master key used to encrypt secrets at rest (API model keys, MCP credentials). On Docker you must set this explicitly; rotating it invalidates all stored secrets. | —                   |
| `BODHI_MULTITENANT_CLIENT_ID`     | string | —                                               | OAuth client ID for multi-tenant deployments. **Multi-tenant is out of scope for these docs.**                                                                    | —                   |
| `BODHI_MULTITENANT_CLIENT_SECRET` | string | —                                               | OAuth client secret for multi-tenant deployments. **Multi-tenant is out of scope for these docs.**                                                                | —                   |

## Database

| Variable               | Type   | Default                               | Description                                                                                                                     | Editable at runtime |
| ---------------------- | ------ | ------------------------------------- | ------------------------------------------------------------------------------------------------------------------------------- | ------------------- |
| `BODHI_APP_DB_URL`     | string | `sqlite://$BODHI_HOME/bodhi.sqlite`   | Connection URL for the application DB (users, tokens, models, MCPs, settings). Set to a `postgres://...` URL to use PostgreSQL. | —                   |
| `BODHI_SESSION_DB_URL` | string | `sqlite://$BODHI_HOME/session.sqlite` | Connection URL for the session store.                                                                                           | —                   |

## llama.cpp / inference

These control the local inference subsystem. See [Inference Stack](/docs/advanced/inference-stack) for the variant matrix and per-hardware tuning notes.

| Variable                 | Type   | Default                                 | Description                                                                                                                                                                                    | Editable at runtime |
| ------------------------ | ------ | --------------------------------------- | ---------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- | ------------------- |
| `BODHI_EXEC_LOOKUP_PATH` | path   | bundled (desktop) / `/app/bin` (Docker) | Directory tree that contains the `target/<variant>/<exec>` binaries.                                                                                                                           | —                   |
| `BODHI_EXEC_NAME`        | string | platform-default                        | Filename of the llama.cpp server binary (e.g. `llama-server`).                                                                                                                                 | —                   |
| `BODHI_EXEC_TARGET`      | string | platform-default                        | Build target sub-directory (e.g. `aarch64-apple-darwin`).                                                                                                                                      | —                   |
| `BODHI_EXEC_VARIANT`     | option | image-default (`cpu`, `cuda`, etc.)     | Active inference variant. Choose from `BODHI_EXEC_VARIANTS`. The Docker tag implies a default; override only if the image bundles multiple.                                                    | ✓                   |
| `BODHI_EXEC_VARIANTS`    | csv    | image-default                           | Comma-separated list of variants the binary bundle supports. Read-only; emitted by the build.                                                                                                  | —                   |
| `BODHI_KEEP_ALIVE_SECS`  | i64    | `300`                                   | Seconds the loaded model stays warm in VRAM after the last request. Lower it on small GPUs; raise it for bursty workloads.                                                                     | ✓                   |
| `BODHI_LLAMACPP_ARGS`    | string | —                                       | Extra CLI args forwarded to every spawn of the llama.cpp server (e.g. `--n-gpu-layers 999`). Variant-specific overrides via `BODHI_LLAMACPP_ARGS_<VARIANT>` (e.g. `BODHI_LLAMACPP_ARGS_CUDA`). | —                   |

## Logging

| Variable           | Type   | Default            | Description                                                                                              | Editable at runtime |
| ------------------ | ------ | ------------------ | -------------------------------------------------------------------------------------------------------- | ------------------- |
| `BODHI_LOGS`       | path   | `$BODHI_HOME/logs` | Directory for daily-rotated log files.                                                                   | —                   |
| `BODHI_LOG_LEVEL`  | option | `warn`             | Log threshold. One of `error`, `warn`, `info`, `debug`, `trace`.                                         | —                   |
| `BODHI_LOG_STDOUT` | bool   | `false`            | Mirror logs to stdout in addition to `BODHI_LOGS`. Recommended for Docker so `docker logs` shows output. | —                   |

See [Observability](/docs/advanced/observability) for log routing patterns.

## HuggingFace

| Variable   | Type   | Default                   | Description                                                                     | Editable at runtime |
| ---------- | ------ | ------------------------- | ------------------------------------------------------------------------------- | ------------------- |
| `HF_HOME`  | path   | `$BODHI_HOME/huggingface` | HuggingFace cache root. Bodhi reads `$HF_HOME/hub` for downloaded GGUF models.  | —                   |
| `HF_TOKEN` | string | —                         | HuggingFace access token used for gated repos and to lift download rate limits. | —                   |

## Platform

| Variable          | Type   | Default            | Description                                                                                                                                                                                                                    | Editable at runtime |
| ----------------- | ------ | ------------------ | ------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------ | ------------------- |
| `BODHI_ON_RUNPOD` | bool   | `false`            | Enables RunPod-aware behavior — `BODHI_PUBLIC_SCHEME` defaults to `https`, `BODHI_PUBLIC_HOST` becomes `<pod-id>-<port>.proxy.runpod.net`, and `BODHI_PUBLIC_PORT` becomes `443`. Requires `RUNPOD_POD_ID` to also be present. | —                   |
| `RUNPOD_POD_ID`   | string | injected by RunPod | Pod identifier surfaced by the RunPod runtime.                                                                                                                                                                                 | —                   |

## Editable at runtime

Only two settings are writable from the running app:

- `BODHI_EXEC_VARIANT`
- `BODHI_KEEP_ALIVE_SECS`

The `/ui/settings/` page surfaces these alongside their current value and the source the active value came from. PUT/DELETE on any other key returns an error. Read [Settings precedence](/docs/reference/settings) for the gotcha around saving from the UI when a higher-priority source already wins.

## Where to set them

- **Tauri desktop** — set in your shell environment before launching, or edit `~/.bodhi/settings.yaml`. The two editable-at-runtime settings can be changed from the in-app settings page.
- **Docker** — pass with `-e` on `docker run` or list under `environment:` in Compose. See [Docker deployment](/docs/deployment/docker).
- **Reverse proxy** — set the `BODHI_PUBLIC_*` triple to your external hostname/scheme/port. See [Reverse proxy](/docs/deployment/reverse-proxy).

## Related

- [Settings precedence](/docs/reference/settings) — how these variables interact with `settings.yaml`, the database, and CLI overrides.
- [App settings page](/docs/features/settings/app-settings) — the in-app UI for the two editable settings.
- [Observability](/docs/advanced/observability) — log levels and where logs go.
- [Inference stack](/docs/advanced/inference-stack) — what the `BODHI_EXEC_*` and `BODHI_LLAMACPP_*` variables actually do.
