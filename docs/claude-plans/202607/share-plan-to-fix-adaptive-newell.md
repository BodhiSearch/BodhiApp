# Fix: PBKDF2 Key Derivation Uses 1,000 Iterations

## Context

`docs/architecture/security.md` records `PBKDF2_ITERATIONS = 1000` (`crates/services/src/db/encryption.rs:13`) as an accepted risk, justified on the grounds that OWASP's 600,000 would cost ~200 ms per decrypt and that the DB-stolen-but-key-safe scenario is unlikely. Re-analysis found the justification is sound for one of the two key-provenance paths, unsound for the other, and that the performance tradeoff it rests on is avoidable entirely.

**Two master keys, not one.** `build_encryption_key` (`crates/lib_bodhiserver/src/app_service_builder.rs:362`) branches:

- **Keyring path** (desktop/Tauri — never sets `BODHI_ENCRYPTION_KEY`): `SystemKeyringStore::get_or_generate` returns 32 CSPRNG bytes (`crates/services/src/utils/keyring_service.rs:89`). Uniform 256-bit — iteration count is genuinely irrelevant, the same argument security.md already accepts for SHA-256 API tokens.
- **Env-var path** (Docker, multi-tenant cloud, Railway): `hash_key(&key)` is a single unsalted SHA-256 over an operator-supplied string (`keyring_service.rs:38`), with **no length or entropy validation** — only an equality check against one placeholder. Here 1,000 iterations is the entire work factor.

**The perf tradeoff is false.** Measured: 600k iterations costs 70 ms, not 200 ms. The per-request cost exists only because `derive_key` runs full PBKDF2 *per row* against a random per-row salt. Per-row salting buys nothing when there is exactly one master key — attacker work is per-candidate-key, not per-row. A two-tier KEK (PBKDF2 once at boot, HKDF per row) delivers the OWASP work factor at zero per-request cost, and keeps a 70 ms blocking CPU call out of tokio transaction futures.

**Secondary defect.** The placeholder guard checks `key == "your-strong-encryption-key-here"` (`app_service_builder.rs:369`) but `devops/.env.example:7` ships `your-encryption-key-here`. The guard does not match the string it exists to catch.

**Intended outcome:** OWASP-grade key derivation with no measurable per-request cost, weak `BODHI_ENCRYPTION_KEY` values rejected at boot, no schema migration, and no deployment locked out of its own app.

## Scope

Key derivation and the `BODHI_ENCRYPTION_KEY` provenance feeding it. Every other `security.md` entry (rate limiting, HSTS, SSRF, token expiry, `context_params`) is out of scope.

## The governing constraint

Changing the derivation invalidates existing ciphertext **whether or not the master key is rotated**. Cloud keeping its current key does not protect it; the moment `derive_key` changes, every previously-encrypted row is unreadable in every deployment.

That is survivable for most secrets but not for `tenants.encrypted_client_secret`. `decrypt_tenant_row` (`tenant_repository.rs:44`) decrypts eagerly on *every* tenant read (`:201, 219, 233, 378, 429`), so `auth_middleware.rs:162`, `token_service.rs:154` and `/bodhi/v1/info` (`routes_setup.rs:65`) all 500 — and `AppInitializer.tsx:105` renders a bare red alert with no login, no setup wizard, no logout. There is no in-app recovery. Two existing tests already pin this: `test_app_status_or_default_propagates_encryption_error` (`middleware/utils.rs:115`) and `test_app_info_handler_encryption_error` (`setup/test_setup.rs:673`).

Hence: **`tenants` gets a legacy read path and is migrated; nothing else does.** Every other secret becomes a clear, user-actionable "recreate this resource" error, and affected users are contacted directly.

## Settled decisions

| Decision | Choice |
|---|---|
| Keyring path | Uniform — PBKDF2 on both paths, one code path |
| Validator rule | Length floor (20) + placeholder match. **No** blocklist of `dummy-key`/`testkey`/etc. |
| Validator scope | Applies everywhere, not gated on `is_production`. Fixtures get compliant keys. |
| Version marker | `v2:` prefix on the ciphertext string — no schema migration, no new column |
| Legacy read path | **`tenants` only**, via an explicitly-named legacy function |
| All other legacy rows | Left in place; distinct error on use, resource recreated by the user |
| Legacy error | New code `db_error-legacy_encryption`, `ErrorType::UnprocessableEntity` → 422 |
| Response flags | `has_api_key` etc. left as-is (still report presence, not usability) |
| Key rotation | No previous-key fallback |
| Genuine key mismatch on `tenants` | Abort startup, naming the `client_id` |

## Data surface

**9 encrypted triples across 6 tables** (27 columns), each `(ciphertext, salt, nonce)` as base64 strings:

