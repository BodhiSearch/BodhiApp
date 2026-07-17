# API Token Format v2: `sk-bodhiapp_` + checksum (hard migration)

## Context

**Why change the prefix:** We want GitHub secret scanning to alert on leaked BodhiApp API tokens. GitHub's guidance for a high-quality [identifiable secret](https://docs.github.com/en/code-security/tutorials/secret-scanning-partner-program) is three things together: a **uniquely defined prefix**, **high-entropy random**, and a **32-bit checksum**. We satisfy the first by adopting `sk-` (the industry "secret key" convention from OpenAI/Anthropic) with `bodhiapp` as the issuer segment, and the third by adding a CRC32 checksum.

**Why the checksum:** It lets a token be validated **offline, without a DB lookup** — the scanner (and our own middleware) recomputes the checksum from the token's own bytes and rejects malformed/corrupted candidates before any query. Critically, GitHub **push protection only blocks token versions it can identify "with confidence"**; a checksummed format is what raises that confidence and cuts false positives, strengthening partner-program eligibility. The checksum is **not** a security control (anyone can compute it) — security stays with the SHA-256 hash + DB lookup. It's format-integrity + scanner-confidence + cheap fast-fail.

**Why hard migration (no backwards compatibility):** There are very few production deployments and even fewer API tokens issued, so we can break existing tokens rather than carry a dual-prefix legacy path. Old `bodhiapp_…` tokens simply stop validating — no special-casing needed, because `strip_prefix("sk-bodhiapp_")` naturally fails for them. Users regenerate.

**Why `.<client_id>` suffix stays:** Multi-tenant DB rows are scoped by `client_id`; the token carries `client_id` so lookup can resolve the tenant. Unchanged.

## New token format

```
sk-bodhiapp_<random><checksum>.<client_id>
            └─ 43 ─┘└─ 6 ──┘
```
- `random` = 32 random bytes, base64url-no-pad → **43 chars**
- `checksum` = CRC32(IEEE) over the `random` string bytes, 4 bytes big-endian, base64url-no-pad → **6 chars**, appended directly to `random`
- `.<client_id>` = tenant client_id suffix (unchanged)
- Stored `token_prefix` (DB lookup key) = `sk-bodhiapp_` + first 8 chars of **`random`** (before checksum) = 20 chars. The DB column is unbounded (`m20250101_000005_api_tokens.rs:32` uses `string()` → no length cap), so no migration needed.
- `token_hash` = SHA-256 of the full token string (unchanged mechanism).

Checksum sits *between* `random` and the `.` so the existing "split on last `.`" logic and the `first-8-of-random` prefix derivation both keep working.

## Approach

### 1. Shared format constants + checksum helper (single source of truth)
The prefix is currently duplicated (a `const` in `routes_app`, inline `format!` literals in `services`). Consolidate in **`crates/services/src/tokens/`** (e.g. `token_objs.rs` or a small `token_format.rs`) and export:
- `pub const BODHIAPP_TOKEN_PREFIX: &str = "sk-bodhiapp_";`
- `pub const TOKEN_CHECKSUM_LEN: usize = 6;`
- `pub fn token_checksum(random: &str) -> String` — `crc32fast::hash(random.as_bytes())` → `to_be_bytes()` → `URL_SAFE_NO_PAD.encode(..)`.

Add `crc32fast` as a direct dependency of `services` (already present transitively in `Cargo.lock`). Both the generator and validator import these — no legacy constant (hard migration).

### 2. Generation — mint v2 format
`crates/services/src/tokens/token_service.rs:115-120`:
```rust
let random_string = general_purpose::URL_SAFE_NO_PAD.encode(random_bytes);
let checksum = token_checksum(&random_string);
let token_str = format!("{}{}{}.{}", BODHIAPP_TOKEN_PREFIX, random_string, checksum, client_id);
let token_prefix = format!("{}{}", BODHIAPP_TOKEN_PREFIX, &random_string[..8]);
```

### 3. Validation — strip new prefix, verify checksum offline, then lookup
`crates/routes_app/src/middleware/token_service/token_service.rs:17,91-105` — drop the local const, use the shared one, and insert checksum verification before the DB call:
```rust
let after_prefix = bearer_token
    .strip_prefix(BODHIAPP_TOKEN_PREFIX)          // legacy bodhiapp_ tokens fail here → InvalidToken
    .ok_or_else(|| TokenError::InvalidToken(...))?;
let dot_pos = after_prefix.rfind('.').ok_or(...)?;
let rand_and_sum = &after_prefix[..dot_pos];       // <random><checksum>
let token_client_id = &after_prefix[dot_pos + 1..];
if rand_and_sum.len() < TOKEN_CHECKSUM_LEN + 8 {   // need 8 for prefix + 6 for checksum
    return Err(TokenError::InvalidToken("Token too short".into()))?;
}
let split = rand_and_sum.len() - TOKEN_CHECKSUM_LEN;
let (random_part, checksum) = (&rand_and_sum[..split], &rand_and_sum[split..]);
if token_checksum(random_part) != checksum {       // offline reject — NO DB hit
    return Err(TokenError::InvalidToken("Invalid token checksum".into()))?;
}
let token_prefix = format!("{}{}", BODHIAPP_TOKEN_PREFIX, &random_part[..8]);
// ...existing get_api_token_by_prefix + status + hash compare unchanged...
```

