# Kickoff prompt: OpenAI Codex OAuth as a first-class `ApiFormat`

> **Status:** Deferred. Archived here as a kick-off prompt for a future work session.
> **Intended use:** Paste this file (or its key sections) into a fresh Claude Code planning session when we are ready to pick this up. It is deliberately exploratory, not prescriptive — the next session should re-examine the referenced commits and existing OAuth infrastructure before committing to a design.

---

## Why this was deferred

We originally scoped a minimal "paste access/refresh tokens from `~/.codex/auth.json`" path. On review, that shortcut pushes too much complexity onto users (rotating tokens, JWT decoding, atomic pair edits) and bypasses the OAuth primitives BodhiApp already runs for user login and MCP servers. A proper implementation needs:

- A real OAuth authorization-code + PKCE flow initiated from the backend (same shape as login/MCP flows), not a paste form.
- A new **temporary credentials table** to stage OAuth tokens obtained mid-flow, linked into the `api_model_aliases` row only on successful create/update.
- Token refresh with per-alias atomic read-modify-write against the encrypted store (single-use refresh tokens — racing is a correctness bug, not just a perf issue).
- Re-shaping of the API-model create/update page around a "Login with provider" button that appears when the selected format requires OAuth, plus UI states for in-progress / success / expired / needs-reconnect.

This touches DB schema, new service(s), new routes, and material frontend state. Out of scope for the current iteration; parked for a dedicated push.

---

## Goal

Add `ApiFormat::OpenAICodex` so a user signed into a ChatGPT Codex subscription can drive BodhiApp chat/inference through their subscription (upstream `https://chatgpt.com/backend-api/codex/responses`). The user experience is: pick **OpenAI (Codex)** in the API-model form → click **Connect with OpenAI** → complete OAuth in a popup/redirect → alias is saved with the short-lived JWT + single-use refresh token → BodhiApp refreshes transparently before expiry.

---

## Anchor commits — study before designing

These are the two squash merges that shaped the Anthropic and Gemini variants. They are the closest prior art for ApiFormat addition, provider-client strategy, route/middleware wiring, OpenAPI regen, frontend presets, and E2E fixtures. Read them end-to-end, including the component commits they bundle.

- **`d064f68f`** — `feat(services): add extra_headers/extra_body fields and AnthropicOAuth variant` (parent squash of the AnthropicOAuth rollout). Notable sub-commits in ancestry:
  - `ef590f80` / `49038f69` — `ApiFormat::Anthropic` + `LlmEndpoint` variants
  - `d3861043` — format-aware auth headers + `test_prompt`/`fetch_models`
  - `3901796c` — `/anthropic/v1/*` pass-through proxy
  - `778b0051` — frontend form integration + ts-client regen
  - `9560f148` / `72b06efd` — server_app live tests + NAPI E2E
  - `933fb595` — chat UI routing via pi-ai anthropic-messages
- **`19ee7e73`** — `feat(ts-client): automate Gemini OpenAPI spec sync and type generation` (parent squash of the Gemini rollout). Includes `ApiFormat::Gemini`, `GeminiModel`, action-style URLs, `gemini_auth_middleware`, `/v1beta/*` proxy, format-mismatch rejection in `/v1/chat/completions`, and the NAPI Gemini chat spec.

Use these to calibrate: how big is a "new ApiFormat" change, what layers does it touch, what tests accompany each layer, how are regressions guarded.

---

## Existing OAuth infrastructure to reuse, not reinvent

BodhiApp already runs OAuth authorization flows in two places. Read these first and design Codex OAuth as a **third instance of the same pattern**, sharing helpers wherever it makes sense:

1. **User login** (Keycloak) — browser-initiated authorization code flow with state + PKCE, callback handler, token refresh on use.
   - `crates/routes_app/src/auth/routes_auth.rs` (initiate + callback)
   - `crates/routes_app/src/auth/test_login_callback.rs` (callback correctness tests)
   - `crates/services/src/auth/auth_service.rs` (refresh_token method, token lifecycle)