| Table | Triples | Treatment |
|---|---|---|
| `tenants` | `encrypted_client_secret` | **Migrated** |
| `api_model_aliases` | `encrypted_api_key` | Legacy error on use |
| `mcp_oauth_tokens` | `encrypted_access_token`, `encrypted_refresh_token` | Legacy error on use |
| `mcp_oauth_config_details` | `encrypted_client_secret`, `encrypted_registration_access_token` | Legacy error on use |
| `mcp_auth_params` | `encrypted_value` | Legacy error on use |
| `api_model_oauth_credentials` | `encrypted_access_token`, `encrypted_refresh_token` | Legacy error on use |

`encrypted_registration_access_token` is written but **never decrypted** (`mcp_repository.rs:705` only reads `.is_some()`), so it needs no error path at all — it simply becomes permanently unreadable, which nothing observes.

## Design

### Ciphertext format — no schema migration

Base64's alphabet (`A–Z a–z 0–9 + / =`) never contains `:`, so a prefix is an unambiguous discriminator. Detection is a string check, never a failed decrypt — which keeps "legacy scheme" cleanly distinguishable from "wrong key".

- **v1** (existing): bare base64. Row key = `PBKDF2-SHA256(master, row_salt, 1_000)`.
- **v2** (new): `v2:<base64>`. Row key = `HKDF-SHA256(ikm=KEK, salt=row_salt, info=b"bodhiapp:row:v2")`.

Salt and nonce columns are untouched — still 32-byte and 12-byte random per row, so v2 keeps per-row key separation.

### KEK

```
KEK = PBKDF2-SHA256(master_key, b"bodhiapp:kek:v2", 600_000) -> [u8; 32]
```

A fixed domain-separation constant is the correct KEK salt: with a single global master key a salt is a domain separator, not a security parameter, and it must be deterministic since it is needed before the DB opens.

### API shape

```rust
pub struct EncryptionKeys {
  master: Vec<u8>,   // legacy tenant decrypt only
  kek: [u8; 32],     // v2 encrypt + decrypt
}

// Refuses unprefixed input with EncryptionError::LegacyCiphertextUnsupported.
fn decrypt_api_key(keys: &EncryptionKeys, enc: &str, salt: &str, nonce: &str) -> Result<String>;

// Explicitly named, used ONLY by the tenant migration pass.
fn decrypt_api_key_legacy(master: &[u8], enc: &str, salt: &str, nonce: &str) -> Result<String>;
```

Add `hkdf` as a direct workspace dependency (already in `Cargo.lock` at 0.12.4 transitively, so no new compilation units).

### Legacy error

New `EncryptionError::LegacyCiphertextUnsupported` with `ErrorType::UnprocessableEntity`, surfaced as a new `DbError` variant (not the existing `EncryptionError(String)`, which is `InternalServer`) so the 422 and the `db_error-legacy_encryption` code survive. Propagation through `McpError::Db` (`mcps/error.rs:122`) and `LlmLibertyRefreshError::Db` (`refresh.rs:59`) is already `#[error(transparent)]`, so the code carries through unchanged.

**The one place this must be un-swallowed:** `resolve_api_key_for_alias` (`crates/routes_app/src/providers/mod.rs:18-23`) currently turns any decrypt error into `warn!` + `None`, forwarding the request upstream with no credential so the user sees a confusing provider 401. This is the most common path — chat completions against an API model — via `oai/routes_oai_chat.rs:180,273`, `oai/routes_oai_responses.rs:109`, `anthropic/routes_anthropic.rs:101`, `gemini/routes_gemini.rs:72`. It must propagate `LegacyCiphertextUnsupported` specifically. Other decrypt failures keep their current swallow behaviour to hold scope (see Follow-ups).

The other five decrypt sites (`mcp_repository.rs:741,857,1089,1123`, `llm_liberty_credentials_repository.rs:293,300`) already propagate and need no change beyond the new error type.

### Validator

Applied to the `BODHI_ENCRYPTION_KEY` **string** in the env-var branch only — the keyring branch produces 32 random bytes and has no operator input:

- Reject length < **20**.
- Reject `your-strong-encryption-key-here` (31) — the new `.env.example` placeholder.
- Reject `your-encryption-key-here` (24) — the *legacy* placeholder. It passes a 20-char floor and copies of the old file exist in the wild. Two entries in one constant; drop if you'd rather keep it to a single string.

No dev-key blocklist: a 20-char floor already rejects `dummy-key` (9), `testkey` (7), `local-dev-key` (13) and `test-encryption-key` (19) on length alone.

Surface as a new `BootstrapError` variant alongside the existing `PlaceholderValue`.

