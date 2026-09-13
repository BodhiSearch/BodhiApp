---
title: 'Troubleshooting'
description: 'Symptom-to-fix guide for the most common issues installing, running, and integrating with Bodhi App'
order: 1
---

# Troubleshooting

This page is organized by _where_ the problem shows up — startup, login, models, API calls, MCPs, settings, performance. For each, we list the most common symptoms with a likely cause and a concrete fix.

If you can't find your issue here, check `$BODHI_HOME/logs` first (or stdout when running with `BODHI_LOG_STDOUT=true`), then the [GitHub Issues](https://github.com/BodhiSearch/BodhiApp/issues) tracker, then Discord.

## Startup

### App won't launch / exits immediately

- **Cannot create or write to `BODHI_HOME`.** The default home is `~/.bodhi` on desktop and `/data/bodhi` in Docker. If the directory exists but is owned by a different user, the process exits with a permission error.
  - Fix: `chown -R $USER ~/.bodhi` (desktop) or `chown -R 1000:1000 /data/bodhi` (Docker, where the image runs as UID 1000).
- **Port 1135 already bound.** Bodhi binds to `0.0.0.0:1135` by default; another local process or another Bodhi instance is already there.
  - Fix: stop the conflicting process, or run on a different port: `BODHI_PORT=1136`. See [Environment variables](/docs/reference/env-vars).
- **Disk full.** SQLite cannot write WAL/journal — startup fails before logs initialize.
  - Fix: free space in `BODHI_HOME` and on `/tmp`.

### Secret store rejected the encryption key

Bodhi stores the master encryption key in the OS-native credential store on desktop. If the OS denies access, Bodhi cannot decrypt your stored secrets (API model keys, MCP credentials).

- **macOS Keychain prompt was denied.** Bodhi prompts the first time it accesses Keychain.
  - Fix: open Keychain Access, find the `bodhi` entry, set "Always allow" for the Bodhi binary. Re-launch.
- **Windows Credential Manager errors.** Roaming profiles or group policy can block Credential Manager.
  - Fix: confirm the user has access to `Control Panel → Credential Manager → Windows Credentials`. If you control the machine, remove restrictive policies and retry.
- **Linux secret-service unavailable.** Bodhi uses the freedesktop secret-service via libsecret (gnome-keyring or KWallet). On a headless server this isn't running.
  - Fix: install `gnome-keyring` and unlock it at startup, or run Bodhi in Docker where the encryption key is taken from `BODHI_ENCRYPTION_KEY` directly.

### Pre-init crashes (no log file)

If the app crashes before logging initializes, errors go to stdout. Run from a terminal so they're visible:

```bash
# Desktop on macOS
/Applications/Bodhi.app/Contents/MacOS/Bodhi
# Docker
docker run --rm -e BODHI_LOG_STDOUT=true ...
```

## Authentication and login

### OAuth callback URL mismatch

Symptom: after clicking Sign In, the OAuth provider returns "redirect_uri does not match" or the browser hangs after the redirect.

