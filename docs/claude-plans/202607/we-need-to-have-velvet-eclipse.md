# Per-resource `access` flag for `/bodhi/v1/mcps` and `/bodhi/v1/models`

## Context

API tokens and 3rd-party OAuth app JWTs can **list** resources they cannot **invoke**.
The token-grants model (locked design in `docs/claude-plans/202606/20260628-token-api-app.md`)
made *listable* deliberately broader than *invokable*: a resource is listable when the
"list all" toggle (`mcps_list` / `models_list`) is on **OR** the resource is individually
granted. So with the list toggle on, a token sees MCPs/models it has no connect/inference
grant for — with no signal in the payload distinguishing the two.

This plan adds an `access: boolean` to each listed resource that answers exactly one
question: *can this principal invoke this specific resource?* It mirrors the existing
enforcement predicate (`ensure_mcp_connect` / `ensure_model_inference`), so the list
payload now agrees with what the request path would allow. No production data exists;
this is a clean contract-breaking change with no migration.

**Semantics of `access`** (narrower than the existing listable filter):
- Session / Anonymous principals (`AccessPolicy::Unrestricted`) → `access = true` for all.
- Unbound external app (`AccessPolicy::Deny`) → n/a (its listings are already empty).
- Grant-bound principal (`AccessPolicy::Grants`) → `access = allows_*_connect/inference(id)`.

**Listing behavior is unchanged** (per the "Keep filter, annotate within" decision):
the existing `*_listable` filter still hides non-listable resources; `access` only
annotates *within* the visible set. When the list toggle is on, all show with `access`
true/false — which is the reported problem. When the toggle is off, only granted
resources show (all `access = true`).

## Design: reuse the existing policy seam

`AccessPolicy` (`crates/routes_app/src/shared/token_grants.rs`) already has the invoke
guards (`ensure_mcp_connect`, `ensure_model_inference`) and the visibility predicates
(`mcp_listable`, `model_listable`). Add two thin boolean wrappers mirroring the
`*_listable` ones but delegating to the underlying `ResourceGrants` **invoke** methods
(`allows_mcp_connect` / `allows_model_inference`, which already exist on the trait):

```rust
pub fn mcp_accessible(&self, mcp_id: &str) -> bool {
  match self { Unrestricted => true, Deny => false, Grants(g) => g.allows_mcp_connect(mcp_id) }
}
pub fn model_accessible(&self, model_id: &str) -> bool {
  match self { Unrestricted => true, Deny => false, Grants(g) => g.allows_model_inference(model_id) }
}
```

No new domain logic — `ResourceGrants` (`crates/services/src/grants/grant_objs.rs`) is the
single source of truth for both grant envelopes (`TokenGrantsV1`, `ApprovedResourcesV1`).

The model-id **key** must match `Alias::retain_listable_models`
(`crates/services/src/models/model_objs.rs:1007`): API aliases key per-model on
`format!("{prefix}{model.id()}")`; User/Model/Router aliases key on the alias name.

## Backend changes

### 1. `AccessPolicy` — add the two accessors
`crates/routes_app/src/shared/token_grants.rs`: add `mcp_accessible` and `model_accessible`
next to the existing `*_listable` methods.

### 2. MCP DTO + handler
- `crates/services/src/mcps/mcp_objs.rs`: add `pub access: bool` to `Mcp`. In
  `From<McpWithServerEntity> for Mcp`, default `access: true` (owner/create context is a
  session; the listing handler overrides it).
