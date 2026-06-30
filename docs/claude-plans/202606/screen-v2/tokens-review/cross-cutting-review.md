# Cross-Cutting (API consistency & security) Review

Ultracode re-review (Sonnet workflow) of diff range `4dea5ea9..HEAD` — "tokens screen-v2 migration + App Token grants" effort. Findings below survived adversarial verification (refute-by-default); each carries a verdict (`confirmed` = defect traced in committed code; `plausible` = likely real, severity/reachability not fully confirmed). Review only — no source modified.

## Summary
- Findings in this layer: 4 (Critical: 0, Important: 2, Nice-to-have: 2)

## Findings

### F13: models_access/mcps_access: bool in RequestedResourcesV1 but ModelGrant/McpGrant in ApprovedResourcesV1 under the same field names
- **Priority**: important  ·  **Verdict**: confirmed  ·  **Category**: api-consistency
- **File**: `crates/services/src/app_access_requests/access_request_objs.rs`
- **Location**: RequestedResourcesV1.models_access (line ~234, bool) and ApprovedResourcesV1.models_access (line ~255, ModelGrant); same for mcps_access
- **Issue**: The field names models_access and mcps_access are identical in both structs but hold completely different types: bool (UI-driver flag) in the request vs ModelGrant/McpGrant (actual grant object) in the approved body. This is reflected verbatim in openapi.json and ts-client. Any non-TS client (Python, Rust, curl) that reads requested.models_access and naively copies it to approved.models_access will send true (bool) where a {type:'all'} or {type:'specific',ids:[...]} object is required.
- **Failure scenario**: A Python client reads GET /apps/access-requests/:id/review → requested.models_access = true. Constructs PUT body as {"approved": {"models_access": true, ...}}. Rust backend attempts to deserialize true as a tagged ModelGrant enum → 400/422 deserialization error, silently failing the approval.
- **Recommendation**: Rename the request booleans to break the collision: RequestedResourcesV1.models_access → show_models_access (or want_models_access), mcps_access → show_mcps_access. Update openapi.json, ts-client, frontend, and E2E fixtures. Alternatively version the RequestedResources envelope (V2) if external clients already use the old names.
- **Rationale**: Same field name with different JSON types in sibling schemas that appear together in the same API flow is a classic client trap. The Rust type system and TypeScript prevent it internally, but generated or hand-rolled clients in any other language see only the field name.
- **Evidence**: Git diff 4dea5ea9..HEAD introduces both structs in crates/services/src/app_access_requests/access_request_objs.rs: RequestedResourcesV1.models_access is `pub models_access: bool` (line 240) and RequestedResourcesV1.mcps_access is `pub mcps_access: bool` (line 246), while ApprovedResourcesV1.models_access is `pub models_access: ModelGrant` (line 260) and ApprovedResourcesV1.mcps_access is `pub mcps_access: McpGrant` (line 268). OpenAPI spec confirms: RequestedResourcesV1.models_access = {type: boolean}, ApprovedResourcesV1.models_access = {$ref: ModelGrant} (a oneOf of {type:"all"} or {type:"specific",ids:[...]}). Same collision for mcps_access. Both structs are in the same diff (new code). The failure scenario is exact — submitting {"approved": {"models_access": true}} against the approval endpoint would fail deserialization because the backend expects a ModelGrant tagged enum, not a boolean.
- **Verify notes**: The collision is real and in new code introduced in this diff. The doc comments in both structs acknowledge the mirroring intentionally ("Field names mirror RequestedResourcesV1") but this actually makes the problem worse — the deliberate naming symmetry amplifies the client trap. Priority stays "important": no security or data-loss risk (the 400 error is immediate and loud), but it is a genuine external API design inconsistency that will confuse any non-TypeScript client author reading the field names without carefully checking the schema types.
- **Sources**: api:typechain, prior:cross-cutting-review F2

### F7: Apps self-select whether models can be restricted — owner has no UI to clamp models for an app that did not request the control
- **Priority**: important  ·  **Verdict**: confirmed  ·  **Category**: security
- **File**: `crates/services/src/app_access_requests/access_request_objs.rs`
- **Location**: ApprovedResourcesV1 Default impl ~line 281; consent crates/bodhi/src/routes/apps/access-requests/review/index.tsx lines 270-274 and 333
- **Issue**: ModelGrant::default() = All and ApprovedResourcesV1::default().models_access = All. The consent screen's AI-Models section only renders when the app set models_list || models_access in its request. If the app omits both flags, the approve payload forces models_access: {type:'all'} and the owner has no UI affordance to restrict. The party being restricted unilaterally decides restrictability. MCP already does the right thing (default empty Specific); models is the opposite.
- **Failure scenario**: An over-broad app omits models_list and models_access from its access request. The consent screen renders without the model-restriction UI. The owner approves. The app receives all models — identical to the pre-effort behaviour that the grant system was designed to fix.
- **Recommendation**: Either (a) always render the model-restrict GrantBlock at consent regardless of the app's requested flags (defaulting to All for continuity), giving the owner a ceiling they can lower; or (b) document this as an accepted design decision in docs/architecture/security.md so it is not mistaken for default-deny. Make models_access default symmetric with mcps_access (empty Specific) if the intent is restrictive-by-default.
- **Rationale**: Owner consent should be a hard ceiling, not gated by the requester's self-declared flags. The effort's headline goal ('apps can now be restricted') is only partially met — restriction requires the app to opt in, leaving legacy or poorly-written apps with full model access.
- **Evidence**: All four code paths described in the finding are present in the committed diff:

