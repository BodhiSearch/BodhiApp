# Origin-aware `review_url` for App Access Requests

## Context

When an external app calls `POST /bodhi/v1/apps/request-access`, the server returns a
`review_url` that the app opens in the user's browser so they can review/approve the
grant. Today that URL is built from a **static** `frontend_url` string captured once at
service-construction time from `SettingService::public_server_url()`:

```rust
// access_request_service.rs:284
format!("{}/ui/apps/access-requests/review?id={}", self.frontend_url, access_request_id)
```

For a **local install** `public_server_url()` resolves to the bind host `BODHI_HOST`,
which defaults to `0.0.0.0` → the review URL becomes `http://0.0.0.0:1135/ui/...`. That is
browser-hostile and origin-mismatched. A local server is actually reachable at **four**
equivalent hosts — `0.0.0.0`, `127.0.0.1`, `localhost`, and the machine's LAN IP — and the
review link should reflect **the host the request actually arrived on** so the browser opens
a same-origin URL (session cookies match, no dead link).

The setup flow already solves this exact problem for OAuth redirect URIs
(`crates/routes_app/src/setup/routes_setup.rs:133-170`): it branches on
`get_public_host_explicit()`, and in local mode enumerates `LOOPBACK_HOSTS` + the request's
`Host` header + the detected server IP. We mirror that logic for the review URL.

**Outcome**: the review URL reflects the originating host (validated against the server's
known-valid host set), fixing broken local review links while leaving Docker/RunPod/
explicitly-configured deployments on their configured public URL.

### Decisions (confirmed with user)
- **Detection source: HTTP `Host` header only.** No `server_url` request-body field for now
  (so no OpenAPI/ts-client regeneration). Can be layered on later.
- **Fallback: unchanged `public_server_url()`.** When the `Host` header is absent or is
  *not* one of the known-valid hosts, keep today's behavior. No `localhost` substitution.
- Host reflection is **validated** (loopback set + detected server IP, or explicit configured
  host) to avoid Host-header injection / open-redirect into the review link.

## Approach

Move URL resolution off the frozen string and onto `SettingService`, made request-aware.
`DefaultAccessRequestService` stops holding `frontend_url: String` and instead holds
`Arc<dyn SettingService>` + `Arc<dyn NetworkService>` (matching the two data sources the
setup route uses). `build_review_url` becomes `async` and takes the request host.

### 1. `services` — new `SettingService` helper (default trait method)

File: `crates/services/src/settings/setting_service.rs` (trait at :48; add near
`public_server_url` :425). Also relocate `LOOPBACK_HOSTS`.

- **Relocate `LOOPBACK_HOSTS`** from `routes_app/src/setup/routes_setup.rs:11` into `services`
  (add to `crates/services/src/settings/constants.rs`, re-export). Update `routes_setup.rs`
  to import it from `services::` (dedup — one source of truth).
- **Add default method** on `SettingService`:

  ```rust
  /// Base URL to embed in user-facing links (e.g. the access-request review URL),
  /// resolved for the host the request actually arrived on.
  ///
  /// - Explicit public host (Docker/RunPod/configured `BODHI_PUBLIC_HOST`) always wins.
  /// - Local/network install: reflect `request_host` only when it is a known-valid host
  ///   (a `LOOPBACK_HOSTS` entry or the detected `server_ip`) — guards against Host-header
  ///   injection.
  /// - Otherwise fall back to `public_server_url()` (unchanged behavior).
  async fn resolve_public_server_url(
    &self,
    request_host: Option<&str>,
    server_ip: Option<&str>,
  ) -> String {
    if self.get_public_host_explicit().await.is_some() {
      return self.public_server_url().await;
    }
    if let Some(host) = request_host {
      let valid = LOOPBACK_HOSTS.contains(&host) || server_ip == Some(host);
      if valid {
        let scheme = self.public_scheme().await;
        let port = self.public_port().await;
        return match (scheme.as_str(), port) {
          ("http", 80) | ("https", 443) => format!("{}://{}", scheme, host),
          _ => format!("{}://{}:{}", scheme, host, port),
        };
      }
    }
    self.public_server_url().await
  }
  ```

  Reuses existing `get_public_host_explicit()`, `public_scheme()`, `public_port()`,
  `public_server_url()`. Default method → real impls (`DefaultSettingService`,
  `SettingServiceStub`) inherit it for free.

### 2. `services` — request-aware `AccessRequestService`

File: `crates/services/src/app_access_requests/access_request_service.rs`

- **Struct/constructor** (:74-95): replace `frontend_url: String` with
  `setting_service: Arc<dyn SettingService>` and `network_service: Arc<dyn NetworkService>`.
  Update `DefaultAccessRequestService::new(...)` signature accordingly.
- **Trait method** (:68): change to
  `async fn build_review_url(&self, request_host: Option<&str>, access_request_id: &str) -> String;`
  (trait already `#[async_trait]`).
- **Impl** (:284): resolve base URL, then append the review path:
  ```rust
  async fn build_review_url(&self, request_host: Option<&str>, access_request_id: &str) -> String {
    let server_ip = self.network_service.get_server_ip();
    let base = self
      .setting_service
      .resolve_public_server_url(request_host, server_ip.as_deref())
      .await;
    format!("{}/ui/apps/access-requests/review?id={}", base, access_request_id)
  }
  ```
