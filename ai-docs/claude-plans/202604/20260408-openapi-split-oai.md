# Plan: Split OpenAPI spec into BodhiApp and OAI-compatible specs

## Context

`crates/routes_app/src/shared/openapi.rs` currently combines BodhiApp's internal management API with OpenAI/Ollama-compatible LLM endpoints into one `BodhiOpenAPIDoc`. This produces a 449-schema `openapi.json` where async-openai types (`CreateChatCompletionRequest`, `Response`, `Message`, etc.) sit in the same flat namespace as BodhiApp types (`Alias`, `UserResponse`, etc.). Risks of name collisions grow as the OpenAI spec evolves.

**Approach:** Generate **two** OpenAPI specs from two separate `#[derive(OpenApi)]` structs in `routes_app`:
- `openapi.json` — BodhiApp management/internal API only
- `openapi-oai.json` — OpenAI Chat/Embeddings/Responses + Ollama compatibility endpoints

The Axum router stays unchanged — both spec docs describe handlers that all live in one running server. Only the *documentation grouping* splits. The Swagger UI will expose both specs, and `ts-client` will generate two separate type modules.

## Files to create

### 1. `crates/routes_app/src/shared/openapi_oai.rs` (NEW)

A second `#[derive(OpenApi)]` struct, `BodhiOAIOpenAPIDoc`, that contains ONLY the OAI-compatible surface.

```rust
// crates/routes_app/src/shared/openapi_oai.rs

use crate::oai::{
    __path_chat_completions_handler, __path_embeddings_handler, __path_oai_model_handler,
    __path_oai_models_handler, __path_responses_cancel_handler, __path_responses_create_handler,
    __path_responses_delete_handler, __path_responses_get_handler,
    __path_responses_input_items_handler,
};
use crate::ollama::{
    __path_ollama_model_chat_handler, __path_ollama_model_show_handler,
    __path_ollama_models_handler,
};
use crate::OpenAIApiError;
use crate::{API_TAG_OPENAI, API_TAG_OLLAMA, API_TAG_RESPONSES};
use async_openai::types::{
    chat::{
        ChatChoice, ChatChoiceStream, ChatCompletionRequestMessage, ChatCompletionResponseMessage,
        CompletionUsage, CreateChatCompletionRequest, CreateChatCompletionResponse,
        CreateChatCompletionStreamResponse,
    },
    embeddings::{
        CreateEmbeddingRequest, CreateEmbeddingResponse, Embedding, EmbeddingInput, EmbeddingUsage,
    },
    models::{ListModelResponse, Model},
    responses::{CreateResponse, DeleteResponse as OaiDeleteResponse, Response as OaiResponse},
};
use utoipa::OpenApi;

#[derive(OpenApi)]
#[openapi(
    info(
        title = "Bodhi App - OpenAI Compatible APIs",
        version = env!("CARGO_PKG_VERSION"),
        contact(
            name = "Bodhi API Support",
            url = "https://github.com/BodhiSearch/BodhiApp/issues",
            email = "support@getbodhi.app"
        ),
        description = "OpenAI Chat Completions, Embeddings, Responses API and Ollama-compatible endpoints exposed by Bodhi App. Use the standard OpenAI/Ollama SDKs against these routes."
    ),
    servers(
        (url = "http://localhost:1135", description = "Local running instance"),
    ),
    tags(
        (name = API_TAG_OPENAI, description = "OpenAI-compatible API endpoints"),
        (name = API_TAG_RESPONSES, description = "OpenAI Responses API proxy endpoints"),
        (name = API_TAG_OLLAMA, description = "Ollama-compatible API endpoints"),
    ),
    components(
        schemas(
            // shared error type used in 4xx/5xx responses
            OpenAIApiError,
            // openai
            ListModelResponse, Model,
            CreateChatCompletionRequest, CreateChatCompletionResponse,
            CreateChatCompletionStreamResponse,
            ChatCompletionRequestMessage, ChatCompletionResponseMessage,
            ChatChoice, ChatChoiceStream, CompletionUsage,
            CreateEmbeddingRequest, CreateEmbeddingResponse,
            Embedding, EmbeddingInput, EmbeddingUsage,
            // responses api
            CreateResponse, OaiResponse, OaiDeleteResponse,
        ),
    ),
    paths(
        // OpenAI endpoints
        oai_models_handler, oai_model_handler,
        chat_completions_handler, embeddings_handler,
        // Responses API endpoints
        responses_create_handler, responses_get_handler,
        responses_delete_handler, responses_input_items_handler,
        responses_cancel_handler,
        // Ollama endpoints
        ollama_models_handler, ollama_model_show_handler, ollama_model_chat_handler,
    )
)]
pub struct BodhiOAIOpenAPIDoc;
```