### 4. Do NOT touch look-alikes
`bodhiapp_sentinel_api_key_ignored` (sentinel — `middleware/mod.rs:25,28`, `agentStore.ts:16`) and `bodhiapp_session_id` (session cookie — `auth/session_service.rs:190`) share the substring but are unrelated. Leave them.

### 5. Source-side docs / OpenAPI annotations, then regenerate
Update `sk-bodhiapp_…` examples in source, then run the pipeline (never hand-edit generated files):
- `crates/services/src/tokens/token_objs.rs:142-146` — `#[schema(example=…)]` + doc comment (use a realistic checksummed example)
- `crates/routes_app/src/tokens/routes_tokens.rs:40` — response example
- `crates/routes_app/src/shared/openapi.rs:153,213,552,553` — security-scheme description, `bearer_format("sk-bodhiapp_<token>")`, curl examples
- `crates/routes_app/src/oai/openapi.rs:42` — OAI `Authorization: Bearer` example
- `crates/services/CLAUDE.md` "API Token Format" section — document the v2 format + checksum
- Regenerate: `cargo run --package xtask openapi && make build.ts-client` (updates `openapi*.json`, `ts-client/src/**`, `dist/**`)

### 6. Human-authored guides
Refresh `docs/guides/authentication.md`, `docs/guides/bodhi-api.md`, `docs/guides/api-reference.md`, `docs/conventions/llm-resource-server.md` to `sk-bodhiapp_`. Frozen `docs/claude-plans/**` and `docs/archive/**` — leave. `getbodhi.app/out/**` is generated marketing HTML — regenerate via its own build only if you want the site updated (optional).

## Tests

**Introduce a shared test mint helper.** Many current test fixtures hand-build tokens as `format!("bodhiapp_{}.{}", …)`. Under v2 a hand-built token needs a *correct checksum* to pass validation, so add a `services` test-util (e.g. `make_api_token_string(random, client_id) -> String`) that mirrors generation (prefix + random + `token_checksum` + `.client_id`), and route validation-path fixtures through it. This keeps every test drift-proof against the format.

**Rust — validation module** (`crates/routes_app/src/middleware/token_service/test_token_service.rs`):
- **KEEP an old-format test, now asserting REJECTION** — a `bodhiapp_<random>.<client_id>` token fails `validate_bearer_token` (prefix strip fails → `InvalidToken`). This proves the hard migration.
- **ADD a new-format acceptance test** — a properly checksummed `sk-bodhiapp_…` token (via the helper) validates and resolves the row.
- **ADD a checksum-tamper test** — flip a char in `random` so the checksum no longer matches; assert rejection with the checksum error **and that no DB lookup occurs** (offline reject).
- **ADD malformed cases** — missing checksum / `rand_and_sum` shorter than `CHECKSUM_LEN + 8` → `InvalidToken`.

**Rust — generation** (`crates/services/src/tokens/test_token_service.rs:133,141,146,156`): assert minted token `starts_with("sk-bodhiapp_")`, round-trips through `token_checksum` (recompute over the extracted random == embedded checksum), and `token_prefix` == `sk-bodhiapp_` + 8 random chars.

**Rust — other fixtures** to migrate to the helper / new prefix: `middleware/auth/test_auth_middleware.rs:687,751,814,881`, `tokens/test_tokens_crud.rs:67,242-246,313-314,321,384,532`, `test_utils/router.rs:203-207`, `services/tokens/test_token_repository.rs:262`, `token_objs.rs:299`. Live/integration: `tests/test_live_multi_tenant.rs`, `server_app/tests/test_live_anthropic.rs`, `utils/live_server_utils.rs`.

**Frontend (Vitest):** `test-fixtures/tokens.ts:13`, `hooks/tokens/useTokens.test.ts:29,52`, `routes/tokens/index.test.tsx:36,53,191,254`, `routes/tokens/new/index.test.tsx:71`, `test-utils/msw-v2/handlers/tokens.ts` — swap fixture strings to `sk-bodhiapp_…`. UI renders `token_prefix` verbatim (no slicing), no logic changes.

**Playwright E2E:** `specs/tokens/api-tokens.spec.mjs:57`, `pages/TokensPage.mjs:321` — assert `/^sk-bodhiapp_/`. `fixtures/tokenFixtures.mjs:28,30` malformed/nonexistent → `sk-bodhiapp_`.

## Verification (end-to-end)

1. **Backend:** `make test.backend` (esp. `cargo test -p services -p routes_app`) — old-format-rejected, new-format-accepted, checksum-tamper, and generation round-trip all pass.
2. **Regen sync:** `cargo run --package xtask openapi && make build.ts-client`; `make ci.ts-client-check` shows no drift.
3. **Frontend:** `cd crates/bodhi && npm test`.
4. **E2E:** `make build.dev-server` then `make test.e2e` — token-create spec sees `sk-bodhiapp_`.
5. **Manual (Chrome, `make app.run.live`):** create a token in the UI → it renders `sk-bodhiapp_…`; use `Authorization: Bearer sk-bodhiapp_…` against `/v1/…` → 200. Corrupt one char of the random portion → 401 (checksum rejects offline). Confirm any old `bodhiapp_…` token now 401s.

## Follow-up (out of scope, ops)
Register the format with GitHub so it actually alerts: email `secret-scanning@github.com` for partner enrollment (public-repo coverage + revoke webhook), or add a custom pattern under GitHub Advanced Security. Regex anchor for the checksummed v2 format: `sk-bodhiapp_[A-Za-z0-9_-]{49}\.[A-Za-z0-9-]+`.