- Cause: Bodhi advertises its callback URL using `BODHI_PUBLIC_SCHEME://BODHI_PUBLIC_HOST:BODHI_PUBLIC_PORT/ui/auth/callback`. If those don't match what your auth provider expects, the round-trip fails.
- Fix: behind a reverse proxy, set `BODHI_PUBLIC_SCHEME=https`, `BODHI_PUBLIC_HOST=bodhi.example.com`, and `BODHI_PUBLIC_PORT=443` (or omit `BODHI_PUBLIC_PORT` if it's `443`/`80`). See [Reverse proxy](/docs/deployment/reverse-proxy).

### Token expired / 401 on every API call

- Session cookies refresh silently as long as you're active. If you've been idle for a long time, the refresh token expires too — log in again.
- API tokens do not auto-refresh. If revoked or deactivated, all calls return 401 with `code: token_expired` or `token_invalid`. Mint a new one from `/ui/tokens/`.

### CORS preflight failures from a browser-based client

Symptom: `OPTIONS` to a Bodhi endpoint fails with no body, or the browser logs "CORS policy: No 'Access-Control-Allow-Origin' header."

- Cause: per-route CORS. Session-only APIs (under `/bodhi/v1/`) intentionally have restrictive CORS. External-app APIs under `/bodhi/v1/apps/` accept arbitrary origins.
- Fix: confirm you're hitting the right endpoint group. For browser-based clients calling inference endpoints, use `/v1/chat/completions` (permissive). For UI APIs, the call should be same-origin.

### 401 vs 403 — authentication vs authorization

- **401** = "I don't know who you are." Token missing, malformed, expired, or revoked.
- **403** = "I know who you are, but you can't do this." The endpoint requires a higher role or scope than your credential carries. Cross-check [Roles and scopes](/docs/reference/roles-and-scopes).

A particularly common 403: an API token with `User` scope calling a PowerUser-only endpoint. The token is valid; the scope just isn't enough.

## Models

### GGUF download stuck or fails

- **HuggingFace rate-limited.** Without `HF_TOKEN`, anonymous rate limits kick in fast on large models.
  - Fix: set `HF_TOKEN` to a free HuggingFace token. See [Environment variables](/docs/reference/env-vars).
- **Partial file in `$HF_HOME`.** A previous run died mid-download.
  - Fix: delete the offending file under `$HF_HOME/hub/models--<repo>--<name>/blobs/` and retry. Bodhi resumes from scratch.
- **Gated repo.** The model requires accepting a license on HuggingFace.
  - Fix: accept the license in your HF account, then retry with `HF_TOKEN` set to a token from the same account.

### llama.cpp fails to start when loading a model

- **CUDA driver mismatch.** The CUDA Docker variant requires a host driver matching its CUDA toolkit. A too-old host driver fails the runtime check.
  - Fix: upgrade the host NVIDIA driver, or switch to the CPU variant. See [Inference stack](/docs/advanced/inference-stack).
- **ROCm device not visible.** AMD GPU not exposed inside the container.
  - Fix: pass `--device=/dev/kfd --device=/dev/dri` and add the user to the `render` group on the host.
- **Model too large for VRAM.** Out-of-memory during load.
  - Fix: pick a smaller quant (e.g. Q4_K_M instead of Q8_0) or set `BODHI_LLAMACPP_ARGS=--n-gpu-layers 20` to keep some layers on CPU.

### "Model unloaded after a few minutes"

Bodhi unloads models after `BODHI_KEEP_ALIVE_SECS` (default 300s) of inactivity to free VRAM. The next request reloads — first response will feel slow.

- Fix: raise `BODHI_KEEP_ALIVE_SECS` from `/ui/settings/`. This setting is editable at runtime. See [Settings precedence](/docs/reference/settings).

### Windows: HuggingFace cache symlink errors

- Cause: HuggingFace's cache layout uses symlinks. On NTFS without developer mode, symlink creation needs Administrator.
- Fix: enable Windows Developer Mode (Settings → Privacy & Security → For developers), or run Bodhi as Administrator once to seed the cache, or use the Docker desktop runtime which bypasses this entirely.

## API calls

### "I'm sending the API key the chat UI shows but curl gets 401"

The chat UI configures SDKs that _require_ an `apiKey` parameter with a sentinel placeholder — it's not a real Bodhi credential. Bodhi recognizes the request via the session cookie that the browser sends alongside.

- Fix: do not paste the sentinel into your terminal. For curl, mint a real API token from `/ui/tokens/` and pass it as `Authorization: Bearer bodhiapp_...`.

### Different headers per provider format

| Endpoint family      | Auth header                                                   | Notes                                                                                                                          |
| -------------------- | ------------------------------------------------------------- | ------------------------------------------------------------------------------------------------------------------------------ |
| `/v1/*` (OpenAI)     | `Authorization: Bearer <bodhi token>`                         | Same header even though you're hitting an API model that proxies to a real OpenAI key — Bodhi handles the upstream credential. |
| `/anthropic/v1/*`    | `x-api-key: <bodhi token>` or `Authorization: Bearer ...`     | Both accepted. Anthropic SDKs default to `x-api-key`.                                                                          |
| `/v1beta/*` (Gemini) | `x-goog-api-key: <bodhi token>` or query `?key=<bodhi token>` | The Google SDKs use either; both work.                                                                                         |
| `/api/*` (Ollama)    | `Authorization: Bearer <bodhi token>`                         | Ollama itself has no auth; Bodhi adds its own.                                                                                 |

Mixing these up gives a 401 with the corresponding envelope's error format. See [Error format](/docs/api-compatibility/error-format).

### Browser-based client gets CORS errors on `/v1/chat/completions`

- Cause: `/v1/*` does have permissive CORS; the failure is usually missing `Authorization`.
- Fix: ensure your client sends `Authorization: Bearer ...`. With OpenAI's browser SDK, use a backend-proxied token, never expose Bodhi's token directly to end users in production.

## MCP

### MCP server unreachable

Symptoms: `mcp_error-mcp_disabled`, persistent timeouts on the playground.

- Confirm the URL is correct and reachable from where Bodhi runs (not just from your laptop). In Docker, `localhost` means the container, not the host.
- Check `$BODHI_HOME/logs` for the underlying request error — DNS, TLS, or HTTP status.

### OAuth DCR fails for an MCP server

Symptom: `mcp_error-oauth_discovery_failed` when adding an MCP server with the "OAuth (Dynamic)" auth method.

- Cause: the upstream server doesn't expose RFC 8414 metadata at `.well-known/oauth-authorization-server`, or doesn't accept RFC 7591 client registration.
- Fix: switch the server's auth method to "OAuth (Preregistered)" and register a client manually with the upstream provider. See [MCP auth methods](/docs/features/mcps/auth-methods).

### Tool not callable from chat

- Cause: tool not on the instance's whitelist.
- Fix: open the MCP instance under `/ui/mcps/`, enable the tool, save. Whitelist updates take effect immediately.

### Refresh failure cascade for an OAuth MCP

Symptom: every chat call that uses tool X fails with `mcp_error-oauth_refresh_failed`. Reconnecting fixes it for a few minutes, then breaks again.

- Cause: the upstream OAuth server is rotating refresh tokens (RFC 6749 §6) and Bodhi's stored refresh token has been invalidated by a parallel session.
- Fix: disconnect the MCP, re-authorize, and avoid running multiple Bodhi instances against the same MCP credentials simultaneously.

## Settings

### "I changed a setting in the UI but nothing happened"

This is the most common confusion. Saving a setting in `/ui/settings/` writes to the database, which is _priority 4_ in the resolution order. If the same key is set in the environment (priority 3) or on the command line (priority 2), the saved value is silently overridden.

- Diagnose: open `/ui/settings/`. The "Source" column for each setting shows where the active value came from. If it reads `environment` after you save, your save is stored but inactive.
- Fix: remove the env var (e.g. unset it in `docker-compose.yml`), restart Bodhi, then your DB-saved value will be used. See [Settings precedence](/docs/reference/settings).

### Only two settings are editable at runtime

`BODHI_EXEC_VARIANT` and `BODHI_KEEP_ALIVE_SECS`. Trying to PUT/DELETE any other key returns `setting_service_error-invalid_setting_key`. To change anything else, set the env var (or `settings.yaml`) and restart.

## Performance

### First response after idle is slow

Cold-start. The model has been unloaded after `BODHI_KEEP_ALIVE_SECS` of inactivity. The next request reloads it (a few seconds for small quants, tens of seconds for large ones).

- Fix: raise `BODHI_KEEP_ALIVE_SECS` if you need consistent latency, accept the cost in idle VRAM.

### VRAM exhaustion mid-stream

Symptom: model load succeeds but generation crashes after a few hundred tokens.

- Cause: KV cache grows with context length. Large prompts blow up memory mid-stream.
- Fix: reduce the max-context flag in `BODHI_LLAMACPP_ARGS` (e.g. `--ctx-size 4096`), use a smaller quant, or move some layers to CPU with `--n-gpu-layers <N>`. See [Performance tuning](/docs/advanced/performance-tuning).

### Context-window overflow

Symptom: `context window exceeded` errors from the inference layer.

- Cause: the prompt + previous messages exceeded the model's max context.
- Fix: trim conversation history (the chat UI does not auto-trim), or pick a model with a larger context window.

### Slow streaming over a reverse proxy

- Cause: the proxy is buffering responses.
- Fix: disable buffering for `/v1/*` and `/anthropic/v1/*` endpoints (e.g. `proxy_buffering off;` in nginx). See [Reverse proxy](/docs/deployment/reverse-proxy).

## Access requests

### User access request stuck on Pending

There are no automatic notifications. An Admin or Manager must check `/ui/users/access-requests` and approve or reject the request.

### App access request expired

App access request drafts expire after **10 minutes**. The third-party app needs to start a fresh request — old draft IDs return `access_request_error-expired`.

### After approval, I'm still on the Pending page

Your session was invalidated when your role was assigned. Log out completely (the chat UI may cache state), clear cookies if needed, and log in again.

## Diagnostic workflow

When all else fails:

1. **Check logs.** `$BODHI_HOME/logs/bodhi.log.<date>` for desktop/Docker, or `docker logs <container>` if `BODHI_LOG_STDOUT=true`.
2. **Bump log level.** Set `BODHI_LOG_LEVEL=debug` and reproduce. Reset to `warn` afterward.
3. **Run from a terminal.** Catches pre-init crashes before logging initializes.
4. **Try a fresh `BODHI_HOME`.** Eliminates database / settings corruption.

## Getting help

- **GitHub Issues** — [github.com/BodhiSearch/BodhiApp/issues](https://github.com/BodhiSearch/BodhiApp/issues). Search before filing; include `BODHI_VERSION`, OS, and the relevant log excerpt.
- **Discord** — community support and quick questions. Link in the GitHub README.
- **FAQ** — [Frequently asked questions](/docs/support/faq) for higher-level "what is X" questions.
- **What's new** — [recent feature highlights](/docs/support/whats-new) if you're upgrading from an older version.
