# Rust 1.87 → 1.93 Upgrade Context & Research

This file captures research findings that informed the upgrade plan in `lucky-stirring-pearl.md`.

---

## Rust Release Highlights (1.88 → 1.93)

### Rust 1.88.0 (June 26, 2025)
- **Let chains** stabilized (Edition 2024 only): `if let Some(x) = a && let Some(y) = b { ... }`
- **Naked functions**: `#[unsafe(naked)]` for full assembly control
- **Boolean cfg literals**: `#[cfg(true)]` / `#[cfg(false)]`
- **New APIs**: `HashMap::extract_if`, `HashSet::extract_if`, `Cell::update`, slice chunking methods
- **New lints**: `dangerous_implicit_autorefs` (warn), `invalid_null_arguments` (warn)
- **Cargo**: Automatic garbage collection of unused downloads
- **Breaking**: `#[bench]` without custom_test_frameworks is hard error; LLVM min → 19

### Rust 1.89.0 (August 7, 2025)
- **Inferred const generics**: `_` usable as const generic argument
- **`repr(u128)`/`repr(i128)`** for enums
- **AVX-512** target features stabilized
- **New APIs**: `File::lock`/`unlock` (cross-platform), `Result::flatten`, `OsString::leak`
- **New lints**: `mismatched_lifetime_syntaxes` (warn) — **this is what triggered our Cookie fix**
- `dangerous_implicit_autorefs` upgraded to **deny**
- **Clippy feature freeze** started (12 weeks, through Sept 18)
- **Breaking**: x86_64-apple-darwin begins Tier 1 → Tier 2 transition

### Rust 1.90.0 (September 18, 2025)
- **LLD default linker** for `x86_64-unknown-linux-gnu` — faster CI builds
- **New APIs**: `u*::checked_sub_signed`, `CStr`/`CString` PartialEq variants
- **Const**: `f32`/`f64` floor/ceil/trunc/round now const
- **Cargo**: `cargo publish --workspace` stabilized
- **Breaking**: LLD default may expose previously-hidden link errors

### Rust 1.91.0 (October 30, 2025)
- **Pattern binding drop order** improvements
- **C-style variadic functions** for more ABIs (sysv64, win64, efiapi, aapcs)
- **New APIs**: `Path::file_prefix`, `PathBuf::add_extension`, `Duration::from_mins`/`from_hours`, `str::ceil_char_boundary`/`floor_char_boundary`, `BTreeMap::extract_if`, `core::iter::chain`, `core::array::repeat`
- **New lints**: `dangling_pointers_from_locals` (warn), `integer_to_ptr_transmutes` (warn), `semicolon_in_expressions_from_macros` upgraded to **deny**
- **Cargo**: `build.build-dir` stabilized
- **aarch64-pc-windows-msvc** promoted to Tier 1
- **Breaking**: LLVM → 21; static closures now syntax error; coroutine capture changes

### Rust 1.92.0 (December 11, 2025)
- **Safe `&raw` for union fields**
- **`#[track_caller]` + `#[no_mangle]`** combinable
- **Never type lints** upgraded to **deny**: `never_type_fallback_flowing_into_unsafe`, `dependency_on_unit_never_type_fallback`
- **New APIs**: `RwLockWriteGuard::downgrade`, `Arc::new_zeroed`, `btree_map::Entry::insert_entry`
- **Breaking**: `iter::Repeat::last`/`count` now panic; LLVM min → 20; `DerefMut for Pin` sealed

### Rust 1.93.0 (January 22, 2026)
- **`asm_cfg`**: cfg attributes in asm! blocks
- **C-style variadic** for `system` ABI
- **`-Cjump-tables=bool`** stabilized
- **New APIs**: `Vec::into_raw_parts`, `String::into_raw_parts`, `std::fmt::from_fn`, `VecDeque::pop_front_if`/`pop_back_if`, `MaybeUninit` slice methods
- **New lints**: `const_item_interior_mutations` (warn), `function_casts_as_integer` (warn), `deref_nullptr` upgraded to **deny**
- **Cargo**: `CARGO_CFG_DEBUG_ASSERTIONS` in build scripts; config include stabilized
- **Breaking**: `#[test]` on structs/traits now errors; `BTree::append` key behavior changed; musl 1.2.5 update

---

## New Deny-by-Default Lints (May Break Compilation)

