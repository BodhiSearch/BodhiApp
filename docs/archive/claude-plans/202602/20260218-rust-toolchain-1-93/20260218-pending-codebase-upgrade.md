# Plan: Holistic Rust 1.87 → 1.93 Codebase Upgrade + Edition 2024

## Context

Upgrading Rust from 1.87.0 to 1.93.0 spans 6 releases with new lints, language features, and dependency compatibility improvements. The core toolchain files are already updated to 1.93.0. This plan covers remaining cleanup (workaround removal, dependency unpin, docs), clippy/compiler lint fixes, and Edition 2024 migration — all in a single effort with two commits.

## Already Completed (Previous Iteration)

- rust-toolchain.toml → 1.93.0
- Cargo.toml workspace rust-version → 1.93.0
- Trybuild stderr regenerated
- `#[allow(dead_code)]` on `TestArgsDelegateFalse`
- Removed unused `TestConfig` struct + `serde` import
- `Cookie<'_>` explicit lifetime fix
- GitHub Actions → 1.93.0
- Dockerfile → `rust:1.93.0-bookworm`, removed `time` crate pin workaround

---

## Commit 1: Rust 1.93 Cleanup & Compatibility

### Phase 1: Remove Legacy Workarounds

**1.1 Remove `cargo update -p deranged` CI workaround**
- **File:** `.github/actions/setup-rust/action.yml:32-35`
- Remove the "Update for deranged issue" step entirely — no longer needed with Rust 1.93

**1.2 Unpin `time` crate and update lockfile**
- **File:** `Cargo.toml:129` — change `time = "0.3.41"` to `time = "0.3"`
- Run `cargo update -p time` to get 0.3.47 in lockfile

### Phase 2: Compiler & Clippy Warning Fixes

**2.1 Run `cargo clippy --all-targets`** — fix any new warnings from 1.88-1.93 lints:
- `mismatched_lifetime_syntaxes` (1.89) — check for other instances
- `const_item_interior_mutations` (1.93)
- `function_casts_as_integer` (1.93)

**2.2 Run `make test.backend`** — verify no failures from new deny-by-default lints:
- `never_type_fallback_flowing_into_unsafe` (1.92)
- `dependency_on_unit_never_type_fallback` (1.92)
- `deref_nullptr` (1.93)

### Phase 3: Documentation Updates

- **`devops/PACKAGE.md:29`** — `rust:1.87.0-bookworm` → `1.93.0`
- **`ai-docs/context/github-workflows-context.md:112`** — "Rust 1.87.0" → "1.93.0"
- **`ai-docs/specs/20250929-openai-rs/MAINTENANCE.md:282`** — "1.87.0+" → "1.93.0+"
- changelogs/ — leave as-is (historical)
- llama.cpp submodule README — not our code

### Commit 1 Verification

1. `cargo clippy --all-targets` — clean
2. `cargo build --tests 2>&1 | grep warning` — only llama_server_proc build script info
3. `make test.backend` — passes
4. `grep -r "1\.87" --include="*.yml" --include="*.toml" --include="*.Dockerfile" .` — no hits

---

## Commit 2: Edition 2024 Migration

### Scope Assessment (from exploration)

- **15 workspace crates** + 2 vendored async-openai crates use `edition = "2021"`
- **48 files** touched by `cargo fix --edition` (15 Cargo.toml + 33 .rs files)
- **Risk: Very Low** — no unsafe fn body issues, no `gen` keyword conflicts, no never-type fallback reliance, no RPIT lifetime capture issues
- Only `wait_for_event!` macro in `routes_app/src/test_utils/mod.rs` uses `$e:expr` — backward-compatible change

### Migration Steps

**4.1 Run automated migration**
```bash
cargo fix --edition --all
```
This handles all mechanical changes across the workspace.

**4.2 Update edition field in all Cargo.toml files**
Change `edition = "2021"` → `edition = "2024"` in all 15 workspace crates + 2 async-openai crates.
Also update `resolver = "2"` → remove (resolver 2 is default in edition 2024).

**4.3 Verify**
- `cargo clippy --all-targets` — clean
- `make test.backend` — passes
- `cargo test -p errmeta_derive` — trybuild tests pass (may need stderr regeneration if diagnostics changed)

### Key Edition 2024 Features Unlocked

- **Let chains**: `if let Some(x) = a && let Some(y) = b { ... }`
- **Unsafe blocks in unsafe fns**: Body of `unsafe fn` no longer implicitly unsafe
- **Prelude additions**: `Future`, `IntoFuture` in prelude
- **Improved temporary scoping**: More predictable drop order

---

## Files Modified Summary

| Phase | File | Change |
|-------|------|--------|
| 1 | `.github/actions/setup-rust/action.yml` | Remove deranged workaround |
| 1 | `Cargo.toml` | Unpin `time` to `"0.3"` |
| 1 | `Cargo.lock` | `cargo update -p time` |
| 3 | `devops/PACKAGE.md` | Version ref 1.87→1.93 |
| 3 | `ai-docs/context/github-workflows-context.md` | Version ref 1.87→1.93 |
| 3 | `ai-docs/specs/20250929-openai-rs/MAINTENANCE.md` | Version ref 1.87→1.93 |
| 2 | Various `.rs` files | Clippy fixes (if any) |
| 4 | 15+ `Cargo.toml` files | `edition = "2024"` |
| 4 | ~33 `.rs` files | `cargo fix --edition` changes |
