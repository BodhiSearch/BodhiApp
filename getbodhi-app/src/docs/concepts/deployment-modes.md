---
title: 'Deployment Modes'
description: 'Tauri desktop vs Docker single-tenant — the two ways to run Bodhi App, and how to pick'
order: 1
---

# Deployment Modes

Bodhi App ships in two flavors. Both run the same server, expose the same APIs, and use the same OAuth2 authentication — but they target very different workflows. This page explains the differences so you can pick the right one.

## At a glance

|                             | **Tauri Desktop**                                                          | **Docker (single-tenant)**                                                          |
| --------------------------- | -------------------------------------------------------------------------- | ----------------------------------------------------------------------------------- |
| Who runs the OS process?    | You — it's a desktop app on your machine.                                  | A server you control (your VPS, a homelab box, a cloud VM, your team's NAS).        |
| Where does data live?       | `~/.bodhi/` on your home directory.                                        | A Docker volume you mount (typically `/data` inside the container).                 |
| Who can reach the UI / API? | Just you, on `localhost:1135`.                                             | Anyone you allow — bind to a port, put a reverse proxy in front for TLS.            |
| Single user or many?        | Single user (you).                                                         | Multiple users with separate accounts and roles.                                    |
| OAuth required?             | Yes — first launch walks you through admin sign-in.                        | Yes — first user becomes admin, others request access.                              |
| Updates?                    | Download a new installer (Bodhi notifies you in-app).                      | `docker pull` the new tag and recreate the container.                               |
| Best for...                 | Personal use, dev workstations, "I just want to run a model on my laptop." | Team usage, always-on inference, GPU servers, sharing local models with co-workers. |
| GPU acceleration            | Yes — Metal on macOS, CUDA on Windows/Linux (variant-dependent).           | Yes — pick a Docker variant (CUDA, ROCm, Vulkan, MUSA, Intel, CANN).                |

## Tauri Desktop

Bodhi runs as a native desktop app on macOS (Intel + Apple Silicon), Windows, and Linux. Under the hood it's the same server binary — wrapped in Tauri, with a system-tray icon and an embedded UI shell.

**What's different from Docker:**

- The server starts when you launch the app and stops when you quit. There's a tray icon to keep it running in the background.
- Data lives in `~/.bodhi/` (logs, database, downloaded model aliases). GGUF files themselves go in your HuggingFace cache (`~/.cache/huggingface/hub/`) so they're shared with other tools.
- The app pops a browser to `http://localhost:1135` automatically.
- Only you, on this machine, can reach it. No port is exposed externally.
- macOS: encryption keys are stored in Keychain. Windows: in DPAPI. Linux: in the secret service or a fallback file.

**Pick this if:** you want zero ops, a single-user experience, and the lowest possible friction. It's also the fastest way to evaluate Bodhi.

For installation, see [Install → macOS / Windows / Linux](/docs/install). The full deployment page lands at `/docs/deployment/desktop` in a later phase.

## Docker (single-tenant)

You run Bodhi as a container — typically `ghcr.io/bodhisearch/bodhiapp:latest-cpu` or one of the GPU variants — on a server you control. One Bodhi instance serves multiple human users, each with their own login, role, and API tokens.

**What's different from Desktop:**

- You manage the process: `docker run`, `docker compose`, systemd, Kubernetes — your call.
- You bind a port (default 1135) and decide who can reach it. Production deployments typically put a reverse proxy (Caddy, nginx, Traefik) in front for TLS termination.
- Data lives in a mounted volume. You set `BODHI_HOME` on the container and back up that path.
- Multiple users can sign up via OAuth. The first one becomes admin; others submit access requests that admins/managers approve.
- You pick a variant matching your hardware: AMD64 / ARM64 CPU, NVIDIA CUDA, AMD ROCm, cross-vendor Vulkan, Moore Threads MUSA, Intel GPU, or Huawei CANN.

**Pick this if:** you want a shared, always-on inference server for a team, or you want to run on a GPU box that isn't your daily-driver laptop.

For deployment instructions, see [Docker Deployment](/docs/deployment/docker). The decision-flow page (Tauri vs Docker, with reverse-proxy guidance) lands at `/docs/deployment/overview` in a later phase.

## What both modes share

- **Same APIs.** Every endpoint described in [API Compatibility](/docs/concepts/api-compatibility) works identically on both.
- **Same auth model.** OAuth2 PKCE login, the four roles, scoped API tokens — see [Auth and Roles](/docs/concepts/auth-and-roles).
- **Same MCP infrastructure.** Each user has their own MCP instances; admins maintain a shared catalog of pre-registered servers — see [MCP Overview](/docs/concepts/mcp-overview).
- **Same model concepts.** Local model aliases, GGUF files, API model aliases — see [Models, Aliases, and Files](/docs/concepts/models-aliases-files).
- **Same swagger.** The running app exposes its full schema at `/swagger-ui` regardless of mode.

## Common questions

**Can I switch later?**
Yes — there's no lock-in on either side. Both modes use the same SQLite/Postgres schema (Desktop is SQLite; Docker can be either). The simplest migration is "export aliases and API model configs, re-create on the other side." Direct DB-file portability between Desktop and Docker is not officially supported.

**Can I run both at the same time?**
Yes — they're just two server processes. They won't share data, though.

**What about reverse proxies, TLS, rate limits?**
That's a Docker-mode topic. The full reverse-proxy guide lands at `/docs/deployment/reverse-proxy` in a later phase. Short version: terminate TLS upstream, forward `Host` and `X-Forwarded-*` headers, and set `BODHI_PUBLIC_HOST` / `BODHI_PUBLIC_SCHEME` so OAuth callback URLs come back to the right place.

Ready for the next building block? Head to **[Models, Aliases, and Files](/docs/concepts/models-aliases-files)** to disambiguate the three things "model" can mean in Bodhi.
