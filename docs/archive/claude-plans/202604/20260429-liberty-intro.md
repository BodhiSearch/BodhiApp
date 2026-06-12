# Plan: Add `LLM Liberty OAuth` API Model Format (v1: Anthropic provider)

## Context

BodhiApp currently exposes an `anthropic_oauth` API format that takes a single Bearer token (`sk-ant-oat01-…`) the user pastes from the `claude setup-token` CLI. The token is treated as long-lived; there is no refresh path, no expires_at, and the only token slot is `api_model_aliases.encrypted_api_key`.

Two new realities motivate this plan:

1. **Sibling project `llm-liberty`** (`/Users/amir36/Documents/workspace/src/github.com/BodhiSearch/llm-liberty`) is now the supported way for users to acquire *real* OAuth credentials. It runs the upstream OAuth flow against the provider's real authorize URL and emits a versioned JSON envelope (`{ version, provider, access_token, refresh_token, expires_at, auth, oauth, api, headers, body, extra? }`) to clipboard/stdout. Users will paste this JSON into BodhiApp.
2. **Anthropic OAuth access tokens are short-lived (~8h)**. Refresh is required: `POST oauth.token_url` with `{ grant_type: "refresh_token", client_id, refresh_token }`, returning a rotated `(access_token, refresh_token, expires_in)`.

The existing `anthropic_oauth` format keeps its current behaviour and database schema, but its UI label is renamed to **"Anthropic Setup Token"** to reflect that it accepts the long-lived setup token from `claude setup-token`. A **new** `ApiFormat::LlmLibertyOauth` is introduced for the dynamic envelope flow.

**Scope decisions (locked):**

- v1 supports `provider: "anthropic"` only. The other 4 llm-liberty providers (`openai-codex`, `google-gemini`, `google-antigravity`, `github-copilot`) are out of scope; their providers diverge in body shape and refresh protocol and need their own phases.
- llm-liberty has been modified to embed `client_id` (and optional `client_secret`) into `oauth.{client_id,client_secret}`. We consume them from the envelope rather than hard-coding in Rust.
- Both formats coexist: `ApiFormat::AnthropicOAuth` (renamed in UI to "Anthropic Setup Token") and `ApiFormat::LlmLibertyOauth` are independently configurable. Both can route through `/anthropic/v1/messages`.
- Edit UX: re-paste replaces credentials atomically; alias-scope fields (prefix, forward_all_with_prefix, models) remain individually editable.
- No user data exists for the new format; new DDL is purely additive — no destructive migration of `api_model_aliases`. Refresh runs **reactively** on each outbound request (per-alias mutex) — no background scheduler.

## Architectural Approach

### 1. Schema: new sibling table `api_model_oauth_credentials`

Generic across future llm-liberty providers (per user direction "instead of anthropic specific, we need api model oauth credentials… check those providers and their contracts/json so we can design table that is generic and adaptive enough"). Mirrors the `mcps/mcp_oauth_token_entity.rs` pattern of one ciphertext per secret slot.

New migration: `crates/services/src/db/sea_migrations/m20250101_000021_api_model_oauth_credentials.rs`

```
api_model_oauth_credentials
├── api_alias_id          TEXT PK, FK → api_model_aliases(id) ON DELETE CASCADE
├── tenant_id             TEXT NOT NULL                  -- RLS scope
├── user_id               TEXT NOT NULL                  -- RLS scope
├── envelope_version      TEXT NOT NULL                  -- e.g. "1.0.0"
├── provider              TEXT NOT NULL                  -- "anthropic" (others later)
├── encrypted_access_token  TEXT NOT NULL                -- AES-GCM ciphertext
├── access_salt           TEXT NOT NULL
├── access_nonce          TEXT NOT NULL
├── encrypted_refresh_token TEXT NOT NULL
├── refresh_salt          TEXT NOT NULL
├── refresh_nonce         TEXT NOT NULL
├── expires_at            TIMESTAMPTZ NOT NULL
├── auth_in               TEXT NOT NULL                  -- "header" | "query"
├── auth_key              TEXT NOT NULL                  -- e.g. "Authorization"
├── auth_scheme           TEXT NOT NULL                  -- e.g. "Bearer"
├── oauth_authorize_url   TEXT NOT NULL
├── oauth_token_url       TEXT NOT NULL                  -- used by refresh
├── oauth_revoke_url      TEXT NULL
├── oauth_client_id       TEXT NOT NULL                  -- now in envelope; public installed-app id
├── oauth_client_secret   TEXT NULL                      -- only present for Google providers; stored plaintext (known public secrets per llm-liberty docs)
├── api_base_url          TEXT NOT NULL
├── api_chat_url          TEXT NOT NULL
├── api_models_url        TEXT NULL                      -- null for Code Assist providers (future)
├── headers_json          JSONB NOT NULL DEFAULT '{}'
├── body_json             JSONB NOT NULL DEFAULT '{}'
├── extra_json            JSONB NULL
├── created_at            TIMESTAMPTZ NOT NULL
└── updated_at            TIMESTAMPTZ NOT NULL

INDEX idx_api_model_oauth_credentials_provider ON api_model_oauth_credentials(provider);
INDEX idx_api_model_oauth_credentials_expires_at ON api_model_oauth_credentials(expires_at);

POLICY tenant_isolation ON api_model_oauth_credentials
  USING (tenant_id = current_setting('app.current_tenant_id', true));
```