- `crates/routes_app/src/mcps/routes_mcps.rs`:
  - `mcps_index`: keep the `mcp_listable` filter; after `.into()`, stamp
    `m.access = policy.mcp_accessible(&m.id)`.
  - `mcps_show`: after the listable 404 check, stamp `access = policy.mcp_accessible(&id)`
    on the returned `Mcp` (a token with the list toggle on may `show` an MCP it can't invoke).
  - `mcps_create`: owner session → the `true` default is correct; no change needed.
- `apps_mcps_index`/`apps_mcps_show` delegate to the above — covered automatically.

### 3. Models DTOs + handler
`crates/services/src/models/model_objs.rs`:
- New **response-only** type (not stored; `ApiModelVec`/DB stay `Vec<ApiModel>`):
  ```rust
  #[derive(Debug, Clone, Serialize, Deserialize, PartialEq, ToSchema)]
  pub struct ApiModelResponse {
    #[serde(flatten)]
    pub model: ApiModel,   // internally tagged (provider); flatten keeps fields top-level
    pub access: bool,
  }
  ```
  Flatten yields `{ "provider": "...", ...metadata, "access": true }` — the shape the user
  approved (keeps all provider metadata, minimal frontend churn).
- `ApiAliasResponse.models`: change `Vec<ApiModel>` → `Vec<ApiModelResponse>`. In
  `From<ApiAlias>`, wrap each model as `ApiModelResponse { model, access: true }`.
- Add `pub access: bool` to `UserAliasResponse`, `ModelAliasResponse`, `ModelRouterResponse`;
  default `true` in their `From` impls. (`ApiAliasResponse` gets **no** top-level `access` —
  its access is per-model.)
- Add a closure-based stamper on `AliasResponse` (services can't see `AccessPolicy`; use the
  same `impl Fn(&str) -> bool` pattern as `retain_listable_models`):
  ```rust
  pub fn stamp_access(&mut self, accessible: impl Fn(&str) -> bool) {
    match self {
      AliasResponse::User(r) => r.access = accessible(&r.alias),
      AliasResponse::Model(r) => r.access = accessible(&r.alias),
      AliasResponse::ModelRouter(r) => r.access = accessible(&r.alias),
      AliasResponse::Api(r) => {
        let prefix = r.prefix.clone().unwrap_or_default();
        for m in &mut r.models { m.access = accessible(&format!("{}{}", prefix, m.model.id())); }
      }
    }
  }
  ```

`crates/routes_app/src/models/routes_models.rs` (`models_index`): keep the existing
`retain_listable_models(|id| policy.model_listable(id))` prune; after building each
`AliasResponse`, call `resp.stamp_access(|id| policy.model_accessible(id))`.
In `models_show` (returns `UserAliasResponse`), stamp `access` too.

Single-alias CRUD endpoints returning these responses directly (`ApiModelService` →
`ApiAliasResponse`, create/update/show) are owner/session-scoped, so the `true` default is
correct and they need no stamping.

### 4. OpenAPI + TS client
- Register `ApiModelResponse` in `crates/routes_app/src/shared/openapi.rs` schemas list
  (the others are pulled in transitively; add it explicitly to be safe).
- Fix the stale `"models": ["gpt-4", ...]` example in the `models_index`/`ApiAliasResponse`
  `#[utoipa::path]` docs.
- Regenerate: `cargo run --package xtask openapi && make build.ts-client`.
- **Verify** the flattened `ApiModelResponse` schema renders correctly (utoipa + a
  `#[serde(flatten)]` over an internally-tagged enum). If ToSchema mis-generates, fall back
  to a nested `{ access, model }` wrapper for the schema while keeping serde flatten, or add
  an explicit `#[schema(...)]` override.

## Frontend changes
Because `access` is *flattened*, the model objects keep `provider`/`id`/metadata at the top
level, so structural helpers keep working — churn is mostly types + fixtures:
- Type regen from `@bodhiapp/ts-client` (`ApiAliasResponse.models` element gains `access`;
  single-model responses gain top-level `access`).
- Verify these still compile/behave (they read model fields structurally, unaffected by the
  extra key): `crates/bodhi/src/lib/modelAlias.ts`, `crates/bodhi/src/schemas/apiModel.ts`
  (`getApiModelId`, `convertApiToForm`), `crates/bodhi/src/lib/grantItems.ts`,
  `crates/bodhi/src/routes/chat/-components/settings/AliasSelector.tsx`.
- Update fixtures/mocks to include `access`: `crates/bodhi/src/test-fixtures/models.ts`,
  `crates/bodhi/src/test-utils/msw-v2/handlers/models.ts`, `.../api-models.ts`, and any MCP
  fixtures/handlers.
- **Optional UI surface** (not required by this plan): an access badge in
  `crates/bodhi/src/routes/models/-components/ModelDetailRail.tsx` and the MCP list.

## Out of scope
- OpenAI-compat `/v1/models`, Anthropic `/anthropic/v1/models`, Gemini model listings —
  their wire schemas are provider-fixed and the user asked only for `/bodhi/v1/*`.
- MCP `Mcp` gets a top-level `access` (single resource); no per-tool granularity.

## Verification

`access` must be exercised at **every layer**, varying the grant knobs across **both**
grant-bearing principals (API token → `TokenGrantsV1`; app/`ExternalApp` token →
`ApprovedResourcesV1`) plus the session baseline. The knobs are:
`mcps_list` / `mcps` (API) & `mcps` approved-instances + `mcps_access` (app);
`models_list` / `models` (API) & `models_access` (app).

### Grant-combination matrix (the canonical cases — reuse at each layer)

MCP (`R` = resources the user owns; `A`,`B` ∈ `R`):

| # | Principal | list toggle | connect grant | visible | `access` |
|---|-----------|-------------|---------------|---------|----------|
| M0 | Session | n/a | n/a | all `R` | all `true` |
| M1 | API token | `mcps_list=true` | `mcps=All` | all `R` | all `true` |
| M2 | API token | `mcps_list=true` | `mcps=Specific{A}` | all `R` | `A→true`, rest `false` ← **the reported bug case** |
| M3 | API token | `mcps_list=false` | `mcps=Specific{A}` | `{A}` | `A→true` |
| M4 | API token | `mcps_list=true` | `mcps=Specific{}` | all `R` | all `false` |
| M5 | API token | `mcps_list=false` | `mcps=Specific{}` | `{}` (empty) | — |
| M6 | App token | `mcps_list=true` | approved-instance `A`, `mcps_access=Specific{}` | all `R` | `A→true`, rest `false` |
| M7 | App token | `mcps_list=true` | no instance, `mcps_access=Specific{B}` | all `R` | `B→true`, rest `false` |
| M8 | App token | `mcps_list=true` | approved-instance `A` + `mcps_access=Specific{B}` | all `R` | `A,B→true`, rest `false` |
| M9 | App token (unbound, `grants=None` → `Deny`) | — | — | `{}` (empty) | — |

Models: the **same 10-row shape** with `models_list` + `models`(API)/`models_access`(app),
plus the per-model dimension — at least one `ApiAliasResponse` carrying a `prefix` and ≥2
models so a `Specific{prefix+modelId}` grant yields mixed `access` **inside one alias**, and
one each of User/Model/ModelRouter alias to assert the **top-level** `access` (keyed on alias
name). Cover `models=All` (all `true`) and the empty-grant (all `false`) extremes too.

### Applying the matrix per layer (upstream→downstream)

**Rust unit — `services` (`cargo test -p services`)**
- Serialize `ApiModelResponse` → assert flattened shape (`provider`/`id`/`access` all
  top-level, `access` present). Serialize `Mcp` + User/Model/ModelRouter responses → assert
  top-level `access`. Unit-test `AliasResponse::stamp_access` directly with a stub closure
  over the M0–M9 keying (alias-name vs `prefix+id`).

**Rust unit — `routes_app` (`cargo test -p routes_app`)** — the primary coverage
- `test_token_grants`: unit-test the new `mcp_accessible` / `model_accessible` across
  `Unrestricted` / `Deny` / `Grants(TokenGrantsV1)` / `Grants(ApprovedResourcesV1)`, covering
  every matrix row (including the app-token instance-vs-`mcps_access` split, M6–M8).
- `test_mcps` / `test_models`: drive `mcps_index` / `models_index` (and `*_show`) via
  `tower::oneshot` once per matrix row using the `AuthContext` test factories
  (`test_session`, `test_api_token`, `test_external_app` in
  `services::test_utils::auth_context`), asserting both the **visible set** and the **`access`
  value per resource** — explicitly asserting `access=false` on M2/M4/M6/M7 and per-model
  mixed access inside the prefixed `ApiAliasResponse`.

**server_app integration (`cargo test -p server_app`, `#[serial(live)]`)**
- Real HTTP round-trip proving the field survives real serialization + the token-service
  grant-parsing path. Mint a real API token and construct a real approved app grant, then hit
  `/bodhi/v1/mcps`, `/bodhi/v1/apps/mcps`, and `/bodhi/v1/models` for the representative rows
  (at minimum M2, M6/M7, and their model equivalents), asserting the `access` booleans match
  the grant.

**Frontend (`cd crates/bodhi && npm test`)**
- Update fixtures/MSW to carry `access` (both variants: mixed true/false). Assert components
  consuming `models`/`mcps` still render, and (if the badge is added) that they reflect
  `access`.

**E2E (`make test.e2e`) — verify `access` is correctly computed end-to-end**
- Black-box, through the **real external-app + API-token product surface** (the actual
  consumer of this field), not `page.evaluate`. Extend the existing token-grants E2E flow:
  1. As a session user (UI), create an API token and/or approve an app access request whose
     grants match a matrix row that yields **mixed** access — e.g. `list=true` +
     `Specific{one-of-two}` (M2) so exactly one resource is `access:true` and one is
     `access:false`.
  2. As that scoped principal, list `/bodhi/v1/(apps/)mcps` and `/bodhi/v1/models` and assert
     `access:true` on the granted resource **and** `access:false` on the non-granted one, and
     that per-model access inside a prefixed API alias is mixed as expected.
  3. Cover both principal types (API token and app/`ExternalApp` token) since their grant
     envelopes differ (`mcps_access` vs approved-instance path).
  - Follow the loud-failure rule: throw in `beforeAll` if required env/tokens are missing —
    never `test.skip`. No if/else branching; wait for deterministic ready-state.
  - If a first-party UI access badge is added (see Frontend), additionally assert it via UI
    interactions on the models/MCP screens.

**Backend gate**: `make format` then `make test.backend`. Regenerate types
(`cargo run --package xtask openapi && make build.ts-client`) before frontend work, and
`cargo test -p routes_app -- openapi` to confirm the spec is in sync.