**Note:** `OpenAPIEnvModifier`, `SecurityModifier`, `GlobalErrorResponses` continue to live in `openapi.rs` and are reused by both specs (they take any `&mut openapi::OpenApi`).

### 2. `xtask/src/openapi_oai.rs` (NEW)

```rust
use anyhow::Result;
use routes_app::{BodhiOAIOpenAPIDoc, GlobalErrorResponses, SecurityModifier};
use std::{fs::File, io::Write};
use utoipa::{Modify, OpenApi};

pub fn generate() -> Result<()> {
  let mut openapi = BodhiOAIOpenAPIDoc::openapi();
  SecurityModifier.modify(&mut openapi);
  GlobalErrorResponses.modify(&mut openapi);
  let mut file = File::create("openapi-oai.json")?;
  file.write_all(openapi.to_pretty_json()?.as_bytes())?;
  println!("OpenAI-compat spec written to openapi-oai.json");
  Ok(())
}
```

### 3. `ts-client/openapi-ts-oai.config.ts` (NEW)

```typescript
import { defineConfig } from '@hey-api/openapi-ts';

export default defineConfig({
  input: '../openapi-oai.json',
  output: 'src/types-oai',
  plugins: ['@hey-api/typescript'],
});
```

## Files to modify

### 4. `crates/routes_app/src/shared/openapi.rs`

**Remove** from `BodhiOpenAPIDoc`:
- Lines 1-9: imports of `__path_*` from `crate::oai` and `crate::ollama`
- Lines 64-75: `use async_openai::types::{...}`
- Lines 277-279: `API_TAG_OPENAI`, `API_TAG_RESPONSES`, `API_TAG_OLLAMA` from the `tags(...)` macro
- Lines 376-395 in `schemas(...)`: all async-openai types (the `// openai` and `// responses api` blocks)
- Lines 485-501 in `paths(...)`: OpenAI/Responses/Ollama handler entries

**Update** the `info(title = ...)` to clarify scope: `"Bodhi App - Management API"` (or keep as-is, since this is the "primary" spec).

**Keep** in `openapi.rs`:
- `ENDPOINT_*` constants (still used by handlers/tests)
- `OpenAIApiError` schema (still used as the error type for ALL endpoints, including Bodhi management — verify by grep)
- `apply_security_schemes`, `OpenAPIEnvModifier`, `SecurityModifier`, `GlobalErrorResponses` — reused by both specs

### 5. `crates/routes_app/src/shared/mod.rs`

Add new module declaration and re-export:
```rust
pub mod openapi;
pub mod openapi_oai;  // NEW

pub use openapi::*;
pub use openapi_oai::*;  // NEW — re-exports BodhiOAIOpenAPIDoc
```

### 6. `crates/routes_app/src/routes.rs` (line 486-498)

Mount BOTH specs in a single Swagger UI using utoipa-swagger-ui's multi-URL support (provides a dropdown to switch between specs):