1. `crates/services/src/grants/grant_objs.rs` line 9: `#[default]` is on `All` — `ModelGrant::default() = All`.

2. `crates/services/src/app_access_requests/access_request_objs.rs` line 281: `models_access: ModelGrant::default()` — `ApprovedResourcesV1::default().models_access = All`.

3. `crates/bodhi/src/routes/apps/access-requests/review/index.tsx` line 333: `{(req.models_list || req.models_access) && (` — the entire AI-Models GrantBlock section (including the restriction controls) is conditional on the app-requested flags being set. When the app omits both, the section is not rendered.

4. `index.tsx` lines 270-274: the approve payload builder unconditionally emits `{ type: 'all' }` in the else branch: `models_access: req.models_access ? (modelMode === 'all' ? { type: 'all' } : { type: 'specific', ids: models }) : { type: 'all' }`.

5. The asymmetry with MCPs is real and explicitly coded differently: `index.tsx` lines 288-292 emit `{ type: 'specific', ids: [] }` in the MCP else branch. The Rust comment at line 265-266 of `access_request_objs.rs` even reads: "Owner-granted MCP instances beyond the by-url requests. Defaults to none (empty `Specific`) — unlike a token's all-access default." The author deliberately restricted MCPs but not models.

6. `docs/architecture/security.md` has no entry for …(truncated)
- **Verify notes**: The design choice is intentional and explicitly commented in the source: "If the app didn't request a model selector, the owner can't restrict — the app keeps all-model access (the pre-grants default)" (index.tsx lines 268-269). This was a deliberate backward-compatibility decision. However, it is not documented in security.md as an accepted risk, and it structurally defeats the owner-ceiling property the grant system was built to provide: any app that omits `models_access: true` from its request descriptor opts out of model restrictions entirely, without the owner having any UI affordance to override. The MCPs path (which correctly defaults to empty Specific) shows the pattern for a restric …
- **Sources**: prior:cross-cutting-review F4, prior:routes_app-review F1, prior:services-review F5

### F14: mcps field name collision: McpGrant in TokenGrantsV1 vs Vec<McpApproval> in ApprovedResourcesV1
- **Priority**: nice-to-have  ·  **Verdict**: plausible  ·  **Category**: api-consistency
- **File**: `crates/services/src/tokens/token_objs.rs`
- **Location**: TokenGrantsV1.mcps (line ~42, McpGrant) vs crates/services/src/app_access_requests/access_request_objs.rs ApprovedResourcesV1.mcps (line ~264, Vec<McpApproval>)
- **Issue**: Both TokenGrantsV1 and ApprovedResourcesV1 expose a JSON field named mcps, but with completely unrelated types: McpGrant ({type:'all'} or {type:'specific',ids:[...]}) in the API-token envelope vs Vec<McpApproval> (array of {url, status, instance?} consent records) in the app-approval envelope. In ts-client this manifests as TokenGrantsV1.mcps?: McpGrant vs ApprovedResourcesV1.mcps?: Array<McpApproval>.
- **Failure scenario**: A developer writes a generic function getMcpGrant(envelope) that accesses envelope.mcps, intending to check whether a grant covers a given MCP id. For a TokenGrantsV1 this yields a McpGrant with an allows(id) check. When accidentally passed an ApprovedResourcesV1, mcps is an array of approval records — mcps.allows is not a function. TypeScript catches this only if both types are in scope; a JS or OpenAPI-generated client silently produces wrong values.
- **Recommendation**: Rename ApprovedResourcesV1.mcps to mcps_by_url (or mcp_approvals) to distinguish the url-consent list from the grant-style mcps field. Update the Rust struct, serde rename attribute, openapi.json, ts-client, consent review handler, and all callers. It is a breaking change to the approve request body shape.
- **Rationale**: The collision surfaced because mcps is the natural shorthand for two different things in the two envelopes. With the domain-first renaming from commit 8844c135 the gap widened: ApprovedResourcesV1.mcps_access now clearly names the extra grant, but the legacy mcps array kept its bare name while TokenGrantsV1.mcps is the main grant field.
- **Evidence**: Both fields confirmed in current source: `TokenGrantsV1.mcps: McpGrant` (token_objs.rs:42) and `ApprovedResourcesV1.mcps: Vec<McpApproval>` (access_request_objs.rs:264). Both pre-dated 4dea5ea9 (verified via `git show 4dea5ea9`). The diff added `ApprovedResourcesV1.mcps_access: McpGrant` (absent at 4dea5ea9 — no output from grep), which is a McpGrant alongside the pre-existing Vec<McpApproval> mcps field, slightly compounding naming confusion. TypeScript generated types confirm the structural collision: `ApprovedResourcesV1.mcps?: Array<McpApproval>` vs `TokenGrantsV1.mcps?: McpGrant` (ts-client/src/types/types.gen.ts lines 348, 1731). Rust doc on TokenGrantsV1 says "Intentionally standalone — NOT shared with the App-access-request envelope; the two may diverge."
- **Verify notes**: The naming collision is real but pre-existing — the diff range did not introduce the TokenGrantsV1.mcps vs ApprovedResourcesV1.mcps collision; it only added mcps_access:McpGrant alongside the existing mcps:Vec<McpApproval> in ApprovedResourcesV1. TypeScript prevents the described failure scenario in correctly-typed code: a function parameterized as `TokenGrantsV1 | ApprovedResourcesV1` would have `mcps` typed as `McpGrant | Array<McpApproval>` and the structural difference catches misuse at compile time. The failure requires deliberately untyped JavaScript or duck-typed generic code, which is contrived for this strict-TypeScript codebase. This is a legitimate API naming inconsistency worth t …
- **Sources**: api:typechain, prior:cross-cutting-review F3

