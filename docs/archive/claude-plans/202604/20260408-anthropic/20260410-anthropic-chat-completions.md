# Enable Anthropic on /v1/chat/completions + Live API Token E2E Tests

## Implementation Status

**All phases shipped in HEAD commit `c9e3568a`** (squashed with the full Anthropic support batch).

| Phase | Description                                                              | Status | Deviation                                      |
| ----- | ------------------------------------------------------------------------ | ------ | ---------------------------------------------- |
| 1     | Backend: remove Anthropic rejection, extend allowlist                    | ✅ Done | None                                           |
| 2     | Fixtures: add `anthropic` to `API_FORMATS`, update `api-models.spec.mjs` | ✅ Done | `beforeAll` uses `throw` (not `test.skip()`)   |
| 3     | New `api-live-upstream.spec.mjs` E2E spec                                | ✅ Done | `beforeAll` uses `throw` (not `test.skip()`)   |
| 4     | TECHDEBT.md cleanup                                                      | ✅ Done | Items fully removed (not just marked resolved) |

**Key deviation from plan text**: Phase 2 and the `api-live-upstream.spec.mjs` draft in Phase 3 both proposed `test.skip(!apiKey, ...)` guards inside tests or `beforeAll` storing null. The actual implementation uses `throw new Error(...)` in `beforeAll` for missing env vars — tests hard-fail if the required API key is absent, matching BodhiApp policy.

## Context

Anthropic has a less-advertised `/v1/chat/completions` endpoint that accepts OpenAI-compatible format. BodhiApp's HEAD commit (`61656f0e`) added Anthropic Messages API support via a dedicated `/anthropic/v1/messages` proxy, and defensively rejected Anthropic-format aliases at `/v1/chat/completions` with a 400 error pointing to the Messages endpoint.

Since the existing opaque proxy pipeline (`routes_oai_chat.rs` -> `InferenceService::forward_remote` -> `AiApiService::forward_request_with_method`) already handles format-aware auth (`x-api-key` + `anthropic-version` for `ApiFormat::Anthropic`), simply removing the defensive rejection enables Anthropic models on `/v1/chat/completions` with zero additional plumbing.

**`/v1/models` already includes Anthropic aliases** (no format filter in `routes_oai_models.rs:78-84`). No schema change needed — `/bodhi/v1/models` exposes `api_format` for clients that need it.

## Phase 1: Backend — Remove Anthropic rejection

### `crates/routes_app/src/oai/routes_oai_chat.rs`

**Line 152**: Extend the allowlist guard to include `ApiFormat::Anthropic`:
```rust
// Before:
if matches!(api_alias.api_format, ApiFormat::OpenAI | ApiFormat::Placeholder) =>
// After:
if matches!(api_alias.api_format, ApiFormat::OpenAI | ApiFormat::Placeholder | ApiFormat::Anthropic) =>
```

**Lines 160-165**: Remove the Anthropic rejection arm entirely:
```rust
// DELETE this entire arm:
Alias::Api(ref api_alias) if api_alias.api_format == ApiFormat::Anthropic => {
  return Err(OaiApiError::from(OAIRouteError::InvalidRequest(format!(
    "Model '{}' is configured with 'anthropic' format which does not support the chat completions endpoint. Use the /anthropic/v1/messages endpoint instead.",
    api_alias.id
  ))));
}
```

### `crates/routes_app/src/oai/test_chat.rs`

**Convert `test_chat_completions_rejects_anthropic_format_alias` (lines 480-541)** to assert forwarding success. The test should:
- Seed an `ApiFormat::Anthropic` alias
- Use `MockInferenceService::expect_forward_remote` with `LlmEndpoint::ChatCompletions`
- Assert 200 OK response (not 400 rejection)

### Verification
```bash
cargo test -p routes_app -- test_chat
cargo test -p routes_app
```

## Phase 2: Fixtures — Add Anthropic to API_FORMATS

### `crates/lib_bodhiserver_napi/tests-js/fixtures/apiModelFixtures.mjs`

**Add `anthropic` to `API_FORMATS`** (line 18), with chat fields:
```js
anthropic: {
  format: 'anthropic',
  formatDisplayName: 'Anthropic',
  model: 'claude-3-5-haiku-20241022',
  baseUrl: 'https://api.anthropic.com/v1',
  envKey: 'INTEG_TEST_ANTHROPIC_API_KEY',
  chatQuestion: 'What day comes after Monday?',
  chatExpected: 'tuesday',
  chatEndpoint: '/v1/chat/completions',
},
```