```rust
let mut openapi = BodhiOpenAPIDoc::openapi();
OpenAPIEnvModifier::new(app_service.setting_service())
  .modify(&mut openapi)
  .await;
GlobalErrorResponses.modify(&mut openapi);

let mut openapi_oai = BodhiOAIOpenAPIDoc::openapi();
OpenAPIEnvModifier::new(app_service.setting_service())
  .modify(&mut openapi_oai)
  .await;
GlobalErrorResponses.modify(&mut openapi_oai);

let router = Router::<Arc<dyn AppService>>::new()
  .merge(public_router)
  .merge(session_protected)
  .merge(api_protected)
  .merge(
    SwaggerUi::new("/swagger-ui")
      .url("/api-docs/openapi.json", openapi)
      .url("/api-docs/openapi-oai.json", openapi_oai)
  )
  .with_state(state);
```

Both JSON specs are served at distinct URLs (`/api-docs/openapi.json`, `/api-docs/openapi-oai.json`) and the Swagger UI dropdown lets users switch between them. (Alternative: two separate `SwaggerUi::new("/swagger-ui")` and `SwaggerUi::new("/swagger-ui-oai")` mounts — but the multi-URL form is simpler and what utoipa-swagger-ui is designed for.)

Add the import: `use routes_app::BodhiOAIOpenAPIDoc;` near the existing `BodhiOpenAPIDoc` import.

### 7. `xtask/src/main.rs`

```rust
mod openapi;
mod openapi_oai;  // NEW
mod typescript;

use anyhow::Result;

fn main() -> Result<()> {
  let args: Vec<String> = std::env::args().collect();
  match args.get(1).map(|s| s.as_str()) {
    Some("openapi") => openapi::generate(),
    Some("openapi-oai") => openapi_oai::generate(),  // NEW
    Some("types") => typescript::generate_types(),
    _ => xtaskops::tasks::main(),
  }
}
```

### 8. `ts-client/package.json`

Add new scripts and update existing ones:
```json
{
  "scripts": {
    "build": "npm run generate && npm run bundle",
    "generate": "npm run generate:openapi && npm run generate:openapi-oai && npm run generate:types && npm run generate:types-oai && npm run generate:msw-types && npm run generate:msw-types-oai",
    "generate:openapi": "cd .. && cargo run --package xtask openapi",
    "generate:openapi-oai": "cd .. && cargo run --package xtask openapi-oai",
    "generate:types": "openapi-ts",
    "generate:types-oai": "openapi-ts --config openapi-ts-oai.config.ts",
    "generate:msw-types": "openapi-typescript ../openapi.json -o src/openapi-typescript/openapi-schema.ts",
    "generate:msw-types-oai": "openapi-typescript ../openapi-oai.json -o src/openapi-typescript/openapi-schema-oai.ts",
    "bundle:types": "tsc --emitDeclarationOnly --outDir dist && cpy 'src/types/**' 'dist/types' && cpy 'src/types-oai/**' 'dist/types-oai' && cpy 'src/openapi-typescript/**' 'dist/openapi-typescript'",
    "clean": "rm -rf dist && rm -rf src/types && rm -rf src/types-oai && rm -rf src/openapi-typescript"
  }
}
```

(Other scripts — `bundle:esm`, `bundle:cjs`, `test`, `build:openapi` — stay as-is.)

### 9. `ts-client/src/index.ts`

Re-export the OAI types as a namespaced module so consumers can disambiguate:

```typescript
// Re-export BodhiApp management API types
export * from './types';

// Re-export OpenAI/Ollama-compatible API types under explicit `oai` namespace
// to avoid name collisions (e.g. both have `Model`, `CreateResponse`, etc.)
export * as oai from './types-oai';

// MSW-compatible schema types for both specs
export type { paths, components } from './openapi-typescript/openapi-schema';
export type {
  paths as pathsOai,
  components as componentsOai,
} from './openapi-typescript/openapi-schema-oai';
```

Consumers then use:
```typescript
import { UserResponse, oai } from '@bodhiapp/ts-client';
const req: oai.CreateChatCompletionRequest = { ... };
```

### 10. `.gitignore` (verify)