- `AccessRequestService` is `#[mockall::automock]` → `MockAccessRequestService` regenerates
  with the new async signature automatically; update any `expect_build_review_url` call sites.

### 3. `routes_app` — pass the request host from the handler

File: `crates/routes_app/src/apps/routes_apps.rs` (`apps_create_access_request` :52).

- Add `headers: axum::http::HeaderMap` to the handler signature.
- Derive host via the existing helper: `let request_host = crate::shared::utils::extract_request_host(&headers);`
- Await the new signature: `let review_url = access_request_service.build_review_url(request_host.as_deref(), &created.id).await;`
- No OpenAPI change (response shape `CreateAccessRequestResponse` unchanged). Axum extracts
  `HeaderMap` freely; keep `ValidatedJson` body extractor last.

### 4. Update construction sites

- **Production**: `crates/lib_bodhiserver/src/app_service_builder.rs:329-342`
  (`build_access_request_service`) — drop the `public_server_url().await` string; pass
  `setting_service.clone()` and the built `network_service` into
  `DefaultAccessRequestService::new(...)`. Both are already available in the builder
  (`build_network_service()` at ~:345). Verify ordering so `network_service` is constructed
  before the access-request service.
- **Tests**:
  - `crates/services/src/app_access_requests/test_access_request_service.rs:30-35` — fixture
    passes a `SettingServiceStub` + `StubNetworkService` instead of the `"http://localhost:1135"`
    string.
  - `crates/routes_app/src/apps/test_access_request.rs:47-52` — same.
  - `crates/server_app/tests/utils/live_server_utils.rs:207,589,976` — these already build a
    `setting_service` and a `StubNetworkService`; pass those in instead of
    `setting_service.public_server_url().await`.

## Tests

### Unit — `SettingService::resolve_public_server_url` (`services`)
Add `test_setting_service.rs` cases (or extend existing settings tests) using
`SettingServiceStub`:
- Local default (`BODHI_HOST=0.0.0.0`, port 1135), `request_host = Some("localhost")` →
  `http://localhost:1135`.
- Same, `request_host = Some("127.0.0.1")` → `http://127.0.0.1:1135`.
- `request_host = Some("192.168.1.5")` with matching `server_ip` → reflected; with a
  *different* `server_ip` (unknown host) → falls back to `public_server_url()`.
- `request_host = None` → `public_server_url()`.
- Explicit public host set (`BODHI_PUBLIC_HOST=example.com`) → always `https://example.com`
  regardless of `request_host` (injection guard).
- Default-port collapse (public scheme https + port 443) omits the port.

### Unit — `build_review_url` (`services`, `test_access_request_service.rs`)
New tests (currently **zero** coverage): with `StubNetworkService{ip:None}` +
`SettingServiceStub`, assert `build_review_url(Some("localhost"), "ar-1")` ==
`http://localhost:1135/ui/apps/access-requests/review?id=ar-1`; and that a valid LAN-IP host
(via `StubNetworkService{ip:Some(...)}`) is reflected.

### Integration — handler (`routes_app`, `test_access_request.rs`)
Extend the create-access-request test: send the request with a `Host: 127.0.0.1:1135`
header and assert the response `review_url` host is `127.0.0.1` (not `0.0.0.0`); a request
with no/unknown Host falls back to the configured public URL.

### Integration — live HTTP (`server_app`)
In the access-request live test (`server_app/tests`), issue the real
`POST /bodhi/v1/apps/request-access` and assert the returned `review_url` reflects the
`Host` the reqwest client used (loopback), exercising the full extractor → service path.

### E2E (Playwright)
Not applicable for host-header variation — the browser sets its own `Host`, and black-box
E2E cannot forge alternate valid hosts. The server_app live test above is the end-to-end
guard. (Flag if a real cross-host E2E scenario is desired later.)

## Verification
1. `cargo test -p services --lib -- app_access_requests settings 2>&1 | grep -E "test result|FAILED"`
2. `cargo test -p routes_app -- apps 2>&1 | grep -E "test result|FAILED"`
3. `cargo test -p server_app 2>&1 | grep -E "test result|FAILED"` (Docker up for PG).
4. `make test.backend` for the full backend gate.
5. Manual: `make app.run.live`, then `curl -si -H 'Host: localhost:1135' -X POST \
   http://localhost:1135/bodhi/v1/apps/request-access -H 'content-type: application/json' \
   -d '{"app_client_id":"x","requested_role":"scope_user_user","requested":{"version":"1","mcp_servers":[]}}'`
   and confirm `review_url` starts with `http://localhost:1135`; repeat with
   `Host: 127.0.0.1:1135` and confirm it reflects `127.0.0.1`.

## Notes / non-goals
- No `CreateAccessRequest` body change → **no** `xtask openapi` / `make build.ts-client` needed.
- `extract_request_host` already strips the port and validates the hostname charset
  (`utils.rs:3-28`); IP hosts and loopback names pass. Reused as-is.
- `build_authorize_endpoint()` is unaffected (delegates to `AuthService::authorize_url()`).
- Keep the existing `info!("... review_url: {}", review_url)` log line.
