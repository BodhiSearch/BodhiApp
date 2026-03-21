# Plan: Migrate Live Tests from server_app to routes_app (Phases 1 & 2)

## Context

Phase 0 (non-streamed chat completions) was successfully implemented. The infrastructure is proven: `build_live_test_router()` creates a `Router` with `DefaultSharedContext` (real llama.cpp), and `tower::oneshot()` exercises the full inference chain without TCP listener or Keycloak dependency.

This plan migrates 7 additional tests across 2 phases:
- **Phase 1**: Streamed chat + thinking/reasoning tests (4 tests)
- **Phase 2**: Tool calling tests — non-streamed & streamed, single & multi-turn (4 tests)

## Files to Modify

### 1. `crates/routes_app/src/routes_oai/tests/mod.rs` — Register new module

Add `mod test_live_utils;`

### 2. NEW: `crates/routes_app/src/routes_oai/tests/test_live_utils.rs` — SSE + tool helpers

Direct copy of 3 functions from `crates/server_app/tests/utils/tool_call.rs`:

- `get_weather_tool() -> Vec<ChatCompletionTools>` — weather tool definition for tool calling tests
- `parse_streaming_tool_calls(response_text: &str) -> (Vec<Value>, String)` — parses SSE lines, accumulates tool calls by index, returns `(tool_calls, finish_reason)`
- `parse_streaming_content(response_text: &str) -> (String, String)` — parses SSE lines, concatenates delta content, returns `(content, finish_reason)`

All three handle `data: [DONE]` filtering internally. No modifications needed — they are pure string/JSON parsers with no server dependency.

### 3. `crates/routes_app/src/routes_oai/tests/test_live_chat.rs` — Add 7 tests

All tests follow the established scaffolding:
```rust
#[rstest] #[awt] #[tokio::test(flavor = "multi_thread")] #[anyhow_trace]
#[serial_test::serial(live)] #[timeout(std::time::Duration::from_secs(300))]
async fn test_name() -> anyhow::Result<()> {
  let (router, app_service, ctx, _temp_home) = build_live_test_router().await?;
  let session_cookie = create_authenticated_session(..., &["resource_user"]).await?;
  // ... test body using session_request_with_body() + router.oneshot() ...
  ctx.stop().await?;
  Ok(())
}
```

## Phase 1 Tests

### 1a. `test_live_chat_completions_streamed`
**Source**: `server_app/tests/test_live_chat_completions_streamed.rs`

- Request: `serde_json::json!` with `"stream": true`, same model/messages as non-streamed
- Response: Use `ResponseTestExt::text()` + `parse_streaming_content()` from `test_live_utils`
- Assertions: content contains "Tuesday", finish_reason is "stop"

**SSE parsing note**: Cannot use `ResponseTestExt::sse<T>()` — it doesn't handle `data: [DONE]` and would fail deserializing `[DONE]` as JSON. Use `.text()` + the ported helpers instead.

### 1b. `test_live_chat_completions_thinking_disabled`
**Source**: `server_app/tests/test_live_thinking_disabled.rs` (test 1)

- Request: `serde_json::json!` with `"chat_template_kwargs": {"enable_thinking": false}`
- Response: Parse as `Value` (not typed struct — `reasoning_content` is not in async_openai types)
- Assertions: content contains "Tuesday", `reasoning_content` absent/null, finish_reason "stop"

### 1c. `test_live_chat_completions_reasoning_format_none`
**Source**: `server_app/tests/test_live_thinking_disabled.rs` (test 2)

- Request: `serde_json::json!` with `"reasoning_format": "none"`
- Assertions: Same as 1b — content contains "Tuesday", `reasoning_content` absent/null

### 1d. `test_live_chat_completions_thinking_enabled_default`
**Source**: `server_app/tests/test_live_thinking_disabled.rs` (test 3)

- Request: `serde_json::json!` with NO thinking parameters (tests default behavior)
- Assertions: content contains "Tuesday", `reasoning_content` IS present and non-empty string, finish_reason "stop"

## Phase 2 Tests

### 2a. `test_live_tool_calling_non_streamed`
**Source**: `server_app/tests/test_live_tool_calling_non_streamed.rs` (test 1)

- Request: `CreateChatCompletionRequestArgs` builder with `.tools(get_weather_tool())`, `.stream(false)`, developer + user messages
- Response: Parse as `Value`
- Assertions: finish_reason "tool_calls", function name "get_current_temperature", arguments contain "london", non-empty tool call ID

### 2b. `test_live_tool_calling_multi_turn_non_streamed`
**Source**: `server_app/tests/test_live_tool_calling_non_streamed.rs` (test 2)