### F45: TokenGrantsV1 uses verb-first field names (list_models, list_mcps, models, mcps) diverging from the domain-first convention applied to app envelopes
- **Priority**: nice-to-have  ·  **Verdict**: plausible  ·  **Category**: api-consistency
- **File**: `crates/services/src/tokens/token_objs.rs`
- **Location**: TokenGrantsV1 struct fields, lines 36-43
- **Issue**: Commit 8844c135 (part of this diff) renamed the app approval fields to domain-first order: models_list, models_access, mcps_list, mcps_access in both RequestedResourcesV1 and ApprovedResourcesV1. TokenGrantsV1 was left with the old pre-rename names (list_models, models, list_mcps, mcps). Both implement ResourceGrants and appear together in the OpenAPI schema. The code comment acknowledges 'intentionally standalone', but the divergence is now on a naming axis (ordering convention) rather than a structural one, making cross-reference confusing in documentation and generated SDKs.
- **Recommendation**: If the rename is intended as a project-wide convention, align TokenGrantsV1 to models_list/models_access/mcps_list/mcps_access and provide a DB migration updating stored JSON in the api_tokens.grants column. If the divergence is intentional and permanent, document it explicitly in the TokenGrantsV1 struct comment with a cross-walk table so future contributors understand the mapping.
- **Rationale**: The inconsistency was introduced as a side-effect of applying the domain-first rename only to the app approval side. Since both schemas appear in the same OpenAPI spec and represent semantically parallel constructs, the naming divergence is a discoverability hazard for SDK and documentation consumers.
- **Evidence**: Confirmed the asymmetry in current source:

`TokenGrantsV1` (crates/services/src/tokens/token_objs.rs, lines 34-43):
- `pub list_models: bool`
- `pub models: ModelGrant`
- `pub list_mcps: bool`
- `pub mcps: McpGrant`

`ApprovedResourcesV1` (crates/services/src/app_access_requests/access_request_objs.rs, lines 256-268):
- `pub models_list: bool`
- `pub models_access: ModelGrant`
- `pub mcps_list: bool`
- `pub mcps_access: McpGrant`

Commit 8844c135 renamed all four fields in `ApprovedResourcesV1` (list_models→models_list, models→models_access, list_mcps→mcps_list, mcps_extra→mcps_access) but left `token_objs.rs` entirely untouched (git show 8844c135 -- crates/services/src/tokens/token_objs.rs produced no diff).

Mitigating evidence: The commit message for 8844c135 explicitly states "TokenGrantsV1 (the separate API-token type) keeps its existing list_models/list_mcps naming." The struct comment at lines 31-32 reads "Intentionally standalone — NOT shared with the App-access-request envelope (`ApprovedResources`); the two may diverge." The divergence is therefore a documented, intentional design decision, not an overlooked side-effect.
- **Verify notes**: The naming asymmetry is real and will be visible in the OpenAPI spec to SDK and documentation consumers — the finding is factually accurate. However, the finding characterizes it as an unintentional "side-effect," whereas the commit message and the struct-level doc comment both explicitly acknowledge the divergence as intentional. This falls into the "documented accepted design decision" category rather than an unnoticed bug. Priority nice-to-have is correct: there is a legitimate discoverability concern for API consumers, but no functional breakage, and a future PR aligning the names would also need a DB migration for any stored TokenGrantsV1 JSON (the migration crate already seeds the old  …
- **Sources**: api:typechain F11, arch:backend F18, prior:cross-cutting-review F1