The corresponding `api_model_aliases` row keeps its alias-scope fields (`base_url`, `models`, `prefix`, `forward_all_with_prefix`, `models_cache`, `cache_fetched_at`) and sets `api_format = 'llm_liberty_oauth'`. For LlmLibertyOauth rows, `encrypted_api_key`, `salt`, `nonce`, `extra_headers`, `extra_body` are NULL — secrets and request shape live exclusively in the new table.

### 2. ApiFormat enum + DTOs

`crates/services/src/models/model_objs.rs`:

- Add `LlmLibertyOauth` variant to `ApiFormat` (serde value `"llm_liberty_oauth"`).
- New struct `LlmLibertyEnvelope` mirrors the JSON shape verbatim with serde validation; lives next to `AnthropicModel` in `crates/services/src/models/llm_liberty_envelope.rs`.
- New request/response DTOs:
  - `LlmLibertyEnvelopeUpdate` — tagged enum `{action: keep}` or `{action: set, value: LlmLibertyEnvelope}`, parallels `ApiKeyUpdate`.
  - Extend `ApiModelRequest` with optional `llm_liberty_envelope: Option<LlmLibertyEnvelopeUpdate>`. Mutually exclusive with `api_key` at the request level (validated server-side based on `api_format`).
  - `ApiAliasResponse` adds optional `llm_liberty: Option<LlmLibertySummary>` ({ provider, expires_at, has_refresh_token: bool }) for read-back; never returns ciphertext.

### 3. Provider trait + per-provider implementations

New module `crates/services/src/ai_apis/llm_liberty/`:

```
llm_liberty/
├── mod.rs                     -- provider trait, dispatch by envelope.provider
├── envelope.rs                -- LlmLibertyEnvelope parsing/validation
├── refresh.rs                 -- per-alias mutex registry (DashMap<alias_id, Arc<Mutex>>)
└── anthropic.rs               -- Anthropic-specific implementation
```

`LlmLibertyProvider` trait (per provider impl):

```rust
trait LlmLibertyProvider {
    fn provider_id() -> &'static str;
    fn validate(envelope: &LlmLibertyEnvelope) -> Result<()>;
    async fn refresh(envelope: &StoredEnvelope) -> Result<RefreshedTokens>;
    async fn fetch_models(client: &Client, envelope: &StoredEnvelope) -> Result<Vec<AnthropicModel>>;
    async fn test_prompt(client: &Client, envelope: &StoredEnvelope, req: &TestPromptRequest) -> Result<TestPromptResponse>;
    async fn forward(client: &Client, envelope: &StoredEnvelope, builder: RequestBuilder) -> Result<Response>;
}
```

`anthropic.rs` reuses 80%+ of `crates/services/src/ai_apis/provider_anthropic_oauth.rs:36-200` logic — Bearer auth construction, `anthropic-version` injection, `extra_body` merge for `/messages`, model listing pagination — but reads everything from the dynamic envelope rather than the static alias `extra_headers`/`extra_body` columns.

### 4. Reactive refresh path

Single new function in `crates/services/src/ai_apis/llm_liberty/refresh.rs`:

```rust
pub async fn ensure_fresh_access_token(
    db: &dyn DbService,
    encryption: &EncryptionKey,
    tenant: &str, user: &str, alias_id: &str,
) -> Result<String /* fresh access_token */>
```

