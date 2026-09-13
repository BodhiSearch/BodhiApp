---
title: 'Observability'
description: 'Logs, settings introspection, the background queue, and what is honest about today’s observability gaps in Bodhi App'
order: 4
---

# Observability

This page covers what you can see about a running Bodhi App: where the logs live, how to introspect the settings the app actually loaded, how to watch background work, and how to audit token usage. It also says honestly what isn't yet exposed — there are real gaps and you should plan around them.

## Logs

Bodhi writes structured logs (HTTP requests, internal events, llama-server child output) using a tracing pipeline that rotates daily.

### Where logs live

Logs are written under `$BODHI_HOME/logs/`, with files rotated daily:

```
$BODHI_HOME/
└── logs/
    ├── bodhi.log.2026-04-26
    ├── bodhi.log.2026-04-27
    └── bodhi.log.2026-04-28
```

The directory is configurable via `BODHI_LOGS` if you want logs on a separate disk from the app data. Old files are not auto-pruned — you're responsible for retention (a `logrotate` or `find -mtime` job is the simplest answer).

### Controlling verbosity

`BODHI_LOG_LEVEL` accepts `trace`, `debug`, `info`, `warn`, `error` (default `warn`). You can also use the `tracing` filter syntax to scope per-module:

```bash
# Default: warnings only
BODHI_LOG_LEVEL=warn

# Debug everything — verbose, useful while diagnosing a problem
BODHI_LOG_LEVEL=debug

# Quiet most noise, but turn on info-level for HTTP and auth
BODHI_LOG_LEVEL="warn,bodhi=info,auth_middleware=info"
```

`BODHI_LOG_STDOUT` (`true`/`false`, default `false`) mirrors logs to stdout in addition to the file. Turn this on inside Docker, where `docker logs` is the natural place to read them. Leave it off on desktop, where the file is fine.

### Reading HTTP traces

At `info` level or below, every request emits one line with method, path, status, and latency:

```
2026-04-28T12:34:56.789Z  INFO bodhi::http: 200 GET /v1/models 12ms
2026-04-28T12:34:58.012Z  INFO bodhi::http: 401 POST /v1/chat/completions 3ms
2026-04-28T12:35:01.456Z  INFO bodhi::http: 200 POST /v1/chat/completions 4523ms
```

Repeated `401`/`403` from the same client IP is the classic signal of a misconfigured token or a brute-force attempt. The `whats hammering my server` answer almost always lives in this stream.

llama-server child processes have their stdout/stderr captured into the same log file, prefixed so you can tell which alias produced which line.

## Settings introspection

The settings page at `/ui/settings/` shows every effective setting plus _where it came from_. Bodhi composes settings from multiple sources; the source field is the answer to "why is this value what it is?":

- **System** — set internally by the binary at startup (e.g. version metadata).
- **CommandLine** — passed as a CLI argument to the binary.
- **Environment** — read from the environment at process start.
- **Database** — set via the settings UI; persisted in the app DB.
- **SettingsFile** — read from the YAML settings file on disk.
- **Default** — the built-in default; nothing else overrode it.

Precedence is roughly: CommandLine > Environment > Database > SettingsFile > Default, with a small System layer for things that are not user-configurable. Database wins over SettingsFile so that runtime UI edits stick across restarts.

Only a small set of settings is editable from the UI at runtime — currently `BODHI_EXEC_VARIANT` and `BODHI_KEEP_ALIVE_SECS`. The rest are read-only on that page. The full matrix and precedence rules live in [Reference → Settings](/docs/reference/settings).

## The background queue

Some operations don't return synchronously — model downloads, in particular, can take many minutes. Bodhi exposes a small queue endpoint for visibility:

```bash
GET /bodhi/v1/queue
```

The response reports queue state ("idle" or "running"), letting you tell whether a download is still in progress or has stalled. The model-files page and the model-downloads page in the UI poll this endpoint to drive their progress indicators.

For per-job state (download progress on a specific repo+filename), use the model-downloads UI; it's the same data shown in a more useful form.

## API token usage

Each API token row carries two timestamps that help with audit:

- `created_at` — when the token was minted.
- `last_used_at` — when the token last successfully authenticated a request.

Both are visible at `/ui/tokens/`. A `last_used_at` that hasn't moved in months is the easiest signal of a token to revoke. The page also shows the token's status (Active / Inactive) — flip a token to Inactive instead of deleting if you want to keep its audit row but block further use.

For the API surface (creating, listing, updating tokens) see [Features → API Tokens](/docs/features/auth/api-tokens).

## What's NOT yet available

Be honest about gaps. Today, Bodhi App does not provide:

- **Aggregated request metrics.** No Prometheus endpoint, no per-endpoint latency histogram, no requests/sec dashboard. If you need this, scrape your reverse proxy's access log instead — every request is mediated by the proxy, and it's a much better metrics source than the app for traffic patterns.
- **Per-model cost or token-usage tracking.** Bodhi doesn't sum up tokens-per-user or attach a cost-per-call to API model usage. The upstream provider (OpenAI/Anthropic/Gemini) is the source of truth for billing.
- **Audit log export.** Logs are flat files. There is no built-in shipper to S3 / a SIEM / a log warehouse. Configure your host's log collector if you need that.
- **Per-tenant resource quotas.** No hard caps on chat throughput or model-download size in the single-tenant deployment shapes this documentation covers.
- **Live inference metrics from the llama-server child.** The child's own `/metrics` endpoint isn't exposed through the gateway. If you need it, run llama-server side-by-side outside of Bodhi for benchmarking.

These are real limitations, not oversights. The fastest workaround for most of them is "instrument the reverse proxy and the host" — that's where the data is rich and the tooling is mature.

## A practical observability checklist

For a self-hosted deployment, the minimum useful observability setup is:

1. **Tail the proxy log** for traffic patterns and per-IP behaviour.
2. **Tail `$BODHI_HOME/logs/`** at `info` level for HTTP responses and auth decisions.
3. **Set `BODHI_LOG_STDOUT=true`** in Docker so `docker logs bodhi` works.
4. **Bookmark `/ui/settings/`** for "why is the app behaving like this?" debugging.
5. **Bookmark `/ui/tokens/`** for periodic token-rotation review.
6. **Watch GPU/CPU at the host level** — `nvidia-smi`, `nvtop`, `htop` — Bodhi doesn't surface this.

For wider context:

- [Architecture](/docs/advanced/architecture) — request flow, where to look when a request fails.
- [Performance Tuning](/docs/advanced/performance-tuning) — when slow is "the model is huge" vs. "something is misconfigured."
- [Reference → Settings](/docs/reference/settings) — settings precedence and the editable subset.
- [Reference → Environment Variables](/docs/reference/env-vars) — full env-var matrix.
