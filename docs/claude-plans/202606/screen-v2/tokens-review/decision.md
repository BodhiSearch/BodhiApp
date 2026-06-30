# Tokens Review — Final Decisions

These are the **final, authoritative dispositions** for the highlighted findings in the ultracode
re-review (`index.md` + the per-layer reports, diff `4dea5ea9..HEAD`). They are the input for the
fix/implementation plan. **Where a decision below conflicts with a finding's original
recommendation in `index.md`, this file wins.** Findings not listed here stand as reported.

## Disposition at a glance
| Ref | Disposition | One-liner |
|-----|-------------|-----------|
| F1 | **Fix** | `/v1/embeddings` must enforce the same access checks as `/v1/chat/completions`. |
| F7 | **Fix (redesign)** | Request flag `models_access: bool` defaults **true** (consent shows model selector unless app opts out); `ModelGrant` default becomes empty-deny. |
| F45 + TokenGrantsV1 | **Fix (rename)** | Domain-first field names: `list_models`→`models_list`, `list_mcps`→`mcps_list`. |
| F13 | **Accept (deliberate design)** | `models_access`/`mcps_access` as `bool` in request → `ModelGrant`/`McpGrant` in approved, **same name**, is intentional. |
| F14 | **Accept (no change)** | `mcps` (`McpGrant` in token vs `Vec<McpApproval>` in approved) is acceptable, not confusing. |
| F29 | **Won't fix** | Cache-eviction compact-JSON needle: existing code comment is sufficient. |
| F32 | **Partial** | Reject the "sensitive endpoint under apps-prefix" concern; **now**: drop `/apps` qualifier on the list endpoint; **defer** the module relocation + full path normalization to techdebt. |
| F6 | **No-op** | Already handled previously. |

---

## F1 — `/v1/embeddings` grant bypass (CRITICAL) — **FIX**
`/v1/embeddings` must apply the **same access checks as `/v1/chat/completions`** — i.e. call
`auth_scope.access_policy().ensure_model_inference(&model)?` before forwarding upstream, with unit
tests for ApiToken + ExternalApp restricted cases. (Treat embeddings as a first-class inference
surface everywhere enforcement is applied.)

---

## F45 + TokenGrantsV1 field rename — **FIX (domain-first naming)**
Rename `TokenGrantsV1` fields to the domain-first `<domain>_<qualifier>` convention used by the app
envelopes. `models`/`mcps` (the grant objects) keep their names; only the listing toggles are
renamed.

```rust
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, ToSchema, Default)]
pub struct TokenGrantsV1 {
  #[serde(default)]
  pub models_list: bool,   // was: list_models
  #[serde(default)]
  pub models: ModelGrant,
  #[serde(default)]
  pub mcps_list: bool,     // was: list_mcps
  #[serde(default)]
  pub mcps: McpGrant,
}
```

Propagate the rename through: `openapi.json`, `ts-client`, frontend (`TokenForm` payload assembly,
`useTokens` hooks, detail-rail readers), fixtures, and MSW handlers. This collapses the
verb-first/noun-first split flagged by F45 (and the related cross-cutting naming finding) into one
convention. **This convention (`<domain>_list`, `<domain>` / `<domain>_access` for the grant) is to
be followed elsewhere too.**

---

## F13 / `models_access` bool→grant under the same name — **ACCEPT (deliberate design)**
`models_access` (and `mcps_access`) is intentionally a `bool` in `RequestedResourcesV1` and a
`ModelGrant`/`McpGrant` in `ApprovedResourcesV1`, **under the same field name**. This is a
deliberate API design choice — **not** a bug to rename:
- A request of `models_access: bool` becomes `models_access: ModelGrant` after owner approval.
- It avoids field-name explosion and **creates a deliberate link between the request and its effect
  in the response** (the requested capability and the granted resource share one name).

This is a **known, deliberate design of API field names and should be followed elsewhere as well.**
(Supersedes the F13 "rename the request booleans" recommendation in `index.md`.)

---

## F7 — owner must be able to restrict models — **FIX (redesign)**
The original concern ("apps self-select restrictability; owner can't clamp") is a
**miscommunication** of the intended design. The fix:

