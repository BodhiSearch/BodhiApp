# Fix: OpenRouter (and other OpenAI-compatible) model fetch returns empty list

## Context

`POST /bodhi/v1/models/api/fetch-models` returns `{"models":[]}` for OpenRouter
(`base_url: https://openrouter.ai/api/v1`), even though `curl https://openrouter.ai/api/v1/models`
returns a full list.

**Root cause** — `crates/services/src/ai_apis/provider_shared.rs:66-76`
(`fetch_openai_models`): the OpenAI-format parser deserializes each upstream entry directly
into the **strict third-party type** `async_openai::types::models::Model`, which has four
**non-optional** fields:

```rust
pub struct Model { id: String, object: String, created: u32, owned_by: String }
```

OpenRouter's `/models` entries carry `id` + `created` but **omit `object` and `owned_by`**
(confirmed live — keys are `id, canonical_slug, name, created, architecture, pricing, …`). So
`serde_json::from_value::<OpenAIModel>(...)` fails for **every** entry, and the
`.filter_map(|v| ….ok())` silently swallows all of them → empty list. A raw `curl` works
because it imposes no such struct. This affects **any** OpenAI-compatible provider that trims
the response (OpenRouter, Together, Groq, some local servers, …).

We **cannot** modify `async-openai` (third-party crate).

## Approach (surgical — confine the change to the parsing seam)

Keep `async_openai::types::models::Model` as the canonical type **everywhere it is used today**
— the `ApiModel::OpenAI(...)` variant, all construction sites, fixtures, and the OAI response
path all stay exactly as they are. Introduce a small **`LenientOpenAIModel`** used *only* to
parse the upstream `/models` response, with `#[serde(default)]` on the fields OpenRouter omits.
Immediately convert each parsed `LenientOpenAIModel` into `async_openai::Model` and continue
the existing flow unchanged.

This is safe because a full, verified inventory shows **nothing downstream ever reads
`.object`, `.created`, or `.owned_by`** off the OpenAI model — only `.id` is consumed
(`ApiModel::id()` at `model_objs.rs:126`; the endpoint returns only `m.id()`). The defaulted
values are never observed; they exist purely to satisfy the strict `async_openai::Model` shape
we keep using everywhere else.

### Single-file change

**`crates/services/src/ai_apis/provider_shared.rs`**

1. Add the lenient parse type (private to this module):

```rust
use async_openai::types::models::Model as OpenAIModel;
use serde::Deserialize;

/// Parse-only mirror of an OpenAI `/models` entry. OpenAI-compatible providers
/// (OpenRouter, Together, Groq, local servers, …) only reliably send `id`; the
/// strict `async_openai::Model` also requires `object`/`created`/`owned_by` and
/// would drop every such entry. We default the missing fields here, then convert
/// to the canonical `async_openai::Model` so the rest of the codebase is unchanged.
#[derive(Debug, Deserialize)]
struct LenientOpenAIModel {
  id: String,
  #[serde(default = "default_object")]
  object: String,
  #[serde(default)]
  created: u32,
  #[serde(default)]
  owned_by: String,
}

fn default_object() -> String {
  "model".to_string()
}

impl From<LenientOpenAIModel> for OpenAIModel {
  fn from(m: LenientOpenAIModel) -> Self {
    OpenAIModel { id: m.id, object: m.object, created: m.created, owned_by: m.owned_by }
  }
}
```

Defaults: `object` → `"model"`, `owned_by` → `""` (empty string via `Default`), `created` → `0`.
(`created` stays `u32` to match `async_openai::Model`; OpenRouter's value fits, and it is never
read. If serde's u64→u32 coercion on a large `created` ever errors for some provider, that entry
simply falls back to the existing skip behavior — acceptable since the field is unused.)

2. In `fetch_openai_models`, change only the deserialize target — the `data`/`filter_map`/
   `unwrap_or_default` structure stays identical:

```rust
let models: Vec<ApiModel> = body
  .get("data")
  .and_then(|d| d.as_array())
  .map(|arr| {
    arr
      .iter()
      .filter_map(|v| serde_json::from_value::<LenientOpenAIModel>(v.clone()).ok())
      .map(|m| ApiModel::OpenAI(m.into()))   // LenientOpenAIModel -> async_openai::Model
      .collect()
  })
  .unwrap_or_default();
```

No `tracing` logging (stay silent, matching the existing Gemini/Anthropic parsers).

### Everything else is unchanged (verified inventory)

These keep using `async_openai::Model` as-is — **do not touch**:
- `crates/services/src/models/model_objs.rs:4,116,126` — `ApiModel::OpenAI(OpenAIModel)` + `id()`.
- `crates/services/src/ai_apis/clients/liberty_codex.rs:9,270` — `parse_codex_models` already
  constructs `OpenAIModel { … }` with explicit values; leave it.
- `crates/services/src/test_utils/fixtures.rs:6,9` — `openai_model(id)` helper.
- `crates/routes_app/src/routes_dev.rs:312`, `crates/server_core/src/test_shared_rw.rs:304` — fixtures.
- `crates/routes_app/src/oai/routes_oai_models.rs:185-192` — builds the OAI `/v1/models`
  **response** `Model` fresh from `ApiAlias`; correct as-is.
- `crates/routes_app/src/anthropic/routes_anthropic.rs:202` — `ApiModel::OpenAI(_)` no-op arm.

### No OpenAPI / TS-client change

Because the public `ApiModel::OpenAI` variant still wraps `async_openai::Model`, the generated
OpenAPI schema and `ts-client` types are unchanged. `LenientOpenAIModel` is a private,
non-`ToSchema` parse helper. (Confirm no spec diff by running the generator in verification.)

## Verification

1. **Unit test** in `provider_shared.rs` (`#[cfg(test)] mod tests`): feed an OpenRouter-shaped
   `data` array (entries with `id`/`created` but **no** `object`/`owned_by`, plus the extra
   OpenRouter keys like `canonical_slug`, `pricing`) through the parse closure and assert the
   `ApiModel::OpenAI` ids come through, with `object == "model"` and `owned_by == ""` defaulted.
   Also keep a case with the full strict shape to prove no regression.
2. `cargo check -p services`, then `cargo test -p services` (ai_apis + model tests).
3. `make test.backend` (tee to a file per the slow-command convention).
4. `cargo run --package xtask openapi` and `make ci.ts-client-check` — expect **no** spec diff.
5. **End-to-end manual** — needs the rebuilt binary (`make app.run`; stale `app.run.live`
   would ignore the change). Replay the failing request:
   ```
   curl -s http://localhost:1135/bodhi/v1/models/api/fetch-models \
     -H 'Content-Type: application/json' -b 'bodhiapp_session_id=…' \
     --data-raw '{"api_format":"openai","creds":{"type":"api_key","value":null},"base_url":"https://openrouter.ai/api/v1"}'
   ```
   Expect a populated `{"models":[ … ]}`. Also confirm an OpenAI / llm-liberty-codex base_url
   still returns models (no regression on providers that DO send the full shape).