`openapi-oai.json` should be tracked alongside `openapi.json` (it's a generated artifact but committed for diff visibility). Check current `.gitignore` for `openapi.json` — if not ignored, no change needed.

## Things to be careful about

1. **`OpenAIApiError` cross-reference**: The `GlobalErrorResponses` modifier (`openapi.rs:660`) hardcodes `Ref::from_schema_name("OpenAIApiError")` for 400/401/403/500 responses. Both specs MUST register `OpenAIApiError` in their schemas, otherwise refs will dangle. The plan registers it in both — verified.

2. **Schemas referenced transitively**: `OAIRequestParams` (in `BodhiOpenAPIDoc`) is a Bodhi-specific wrapper around OpenAI request params for alias config. utoipa will try to resolve any types it transitively pulls in. Run the generation and check for missing-schema warnings. If `OAIRequestParams` references async-openai types, those types may need to be re-registered in `BodhiOpenAPIDoc` too, OR we just accept some duplication across the two specs.

3. **Ollama vs OpenAI grouping**: The user's framing was "openai compatible endpoints". I'm including Ollama in the OAI spec since both are LLM-compatibility surfaces (vs BodhiApp management). If Ollama should be a third spec, that's a trivial extension of this plan. **Confirm during execution.**

4. **`OpenAPIEnvModifier::new(...)` is called twice**: Once per spec. It's cheap (reads settings, sets server URL). Both calls must `.await` since it's async.

5. **Tag constants (`API_TAG_OPENAI` etc.)** stay in `crates/routes_app/src/shared/constants.rs` — they're shared between handler annotations and the openapi-oai.rs.

## Verification

1. **Rust build**: `cargo build -p routes_app && cargo build -p xtask` — must compile clean
2. **Generate both specs**:
   ```bash
   cargo run --package xtask openapi       # writes openapi.json
   cargo run --package xtask openapi-oai   # writes openapi-oai.json
   ```
3. **Inspect counts**:
   ```bash
   python3 -c "
   import json
   for f in ['openapi.json', 'openapi-oai.json']:
     d = json.load(open(f))
     print(f, '— paths:', len(d['paths']), 'schemas:', len(d['components']['schemas']))
     print('  paths:', sorted(d['paths'].keys())[:5], '...')
   "
   ```
   Expect: `openapi.json` has ~69 paths (no `/v1/*` or `/api/*`); `openapi-oai.json` has ~12 paths (only `/v1/*` and `/api/*`).

4. **Server smoke test**:
   ```bash
   make app.run
   ```
   Then in browser:
   - http://localhost:1135/swagger-ui — Swagger UI loads with dropdown showing both specs
   - http://localhost:1135/api-docs/openapi.json — BodhiApp management spec
   - http://localhost:1135/api-docs/openapi-oai.json — OAI/Ollama spec
   - Verify both dropdowns work and render their respective endpoints

5. **TypeScript generation**:
   ```bash
   cd ts-client && npm run generate
   ```
   Expect both `src/types/types.gen.ts` and `src/types-oai/types.gen.ts` to be created. The OAI version should contain `CreateChatCompletionRequest` etc. but NOT `UserResponse`.

6. **Build full ts-client package**:
   ```bash
   make build.ts-client
   ```
   Must pass tests and bundle without errors. `dist/types-oai/` should exist.

7. **Type collision check**: Try a sample import in a TS file to confirm `oai.CreateChatCompletionRequest` works and conflicts are resolved by namespacing.

## Out of scope for this iteration

- Frontend code in `crates/bodhi/` doesn't need updating — it currently doesn't import async-openai types from `@bodhiapp/ts-client`. If/when it does, it will use `import { oai } from '@bodhiapp/ts-client'`.
- The `openai.` schema-name prefix work (utoipa `#[schema(as = openai::Foo)]`) explored earlier in this session is **not** part of this plan — splitting into two OpenAPI specs replaces the need for in-spec namespacing.
- Updating CLAUDE.md / PACKAGE.md docs for the affected crates — do as a follow-up after the spike lands.
