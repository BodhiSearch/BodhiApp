# Third-party app authorization — request/response specification

**BodhiApp · `/ui/apps/auth/`**

---

## 1. Flow

```
third-party app          browser              BodhiApp                    Keycloak
      |                     |                     |                          |
   1  |--navigate---------->|--GET /ui/apps/auth/?<oauth params>------------->|
      |                     |                     |
   2  |                     |  not logged in -> stash URL, log in, return
      |                     |                     |
   3  |                     |  GET /bodhi/v1/apps/{client_id}/info
      |                     |  consent screen; user decides
      |                     |                     |
   4  |                     |--POST /bodhi/v1/apps/access-requests----------->|
      |                     |<-{ id, redirect_url }
      |                     |
   5  |                     |--window.location.href = redirect_url----------->|
      |                     |                                                 | (approve: Keycloak /authorize)
      |                     |                                                 | (deny/error: app redirect_uri)
   6  |<--code + state------|<------------------------------------------------|
   7  |--exchange at Keycloak token endpoint----------------------------------->|
```

The app is never authenticated at step 1 — it is a top-level browser navigation, so no request
headers can be attached. Everything the app needs to communicate travels in the query string.

---

## 2. Scope vocabulary

A public contract once published.

### App-facing — sent by the app to `/ui/apps/auth/`

| Token | Meaning |
|---|---|
| `scope_user_user` | requests the User role ceiling |
| `scope_user_power_user` | requests the PowerUser role ceiling |
| `scope_apps_llms` | requests LLM inference access |
| `scope_apps_mcps` | requests MCP/tool access |

Exactly one role token is required. **At least one of `scope_apps_llms` / `scope_apps_mcps` is
required.** Unknown tokens are rejected, not ignored.

### Keycloak-facing — composed by the BodhiApp backend

```
openid profile email roles <approved_role_scope> scope_access_request:<resource-client-id>.<uuid>
```

