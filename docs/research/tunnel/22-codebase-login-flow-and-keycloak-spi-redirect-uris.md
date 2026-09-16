# 22 — Login-flow redirect-URI handling & Keycloak SPI redirect-URI sync

**Date:** 2026-09-15
**Scope:** How BodhiApp composes `redirect_uri` today, what the Bodhi Keycloak SPI (repo `keycloak-bodhi-ext`) can and cannot do with a client's redirect URIs today, and the concrete design for a new "register tunnel URL as redirect URI" sync. Extends `01-bodhi-app-codebase-map.md` §6 and corrects one assumption in `bodhiapp-cloudflare-tunnel-feasibility.md`'s Keycloak section (see "Correction" callout below).

> Repos: BodhiApp paths are relative to this repo's root. SPI paths are relative to `/Users/amir36/Documents/workspace/src/github.com/BodhiSearch/keycloak-bodhi-ext`.

## TL;DR

- **(a)** Login via `https://<tunnel-host>/...` does **not** just work once the redirect URI is registered. BodhiApp's non-explicit-host branch of `auth_initiate` echoes the request's `Host` header but **not** its scheme or port — it always uses `settings.public_scheme()` (defaults `http`) and `settings.public_port()` (defaults `1135`), and always appends the port explicitly. Enabling the tunnel must also set `BODHI_PUBLIC_SCHEME=https` (or an equivalent tunnel-aware override), or every constructed `redirect_uri` will be `http://<tunnel-host>:1135/ui/auth/callback` instead of `https://<tunnel-host>/ui/auth/callback` — a string Keycloak will reject as not-registered even if you registered the correct one. See §1.
- **(b)** No endpoint exists today that can mutate an **existing** resource/tenant client's redirect URIs — only `getAppInfo` (for **app**-type OAuth clients, a different client class) reads them back. Add one self-service endpoint, `PUT /realms/{realm}/bodhi/resources/redirect-uris`, authenticated the same way as `make-resource-admin`/`has-resource-admin` (service-account bearer token, self-targeting — no separate authorization check needed). See §2.
- **(c)** Add `AuthService::update_redirect_uris(client_id, client_secret, redirect_uris) -> Result<()>` to `services::auth::AuthService`, implemented via `forward_request`-style plumbing (reuse `get_client_access_token` for the service-account exchange). Call it from bootstrap/startup code and from the tunnel enable/disable handler directly (no new `TunnelService` needed purely for this) — same shape as `TenantService`/`AuthService` are already injected into route handlers and bootstrap. See §3.
- **(d)** Web origins (CORS) are **already** self-maintaining: `ResourceService.createResourceClientInternal` sets `client.addWebOrigin("+")` at creation (`ResourceService.java:139`), and Keycloak resolves `"+"` to the client's redirect-URI origins **dynamically on every CORS check** (`WebOriginsUtils.resolveValidWebOrigins`, verified against Keycloak 26.6.4 source) — so adding the tunnel host as a redirect URI is *automatically* sufficient for CORS too, no separate web-origins write needed. Post-logout redirect URIs (`post.logout.redirect.uris` client attribute) are **not set anywhere** in the SPI and **not used anywhere** in BodhiApp (`auth_logout` never calls Keycloak's OIDC end-session endpoint) — irrelevant today, flag only if RP-initiated SSO logout is added later. See §4.
- **(e)** No version negotiation exists between BodhiApp and the SPI today beyond a one-way `x-bodhi-app-version` header the SPI does not read. Detect an SPI without the new endpoint the same way `AppClientInfo.redirect_uris: Option<...>` already models "older SPI" — a 404 from the new endpoint (unknown JAX-RS route) — and degrade to "skip the sync, log a warning" rather than failing tunnel enable. See §5.

---

## 1. BodhiApp side: how `redirect_uri` is actually built

### 1.1 Login initiate — `crates/routes_app/src/auth/routes_auth.rs:88-103`

```rust
let callback_url = if settings.get_public_host_explicit().await.is_some() {
  // Covers explicit-host deployments like RunPod.
  settings.login_callback_url().await
} else {
  if let Some(request_host) = extract_request_host(&headers) {
    format!(
      "{}://{}:{}{}",
      settings.public_scheme().await, request_host, settings.public_port().await,
      services::LOGIN_CALLBACK_PATH
    )
  } else {
    settings.login_callback_url().await
  }
};
```

Two branches:

1. **`get_public_host_explicit().await.is_some()`** (`crates/services/src/settings/setting_service.rs:390-405`) — true only when `BODHI_PUBLIC_HOST` is explicitly configured (env/db/settings-file — not `Default` source) or RunPod auto-detection kicked in. In this branch, `callback_url = login_callback_url()` = `public_server_url() + LOGIN_CALLBACK_PATH` (`setting_service.rs:463-464`), and `public_server_url()` (`setting_service.rs:425-433`) elides the port for `(http,80)`/`(https,443)`.
2. **Else** (the common desktop/local-network case) — `request_host` comes from the incoming `Host` header only (`crates/routes_app/src/shared/utils.rs:3-18`, `extract_request_host`, hostname-portion only, port stripped). Scheme and port are **not** derived from the request at all — always `settings.public_scheme()`/`settings.public_port()` — and the format string **always appends `:{port}`**, unlike `public_server_url()`.

`settings.public_scheme()` (`setting_service.rs:355-365`) and `settings.public_port()` (`setting_service.rs:407-422`) both fall through to `BODHI_SCHEME`/`BODHI_PORT` when `BODHI_PUBLIC_SCHEME`/`BODHI_PUBLIC_PORT` are unset — i.e. **`http`** and **`1135`** by default (`crates/services/src/settings/constants.rs:31-33`, `DEFAULT_SCHEME`/`DEFAULT_PORT`).

**Consequence for the tunnel feature:** a request arriving through `cloudflared` with `Host: mytunnel.example.com` (over HTTPS, no port in the URL) falls into branch 2 and — with default settings — produces `callback_url = "http://mytunnel.example.com:1135/ui/auth/callback"`. That is wrong on two axes (scheme, port) and will not match whatever gets registered in Keycloak (`https://mytunnel.example.com/ui/auth/callback` most likely), so the OAuth authorize call itself will fail with Keycloak's "Invalid parameter: redirect_uri" **before** any tunnel-registration work even comes into play.

**Fix required alongside redirect-URI registration:** when the tunnel is enabled, either (a) set `BODHI_PUBLIC_SCHEME=https` and `BODHI_PUBLIC_PORT=443` so branch 2 composes `https://mytunnel.example.com:443/...` — note this **still doesn't elide the port** the way `public_server_url()` does, so the registered Keycloak URI must include the literal `:443` suffix to match, **or** (b) set `BODHI_PUBLIC_HOST=mytunnel.example.com` explicitly, which routes through branch 1 and correctly elides port 443 via `public_server_url()`. Option (b) is cleaner but has a side effect — see §1.3.