**Remove the "intentionally NOT included" comment** (lines 11-17) that says "Anthropic chat routing through `/v1/chat/completions` is Phase 2 work".

### `crates/lib_bodhiserver_napi/tests-js/specs/api-models/api-models.spec.mjs`

**Change `beforeAll` error handling** (line 20-21) from `throw` to `test.skip()` for missing keys so CI without Anthropic key isn't blocked:
```js
// Before:
if (!apiKey) {
  throw new Error(`${formatConfig.envKey} environment variable not set`);
}
// After: use test.skip() or set a flag to skip individual tests
```

Since `test.skip()` isn't available in `beforeAll`, instead capture the skip condition and check in each test:
```js
test.beforeAll(async () => {
  const apiKey = process.env[formatConfig.envKey];
  // Store apiKey; tests check and skip if null
  testData = { apiKey, ... };
});

test('complete API model lifecycle...', async ({ page }) => {
  test.skip(!testData.apiKey, `${formatConfig.envKey} not set`);
  // ... rest of test
});
```

This enables the existing `api-models.spec.mjs` to automatically gain Anthropic chat-UI coverage via the `/anthropic/v1/messages` proxy path (already working from HEAD).

## Phase 3: New Playwright E2E — Live Upstream API Token Test

### New file: `crates/lib_bodhiserver_napi/tests-js/specs/api-models/api-live-upstream.spec.mjs`

Tests external API client flow: create alias via UI, create API token via UI, call endpoints with raw `fetch()` + Bearer token, verify real upstream responses.

**Structure**:
```js
import { test, expect } from '@/fixtures.mjs';
import { ApiModelFixtures } from '@/fixtures/apiModelFixtures.mjs';
import { ApiModelFormPage } from '@/pages/ApiModelFormPage.mjs';
import { LoginPage } from '@/pages/LoginPage.mjs';
import { ModelsListPage } from '@/pages/ModelsListPage.mjs';
import { TokensPage } from '@/pages/TokensPage.mjs';
import { TokenFixtures } from '@/fixtures/tokenFixtures.mjs';
import { getAuthServerConfig, getTestCredentials } from '@/utils/auth-server-client.mjs';

for (const [formatKey, formatConfig] of Object.entries(ApiModelFixtures.API_FORMATS)) {
  test.describe(`Live upstream API [${formatConfig.format}]`, () => {
    // ... setup pages

    test('create alias, mint token, call endpoint with Bearer', async ({ page, sharedServerUrl }) => {
      const apiKey = process.env[formatConfig.envKey];
      test.skip(!apiKey, `${formatConfig.envKey} not set`);

      // 1. Login
      await loginPage.performOAuthLogin();

      // 2. Create API model alias via UI
      const { modelId } = await registerApiModelViaUI(modelsPage, formPage, apiKey, formatConfig);

      // 3. Create API token via UI
      await tokensPage.navigateToTokens();
      await tokensPage.createToken('live-test-token', 'scope_token_user');
      const clipboard = await TokenFixtures.mockClipboard(page);
      const token = await tokensPage.copyTokenFromDialog();
      await tokensPage.closeTokenDialog();

      // 4. Call the endpoint with raw fetch + Bearer token
      const baseUrl = new URL(sharedServerUrl);
      const endpoint = formatConfig.chatEndpoint; // '/v1/chat/completions' or '/v1/responses'

      const body = endpoint === '/v1/responses'
        ? { model: formatConfig.model, input: 'What day comes after Monday?' }
        : { model: formatConfig.model, messages: [{ role: 'user', content: 'What day comes after Monday?' }] };

      const resp = await fetch(`${sharedServerUrl}${endpoint}`, {
        method: 'POST',
        headers: {
          'Content-Type': 'application/json',
          'Authorization': `Bearer ${token}`,
        },
        body: JSON.stringify(body),
      });

      expect(resp.status).toBe(200);
      const data = await resp.json();
      // Verify response shape varies by endpoint
      if (endpoint === '/v1/responses') {
        expect(data.output).toBeDefined();
      } else {
        expect(data.choices).toBeDefined();
        expect(data.choices.length).toBeGreaterThan(0);
      }

      // 5. Cleanup
      await modelsPage.navigateToModels();
      await modelsPage.deleteModel(modelId);
    });
  });
}
```

