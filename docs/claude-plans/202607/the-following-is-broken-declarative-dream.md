# Fix: MCP OAuth reconnect flow

## Context

Reconnecting an existing OAuth MCP (e.g. Exa) is broken end-to-end. Repro: open
`/ui/mcps/new/?id=<ID>`, Disconnect, Connect → OAuth to exa.ai. The redirect back lands on
`/ui/mcps/new/?id=<ID>/` (a trailing slash appended *after* the `id` value), and pressing
**Update connection** fails with:

```
Item '01kwazda60ggy184yvdzaz5g39' of type 'mcp_oauth_token' not found. (db_error-item_not_found)
```

Investigation found **three compounding defects**. The reported flow needs A+B fixed to work at
all; C is a real adjacent bug in the same code path (a plain rename-and-save of any OAuth MCP hits
the same error) and is the exact source of the pasted error message via the manual-reload workaround.

No OpenAPI/request/response schema changes — so **no ts-client regeneration needed**.

---

## Defect A — trailing slash corrupts the return URL (frontend)

`mcpFormStore.saveToSession` builds `return_url` as a combined string `pathname + window.location.search`
= `/mcps/new/?id=<ID>` (`crates/bodhi/src/stores/mcpFormStore.ts:66-74`). The callback then calls
`navigate({ to: returnUrl })` (`crates/bodhi/src/routes/mcps/oauth/callback/index.tsx:98-99`, and again
on the error "Back to form" button at `:139-149`). With the router's `trailingSlash: 'always'`
(`crates/bodhi/src/main.tsx:13`), the whole string is treated as a path and `/` is appended at the end
→ `/mcps/new/?id=<ID>/`, corrupting `id`.

The login/auth callback avoids this by routing through the existing helper `handleSmartRedirect`
(`crates/bodhi/src/lib/utils.ts:23-61`), which parses the URL and passes pathname + search as
**separate** `navigate({ to, search })` args, so `trailingSlash` only touches the path.

### Fix A

In `crates/bodhi/src/routes/mcps/oauth/callback/index.tsx`, replace both raw
`navigate({ to: returnUrl })` calls with the existing helper:

```ts
import { handleSmartRedirect } from '@/lib/utils';
// success path (~line 98-99):
const returnUrl = formState.return_url || '/mcps/new/';
handleSmartRedirect(returnUrl, navigate);
// error "Back to form" button (~line 139-149): same substitution
```

Reuses `handleSmartRedirect` — no new parsing logic. `return_url` in the store stays as-is (a
`/`-prefixed relative string, which `handleSmartRedirect` already handles).

---

## Defect B — Update deletes the token it is about to link (backend)

On reconnect the callback exchanges the code with `mcp_id = editId`, so `store_oauth_token` mints the
new token **already linked** to the MCP (`mcp_repository.rs` store path deletes existing `(mcp_id,user)`
tokens then inserts the new row with `mcp_id` set). Then `update_mcp_with_auth`
(`crates/services/src/mcps/mcp_repository.rs:1338-1367`) runs `delete_many WHERE McpId == mcp_id`
**before** `find_by_id(token_id)`, deleting the very row it needs → lookup returns `None` →
`ItemNotFound`. This is why fixing only the URL still leaves Update failing.

### Fix B (backend, per decision)

Exclude the token being linked from the sibling-cleanup delete in `update_mcp_with_auth`
(`crates/services/src/mcps/mcp_repository.rs:1339-1343`):

```rust
mcp_oauth_token_entity::Entity::delete_many()
  .filter(mcp_oauth_token_entity::Column::McpId.eq(&mcp_id))
  .filter(mcp_oauth_token_entity::Column::Id.ne(&token_id)) // keep the token being linked
  .exec(txn)
  .await
  .map_err(DbError::from)?;
```

This makes the operation idempotent: stale sibling tokens for the MCP are cleared, the linked token
survives, and re-linking an already-linked token is a no-op-equivalent. The subsequent `find_by_id`
+ `update` (setting `mcp_id`) is unchanged; the `..Default::default()` at `:1357` correctly leaves
other columns untouched (only `mcp_id` is Set — not the SeaORM update-default trap since we intend a
partial update here).

---

## Defect C — edit-load conflates `auth_config_id` with `oauth_token_id` (frontend)

On edit-load of an OAuth MCP, `crates/bodhi/src/routes/mcps/new/index.tsx:250-252` calls
`store.completeOAuthFlow(existingMcp.auth_config_id)`, which sets `oauthTokenId = auth_config_id`
(`stores/mcpFormStore.ts:37-41`) — the auth-config id, not a token id. A plain Update (or the
manual-reload workaround that lost sessionStorage) then sends that id as `oauth_token_id` →
`ItemNotFound`. `McpResponse`/`Mcp` does not expose the real token id, so the frontend was using
`auth_config_id` as a "connected" proxy but wrongly assigning it to the token field.

