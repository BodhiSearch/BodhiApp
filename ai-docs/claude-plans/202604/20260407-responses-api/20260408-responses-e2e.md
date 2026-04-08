# Plan: OpenAI Responses API Frontend Support + Parameterized E2E Tests

## Context

The backend supports two API formats (`openai` for chat completions, `openai_responses` for Responses API) as pass-through proxies. However, the frontend chat always uses `api: 'openai-completions'` via pi-ai, so `openai_responses` models fail with a 400 error in chat. The pi-ai library already supports `'openai-responses'` as a Model API type. Additionally, E2E tests only cover `openai` format — they need parameterization for both formats and future ones (Anthropic, Gemini).

This plan also addresses review findings: UI #1 (stale agent state), UI #3 (missing abort on unmount), E2E #1 (mock missing `/responses`), E2E #3 (no `openai_responses` E2E test), cross-cutting #2 (chat UI doesn't use Responses API).

## Phase 1: Frontend — Wire api_format to chat agent

### 1.1 Update `buildModel()` to accept api format
**File**: `crates/bodhi/src/hooks/chat/useBodhiAgent.ts`

- Change `buildModel(modelId, baseUrl)` → `buildModel(modelId, baseUrl, apiFormat)`
- Map `'openai'` → `api: 'openai-completions'`, `'openai_responses'` → `api: 'openai-responses'`
- Return type: `Model<Api>` (generic, since pi-ai's `streamSimple` accepts any `Model<Api>`)
- In `append()` callback (line 195): pass `chatSettings.apiFormat` to `buildModel()`
- In restored message fallback (line 226): use mapped api string instead of hardcoded `'openai-completions'`

### 1.2 Detect api_format on model selection in AliasSelector
**File**: `crates/bodhi/src/routes/chat/-components/settings/AliasSelector.tsx`

- Import `setApiFormat` from `useChatSettings` (already available, just not destructured)
- Build a `modelToAliasMap: Map<string, AliasResponse>` via `useMemo` from `models` prop
  - For API aliases: map each `formatPrefixedModel(modelName, m.prefix)` → alias
  - For local aliases: map `m.alias` → alias
- Update selection handler (line 98): after `setModel(value)`, look up alias in map
  - If `isApiAlias(alias)` → `setApiFormat(alias.api_format as ApiFormatSetting)`
  - Otherwise → `setApiFormat('openai')` (local models use completions)

### 1.3 Display read-only api_format label in settings sidebar
**File**: `crates/bodhi/src/routes/chat/-components/settings/SettingsSidebar.tsx`

- After `<AliasSelector>` (line 59-60), add a read-only label:
  ```tsx
  {settings.apiFormat && settings.model && (
    <div className="text-xs text-muted-foreground" data-testid="api-format-label">
      API Format: {API_FORMAT_PRESETS[settings.apiFormat]?.name ?? settings.apiFormat}
    </div>
  )}
  ```
- Import `API_FORMAT_PRESETS` from `@/schemas/apiModel`
- Only display when a model is selected

### 1.4 Fix review findings UI #1 and UI #3
**File**: `crates/bodhi/src/hooks/chat/useBodhiAgent.ts`

**UI #1 (agent state sync)**: Add `useEffect` to sync agent state when settings change:
```ts
useEffect(() => {
  const agent = getAgent();
  if (chatSettings.model) {
    agent.state.model = buildModel(chatSettings.model, baseUrl, chatSettings.apiFormat);
  }
  agent.state.tools = tools;
  agent.state.systemPrompt = (chatSettings.systemPrompt_enabled && chatSettings.systemPrompt) 
    ? chatSettings.systemPrompt : '';
}, [chatSettings.model, chatSettings.apiFormat, chatSettings.systemPrompt, chatSettings.systemPrompt_enabled, tools, getAgent, baseUrl]);
```

**UI #3 (abort on unmount)**: In the subscription `useEffect` cleanup (line 135):
```ts
return () => {
  unsubscribe();
  agentRef.current?.abort();
};
```

### 1.5 Unit tests
**File**: `crates/bodhi/src/routes/chat/-components/settings/AliasSelector.test.tsx`

- Add test: selecting an API model with `api_format: 'openai_responses'` calls `setApiFormat('openai_responses')`
- Add test: selecting a local model calls `setApiFormat('openai')`

**File**: `crates/bodhi/src/hooks/chat/useBodhiAgent.test.tsx`

- Add test: `buildModel` with `'openai_responses'` returns model with `api: 'openai-responses'`

### 1.6 Build
```bash
make build.ui-rebuild
```

## Phase 2: Parameterize E2E fixtures

### 2.1 Add format configs to apiModelFixtures
**File**: `crates/lib_bodhiserver_napi/tests-js/fixtures/apiModelFixtures.mjs`

```js
static API_FORMATS = {
  openai: {
    format: 'openai',
    formatDisplayName: 'OpenAI - Completions',
    model: 'gpt-4.1-nano',
    baseUrl: 'https://api.openai.com/v1',
    envKey: 'INTEG_TEST_OPENAI_API_KEY',
    chatQuestion: 'What day comes after Monday?',
    chatExpected: 'tuesday',
    chatEndpoint: '/v1/chat/completions',  // for mock server request log filtering
  },
  openai_responses: {
    format: 'openai_responses',
    formatDisplayName: 'OpenAI - Responses',
    model: 'gpt-4.1-nano',
    baseUrl: 'https://api.openai.com/v1',
    envKey: 'INTEG_TEST_OPENAI_API_KEY',
    chatQuestion: 'What day comes after Monday?',
    chatExpected: 'tuesday',
    chatEndpoint: '/v1/responses',
  },
};
```

Also add `createModelDataForFormat(formatKey)` helper.

### 2.2 Update ApiModelFormComponent for responses format
**File**: `crates/lib_bodhiserver_napi/tests-js/pages/components/ApiModelFormComponent.mjs`

- In `selectApiFormat()`, update the base URL auto-population check (line 52) to also handle `openai_responses`:
  ```js
  if (format === 'openai' || format === 'openai_responses') {
    await expect(this.page.locator(this.selectors.baseUrlInput)).toHaveValue('https://api.openai.com/v1');
  }
  ```

## Phase 3: Add responses endpoint to mock server

### 3.1 Add `POST /v1/responses` handler
**File**: `crates/lib_bodhiserver_napi/tests-js/utils/mock-openai-server.mjs`

Add handler for `POST /v1/responses` with:
- Auth check (same as chat completions)
- Extract user message from `input` field (can be string or array of message objects)
- **Streaming** (`stream: true`): Send SSE with proper Responses API event types:
  - `response.created`, `response.output_item.added`, `response.content_part.added`
  - `response.output_text.delta` (with content), `response.output_text.done`
  - `response.content_part.done`, `response.output_item.done`, `response.completed`
- **Non-streaming**: Return JSON with `{ id, object: 'response', status: 'completed', output: [{type: 'message', role: 'assistant', content: [{type: 'output_text', text: '...'}]}], usage: {...} }`

## Phase 4: Parameterize E2E test specs

### 4.1 Parameterize `api-models.spec.mjs` (live API tests)
**File**: `crates/lib_bodhiserver_napi/tests-js/specs/api-models/api-models.spec.mjs`

Wrap in `for (const [formatKey, formatConfig] of Object.entries(ApiModelFixtures.API_FORMATS))`:
- `test.describe(\`API Models Integration [${formatConfig.format}]\`, ...)`
- In `beforeAll`: get API key from `process.env[formatConfig.envKey]`
- Lifecycle test: call `selectApiFormat(formatConfig.format)` before filling form
- Use `formatConfig.model` throughout
- Chat assertions use `formatConfig.chatQuestion` / `formatConfig.chatExpected`
- Verify pre-filled format: `verifyFormPreFilled(formatConfig.format, formatConfig.baseUrl)`
- Model list verification: `verifyApiModelInList(id, formatConfig.format, baseUrl)`

### 4.2 Parameterize `api-models-no-key.spec.mjs` (mock server tests)
**File**: `crates/lib_bodhiserver_napi/tests-js/specs/api-models/api-models-no-key.spec.mjs`

Same loop pattern. Key differences:
- Filter request log by `formatConfig.chatEndpoint` instead of hardcoded `/v1/chat/completions`
- Call `selectApiFormat(formatConfig.format)` when creating model

### 4.3 Update `api-model-helpers.mjs`
**File**: `crates/lib_bodhiserver_napi/tests-js/utils/api-model-helpers.mjs`

- Add optional `formatConfig` parameter to `registerApiModelViaUI`:
  ```js
  export async function registerApiModelViaUI(modelsPage, formPage, apiKey, formatConfig = null) {
    const config = formatConfig || ApiModelFixtures.API_FORMATS.openai;
    // ...use config.format, config.model, config.baseUrl
  }
  ```

## Phase 5: Verify

### Gate checks (run in order):
1. `cd crates/bodhi && npm test` — frontend unit tests pass
2. `make build.ui-rebuild` — rebuild embedded UI
3. `cd crates/lib_bodhiserver_napi && npx playwright test --project=standalone tests-js/specs/api-models/api-models.spec.mjs` — live API tests pass for both formats
4. `cd crates/lib_bodhiserver_napi && npx playwright test --project=standalone tests-js/specs/api-models/api-models-no-key.spec.mjs` — mock tests pass for both formats
5. `cd crates/lib_bodhiserver_napi && npx playwright test --project=standalone tests-js/specs/api-models/` — full api-models suite passes
6. `cd crates/lib_bodhiserver_napi && npx playwright test --project=standalone tests-js/specs/chat/chat.spec.mjs` — chat tests still pass
7. `cd crates/lib_bodhiserver_napi && npx playwright test --project=standalone tests-js/specs/setup/` — setup tests still pass

### Manual verification:
- Launch app with `make app.run`
- Create an `openai_responses` format API model
- Select it in chat → verify "API Format: OpenAI - Responses" label appears
- Send a message → verify response comes back (uses `/v1/responses` endpoint)
- Switch to an `openai` format model → label shows "OpenAI - Completions"
- Chat works normally with completions model

## Critical files

| File | Change |
|------|--------|
| `crates/bodhi/src/hooks/chat/useBodhiAgent.ts` | buildModel format param, useEffect sync, abort cleanup |
| `crates/bodhi/src/routes/chat/-components/settings/AliasSelector.tsx` | Detect api_format on selection |
| `crates/bodhi/src/routes/chat/-components/settings/SettingsSidebar.tsx` | Read-only format label |
| `crates/lib_bodhiserver_napi/tests-js/fixtures/apiModelFixtures.mjs` | API_FORMATS config |
| `crates/lib_bodhiserver_napi/tests-js/utils/mock-openai-server.mjs` | `/v1/responses` endpoint |
| `crates/lib_bodhiserver_napi/tests-js/specs/api-models/api-models.spec.mjs` | Parameterize |
| `crates/lib_bodhiserver_napi/tests-js/specs/api-models/api-models-no-key.spec.mjs` | Parameterize |
| `crates/lib_bodhiserver_napi/tests-js/utils/api-model-helpers.mjs` | Accept format config |
| `crates/lib_bodhiserver_napi/tests-js/pages/components/ApiModelFormComponent.mjs` | Base URL check for responses |
