# Fix Intermittent CI Build Failures from File Lock Contention

## Context

The `ci.build-only` CI job intermittently fails with "Timeout waiting for llama server bin lock". The root cause is a build sequencing issue:

1. Step 3 builds `llama_server_proc` WITHOUT `--all-features`
2. Step 4 builds all packages WITH `--all-features`, causing Cargo to **re-run** `llama_server_proc/build.rs` (different feature set triggers rebuild)
3. The re-run acquires an exclusive lock during llama binary compilation
4. Step 5 builds `bodhi` whose build.rs tries a shared lock and times out after 180s

Additionally, the frontend is built twice (once in Makefile, once in `lib_bodhiserver/build.rs` due to `is_ci()` check), and commands use `;` chaining so failures cascade.

A secondary issue: the current `find_latest_binary_release` in `build.rs` picks the first release with non-empty assets. The BodhiSearch/llama.cpp fork has BOTH `llama-server.yml` releases (tag prefix `server-`, assets like `llama-server--{target}--{variant}`) AND upstream-style `release.yml` releases (tag `b{N}`, assets like `llama-{tag}-bin-*.tar.gz`). If an upstream-style release is newer, the build selects it, finds no matching assets for the expected prefix, and fails.

## Changes

### 1. Fix `Makefile.ci.mk` — `ci.build-only` target

**File**: `Makefile.ci.mk` (lines 17-23)

- Remove `cd crates/bodhi && npm install && npm run build;` — redundant, `lib_bodhiserver/build.rs` handles this on CI
- Add `--all-features` to `cargo build -p llama_server_proc` — prevents Cargo from re-running build.rs in step 4
- Add `--all-features` to `cargo build -p async-openai` — consistent feature set
- Switch all `;` to `&&` — fail-fast behavior

```makefile
ci.build-only: ## Build without running tests for faster CI
	cargo build --all-features -p async-openai && \
	cargo build --all-features -p llama_server_proc && \
	PACKAGES=$$(cargo metadata --no-deps --format-version 1 | jq -r '.packages[] | select(.name != "async-openai" and .name != "bodhi") | .name' | sed 's/^/-p /') && \
	cargo build --all-features $$PACKAGES && \
	cargo build --all-features -p bodhi
```

### 2. Create `scripts/download-llama-bins.js`

**New file**: `scripts/download-llama-bins.js`

Extracts the CI_RELEASE binary download logic from `llama_server_proc/build.rs` into a Node.js script. Follows existing script conventions (CJS, `#!/usr/bin/env node`).

**CLI interface**:
```
node scripts/download-llama-bins.js --target <target> --variants <v1,v2> --extension <ext> --bin-dir <path>
```

**Behavior** (improved over current Rust logic):
- Read `GH_PAT` from env
- GitHub API: `GET /repos/BodhiSearch/llama.cpp/releases` with headers: `Authorization: Bearer {GH_PAT}`, `Accept: application/vnd.github.v3+json`, `X-GitHub-Api-Version: 2022-11-28`, `User-Agent: Bodhi-Build`
- Fetch releases with `per_page=100` (GitHub API max) to get a full page. Releases are NOT guaranteed sorted by date.
- **Filter by tag prefix `server-`** to find llama-server-standalone releases (ignores upstream `b{N}` releases). This fixes the misidentification bug.
- **Sort filtered releases by `created_at` descending** (ISO 8601 string compare works), pick the latest one with non-empty assets
- For each variant: find asset matching prefix `llama-server--{target}--{variant}`
- Download with retry (3 attempts, 5s backoff)
- If zip: extract with `unzip -o` (Unix) or `pwsh Expand-Archive` (Windows), move contents to `{bin-dir}/{target}/{variant}/`, set chmod 755 on `llama-server` only
- If non-zip: write directly to `{bin-dir}/{target}/{variant}/{execname}`, set chmod 755
- Exit non-zero on any failure

### 3. Simplify `llama_server_proc/build.rs`

**File**: `crates/llama_server_proc/build.rs`

**Replace** the `CI_RELEASE` branch (lines 102-126) with Node.js script invocation:
```rust
if env::var("CI_RELEASE").unwrap_or("false".to_string()) == "true" {
    clean_bin_dir(project_dir)?;
    let script = project_dir.join("../../scripts/download-llama-bins.js");
    let status = Command::new("node")
        .arg(&script)
        .arg("--target").arg(&build.target)
        .arg("--variants").arg(build.variants.join(","))
        .arg("--extension").arg(&build.extension)
        .arg("--bin-dir").arg(project_dir.join("bin"))
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .status()
        .context("Failed to run download-llama-bins.js")?;
    if !status.success() {
        bail!("download-llama-bins.js failed");
    }
}
```

**Remove** (only used by CI_RELEASE download path):
- `build_gh_client()`, `find_latest_binary_release()`, `fetch_llama_server()`, `try_fetch_llama_server()`, `check_zip_installation()`, `binary_exists_for_platform()`
- `GithubRelease`, `GithubAsset` structs
- Imports: `reqwest::header::{HeaderMap, HeaderValue}`, `serde::Deserialize`, `std::io`

**Keep** (used by other paths):
- Lock mechanism, `LLAMA_SERVER_BUILDS`, `LlamaServerBuild`, `set_build_envs()`, `get_target_from_platform()`, `clean_bin_dir()`
- Local dev path: `build_llama_server()`, `exec_make_target()`, `get_makefile_args()`, `clean()`
- Docker/CI_BUILD_TARGET short-circuit
- `std::thread`, `std::time::{Duration, Instant}` (used by lock timeout), `std::process::{Command, Stdio}`

### 4. Clean up `llama_server_proc/Cargo.toml` build-dependencies

**File**: `crates/llama_server_proc/Cargo.toml`

**Remove** build-dependencies no longer needed:
- `reqwest` (line 38)
- `serde_json` (line 39)
- `serde` (line 43)
- `tempfile` (line 44)

**Keep**: `anyhow`, `fs2`, `once_cell`

**Update** `[package.metadata.cargo-machete]` ignored list (line 50):
- From: `["anyhow", "fs2", "once_cell", "serde", "tempfile"]`
- To: `["anyhow", "fs2", "once_cell"]`

## Verification

1. `cargo check -p llama_server_proc` — build.rs compiles after refactor
2. `cargo build -p llama_server_proc` — local dev build still works (Make target path)
3. `node scripts/download-llama-bins.js --target aarch64-apple-darwin --variants metal,cpu --extension "" --bin-dir /tmp/test-bin` with `GH_PAT` set — download script works
4. Push branch, verify `ci.build-only` GitHub Action passes
5. `make test.backend` — no regressions
