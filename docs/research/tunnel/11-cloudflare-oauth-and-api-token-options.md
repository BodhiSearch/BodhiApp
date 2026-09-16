# 11 — Cloudflare OAuth & API token options for a third-party desktop app

**Date:** 2026-09-15
**Question:** how can BodhiApp (a locally-running third-party desktop app) get Cloudflare API authorization for named-tunnel + DNS management, without the user hand-copying secrets — and what's the fallback?

## TL;DR

| Tier | Verdict | Why |
|---|---|---|
| **Reuse `cloudflared tunnel login`'s cert** | **POSSIBLE, best UX if `cloudflared` is already the install path** | `cert.pem` is literally a wrapped Cloudflare API token (see §2). If BodhiApp is already asking the user to install/run `cloudflared` (per the codebase-map plan), parsing the existing cert avoids a second auth step entirely. Caveat: the embedded token is scoped to Tunnel management only, **not** DNS — a supplementary DNS-scoped credential is still needed. |
| **BodhiApp registers its own Cloudflare OAuth client (self-managed OAuth, PKCE, public client)** | **POSSIBLE as of 2026-06-03, but scope coverage for Tunnel+DNS is UNVERIFIED** | Cloudflare shipped "self-managed OAuth clients" — any developer can register a public (secret-less, PKCE) OAuth app. This is the right shape for a desktop app. But nothing in the docs enumerates the actual selectable scope catalog, and it's unknown whether `cfd_tunnel`/DNS permission groups are exposed as OAuth scopes at all (see §1.2). **Must be verified by actually registering a test client** before committing to this as Tier 2. |
| **Reuse wrangler's own OAuth client_id** | **NOT RECOMMENDED / functionally NOT POSSIBLE for this use case** | Wrangler's fixed scope catalog has no DNS-write scope at all (only `zone:read`) — see §1.1. Even setting aside the identity/ToS concerns of presenting as a different app, it cannot drive DNS routing. |
| **User-created scoped API token, pasted or via prefilled dashboard link** | **POSSIBLE, always-works fallback** | Cloudflare has an official prefilled "create token" template URL (`dash.cloudflare.com/?to=/:account/api-tokens&permissionGroupKeys=...`). This is the most reliable, fully-documented path and should be the guaranteed fallback regardless of what Tier 1/2 turn out to support. |

Recommended order for BodhiApp, refined from the product owner's stated preference: **(1) if `cloudflared` is already installed and logged in, offer to reuse its cert's embedded token for tunnel ops + prompt for a small DNS-only token/scope on top; (2) self-managed OAuth client, once its scope catalog is confirmed to cover `cfd_tunnel`+DNS; (3) prefilled-link API token, always available.** Full comparison in §4.

---

## 1. Cloudflare OAuth

### 1.1 How `wrangler login` actually works

As of the current `cloudflare/workers-sdk` monorepo, wrangler's auth code has moved into a shared package, **`@cloudflare/workers-auth`** (`packages/workers-auth/src/`), with a thin `wrangler`-specific binding at `packages/workers-auth/src/wrangler/`. `packages/wrangler/src/user/user.ts` is now just a re-export shim over that package. This matters for anyone reading older articles about "wrangler's OAuth client" — the client id/scope catalog now lives in `workers-auth/src/wrangler/env.ts` and `workers-auth/src/core/scopes.ts`, not in `wrangler` itself.

**Endpoints** (`packages/workers-auth/src/env-vars.ts`):
| Purpose | URL |
|---|---|
| Auth domain | `dash.cloudflare.com` (staging: `dash.staging.cloudflare.com`) |
| Authorize | `https://dash.cloudflare.com/oauth2/auth` |
| Token | `https://dash.cloudflare.com/oauth2/token` |
| Revoke | `https://dash.cloudflare.com/oauth2/revoke` |
| Device authorization (new, RFC 8628) | `https://dash.cloudflare.com/oauth2/device/auth` |
| JWKS / OIDC discovery | `https://dash.cloudflare.com/.well-known/jwks.json`, `/.well-known/openid-configuration` |

**Client ID** (`packages/workers-auth/src/wrangler/env.ts`, env var `WRANGLER_CLIENT_ID`):
- Production: `54d11594-84e4-41aa-b438-e81b8fa78ee7`
- Staging: `4b2ea6cc-9421-4761-874b-ce550e0e3def`

**Redirect URI** (`packages/workers-auth/src/wrangler/constants.ts`): fixed at `http://localhost:8976/oauth/callback`. This is the port wrangler's local callback HTTP server binds to (`packages/workers-auth/src/callback-server.ts`); a 2-minute timeout applies while waiting for the browser round-trip.

**PKCE** (`packages/workers-auth/src/pkce.ts`): `code_verifier` is 96 random bytes from the RFC 7636 charset, base64url-encoded (43–128 chars); `code_challenge = base64url(SHA-256(code_verifier))`; `code_challenge_method=S256` always. `state` is a 32-byte random anti-CSRF value. This is a textbook public-client PKCE flow — no client secret is ever used.

**Auth URL shape** (`generate-auth-url.ts`):
```
https://dash.cloudflare.com/oauth2/auth?response_type=code&client_id=<uuid>
  &redirect_uri=http%3A%2F%2Flocalhost%3A8976%2Foauth%2Fcallback
  &scope=<space-joined scopes>+offline_access&state=<random>
  &code_challenge=<b64url sha256>&code_challenge_method=S256
```
`offline_access` is appended to every request automatically, which is why a refresh token always comes back.

