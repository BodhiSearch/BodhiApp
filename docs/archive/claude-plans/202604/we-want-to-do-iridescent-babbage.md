# Access Request: allow selecting any configured MCP, not just exact-URL matches

## Context

When a 3rd-party app requests access to an MCP, it sends the MCP **server URL**. Because of AI
gateways/aggregators (Composio, Dev, self-hosted proxies, etc.), the same logical MCP (e.g. Gmail)
can be reached through different URLs. Today the review screen filters the user's configured MCP
instances to **exact URL matches only** — so a user who reaches Gmail through Composio cannot satisfy
a request that names the direct Gmail URL, even though it is the right tool for them.

We want to keep good UX (exact-URL matches surfaced **on top**) while adding flexibility: the dropdown
should also list **all other configured MCP instances**, so the user can connect the request to
whichever instance fits their setup.

**Verified safe:** downstream resolution (when the app actually calls the MCP proxy at
`/bodhi/v1/apps/mcps/{id}/mcp`) authorizes purely by **approved instance ID** — see
`crates/routes_app/src/mcps/routes_mcps.rs:39-71` (filters the user's MCPs by `approved.mcps[*].instance.id`)
and `crates/routes_app/src/mcps/mcp_proxy.rs` (routes to the instance's `server_url`). The requested URL
is **never re-compared after approval**, and the OAuth scope is keyed on the access-request id
(`scope_access_request:<id>`), not the URL. So relaxing the approval-time URL check does not break proxying.

**Decisions confirmed with user:**
- Dropdown is a **plain flat list** — exact-URL matches sorted first, no group titles, no badges.
- **No** auto-selection (user always picks explicitly).
- **No** mismatch warning when a non-matching instance is chosen.

Because the response struct shape (`McpServerReviewInfo { url, instances }`) is unchanged, **no OpenAPI
or ts-client regeneration is required** — only behavior, validation, and copy change.

## Changes

### 1. Backend — review handler returns all instances, exact-match first
`crates/routes_app/src/apps/routes_apps.rs` (`apps_get_access_request_review`, ~191-208)

Replace the exact-URL filter with a stable partition: instances whose `mcp_server.url == mcp_server_req.url`
first, then all remaining user instances. Currently:

```rust
let instances = all_user_mcps
  .iter()
  .filter(|m| m.mcp_server.url == mcp_server_req.url)   // <-- drops non-matching
  .cloned()
  .collect();
```

becomes a partition that keeps every instance, matches first (preserve existing relative order within each
group). `all_user_mcps` is already fetched once above the loop (`mcps_svc.list()`), so each requested URL
reuses it. `McpServerReviewInfo.instances` now means "all selectable instances, best-match first" rather
than "instances connected to this URL".

### 2. Backend — drop the approval-time URL-match restriction
`crates/routes_app/src/apps/routes_apps.rs` (`apps_approve_access_request`, lines 325-330)

Remove the block that rejects an instance whose `server_url != approval.url`
(`AppsRouteError::InvalidMcpType`). **Keep** the two other guards in the same loop:
- ownership: `auth_scope.mcps().get(&instance.id)` returns `None` → `McpInstanceNotOwned`
- enabled: `!mcp_entity.enabled` → `McpInstanceNotConfigured`

After this, any owned + enabled instance can be approved for any requested URL.
(Consider whether `AppsRouteError::InvalidMcpType` becomes unused; if so, remove it from
`crates/routes_app/src/apps/error.rs` — grep first per repo convention before deleting.)

### 3. Frontend — McpServerCard dropdown + empty states
`crates/bodhi/src/routes/apps/access-requests/review/-components/McpServerCard.tsx`

The existing `Select` already renders `validInstances` (enabled) in the order received, so it will now show
the full sorted list with **no structural change** to the dropdown. Update only the empty-state semantics,
since `instances` no longer means "connected to this server":
- `!hasInstances` (user has **zero** configured MCPs): change copy to e.g. *"No MCP instances configured.
  Create one first."* (was "No MCP instances connected to this server."). Keep the `review-no-mcp-instances-*`
  testid.
- `hasInstances && validInstances.length === 0` (has MCPs but all disabled): keep the existing
  "All MCP instances are disabled" alert.

### 4. Frontend — review page wiring (verify, minimal/no change)
`crates/bodhi/src/routes/apps/access-requests/review/index.tsx`

`handleApprove` resolves the chosen instance via `mcp.instances.find(i => i.id === selectedMcpInstances[mcp.url])`
and `canApprove` uses `mcp.instances.filter(i => i.enabled)` — both keep working unchanged because `instances`
is now the full list. Confirm no copy/logic still assumes exact-URL semantics.

## Tests

### Rust — `crates/routes_app/src/apps/test_access_request.rs`
- **Add** a cross-URL approval success test: seed an enabled MCP instance whose `server_url` differs from the
  requested URL, approve with that `instance.id`, assert `200 / Approved`. (Use the same harness +
  `seed_draft_request` pattern as `test_approve_access_request_success` at line 107; add an MCP-seed helper if
  none exists.)
- **Add/extend** a review test asserting `mcps_info[].instances` now includes non-matching instances with the
  exact-URL match ordered first.
- Existing `test_approve_access_request_mcp_instance_not_owned` (line 192) and the enabled check stay valid.
- If `InvalidMcpType` is removed, drop any reference to its error code.
- Run: `cargo test -p routes_app -- apps` (and `cargo test -p routes_app -- openapi` to confirm spec unchanged).

### Frontend — `crates/bodhi/src/routes/apps/access-requests/review/index.test.tsx`
- Update the empty-state assertion at ~line 633 (`/No MCP instances connected/`) to the new copy.
- Add a case where the review fixture returns instances spanning multiple server URLs; assert all appear as
  `review-mcp-instance-option-*` and that an exact-URL match renders before a non-matching one.
- Add a case approving with a **non-matching** instance and assert the approve payload carries that
  `instance.id`/`path`.
- Run: `cd crates/bodhi && npm test -- review`.

### E2E (Playwright) — black-box, optional but recommended
`crates/lib_bodhiserver/tests-js/` (page object `pages/AccessRequestReviewPage.mjs`, spec under
`specs/request-access/` or `specs/mcps/`). If extending: configure two MCP instances (one matching the
requested URL, one not), drive the review UI to pick the **non-matching** instance, approve, and verify the
app can call that instance through the proxy. Pure UI interactions only (no `page.evaluate`/direct fetch),
throw in `beforeAll` if required env/instances are missing. Run: `make test.e2e` from
`crates/lib_bodhiserver/tests-js`.

## Verification

1. `cargo test -p routes_app -- apps openapi` — backend handler + spec-unchanged checks pass.
2. `cd crates/bodhi && npm test -- review` — frontend review tests pass.
3. Manual (`make app.run.live`): create two MCP instances for the same logical tool under different server
   URLs; trigger an access request naming one URL; on the review screen confirm the dropdown lists **both**
   with the exact-URL match on top; approve with the non-matching one; confirm the requesting app can call
   that MCP through the proxy.
4. `make test.e2e` if the E2E case is added.

## Out of scope
- `crates/bodhi/src/routes/chat/-components/McpsPopover.tsx` (chat tool picker) — unrelated overflow fix on
  the `fix/mcp-tools-popover-overflow` branch.