Algorithm:
1. Acquire `Arc<Mutex>` keyed by `alias_id` (DashMap<String, Arc<Mutex>>); awaits if another request is mid-refresh.
2. Read `api_model_oauth_credentials` row; decrypt `access_token`, `refresh_token`, optional `client_secret`.
3. If `expires_at - now() > 60s`, return decrypted access_token unchanged (release mutex).
4. Otherwise dispatch to provider impl's `refresh()` which performs the protocol-specific HTTP call.
5. Re-encrypt new tokens with fresh salt+nonce, `UPDATE api_model_oauth_credentials SET ...` atomically, return new access_token.

This wraps `routes_app/src/providers/mod.rs:5` (`resolve_api_key_for_alias`) with a new `resolve_oauth_credentials_for_alias` that returns a `ResolvedCredentials { access_token, headers, body, api_chat_url, ... }` struct. Callers in `ai_api_service.rs` at lines 123, 166, 214 add a third match arm for `ApiFormat::LlmLibertyOauth` that calls the resolver and dispatches to the provider impl.

### 5. Repository

New `crates/services/src/models/llm_liberty_credentials_repository.rs` mirrors `crates/services/src/mcps/mcp_oauth_token_entity.rs:17-80` and `crates/services/src/models/api_alias_repository.rs:76-372`:

- `create(api_alias_id, tenant, user, envelope) -> Result<()>` — encrypts the 2 token secrets (access_token, refresh_token) with per-row salts+nonces; stores `client_secret` as plaintext (known public installed-app secret per llm-liberty docs); inserts row.
- `update(api_alias_id, envelope) -> Result<()>` — re-encrypts tokens and replaces (atomic re-paste semantics).
- `update_tokens(api_alias_id, new_access, new_refresh, new_expires_at) -> Result<()>` — refresh path; only touches token columns, leaves headers/body/api/oauth alone.
- `read(api_alias_id) -> Result<StoredEnvelope>` — decrypts on demand.
- `delete(api_alias_id) -> Result<()>` — explicit; FK CASCADE handles the alias-delete case.

The `DbService` trait gains these as methods so it's mockable in `services` and `routes_app` tests.

### 6. Routes (no new public endpoints in v1)

The existing CRUD endpoints (`POST/PUT/GET/DELETE /bodhi/v1/api-models`) handle `llm_liberty_oauth` via `ApiModelRequest`'s new `llm_liberty_envelope` field. `crates/routes_app/src/models/api/routes_api_models.rs` create/update handlers route to a new `LlmLibertyApiModelService::create/update` when `api_format == LlmLibertyOauth`.

`/anthropic/v1/messages`, `/anthropic/v1/models`, `/anthropic/v1/models/{id}` proxy in `crates/routes_app/src/anthropic/routes_anthropic.rs:60-194` extend the format match: `Anthropic | AnthropicOAuth | LlmLibertyOauth` (the LlmLibertyOauth branch additionally validates `provider == "anthropic"` on the stored envelope).

`POST /bodhi/v1/api-models/test` and `/fetch-models` accept `LlmLibertyEnvelope` directly (not yet persisted) so the user can test the credentials before saving. Validation lives in `LlmLibertyApiModelService`.

### 7. Frontend

`crates/bodhi/src/components/api-models/providers/constants.ts`:

- Existing entry `id: 'anthropic-oauth'` — rename `name` to `"Anthropic Setup Token"`, update `description` to mention `claude setup-token` CLI command. Format string `'anthropic_oauth'` is unchanged (DB enum stays the same).
- New entry `id: 'llm-liberty-oauth'`, `format: 'llm_liberty_oauth'`, `name: "LLM Liberty OAuth"`, baseUrl placeholder, description with the `npx @bodhiapp/llm-liberty@latest login` instructions.

`crates/bodhi/src/components/api-models/`:

- New `form/LlmLibertyEnvelopeInput.tsx` — single textarea + parse + zod validation; replaces `ApiKeyInput` and `ExtrasSection` for the `llm_liberty_oauth` format. Show parsed summary (provider, expires_at, has refresh_token) below the textarea after successful parse. Hide it for other formats.
- `ApiModelForm.tsx:126-131` — conditional render: when `api_format === 'llm_liberty_oauth'`, show LlmLibertyEnvelopeInput + ModelSelectionSection + PrefixInput + ForwardModeSelector. Hide BaseUrlInput, ApiKeyInput, ExtrasSection (the envelope provides them).
- Edit form: same textarea + alias-scope fields editable. Re-pasting valid JSON sets `llm_liberty_envelope: { action: 'set', value: <parsed> }`; leaving textarea empty sets `{ action: 'keep' }`.