**Token exchange / refresh** (`token-exchange.ts`): standard `grant_type=authorization_code` (with `code_verifier`) and `grant_type=refresh_token` POSTs to the token endpoint, `application/x-www-form-urlencoded`. Response: `access_token`, `expires_in`, `refresh_token`, `scope` (space-delimited). Refresh tokens are stored client-side (plaintext TOML under wrangler's config dir by default, or OS keyring with `--use-keyring`) and re-read from disk on every refresh (so concurrent wrangler processes share rotation).

**Device flow** (new; `device-flow.ts`, shipped per the 2026-08-04 changelog "Log in to Wrangler without a local callback server"): `wrangler login --device` implements RFC 8628 — POSTs to `/oauth2/device/auth`, shows a `user_code` + `verification_uri`, polls `/oauth2/token` with `grant_type=urn:ietf:params:oauth:grant-type:device_code`. Max wait is `min(server expires_in, 300s)`; poll interval starts at 1s (below the RFC's 5s default) and backs off +5s on `slow_down`. **This is directly relevant to BodhiApp**: device flow needs no local HTTP server/port, which is a strictly better fit for a desktop app driving a browser than the PKCE+localhost-callback flow — if BodhiApp registers its own OAuth client (§1.2), prefer requesting the device-authorization grant type if Cloudflare's self-managed OAuth clients support it (UNVERIFIED — the self-managed OAuth docs only describe `authorization_code`; device grant may be wrangler-specific/internal).

**Scope catalog** (`packages/workers-auth/src/core/scopes.ts`, `DefaultScopes`) — this is wrangler's own fixed, hardcoded list, requested via the `scope` query param above:

```
account:read, user:read, workers:write, workers_kv:write, workers_routes:write,
workers_scripts:write, workers_tail:read, d1:write, pages:write, zone:read,
ssl_certs:write, ai:write, ai-search:write, ai-search:run, agent-memory:write,
queues:write, pipelines:write, secrets_store:write, artifacts:write,
flagship:write, containers:write, cloudchamber:write, connectivity:admin,
email_routing:write, email_sending:write, browser:write, challenge-widgets.write
```

Two findings matter for the tunnel feature:
- **No DNS-write scope exists in this catalog.** Only `zone:read` is present — nothing like `dns:write` or `zone:write`. Wrangler's OAuth token literally cannot create/update a DNS record.
- **`connectivity:admin`** is described as *"See, change, and bind to Connectivity Directory services, including creating services targeting Cloudflare Tunnel."* "Connectivity Directory" is a newer Cloudflare One construct, distinct from the classic `cfd_tunnel` REST resource that named tunnels use (see §3). Whether this scope actually authorizes calls to `POST /accounts/{id}/cfd_tunnel` is **UNVERIFIED** — nothing in wrangler's own code exercises tunnel creation via OAuth (wrangler doesn't manage tunnels at all).

**Conclusion for reusing wrangler's client_id (option in the task brief):** **NOT POSSIBLE for this feature, and NOT RECOMMENDED even where technically reachable.** Technically the authorize/token endpoints don't cryptographically bind a client_id to a specific app (any PKCE public client can present `client_id=54d11594-...`), but (a) the registered `redirect_uri` is fixed to `localhost:8976`, so a third party would have to bind that exact port and would collide with a real wrangler login on the same machine, (b) the scope catalog above cannot cover DNS routing regardless, and (c) presenting as "Wrangler" to the Cloudflare consent screen while being a different app is a user-facing misrepresentation that Cloudflare's terms would reasonably prohibit even though no specific clause enumerating this was found (UNVERIFIED — no explicit ToS text located; flagged as a policy risk, not just a technical one).

### 1.2 Can BodhiApp register its own OAuth client? — YES, "self-managed OAuth clients"

Cloudflare shipped **self-managed OAuth clients** ([changelog, 2026-06-03](https://developers.cloudflare.com/changelog/post/2026-06-03-public-oauth-clients/)): any developer can create and manage their own OAuth application that integrates with the Cloudflare API, as an alternative to asking users to paste API tokens. This is available on all plan tiers (Free/Pro/Business/Enterprise) — it's an account-level feature, not gated by the *user's* Cloudflare plan.

**Registration (one-time, done by BodhiApp's developers, not by each end user):**
- Dashboard: **Manage Account → OAuth clients → Create client**.
- Or via API: `POST /accounts/{account_id}/oauth_clients` (requires the calling API token/account role to already have "OAuth Clients Write" — i.e. registering the client itself needs a one-time authenticated bootstrap step by BodhiApp's own Cloudflare account, not the end user's). Example body:
  ```json
  {
    "client_name": "BodhiApp",
    "grant_types": ["authorization_code"],
    "redirect_uris": ["https://example.com/oauth/callback"],
    "scopes": ["workers-platform.read", "workers-platform.write"],
    "response_types": ["code"],
    "token_endpoint_auth_method": "client_secret_basic",
    "logo_uri": "...", "policy_uri": "...", "tos_uri": "...", "client_uri": "..."
  }
  ```
  (Field names/example per Cloudflare's docs; the `scopes` values shown, `workers-platform.read`/`.write`, are the *only* concrete scope strings surfaced in the public docs — **no Tunnel or DNS scope example was found anywhere in the OAuth-client docs.**)

**Public/native-app support:** clients can be registered as **public** (`token_endpoint_auth_method: "none"`, PKCE `S256` required, no client secret) — exactly the shape a desktop app needs, matching wrangler's own flow. New clients default to **private** (visible/usable only by members of the registering Cloudflare account); making a client **public** (usable by any Cloudflare user, which BodhiApp needs since end users are on their own accounts) requires: a client logo, a client URL, at least one scope, and **DNS domain verification via a TXT record** on the client's domain — and per the docs, **"setting a client's visibility to public is permanent."** Plan for this as a one-time, deliberate release step (own a domain, add the TXT record, verify) before shipping OAuth as a real login option.

**Scopes:** the docs state *"OAuth scope names correspond to Cloudflare API token permission names"* and *"fetch the available scopes from the API [and] use the scope ID when you create a client."* This strongly suggests OAuth scopes are meant to mirror the same permission-group catalog used by API tokens (§3), which **does** include `Cloudflare Tunnel Edit`/`Write` and `DNS Write`. But:
- The only concrete scope strings found in any doc are `workers-platform.read`/`workers-platform.write` — a different naming style (`namespace.verb`) than both wrangler's `resource:verb` catalog and API tokens' `Resource Action` display names.
- No page enumerates the full scope catalog, and no "list available OAuth scopes" endpoint URL was found documented (the docs only say to "fetch the available scopes from the API").
- **This is a GREY / UNVERIFIED finding, not a NOT POSSIBLE one** — it needs to be settled empirically: register a test client, call whatever the "list scopes" API turns out to be (or attempt an authorize request with a tunnel/DNS-shaped scope string and see if Cloudflare rejects it), before committing engineering time to this path as BodhiApp's Tier 2.

### 1.3 Dynamic Client Registration (RFC 7591) — NOT for Cloudflare's own API auth

RFC 7591 dynamic client registration **is** something Cloudflare ships — but as part of **`@cloudflare/workers-oauth-provider`**, a library for building *your own* OAuth 2.1 authorization server on Workers (used for e.g. remote MCP servers where *Cloudflare Workers is the authorization server* and some other app is the client). It is unrelated to authorizing third-party apps against Cloudflare's *own* control-plane API. No evidence was found that `dash.cloudflare.com/oauth2` (the endpoint wrangler and self-managed OAuth clients both use) exposes a public self-serve DCR endpoint — registering a self-managed OAuth client (§1.2) still requires either the dashboard UI or an authenticated `POST /accounts/{id}/oauth_clients` call, i.e. a human (or a pre-existing credential) in the loop once per *developer*, not RFC 7591's fully automated no-prior-credential registration. **Conclusion: NOT POSSIBLE as true zero-touch DCR; POSSIBLE as a one-time manual/API registration done by BodhiApp's own team.**

---

## 2. `cloudflared tunnel login` — what the origin cert actually authorizes

This is **not an OAuth2 flow at all**. Confirmed by reading `cloudflared`'s source directly (`cmd/cloudflared/tunnel/login.go`, `credentials/origin_cert.go`, `github.com/cloudflare/cloudflared`):

- `login()` opens the browser to `https://dash.cloudflare.com/argotunnel` (constant `baseLoginURL`), with a callback-store URL of `https://login.cloudflareaccess.org/` (constant `callbackURL`). This is Cloudflare's proprietary **"token transfer" protocol** (`token.RunTransfer(...)`): the dashboard, after the user logs in and picks a zone, POSTs a resource blob to the callback store; the CLI polls that store and downloads it. There is no `client_id`, no PKCE, no `code`/`state` — it's a bespoke browser-hand-off mechanism predating (and separate from) the OAuth machinery in §1.
- The downloaded resource is decoded (`credentials.DecodeOriginCert`) from a PEM block of type `ARGO TUNNEL TOKEN` whose payload is **plain JSON**:
  ```go
  type OriginCert struct {
      ZoneID    string `json:"zoneID"`
      AccountID string `json:"accountID"`
      APIToken  string `json:"apiToken"`
      Endpoint  string `json:"endpoint,omitempty"`
  }
  ```
  i.e. **`~/.cloudflared/cert.pem` is a wrapped, ordinary Cloudflare API token**, plus the account/zone IDs the user selected during the browser step.
- **What it authorizes:** per Cloudflare's tunnel-permissions docs, the cert/token permits creating, deleting, and managing all tunnels for the account (the "Cloudflare Tunnel" permission group). It explicitly does **not** cover DNS routing — routing a hostname to a tunnel (creating the CNAME) needs a separate DNS-write-capable credential/role, and the docs call out that the login flow also needs "DNS and Load Balancer" role permissions for public-hostname routing to work end-to-end via `cloudflared tunnel route dns`.
- **Lifetime:** per Cloudflare's docs, the certificate itself "remains valid for at least 10 years, and the service token it contains is valid until revoked" — effectively non-expiring absent action.
- **Revocation:** delete the underlying token from **My Profile → API Tokens** (it's listed there, historically labeled "Cloudflare Tunnel API Token" / "Argo Tunnel API Token"). Once deleted, the old `cert.pem` stops working — same mechanism as revoking any API token.
- **Is it reusable by BodhiApp directly via the REST API?** **YES, confirmed by source.** Because `apiToken` in the decoded cert is a normal bearer API token, BodhiApp can, after the user runs `cloudflared tunnel login` once, read and parse `cert.pem` (or `$TUNNEL_ORIGIN_CERT` if set), extract `apiToken`/`accountID`/`zoneID`, and call `https://api.cloudflare.com/client/v4/accounts/{accountID}/cfd_tunnel` directly with `Authorization: Bearer <apiToken>` — no need to shell out to `cloudflared tunnel create`. This is an attractive Tier-1 UX if BodhiApp is already telling users to install `cloudflared` (per the existing plan in `bodhiapp-cloudflare-tunnel-feasibility.md`): "already logged in via `cloudflared`? we'll reuse it" is zero extra clicks for that subset of users. The caveat above still applies — this token alone likely can't write DNS, so BodhiApp still needs a second, smaller credential (or to ask the user to grant DNS role permissions before login) to complete the CNAME step.

---

## 3. API token path (guaranteed fallback)

**Exact permission groups needed**, per Cloudflare's own "Create a tunnel via API" guide (`developers.cloudflare.com/cloudflare-one/.../create-remote-tunnel-api/`):

| Scope | Permission | Notes |
|---|---|---|
| Account | **Cloudflare Tunnel · Edit** | Create/delete/configure the tunnel (`cfd_tunnel` resource). |
| Zone | **DNS · Edit** | Create the `CNAME` routing the hostname to `<tunnel-id>.cfargotunnel.com`. |
| Zone | **Zone · Read** | Look up the zone ID from the zone name. |
| Account | **Account Settings · Read** (optional) | Only needed if BodhiApp must list/discover which account the user's zone belongs to via `GET /accounts` rather than the user supplying an account ID directly. |

**Newer, more granular alternative permission groups** (per Cloudflare's "Granular permissions for Tunnels and Mesh nodes" — opt-in, additive, does not replace the above): `Cloudflare One Connectors Write`, `Cloudflare One Connector: cloudflared Write`, or the plain `Cloudflare Tunnel Write` — any one of these is accepted for tunnel-management calls. These let an account scope a token/role to a *specific* tunnel resource rather than "every tunnel on the account," which is worth surfacing in the in-app token-creation instructions as the more-precise option for security-conscious users, once BodhiApp knows the exact permission-group `id` for it (fetch via `GET /user/tokens/permission_groups`, which returns stable IDs + names — UNVERIFIED exact `id` value for these newer groups without calling that endpoint against a live account).

**Concrete API calls** (from the same official guide):
```
POST https://api.cloudflare.com/client/v4/accounts/$ACCOUNT_ID/cfd_tunnel
Body: {"name": "bodhi-tunnel", "config_src": "cloudflare"}
→ returns tunnel id + a connector token

PUT https://api.cloudflare.com/client/v4/accounts/$ACCOUNT_ID/cfd_tunnel/$TUNNEL_ID/configurations
Body: {"config": {"ingress": [
  {"hostname": "app.example.com", "service": "http://localhost:1135"},
  {"service": "http_status:404"}
]}}

# then create the CNAME (via DNS API, or `cloudflared tunnel route dns <id> app.example.com`)
# then run the connector:
cloudflared tunnel --no-autoupdate run --token <TUNNEL_TOKEN>
# or: docker run cloudflare/cloudflared:latest tunnel --no-autoupdate run --token <TUNNEL_TOKEN>
```

**Prefilled "create token" deep link** — CONFIRMED, official and documented (`developers.cloudflare.com/fundamentals/api/how-to/account-owned-token-template/`):
```
https://dash.cloudflare.com/?to=/:account/api-tokens&permissionGroupKeys=<url-encoded JSON>&name=<token name>
```
- `permissionGroupKeys` is a URL-encoded JSON array of `{"key": "<permission key>", "type": "read"|"edit"|"revoke"|"run"|"purge"}` objects, e.g. (Workers example from the docs) `[{"key":"workers_scripts","type":"edit"},{"key":"workers_kv_storage","type":"edit"},{"key":"workers_routes","type":"edit"}]`.
- Account tokens omit `accountId`/`zoneId` (resource scoping — which account/zone the token applies to — is chosen by the user in the dashboard UI after landing on the prefilled page); there's also a user-token variant of the template with `accountId`/`zoneId` params instead, per community write-ups, but that variant was **not found in Cloudflare's own docs** — treat as **UNVERIFIED** and prefer the officially documented account-token template above.
- **The exact `key` strings for Tunnel/DNS/Zone permissions were not found documented anywhere** (only the Workers example is shown). These should be confirmed by calling `GET /user/tokens/permission_groups` once against a real account and matching by display name (`"Cloudflare Tunnel"`, `"DNS"`, `"Zone"`) before hardcoding a link in the BodhiApp UI — likely candidates by convention are `cfd_tunnel`/`tunnel`, `dns`, `zone`, but this is **UNVERIFIED, do not hardcode without confirming.**
- BodhiApp's in-app flow: build this link server-side (or client-side) with the right `permissionGroupKeys` + a descriptive `name` like `"BodhiApp Tunnel (<hostname>)"`, open it in the user's browser, then have the user paste the resulting token back into BodhiApp.

**Verification / discovery endpoints:**
- `GET https://api.cloudflare.com/client/v4/user/tokens/verify` with `Authorization: Bearer <token>` — confirms the token is valid/active and returns its id/status. Call this immediately after the user pastes a token, before storing it, to fail fast on typos/expired tokens.
- `GET https://api.cloudflare.com/client/v4/accounts` — lists accounts the token can act on (auto-select if exactly one, else prompt).
- `GET https://api.cloudflare.com/client/v4/zones` (optionally `?account.id=`) — lists zones, to let the user pick which domain to route the tunnel hostname under.

**Token schema / TTL** (`POST /user/tokens` request body, per Cloudflare's API reference):
```json
{
  "name": "BodhiApp Tunnel",
  "policies": [{
    "effect": "allow",
    "permission_groups": [{"id": "<permission group id>"}],
    "resources": {"com.cloudflare.api.account.<account_id>": "*"}
  }],
  "expires_on": "2027-09-15T00:00:00Z",
  "not_before": "2026-09-15T00:00:00Z"
}
```
`expires_on`/`not_before` are optional ISO-8601 datetimes — Cloudflare tokens don't expire by default. Since BodhiApp is only guiding the user to the dashboard UI (not calling `POST /user/tokens` itself, unless BodhiApp later gets its own token to create sub-account tokens programmatically), the practical TTL control is whatever the user sets in the "TTL (time to live)" field the create-token UI exposes — worth calling out in the in-app instructions ("we recommend no expiry, or a long TTL — BodhiApp will warn you and prompt for a fresh token if it starts failing 401s").

**UX best practices for guiding the user to a token** (synthesized from the above, cf. the `apievangelist.com` critique that Cloudflare has "no public self-serve path... without a human in a browser" for API tokens specifically — this is still true for tokens, which is exactly why §1.2's OAuth path is the better long-term Tier 2):
1. Use the prefilled deep link (once `permissionGroupKeys` values are confirmed) so the user never has to hand-pick permissions from a long list — this is the single biggest source of misconfigured/under-scoped tokens.
2. Immediately `GET /user/tokens/verify` on paste; on failure, tell the user which permission is likely missing rather than a generic "invalid token."
3. Call `GET /accounts` and `GET /zones` right after verify to auto-populate the account/zone pickers instead of asking the user to find IDs manually.
4. Store the token encrypted at rest (BodhiApp already has the `BODHI_ENCRYPTION_KEY` pattern — reuse it, per the existing feasibility doc).

---

## 4. Comparison and recommendation

| | Reuse `cloudflared` login cert | Self-managed OAuth client | Pasted API token (prefilled link) |
|---|---|---|---|
| **User friction** | Zero *if* `cloudflared` already installed+logged-in; otherwise same install friction as any `cloudflared` dependency | Lowest possible once shipped: standard "Sign in with Cloudflare" consent click, no copy-paste, no dashboard permission-picking | Medium: one browser trip to a prefilled page, click Create, copy, paste back |
| **What's stored at rest, where** | The `apiToken` string parsed out of `cert.pem` (BodhiApp reads the user's existing file — doesn't create a new secret) — store encrypted the same as an API token | `access_token` (short-lived) + `refresh_token` (long-lived, sensitive) — must be encrypted at rest; refresh token is the durable secret | The pasted API token string — must be encrypted at rest |
| **Revocation** | User deletes the underlying token in **My Profile → API Tokens** (same mechanism as any token) | User revokes BodhiApp's access from Cloudflare's OAuth consent-management UI, or BodhiApp calls the revoke endpoint | User deletes the token in **My Profile → API Tokens** |
| **Creates tunnel (`cfd_tunnel`)** | Yes — token has Tunnel Edit by design | UNVERIFIED — depends on whether a Tunnel-equivalent scope is grantable to self-managed OAuth clients (§1.2) | Yes, if the right permission group is selected |
| **Routes DNS** | **No** — origin cert token lacks DNS write; needs a second credential | UNVERIFIED, same open question as above | Yes, if DNS Edit is selected |
| **Runs the connector** | Yes (`cloudflared tunnel run --token`) works with any valid tunnel token, regardless of which tier produced it | Same | Same |
| **Depends on `cloudflared` being installed** | Yes (that's the whole point of this tier) | No | No |

**Recommended order, refined:**
1. **Detect an existing `cloudflared` login** (`~/.cloudflared/cert.pem` or `$TUNNEL_ORIGIN_CERT`) and offer to reuse its embedded token for tunnel creation — zero extra auth step for users who already have `cloudflared` set up. Pair it with a small supplementary DNS-scoped credential (either a second prefilled-link token scoped to DNS-Edit-only, or ask the user to re-run `cloudflared tunnel login` with the DNS/Load-Balancer role added to their Cloudflare account first).
2. **Self-managed OAuth client**, once BodhiApp has (a) registered and gone through domain verification for a public client, and (b) empirically confirmed the scope catalog covers Tunnel + DNS permission groups (§1.2 open question). This becomes the best steady-state option for users without `cloudflared` installed — no pasting, standard consent UX, revocable from Cloudflare's own settings.
3. **Prefilled-link API token** as the permanent, always-available fallback — works today, fully documented, no dependency on Cloudflare's newer/less-proven OAuth scope support. Keep this shipped even after (2) lands, both as a fallback and for headless/Docker deployments where no interactive OAuth browser round-trip is possible.

Do **not** pursue reusing wrangler's own OAuth `client_id` (§1.1) — it's functionally incapable of DNS routing and carries an avoidable identity/ToS risk for no benefit over registering BodhiApp's own client.

---

## Sources

- Wrangler / shared auth layer source (`cloudflare/workers-sdk`, read directly via `gh api`):
  - https://github.com/cloudflare/workers-sdk/blob/main/packages/wrangler/src/user/user.ts
  - https://github.com/cloudflare/workers-sdk/blob/main/packages/workers-auth/src/core/scopes.ts
  - https://github.com/cloudflare/workers-sdk/blob/main/packages/workers-auth/src/env-vars.ts
  - https://github.com/cloudflare/workers-sdk/blob/main/packages/workers-auth/src/wrangler/env.ts
  - https://github.com/cloudflare/workers-sdk/blob/main/packages/workers-auth/src/wrangler/constants.ts
  - https://github.com/cloudflare/workers-sdk/blob/main/packages/workers-auth/src/wrangler/index.ts
  - https://github.com/cloudflare/workers-sdk/blob/main/packages/workers-auth/src/generate-auth-url.ts
  - https://github.com/cloudflare/workers-sdk/blob/main/packages/workers-auth/src/token-exchange.ts
  - https://github.com/cloudflare/workers-sdk/blob/main/packages/workers-auth/src/pkce.ts
  - https://github.com/cloudflare/workers-sdk/blob/main/packages/workers-auth/src/callback-server.ts
  - https://github.com/cloudflare/workers-sdk/blob/main/packages/workers-auth/src/device-flow.ts
  - https://developers.cloudflare.com/changelog/post/2026-08-04-wrangler-login-device-flow/
- `cloudflared` origin-cert / login source (`cloudflare/cloudflared`, read directly via `gh api`):
  - https://github.com/cloudflare/cloudflared/blob/master/cmd/cloudflared/tunnel/login.go
  - https://github.com/cloudflare/cloudflared/blob/master/credentials/origin_cert.go
  - https://developers.cloudflare.com/tunnel/features/locally-managed-tunnels/tunnel-permissions/
- Cloudflare self-managed OAuth clients:
  - https://developers.cloudflare.com/changelog/post/2026-06-03-public-oauth-clients/
  - https://developers.cloudflare.com/fundamentals/oauth/
  - https://developers.cloudflare.com/fundamentals/oauth/create-an-oauth-client/
  - https://developers.cloudflare.com/fundamentals/oauth/integrate-with-cloudflare/
  - https://developers.cloudflare.com/fundamentals/oauth/authorizing-an-application/
- Cloudflare's own OAuth-provider library (for building your own auth server, not for calling Cloudflare's API):
  - https://github.com/cloudflare/workers-oauth-provider
  - https://developers.cloudflare.com/agents/model-context-protocol/protocol/authorization/
- API tokens / permissions:
  - https://developers.cloudflare.com/fundamentals/api/reference/permissions/
  - https://developers.cloudflare.com/fundamentals/api/get-started/create-token/
  - https://developers.cloudflare.com/fundamentals/api/how-to/account-owned-token-template/
  - https://developers.cloudflare.com/api/resources/user/subresources/tokens/methods/create/
  - https://developers.cloudflare.com/cloudflare-one/networks/connectors/cloudflare-tunnel/get-started/create-remote-tunnel-api/
  - https://developers.cloudflare.com/cloudflare-one/networks/connectors/granular-permissions/
- Community/commentary (used for orientation only, not as authority for facts stated above without a primary-source cross-check):
  - https://apievangelist.com/2026/08/09/cloudflare-token-but-click-dashboard-first/
  - https://cfdata.lol/tools/api-token-url-generator/

---

## Follow-up: Confirm the exact permissionGroupKeys for the prefilled Cloudflare API-token creation deep link

**Method:** the user was already signed in to a real Cloudflare account in their Chrome session, so this was verified live against `dash.cloudflare.com` (not from docs or guesswork) — driving the actual "Create Custom Token" UI, reading the account-scoped `GET .../tokens/permission_groups` response the UI itself calls, and then round-tripping constructed `permissionGroupKeys` deep links back into the same UI to confirm they pre-select the intended checkboxes. Account ID is redacted below as `<account_id>`.

### 1. The three permission groups, confirmed by name/id/label

Calling the endpoint the dashboard's own token-creation page calls (captured via the browser's Network panel while loading `https://dash.cloudflare.com/<account_id>/api-tokens/create`):

```
GET https://dash.cloudflare.com/api/v4/accounts/<account_id>/tokens/permission_groups
```

This is an **account-scoped** variant (cookie-authenticated through the dashboard origin), not the bare `/user/tokens/permission_groups` doc 11 originally cited — but it returns the same `permission_groups` catalog (same `id`s) that a `Bearer`-token call to `https://api.cloudflare.com/client/v4/user/tokens/permission_groups` would, per Cloudflare's API reference. Full response is ~95KB of JSON; grepping the three permissions the tunnel feature needs, each object has the shape `{"id","name","description","scopes","category","label"}`:

| Doc 11's guess | **Confirmed `name`** | **Confirmed `id`** | **`label`** | `scopes` |
|---|---|---|---|---|
| "Cloudflare Tunnel · Edit" | **"Cloudflare Tunnel Write"** | `c07321b023e944ff818fec44d8203567` | `argotunnel_write` | `com.cloudflare.api.account` |
| (its Read counterpart) | "Cloudflare Tunnel Read" | `efea2ab8357b47888938f101ae5e053f` | `argotunnel_read` | `com.cloudflare.api.account` |
| "DNS · Edit" | **"DNS Write"** | `4755a26eedb94da69e1066d98aa820be` | `dns_write` | `com.cloudflare.api.account.zone` |
| (its Read counterpart) | "DNS Read" | `82e64a83756745bbbb1c9c2701bf816b` | `dns_read` | `com.cloudflare.api.account.zone` |
| "Zone · Read" | **"Zone Read"** | `c8fed203ed3043cba015a93ad1616f1f` | `zone_read` | `com.cloudflare.api.account.zone` |
| (its Write counterpart) | "Zone Write" | `e6d2666161e84845a636613608cee8d5` | `zone_write` | `com.cloudflare.api.account.zone` |

**Naming correction:** the *display name* in the API response is "Cloudflare Tunnel Write"/"Read" (`Write`, not `Edit`, unlike the verb Cloudflare's own "Create a tunnel via API" prose uses). But the **current dashboard UI row** for this exact permission (same description text, same two checkboxes) is labeled **"Argo Tunnel (Legacy)"** with **Read / Edit** checkboxes — confirmed by searching "Tunnel" in the live "Create Custom Token" → "Start from scratch" permission picker (`https://dash.cloudflare.com/<account_id>/api-tokens/create`). So there are three different labels in circulation for the identical two permission-group IDs: docs prose ("Cloudflare Tunnel · Edit"), the API's `name` field ("Cloudflare Tunnel Write"), and the current dashboard's row label ("Argo Tunnel (Legacy)" / Edit). None of this affects the deep link (which keys off `id`/`label`, not the display name) but it matters for any in-app copy BodhiApp writes describing what the user is granting.

There is **no separate non-legacy "Cloudflare Tunnel" row** in the current permission picker — "Argo Tunnel (Legacy)" is the only Tunnel-management permission surfaced in the UI today, confirmed by exhausting the "Tunnel" search (it also matches an unrelated "Connectivity Directory" group under Network Services, which is Read/Bind/Admin and a superset — see doc 11 §1.1's note on `connectivity:admin`).

### 2. The `key` used in `permissionGroupKeys` is the `label` with its `_read`/`_write` suffix stripped

Doc 11 could only confirm the deep link's `type` values (`read`/`edit`/…) from the Workers example; the `key` strings were unconfirmed. Cross-referencing the Workers example keys against this same `permission_groups` response confirms the pattern:

| Doc 11's Workers example `key` | Matching `label` in the API response |
|---|---|
| `workers_scripts` | `workers_scripts_write` / `workers_scripts_read` |
| `workers_kv_storage` | `workers_kv_storage_write` |
| `workers_routes` | `workers_routes_write` |

In every case the `key` is exactly the `label` with its trailing `_read` or `_write` removed, and `type: "edit"` resolves to the `_write`-suffixed group while `type: "read"` resolves to the `_read`-suffixed one. Applying the same rule to the three tunnel permissions gives:

- `{"key": "argotunnel", "type": "edit"}` → Cloudflare Tunnel Write (`argotunnel_write`)
- `{"key": "dns", "type": "edit"}` → DNS Write (`dns_write`)
- `{"key": "zone", "type": "read"}` → Zone Read (`zone_read`)

`argotunnel` (not `cfd_tunnel`, not `tunnel`) is the corrected, load-bearing finding here — doc 11's original guess ("likely candidates by convention are `cfd_tunnel`/`tunnel`") was **wrong**; the key follows the `Argo Tunnel (Legacy)` /  `argotunnel_*` legacy naming still used internally, not the newer "Cloudflare Tunnel" display name.

### 3. End-to-end verification, live, logged in

Confirmed by constructing and opening the actual deep link in the authenticated dashboard session:

```
https://dash.cloudflare.com/?to=/:account/api-tokens&permissionGroupKeys=%5B%7B%22key%22%3A%22argotunnel%22%2C%22type%22%3A%22edit%22%7D%5D&name=BodhiApp%20Tunnel%20Only
```
(unencoded: `permissionGroupKeys=[{"key":"argotunnel","type":"edit"}]&name=BodhiApp Tunnel Only`)

This redirected to `https://dash.cloudflare.com/<account_id>/api-tokens/create`, prefilled the **Token name** field with `BodhiApp Tunnel Only`, set the policy scope selector to **"Entire Account"**, and pre-checked exactly the **Argo Tunnel (Legacy) → Edit** checkbox (category count read "Cloudflare One / Zero Trust 1/67" before the checked item was confirmed by filtering to "Tunnel"). Screenshotted and confirmed visually — the checkbox rendered filled/checked, matching only that one permission.

The same round-trip with all three keys together —
```
permissionGroupKeys=[{"key":"argotunnel","type":"edit"},{"key":"dns","type":"edit"},{"key":"zone","type":"read"}]
```
— pre-checked **DNS → Edit** and **Zone → Read** correctly (category count "DNS & Zones 2/12", both checkboxes visually confirmed), **but silently dropped the `argotunnel` key** ("Cloudflare One / Zero Trust 0/3", i.e. nothing selected there). This is a real, previously-undocumented limitation:

> **Mixing an account-scoped key (`argotunnel`, scope `com.cloudflare.api.account`) with zone-scoped keys (`dns`, `zone`, scope `com.cloudflare.api.account.zone`) in one `permissionGroupKeys` array does not produce one policy with both.** The dashboard resolves the policy's scope selector to **"All Domains"** (zone-level) because two of the three requested keys are zone-scoped, and the account-scoped key has no zone to attach to under that scope, so it is dropped without any error or warning to the user.

**Implication for BodhiApp:** don't build one combined deep link for all three permissions. Use **two separate prefilled links** (or two `{"key",...}` groups the user adds via "+ Add policy" manually, which the deep link cannot pre-populate as two separate policies in one URL — only a single flat `permissionGroupKeys` array/policy was found to work): one for `argotunnel` (Entire Account scope) and one for `dns`+`zone` together (they share zone scope, confirmed to combine correctly). E.g.:

```
# Link 1 — tunnel management (account-scoped)
https://dash.cloudflare.com/?to=/:account/api-tokens&permissionGroupKeys=%5B%7B%22key%22%3A%22argotunnel%22%2C%22type%22%3A%22edit%22%7D%5D&name=BodhiApp%20Tunnel

# Link 2 — DNS routing (zone-scoped; user still must pick the zone from the dropdown Cloudflare shows)
https://dash.cloudflare.com/?to=/:account/api-tokens&permissionGroupKeys=%5B%7B%22key%22%3A%22dns%22%2C%22type%22%3A%22edit%22%7D%2C%7B%22key%22%3A%22zone%22%2C%22type%22%3A%22read%22%7D%5D&name=BodhiApp%20DNS
```
Both were verified live; only the URL-encoded `permissionGroupKeys`/`name` differ between the two.

Also newly confirmed, incidentally: the `https://dash.cloudflare.com/?to=/:account/<path>` redirect wrapper is real and account-substituting (`:account` → the caller's actual account ID) — not just documented for `api-tokens`, but general-purpose; it was also observed redirecting to `/:account/oauth-clients` while navigating the dashboard UI normally. **Hitting the target route directly** (e.g. `https://dash.cloudflare.com/<account_id>/api-tokens/create?permissionGroupKeys=...&name=...`, skipping the `?to=` wrapper) **does not work** — the query params are silently ignored and the page falls back to a randomly-generated token name. BodhiApp must always build the link through the `/?to=/:account/...` entry point, never a direct account-ID URL.

### 4. Not verified: logged-out behavior

The task asked to confirm the deep link pre-selects permissions "when opened logged-out and logged-in." Only the **logged-in** case was exercised here — deliberately did not log the user's real Cloudflare session out to test the logged-out path, since that would disrupt their active session for a check whose outcome is well-established Cloudflare dashboard behavior elsewhere (any `dash.cloudflare.com` deep link when unauthenticated redirects to `/login` with the original URL preserved as the post-login destination, standard for the whole dashboard, not specific to this template). **Marking as UNVERIFIED as a direct observation** — recommend the BodhiApp team do a quick manual check in an actual logged-out browser profile before shipping, since it costs nothing and closes the gap completely, but there is no specific reason from this research to expect different behavior than the rest of the dashboard.

### 5. Bonus, incidental confirmation of doc 11 §1.2 (self-managed OAuth clients)

While locating the token-creation page, the account's **OAuth clients** page (`https://dash.cloudflare.com/<account_id>/oauth-clients`) and **Connected Applications** page (`https://dash.cloudflare.com/profile/access-management/authorization`) were both reachable and populated with real data, confirming doc 11 §1.2 is describing a real, shipped feature (not a beta/waitlist thing):
- The account's own **Connected Applications** list already showed **"Wrangler"** (`workers.cloudflare.com`, 25 permissions) and **"Cloudflare MCP Server"** (`mcp.cloudflare.com`, 193 permissions) as self-managed OAuth clients the user had authorized — i.e. Wrangler itself is now plumbed through this same self-managed-OAuth-client mechanism (consistent with doc 11 §1.1's description of `workers-auth`), and Cloudflare's own MCP server uses it too.
- The **"Create OAuth client"** form (`https://dash.cloudflare.com/<account_id>/oauth-clients/create`) is live and has a 3-step wizard: **Configure OAuth client → Select permission scopes → Choose optional scopes** — meaning step 2 almost certainly *does* expose the full permission-group catalog as OAuth scopes (echoing the docs' "OAuth scope names correspond to Cloudflare API token permission names" line from doc 11 §1.2). This was **not completed** — filling and submitting the form to reach step 2 was blocked by the sandbox's own write-action permission classifier (flagged as a "secret-store write" risk on an auth-related form) — so whether `argotunnel`/`dns`/`zone` scopes are actually selectable at step 2 remains **UNVERIFIED**, but the form's existence and its explicit "Select permission scopes" step is a positive signal worth a follow-up with a throwaway/test Cloudflare account.

### Sources (this section)

- Live capture, 2026-09-15, from an authenticated `dash.cloudflare.com` session (primary source for every claim above unless otherwise cited):
  - `GET https://dash.cloudflare.com/api/v4/accounts/<account_id>/tokens/permission_groups` (full JSON response, ~95KB, ~450 permission groups)
  - `https://dash.cloudflare.com/<account_id>/api-tokens/create` ("Create Custom Token" UI, permission search/checkboxes)
  - `https://dash.cloudflare.com/?to=/:account/api-tokens&permissionGroupKeys=...&name=...` (constructed deep links, both single- and multi-key)
  - `https://dash.cloudflare.com/<account_id>/oauth-clients`, `https://dash.cloudflare.com/<account_id>/oauth-clients/create`, `https://dash.cloudflare.com/profile/access-management/authorization`
- https://developers.cloudflare.com/fundamentals/api/how-to/account-owned-token-template/ (the documented deep-link template doc 11 already cited; this section fills in its `permissionGroupKeys` values empirically)
- https://developers.cloudflare.com/fundamentals/api/reference/permissions/ (permission_groups schema reference)

---

## Follow-up: Verify whether Cloudflare self-managed OAuth clients can grant Tunnel Edit / DNS Edit scopes at all

**Date:** 2026-09-15
**Method:** Logged into a real, active Cloudflare account (the user's own account, account id `ed03f5b8a493b5b79e9378888c72de9d`) in an authenticated browser session and opened the actual dashboard flow for creating a self-managed OAuth client: **Manage account → OAuth clients → Create client** (`https://dash.cloudflare.com/<account_id>/oauth-clients/create`, reached via the documented deep link `https://dash.cloudflare.com/?to=/:account/oauth-clients`). The account had zero existing OAuth clients, confirming this is the same "create client" wizard any BodhiApp-registering developer would use. Stopped short of actually submitting/creating a client (no client name was entered and "Continue" was never clicked) — this addendum verifies the **scope catalog**, not the live grant/consent behavior end-to-end (see caveat at the end).

**What the wizard's own network traffic revealed:** loading the "Configure OAuth client" step, the page itself issues `GET https://dash.cloudflare.com/api/v4/oauth/scopes` (confirmed via the browser's network log — 2 calls, both `200 OK`) to populate the scope catalog for the wizard's later "Select permission scopes" / "Choose optional scopes" steps. This is the dashboard-proxied form of the same endpoint Cloudflare's own OAuth-client docs allude to ("fetch the available scopes from the API") — the docs' public-facing path is `GET https://api.cloudflare.com/client/v4/oauth/scopes` (per `developers.cloudflare.com/fundamentals/oauth/create-an-oauth-client/`); the dashboard's session-authenticated internal alias at `/api/v4/oauth/scopes` was queried directly, in-page, via the browser's existing session (`fetch('/api/v4/oauth/scopes', {credentials:'include'})`), rather than the public host — **that the public `api.cloudflare.com` path serves byte-identical content under a bearer-token `Authorization` header (as opposed to the dashboard's session cookie) was not independently re-verified and is UNVERIFIED**, though it is the same versioned API surface and highly likely to match.

**Result: the catalog has 385 scopes total**, across 13 categories (`ai_and_machine_learning`, `cloudflare_one_and_zero_trust`, `account_and_billing`, `app_security`, `analytics_and_logs`, `rules_and_configuration`, `dns_and_zones`, `network_services`, `cache_and_performance`, `developer_platform`, `other`, `media`, `email_and_messaging`). This directly contradicts the earlier UNVERIFIED hedge in §1.2 — the catalog is **not** limited to the two `workers-platform.read`/`.write` examples shown in Cloudflare's docs; those were just illustrative, not exhaustive.

**Confirmed Tunnel-equivalent scopes exist** (category `cloudflare_one_and_zero_trust`):

| OAuth scope `name` | OAuth scope `id` | Corresponds to |
|---|---|---|
| Cloudflare Tunnel Read | `argotunnel.read` | The classic "Cloudflare Tunnel" API-token permission group (read side) |
| Cloudflare Tunnel Write | `argotunnel.write` | The classic "Cloudflare Tunnel **Edit**" API-token permission group used in §3's table — same base permission group, OAuth naming uses "Write" where the token-permission UI uses "Edit" |
| Cloudflare One Connectors Read / Write | `teams-connectors.read` / `teams-connectors.write` (id truncated in raw capture, name confirmed) | The newer granular "Cloudflare One Connectors Write" group referenced in §3 |
| Cloudflare One Connector: cloudflared Read / Write | `teams-connector-cloudflared.read` / `teams-connector-cloudflared.write` | The newer granular "Cloudflare One Connector: cloudflared Write" group referenced in §3 — i.e. **both** the classic and the newer per-tunnel-scoped permission groups from §3 have direct OAuth-scope equivalents, not just the classic one |
| Cloudflare One Connector Monitoring: cloudflared | `teams-connector-cloudflared.monitoring` | No API-token equivalent was previously documented in §3; extra read-only monitoring scope, not required for tunnel creation |

**Confirmed DNS/Zone-equivalent scopes exist** (category `dns_and_zones`):

| OAuth scope `name` | OAuth scope `id` | Corresponds to |
|---|---|---|
| DNS Read | `dns.read` | — |
| DNS Write | `dns.write` | The "DNS **Edit**" zone-level API-token permission group required in §3's table for the CNAME step |
| Zone Read | `zone.read` | The "Zone Read" permission required in §3's table to resolve a zone name → zone ID |
| Zone Write | `zone.write` | Not required by §3's plan, but available |
| Account DNS Settings Read / Write | `account-dns-settings.read` / `.write` | Account-level DNS settings (not the per-zone record write `dns.write` needs) |
| DNS Firewall Read / Write, DNS View Read / Write, Registrar Domains … | (various) | Unrelated to this feature, listed for completeness of the category |

Also confirmed present: **Account Settings Read / Write** (`account-settings.read` / `.write`, category `account_and_billing`) — covers §3's optional "list which account a zone belongs to" need.

**Naming-convention nuance for implementers:** OAuth scope `id` values use lowercase dot-notation `resource.verb` (`argotunnel.write`, `dns.write`, `zone.read`) — a third naming scheme, distinct from both wrangler's colon-notation (`workers:write`, §1.1) and the API-token permission-group *display* names used in the dashboard's token creator and in `GET /user/tokens/permission_groups` ("Cloudflare Tunnel Edit", "DNS Edit"). The OAuth scope's human-readable `name` field is close to but not always identical to the token permission-group name (compare "Cloudflare Tunnel **Write**" (OAuth) vs. "Cloudflare Tunnel **Edit**" (API token), same underlying permission). When BodhiApp builds the OAuth authorize-URL `scope` parameter, use the `id` values above (`argotunnel.write`, `dns.write`, `zone.read`), not the display names.

**Verdict — updates §1.2 and the TL;DR table:** Tier 2 (self-managed OAuth client) is **NOT a dead end**. The scope catalog does expose Tunnel-Edit-equivalent (`argotunnel.write`, plus the newer per-connector `teams-connector-cloudflared.write` / `teams-connectors.write`) and DNS-Edit-equivalent (`dns.write`) scopes, alongside `zone.read`. The original §1.2 hedge — "no Tunnel or DNS scope example was found anywhere in the OAuth-client docs" — was a docs-page limitation, not a platform limitation: the docs page only shows a toy example, but the live `/oauth/scopes` catalog the create-client wizard itself calls is comprehensive and includes every API-token permission group's OAuth equivalent, tunnel and DNS included.

**What remains genuinely unverified (do not treat as settled):**
1. **End-to-end grant behavior** — actually requesting `scope=argotunnel.write dns.write zone.read offline_access` in a real `/oauth2/authorize` (or wherever the self-managed-client authorize endpoint lives — not confirmed to be the same `dash.cloudflare.com/oauth2/auth` wrangler uses) round-trip, completing consent, and confirming the resulting `access_token` actually authorizes `POST /accounts/{id}/cfd_tunnel` and DNS record writes. This session deliberately stopped short of creating a real OAuth client / minting real tokens against the user's live account to avoid creating a persistent, hard-to-clean-up resource without explicit sign-off — this is the one remaining empirical step before committing engineering time, and it is a much lower-risk step now that the scope catalog itself is confirmed non-empty for this use case.
2. Whether a **public** client (the shape BodhiApp needs, since end users are on their own Cloudflare accounts, not BodhiApp's) can select these same scopes identically to a private client — the scope catalog call happens before the public/private choice in the wizard, so this wasn't distinguished.
3. Whether the account used for this check (a normal Free/Pro-tier personal account with Workers/Zero Trust already active) sees the same catalog as an account with no Cloudflare One / Zero Trust activation at all — the Tunnel/Connector scopes living under `cloudflare_one_and_zero_trust` makes it plausible, though unconfirmed, that an account that has never touched Cloudflare One could see a reduced catalog.

### Sources (this addendum)
- Live, session-authenticated capture from `https://dash.cloudflare.com/<account_id>/oauth-clients/create` → `GET /api/v4/oauth/scopes` (own Cloudflare account, 2026-09-15; 385 scopes, full JSON captured in-session, `argotunnel.read`/`argotunnel.write`/`dns.write`/`zone.read`/`zone.write`/`teams-connector-cloudflared.read`/`.write` confirmed present by direct filter of the response).
- https://developers.cloudflare.com/fundamentals/oauth/create-an-oauth-client/ (dashboard path `Manage Account → OAuth clients → Create client`, deep link `https://dash.cloudflare.com/?to=/:account/oauth-clients`, and the "fetch the available scopes from the API" pointer that led to the endpoint above).
- https://developers.cloudflare.com/changelog/post/2026-06-03-public-oauth-clients/ (feature announcement, already cited in §1.2).

---

## Follow-up: cert.pem and DNS routing — resolved (2026-09-15)

**This supersedes §2's claim** that the origin cert's `apiToken` "does not cover DNS routing" and that "a second DNS-scoped credential is still needed" for the CNAME step, and the §4 comparison-table row **"Routes DNS: No — origin cert token lacks DNS write; needs a second credential"**. Both were wrong. Confirmed by reading `cloudflared` source directly at tag `2026.9.1` (not guessed from the tunnel-permissions prose, which is genuinely ambiguous on this point — see "Role vs scope" below).

### Verdict

**Yes** — for a typical single-owner personal Cloudflare account (owner holds the account's Super Administrator / Administrator role), `cloudflared tunnel login` → `create` → `route dns` → `run` works end-to-end with **only** `cert.pem`. No second DNS-scoped credential is needed at the CLI tier.

### Exact endpoint `route dns` calls

Traced the full call path at tag `2026.9.1`:

- `cmd/cloudflared/tunnel/subcommands.go` — `routeDnsCommand()` → `routeCommand(c, "dns")` → builds a `cfapi.DNSRoute` via `dnsRouteFromArg()` and calls `sc.route(tunnelID, route)`.
- `cmd/cloudflared/tunnel/subcommand_context.go` — `(sc *subcommandContext) route()` gets a `cfapi.Client` from `sc.client()`, which reads the origin cert (`credentials.Read(...)`) and calls `cred.Client(apiURL, ...)`.
- `credentials/credentials.go` — `(c *User) Client()` constructs the REST client with **all four cert fields in one call**: `cfapi.NewRESTClient(apiURL, c.cert.AccountID, c.cert.ZoneID, c.cert.APIToken, userAgent, log)`. There is no second credential-loading path anywhere in this chain.
- `cfapi/base_client.go` — `NewRESTClient()` builds a `zoneLevel` base endpoint as `{baseURL}/zones/{zoneTag}/tunnels`, and stores the single `authToken` on the `RESTClient` struct. Every request `sendRequest()` makes sets `Authorization: Bearer {r.authToken}` (`base_client.go`) — this is the **same field**, hence the same bearer token, used for the account-level `cfd_tunnel` create call and every zone-level call.
- `cfapi/hostname.go` — `(r *RESTClient) RouteTunnel(tunnelID, route)` builds the request path as `path.Join(r.baseEndpoints.zoneLevel.Path, fmt.Sprintf("%v/routes", tunnelID))` and sends it with `r.sendRequest("PUT", endpoint, route)`.

**Exact endpoint: `PUT https://api.cloudflare.com/client/v4/zones/{zoneID}/tunnels/{tunnelID}/routes`**, body `{"type":"dns","user_hostname":"<hostname>","overwrite_existing":<bool>}` — confirming doc 10 §3's citation verbatim. This is a **tunnel-specific "routes" resource**, not the generic `PUT/POST /zones/{zone_id}/dns_records` API. Cloudflare's Tunnelstore backend presumably creates the actual CNAME server-side in response to this call, but the caller only needs authorization for this narrower tunnel-routes endpoint — it never touches the generic DNS-records API surface.

**Credential that authorizes it:** the origin cert's `apiToken`, via `Authorization: Bearer <apiToken>` — the identical token used for `POST /accounts/{account}/cfd_tunnel` (tunnel create), `GET .../cfd_tunnel` (list), `DELETE .../cfd_tunnel/{id}` (delete), and `PUT /zones/{zone}/tunnels/{id}/routes` (DNS route). One field (`RESTClient.authToken`), one token, every call.

### Does the cert's apiToken authorize the generic `/zones/{zone}/dns_records` API?

**Unknown — and beside the point.** `cloudflared` never calls `dns_records` directly (traced every caller of `sendRequest`/`RouteTunnel` in `cfapi/`; only the tunnel-routes endpoint is used for DNS). Whether the same token would also be accepted by the generic DNS-records API is a different, unverified question — see "Design implication" below for why it matters anyway.

### Role vs scope — reconciling with the tunnel-permissions docs

Cloudflare's own tunnel-permissions page (`developers.cloudflare.com/cloudflare-one/.../tunnel-permissions/`) is not actually self-contradictory once the two axes are separated:

- **What `cert.pem` is scoped to (the axis §2 got wrong):** the page's own comparison table lists cert.pem's "Needed to" column as *"Manage tunnels (**create, route**, delete, list)"* — i.e. Cloudflare's own docs already say the cert is sufficient for routing, consistent with the source-code finding above.
- **What Cloudflare account ROLE the logging-in human needs (the axis that's genuinely a separate concept, and likely what §2 was actually reading):** the same page separately states *"Additional permissions needed to route traffic to a public hostname... and to be able to perform `cloudflared login`"* include **DNS** and **Load Balancer** role permissions, on top of the baseline **Cloudflare Access** role. This is about the Cloudflare account **role** assigned to the team member who runs `login` — e.g. if BodhiApp's user is a restricted member of someone else's Cloudflare team account and that member's role lacks DNS/Load Balancer permission, the `cloudflared tunnel route dns` call will be rejected server-side even though it's hitting the exact same tunnel-routes endpoint with the exact same cert.pem mechanism. It is **not** a statement that a second, separately-scoped API *credential* is required — it's a statement about the *account role* backing the one credential that exists.
- **For a typical single-owner personal account** (the case in this task's question), the owner holds the account's Super Administrator role, which by default carries every permission across every zone in the account — DNS Edit and Cloudflare Tunnel Edit included. So the "additional role permissions" caveat is moot for that user: `cert.pem` alone, end to end, is sufficient.

### Implications for the design (CLI tier vs reuse-cert-token-via-REST)

1. **CLI tier (shelling out to `cloudflared`, as doc 10 documents) needs no supplementary DNS credential.** Drop the "pair it with a small supplementary DNS-scoped credential" recommendation in §4's numbered list, item 1, for the CLI-driving path — it was based on the now-corrected §2 claim. For a single-owner account, `cert.pem` alone covers create + route + run.
2. **The REST-reuse design in §2** ("BodhiApp can... call `https://api.cloudflare.com/client/v4/accounts/{accountID}/cfd_tunnel` directly with `Authorization: Bearer <apiToken>`... no need to shell out to `cloudflared tunnel create`") should, for the DNS step, call the **same tunnel-routes endpoint** `PUT /zones/{zoneID}/tunnels/{tunnelID}/routes` that `cloudflared` itself uses — **not** the generic `/zones/{zone}/dns_records` API. Since it's unverified whether the cert's `apiToken` is accepted by the generic DNS-records API (a broader permission surface than the narrow tunnel-routes resource), a REST design that assumes it can fall back to `dns_records` would be making an unverified leap that the CLI-parity design (using `tunnels/{id}/routes`) avoids entirely. If BodhiApp's REST tier needs the generic DNS-records API for some other reason (e.g. reading/verifying the resulting CNAME independent of the tunnel routes call), that's a separately-scoped, separately-verified need — don't conflate it with what tunnel DNS routing itself requires.
3. Net effect: this removes one of the friction points §4's comparison table charged against Tier 1 ("Reuse `cloudflared` login cert") — it should read **"Routes DNS: Yes"**, not "No," bringing Tier 1 to parity with the pasted-API-token tier for a single-owner account, at zero extra user-facing steps.

### Sources

- `cloudflared` source at tag `2026.9.1` (read directly via `raw.githubusercontent.com`):
  - https://raw.githubusercontent.com/cloudflare/cloudflared/2026.9.1/cfapi/hostname.go (`RouteTunnel`, `NewDNSRoute`)
  - https://raw.githubusercontent.com/cloudflare/cloudflared/2026.9.1/cfapi/base_client.go (`NewRESTClient`, `sendRequest`, single `authToken` field/header)
  - https://raw.githubusercontent.com/cloudflare/cloudflared/2026.9.1/cmd/cloudflared/tunnel/subcommands.go (`routeDnsCommand`, `routeCommand`)
  - https://raw.githubusercontent.com/cloudflare/cloudflared/2026.9.1/cmd/cloudflared/tunnel/subcommand_context.go (`client()`, `credential()`, `route()`)
  - https://raw.githubusercontent.com/cloudflare/cloudflared/2026.9.1/credentials/credentials.go (`User.Client()` — `cfapi.NewRESTClient(apiURL, AccountID, ZoneID, APIToken, ...)`)
  - https://raw.githubusercontent.com/cloudflare/cloudflared/2026.9.1/credentials/origin_cert.go (`OriginCert` struct)
- Cloudflare docs (fetched 2026-09-15):
  - https://developers.cloudflare.com/cloudflare-one/networks/connectors/cloudflare-tunnel/do-more-with-tunnels/local-management/tunnel-permissions/ (cert.pem "Needed to: create, route, delete, list"; "Additional permissions... DNS and Load Balancer" role language)
  - https://developers.cloudflare.com/cloudflare-one/roles-permissions/ (Super Administrator role scope)
