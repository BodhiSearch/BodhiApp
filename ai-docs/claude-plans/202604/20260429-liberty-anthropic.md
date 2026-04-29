# Wire chat UI to Anthropic endpoint for `llm_liberty_oauth` aliases

## Context

A new `api_format = "llm_liberty_oauth"` recently landed. The backend's Anthropic proxy (`/anthropic/v1/messages`) already accepts requests against `llm_liberty_oauth` aliases when `provider == "anthropic"`, and the OpenAPI-generated TS types (`ApiAliasResponse.llm_liberty: LlmLibertySummary | null`, `ApiFormat` includes `'llm_liberty_oauth'`) are in place.

The chat UI, however, still picks the route off `apiFormat` alone. `agentStore.ts` falls into the default branch for `llm_liberty_oauth`, so requests go to `/v1/chat/completions` (OpenAI flat path). The Anthropic-protocol server doesn't answer there, so the chat UI hangs silently.

Goal: route chat through the Anthropic endpoint when the selected alias is `llm_liberty_oauth + provider=anthropic`. Anthropic only — keep the change minimal and modular; no registry abstraction.

## Decisions (from clarifying answers)

- **New field name**: `llmLibertyProvider: string | null` (scoped, not generic `apiProvider`).
- **Reset behavior**: AliasSelector always calls `setLlmLibertyProvider` on every selection — pulls from `alias.llm_liberty?.provider` for llm_liberty aliases, sets `null` otherwise.
- **Unknown-provider behavior**: silently fall back to OpenAI routing (the existing default branch). No toast, no validation gate. Anthropic is the only special case wired up.

## Routing matrix (only what changes)

| api_format            | provider     | piApi              | base path             |
|-----------------------|--------------|--------------------|-----------------------|
| `llm_liberty_oauth`   | `anthropic`  | `anthropic-messages` | `${origin}/anthropic` |
| `llm_liberty_oauth`   | anything else / null | `openai-completions` | `${origin}/v1`     |

Existing rows (`anthropic`, `anthropic_oauth`, `gemini`, `openai`, `openai_responses`) stay untouched.

## Files to change

### 1. `crates/bodhi/src/lib/chatDb.ts`
Extend `PersistedChatSettings` with `llmLibertyProvider?: string | null`. Dexie stores arbitrary fields under existing schema version 1 — no version bump needed.

### 2. `crates/bodhi/src/stores/chatSettingsStore.ts`
- Add `llmLibertyProvider: null` to `defaultSettings`.
- Add `setLlmLibertyProvider: (provider: string | null) => void` to `ChatSettingsStoreState` and implement: `set({ llmLibertyProvider: provider })`.
- In `saveForChat`, include `llmLibertyProvider: state.llmLibertyProvider` in the `PersistedChatSettings` snapshot.
- In `loadForChat`, the existing `{ ...defaultSettings, ...settings, ...sessionToken }` spread already pulls the saved value; for the "no chatId" / "no settings" branches that preserve `model`/`apiFormat`, also preserve `llmLibertyProvider` so it follows along when starting a new chat from the same alias.

### 3. `crates/bodhi/src/routes/chat/-components/settings/AliasSelector.tsx`
- Read `setLlmLibertyProvider` from the store.
- In the `setSelectedStatus` callback (line 129–138): after `setApiFormat(...)`, also call `setLlmLibertyProvider(alias && isApiAlias(alias) ? (alias.llm_liberty?.provider ?? null) : null)`. Always invoke — even on local models — so a stale provider can never linger.
- Extend the `useEffect` sync block (line 56–60) similarly. Derive `selectedLlmLibertyProvider` next to `selectedApiFormat` and call `setLlmLibertyProvider` inside the effect.

### 4. `crates/bodhi/src/stores/agentStore.ts`
- Change the three routing helpers to accept an optional second arg:
  - `apiFormatToPiApi(apiFormat, provider?: string | null): PiApi` — add a case `case 'llm_liberty_oauth': return provider === 'anthropic' ? 'anthropic-messages' : 'openai-completions'`.
  - `apiFormatToProvider(apiFormat, provider?: string | null): string` — return `'anthropic'` for `llm_liberty_oauth + 'anthropic'`, otherwise fall through to `'openai'`.
  - `getBaseUrl(apiFormat, provider?: string | null): string` — return `${origin}/anthropic` for `llm_liberty_oauth + 'anthropic'`, otherwise `${origin}/v1`.
- `buildModel(modelId, baseUrl, apiFormat, provider?)` — thread `provider` through to the two helpers it calls.
- Update all call sites to pass `settings.llmLibertyProvider`:
  - `append` (line 254–255): `getBaseUrl(settingsStore.apiFormat, settingsStore.llmLibertyProvider)` and `buildModel(modelId, baseUrl, settingsStore.apiFormat, settingsStore.llmLibertyProvider)`.
  - `syncAgentSettings` (line 369–371): same threading.
  - `restoreMessagesForChat` (line 182–207): pass `settingsStore.llmLibertyProvider` into both `apiFormatToPiApi` and `apiFormatToProvider` so reconstructed messages carry the right `api`/`provider` shape.

