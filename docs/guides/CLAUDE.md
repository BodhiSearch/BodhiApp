# docs/guides/ — CLAUDE.md

Integration and usage guides for **external consumers** of BodhiApp's APIs and embedding bindings (deliberately distinct from internal crate docs). `README.md` is the human-facing suite landing page; this file is the assistant index.

| Doc | Covers |
|---|---|
| `overview.md` | What BodhiApp is and its API compatibility layers (OpenAI/Anthropic/Responses/native) |
| `getting-started.md` | Install, 4-step setup, first API call (`localhost:1135`) |
| `authentication.md` | Role hierarchy, session vs API-token auth, scopes, per-resource token grants (immutable at create) |
| `app-to-bodhi-oauth.md` | External-app access-request + owner-consent flow (draft→approve/deny/revoke), grant envelopes, internal RFC-8693 exchange |
| `openai-api.md` | OpenAI-compatible `/v1/*` endpoints (incl. Responses `/v1/responses`) |
| `bodhi-api.md` | Native `/bodhi/v1/*` endpoints (info, user + `access` envelope, tokens, app access-requests, settings, setup) |
| `model-management.md` | Model files, aliases, downloads, parameter config by permission level |
| `api-reference.md` | Quick endpoint reference + authorization matrix across all base URLs (incl. Anthropic `/anthropic/v1/*`) |
| `error-handling.md` | Error envelope formats, codes, retry patterns |
| `examples.md` | End-to-end integration examples using `@bodhiapp/ts-client` |
| `embedded-ui.md` | The built-in chat UI (Vite + React + TanStack) |
| `app-bindings-guide.md` | Embedding the server via `@bodhiapp/app-bindings` (NAPI): config layers, app states, lifecycle |
