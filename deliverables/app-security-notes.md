# BodhiApp Security Assessment Notes

These notes document by-design decisions and accepted architectural choices that were flagged during security assessments. They exist so that future security re-scans recognize these as known, intentional behaviors — not vulnerabilities.

---

## By-Design Decisions

### MCP Auth-Configs Are Tenant-Level (Not Per-User)

**Assessment finding:** Any authenticated user can list/read/delete other users' MCP auth configurations.

**Status:** By design.

**Rationale:** MCP auth-configs are shared at the tenant level. Once an auth-config is created, any user in the tenant can create MCP instances referencing it. The `created_by` field exists for audit purposes only — it is not an access control boundary. This is the intended multi-user collaboration model where teams share MCP authentication configurations.

---

### Access Request Endpoints Have No Per-User Ownership Check

**Assessment finding:** Any authenticated user can review, approve, or deny any pending external app access request.

**Status:** By design.

**Rationale:** External app access requests are created by 3rd-party applications that have no knowledge of the target tenant or user. Requests start their lifecycle as orphaned records (`tenant_id = NULL`). Ownership is established only when a user approves or denies the request — at that point, the request is bound to the approving user's tenant. The service architecture (`DefaultAccessRequestService`) intentionally skips tenant filtering during the review/approve/deny phase to accommodate this lifecycle. The comment in `access_request_service.rs` explicitly documents this: _"NOTE: This service is intentionally NOT auth-scoped... app access requests have a different lifecycle."_

---

### Download Jobs Are Tenant-Wide (Not Per-User)

**Assessment finding:** Download job list endpoint filters by tenant but not by user, allowing cross-user pull job enumeration.

**Status:** By design.

**Rationale:** Download jobs are scoped per-deployment (standalone mode) or per-tenant (multi-tenant mode). All users within a tenant can see download progress for all model pulls. The `created_by` field exists for audit purposes, not for access control. This enables team visibility into model download status.

---

### No Application-Layer Rate Limiting

**Assessment finding:** No rate-limiting middleware exists anywhere in the application.

**Status:** Deferred to infrastructure layer.

**Rationale:** BodhiApp runs in multiple deployment modes (desktop, Docker, multi-tenant cloud). Rate limiting is best handled at the reverse proxy / infrastructure layer (nginx `rate_limit`, cloud WAF, Cloudflare) where per-IP and per-endpoint policies can be tuned per deployment. Production deployments MUST configure rate limiting at the proxy layer.

---

### Exposed OAuth Client Secret via /dev/secrets

**Assessment finding:** The OAuth client secret exposed by `/dev/secrets` enables minting Keycloak service-account JWTs.

**Status:** Mitigated.

**Rationale:** The `/dev/*` endpoints are only registered when `is_production() == false` — they are never active in production. Additionally, the app already validates the JWT `audience` claim on all incoming tokens, which means `client_credentials` tokens minted with the exposed secret are rejected with `token_error-invalid_audience`.

---

### Dynamic-Register Endpoint Is Stateless (No Prior-Step Enforcement)

**Assessment finding:** The `POST /bodhi/v1/mcps/oauth/dynamic-register` endpoint makes outbound POST requests without verifying prior discovery steps.

**Status:** Mitigated by URL scheme validation.

**Rationale:** The `SafeReqwest` wrapper enforces http/https-only scheme validation on all outbound requests, preventing `javascript:`, `file:`, and other dangerous URI schemes. The endpoint's purpose IS to POST to arbitrary external URLs for OAuth dynamic client registration — this is correct behavior. Private IP connections are allowed because users may run local OAuth providers. Session state enforcement would add complexity without meaningful security benefit.

---

### SSRF Private IP Blocklist Not Enforced for AI API and MCP Services

**Assessment finding:** Outbound HTTP requests from AI API and MCP services can target private/loopback IP addresses.

**Status:** By design.

**Rationale:** Users legitimately connect to local LLM inference services (e.g., Ollama at `http://localhost:11434`) and local MCP servers. Blocking private IPs would break core functionality. URL scheme validation (http/https only) IS enforced everywhere — this blocks `javascript:`, `file:`, `data:` and other dangerous URI schemes that enable XSS attacks. The SSRF risk from private IP access is accepted because:
1. All SSRF-vulnerable endpoints require authentication (minimum User role)
2. The /dev/* destructive endpoints are not available in production
3. The user explicitly configures which URLs to connect to