`crates/bodhi/src/schemas/apiModel.ts`:

- New zod schema `llmLibertyEnvelopeSchema` validating: `version === '1.0.0'`, `provider === 'anthropic'` (v1 only), required fields (`access_token`, `refresh_token`, `expires_at` is integer, `auth.{in,key,scheme}`, `oauth.{authorize_url,token_url,client_id}`, `api.{base_url,chat_url}`).
- `createApiModelSchema`/`updateApiModelSchema` add a discriminated branch on `api_format === 'llm_liberty_oauth'` requiring the envelope.
- `convertFormToCreateRequest`/`convertFormToUpdateRequest` produce `llm_liberty_envelope: { action: 'set', value }` instead of `api_key`.

`crates/bodhi/src/hooks/api-models/` already exposes the generic CRUD hooks (`useCreateApiModel`, `useUpdateApiModel`); no new hook is needed since the envelope rides on the existing request DTO.

### 8. ts-client / OpenAPI regeneration

After Rust changes: `cargo run --package xtask openapi && cd ts-client && npm run generate && make build.ts-client`. The generated `ApiFormat` union in `ts-client/src/types/types.gen.ts:172` gains `'llm_liberty_oauth'`, and the new `LlmLibertyEnvelope`, `LlmLibertyEnvelopeUpdate`, `LlmLibertySummary` types appear automatically via utoipa schema annotations.

## Critical files to modify

### Rust (services)

| File | Change |
|---|---|
| `crates/services/src/db/sea_migrations/m20250101_000021_api_model_oauth_credentials.rs` | NEW migration |
| `crates/services/src/db/sea_migrations/mod.rs` | register migration |
| `crates/services/src/models/model_objs.rs:771` | add `LlmLibertyOauth` variant |
| `crates/services/src/models/model_objs.rs:1144-1191` | extend `ApiModelRequest` with `llm_liberty_envelope` field |
| `crates/services/src/models/model_objs.rs:1517-1536` | extend `ApiAliasResponse` with `llm_liberty` summary |
| `crates/services/src/models/llm_liberty_envelope.rs` | NEW envelope types + serde |
| `crates/services/src/models/llm_liberty_credentials_entity.rs` | NEW SeaORM entity |
| `crates/services/src/models/llm_liberty_credentials_repository.rs` | NEW repository |
| `crates/services/src/db/service.rs` | extend `DbService` trait + default impl |
| `crates/services/src/ai_apis/llm_liberty/mod.rs` | NEW module: trait + dispatch |
| `crates/services/src/ai_apis/llm_liberty/anthropic.rs` | NEW: Anthropic provider impl (auth, refresh, fetch_models, test_prompt, forward) |
| `crates/services/src/ai_apis/llm_liberty/refresh.rs` | NEW: per-alias mutex + ensure_fresh_access_token |
| `crates/services/src/ai_apis/ai_api_service.rs:123,166,214` | add `LlmLibertyOauth` arm to test_prompt, fetch_models, forward_request_with_method |
| `crates/services/src/ai_apis/mod.rs` | re-export |

### Rust (routes_app)

| File | Change |
|---|---|
| `crates/routes_app/src/providers/mod.rs:5-23` | add `resolve_oauth_credentials_for_alias` returning ResolvedCredentials |
| `crates/routes_app/src/anthropic/routes_anthropic.rs:60-194` | extend ApiFormat match to include `LlmLibertyOauth` (validate provider=='anthropic') |
| `crates/routes_app/src/models/api/routes_api_models.rs:73-330` | dispatch create/update to new LlmLibertyApiModelService when format==LlmLibertyOauth |
| `crates/routes_app/src/models/api/llm_liberty_service.rs` | NEW: Service for create/update/test/fetch-models specific to envelope |
| `crates/routes_app/src/shared/openapi.rs` | register new schemas |

### Frontend

| File | Change |
|---|---|
| `crates/bodhi/src/components/api-models/providers/constants.ts:38-48` | rename `Anthropic OAuth` label to `Anthropic Setup Token`; add new `LLM Liberty OAuth` entry |
| `crates/bodhi/src/components/api-models/form/LlmLibertyEnvelopeInput.tsx` | NEW |
| `crates/bodhi/src/components/api-models/ApiModelForm.tsx` | conditional render based on api_format |
| `crates/bodhi/src/schemas/apiModel.ts` | new envelope schema; discriminated branch in create/update schemas; converters |
| `crates/bodhi/src/test-utils/msw-v2/handlers/api-models.ts` | MSW handlers for the new format |

