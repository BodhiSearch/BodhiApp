# Nice-to-have / deferred hardening

Backlog of non-blocking hardening items captured during feature work. Each is safe to defer; pick up when the related area is next touched.

## Exchange-review UI sugar (granted-vs-new highlighting + role cards)

Captured while implementing the exchange/upgrade preselect flow. The core UX — **pre-populating the review form from the source grant** — is done and verified. These are purely visual/UX enhancements on the review page (`crates/bodhi/src/routes/apps/access-requests/review/index.tsx`), modelled on `design/App-Access-Review.html?mode=upgrade` (source: `design/users/access-request-app.jsx`). Explicitly **out of scope** for the current iteration: no "Existing app requesting more access" / cancel-and-supersede banner (we don't invalidate the old token yet).

- **Granted-vs-new pills.** For fields the source grant already held, show a green `✓ previously granted` pill; for newly-requested items, an amber `new access` / `new` pill. Applies at each layer: the list-all toggles, the All/Specific tier, per-model chips, per-MCP-server rows, and role. Reuse the prototype's `.map-granted-pill` / `.map-new-pill` classes. Derivation: diff the current form state against `reviewData.previous_grant`.
- **Role cards.** Replace the role `<select>` dropdown with the card-based selector used by New API Token (`crates/bodhi/src/routes/tokens/-components/TokenForm.tsx`, `.nt-role-card` / `scope-card-*` testids): User / Power User cards with a radio dot + scope badge. In exchange mode mark the source role as `✓ previously granted` while the elevated (requested) role is selected by default.
- **"Review Permission Upgrade" title** variant when `previous_grant` is present.

## Exchanged-token hardening (token exchange / upgrade flow)

Captured while implementing the **exchange (upgrade) access-token preselect flow** (`docs/claude-plans/202607/next-we-want-to-delegated-church.md`). That iteration only stores `source_access_request_id` and preselects the review form; on approve it mints a fresh grant and leaves the old token working. This item hardens invalidation of the superseded (source) token.

- **Middleware invalidation on first use.** When a 3rd-party OAuth token is parsed **for the first time** (in `handle_external_client_token`), check whether it is an *exchanged* token — i.e. its access request was superseded by an approved upgrade. If so, mark the **source** access request as `Exchanged`, and purge it from the exchange cache using the existing needle-based removal method (`cache_service.remove_entries_containing` via `access_request_cache_needle(id)`, the same path `revoke` uses).
- **New status + rejection.** Add an `Exchanged` value to the `AppAccessRequestStatus` enum + column, and reject any token whose bound access request is `Exchanged` (parallel to how `Revoked` is rejected at the exchange path).
- **Immutable terminal state.** An `Exchanged` request's status must **not** be resettable via Revoke or Approve — guard those transitions so a superseded grant can't be resurrected.

Wiring the source→upgrade link: the upgrade draft already stores `source_access_request_id`; on approve, set the source request's status to `Exchanged` (transactionally with minting the new grant) so the two are never both live.