| Lint | Version | Impact |
|------|---------|--------|
| `dangerous_implicit_autorefs` | 1.89 | Implicit autoref of raw pointer deref |
| `missing_fragment_specifier` | 1.89 | Unconditional error in macro_rules |
| `semicolon_in_expressions_from_macros` | 1.91 | Semicolons in macro expression position |
| `never_type_fallback_flowing_into_unsafe` | 1.92 | `!` fallback to `()` in unsafe contexts |
| `dependency_on_unit_never_type_fallback` | 1.92 | Code depending on `!` → `()` fallback |
| `invalid_macro_export_arguments` | 1.92 | Invalid `#[macro_export]` args |
| `deref_nullptr` | 1.93 | Null pointer dereference |

---

## Dependency Compatibility

### `time` crate chain
- `time` 0.3.41 was pinned because 0.3.42+ requires Rust 1.88+
- `deranged` (dependency of `time`) also had MSRV issues
- With Rust 1.93: `time` 0.3.47, `deranged` 0.5.6, `num-conv` 0.2.0, `time-core` 0.1.8, `time-macros` 0.2.27 all work
- `cargo update -p time --dry-run` confirmed these versions resolve cleanly
- The `cargo update -p deranged` workaround in `.github/actions/setup-rust/action.yml` is no longer needed

### `libc` crate
- Rust 1.93 bundles musl 1.2.5, requiring `libc >= 0.2.146`
- BodhiApp already uses `libc = "0.2.172"` — no issue

---

## Edition 2024 Migration Assessment

### Scope
- **15 workspace crates** + 2 vendored async-openai crates use `edition = "2021"`
- **48 files** affected by `cargo fix --edition` (15 Cargo.toml + 33 .rs files)
- `resolver = "2"` can be removed (default in Edition 2024)

### Risk: Very Low
- **No unsafe fn body issues** — no `unsafe fn` with unguarded operations
- **No `gen` keyword conflicts** — `gen` becomes reserved in 2024
- **No never type fallback reliance**
- **No RPIT lifetime capture issues** — no `-> impl Trait` return types affected
- **One macro with `$e:expr`**: `wait_for_event!` in `routes_app/src/test_utils/mod.rs` — backward-compatible change (now also matches `const { ... }` blocks)

### What Edition 2024 Unlocks
- **Let chains**: `if let Some(x) = a && let Some(y) = b { ... }`
- **Unsafe blocks in unsafe fns**: Body is no longer implicitly unsafe
- **Prelude**: `Future`, `IntoFuture` added
- **Temporary scoping**: More predictable drop order in `if let`

### async-openai vendored dependency
- Has its own workspace, can be migrated independently
- Only uses `$from_typ:ty` / `$to_typ:ty` fragment specifiers (not `expr`)
- One-way dependency: main workspace depends on async-openai, not vice versa

---

## CI/CD and Infrastructure

### Files Already Updated to 1.93.0
- `rust-toolchain.toml`
- `Cargo.toml` (`rust-version = "1.93.0"`)
- `.github/actions/setup-rust/action.yml` (toolchain@1.93.0)
- `.github/actions/setup-rust-docker/action.yml` (toolchain@1.93.0)
- `.github/workflows/publish-app-bindings.yml` (toolchain@1.93.0)
- `devops/app-binary.Dockerfile` (`rust:1.93.0-bookworm`, removed time pin workaround)

### Remaining Workarounds to Remove
- `.github/actions/setup-rust/action.yml:32-35` — `cargo update -p deranged` step

### Documentation with Stale 1.87 References
- `devops/PACKAGE.md:29` — `FROM rust:1.87.0-bookworm`
- `ai-docs/context/github-workflows-context.md:112` — "Rust 1.87.0"
- `ai-docs/specs/20250929-openai-rs/MAINTENANCE.md:282` — "1.87.0+"
- `changelogs/20250204-20250612-*.md` — historical, leave as-is
- `crates/llama_server_proc/llama.cpp/.devops/base-images/README.md` — submodule, not ours

### Submodule Note
`crates/llama_server_proc/llama.cpp/.devops/base-images/README.md` contains `FROM rust:1.87.0-bookworm` but is inside the llama.cpp git submodule — do not modify.

---

## Codebase Quality Notes from Exploration

- No systematic dead code or lifetime issues beyond what was already fixed
- Error handling architecture is well-structured with errmeta_derive
- Test infrastructure is modern (rstest, tokio::test, anyhow_trace)
- No patterns found that would benefit from let chains specifically (but having the option is good)
- `lazy_static` could potentially be replaced with `std::sync::LazyLock` (stabilized 1.80) but that's a separate effort