### Fix C (frontend)

Separate the "is connected" flag from the token id. Add a store action that marks connected without
touching `oauthTokenId`:

- `crates/bodhi/src/stores/mcpFormStore.ts`: add `setConnected: (connected: boolean) => void` that
  sets only `isConnected` (leave `oauthTokenId` untouched).
- `crates/bodhi/src/routes/mcps/new/index.tsx:250-252`: replace
  `store.completeOAuthFlow(existingMcp.auth_config_id)` with `store.setConnected(true)`.

Downstream behavior (already correct once C lands):
- `onSubmit` OAuth branch (`new/index.tsx:456-467`) sends `oauth_token_id: store.oauthTokenId || undefined`.
  Plain edit → `oauthTokenId` is `null` → `undefined` sent → backend skips the token block and
  **keeps the existing link**. Reconnect → `oauthTokenId` is the real new token (set by
  `completeOAuthFlow` from the callback's session restore at `:209-210`) → sent → Fix B links it.
- The submit gate (`:435`) still passes because `isConnected` is `true`.
- `handleDisconnect` (`:414-419`) no longer schedules a bogus delete of `auth_config_id`. Frontend
  no longer needs the real token id for cleanup: on reconnect the exchange clears the old token; on
  switching auth away from OAuth the backend deletes tokens by mcp_id (`mcp_service.rs:768-775`).
  (Known minor gap, out of scope: "disconnect an OAuth MCP but save it as still-OAuth-with-no-token"
  — a nonsensical action — won't force-clear the token.)

---

## Files to modify

- `crates/bodhi/src/routes/mcps/oauth/callback/index.tsx` — Fix A (2 navigate calls → `handleSmartRedirect`)
- `crates/services/src/mcps/mcp_repository.rs` — Fix B (add `Id.ne(token_id)` filter, `update_mcp_with_auth`)
- `crates/bodhi/src/stores/mcpFormStore.ts` — Fix C (add `setConnected`)
- `crates/bodhi/src/routes/mcps/new/index.tsx` — Fix C (edit-load uses `setConnected`)

Reused as-is (no change): `crates/bodhi/src/lib/utils.ts` `handleSmartRedirect`.

---

## Tests (all layers)

**Backend (`crates/services/src/mcps/`, rstest + `#[values("sqlite","postgres")]`):**
- Reconnect/idempotent link: create an OAuth MCP with a token already linked to it, call
  `update_mcp_with_auth` with that same `oauth_token_id` → succeeds, token still linked (no
  `ItemNotFound`). This is the regression test for Defect B.
- Sibling cleanup: two tokens exist for the same `mcp_id`; update linking one deletes the other but
  keeps the linked one.
- Update with `oauth_token_id = None` on an OAuth MCP leaves the existing token untouched.

**Frontend (Vitest + MSW):**
- `crates/bodhi/src/routes/mcps/oauth/callback/index.test.tsx`: after a successful exchange with a
  stored `return_url = '/mcps/new/?id=<ID>'`, assert `navigate` is called with `{ to: '/mcps/new/',
  search: { id: '<ID>' } }` (path/search split, no trailing-slash corruption). Regression for A.
- `crates/bodhi/src/routes/mcps/new/index.test.tsx`: edit-load of an OAuth MCP does NOT set
  `store.oauthTokenId` to `auth_config_id`; a plain Update submits with `oauth_token_id` omitted.
  Regression for C.
- Store test for `setConnected` toggling `isConnected` without altering `oauthTokenId`.

**E2E / manual:** the full exa.ai reconnect requires a live OAuth provider, so cover deterministically
via the unit tests above and verify the end-to-end reconnect manually in Chrome (see Verification).

---

## Verification

1. Backend: `cargo test -p services --lib 2>&1 | grep -E "test result|FAILED"` (Docker up for PG).
   Then `make test.backend`.
2. Frontend: `cd crates/bodhi && npm test` (callback + new-form + store suites).
3. Manual end-to-end (real reconnect), per repo workflow:
   - `make app.run.live` (Rust needs rebuild for Fix B — ensure the live server is the rebuilt binary).
   - In Chrome: open `/ui/mcps/new/?id=<ID>` for an existing Exa OAuth MCP → Disconnect → Connect →
     complete exa.ai OAuth.
   - Assert the redirect returns to a clean `/ui/mcps/new/?id=<ID>` (no trailing slash on `id`).
   - Press **Update connection** → expect "MCP updated successfully", no `ItemNotFound`.
   - Also verify a plain rename+Update of an OAuth MCP (no reconnect) succeeds (Defect C).
4. `make format` before committing.
