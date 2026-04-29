---
title: 'Security Model'
description: 'What Bodhi App protects, what it relies on the deployment to provide, and how to harden a self-hosted installation'
order: 1
---

# Security Model

This page is the public summary of Bodhi App's security posture. It covers the guarantees the app makes by itself, the responsibilities it leaves to the deployment (your reverse proxy, your identity provider, your filesystem), and the hardening steps a self-hoster should take. The audience is operators making decisions about how to expose Bodhi to the network — not security auditors.

If you're choosing between desktop and Docker, [Deployment → Overview](/docs/deployment/overview) will save you a step. If you're configuring TLS and rate limiting, jump to [Deployment → Reverse Proxy](/docs/deployment/reverse-proxy).

## Trust boundaries at a glance

Bodhi App is one process. There are four trust boundaries around it:

1. **Network ↔ reverse proxy.** Your TLS-terminating proxy is the public face. Bodhi speaks plain HTTP behind it.
2. **Reverse proxy ↔ Bodhi.** Anything that reaches the app is treated as authenticated-or-anonymous; the proxy is trusted to forward client IPs and not strip auth headers.
3. **Bodhi ↔ identity provider.** OAuth2 PKCE flows go to your configured Keycloak realm. Bodhi never stores user passwords.
4. **Bodhi ↔ disk.** The app's database, alias YAML, and HuggingFace cache live on a filesystem that you trust at the OS level.

Anything described below sits _inside_ boundary 4. Anything outside is your responsibility.

## What Bodhi protects against

### Authentication

- **OAuth2 with PKCE** for all interactive logins. PKCE is mandatory — there is no legacy implicit or password-grant fallback.
- **Session cookies** are `HttpOnly`, `SameSite=Strict`, and marked `Secure` whenever `BODHI_PUBLIC_SCHEME=https` (so the browser will not transmit them over HTTP).
- **Session ID rotation** on every successful login (and on dashboard re-auth) — defends against fixation attacks where an attacker pre-plants a session ID.
- **API tokens** are 32 bytes of cryptographically random data, prefixed with `bodhiapp_` and a client-id suffix. They are stored as SHA-256 hashes; the plaintext is shown to you exactly once at creation. Token comparison is constant-time, so timing attacks against the hash table are not viable.
- **Token revocation** — any token can be revoked instantly via the token management page; the next request with that token returns `401`.

### Authorization

- **Role hierarchy** with strict ordering: `User < PowerUser < Manager < Admin`. Role-elevation attacks are mitigated by a _role ceiling_ check — a Manager cannot delete or demote an Admin, regardless of how the request is shaped.
- **Status guards** on access requests — only a `Pending` request can be approved or denied, so a re-played approval call cannot reactivate a closed request.
- **Per-route auth checks** — every endpoint declares its required scope. The middleware refuses requests that don't carry an identity meeting that floor.
- **Resource consent** for external apps — third-party apps registered against Bodhi only see the scopes the user explicitly granted at consent time.

For the full role × endpoint matrix, see [Reference → Roles and Scopes](/docs/reference/roles-and-scopes).

### Encryption at rest

Sensitive credentials — API-model provider keys, MCP OAuth client secrets, MCP OAuth access/refresh tokens — are encrypted at rest in the application database using AES-256-GCM. The master key is read from the `BODHI_ENCRYPTION_KEY` environment variable; it is never written to the database.

If `BODHI_ENCRYPTION_KEY` is lost, encrypted credentials cannot be recovered — you must rotate the upstream secrets and re-enter them. Treat this variable like a database master password: store it in a secrets manager, back it up out-of-band, and never commit it to a repo.

### Browser-side hardening

- **Content Security Policy** — Bodhi serves a strict CSP on its UI: scripts and styles are restricted to first-party origins, fonts are self-hosted (no Google Fonts CDN), and there are no inline event handlers in the shipped UI.
- **No `javascript:` URIs accepted.** Any user-supplied URL field (access-request redirect URLs, MCP OAuth authorization endpoints) is validated server-side and re-validated client-side before navigation.
- **Cache-Control on token creation** — the response that contains a freshly minted plaintext API token is marked `no-store`, so it does not end up in the browser's disk cache or in proxies along the way.

### Network egress