## Alternatives considered (and why rejected)

1. **Single-blob encrypted column on `api_model_aliases`** instead of sibling table. Rejected — refresh would require decrypt/parse/re-encrypt of the entire envelope on every reactive refresh, and the user-facing `ApiAliasResponse` would either need to expose the blob or re-decrypt for read paths. The sibling table mirrors the established `mcp_oauth_tokens` pattern (`crates/services/src/mcps/mcp_oauth_token_entity.rs:17-80`), which the project's own memory (`feedback_oauth_provider_pattern.md`) flags as the canonical OAuth-storage pattern.
2. **Hard-coded `client_id`/`client_secret` in Rust** mirroring llm-liberty's hex-encoded constants. Rejected per user direction — llm-liberty has been modified to embed them in the envelope, so we consume them dynamically.
3. **Replace `anthropic_oauth` in place** rather than adding a new format. Rejected per user direction — the setup-token (`claude setup-token`) flow has different operational quirks (long-lived, no refresh) than the `llm-liberty login anthropic` OAuth flow, so they coexist.
4. **Background refresh scheduler**. Rejected per user direction — reactive on-demand with per-alias mutex is sufficient. Multi-node refresh has a known race documented in `feedback_oauth_provider_pattern.md`; we accept it for now.

## Layered implementation order

Per `CLAUDE.md` upstream-first methodology:

