---
title: 'Deployment Overview'
description: 'Pick the right way to run Bodhi App — Tauri desktop or Docker single-tenant'
order: 0
---

# Deployment Overview

Bodhi App ships in two supported flavors: a **Tauri desktop app** for personal use, and a **Docker container** for shared, always-on inference. Both run the same server, expose the same APIs, and use the same OAuth2 authentication. This page helps you pick.

If you want the conceptual difference first, read [Concepts → Deployment Modes](/docs/concepts/deployment-modes). This page is the operator-facing decision sheet.

## Side-by-side

|                         | **Tauri Desktop**                                                  | **Docker (single-tenant)**                                         |
| ----------------------- | ------------------------------------------------------------------ | ------------------------------------------------------------------ |
| Who runs the OS process | You — launched like any other desktop app                          | A server you control (VPS, homelab, GPU box, cloud VM)             |
| Where data lives        | `~/.bodhi/` on your home directory                                 | A Docker volume mounted at `/data` (configurable via `BODHI_HOME`) |
| Single vs multi-user    | Single user (you, on this machine)                                 | Multiple users with separate accounts and roles                    |
| OAuth required          | Yes — first launch walks you through admin sign-in                 | Yes — first user becomes admin; others request access              |
| GPU access              | Native — Metal on macOS, CUDA on Windows/Linux (variant-dependent) | Pick a GPU variant (CUDA, ROCm, Vulkan, MUSA, Intel, CANN)         |
| Network reach           | `localhost:1135` only — not exposed externally                     | Any host/port you bind; pair with a reverse proxy for TLS          |
| Update mechanism        | Download a new installer; `~/.bodhi` is preserved                  | `docker pull` the new tag and recreate the container               |
| Intended use case       | Personal use, dev workstations, evaluation                         | Team usage, always-on inference, GPU servers, sharing local models |

## Pick yours

```
Are you the only user, on your own machine?
├── Yes → Tauri Desktop
│         (/docs/deployment/desktop)
└── No
    ├── Need TLS / a public hostname?
    │   └── Docker + reverse proxy
    │       (/docs/deployment/docker, /docs/deployment/reverse-proxy)
    └── Internal LAN / homelab only?
        └── Docker (no proxy needed)
            (/docs/deployment/docker)
```

In short:

- **Choose desktop** if you want zero ops and the lowest friction. It's also the fastest way to evaluate Bodhi.
- **Choose Docker** if you want a shared, always-on server, want GPU acceleration on a non-laptop machine, or want to put it behind a domain name with TLS.

## What this section covers

- **[Desktop](/docs/deployment/desktop)** — Tauri specifics: system tray, browser auto-launch, the `~/.bodhi` layout, per-platform secret storage, log locations, update flow.
- **[Docker](/docs/deployment/docker)** — Every published variant, `docker run` examples, required environment variables, volume mounts, and a `docker-compose` template.
- **[Reverse Proxy](/docs/deployment/reverse-proxy)** — TLS termination and rate limiting in front of Docker, with Nginx and Caddy examples. Per the Bodhi security model, **rate limiting is the responsibility of the reverse proxy, not the app**.

## What this section does not cover

- **Multi-tenant production deployment** — exists internally for `getbodhi.app` itself, but is out of scope for this documentation.
- **The NAPI library** (`@bodhiapp/app-bindings`) — for embedding the server inside another Node.js process; covered in the developer docs, not here.
- **Standalone HTTP binary** (`bodhi serve` outside Docker) — supported for development, but not a recommended deployment path.

If you've picked a path, jump to **[Desktop](/docs/deployment/desktop)** or **[Docker](/docs/deployment/docker)** next.