2. **MCP server OAuth** (2.1 DCR + pre-registered) — per-MCP-server OAuth clients, state storage during flow, token persistence post-flow, refresh on demand.
   - `crates/services/src/mcps/mcp_oauth_token_entity.rs`
   - `crates/services/src/mcps/mcp_oauth_config_detail_entity.rs`
   - `crates/services/src/mcps/mcp_repository.rs` and `mcp_service.rs`
   - `crates/routes_app/src/mcps/routes_mcps_auth.rs`
   - `crates/routes_app/src/mcps/test_oauth_flow.rs`
   - Squash `76da6257` — `feat: unified MCP auth system with OAuth 2.1 pre-registered and DCR support`

3. **External app OAuth tokens** (`/apps/` prefix) — pattern for third-party app OAuth with scope filtering.
   - Squash `ff24c4df` / `1a9ab921` — `feat: add /apps/ API prefix for external OAuth apps`
   - `crates/services/src/mcps/test_mcp_auth_repository*.rs` (repository isolation patterns)

Codex-specific quirks to slot into this pattern:
- **Client is pre-registered** (OpenAI public client id `app_EMoamEEZ73f0CkXaXp7hrann`, default token endpoint `https://auth.openai.com/oauth/token`, authorize endpoint `https://auth.openai.com/oauth/authorize`). No DCR. Keep those as constants with a config override escape hatch.
- **Headers are non-negotiable** on upstream calls: `Authorization: Bearer`, `chatgpt-account-id` (extracted from JWT claim `https://api.openai.com/auth.chatgpt_account_id`), `originator: codex_cli_rs` (non-whitelisted originator → 403), `OpenAI-Beta: responses=v2`.
- **Refresh token is single-use** — the instant you POST a refresh, the prior `refresh_token` is dead. Two concurrent refreshes = one success, one permanent 401. Per-alias mutex is mandatory; distributed coordination is a known gap on multi-node Docker (document, don't solve in v1).
- **Upstream only exposes the Responses API**, not chat/completions. Rejection on `/v1/chat/completions` mirrors the Gemini pattern; chat UI goes through pi-ai `openai-responses`.

---

## Exploratory questions the next session should answer

Before writing a detailed plan, spawn Explore agents and answer these:

1. **DB shape.** What does a "staged OAuth credentials" table look like? Options:
   - A generic `oauth_flow_state` table (state, code_verifier, created_at, expires_at, initiator_context enum { user_login, mcp, api_model_codex, ... }) to unify all three flows.
   - A Codex-specific `codex_oauth_tokens` table linked by `api_alias_id` (null while staged, populated on alias create/update).
   - Extending the encrypted `api_key` column with a tagged JSON envelope (rejected in prior scoping — too coarse for staged-then-linked flows).
   Find out what MCP OAuth did and whether its abstractions are lift-and-shift.

2. **Flow initiation & callback endpoint.** Where does the callback land (`/bodhi/v1/oauth/codex/callback`? reuse `/apps/` prefix?) and how does it correlate back to the in-progress form session? User login uses server-side session; MCP OAuth uses a server-side state record keyed by a nonce returned to the browser. Pick one pattern.

3. **Form UX.** API-model form today is one save button. With Codex:
   - On format switch to `openai_codex`, api-key input disappears; **Connect with OpenAI** button appears.
   - Clicking initiates OAuth in a popup/redirect; on success the form receives a `staged_credential_id` (opaque handle) to include in the `POST /bodhi/v1/models/api` body.
   - Edit mode: show "Tokens connected · Reconnect" CTA; reconnect re-runs the flow and produces a new `staged_credential_id`. No "keep/set" checkbox — reconnect is explicit and atomic.
   - Error/expiry: when refresh fails (single-use race, revoked), surface a **Reconnect required** banner on the alias row in the models list.
   Decide whether to move the API-model page to Zustand as part of this work or keep react-hook-form (my earlier draft leaned Zustand; reconsider once the flow shape is settled).

4. **Chat UI routing.** Codex aliases should route through pi-ai `openai-responses` with base URL `${origin}/v1`. Verify that the existing `/v1/responses` passthrough proxy (commit `67bd84eb`) can host `openai_codex` by extending its alias-format guard — or whether a dedicated `/codex/*` proxy path is cleaner. Gemini added its own path (`/v1beta/*`); Anthropic reused `/anthropic/v1/*`. Pick based on what the pi-ai SDK expects.

5. **Refresh mechanics.** Where does `ensure_fresh_token` live?
   - In the provider client (inline, per-request, holding a handle to the refresh service)?
   - As a dedicated `CodexTokenService` keyed by alias_id → `Arc<Mutex<CachedTokens>>` with a `DashMap` registry?
   Factor the single-use refresh carefully: the write-back path must be transactional (decrypt → refresh over HTTP → encrypt new blob → atomic UPDATE) and idempotent on retry. Consider whether the refresh service should also debounce refresh calls when multiple requests arrive for the same alias within a short window.

6. **Scope of chat-completions compatibility.** Upstream doesn't support it, but do we want BodhiApp to translate incoming `/v1/chat/completions` → upstream `/responses` for better third-party app compatibility? Default: no, reject with format-mismatch (Gemini pattern). Revisit if a concrete integration needs it.

---

## Suggested phasing (sketch, not contract)

Once the exploratory questions are answered, the build likely looks like:

1. **Staged-credentials table + migration** (SQLite + PostgreSQL; RLS policies mirroring `api_model_aliases`).
2. **`CodexAuthService` in services crate** — initiate flow (generate state + PKCE verifier, persist to staging), exchange code for tokens at callback, refresh tokens with per-alias mutex, extract `chatgpt-account-id` from JWT.
3. **Routes for initiate + callback** — under `/bodhi/v1/models/api/oauth/codex/{initiate,callback}` (or reuse the `/apps/` umbrella if that pattern fits).
4. **`ApiFormat::OpenAICodex` + `OpenAICodexProviderClient`** — parallels `d064f68f`'s `AnthropicOAuthProviderClient` but routes through `CodexAuthService` for token fetches.
5. **Dispatcher wiring in `DefaultAiApiService`** — `test_prompt` / `fetch_models` / `forward` branches.
6. **Route integration** — `/v1/responses` accepts `openai_codex`; `/v1/chat/completions` rejects with format-mismatch; `/v1/models` surfaces codex aliases (regression test per Gemini pattern).
7. **OpenAPI + ts-client regen** — `cargo run --package xtask openapi && make build.ts-client`.
8. **Frontend**: format preset, `ConnectWithOpenAIButton` in the API-model form, popup/redirect flow hooks, staged-credential-id threading into create/update mutations, reconnect UX in edit + list views. Zustand migration can ride along if the state fan-out warrants it.
9. **Chat routing** — `agentStore.ts` maps `openai_codex` → pi-ai `openai-responses` + base URL `${origin}/v1`.
10. **Tests at every layer** (per `feedback_testing_depth`):
    - services unit: mockito-backed refresh happy-path, expired, concurrent, single-use 401, staged-credential cleanup.
    - routes_app: initiate returns redirect URL; callback persists staged creds; create+link purges staging.
    - server_app integration: end-to-end alias create with mocked upstream token endpoint.
    - NAPI E2E (`feedback_blackbox_e2e`, `feedback_no_skip_for_missing_env`): form flow stubs the OAuth redirect (likely a helper that simulates the popup callback), then exercises chat.
11. **Docs** — crate-level CLAUDE.md updates for `services`, `routes_app`, `bodhi`; `ai-docs/func-specs/security/security.md` note on single-node refresh assumption.

---

## Deliberately open

- Exact name of the staged-credentials table and whether it unifies with MCP OAuth state storage.
- Whether to embed the OpenAI client id / auth endpoints as constants or expose as settings (desktop power-user may want to point at a self-hosted gateway).
- Whether to surface `originator`, `chatgpt-account-id`, and `OpenAI-Beta` as user-overridable `extra_headers` (current `AnthropicOAuth` pattern) or keep them server-owned for correctness. Codex is stricter than Anthropic — leans toward server-owned.
- Multi-node refresh correctness. Single-use refresh tokens + naive mutex = broken under horizontal scale. Acceptable v1 limitation, loud note in docs; revisit with a DB advisory lock or a leader-elected refresh worker if/when cluster mode needs it (see `project_cluster_deployment` memory).

---

## When revisiting

1. Re-read this file, then read `d064f68f` and `19ee7e73` in full.
2. Read the MCP OAuth squash `76da6257` to understand the existing staging/refresh pattern.
3. Answer the six exploratory questions above via Explore agents.
4. Only then write a concrete implementation plan.

The prior iteration's prescriptive plan (paste-token approach, api_key JSON envelope, inline mutex) is superseded — do not resurrect it without reconciling against the proper-OAuth design decided above.