- **URL scheme allowlist.** Any URL Bodhi fetches outbound (provider APIs, MCP servers, OAuth endpoints) must be `http://` or `https://`. Schemes like `file:`, `data:`, and `javascript:` are rejected.
- **Path validation.** User-supplied filenames and aliases reject `..`, `/`, and `\` to prevent traversal-based filesystem probing.

## What Bodhi does _not_ do (by design)

These are deliberate gaps you need to fill at the deployment layer.

### Rate limiting belongs at the reverse proxy

Bodhi does not rate-limit at the application layer. The right policy depends on the deployment shape (a desktop install needs none; a Docker single-tenant on the public internet needs aggressive per-IP limits on `/login` and the API surface), and the right place to enforce it is the proxy that already sees every request before Bodhi does.

If you expose Bodhi publicly, configure rate limiting on `nginx`, `Caddy`, or your cloud WAF. See [Deployment → Reverse Proxy](/docs/deployment/reverse-proxy) for sample configs and recommended thresholds.

### TLS termination belongs at the reverse proxy

Bodhi serves plain HTTP. TLS is terminated by the proxy in front of it. The `Strict-Transport-Security` (HSTS) header should be set by that proxy, not by Bodhi — an HSTS header on an HTTP response is ignored by browsers anyway.

### Application-layer audit log shipping

Bodhi writes structured logs to disk (see [Observability](/docs/advanced/observability)). It does not push logs to a SIEM, sign them, or enforce immutability. If you need tamper-evident audit logs, configure your log collector to ship `$BODHI_HOME/logs/` to your central audit store with append-only semantics.

### Cloud metadata protection

Bodhi allows outbound requests to private IPs by design — local LLM services (Ollama on `localhost`), self-hosted MCP servers, and on-prem OAuth providers all live on private networks. If you run Bodhi on a cloud VM with an instance metadata service (IMDS), enforce IMDSv2 at the VM level so a compromised request cannot scrape credentials from `169.254.169.254`. This is the cloud platform's job, not the app's.

## Threat model in plain language

Bodhi protects against:

- **Casual credential theft** — leaked API tokens are bounded by SHA-256 hashing and revocation; leaked session cookies are bounded by HttpOnly/Secure flags and ID rotation.
- **CSRF** — session cookies are `SameSite=Strict`, so cross-site form submissions don't carry the user's cookie.
- **Privilege escalation by lower-tier users** — role ceilings stop Managers from operating on Admins; status guards stop replayed approval calls.
- **XSS via stored content** — CSP plus URL scheme validation block the common stored-script vectors.
- **Insider snooping on at-rest data** — sensitive credentials in the database are AES-256-GCM-encrypted with a key that lives outside the database.
- **Open-proxy abuse** — outbound requests are scheme-validated; the forward path uses fixed upstream URLs from the user's API-model record, not arbitrary URLs from request bodies.

Bodhi does **not** by itself protect against:

- A compromised reverse proxy (TLS, rate limiting, header forwarding).
- A compromised filesystem with both the database file _and_ the encryption key.
- A compromised identity provider (issued tokens are accepted at face value).
- Denial-of-service from very high request rates (handle at the proxy).
- Side-channel attacks on the host (CPU vulnerabilities, container escape).

These are the deployment's responsibility.

## Recommended hardening for self-hosters

If you are running Bodhi outside a desktop install, this is the minimum checklist:

1. **TLS at the proxy.** Terminate HTTPS at `nginx`/`Caddy`/your cloud LB. Set `BODHI_PUBLIC_SCHEME=https` so cookies are marked `Secure`. See [Deployment → Reverse Proxy](/docs/deployment/reverse-proxy).
2. **Rate limit at the proxy.** Tighter limits on `/login` and `/ui/auth/*` than on chat endpoints. Reject anonymous traffic to `/bodhi/v1/*` if you don't intend to register external apps.
3. **Set HSTS at the proxy.** `Strict-Transport-Security: max-age=31536000; includeSubDomains` on every response.
4. **Generate `BODHI_ENCRYPTION_KEY` once, store it securely.** Use a 256-bit value from `openssl rand -base64 32`. Back it up where the database is _not_ — losing one without the other should not give an attacker plaintext credentials.
5. **Restrict filesystem access.** `$BODHI_HOME` should be readable only by the user running Bodhi. The HuggingFace cache and the database files in particular should not be world-readable.
6. **Watch the logs.** Tail `$BODHI_HOME/logs/` for repeated `401`/`403` from the same IP. See [Observability](/docs/advanced/observability) for log levels and rotation behaviour.
7. **Rotate API tokens.** Tokens have no forced expiry; revoke unused ones from the token management page on a schedule. The `last_used_at` timestamp helps spot dormant tokens.
8. **Lock down `/dev/*` by deployment type.** These endpoints are automatically disabled when the app is built for production. If you're running a development build on the public internet, stop.

## Where to read more

- [Reference → Roles and Scopes](/docs/reference/roles-and-scopes) — the authoritative role × scope × endpoint matrix.
- [Features → API Tokens](/docs/features/auth/api-tokens) — token creation, revocation, and best practice.
- [Concepts → Auth and Roles](/docs/concepts/auth-and-roles) — the mental model behind the role hierarchy.
- [Deployment → Reverse Proxy](/docs/deployment/reverse-proxy) — sample TLS, rate limit, and header-forwarding configs.
