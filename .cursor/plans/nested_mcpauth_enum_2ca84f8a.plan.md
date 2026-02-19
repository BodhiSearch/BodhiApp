---
name: Nested McpAuth enum
overview: Refactor McpAuth to use a nested `auth` object with `#[serde(tag = "type")]` -- no flatten, no defaults. `Option<McpAuth>` for Update gives clean keep/change semantics. All three request types use the same enum consistently.
todos:
  - id: types-enum
    content: "Restore McpAuth as #[serde(tag = \"type\")] enum, use as nested field on Create/FetchTools (required) and Update (Option)"
    status: completed
  - id: handlers
    content: Update create/update/fetch-tools handlers to match on McpAuth enum variants
    status: completed
  - id: openapi-schema
    content: Re-add McpAuth to openapi.rs schema registration
    status: completed
  - id: backend-tests
    content: Update all mcps_test.rs and test_live_mcp.rs to use nested McpAuth
    status: completed
  - id: backend-verify
    content: cargo test -p routes_app && cargo test -p server_app && cargo fmt --all
    status: completed
  - id: todo-1771466644571-zzdkzpckt
    content: Regenerate OpenAPI spec + TypeScript client (xtask openapi + make build.ts-client)
    status: completed
  - id: frontend-types
    content: Update useMcps.ts types and page.tsx form submission to nested auth object
    status: completed
  - id: frontend-verify
    content: npm run test from crates/bodhi
    status: completed
  - id: regen
    content: Regenerate OpenAPI spec + TypeScript client (xtask openapi + make build.ts-client)
    status: completed
isProject: false
---

# Nested McpAuth Enum Refactor

## Design

The `McpAuth` enum uses `#[serde(tag = "type")]` as a **nested field** (not flattened) on all request types:

```rust
#[serde(tag = "type")]
enum McpAuth {
  #[serde(rename = "public")]
  Public,
  #[serde(rename = "header")]
  Header { header_key: String, header_value: String },
  // Future: OAuth21 { client_id: String, token_url: String, scopes: String }
}
```

JSON shape:

- Create/FetchTools (required): `{ ..., "auth": { "type": "public" } }` or `{ ..., "auth": { "type": "header", "header_key": "...", "header_value": "..." } }`
- Update (optional): omit `auth` to keep existing, include to change

## Backend changes

### 1. [types.rs](crates/routes_app/src/routes_mcps/types.rs)

Replace current flat fields / enum with:

```rust
#[derive(Debug, Clone, Deserialize, Serialize, ToSchema)]
#[serde(tag = "type")]
pub enum McpAuth {
  #[serde(rename = "public")]
  Public,
  #[serde(rename = "header")]
  Header {
    header_key: String,
    header_value: String,
  },
}

struct CreateMcpRequest  { ..., pub auth: McpAuth }           // required
struct UpdateMcpRequest  { ..., pub auth: Option<McpAuth> }   // None = keep existing
struct FetchMcpToolsRequest { ..., pub auth: McpAuth }        // required
```

No `#[serde(flatten)]`, no `#[serde(default)]` on auth fields. Clean serde.

### 2. [mcps.rs](crates/routes_app/src/routes_mcps/mcps.rs) handlers

- `create_mcp_handler`: `match request.auth { McpAuth::Header { header_key, header_value } => (Some(..), Some(..)), McpAuth::Public => (None, None) }`
- `update_mcp_handler`: `match request.auth { Some(McpAuth::Header { .. }) => (.., false), Some(McpAuth::Public) => (None, None, false), None => (None, None, true) }`
- `fetch_mcp_tools_handler`: same pattern as create
- Re-add `McpAuth` to import

### 3. [openapi.rs](crates/routes_app/src/shared/openapi.rs)

Re-add `McpAuth` to schema registration list and import.

### 4. [mcps_test.rs](crates/routes_app/src/routes_mcps/tests/mcps_test.rs)

Update all test request bodies:

- `CreateMcpRequest { ..., auth: McpAuth::Public }` (was `auth: "public".to_string()`)
- `FetchMcpToolsRequest { ..., auth: McpAuth::Public }` (same)
- `UpdateMcpRequest { ..., auth: None }` for keep-existing (already correct, just remove flat fields)
- New auth tests use `McpAuth::Header { header_key: "..".into(), header_value: "..".into() }`
- Re-add `McpAuth` to import

### 5. [test_live_mcp.rs](crates/server_app/tests/test_live_mcp.rs)

Add `"auth": { "type": "public" }` to all JSON create/update payloads. Add auth-specific multi-step test.

### 6. Run and verify backend

- `cargo test -p routes_app`
- `cargo test -p server_app`
- `cargo fmt --all`

## Frontend changes

### 7. [useMcps.ts](crates/bodhi/src/hooks/useMcps.ts)

Update TypeScript types to match nested shape:

```typescript
interface McpAuthPublic { type: 'public' }
interface McpAuthHeader { type: 'header'; header_key: string; header_value: string }
type McpAuth = McpAuthPublic | McpAuthHeader;

interface CreateMcpRequest  { ...; auth: McpAuth }
interface UpdateMcpRequest  { ...; auth?: McpAuth }   // omit = keep existing
interface FetchMcpToolsRequest { ...; auth: McpAuth }
```

### 8. [page.tsx](crates/bodhi/src/app/ui/mcps/new/page.tsx)

- `handleFetchTools`: send `auth: { type: 'header', header_key: ..., header_value: ... }` or `auth: { type: 'public' }`
- `onSubmit` create path: always include `auth` object
- `onSubmit` edit path: omit `auth` to keep existing, or include to change
- Form fields (`auth_header_key`, `auth_header_value` in Zod schema) stay the same -- only the submission shape changes

### 9. [mcps.ts MSW handlers](crates/bodhi/src/test-utils/msw-v2/handlers/mcps.ts)

`mockMcp` already has `auth_type`, `auth_header_key`, `has_auth_header_value` on the response -- these stay unchanged (response shape is not affected).

### 10. Run frontend tests + regenerate

- `npm run test` from `crates/bodhi`
- `cargo run --package xtask openapi` to regenerate spec
- `make build.ts-client` to regenerate TypeScript types

