# Remove Ollama-Compatible Endpoints

## Context

BodhiApp exposes three Ollama-compatible HTTP endpoints (`GET /api/tags`, `POST /api/show`, `POST /api/chat`) as a compatibility layer for clients using Ollama SDKs. This layer has not been maintained for a long time. `crates/routes_app/TECHDEBT.md` already flags it: *"Ollama compatibility is being dropped soon."*

The removal is well-isolated: all code lives in a single `ollama/` module under `routes_app`, there is **no** frontend UI, E2E test, or NAPI usage, and there is **no** `ApiFormat`/provider enum variant for Ollama (it is purely an *exposed* compat surface, not a consumed upstream provider). This makes the change a clean module deletion + route unregistration + regeneration of derived artifacts.

Per BodhiApp's stance, no backwards-compatibility shim is needed — the endpoints go away entirely.

## Scope

Remove only the **exposed** Ollama-compatible endpoints and everything that exists solely to serve them. General OpenAI-format model-conversion utilities are not shared with the Ollama module (its converters — `user_alias_to_ollama_model`, `model_alias_to_ollama_model`, `alias_to_ollama_model_show` — are Ollama-only and get deleted with the directory), so nothing outside `ollama/` loses functionality.

## Changes

### 1. Delete the Ollama module (entire directory)
Delete `crates/routes_app/src/ollama/` — `mod.rs`, `error.rs` (`OllamaRouteError`), `ollama_api_schemas.rs`, `routes_ollama.rs` (3 handlers + converters), `test_handlers.rs` (5 tests).

### 2. Unregister the module — `crates/routes_app/src/lib.rs`
- Remove `pub mod ollama;` (line 19)
- Remove `pub use ollama::*;` (line 42)

### 3. Unregister the routes — `crates/routes_app/src/routes.rs`
- Remove the handler/constant imports (lines 61–62: `ollama_model_chat_handler, ollama_model_show_handler, ollama_models_handler, ENDPOINT_OLLAMA_CHAT, ENDPOINT_OLLAMA_SHOW, ENDPOINT_OLLAMA_TAGS`)
- Remove the `// Ollama APIs` comment + 3 `.route(...)` registrations (lines 213–216)

### 4. Update the OAI OpenAPI doc — `crates/routes_app/src/oai/openapi.rs`
- Remove the `use crate::ollama::{ __path_ollama_* }` import block (lines 7–9)
- Remove `API_TAG_OLLAMA` from the `use crate::{...}` import (line 10)
- Remove the `API_TAG_OLLAMA` tag registration (line 58)
- Remove the 3 handler paths `ollama_models_handler, ollama_model_show_handler, ollama_model_chat_handler` (lines 95–97)
- Update the doc comment (line 26) and `description` string (lines 41–43) to drop "Ollama-compatible"/"or Ollama SDKs" wording

### 5. Remove the tag constant — `crates/routes_app/src/shared/constants.rs`
- Remove `pub const API_TAG_OLLAMA: &str = "ollama";` (line 14)

### 6. Doc-string cleanup — `crates/routes_app/src/shared/openapi.rs`
- Line 156: change `OpenAI/Ollama APIs` → `OpenAI APIs`

### 7. Crate docs — `crates/routes_app/`
- `PACKAGE.md`: remove the `ollama/` module-table row, the `OllamaRouteError` error-table row, "ollama" from the OpenAPI tags list, and "Ollama" from OAI/OpenAPI mentions (lines ~22, 75, 107, 122, 141)
- `TECHDEBT.md`: delete the "## OAI/Ollama model conversion logic" section (lines 8–11) — resolved by this change

### 8. User-facing docs — `docs/`
- Delete `docs/guides/ollama-api.md`
- `docs/guides/CLAUDE.md`: remove the `ollama-api.md` index row (line 12) and drop "Ollama" from the `overview.md` compat-layers list (line 7)
- `docs/CLAUDE.md`: drop "Ollama" from the guides description (line 14)
- Check `docs/guides/overview.md` for an Ollama section/link and remove it

### 9. Regenerate derived artifacts (do NOT hand-edit)
These are generated — regenerate after the Rust changes land:
- `cargo run --package xtask openapi` → regenerates `openapi.json` and `openapi-oai.json` (drops the `OllamaModel`/`OllamaError` schemas, the 3 Ollama operations, the "ollama" tag, and the `(Ollama Compatible)` labels)
- `cd ts-client && npm run generate` (or `make build.ts-client`) → regenerates `ts-client/src/openapi-typescript/openapi-schema-oai.ts` and `ts-client/src/types-oai/types.gen.ts` (drops `OllamaModel`, `OllamaError`, `chatOllamaModel`, `showOllamaModel`, `listOllamaModels` and their `*Data/*Response/*Error` types)
- Hand-written comment lines in `ts-client/src/openai.ts` (line 2) and `ts-client/src/index.ts` (line 2) still say "OpenAI/Ollama-compatible types" — update these to just "OpenAI-compatible types"

## Verification

1. **Compile:** `cargo check -p routes_app 2>&1 | tail -5` — no unresolved `ollama` / `API_TAG_OLLAMA` references.
2. **Backend tests:** `cargo test -p routes_app 2>&1 | grep -E "test result|FAILED|failures:"` — the deleted Ollama tests are gone; nothing else references them. Then `make test.backend`.
3. **OpenAPI in sync:** `cargo run --package xtask openapi` then `make ci.ts-client-check` — confirms the spec + TS client regenerated cleanly with no Ollama artifacts and no drift.
4. **Grep sweep:** `grep -rli ollama crates ts-client docs openapi.json openapi-oai.json | grep -v node_modules` should return nothing (or only unrelated hits) — confirms full removal.
5. **Runtime smoke:** start the app (`make app.run`) and confirm `GET /api/tags` now returns 404 (route removed) while OpenAI-compatible routes (`/v1/chat/completions`, `/v1/models`) still work.
6. **Frontend:** `cd crates/bodhi && npm run build` — verifies the regenerated `@bodhiapp/ts-client` types still compile against frontend code (no code imported the Ollama types, so this should be a no-op safety check).