### Test-runtime hazard

**Sharpest implementation risk.** `DefaultDbService::new` is called from `test_utils/db.rs:48` and `test_utils/sea.rs:30,53`, so effectively every service test constructs a key. At 600k iterations that is +70 ms per test — minutes across the suite.

`EncryptionKeys` must expose a test constructor supplying a fixed precomputed KEK instead of deriving it, gated behind `#[cfg(any(test, feature = "test-utils"))]`. Coverage stays meaningful — both the v2 and legacy paths are still exercised; only the one-way stretch is skipped.

The hardcoded test key `b"01234567890123456789012345678901"` appears in four places (`test_utils/db.rs:47`, `test_utils/sea.rs:28`, `test_utils/sea.rs:51`, `mcps/test_helpers.rs:11`) and must stay byte-identical across all of them.

## Implementation

Each phase leaves the tree green; commit per phase.

### Phase 0 — stale-boundary cleanup (independent)

Delete the `// ---- prod deployment boundary (see ../CLAUDE.md); below not yet deployed ----` comment at `crates/services/src/db/sea_migrations/mod.rs:58`. All migrations are deployed; the sequence is honoured in full. This is the only occurrence in code, and `crates/services/src/db/CLAUDE.md` already states every migration has run against live databases, so no doc edit is needed.

**No migration file is created or modified anywhere in this plan** — the `v2:` prefix design is specifically what makes that unnecessary.

### Phase 1 — crypto core

`crates/services/src/db/encryption.rs`: add `EncryptionKeys` (+ `derive` and the test constructor), `derive_kek`, `derive_row_key`, the `V2_PREFIX` constant, `decrypt_api_key_legacy`, and `EncryptionError::LegacyCiphertextUnsupported`. `encrypt_api_key` always emits v2. Keep the 1,000-iteration constant as `PBKDF2_ITERATIONS_V1`, used only by the legacy function.

Tests: extend the existing four in-file tests, plus a v2 round-trip, an unprefixed fixture returning `LegacyCiphertextUnsupported` (not `DecryptionFailed`), `decrypt_api_key_legacy` succeeding on that same fixture, and wrong-key failure still reading as a decrypt failure on v2.

### Phase 2 — key plumbing

Replace `Vec<u8>` with `EncryptionKeys` across:
- `crates/services/src/db/default_service.rs:16` (field), `:24` (ctor), `:68-70` (accessor)
- `crates/services/src/db/db_core.rs:10` (trait method)
- `crates/services/src/test_utils/db.rs:117,125,133,160` and `:1536` (mockall block)
- 7 `DefaultDbService::new` sites: `app_service_builder.rs:282`, `test_utils/db.rs:48`, `test_utils/sea.rs:30,53`, `live_server_utils.rs:138,528,922`
- ~24 use sites across `api_alias_repository.rs`, `llm_liberty_credentials_repository.rs`, `mcp_repository.rs`, `tenant_repository.rs`, `mcp_service.rs`

All repositories are `impl ... for DefaultDbService` and reach the field directly as `self.encryption_key`; `mcp_service.rs` goes through the trait accessor. Six sites `.clone()` the key into async txn closures.

### Phase 3 — bootstrap, validation, fixture sweep

`crates/lib_bodhiserver/src/app_service_builder.rs`:
- `build_encryption_key` (`:362`) validates the env string, then derives the KEK once. Wrap the 600k derivation in `tokio::task::spawn_blocking` — the runtime is multi-threaded (precedent: `queue_service.rs:204` uses `block_in_place`), and the function is already `async` with no `.await` in it.
- Fix the placeholder constant to cover both strings.

Then the fixture sweep below.

### Phase 4 — tenant migration and legacy error surfacing

**Tenant pass.** New method on `DefaultDbService`, invoked from `app_service_builder.rs:283` immediately after `db_service.migrate().await?`. Migrations cannot do this — `Migrator::up(&self.db, None)` (`default_service.rs:59-62`) receives only a `DatabaseConnection`, never the key.

- Read `tenants` rows with a non-NULL `encrypted_client_secret`.
- Skip rows already `v2:`-prefixed (idempotent and crash-resumable) and rows with NULL triples.
- Otherwise `decrypt_api_key_legacy` → `encrypt_api_key` → write back.
- A failure here means a genuine key mismatch, not a legacy row: abort startup with each `client_id` named.
- Run on `self.db`, **not** `begin_tenant_txn`, so RLS does not hide rows. `tenants` is the tenant registry rather than a tenant-scoped table, so it likely carries no RLS policy — verify against `m20250101_000014_tenants.rs` before relying on it.

