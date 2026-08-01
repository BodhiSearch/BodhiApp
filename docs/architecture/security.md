# BodhiApp — Known Security Issues & Risk Acceptance

This document records all known security vulnerabilities, architectural limitations, and the reasoning for their current status. It serves as the authoritative reference for security posture decisions.

**Last updated:** 2026-08-01

---

## Accepted Risks

### Master Key From the OS Keyring Is Not Password-Stretched

- **Location:** `crates/lib_bodhiserver/src/app_service_builder.rs` — `build_encryption_key`, keyring branch
- **Severity context:** Low
- **Status:** By design

**Reasoning:** On desktop (Tauri) no `BODHI_ENCRYPTION_KEY` is set, so the master key comes from `SystemKeyringStore::get_or_generate`, which returns 32 CSPRNG bytes (`crates/services/src/utils/keyring_service.rs`). This is the same argument as *API Tokens Hashed with SHA-256* below: key stretching exists to compensate for low-entropy human-chosen input, and a uniform 256-bit key has no such deficit. Brute force is infeasible at any iteration count.

The KEK derivation still runs PBKDF2 on this path — one code path is simpler to reason about than two, and the cost is a one-time ~70 ms at startup.

**Related risk:** if the OS keychain entry is lost (machine change, keychain reset), `get_or_generate` mints a *new* key and all previously encrypted data becomes unrecoverable. This is inherent to keychain-backed storage and is not mitigated.

---

### No Application-Layer Rate Limiting

- **Location:** All endpoints — no `tower_governor` or equivalent middleware
- **Severity context:** High (enables brute force, credential stuffing)
- **Status:** Deferred to infrastructure

**Reasoning:** BodhiApp runs in multiple deployment modes:
- **Desktop (Tauri):** Local-only, rate limiting unnecessary
- **Docker single-tenant:** Behind reverse proxy in production
- **Multi-tenant cloud:** Behind cloud load balancer with WAF

Rate limiting policies vary significantly by deployment mode — desktop needs none, while multi-tenant cloud needs per-tenant per-endpoint limits. Implementing this at the application layer would either be too generic (same limits everywhere) or too complex (deployment-aware policies). Infrastructure-layer rate limiting (nginx `limit_req`, AWS WAF, Cloudflare Rate Limiting) provides deployment-appropriate configuration.

**Requirement:** Production Docker and multi-tenant deployments MUST configure rate limiting at the reverse proxy or cloud WAF layer. This should be documented in the deployment guide.

---

### No HSTS Header

- **Location:** All HTTP responses — no `Strict-Transport-Security` header
- **Severity context:** Medium (HTTP downgrade attacks)
- **Status:** Deferred to infrastructure

**Reasoning:** BodhiApp runs HTTP internally. TLS is terminated at the reverse proxy (nginx, Caddy, cloud LB). The HSTS header should be set by the TLS-terminating proxy, not by the app that only speaks HTTP. Setting HSTS on HTTP responses would be ignored by browsers (HSTS requires HTTPS to be effective).

**Requirement:** The TLS-terminating reverse proxy MUST set `Strict-Transport-Security: max-age=31536000; includeSubDomains` on all responses.

---

### Wildcard CORS on /dev/* Endpoints

- **Location:** `crates/routes_app/src/routes_dev.rs` — `Access-Control-Allow-Origin: *`
- **Severity context:** Low (development-only)
- **Status:** Accepted (dev-only)

**Reasoning:** The `/dev/*` endpoints (`/dev/secrets`, `/dev/envs`, `/dev/db-reset`) are only registered when `is_production() == false`. The `routes.rs` startup code checks `!app_service.setting_service().is_production().await` before mounting these routes. In production deployments (`EnvType::Production`), these endpoints do not exist in the router — they return 404. The wildcard CORS is therefore only active in development environments.

---

### API Tokens Hashed with SHA-256 (Not Argon2id/bcrypt)

- **Location:** `crates/routes_app/src/middleware/token_service/token_service.rs`
- **Severity context:** Low
- **Status:** Accepted risk

**Reasoning:** API tokens are generated as 32-byte cryptographically random strings (`bodhiapp_<32-random-bytes>.<client_id>`). SHA-256 is sufficient for hashing high-entropy secrets — brute-force is computationally infeasible regardless of hash speed (~2^256 keyspace). Argon2id and bcrypt are designed for low-entropy user-chosen passwords where the hash function's slowness compensates for predictable input. Since API tokens are machine-generated with full entropy, the additional cost of a memory-hard KDF provides no security benefit.

The token comparison uses `constant_time_eq` to prevent timing attacks.

---

### Session Cookie Has No Max-Age (Browser Session Cookie)

- **Location:** `crates/services/src/auth/session_service.rs` — no `.with_expiry()` or `.with_max_age()`
- **Severity context:** Low
- **Status:** By design

