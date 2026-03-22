# BodhiApp — Known Security Issues & Risk Acceptance

This document records all known security vulnerabilities, architectural limitations, and the reasoning for their current status. It serves as the authoritative reference for security posture decisions.

**Last updated:** 2026-03-23

---

## Accepted Risks

### PBKDF2 Key Derivation Uses 1,000 Iterations (OWASP Recommends 600,000+)

- **Location:** `crates/services/src/db/encryption.rs:13` — `const PBKDF2_ITERATIONS: u32 = 1000`
- **Severity context:** Medium (requires DB file access + master encryption key)
- **Status:** Accepted risk

**Reasoning:** The application uses AES-256-GCM encryption with PBKDF2-HMAC-SHA256 key derivation to protect 11 encrypted field types across 3 database tables (MCP OAuth configs, MCP OAuth tokens, tenant secrets). A single MCP OAuth operation can trigger 3+ decrypt calls (client_secret + access_token + refresh_token).

At OWASP-recommended 600,000 iterations, each decrypt call would take ~200ms, adding ~600ms+ latency to a single request. This is unacceptable for application responsiveness.

The threat model for this encryption is: an attacker who has obtained the database file (SQLite or PostgreSQL dump) but NOT the master encryption key needs to brute-force the key derivation to recover plaintext secrets. However:
1. The master key is a server-side secret, never stored in the database
2. If an attacker has filesystem access to steal the DB, they likely also have access to the master key (stored in the same environment)
3. The partial-compromise scenario (DB stolen, key not) is unlikely in practice

Given the performance impact vs. the narrow threat scenario, 1,000 iterations is accepted.

**Mitigation:** If the threat model changes (e.g., database backups stored separately from the key), increase iterations and run a data migration.

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
