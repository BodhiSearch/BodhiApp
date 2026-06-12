# Plan: Fix Remaining Responses API Code Review Findings

## Context
A code review of the OpenAI Responses API proxy + chat UI migration (commits `8b7c285d..ba3af9b4`) produced 30 findings. 8 were fixed in ba3af9b4. This plan addresses the remaining 22 open items — 13 as code changes, 5 as documentation, 4 deferred.

## Deferred Items (out of scope)
- **U2 (lazy-load chatDB)**: IndexedDB is client-side, MAX_CHATS=100. Perf optimization for a future PR.
- **U4 (OpenAPI typed schemas)**: async-openai types cause utoipa stack overflow. Needs research.
- **SC2 (duplicated wildcard match in shared_rw)**: Cosmetic, low value.
- **SC3 (default trait method for forward_remote_with_params)**: Depends on instance field, can't be default.

---

## Phase 1: services crate

### 1a. S2 — Type-safe HTTP method (`&str` → `http::Method`)
`http` is already a workspace dep. Changes:

- **`crates/services/src/inference/inference_service.rs`**: Change `LlmEndpoint::http_method()` return from `&'static str` to `http::Method`. Return `Method::GET`, `Method::DELETE`, `Method::POST`.
- **`crates/services/src/ai_apis/ai_api_service.rs`**: Change `forward_request_with_method` trait signature: `method: &str` → `method: http::Method`. Update `DefaultAiApiService` impl match to use `Method::GET`, `Method::DELETE`, `_` → POST. Note: `forward_request` calls it with `Method::POST`.
- **`crates/services/Cargo.toml`**: Add `http = { workspace = true }` if not already present.

### 1b. S3 — Add `test_prompt` test for `OpenAIResponses` format
- **`crates/services/src/ai_apis/test_ai_api_service.rs`**: Add test `test_test_prompt_openai_responses_success`:
  - Mock `POST /responses` returning `{"output":[{"type":"message","content":[{"type":"text","text":"Hello response"}]}]}`
  - Call `test_prompt(key, url, model, "Hello", &ApiFormat::OpenAIResponses)`
  - Assert result is `"Hello response"`

### 1c. S4 — Make `forward_request` a default trait method
- **`crates/services/src/ai_apis/ai_api_service.rs`**: Move the body of `forward_request` into the trait as a provided method (it just delegates to `forward_request_with_method(Method::POST, ...)`). Remove from `DefaultAiApiService` impl. mockall's `automock` still generates a mock that overrides it — existing test expectations work.

### 1d. S6 — Add generic `request(method, url)` to SafeReqwest
- **`crates/services/src/shared_objs/safe_reqwest.rs`**: Add:
  ```rust
  pub fn request(&self, method: http::Method, url: &str) -> Result<reqwest::RequestBuilder, UrlValidationError> {
    validate_outbound_url(url, self.allow_private_ips)?;
    Ok(self.inner.request(method, url))
  }
  ```
  Then refactor `forward_request_with_method` in ai_api_service.rs to use `self.client.request(method.clone(), &url)?` with a separate `.header("Content-Type", "application/json")` for POST only.

**Gate**: `cargo test -p services --lib 2>&1 | grep -E "test result|FAILED"`

---

## Phase 2: server_core crate

### 2a. S2 ripple — Update `proxy_to_remote`
- **`crates/server_core/src/standalone_inference.rs`**: `proxy_to_remote` calls `endpoint.http_method()` which now returns `http::Method`. Change `if method == "POST"` to `if method == http::Method::POST`.
- **`crates/server_core/src/multitenant_inference.rs`**: No direct changes (delegates to `proxy_to_remote`).

**Gate**: `cargo test -p server_core --lib 2>&1 | grep -E "test result|FAILED"`

---

## Phase 3: routes_app crate

### 3a. R2 — Validate `response_id` path parameter
- **`crates/routes_app/src/oai/routes_oai_responses.rs`**: Add validation function:
  ```rust
  fn validate_response_id(id: &str) -> Result<(), OAIRouteError> {
    if id.is_empty() || id.len() > 256 || !id.chars().all(|c| c.is_ascii_alphanumeric() || c == '_' || c == '-') {
      return Err(OAIRouteError::InvalidRequest("Invalid response_id format.".into()));
    }
    Ok(())
  }
  ```
  Call at the top of `responses_get_handler`, `responses_delete_handler`, `responses_input_items_handler`, `responses_cancel_handler`.

### 3b. R3 — Cancel handler: send `{}` instead of `null`
- **`crates/routes_app/src/oai/routes_oai_responses.rs`**: In `responses_cancel_handler`, change `serde_json::Value::Null` to `serde_json::json!({})`. Same for `responses_delete_handler` (for consistency, even though DELETE body is discarded).