**Reasoning:** Session lifetime is governed by the Keycloak access/refresh token lifecycle, not by a cookie expiry. When the Keycloak refresh token expires and cannot be renewed, the application invalidates the session and forces re-authentication. The browser session cookie (no Max-Age) is cleared when the browser closes, providing a natural session boundary for desktop use.

---

### SSRF Private IP Blocklist Not Enforced for AI API and MCP Services

- **Location:** `SafeReqwest` configured with `allow_private_ips()` for `DefaultMcpService` and `DefaultAiApiService`
- **Severity context:** High (internal port scanning, cloud metadata access)
- **Status:** Accepted — by design for local service connectivity

**Reasoning:** Users legitimately need to connect to:
- Local LLM inference services (e.g., Ollama at `http://localhost:11434`)
- Local MCP servers running on the same host or network
- Internal AI API endpoints in enterprise environments

Blocking private IPs would break core functionality. URL scheme validation (http/https only) IS enforced everywhere — this blocks `javascript:`, `file:`, `data:` and other dangerous URI schemes that enable XSS/injection attacks.

The SSRF risk from private IP access is accepted because:
1. All endpoints that make outbound requests require authentication (minimum User role)
2. The `/dev/*` destructive endpoints are not available in production
3. The user explicitly configures which URLs to connect to (AI API base URLs, MCP server URLs)
4. Cloud metadata access (169.254.169.254) is a deployment-level concern — mitigated by IMDSv2 enforcement on cloud infrastructure

---

### Argument Injection via `context_params` (INJ-VULN-01)

- **Location:** `crates/services/src/models/model_objs.rs` (`context_params: JsonVec`) → `crates/server_core/src/shared_rw.rs` (`merge_server_args`) → `Command::new().args()`
- **Severity context:** High (requires PowerUser role)
- **Status:** Accepted risk — PowerUser trust boundary

**Reasoning:** PowerUser is an admin-granted elevated role. Users with PowerUser role can configure model aliases including `context_params`, which pass as arguments to the llama-server process. This is by design for advanced model configuration (e.g., `--ctx-size 4096`, `--n-gpu-layers 35`). No untrusted user can reach this path — Admin explicitly grants the PowerUser role. The risk of a PowerUser injecting dangerous flags (e.g., `--host`, `--model`) is accepted as part of the admin trust boundary.

**Mitigation:** If the trust model changes (e.g., self-service PowerUser role), add an allowlist of permitted llama-server flags.

---

### API Tokens Have No Expiration (AUTH-VULN-08)

- **Location:** API token table (no `expires_at` column); token validation in `crates/routes_app/src/middleware/token_service/token_service.rs`
- **Severity context:** Medium
- **Status:** Accepted risk — revocation mechanism exists

**Reasoning:** API tokens are high-entropy machine-generated secrets (`bodhiapp_<32-random-bytes>.<client_id>`), SHA-256 hashed with `constant_time_eq` comparison. They can be revoked at any time via the token management API. Industry standard API key platforms (GitHub PATs, Stripe API keys, AWS access keys) similarly default to no forced expiry. PowerUser+ role is required to create tokens.

**Mitigation:** Document token rotation best practices in the deployment guide. If compliance requirements mandate token expiry, add an optional `expires_at` field to `CreateTokenRequest` and check it during validation.

---

### API Model Forward Response Proxying (SSRF-VULN-06)

- **Location:** `crates/server_core/src/fwd_sse.rs` — forwards full HTTP response from configured AI API endpoints
- **Severity context:** Critical (per assessment) → Medium (with architectural context)
- **Status:** Accepted risk — controlled endpoint forwarding

**Reasoning:** The forward endpoint's purpose is to proxy AI API responses (chat completions). The app controls the forwarding path — it appends the specific API path (e.g., `/v1/chat/completions`) to the user-configured base URL, so it does not function as an open HTTP proxy. Malformed or injected requests are rejected by the upstream AI API service. The base URL is configured by authenticated users (User+ role) who already have direct network access to the same hosts.

**Mitigation:** If the endpoint is extended to support arbitrary paths, add path allowlisting or response Content-Type validation.

---

## By-Design Architectural Decisions

### MCP Auth-Configs Are Tenant-Level Shared Resources

- **Status:** By design

Auth-configs are intentionally not per-user isolated. They are shared at the tenant level so that any team member can create MCP instances referencing existing authentication configurations. The `created_by` field provides audit trail only.

### Access Request Lifecycle Skips Ownership Checks

- **Status:** By design

External app access requests start orphaned (`tenant_id = NULL`) because 3rd-party apps have no knowledge of the target user/tenant at submission time. Ownership is established at approval/denial time. The service intentionally uses empty tenant filtering during the review phase to accommodate this lifecycle.

### Download Jobs Are Tenant-Wide

- **Status:** By design