**Legacy error surfacing.** Map `LegacyCiphertextUnsupported` to the new `DbError` variant in the five propagating repositories, and un-swallow it in `providers/mod.rs:18-23`. Then regenerate the API contract: `cargo run --package xtask openapi && make build.ts-client`.

### Phase 5 — documentation

- `docs/architecture/security.md` — rewrite the entry: split the two key provenances, state the keyring acceptance in the high-entropy terms the API-token entry already uses, and move the item to Remediated.
- `devops/.env.example:7` → `your-strong-encryption-key-here`.
- `devops/PACKAGE.md:110` → replace `local-dev-key` with generated-key guidance.
- `docs/deployments/railway.md` and deployment guides → recommend `openssl rand -base64 32`, and state that changing the key makes existing secrets unrecoverable.

## Fixture and tooling sweep (Phase 3)

Validation applies everywhere, so every short key becomes compliant. Suggested value: `bodhi-integration-test-enc-key` (30 chars).

| File | Current | Length |
|---|---|---|
| `Makefile:160,167,184` (`app.run`, `app.run.pg`, `app.run.live`) | `dummy-key` | 9 |
| `devops/Makefile:142` | `local-dev-key` | 13 |
| `crates/services/src/test_utils/envs.rs:175` | `testkey` | 7 |
| `crates/server_app/tests/utils/live_server_utils.rs:75,465,839` | `testkey` | 7 |
| `crates/lib_bodhiserver/src/test_utils/app_options_builder.rs:14` | `test-encryption-key` | 19 |
| `crates/lib_bodhiserver_napi/src/test_utils/config.rs:62` | — | check |

Also grep `crates/lib_bodhiserver/tests-js/` for any key the Playwright suite sets directly.

**Developer-facing consequence:** `make app.run` uses `BODHI_HOME=~/.bodhi-dev-makefile`. Changing its key makes that home's tenant row fail the migration pass, aborting startup. Developers must delete `~/.bodhi-dev-makefile` and re-run setup after pulling. Call this out in the commit message.

## Verification

1. **Unit** — `cargo test -p services --lib`. Confirm the legacy-vs-v2 error distinction and both round-trips.
2. **Suite timing** — time `cargo test -p services --lib` before and after Phase 2. A large regression means the test KEK constructor is not being used.
3. **Backend** — `make test.backend 2>&1 | tee /tmp/backend.log` (capture, don't re-run). Both matrices matter: `mcps/test_mcp_auth_repository_isolation.rs` and `models/test_llm_liberty_credentials_repository.rs` run `#[values("sqlite","postgres")]` over the decrypt paths.
4. **Tenant upgrade, SQLite** — with a pre-change binary, complete setup so a tenant row exists; run the new binary against that same `BODHI_HOME` and confirm it boots, login works, and the row now carries `v2:`.
5. **Tenant upgrade, Postgres/RLS** — repeat against `make app.run.pg` to prove the pass sees the row. This is the step that catches the Phase 4 risk.
6. **Legacy error** — in that same upgraded home, confirm a pre-existing API model alias returns 422 `db_error-legacy_encryption` on a chat completion (**not** an upstream 401 — that would mean the un-swallow didn't take), and that an MCP server with pre-existing OAuth tokens returns the same on a proxy call.
7. **Abort path** — point the new binary at a `BODHI_HOME` whose tenant was written under a *different* key; confirm startup aborts naming the `client_id`, rather than booting into the `AppInitializer` error alert.
8. **Validator** — boot fails for a 10-char key and both placeholder strings; succeeds for a 26-char passphrase and for `openssl rand -base64 32` output.
9. **Browser** — `make app.run.live`; exercise login, an API model chat completion with a freshly entered key, and an MCP tool call.
10. **E2E** — `make build.dev-server` then `make test.e2e` from `crates/lib_bodhiserver/tests-js`. `bodhiserver_dev` decrypts a tenant at boot (`bin/bodhiserver_dev.rs:100` → `ensure_tenant`), so a fixture-key mistake surfaces here as a hard boot failure.

## Follow-ups (not in scope)

- `decrypt_tenant_row` bundles the secret with identity/status, so `auth_middleware` and `/bodhi/v1/info` decrypt a credential they never use. Splitting identity from credential reads would demote a key mismatch from fatal to login-only.
- `should_clear_session` (`auth_middleware.rs:49-57`) omits `AuthError::Tenant`, leaving users with an un-clearable stale cookie while `optional_auth_middleware` silently anonymises them.
- `providers/mod.rs:20` will still swallow *non-legacy* decrypt failures into a silent upstream 401. Propagating those too would remove a real diagnosis trap.
- `encrypted_registration_access_token` is written but never read, implying DCR client re-registration is not actually implemented.