1. **services/** — migration, entity, repository, envelope types, provider trait, Anthropic impl, refresh logic, ai_api_service dispatch. Run `cargo test -p services` after each step.
2. **routes_app/** — DTO extensions, anthropic proxy match, CRUD dispatch, new test/fetch-models endpoints support. Run `cargo test -p routes_app`.
3. **server_app/** — should not need code changes; run `make test.backend` to validate end-to-end.
4. **ts-client/** — `cargo run --package xtask openapi && cd ts-client && npm run generate && make build.ts-client`.
5. **bodhi/** (frontend) — new envelope input component, schema validation, form conditional rendering, providers catalog rename + new entry. Run `cd crates/bodhi && npm run test`.
6. **lib_bodhiserver/tests-js/** — Playwright regression for the renamed `Anthropic Setup Token` card only (the live OAuth flow is verified manually — see Tests section). `make build.dev-server && make test.e2e`.
7. **Manual verification** — end-to-end paste/fetch/test/chat/refresh round-trip through the running app (see Tests → Manual verification).

Memory feedback `feedback_layered_refactors.md` and `feedback_run_all_gate_checks.md` apply: commit per phase, never skip gate checks.

## Tests

### Unit (services)

- `crates/services/src/ai_apis/llm_liberty/test_anthropic.rs` (NEW)
  - `parse_envelope_v1_anthropic_success`
  - `parse_envelope_rejects_unknown_provider`
  - `parse_envelope_rejects_wrong_version`
  - `refresh_persists_rotated_tokens` (mockito on `oauth.token_url`)
  - `refresh_recovers_from_5xx_with_retry` (or fails fast — codify expectation)
  - `refresh_skew_window_60s` — token within skew triggers refresh
  - `refresh_concurrent_requests_serialize` — two parallel `ensure_fresh_access_token` calls run refresh exactly once (mutex check)
  - `forward_injects_authorization_bearer_from_envelope`
  - `forward_merges_envelope_body_for_messages`
  - `forward_does_not_merge_for_non_messages_path`
  - `fetch_models_paginates_anthropic_models_url`
  - `test_prompt_uses_envelope_chat_url_and_body`
- `crates/services/src/models/test_llm_liberty_credentials_repository.rs` (NEW)
  - CRUD round-trip with encrypt/decrypt
  - `update_tokens_only_changes_token_columns`
  - RLS isolation between tenants
- `crates/services/src/ai_apis/test_ai_api_provider_matrix.rs` — extend matrix to include `LlmLibertyOauth`.

### Unit (routes_app)

- `crates/routes_app/src/models/api/test_llm_liberty_create.rs` (NEW)
  - 201 on valid envelope create
  - 400 on `version != "1.0.0"`
  - 400 on `provider != "anthropic"`
  - 400 on missing required envelope fields
  - 400 if `api_key` and `llm_liberty_envelope` both supplied
- `crates/routes_app/src/models/api/test_llm_liberty_update.rs` (NEW)
  - re-paste replaces credentials atomically (refresh_token rotates; old fails verification)
  - alias-scope fields (prefix, models) update without re-paste (envelope=`{action:keep}`)
- `crates/routes_app/src/anthropic/test_anthropic_oauth_routing.rs` — extend or duplicate for `LlmLibertyOauth` provider=anthropic alias.
- `crates/routes_app/src/anthropic/test_anthropic_messages.rs` — add an `LlmLibertyOauth` alias case to the parameterized matrix.

### Integration (server_app)

- No live-upstream integration test in v1 — llm-liberty Anthropic tokens are short-lived (~8h) and refresh tokens may be server-revoked when flagged as third-party usage. Storing a long-lived integration-test envelope is impractical; `INTEG_TEST_ANTHROPIC_OAUTH_TOKEN` style fixtures don't translate. Manual verification (below) covers this gap.

### Frontend (component)

- `crates/bodhi/src/components/api-models/LlmLibertyEnvelopeInput.test.tsx` (NEW)
  - parses valid JSON; shows summary
  - rejects malformed JSON; shows error
  - rejects wrong `version` / `provider`
  - shows `expires_at` formatted
- `crates/bodhi/src/components/api-models/ApiModelForm.test.tsx` — extend
  - `llm_liberty_oauth` format hides ApiKeyInput, ExtrasSection, BaseUrlInput
  - `anthropic_oauth` format still shows the legacy fields (regression guard)
  - re-paste in edit mode replaces credentials atomically
- `crates/bodhi/src/components/api-models/providers/constants.test.tsx` (or equivalent) — assert UI label "Anthropic Setup Token" for the existing entry.

### E2E (Playwright) — limited

E2E coverage of the live OAuth flow is **not feasible** in v1: llm-liberty Anthropic access tokens are ~8h-lived and refresh tokens may be server-revoked when flagged as third-party usage, so a long-lived envelope cannot be checked in or stored as a CI secret without constant rotation. Verification of the live path is **manual only** (see below).

What we still cover in Playwright:

- `crates/lib_bodhiserver/tests-js/specs/api-models/api-models-extras.spec.mjs` — add a single regression assertion that the existing `Anthropic OAuth` provider card now reads "Anthropic Setup Token". The legacy paste-token flow remains long-lived and is still E2E-testable as today.

Memory feedback `feedback_blackbox_e2e.md` still applies for the assertion above: UI interaction only.

### Manual verification (mandatory before sign-off — primary verification path for v1)

Per `feedback_functional_testing.md`, after backend + frontend unit gates pass, **manual end-to-end is the v1 acceptance gate** for the LLM Liberty OAuth flow:

1. `make app.run.live` (or `make build.dev-server` + start). 
2. Use the browser to:
   - Run `npx @bodhiapp/llm-liberty@latest login anthropic` locally; copy emitted JSON.
   - Navigate to API Models → New → "LLM Liberty OAuth" provider card.
   - Paste JSON; confirm summary renders (provider, expires_at).
   - Click "Fetch Models"; pick a Haiku.
   - Click "Test Connection"; expect successful round-trip.
   - Save; confirm row appears.
   - Open the saved row; click Edit; change prefix only; save; verify prefix updated.
   - Edit again; paste a new envelope (or wait until token expires and verify refresh works in logs); confirm credentials updated atomically.
   - Use `/anthropic/v1/messages` from the chat UI against the saved alias; verify response.
3. Verify "Anthropic OAuth" provider card now reads "Anthropic Setup Token" and its existing flow is unchanged.
4. Inspect logs (or DB) to confirm refresh-on-expiry runs once per alias under concurrent requests (per-alias mutex working).

## Out of scope (Phase 2+)

- llm-liberty providers `openai-codex`, `google-gemini`, `google-antigravity`, `github-copilot`. Each needs its own provider impl in `services/ai_apis/llm_liberty/` plus likely new BodhiApp proxy routes (Codex Responses, Google Code Assist) — significant scope, separate plan.
- Multi-node refresh coordination — known limitation; a single-leader/distributed-lock approach is deferred.
- Background pre-emptive refresh scheduler — reactive-only is sufficient for v1.
- Token revoke endpoint integration — `oauth.revoke_url` is captured in the schema but not invoked on alias delete in v1.