### 3c. R5 — Extract shared API key resolution helper
- **`crates/routes_app/src/oai/routes_oai_responses.rs`**: Extract `resolve_api_key_for_alias` as a `pub(super)` function (or keep in this file since it's only used here + chat handler):
  ```rust
  pub(super) async fn resolve_api_key_for_alias(auth_scope: &AuthScope, api_alias_id: &str) -> Option<String> {
    let tenant_id = auth_scope.tenant_id().unwrap_or("").to_string();
    let user_id = auth_scope.auth_context().user_id().unwrap_or("").to_string();
    auth_scope.db_service().get_api_key_for_alias(&tenant_id, &user_id, api_alias_id).await.ok().flatten()
  }
  ```
  Use in `resolve_responses_alias` and refactor `chat_completions_handler` + `embeddings_handler` in `routes_oai_chat.rs` to use the same helper.

### 3d. R6 — Add query param forwarding + response_id validation tests
- **`crates/routes_app/src/oai/test_oai_responses.rs`**: Add 4 tests:
  1. `test_responses_get_forwards_extra_query_params` — send `?model=resp/gpt-4o&include[]=output&limit=10`, verify `forward_remote_with_params` receives params without `model`
  2. `test_responses_input_items_forwards_extra_query_params` — same for input_items
  3. `test_responses_get_invalid_response_id` — send `response_id=../../evil`, expect 400
  4. `test_responses_cancel_invalid_response_id` — same for cancel

**Gate**: `cargo test -p routes_app --lib 2>&1 | grep -E "test result|FAILED"`

---

## Phase 4: Full backend gate
`make test.backend 2>&1 | grep -E "test result|FAILED"`

---

## Phase 5: ts-client check
```
cargo run --package xtask openapi
make ci.ts-client-check
```
No API shape changes, so this should be a no-op verification.

---

## Phase 6: UI fixes

### 6a. U1 — JSON.parse shape validation in history restore
- **`crates/bodhi/src/hooks/chat/useBodhiAgent.ts`**: In the try block of history restoration, after `JSON.parse(m.content)`, add shape check:
  ```typescript
  const parsed = JSON.parse(m.content);
  if (parsed && typeof parsed === 'object' && 'role' in parsed) {
    return parsed as AgentMessage;
  }
  // Invalid shape — treat as text
  throw new Error('invalid shape');  // falls to catch block
  ```

### 6b. U5 — Wire stop button during streaming
- **`crates/bodhi/src/routes/chat/-components/ChatUI.tsx`**: In `ChatInput`, when `streamLoading` is true, show a stop button instead of the send button. Pass `stop` function from `useBodhiAgent` through to `ChatInput` as a prop. Add `data-testid="stop-button"`.

### 6c. U6 — Scope localStorage key to userId
- **`crates/bodhi/src/hooks/chat/useChatDb.tsx`**: Change from fixed `'current-chat-id'` key to `\`current-chat-id:${userId}\``. Update the `useState` initializer and `setCurrentChatId` to use the scoped key. Note: `userId` is available as prop to `ChatDBProvider`.

### 6d. R7 — Remove debug console logging
- **`crates/lib_bodhiserver_napi/tests-js/specs/chat/chat.spec.mjs`**: Remove the `page.on('console', ...)` block in `test.beforeEach`.

**Gate**: `cd crates/bodhi && npm test 2>&1 | tail -10`

---

## Phase 7: Documentation

### Doc comments (inline)
- **S1**: Add doc comment to `LlmEndpoint`: `/// Note: Clone (not Copy) because response ID variants contain owned String.`
- **R4**: Update utoipa descriptions for GET/DELETE/cancel endpoints: add note that `model` query parameter is required for multi-provider routing (not part of upstream OpenAI API).
- **U3**: No migration from pre-pi-agent-core chat storage — document as intentional. Chat history is local IndexedDB, not server-side.
- **U7**: Add `// TODO: Remove shim when ChatMessage renders AgentMessage directly` at `agentMessageToLegacy`.

### CLAUDE.md updates (U8)
- **`crates/services/CLAUDE.md`**: Add `ApiFormat::OpenAIResponses` variant, `SafeReqwest::request()` method, `http::Method` usage in `AiApiService`.
- **`crates/routes_app/CLAUDE.md`**: Add Responses API route group, `response_id` validation pattern, `model` query param requirement.
- **`crates/server_core/CLAUDE.md`**: Note `http::Method` in `proxy_to_remote`.
- **`crates/bodhi/src/CLAUDE.md`**: Document pi-agent-core migration, Dexie/IndexedDB storage, `useBodhiAgent` hook, `useMcpAgentTools` adapter.

---

## Phase 8: E2E gate
```
make build.ui-rebuild
cd crates/lib_bodhiserver_napi && npm run test:playwright
```

---

## Verification Summary
| Phase | Command | What it validates |
|-------|---------|-------------------|
| 1 | `cargo test -p services --lib` | Method type change, new test, trait default |
| 2 | `cargo test -p server_core --lib` | proxy_to_remote update |
| 3 | `cargo test -p routes_app --lib` | Validation, dedup, new tests |
| 4 | `make test.backend` | Full backend integration |
| 5 | `make ci.ts-client-check` | OpenAPI/TS sync |
| 6 | `cd crates/bodhi && npm test` | UI fixes |
| 8 | `npm run test:playwright` | E2E validation |