Symmetric logic exists in `setup_create` (`crates/routes_app/src/setup/routes_setup.rs:120-185`, uses the same `extract_request_host` + `LOOPBACK_HOSTS` construction to seed the **initial** redirect-URI list at setup) and is the origin of the multi-entry redirect list (loopback hosts + request host + server IP) that already exists per client.

### 1.2 Callback handler — `routes_auth.rs:176-326`

`auth_callback` re-reads `callback_url` from the session (stored at initiate time, `routes_auth.rs:120-122`) and passes it verbatim to `auth_flow.exchange_auth_code(...)` as the `redirect_uri` token-exchange parameter (`routes_auth.rs:242`). It performs **no host validation of its own** against the incoming request — whatever was stored at initiate time is what's exchanged. So once initiate composes the right URL and Keycloak accepts it as a registered redirect URI, the callback leg is a non-issue for the tunnel case.

### 1.3 Logout — `routes_auth.rs:350-361`

`auth_logout` only calls `session.delete()` and returns `{location: "{public_server_url()}/ui/login"}` — it never calls Keycloak's `/protocol/openid-connect/logout` (OIDC RP-initiated logout / end-session) endpoint, so **no `post_logout_redirect_uri` is ever sent to Keycloak** and no server-side Keycloak SSO session is torn down by this flow (see also §4.2).

### 1.4 Interaction with `canonical_url_middleware` (side effect of picking option (b) above)

`crates/routes_app/src/middleware/redirects/canonical_url_middleware.rs:14-75` only activates when `get_public_host_explicit()` is `Some` (`canonical_url_middleware.rs:25-28`) — i.e. exactly the condition triggered by setting `BODHI_PUBLIC_HOST` to the tunnel host. Once active, it 301-redirects **any** GET/HEAD request whose Host/scheme/port doesn't match `public_server_url()` to the canonical URL (`should_redirect_to_canonical`, same file). Concretely: if `BODHI_PUBLIC_HOST` is set to the tunnel hostname to fix §1.1's scheme/port problem, then browsing the app over `localhost:1135` (still expected to keep working for the desktop app) would 301 to the tunnel URL — likely undesirable. The tunnel-enable code needs to either scope this middleware's canonical check to tunnel-origin requests only, or accept the redirect (a genuinely local click would then bounce out to the public hostname), or extend the setting model with a tunnel-specific host that doesn't trip `get_public_host_explicit()`. This is a design decision to make explicitly in the implementation plan, not an incidental detail — flagging it here since it falls directly out of tracing the redirect_uri code.

### 1.5 `/bodhi/v1/apps/*` consent flow is a different, unrelated client class

`crates/routes_app/src/apps/consent.rs:1-4,129-165` calls `AuthService::get_app_client_info` / `services::match_redirect_uri` to validate **third-party OAuth app** redirect URIs against `AppClientInfo.redirect_uris` — this is the `bodhi.client_type=app` client class (Keycloak public clients created via `POST /realms/{realm}/bodhi/apps`, see SPI §2.4), used when an external application asks a BodhiApp instance for delegated access. **This is not the client whose redirect URIs the tunnel needs to update.** The client that needs the tunnel host added is BodhiApp's **own** login client — the `resource` (standalone) or `tenant` (multi-tenant) client created via `setup_create`/`tenants_create`. `bodhiapp-cloudflare-tunnel-feasibility.md` §"Keycloak redirect-URI flow" conflates these two client classes when it says "extend `AuthService::update_redirect_uris(client_id, uris)`" without distinguishing which client type the SPI-side implementation must target — this doc corrects that: the new endpoint must operate on `bodhi.client_type=resource` clients (both `bodhi-resource-*` and `bodhi-tenant-*` prefixes share this attribute — see §2.1), **not** the `app`-type client that `getAppInfo`/`AppClientInfo` already covers.

### 1.6 Tenant creation redirect URIs — `crates/routes_app/src/tenants/routes_tenants.rs:99-155`