`scope_user_user` and `scope_user_power_user` are registered Keycloak client scopes
(`bodhi-realm-v26.json:36-55`) with `include.in.token.scope: true` and
`display.on.consent.screen: true`, so they are forwarded and rendered on Keycloak's consent card with
their configured text ("Basic/Inference/Read-Only APIs" / "… + Write/Download-Models/Configure-Models
APIs").

The forwarded role is the **approved** role, not the requested one — the user may downgrade at
consent.

`scope_apps_llms` / `scope_apps_mcps` are **not** forwarded. They are consumed by the consent screen;
the resulting grant is stored in `ApprovedResourcesV1` on the access-request row. They require no
Keycloak realm configuration.

---

## 3. `/ui/apps/auth/` request

```
https://bodhi.example/ui/apps/auth/
  ?client_id=app-acme-7b2f
  &redirect_uri=https%3A%2F%2Facme.dev%2Fcb
  &response_type=code
  &state=4d1c8b2e9a...
  &code_challenge=E9Melhoa2OwvFrEMTJguCHaoeK1t8URWbuGJSstw-cM
  &code_challenge_method=S256
  &scope=scope_user_user+scope_apps_llms+scope_apps_mcps
  &source_access_request_id=01K4R7M1L5Q6S8T9U0V1W2X3Y4      (optional)
```

| Parameter | Required | Validation |
|---|---|---|
| `client_id` | yes | resolves to a Bodhi app client |
| `redirect_uri` | yes | exact-match against the app's registered URIs — see §4 |
| `response_type` | yes | must be `code` |
| `state` | yes | opaque, echoed back unmodified |
| `code_challenge` | yes | opaque, passed through to Keycloak |
| `code_challenge_method` | yes | must be `S256` |
| `scope` | yes | every token recognised; one role token; ≥1 `scope_apps_*` token |
| `source_access_request_id` | no | reauthorize trigger — see §5 |

Unauthenticated visitors are handled by the existing mechanism: `AppInitializer.tsx:79` stashes
`window.location.href` — the full URL including query string — in `bodhi-return-url`, and
`auth/callback/index.tsx:29-32` restores it after login.

`/apps/auth` must be added to `BARE_PREFIXES` (`resolveShellRoute.ts:16`) so the page renders in
`BareLayout` rather than the full app shell.

---

## 4. Errors and denial

Every failure returns to the app's `redirect_uri` with `error`, `error_description` and the original
`state`, per RFC 6749 §4.1.2.1. The backend composes the URL; the frontend navigates to it without
inspection.

| Condition | `error` | `error_description` |
|---|---|---|
| user denies | `access_denied` | `user denied the access request` |
| missing/malformed params, wrong `response_type`, non-S256 | `invalid_request` | names the offending parameter |
| unknown scope token | `invalid_scope` | names the token |
| no `scope_apps_*` token | `invalid_scope` | `empty access requested, at least one of scope_apps_llms, scope_apps_mcps is required` |
| `client_id` unknown or not a Bodhi app client | `unauthorized_client` | — |
| internal failure | `server_error` | — |

### `redirect_uri` validation

The backend fetches the app's registered `redirect_uris` from Keycloak (§6) and **exact-matches** —
no trailing-`*` wildcards, no RFC 8252 loopback port relaxation.

- **Match** → redirect automatically, both on success and on error.
- **Mismatch** → render an error on the frontend and redirect nowhere, per RFC 6749 §4.1.2.1
  (*"MUST NOT automatically redirect the user-agent to the invalid redirection URI"*).
- **`redirect_uris` unavailable from Keycloak** → redirect automatically without validation.

Exact-match-only is stricter than Keycloak's matcher, so the two can only disagree in the fail-closed
direction. An error redirect carries `error`, `error_description` and `state` — never a code or
token.

---

## 5. Reauthorize

**Trigger:** `source_access_request_id` in the query string. The app already holds this value as the
`access_request_id` claim in its current token.

The backend validates that the id resolves to a row matching `(tenant_id, app_client_id, user_id)`
with `status = approved`. A non-matching id is ignored and the request proceeds as a fresh
authorization.

**Fallback:** when `source_access_request_id` is absent but a prior approved grant exists for
`(tenant_id, app_client_id, user_id)`, the newest such grant is offered as an **unselected**
affordance — *"You previously granted this app access to 3 models. Restore those selections?"* — not
a silently pre-filled form.

**Consent form as a diff.** When a prior grant is in play, the screen renders three groups rather
than a blank form:

- **Already granted, still requested** — pre-checked, shown as continuing.
- **Newly requested** — highlighted; this is what the user is deciding.
- **Previously granted, no longer requested** — shown as being relinquished, with the option to keep.

The third group is required so an app narrowing from `scope_apps_llms scope_apps_mcps` to
`scope_apps_llms` visibly gives up MCP access rather than silently retaining it.

The new row stores `source_access_request_id` so the chain stays auditable. Prior grants remain live
— current behaviour is unchanged.

---

## 6. `GET /bodhi/v1/apps/{app_client_id}/info`

Session auth. Named to match `/bodhi/v1/info`.

```json
{
  "app_name": "Acme Chat",
  "app_description": "Acme's coding assistant",
  "redirect_uris": ["https://acme.dev/cb"],
  "previous_grant": {
    "id": "01K4R7M1L5Q6S8T9U0V1W2X3Y4",
    "approved_role": "scope_user_user",
    "approved": { "version": "1", "...": "..." }
  }
}
```

`app_name` and `app_description` come from Keycloak's app-info endpoint. This retires the
`app_name` / `app_description` columns on the row, which are always `None` at create
(`access_request_service.rs:150`) — today the consent screen shows a raw client id.

`redirect_uris` serves error-redirect validation (§4) and is displayed on the consent screen so the
user sees where the token will go before approving.

**Keycloak prerequisite:** `redirect_uris` is not currently returned —
`ResourceService.java:363` returns name and description only. Adding it is part of this work.

Model and MCP pickers use the existing list endpoints unchanged.

---

## 7. `POST /bodhi/v1/apps/access-requests`

Create and approve in one call. The user is authenticated at consent, so there is no Draft status, no
10-minute TTL, and no NULL-tenant window.

**Auth:** session, `ResourceRole::User` or above — today's approve guard
(`routes_apps.rs:289-302`), moved unchanged.

### Request

```json
{
  "query_params": {
    "client_id": "app-acme-7b2f",
    "redirect_uri": "https://acme.dev/cb",
    "response_type": "code",
    "state": "4d1c8b2e9a...",
    "code_challenge": "E9Melhoa2OwvFrEMTJguCHaoeK1t8URWbuGJSstw-cM",
    "code_challenge_method": "S256",
    "scope": "scope_user_user scope_apps_llms scope_apps_mcps",
    "source_access_request_id": "01K4R7M1L5Q6S8T9U0V1W2X3Y4"
  },
  "decision": "approve",
  "approved_role": "scope_user_user",
  "approved": {
    "version": "1",
    "models_list": false,
    "models_access": { "type": "specific", "ids": ["llama3:8b"] },
    "mcps_list": false,
    "mcps": [],
    "mcps_access": { "type": "specific", "ids": ["01JC7Q..."] }
  }
}
```

`query_params` is the query string **as received** — not destructured and re-serialised by the page.
The backend re-derives `client_id`, the requested role and the requested categories from it rather
than trusting a client-side reading, so adding a parameter later needs no schema change.

`decision` is `approve` or `deny`. On `deny`, `approved_role` and `approved` are omitted.

`tenant_id` and `user_id` come from the session and are never accepted from the body.

The backend re-asserts, against `query_params`, that `approved_role ≤ requested_role` and
`approved_role ≤ ResourceRole::max_user_scope()` for the approving session
(`auth_objs.rs:61-66`).

### Response

```json
{
  "id": "01K5S8N2M6R7T9V0W1X2Y3Z4A5",
  "redirect_url": "https://id.getbodhi.app/realms/bodhi/protocol/openid-connect/auth?response_type=code&client_id=app-acme-7b2f&redirect_uri=https%3A%2F%2Facme.dev%2Fcb&state=4d1c8b2e9a...&code_challenge=E9Mel...&code_challenge_method=S256&scope=openid+profile+email+roles+scope_user_user+scope_access_request%3Aresource-acme-9f3c.01K5S8N2M6R7T9V0W1X2Y3Z4A5"
}
```

On denial the same shape is returned, with `redirect_url` pointing at the validated `redirect_uri`
carrying `error=access_denied&error_description=...&state=...`. On a `redirect_uri` mismatch, the
call returns an error the frontend renders in place.

---

## 8. Frontend behaviour

```ts
const { redirect_url } = await submitConsent(body);
window.location.href = redirect_url;   // full-page navigation, not router.navigate
```

The frontend never composes a URL, never appends a scope, and never branches on approve versus deny.
`validateAuthUrl`, `appendScopeToAuthUrl`, `readState` and `buildErrorRedirect`
(`review/-shared/authUrl.ts`) are deleted along with the `auth_url` / `error_url` query contract.

---

## 9. Envelope

### Requested resources

`RequestedResourcesV1` leaves the wire. Its five fields
(`access_request_objs.rs:184-201`) are UI drivers that bind nothing; the consent screen is now driven
by scope.

| Field | Fate |
|---|---|
| `models_access` | `scope_apps_llms` |
| `mcps_access` | `scope_apps_mcps` |
| `mcp_servers: [{url}]` | dropped — per-URL MCP requests are out of scope |
| `models_list` | consent-screen toggle only; not requestable |
| `mcps_list` | consent-screen toggle only; not requestable |

`requested_role` becomes the role scope token.

An app that names no `scope_apps_*` token is rejected rather than defaulted, replacing today's
`models_access: true` default.

### Approved resources

`ApprovedResourcesV1` (`access_request_objs.rs:222-253`) is **unchanged**, so the enforcement path,
the `ResourceGrants` trait and the least-privilege defaults need no modification.

| Field | Default | Meaning |
|---|---|---|
| `models_list` | `false` | app may enumerate all models |
| `models_access` | `Specific{[]}` — deny | `All` or `Specific{ids}` |
| `mcps_list` | `false` | app may enumerate all MCPs |
| `mcps` | `[]` | retained as stored data with a live reader in `allows_mcp_connect`; no longer populated, since per-URL requests are dropped |
| `mcps_access` | `Specific{[]}` — deny | the granted MCP instances |

### Removed from the request contract

`exchange`, caller-supplied `source_access_request_id` in the body, `auth_url`, `error_url`, and the
`RequestedResources` JSON envelope.

---

## 10. Keycloak changes

1. `GET /realms/{realm}/bodhi/users/apps/{client_id}/info` returns `redirect_uris` in addition to
   name and description (`ResourceService.java:349-363`, `AppInfoResponse.java`).
2. No realm-configuration change. `scope_apps_llms` / `scope_apps_mcps` never reach Keycloak, and
   `scope_user_user` / `scope_user_power_user` / `scope_access_request` are already registered client
   scopes.