**1. Request flag defaults to shown.** In `RequestedResourcesV1`, `models_access: bool` defaults to
`true`:
```rust
pub struct RequestedResourcesV1 {
  // ...
  #[serde(default = "<default-true>")]   // default true
  pub models_access: bool,
  // ...
}
```
Consent behavior:
- `models_access` **absent or `true`** → render the model-access selection component (owner picks
  allowed models).
- An app that does **not** need model access must **explicitly set `models_access = false`**.

**2. `ModelGrant` defaults to empty-deny** (symmetric with `McpGrant`):
```rust
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, ToSchema)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ModelGrant {
  All,
  Specific { ids: Vec<String> },
}

impl Default for ModelGrant {
  fn default() -> ModelGrant { ModelGrant::Specific { ids: vec![] } }   // empty = deny
}
```
(Remove the `#[default] All`; provide the manual `Default` impl above.)

**The only difference from MCP**: for models, the request flag defaults to **shown** (`true`), so
the model-access selector appears by default. The actual grant default is empty-deny, same as MCP.

---

## F14 — `mcps` name overlap — **ACCEPT (no change)**
`mcps` being `McpGrant` in `TokenGrantsV1` vs `Vec<McpApproval>` in `ApprovedResourcesV1` is
reviewed and **OK with the current design — not as confusing** as flagged. No change.

---

## F29 — cache eviction needle (compact-JSON dependency) — **WON'T FIX**
Known and accepted; the **existing code comment is sufficient.** No added test/index required.

---

## F32 — access-requests endpoint placement — **PARTIAL**
- **Reject** the "sensitive session-only endpoint sitting in the apps namespace" concern.
  `/bodhi/v1/apps/*` is a **reserved path prefix for endpoints accessed directly by (primarily)
  3rd-party apps**, so a session-only endpoint living there is not a problem.
- **Accept (in scope now):** the list endpoint does not need the `/apps` qualifier. Use
  **`/bodhi/v1/access-requests/`** to return the list of access-requests for the current user.
  (`access-requests/apps` was historical disambiguation from *user* access-requests, which now live
  under `/users/*`, so the `/apps` suffix is redundant.)
- **Defer to techdebt** (would expand scope): relocating the handlers from
  `routes_app/src/apps/` to a `routes_app/src/access_requests/` module and normalizing the full
  endpoint path set. Logged in `docs/claude-plans/techdebt.md`. The target shape is:
  ```
  ENDPOINT_ACCESS_REQUESTS_REVIEW   = "/bodhi/v1/access-requests/{id}/review"
  ENDPOINT_ACCESS_REQUESTS_APPROVE  = "/bodhi/v1/access-requests/{id}/approve"
  ENDPOINT_ACCESS_REQUESTS_DENY     = "/bodhi/v1/access-requests/{id}/deny"
  ENDPOINT_ACCESS_REQUESTS_APPS     = "/bodhi/v1/access-requests/apps"
  ENDPOINT_ACCESS_REQUESTS_REVOKE   = "/bodhi/v1/access-requests/{id}/revoke"
  ```
  (Note: when the deferred work lands, reconcile `ENDPOINT_ACCESS_REQUESTS_APPS` with the in-scope
  decision above to drop `/apps` for the user's list endpoint.)

---

## F6 — unbound ExternalApp → Unrestricted — **NO-OP**
**Handled previously.** No further action from this review.

---

## Notes for the implementation plan
- The **F7 `ModelGrant` default = empty-deny** change also hardens the F26 display path: a
  `unwrap_or_default()` on a corrupt/missing grant now yields deny rather than all-access for the
  model dimension. Keep this in mind when sequencing.
- The **F45/TokenGrantsV1 rename** and any **F32 list-endpoint rename** both require
  `cargo run --package xtask openapi && make build.ts-client` and frontend/fixture/MSW updates.
- All non-overridden findings (F2, F5, F9, F12, F16–F19, F24, F25, X1, X2, X3, and the remaining
  nice-to-haves) stand as written in `index.md` and the per-layer reports.
</content>
