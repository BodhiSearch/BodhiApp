# Chat UI Migration - Technical Debt

## 1. Dummy API Key Workaround

**Status**: Active workaround
**Impact**: Low risk, cosmetic auth headers

### Problem

pi-ai's built-in providers (`openai-completions`, `openai-responses`) require a truthy `apiKey` in `StreamOptions`. The `streamSimple*` functions throw `No API key for provider: ...` if `apiKey` is falsy. Bodhi's architecture is browser -> Bodhi backend (cookie auth) -> LLM provider, so the browser never holds real API keys.

### Current Approach

Pass `apiKey: 'bodhi-proxy'` to pi-ai providers to satisfy the truthy apiKey requirement. A custom `bodhiStreamFn` wrapper around `streamSimple` patches `model.headers` with `{ Authorization: null }` before forwarding. The OpenAI SDK treats `null` in `defaultHeaders` as "remove this header", so no `Authorization` header is sent. The Bodhi backend then falls through to cookie-based session auth.

The wrapper lives in `crates/bodhi/src/hooks/chat/useBodhiAgent.ts` and is passed as `streamFn` to the pi-agent-core `Agent` constructor.

### Why Not Just Omit the API Key?

pi-ai's `streamSimple*` functions throw `No API key for provider: ...` if `apiKey` is falsy. The dummy key satisfies this check, and the Authorization header it generates is stripped by the wrapper.

### Why Strip Instead of Backend Tolerance?

The Bodhi backend's `auth_middleware` prioritizes the `Authorization` header over session cookies. If present, it validates the bearer token -- `bodhi-proxy` fails validation with `401 Invalid token`. Stripping the header on the frontend is safer than changing backend auth precedence.

### Pros

- Works with all built-in providers without forking pi-ai
- No custom provider maintenance burden
- pi-ai's SSE parsing, message conversion, and event model used as-is
- No backend changes required

### Cons

- Type cast (`null as unknown as string`) to satisfy `Record<string, string>` in model headers
- For future Anthropic provider: will need similar header stripping for `x-api-key`
- For future Google provider: may need query parameter stripping for `?key=`
- Relies on OpenAI SDK internal behavior (null in defaultHeaders removes header)

### Resolution Path

Upstream a PR to `badlogic/pi-mono` making `apiKey` optional in provider `createClient` functions, or adding a `fetch` pass-through option. When merged, remove both the dummy key and the header-stripping wrapper.

## 2. llama.cpp Timings Loss

**Status**: Accepted for now
**Impact**: Performance metrics no longer shown in chat UI

### Problem

The previous hand-rolled SSE parser extracted llama.cpp vendor extension fields (`timings.prompt_per_second`, `timings.predicted_per_second`) from streaming chunks. pi-ai's `openai-completions` provider parses standard OpenAI fields only and does not capture vendor extensions.

### What's Lost

- "Speed: X tokens/s" display in message metadata
- `prompt_per_second` and `predicted_per_second` metrics

### What's Preserved

- Standard OpenAI `Usage` (completion_tokens, prompt_tokens, total_tokens) via pi-ai
- Token count display still possible

### Resolution Path

Either:
1. Register a custom provider (extending `openai-completions`) that captures vendor fields from SSE chunks
2. Use pi-ai's `StreamOptions.onPayload` callback to intercept raw payloads and extract timings alongside the standard pipeline
3. Fetch timings from a separate backend endpoint after completion

## 3. Future Providers (Out of Scope)

### Anthropic Messages API

- Requires new Bodhi backend route: `POST /v1/messages` speaking Anthropic wire protocol
- New `ApiFormat::Anthropic` variant in Rust enum
- pi-ai's `anthropic-messages` provider points `baseUrl` at Bodhi backend
- Backend must forward `anthropic-beta` headers and handle Anthropic SSE event format

### Google Gemini API

- Requires new Bodhi backend route: `POST /v1/gemini/models/{model}:streamGenerateContent`
- New `ApiFormat::Gemini` variant in Rust enum
- pi-ai's `google-generative-ai` provider points `baseUrl` at Bodhi backend
- Backend must inject API key (query param) and handle Gemini SSE format

### OAuth Token Support

- Extend alias/API model configuration to store OAuth credentials alongside API keys
- Backend token refresh logic
- Frontend auth type display in model selector
- pi-ai Anthropic provider auto-detects OAuth tokens by `sk-ant-oat` prefix