- **Turn 1**: Same as 2a, uses `router.clone().oneshot(req1)` (clone because oneshot consumes the service)
- Extract tool call details from Turn 1 response
- **Turn 2**: Build messages with original + assistant message (with tool_calls) + tool result `{"temperature": 15, "unit": "celsius"}`, send via `router.oneshot(req2)`
- Turn 2 assertions: finish_reason "stop", content mentions temperature info

**Router cloning**: `Router` implements `Clone`. Pattern `router.clone().oneshot()` is well-established in `auth_middleware` tests.

### 2c. `test_live_tool_calling_streamed`
**Source**: `server_app/tests/test_live_tool_calling_streamed.rs` (test 1)

- Same as 2a but with `.stream(true)`
- Response: `.text()` + `parse_streaming_tool_calls()`
- Assertions: Same as 2a but on parsed streaming data

### 2d. `test_live_tool_calling_multi_turn_streamed`
**Source**: `server_app/tests/test_live_tool_calling_streamed.rs` (test 2)

- **Turn 1**: Streaming tool call via `router.clone().oneshot()`, parse with `parse_streaming_tool_calls()`
- **Turn 2**: Streaming final answer via `router.oneshot()`, parse with `parse_streaming_content()`
- Assertions: Turn 1 finish_reason "tool_calls", Turn 2 finish_reason "stop" with temperature content

## Key Design Decisions

1. **All tests in single file** (`test_live_chat.rs`): Keeps live tests together, matches existing pattern
2. **Helpers in `test_live_utils.rs`**: Separates reusable parsing logic from test functions
3. **`Value` over typed structs**: All new tests use `serde_json::Value` for response parsing (reasoning_content not in async_openai, tool call inspection is easier on Value, matches server_app originals)
4. **No new dependencies**: Everything needed is already in `routes_app/Cargo.toml`

## Implementation Order (One-at-a-Time, Verify Before Proceeding)

### Step 0: Create shared infrastructure
1. Create `test_live_utils.rs` with all 3 helper functions
2. Register `mod test_live_utils` in `tests/mod.rs`
3. `cargo check -p routes_app` — verify compilation

### Step 1: `test_live_chat_completions_streamed` (1a)
1. Add test to `test_live_chat.rs`
2. `cargo check -p routes_app`
3. `cargo test -p routes_app test_live_chat_completions_streamed` — must pass
4. Proceed to next step only after pass

### Step 2: `test_live_chat_completions_thinking_disabled` (1b)
1. Add test to `test_live_chat.rs`
2. `cargo check -p routes_app`
3. `cargo test -p routes_app test_live_chat_completions_thinking_disabled` — must pass
4. Proceed to next step only after pass

### Step 3: `test_live_chat_completions_reasoning_format_none` (1c)
1. Add test to `test_live_chat.rs`
2. `cargo check -p routes_app`
3. `cargo test -p routes_app test_live_chat_completions_reasoning_format_none` — must pass
4. Proceed to next step only after pass

### Step 4: `test_live_chat_completions_thinking_enabled_default` (1d)
1. Add test to `test_live_chat.rs`
2. `cargo check -p routes_app`
3. `cargo test -p routes_app test_live_chat_completions_thinking_enabled_default` — must pass
4. Proceed to next step only after pass

### Step 5: `test_live_tool_calling_non_streamed` (2a)
1. Add test to `test_live_chat.rs`
2. `cargo check -p routes_app`
3. `cargo test -p routes_app test_live_tool_calling_non_streamed` — must pass
4. Proceed to next step only after pass

### Step 6: `test_live_tool_calling_multi_turn_non_streamed` (2b)
1. Add test to `test_live_chat.rs`
2. `cargo check -p routes_app`
3. `cargo test -p routes_app test_live_tool_calling_multi_turn_non_streamed` — must pass
4. Proceed to next step only after pass

### Step 7: `test_live_tool_calling_streamed` (2c)
1. Add test to `test_live_chat.rs`
2. `cargo check -p routes_app`
3. `cargo test -p routes_app test_live_tool_calling_streamed` — must pass
4. Proceed to next step only after pass

### Step 8: `test_live_tool_calling_multi_turn_streamed` (2d)
1. Add test to `test_live_chat.rs`
2. `cargo check -p routes_app`
3. `cargo test -p routes_app test_live_tool_calling_multi_turn_streamed` — must pass
4. Proceed to next step only after pass

### Step 9: Final validation
1. `cargo test -p routes_app test_live` — all 8 live tests pass serially