All users in a tenant can see all download jobs. This enables team visibility into model download progress. The `created_by` field is for audit, not access control.

### Dynamic-Register Endpoint Is Stateless

- **Status:** Mitigated by URL scheme validation

The endpoint makes outbound POST requests to user-supplied URLs — this is its intended function (OAuth dynamic client registration). The `SafeReqwest` wrapper enforces http/https scheme validation. Private IPs are allowed for local OAuth providers. No session state enforcement is needed.

### /dev/* Endpoints Accessible in Development Mode

- **Status:** Not applicable in production

These endpoints are guarded by `!is_production()` runtime check in `routes.rs`. They are never registered in production deployments. The assessment was run against a development-mode instance where these endpoints are intentionally available for debugging.

---

## Remediated Vulnerabilities

The following vulnerabilities have been fixed as part of the security remediation:

| Category | Description | Fix |
|----------|-------------|-----|
| Session security | Session cookie Secure=false | Derive from `BODHI_PUBLIC_SCHEME` via `is_secure_transport()` |
| Session security | Session fixation (no ID rotation after OAuth) | `session.cycle_id()` in auth callback |
| Session security | No session expiry | Verified: Keycloak token expiry already governs session lifetime |
| Authorization | Manager can delete Admin accounts | Role ceiling check in `users_destroy` via `get_user()` |
| Authorization | Manager re-approves already-processed request | Status guard — only Pending requests can be approved |
| XSS | Stored XSS via `javascript:` in access request `redirect_url` | Validator macro on field + frontend `safeNavigate()` |
| XSS | Stored XSS via `javascript:` in MCP OAuth `authorization_endpoint` | Manual `Validate` impl + `ValidatedJson` + `safeNavigate()` |
| XSS | No Content Security Policy | Basic CSP header on HTML UI responses |
| SSRF | Outbound requests accept any URI scheme | `SafeReqwest` wrapper enforces http/https-only scheme validation |
| Path traversal | Filesystem existence oracle via `../` in filename | Filename character rejection (reject `..`, `/`, `\`) |
| Session security | Dashboard session fixation (no ID rotation after dashboard OAuth) | `session.cycle_id()` in dashboard auth callback |
| Transport | Missing Cache-Control on token creation response | `Cache-Control: no-store` + `Pragma: no-cache` on `tokens_create` response |
| Cryptography | PBKDF2 key derivation used 1,000 iterations | Two-tier KEK: PBKDF2-HMAC-SHA256 at 600,000 iterations once at startup, HKDF-SHA256 per row (see below) |
| Cryptography | `BODHI_ENCRYPTION_KEY` accepted any value, and the placeholder guard checked a string `.env.example` did not ship | Length floor (20) + both placeholder strings rejected at boot |

## Key Derivation (v2 scheme)

Secrets are encrypted with AES-256-GCM under a per-row key. Since v2 the key derivation is two-tier:

```
KEK      = PBKDF2-HMAC-SHA256(master_key, "bodhiapp:kek:v2", 600_000)   # once, at startup
row key  = HKDF-SHA256(ikm = KEK, salt = per-row salt, info = "bodhiapp:row:v2")
```

The original scheme ran the full PBKDF2 stretch *per row* against the row salt, which is what made the OWASP iteration count look unaffordable. Per-row stretching buys nothing when there is a single global master key — an attacker's work is per-candidate-key, not per-row, so cracking one row cracks all of them. Moving the stretch to a once-per-process KEK and using HKDF (~1 µs) per row gives the full OWASP work factor at no per-request cost, and keeps a 70 ms blocking computation out of the async request path.

**On-disk format.** v2 ciphertext carries a `v2:` prefix. Base64's alphabet never contains `:`, so an unprefixed value is unambiguously a pre-v2 row — legacy detection is a string check, never a failed decrypt. This kept the change free of any schema migration.

**Legacy rows.** Only `tenants.encrypted_client_secret` has a legacy read path: it is decrypted on *every* tenant read, so an unreadable row 500s every request and leaves the UI with no login and no setup wizard. `DefaultDbService::reencrypt_legacy_tenant_secrets` upgrades those rows at startup (idempotent, skips `v2:`), and aborts boot on a genuine key mismatch rather than stranding the deployment in a 500 loop.

Every other table surfaces `DbError::LegacyEncryption` (`db_error-legacy_encryption`, HTTP 422) when a pre-v2 secret is used, prompting the user to recreate that one resource. This is deliberate: the deployment footprint at the time of the change was small enough that coordinated recreation beat carrying a dual-decrypt path across six tables indefinitely.

**Operator requirement.** `BODHI_ENCRYPTION_KEY` must be at least 20 characters and must not be a placeholder; boot fails otherwise. Generate one with `openssl rand -base64 32`. There is no previous-key fallback — **changing the key makes every existing encrypted secret unrecoverable.**