Key points:
- Uses Node.js `fetch()` (NOT `page.evaluate()`) — this is an API client test, not a UI assertion
- Skips gracefully when env key is missing
- Reuses existing page objects: `LoginPage`, `TokensPage`, `ApiModelFormPage`, `ModelsListPage`
- Reuses existing `registerApiModelViaUI` helper from `utils/api-model-helpers.mjs`
- Parameterized across all 3 formats (openai, openai_responses, anthropic) via the `API_FORMATS` loop

### Existing helpers to reuse
- `registerApiModelViaUI()` — `tests-js/utils/api-model-helpers.mjs`
- `TokensPage.createToken()`, `copyTokenFromDialog()` — `tests-js/pages/TokensPage.mjs`
- `TokenFixtures.mockClipboard()` — `tests-js/fixtures/tokenFixtures.mjs`
- `ApiModelFixtures.API_FORMATS` — `tests-js/fixtures/apiModelFixtures.mjs`

## Phase 4: TECHDEBT.md Update

### `ai-docs/claude-plans/202604/20260408-anthropic/TECHDEBT.md`

**Update Item 6** (Live integration tests gaps, line 154):
- Remove bullet about "x-api-key: bodhiapp_<token> middleware rewrite via the live HTTP server" — now covered by the new Playwright E2E spec
- Note the live API token test in `api-live-upstream.spec.mjs` addresses this gap

**Update header note**: Add line noting that Anthropic `/v1/chat/completions` is now supported via opaque proxy (not just `/anthropic/v1/messages`).

### `crates/lib_bodhiserver_napi/tests-js/specs/api-models/api-models-anthropic.spec.mjs`

**Update comment** (lines 16-18): Remove "Does NOT exercise chat (Phase 2)" since chat is now supported.

## Verification

1. `cargo test -p routes_app -- test_chat` — unit test for Anthropic forwarding
2. `cargo test -p routes_app` — full routes_app tests (683+)
3. `make test.backend` — all backend tests
4. `make build.ui-rebuild` — rebuild embedded UI
5. `cd crates/lib_bodhiserver_napi && npm run test:playwright` — E2E with real API keys

## Files Modified

| File                                                         | Change                                                                        |
| ------------------------------------------------------------ | ----------------------------------------------------------------------------- |
| `crates/routes_app/src/oai/routes_oai_chat.rs`               | Add `Anthropic` to allowlist, remove rejection arm                            |
| `crates/routes_app/src/oai/test_chat.rs`                     | Convert rejection test to forwarding success test                             |
| `tests-js/fixtures/apiModelFixtures.mjs`                     | Add `anthropic` to `API_FORMATS`, remove "intentionally NOT included" comment |
| `tests-js/specs/api-models/api-models.spec.mjs`              | `throw` in `beforeAll` (policy: no `test.skip` for missing env vars)          |
| `tests-js/specs/api-models/api-live-upstream.spec.mjs`       | **NEW** — live API token test for all 3 formats                               |
| `tests-js/specs/api-models/api-models-anthropic.spec.mjs`    | Move key check to `throw` in `beforeAll`; update comment (chat now supported) |
| `ai-docs/claude-plans/202604/20260408-anthropic/TECHDEBT.md` | Remove resolved items; renumber; update state                                 |

## Out-of-Scope

- **Format conversion**: `/v1/chat/completions` ↔ Anthropic native `/v1/messages` request/response conversion is not implemented and not needed — Anthropic's OpenAI-compat endpoint accepts the OpenAI format directly.
- **SSE streaming live test**: Streaming through `/v1/chat/completions` is not explicitly tested for the Anthropic format (`api-live-upstream.spec.mjs` uses non-streaming requests only). Tracked in TECHDEBT item 5.
- **`/v1/models` format filter**: No change — `/v1/models` already returns Anthropic-format aliases with no filtering. No `api_format` field needed in the OpenAI models list response.
- **`registerApiModelViaUI` changes**: No changes were needed; the helper already handles all format configs generically.

## Pending Items

No items specific to this plan remain open. Ongoing deferred work is in `TECHDEBT.md`:

- TECHDEBT item 5: Add live streaming + upstream error pass-through cases to `test_live_anthropic.rs`.

## Next Phase

- Chat UI via `/anthropic/v1/messages` already works (pi-ai `anthropic-messages` provider).  
- External clients can now call `/v1/chat/completions` with an Anthropic alias directly.  
- Both paths are covered by automated E2E tests (`api-models.spec.mjs` chat lifecycle + `api-live-upstream.spec.mjs` API token flow).  
- The Anthropic feature set is considered complete for Phase 1 scope. Future work is in TECHDEBT.md.