No new error path or toast — silent fallback to OpenAI for unknown providers, per decision.

### 5. Tests — `crates/bodhi/src/stores/agentStore.test.ts`
- Extend the parameterized routing test (line 220–239) with `['llm_liberty_oauth', 'anthropic', 'claude-haiku-4-5-20251001']` asserting `api: 'anthropic-messages'`, `provider: 'anthropic'`, `baseUrl` matches `/anthropic$/`. The third tuple slot threads through into `useChatSettingsStore.setState({ ..., llmLibertyProvider })`.
- Add a separate test for the fallback case: `apiFormat: 'llm_liberty_oauth'`, `llmLibertyProvider: null` (or `'codex'`) → expect OpenAI routing (`api: 'openai-completions'`, `baseUrl` ends `/v1`). This locks in the silent-fallback decision.

### 6. Tests — `crates/bodhi/src/routes/chat/-components/settings/AliasSelector.test.tsx`
- Update `useChatSettingsStore` mock (line 37–46) to include `setLlmLibertyProvider: vi.fn()`.
- Extend the `it.each(...)` table at line 344–408 with an `llm_liberty_oauth` row that includes a `llm_liberty: { provider: 'anthropic', envelope_version: 'v1', expires_at: ..., has_refresh_token: true }` field on the alias fixture, then assert both `setApiFormat` was called with `'llm_liberty_oauth'` and `setLlmLibertyProvider` was called with `'anthropic'`.
- Add an assertion in the existing "local model selected" test (line 410–436) that `setLlmLibertyProvider` is called with `null` — locks in the always-reset behavior.

### 7. Tests — `crates/bodhi/src/routes/chat/index.test.tsx` (line 95)
The existing settings-store mock spreads `apiFormat: 'openai'`. Add `llmLibertyProvider: null` to keep the mock shape consistent. Trivial.

### 8. Existing settings-store mocks elsewhere
Sweep for any other test that hand-constructs `useChatSettingsStore.setState({ ... apiFormat, ... })` and add `llmLibertyProvider: null` only if TypeScript complains. Most tests use partial setState, so most won't need updates — `chatStore.test.ts:272`, `SystemPrompt.test.tsx:15`, etc. should be fine.

## Reusable building blocks already in place

- `isApiAlias(model): model is ApiAliasResponse` at `crates/bodhi/src/lib/utils.ts:66` — clean type narrowing for `alias.llm_liberty?.provider` reads. No new helper needed.
- `LlmLibertySummary` type at `ts-client/src/types/types.gen.ts:805` — `provider: string` field is already strongly typed; no `as` casts.
- The existing parameterized test patterns in `agentStore.test.ts` and `AliasSelector.test.tsx` are already shaped for "add a new row" — no test infra changes.

## Verification

### Unit
```bash
cd crates/bodhi && npm run test -- agentStore.test AliasSelector.test
```
Expect: new llm_liberty_oauth + anthropic case hits `anthropic-messages` / `/anthropic`; fallback case hits `openai-completions` / `/v1`; AliasSelector calls both setters with the right values for both API and local rows.

Then full suite:
```bash
cd crates/bodhi && npm run test
```

### End-to-end (manual)
1. `ports kill 1135 && make app.run.live` (Vite HMR; no UI rebuild needed).
2. Open Chrome → `/ui/chat/`.
3. Pick the existing `claude-haiku-4-5-20251001` alias (mapped through alias id `01kqc6nfpfqh51xef9re31fwar`).
4. Send: `Reply with exactly: hello bodhi`.
5. Expect: streaming response. Spot-check Network panel: request goes to `/anthropic/v1/messages`, not `/v1/chat/completions`.
6. Send a follow-up to confirm message-restore on continued chat still picks the right `api`/`provider` shape (i.e. `restoreMessagesForChat` threading worked).

### Regression spot-checks
- Switch to a regular OpenAI alias → request hits `/v1/chat/completions`.
- Switch to an `anthropic_oauth` alias → request hits `/anthropic/v1/messages`. Both pre-existing paths must remain green.

## Commit cadence

Per layered-refactors preference, one commit per logical phase:

1. **chatSettingsStore + chatDb**: add field, default, setter, persistence.
2. **AliasSelector**: extract provider on selection.
3. **agentStore**: thread provider through the three routing helpers + callers.
4. **tests**: agentStore + AliasSelector + index test mocks.

Run `npm run test` and the manual chat round-trip before reporting done.