`tenants_create` builds `redirect_uris = vec![format!("{}{}", public_server_url(), LOGIN_CALLBACK_PATH)]` (`routes_tenants.rs:111-112`) — a single-entry list (no loopback/LAN variants, unlike `setup_create`), sent once at creation via `AuthService::create_tenant` (`auth_service.rs:837-873`, POSTs `CreateTenantRequest` to `{auth_api_url}/tenants`, bearer-authed with the dashboard user's token). No add/remove ever happens after this — confirms `01-bodhi-app-codebase-map.md`'s finding that redirect URIs are set-once today.

### 1.7 Client credentials storage — `crates/services/src/tenants/tenant_repository.rs`

`client_id`/`client_secret` live in the `tenants` table (`tenant_entity.rs:5-20`), secret stored as `encrypted_client_secret`/`salt_client_secret`/`nonce_client_secret`, AES-GCM under the app's `BODHI_ENCRYPTION_KEY`-derived KEK (`tenant_repository.rs:114-135` `decrypt_tenant_row`, `:146-175` `create_tenant` encrypts on write). This is the **only** table with a legacy (pre-v2 KEK) decrypt path (`tenant_repository.rs:15-17` comment), per the `feedback_encryption_key_derivation` project convention. Any new sync code that needs `client_id`+`client_secret` to authenticate to the SPI reads them straight off `TenantService::get_tenant`/`get_standalone_app` (already-decrypted `TenantRow`/`Tenant`) — no new crypto plumbing needed.

---

## 2. SPI side: what exists today and the endpoint to add

### 2.1 Client model — resource and tenant clients are the same underlying type

`ResourceService.createResourceClientInternal` (`ResourceService.java:126-182`) is the **shared** creation path for both `newResource` (`ResourceService.java:73-90`, `POST /realms/{realm}/bodhi/resources`) and `createTenant` (`ResourceService.java:95-121`, `POST /realms/{realm}/bodhi/tenants`). Both set `client.setAttribute(ATTR_KEY_CLIENT_TYPE, CLIENT_TYPE_RESOURCE)` (`ResourceService.java:148`, constant value `"resource"` — **tenant clients also get `client_type=resource`**, distinguished only by client-ID prefix: `bodhi-resource-`/`test-resource-` vs `bodhi-tenant-`/`test-tenant-`, `Constants.java:11-14`). Redirect URIs are set once, at creation, via `client.addRedirectUri(redirectUri)` in a loop over `request.redirectUris` (`ResourceService.java:136-138`) — **no update path exists**.

Contrast with **app** clients (`AppClientService.newApp`, `AppClientService.java:46-113`): also only set redirect URIs at creation (`AppClientService.java:66-68`), also `client_type=app`, and the **only** existing read-back is `getAppInfo` (`BodhiResourceProvider.java:190-207` → `ResourceService.getAppInfo`, `ResourceService.java:247-263`) — which explicitly filters to `CLIENT_TYPE_APP` (`ResourceService.java:256-258`) and therefore **404s if pointed at a resource/tenant client**. There is genuinely no existing "get or set redirect URIs on a resource/tenant client" endpoint to extend — the tunnel feature needs a wholly new one.

### 2.2 Auth pattern to mirror: self-service via client-credentials token

Two existing endpoints already implement "the client that owns this resource acts on itself" using a service-account bearer token, which is the right auth model for a tunnel-URL sync (BodhiApp always has its own `client_id`/`client_secret`, no separate user/dashboard token is needed or desirable):

- `hasResourceAdmin` (`BodhiResourceProvider.java:133-147` → `ResourceService.hasResourceAdmin`, `ResourceService.java:228-235`)
- `makeFirstResourceAdmin` (`BodhiResourceProvider.java:114-131` → `ResourceService.makeFirstResourceAdmin`, `ResourceService.java:187-223`)

Both call `checkForServiceAccount(authResult)` (`ResourceService.java:303-320`):
```java
private void checkForServiceAccount(AuthenticationManager.AuthResult authResult) throws ResourceProviderException {
  if (authResult == null) throw new ResourceProviderException(Response.Status.UNAUTHORIZED, "invalid session");
  Object serviceAccount = authResult.token().getOtherClaims().get("client_id");
  if (serviceAccount == null) throw new ResourceProviderException(Response.Status.UNAUTHORIZED, "not a service account token");
  String clientId = authResult.token().getIssuedFor();
  if (!clientId.equals(serviceAccount)) throw new ResourceProviderException(Response.Status.UNAUTHORIZED, "client id and authorized party do not match");
  RealmModel realm = session.getContext().getRealm();
  ClientModel client = realm.getClientByClientId(clientId);
  if (client == null) throw new ResourceProviderException(Response.Status.BAD_REQUEST, "client not found");
}
```
Then `clientId = authResult.token().getIssuedFor()` resolves **which client to mutate** — the caller can only ever act on its own client, which is exactly the self-service semantics needed here (no separate authorization check required beyond "is this a valid service-account token for *some* client"). Note this check does **not** gate on `ATTR_KEY_CLIENT_TYPE`, so it works uniformly for both `bodhi-resource-*` and `bodhi-tenant-*` clients (matching what §2.1 established).

On the Rust side this is obtained via `get_client_access_token(client_id, client_secret)` (`auth_service.rs:206-238`, already used internally by `make_resource_admin`, `auth_service.rs:589-618`) — `grant_type=client_credentials&scope=service_account` against the standard OIDC token endpoint, no SPI involvement for the token itself.

### 2.3 New endpoint proposal

**`PUT /realms/{realm}/bodhi/resources/redirect-uris`**

Chosen as a **full-replace `set`**, not incremental `add`/`remove`, for the same reason `setup_create`/`tenants_create` always recompute the complete desired list from scratch rather than mutating incrementally (`routes_setup.rs:132-168`): idempotent, no drift from partial failures, and the caller (BodhiApp) already knows its complete desired set (loopback hosts + LAN host + explicit public host + tunnel host, when enabled) at every sync point. A `POST .../redirect-uris:add` / `:remove` pair is a plausible alternative if a future caller needs pure incrementality, but nothing in this codebase needs it — recommend starting with `set` only and adding `add`/`remove` later only if a concrete caller needs them.

- **Auth:** Bearer service-account token (client-credentials grant of the caller's own `client_id`/`client_secret`), validated via `checkForServiceAccount` exactly as in §2.2.
- **Request:**
  ```json
  { "redirect_uris": ["https://mytunnel.example.com/ui/auth/callback", "http://localhost:1135/ui/auth/callback", "http://127.0.0.1:1135/ui/auth/callback"] }
  ```
  (New `RedirectUrisRequest` DTO, same shape/annotation style as `ClientRequest.java`: `@JsonProperty("redirect_uris") public List<String> redirectUris;`)
- **Response `200`:**
  ```json
  { "client_id": "bodhi-resource-01j...", "redirect_uris": ["http://127.0.0.1:1135/ui/auth/callback", "http://localhost:1135/ui/auth/callback", "https://mytunnel.example.com/ui/auth/callback"] }
  ```
  (New `RedirectUrisResponse` DTO — mirrors `AppInfoResponse`'s "return the sorted list back" convention, `ResourceService.java:260-262`, so BodhiApp can log/verify what actually landed.)
- **Errors:** `401` invalid/non-service-account token (mirrors `hasResourceAdmin`'s `401` cases); `400` empty/malformed list (mirror `validateName`'s style, `ResourceService.java:322-326`); `500` unexpected (mirrors the `tracked()` wrapper's catch-all, `BodhiResourceProvider.java:73-91`).

**Java implementation (`ResourceService.java`, new method alongside `hasResourceAdmin`)**:
```java
public RedirectUrisResponse setRedirectUris(RedirectUrisRequest request) throws ResourceProviderException {
  if (request.redirectUris == null || request.redirectUris.isEmpty()) {
    throw new ResourceProviderException(Response.Status.BAD_REQUEST, "redirect_uris must not be empty");
  }
  AuthenticationManager.AuthResult authResult = new AppAuthManager.BearerTokenAuthenticator(session).authenticate();
  checkForServiceAccount(authResult);
  String clientId = authResult.token().getIssuedFor();
  RealmModel realm = session.getContext().getRealm();
  ClientModel client = realm.getClientByClientId(clientId);
  client.setRedirectUris(new HashSet<>(request.redirectUris)); // ClientModel API, verified against Keycloak 26.6.4
  session.getTransactionManager().commit();
  List<String> sorted = client.getRedirectUris().stream().sorted().collect(Collectors.toList());
  return new RedirectUrisResponse(clientId, sorted);
}
```
`ClientModel.setRedirectUris(Set<String>)` (and `addRedirectUri`/`removeRedirectUri`/`getRedirectUris`) are confirmed present at the pinned SPI Keycloak version (`pom.xml:12`, `<keycloak.version>26.6.4</keycloak.version>`) — verified against `server-spi/src/main/java/org/keycloak/models/ClientModel.java` on the `26.6.4` tag (see Sources). `setRedirectUris` (full replace) matches the `set`-semantics chosen above better than `addRedirectUri` in a loop; use `addRedirectUri`/`removeRedirectUri` only if an incremental variant is added later.

**Provider wiring (`BodhiResourceProvider.java`, new route next to `hasResourceAdmin`)**:
```java
@PUT
@Path("resources/redirect-uris")
@Consumes(MediaType.APPLICATION_JSON)
@Produces(MediaType.APPLICATION_JSON)
@Operation(summary = "Set redirect URIs", description = "Replaces the calling resource/tenant client's redirect URIs. Requires service account token.", tags = {"Resource Management"})
public Response setRedirectUris(RedirectUrisRequest request) {
  return tracked("bodhi.resources.set-redirect-uris",
      () -> Response.ok(resourceManagementService.getResourceService().setRedirectUris(request)).build());
}
```
(`jakarta.ws.rs.PUT` import needed; every other verb is already imported.)

### 2.4 Tests to write (SPI repo)

Follow `TenantEndpointTest.java`'s pattern (integration test extending `BaseTest`, real Keycloak testcontainer, `bodhiProviderClient` helper + `restassured`):

- New `RedirectUrisEndpointTest.java`:
  - `testSetRedirectUrisSuccess` — register a resource client (`registerClientAndReturnClientPair()`), get its service-account token (`getServiceAccountToken(...)`, same helper `TenantEndpointTest.java` uses), PUT a new list, assert `200` + response body, then re-fetch via Keycloak admin client (`realm.clients().findByClientId(...)`) and assert `getRedirectUris()` matches exactly (mirrors `ResourceClientRegistrationTest.testRegisterResourceCreatesClientWithCorrectConfiguration`'s assertion style, `ResourceClientRegistrationTest.java:41-58`).
  - `testSetRedirectUrisWorksOnTenantClient` — same, but starting from a tenant client (`createTenantResponse`), confirming the shared `client_type=resource` attribute means the endpoint is prefix-agnostic (§2.1).
  - `testSetRedirectUrisUnauthorizedUserToken` — call with a user token instead of a service-account token (mirrors `testCreateTenantUnauthorizedNonDashboardToken`'s shape) → `401`.
  - `testSetRedirectUrisUnauthorizedNoToken` → `401` (mirrors `testCreateTenantUnauthorizedNoToken`).
  - `testSetRedirectUrisEmptyListRejected` → `400`.
  - `testSetRedirectUrisWrongClientCannotMutateAnother` — service-account token from client A, but conceptually the endpoint has no `client_id` request field to spoof (it always uses `getIssuedFor()`), so this test mainly documents/locks in that there is no cross-client path — assert client B's redirect URIs are untouched after client A's call.
- Add an `httpyac-scripts/` request block to `resource-management.http` alongside the existing `make_resource_admin`/`has_resource_admin` blocks (same `{{resource_service_token}}` bearer), for manual verification parity with the rest of that file.

---

## 3. Rust side: the `AuthService` method to add

### 3.1 Trait + implementation

Add to `services::auth::AuthService` (`crates/services/src/auth/auth_service.rs:65-146`), next to `create_tenant`:
```rust
/// SPI endpoint: PUT /realms/{realm}/bodhi/resources/redirect-uris
/// Self-service — authenticates as `client_id` via client-credentials, replaces that
/// client's full redirect-URI set. `Ok(None)` when the SPI predates this endpoint (404).
async fn update_redirect_uris(
  &self,
  client_id: &str,
  client_secret: &str,
  redirect_uris: Vec<String>,
) -> Result<Option<Vec<String>>>;
```
Implementation on `KeycloakAuthService` mirrors `make_resource_admin` (`auth_service.rs:589-618`) for the service-account token acquisition, but needs the 404-as-Ok(None) branch for §5's compat handling instead of `make_resource_admin`'s uniform error path:
```rust
async fn update_redirect_uris(&self, client_id: &str, client_secret: &str, redirect_uris: Vec<String>) -> Result<Option<Vec<String>>> {
  let access_token = self.get_client_access_token(client_id, client_secret).await?;
  let endpoint = format!("{}/resources/redirect-uris", self.auth_api_url());
  let response = self.client.put(&endpoint)
    .bearer_auth(access_token.secret())
    .json(&serde_json::json!({ "redirect_uris": redirect_uris }))
    .header(HEADER_BODHI_APP_VERSION, &self.app_version)
    .send().await?;
  match response.status() {
    s if s.is_success() => {
      #[derive(Deserialize)]
      struct Resp { redirect_uris: Vec<String> }
      Ok(Some(response.json::<Resp>().await?.redirect_uris))
    }
    reqwest::StatusCode::NOT_FOUND => Ok(None), // SPI predates this endpoint
    status => {
      let error_text = response.text().await?;
      Err(AuthServiceError::AuthServiceApiError { status: status.as_u16(), body: error_text })
    }
  }
}
```
No new `AuthServiceError` variant is needed — the existing `AuthServiceApiError { status, body }` (`auth_service.rs:44-46`) covers every non-404 failure, matching every other method in this trait.

`forward_request` (`auth_service.rs:791-835`) is the SPI's generic low-level forwarder and is explicitly documented as "mostly used by tests to call cleanup" (`auth_service.rs:129`) — it's a fine mechanical fit (it already returns `(status, json)` so the 404 branch is trivial with it too), but a dedicated typed method is more consistent with every other non-test SPI call in this file (`register_client`, `create_tenant`, `make_resource_admin`, all hand-roll their own request/response types) — recommend the typed method above over routing through `forward_request`.

### 3.2 Where it gets called from

Not a route handler concern primarily — the sync needs to fire from three places per the product decision ("sync only on enable/disable and at app startup, never continuously"):

1. **Tunnel enable/disable route handler** (new, under `admin_session_apis` per `01-bodhi-app-codebase-map.md` §4) — has `AuthScope` and can call `auth_scope.auth_service().update_redirect_uris(...)` directly, the same way `tenants_create` calls `auth_scope.auth_service().create_tenant(...)` (`routes_tenants.rs:109,116-123`) without an intermediate service layer.
2. **App startup** (`lib_bodhiserver`'s `native_init.rs`/`AppServiceBuilder`, or wherever the future `TunnelService`'s "resume tunnel if it was enabled" logic lives) — this code only has `Arc<dyn AppService>`, not `AuthScope`, so it needs `app_service.auth_service().update_redirect_uris(...)` directly (raw `AppService` accessor, matching the "Infrastructure uses `AppService` directly" rule in `crates/CLAUDE.md`).

Because both call sites need it, **`update_redirect_uris` belongs on `AuthService` itself** (as designed above), not hidden inside a future `TunnelService` — `TunnelService` (when it's added) should *depend on* `Arc<dyn AuthService>` and call this method as one step of its enable/disable/startup-resync logic, exactly as `DefaultTenantService` depends on `Arc<dyn DbService>` (`tenant_service.rs:37-40`) rather than owning DB plumbing itself.

The **desired full list** to pass each time is: existing loopback/LAN entries (recompute via the same logic `setup_create` uses, `routes_setup.rs:139-163`) **plus** the tunnel's callback URL (`https://<tunnel-host>/ui/auth/callback`, no port — see §1.1's port-elision caveat if `BODHI_PUBLIC_PORT` ends up non-standard) when the tunnel is enabled, **minus** it when disabled. This needs the current registered list as a baseline; either re-derive it deterministically (safest — avoids ever depending on a stale read) or `GET`/list it first. This design doc doesn't require adding a `GET .../redirect-uris` — recompute-from-settings is sufficient and matches existing conventions (nothing reads redirect URIs back today except `getAppInfo`, which is the wrong client type per §1.5) — but a `GET` is a one-line addition to the SPI endpoint (`@GET` alongside `@PUT` on the same path) if the implementation turns out to want to preserve unknown/manually-added entries.

### 3.3 Test coverage (Rust side)

Per `.claude/skills/test-services` / `test-routes-app` conventions:
- `services`: `test_auth_service.rs` — mock the SPI HTTP call (existing pattern in that file for `register_client`/`create_tenant`), cover success (full list returned), 404-as-`Ok(None)`, and a non-404 error status mapping to `AuthServiceApiError`.
- `routes_app`: whichever new tunnel route handler calls this — `MockAuthService::update_redirect_uris` (already `#[mockall::automock]`-generated per `auth_service.rs:63`) to assert it's called with the right args on enable/disable, and that an `Ok(None)` (old-SPI) result degrades to a warning response/log rather than a hard failure (§5).
- `server_app`/`lib_bodhiserver`: live integration only makes sense once the SPI's actual deployed version is known to have the endpoint — gate any live E2E assertion behind a capability check rather than assuming.

---

## 4. Other Keycloak client fields

### 4.1 Web origins / CORS — already automatic, no new write needed

`ResourceService.createResourceClientInternal` sets `client.addWebOrigin("+")` unconditionally at creation (`ResourceService.java:139`) — confirmed by the existing test assertion `assertThat(fetchedClient.getWebOrigins(), containsInAnyOrder("+"))` (`ResourceClientRegistrationTest.java:44`). `AppClientService.newApp` does the same for app clients (`AppClientService.java:69`). Verified against Keycloak 26.6.4 source (`services/src/main/java/org/keycloak/protocol/oidc/utils/WebOriginsUtils.java`):
```java
public static Set<String> resolveValidWebOrigins(KeycloakSession session, ClientModel client) {
  Set<String> origins = new HashSet<>();
  if (client.getWebOrigins() != null) origins.addAll(client.getWebOrigins());
  if (origins.contains(Constants.INCLUDE_REDIRECTS)) { // "+"
    origins.remove(Constants.INCLUDE_REDIRECTS);
    for (String redirectUri : RedirectUtils.resolveValidRedirects(session, client.getRootUrl(), client.getRedirectUris())) {
      if (redirectUri.startsWith("http://") || redirectUri.startsWith("https://")) origins.add(UriUtils.getOrigin(redirectUri));
    }
  }
  return origins;
}
```
This runs on every CORS preflight check (`DefaultCors.checkAllowedOrigins`), not just at client-creation time — so **once the tunnel host is added as a redirect URI, its origin is automatically an allowed CORS origin too**, with zero additional SPI work. `bodhiapp-cloudflare-tunnel-feasibility.md`'s implementation surface list doesn't mention web origins at all; this confirms that omission was correct (nothing to add there).

### 4.2 Post-logout redirect URIs — not used anywhere today

`grep`-verified: neither `post.logout.redirect.uris`/`post_logout_redirect_uri` nor Keycloak's OIDC end-session endpoint appear anywhere in either repo (SPI `src/main/java` and `src/test/java`; BodhiApp `crates/`). BodhiApp's logout is purely local-session (`routes_auth.rs:350-361`, §1.3) — Keycloak's own SSO session is never explicitly ended by BodhiApp today, so there is no `post_logout_redirect_uri` parameter ever sent and nothing to register for the tunnel host. **Not in scope for this feature.** If a future change adds RP-initiated logout (calling Keycloak's `/protocol/openid-connect/logout?post_logout_redirect_uri=...`), the same `redirect-uris` sync point should also set the `post.logout.redirect.uris` client attribute (Keycloak constant `OIDCConfigAttributes.POST_LOGOUT_REDIRECT_URIS`, confirmed present in the pinned Keycloak version's `OIDCConfigAttributes.java`) — flagging for awareness, not for this implementation.

### 4.3 Nothing else on `ClientModel` is set per-redirect-target

`createResourceClientInternal` (`ResourceService.java:126-182`) sets a fixed set of client properties (`standardFlowEnabled`, `directAccessGrantsEnabled`, `serviceAccountsEnabled`, token-exchange attribute, roles/groups) that are **not** redirect-URI-dependent — no other field needs updating when the tunnel host is added/removed.

---

## 5. Versioning / compatibility: detecting an SPI without the new endpoint

There is **no** existing version-negotiation mechanism between BodhiApp and the SPI:

- `HEADER_BODHI_APP_VERSION` (`auth_service.rs:35`, `x-bodhi-app-version`) is sent by BodhiApp on every request but **never read** by the SPI (verified: no match for `x-bodhi-app-version`/`bodhi-app-version`/`HEADER_BODHI` anywhere in `keycloak-bodhi-ext/src/main/java`) — it's a one-way logging/telemetry header, not a compatibility gate.
- The existing precedent for "older SPI" tolerance is `AppClientInfo.redirect_uris: Option<Vec<String>>` (`auth_service.rs:163-171`):
  ```rust
  /// Registered redirect URIs, sorted; `[]` when none. Absent (`None`) means an older
  /// auth-server extension that predates the field — skip redirect validation then.
  #[serde(default)]
  pub redirect_uris: Option<Vec<String>>,
  ```
  That's a **response-shape** compat trick (missing JSON field deserializes to `None` via `#[serde(default)]`) — it doesn't apply here because the new capability is a **whole new route**, not a new field on an existing response.
- For a whole new route, the only signal available is an HTTP **404** from the SPI (unknown JAX-RS path falls through to Keycloak's/Quarkus's default 404 handler, since `BodhiResourceProvider` has no catch-all route) — exactly what §3.1's `update_redirect_uris` implementation returns as `Ok(None)`.

**Recommended degrade path:** the tunnel enable/disable/startup-resync code treats `Ok(None)` from `update_redirect_uris` as "SPI too old — tunnel will be reachable but login through the tunnel host will fail Keycloak's redirect_uri check; surface a clear warning in the tunnel status/UI ('update your auth server to enable login through the tunnel URL') rather than failing tunnel enable outright." This matches the product framing (a self-hosted instance's operator controls both BodhiApp's version and, indirectly via the shared Railway-deployed auth server, the SPI version — a hard failure would be user-hostile for something recoverable by simply not using tunnel-login until the SPI catches up).

Deployment context for why this matters in practice (`railway.toml:1-2,10-15`): the SPI is deployed as a single Docker image per environment (`ghcr.io/bodhisearch/bodhi-auth-server:latest`) to Railway, with `main`→integration and `prod`→production environments (`railway.toml:10-13`) — i.e. exactly one live SPI version per environment tier, not per-BodhiApp-instance-version pinning. A BodhiApp instance built against a newer contract than the currently-deployed `prod` SPI is a real, expected transient state during rollout (SPI change lands and deploys to Railway *before* or *after* the corresponding BodhiApp release, never atomically) — reinforcing that the 404-degrade path isn't a hypothetical edge case.

---

## Open questions for the implementation plan

1. §1.4 — pick between "set `BODHI_PUBLIC_HOST` to the tunnel host and accept the canonical-redirect side effect" vs. "leave it unset and special-case scheme/port composition for tunnel requests." The latter likely needs a small, explicit change to `auth_initiate`'s branch-2 composition (e.g. recognize the tunnel host via a new setting and force `https`/no-port), rather than reusing `get_public_host_explicit()`'s existing semantics unmodified.
2. §3.2 — whether the redirect-URI sync recomputes the full desired list from settings each time (recommended, matches `setup_create`'s existing idiom) or does a read-modify-write against a new `GET` (only needed if the SPI must preserve entries BodhiApp doesn't know about).
3. §5 — whether the tunnel UI should proactively probe SPI capability (e.g. one throwaway call) at enable-time vs. only discovering incompatibility reactively when the sync call 404s.

---

## Sources

- BodhiApp repo (this repo), files cited inline by path:line above.
- `keycloak-bodhi-ext` repo, files cited inline by path:line above; `pom.xml:12` for the pinned Keycloak version (`26.6.4`).
- Keycloak `26.6.4` tag source, fetched from GitHub raw content, 2026-09-15:
  - `server-spi/src/main/java/org/keycloak/models/ClientModel.java` — `getRedirectUris/addRedirectUri/removeRedirectUri/setRedirectUris`, same shape for web origins. https://raw.githubusercontent.com/keycloak/keycloak/26.6.4/server-spi/src/main/java/org/keycloak/models/ClientModel.java
  - `services/src/main/java/org/keycloak/protocol/oidc/utils/WebOriginsUtils.java` — `resolveValidWebOrigins`, the `"+"` (`Constants.INCLUDE_REDIRECTS`) behavior. https://raw.githubusercontent.com/keycloak/keycloak/26.6.4/services/src/main/java/org/keycloak/protocol/oidc/utils/WebOriginsUtils.java
  - `server-spi-private/src/main/java/org/keycloak/protocol/oidc/OIDCConfigAttributes.java` — `POST_LOGOUT_REDIRECT_URIS = "post.logout.redirect.uris"` constant (UNVERIFIED: exact runtime semantics/format of this attribute — not exercised by either repo today, so not worth deeper verification for this feature). https://raw.githubusercontent.com/keycloak/keycloak/26.6.4/server-spi-private/src/main/java/org/keycloak/protocol/oidc/OIDCConfigAttributes.java
- `01-bodhi-app-codebase-map.md`, `bodhiapp-cloudflare-tunnel-feasibility.md` (this research folder) — prior findings this doc extends/corrects.

---

## Follow-up: Determine keycloak-bodhi-ext (SPI) release/deploy sequencing against BodhiApp's test/CI Keycloak instance

**Repos:** SPI paths below are relative to `/Users/amir36/Documents/workspace/src/github.com/BodhiSearch/keycloak-bodhi-ext`; BodhiApp paths relative to this repo's root, as elsewhere in this doc.

### TL;DR

- **Merging to `main` does NOT auto-deploy anything.** `main`'s push-triggered workflow (`build.yml`) only compiles, quality-checks and tests the SPI — it never builds or pushes a Docker image. The `ghcr.io/bodhisearch/bodhi-auth-server:latest` image (the one both Railway environments run) is only rebuilt/repushed by `release.yml`, which fires **exclusively on a `release/vX.Y.Z` git tag** (or manual `workflow_dispatch`) — a separate, human-triggered step (`make release-server`), not part of the merge-to-main path. See §A.
- **There is no scratch/preview *hosted* Keycloak from a feature branch.** Railway's project only has the two environments declared in `railway.toml` (`prod`, `main`), both **image-sourced**, and Railway's PR-Environments feature (which *would* give per-branch previews) only replicates services that build from a connected GitHub repo — moot here since this service explicitly deploys from a registry image, not from source (`railway.toml:1-2`, "Railway will deploy from Docker image configured in dashboard"). The real scratch mechanism is **local**: `make dev.up` (`Makefile:32-34`) runs `docker-compose up --build` against `docker-compose.yml`, which builds the SPI's `Dockerfile` **from whatever branch/commit is checked out** and serves it on `localhost:8080` — fine for a BodhiApp developer's local loop, not usable from BodhiApp's own hosted CI (GitHub Actions) since nothing in BodhiApp's workflows checks out or runs the SPI repo. See §B.
- **BodhiApp CI actually points its Rust live tests at a *third*, undocumented-in-`railway.toml` Keycloak, not at "integration."** The repo variable is `INTEG_TEST_AUTH_URL = https://test-id.getbodhi.app` (confirmed via `gh variable list`, not `main-id.getbodhi.app` as `.env.test.example` files and this doc's own earlier sections imply) — a still-active, separately-provisioned environment kept specifically because it's the only one with Direct Access Grants *and* Service Accounts both enabled (`realm-import-files/README.md:29-32`), which the Rust password-grant test flows need. The Playwright/E2E suite uses a **different** variable, `INTEG_TEST_MAIN_AUTH_URL = https://main-id.getbodhi.app`, which *does* match `railway.toml`'s `main`/"integration" environment. These two variables can be changed independently but both are **repo-level GitHub Actions variables with no per-branch/per-PR scoping** (confirmed: `gh api repos/BodhiSearch/BodhiApp/environments` shows only a `github-pages` GH Environment, nothing gating these vars) — so pointing either at a scratch SPI deploy affects every concurrent CI run repo-wide, not one PR in isolation. See §C.
- **Net effect for phase sequencing:** because BodhiApp is trunk-based (direct commits to `main`, no PRs — per this repo's own `CLAUDE.md` "Git Workflow" section) and the workflows that exercise live Keycloak are push-triggered, "merge" and "CI run against live Keycloak" are the **same event**, with no buffer window. A commit that adds a `server_app`/`routes_app` live test asserting the new `PUT .../redirect-uris` endpoint (§2.3) needs that endpoint **already deployed and live on `test-id.getbodhi.app` before that commit is pushed**, and a Playwright spec asserting it needs it live on `main-id.getbodhi.app` before push — not "eventually, after the SPI PR merges." See §D for the concrete ordering this implies.

### A. `main`-push vs. release-tag: what each workflow actually does

| Workflow | Trigger | Builds/pushes Docker image? | Where |
|---|---|---|---|
| `build.yml` | `push: branches: ["*"]` (paths-ignore docs), `pull_request` | **No** — `mvn clean compile package -DskipTests`, `make ci.quality`, `make ci.setup` (Playwright), `make ci.test` only | `.github/workflows/build.yml:9-19,45-66` |
| `release.yml` | `push: tags: ["release/v*"]`, `workflow_dispatch` | **Yes** — parses `release/vX.Y.Z`, `make ci.build-release` (multi-platform buildx), pushes `:vX.Y.Z`, `:<short-sha>`, and retags `:latest`, then `make ci.security-scan` (non-blocking Trivy) | `.github/workflows/release.yml:1-25,60-77` |

`make release-server` (Makefile target referenced in `ai-docs/03-context/07-release-process.md` "Creating a Production Release") queries GHCR (`gh api /orgs/{org}/packages/container/bodhi-auth-server/versions`) for the latest published `vX.Y.Z`, computes the next patch, and pushes a `release/vX.Y.Z` git tag — that tag push is what triggers `release.yml`. **This is a manual step a human runs**, not a merge-to-main side effect; `07-release-process.md`'s own "Future Enhancements" list still carries "Deployment Integration: Automated deployment triggers for production releases" as unimplemented, confirming there is no merge→release automation today.

Whether Railway then redeploys automatically once `:latest` is retagged is a **Railway-dashboard-side setting**, not visible in either repo: Railway's own product does support this — its **Image Auto Updates** feature explicitly redeploys a service when a new image is pushed to the same tag it's tracking (`:latest`/`:canary`/`:staging`), with no action needed beyond the registry push (see Sources). `ai-docs/03-context/08-railway-deployment.md`'s "Railway Detection: Railway can be configured to auto-deploy on image updates" is consistent with this feature existing, but **whether it's toggled on for the `keycloak-bodhi-ext` Railway service is UNVERIFIED from either repo** — confirm directly in the Railway project dashboard before relying on it. If it's off, the sequence needs an explicit manual "Deploy" click after every release tag.

**End-to-end timing, worst case (auto-update off or the confirmation above fails):** merge → (human runs `make release-server`) → Maven build + multi-platform Docker buildx + push (`release.yml`, no fixed timeout, typically several minutes) → manual Railway redeploy click → Railway pulls image, restarts, `healthcheckPath = "/realms/master"` with `healthcheckTimeout = 600` and `restartPolicyMaxRetries = 10` (`railway.toml:6-9`) — i.e. up to 10 minutes of health-check budget alone, and no automatic retry-and-alert if the human step is skipped.

### B. No hosted preview environment; the real pre-merge loop is local

- `railway.toml` declares exactly two environments (`prod`, `main`; `railway.toml:11-14`) — no `pr`/`preview`/`test` entry, and the whole file is annotated "This file configures deployment settings only" (`railway.toml:1`) because the actual image reference is chosen in the Railway dashboard, not in git.
- Railway's **PR Environments** product feature (searched independently, see Sources) does exist and would give an ephemeral, isolated per-PR deployment — but it works by replicating services that Railway builds **from a connected GitHub repo**; this project's Keycloak service is explicitly registry-image-sourced (`railway.toml:1-2`, `08-railway-deployment.md` "Docker image source must be configured through Railway dashboard... Cannot use `railway.toml` for image source specification"). A PR Environment cloned from that service would just re-run the *current* `:latest` image, not the feature branch's code — so even if PR Environments were enabled on this Railway project, they would not produce a scratch deploy of unmerged SPI changes without first reconfiguring that service to build from source, which is a bigger change than this feature needs.
- The mechanism that **does** work today is local: `make dev.up` → `docker-compose up --build` (`Makefile:32-34`, `docker-compose.yml:19-22`) builds `Dockerfile` from the working tree (whatever branch is checked out) and serves Keycloak on `http://localhost:8080` with `KC_HOSTNAME_STRICT=false` — no HTTPS/hostname-strictness friction for local use. A BodhiApp engineer developing the tunnel feature can: checkout the SPI feature branch in `keycloak-bodhi-ext`, `make dev.up`, then point their local BodhiApp instance's auth URL setting at `http://localhost:8080` (mirrors how `lib_bodhiserver_napi`'s test config already points at `https://test-id.getbodhi.app` as its default, `crates/lib_bodhiserver_napi/src/test_utils/config.rs:47`, so a local override is a one-line swap of the same knob).
- This local loop is **not reachable from BodhiApp's hosted GitHub Actions CI** — nothing in `.github/workflows/*.yml` checks out, builds, or runs `keycloak-bodhi-ext` as a CI service (no `docker-compose` service block, no cross-repo checkout step referencing it). So a `server_app`/`routes_app` live test or Playwright spec that needs the new endpoint **cannot be exercised in BodhiApp's own CI until the SPI change is deployed to one of the two remote environments** — local scratch dev is for writing/debugging the BodhiApp-side code by hand, not for making CI green.

### C. `INTEG_TEST_AUTH_URL` vs `INTEG_TEST_MAIN_AUTH_URL` — two different variables, two different targets

Confirmed via `gh variable list` in this repo (values as of 2026-09-15):

| Variable | Live value | Consumers | Trigger conditions |
|---|---|---|---|
| `INTEG_TEST_AUTH_URL` | `https://test-id.getbodhi.app` | `crates/server_app/tests/utils/live_server_utils.rs:79-80,831-832`, `crates/server_app/tests/test_live_multi_tenant.rs:35-36`, `crates/routes_app/tests/test_live_multi_tenant.rs:38`, `crates/routes_app/tests/test_live_auth_middleware.rs:127` (read via `std::env::var`, hard `expect`/`Err` if unset — no silent skip, consistent with this project's no-skip-on-missing-env convention) | Wired only in `.github/workflows/build-multiplatform.yml:79` (`server_app`/`routes_app` cargo tests run inside the shared `build-and-test` composite action). That workflow's own trigger is `push: branches: [main]` + `workflow_dispatch` **only** — `build-multiplatform.yml:3-6` has no `pull_request` block, so these live tests never run pre-merge on a PR, only immediately after a push lands on `main` (or on manual dispatch). |
| `INTEG_TEST_MAIN_AUTH_URL` | `https://main-id.getbodhi.app` | Playwright E2E suite (`crates/lib_bodhiserver/tests-js/`) | Wired in `.github/workflows/build.yml:240` (job `playwright-tests`) and `.github/workflows/playwright.yml:1-11,137`. Both of these **do** trigger on `pull_request: branches: [main]` in addition to `push` — `build.yml:1-14` (the earlier grep in this session that suggested build.yml had no `pull_request` trigger only caught the first 5 lines after `on:`; the full block includes it) — so the Playwright/`main-id` path genuinely does run pre-merge, when a PR exists. |

Practical reading: `test-id.getbodhi.app` is not a stale leftover from the documented `test-id` → `main-id` migration (`docs/archive/claude-plans/202602/20260215-e2e-cleanup/20260215-server-app-remove-register-resource.md:5` records that migration for other purposes) — it is a **deliberately still-separate** environment the Rust live-test suite depends on today because `main-id.getbodhi.app`'s realm config has Direct Access Grants disabled (`realm-import-files/README.md:29-32`, `main` row: Direct Access Grants ❌) while `test-id.getbodhi.app`'s has it enabled (✅), and the Rust tests authenticate via password grant. **Its own deploy mechanism is not documented in `railway.toml` at all** (only `prod`/`main` appear in `[environments]`, `railway.toml:11-14`) — UNVERIFIED whether it tracks the same `:latest` image/redeploy path as `main`, or is a separately-managed Railway service/environment. This should be confirmed directly with whoever administers the Railway project before assuming it gets the new endpoint "for free" whenever `main` does.

Neither variable can be scoped to a single run: both are plain repository-level GitHub Actions **variables** (`vars.*`, not `secrets.*`, confirmed present in `gh variable list` output with no `--env` needed), and `gh api repos/BodhiSearch/BodhiApp/environments` returns only the unrelated `github-pages` GH Environment — there is no GH Environment (with its own protection/variable overrides) gating these tests. Temporarily repointing either variable to a scratch SPI deploy (e.g. a Railway one-off service, or an ngrok/cloudflared-fronted `make dev.up` instance) would affect **every** concurrent workflow run across the repo until reverted — there is no built-in per-PR or per-branch override today. A `workflow_dispatch`-with-input override is technically addable (`build-multiplatform.yml`/`playwright.yml` already have `workflow_dispatch:` with no inputs declared) but does not exist yet.

### D. Recommended sequencing for the implementation plan

Given §A–§C, the safe ordering for the "redirect-URI sync" work across both repos is:

1. **SPI PR** (new `PUT /realms/{realm}/bodhi/resources/redirect-uris` endpoint, §2.3 above) merges to `keycloak-bodhi-ext` `main`. This alone changes nothing live (§A) — `main`'s build.yml only compiles/tests.
2. **Cut a release**: run `make release-server` (or push a `release/vX.Y.Z` tag manually) to build+push the image and retag `:latest`. Confirm (directly, in the Railway dashboard, not from either repo) whether Image Auto Updates is enabled for this service; if not, manually click "Deploy" on both the `main` and (separately-managed, per §C) `test-id` Keycloak services.
3. **Verify live** before touching BodhiApp: `curl` (or an `httpyac` script alongside the existing `resource-management.http` blocks, §2.4) the new endpoint against both `https://main-id.getbodhi.app` and `https://test-id.getbodhi.app` directly, confirming a `200`/expected shape rather than `404`.
4. **Only then** push the BodhiApp commit(s) that add the `AuthService::update_redirect_uris` live test coverage (§3.3) — because `build-multiplatform.yml`'s live Rust tests and `build.yml`/`playwright.yml`'s Playwright tests fire on that same push (or on the next push to `main`, since this repo commits straight to trunk), with no gap to defer verification into. The 404-degrade path (§5) is the safety net for *production* SPI-version skew across independent deployments, not a substitute for this ordering — a `server_app` test that specifically asserts the new endpoint's `200` behavior has no legitimate reason to see a `404` if steps 1-3 were done in order, and should treat one as a real ordering bug, not something to design around.
5. If BodhiApp-side development needs to happen *before* step 2 (e.g. writing/exercising the `update_redirect_uris` client code against a real server rather than a mock), do it locally against `make dev.up`'s `localhost:8080` (§B) — write the code and unit/mock-level tests, but hold the new **live** integration test (the one hitting `INTEG_TEST_AUTH_URL`/`INTEG_TEST_MAIN_AUTH_URL`) out of the commit until step 3 has been confirmed, since there is no CI-side way to point it at the local instance instead.

### Sources

- `keycloak-bodhi-ext` repo: `railway.toml`, `.github/workflows/build.yml`, `.github/workflows/release.yml`, `Makefile`, `docker-compose.yml`, `docker/docker-compose.latest.yml`, `ai-docs/03-context/07-release-process.md`, `ai-docs/03-context/08-railway-deployment.md`, `realm-import-files/README.md`, `httpyac-scripts/README.md` — files cited inline by path:line above. `ai-docs/03-context/*` are the project's own (possibly aspirational/slightly-stale — see §A/§C caveats) descriptions, cross-checked against the actual workflow YAML rather than trusted verbatim.
- BodhiApp repo (this repo): `.github/workflows/build-multiplatform.yml`, `.github/workflows/build.yml`, `.github/workflows/playwright.yml`, `crates/server_app/tests/utils/live_server_utils.rs`, `crates/server_app/tests/test_live_multi_tenant.rs`, `crates/routes_app/tests/test_live_multi_tenant.rs`, `crates/routes_app/tests/test_live_auth_middleware.rs`, `crates/lib_bodhiserver/tests-js/.env.test.example`, `crates/server_app/tests/resources/.env.test.example`, `crates/routes_app/tests/resources/.env.test.example`, `crates/lib_bodhiserver_napi/src/test_utils/config.rs`, `docs/archive/claude-plans/202602/20260215-e2e-cleanup/20260215-server-app-remove-register-resource.md` — files cited inline by path:line above.
- `gh variable list` and `gh api repos/BodhiSearch/BodhiApp/environments`, run live against this repo, 2026-09-15 — actual current values of `INTEG_TEST_AUTH_URL`/`INTEG_TEST_MAIN_AUTH_URL` and confirmation of no GH-Environment scoping.
- Railway Docs, "Image Auto Updates" — auto-redeploy behavior for image-sourced services tracking a mutable tag like `:latest`: https://docs.railway.com/deployments/image-auto-updates
- Railway Docs / Guides, "Preview Deployments with PR Environments" — PR Environments replicate services from a connected GitHub repo build, not applicable as-is to a registry-image-sourced service: https://docs.railway.com/guides/preview-deployments-with-pr-environments
- This repo's own `CLAUDE.md` — "Git Workflow (Trunk-Based Development)" section, confirming direct-to-`main` commits with no PRs, which is why §D treats merge and CI-run as the same event.
