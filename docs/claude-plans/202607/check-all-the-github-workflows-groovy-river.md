# GitHub Actions Deprecation Audit & Upgrade Plan

## Context

Recent workflow runs on GitHub (verified live via run annotations, e.g. runs `33145319260`, `33145365382` from 2026-08-28) emit active deprecation warnings:

> **Node.js 20 is deprecated.** The following actions target Node.js 20 but are being forced to run on Node.js 24: `actions/cache@v4`, `actions/checkout@v4`, `actions/setup-node@v4`, `softprops/action-gh-release@v2`, `docker/build-push-action@v5`, `docker/login-action@v3`, `docker/setup-buildx-action@v3`.

Every third-party action in `.github/workflows/*.yml` and the composite actions in `.github/actions/*/action.yml` is one or more majors behind latest stable. GitHub will eventually hard-fail Node 20 actions, so we upgrade now on our own schedule.

**User decisions (settled):**
- Upgrade `actions/*`, `docker/*`, `codecov`, `dorny/test-reporter`, `softprops`, `peter-evans` to **latest stable majors**.
- **Keep `tauri-apps/tauri-action@v0` as-is** (v1 exists but is deferred).
- Keep the existing **major-tag pin style** (`@vN`), matching current convention.
- Verification: user will run `make release.all` and `make release.docker.*` after changes land.
- Add **weekly `github-actions` ecosystem to Dependabot** to prevent future drift.

## Version Bump Table

| Action | Current | Target | Breaking-change notes (from release notes, verified) |
|---|---|---|---|
| `actions/checkout` | v4 | **v7** | Node 24 + ESM only. v6 persists creds to a separate file (transparent); v7 blocks fork-PR checkout for `pull_request_target`/`workflow_run` (not used here). |
| `actions/upload-artifact` | v4 | **v7** | Node 24 + ESM. New optional `archive: false` for single files — no change needed. |
| `actions/download-artifact` | v4 | **v8** | v5 changed path behavior only for downloads **by `artifact-ids`** — repo uses name-based downloads only (verified, no `artifact-ids` anywhere). v8: digest mismatch now errors by default (desirable); non-zipped files no longer auto-unzipped (all our artifacts come from upload-artifact, so zipped — safe). |
| `actions/setup-node` | v4 | **v7** | v5 added auto package-manager caching triggered by `packageManager` field in package.json — **no package.json in repo has that field (verified)**, so no behavior change. Custom npm cache in `setup-node` composite keeps working. |
| `actions/cache` | v4 | **v6** | Node 24 + ESM only. |
| `actions/configure-pages` | v5 | **v6** | Node 24 only. |
| `actions/deploy-pages` | v4 | **v5** | Node 24 only. |
| `actions/upload-pages-artifact` | v3 | **v5** | v4 stopped including dotfiles by default — **`getbodhi.app/out/` contains no dotfiles (verified)**; `.nojekyll` not needed for actions-based Pages deploys. v5 adds opt-in `include-hidden-files`. |
| `docker/setup-buildx-action` | v3 | **v4** | Node 24 + ESM; removed already-deprecated inputs (none used here). |
| `docker/login-action` | v3 | **v4** | Node 24 + ESM only. |
| `docker/build-push-action` | v5 | **v7** | v6 adds build-summary job output (leave enabled; disable later with `DOCKER_BUILD_SUMMARY: false` env if noisy). v7: Node 24, removes deprecated `DOCKER_BUILD_NO_SUMMARY`/`DOCKER_BUILD_EXPORT_RETENTION_DAYS` envs (not used here). |
| `softprops/action-gh-release` | v2 | **v3** | Node 24 runtime move only. |
| `codecov/codecov-action` | v4.0.1 | **v7** | v5 renamed `file`→`files` / `plugin`→`plugins` — **repo already uses `files` (verified)**; `name`, `token`, `slug`, `fail_ci_if_error`, `flags` all still valid. Un-pin from patch to major tag `@v7`. |
| `dorny/test-reporter` | v1 | **v3** | v3 = Node 24. Inputs used (`name`, `path`, `reporter: java-junit`, `fail-on-error`) unchanged. |
| `peter-evans/repository-dispatch` | v3 | **v4** | Node 24 only. |
| `tauri-apps/tauri-action` | v0 | **keep v0** | Per user decision (v1 exists, deferred). |
| `dtolnay/rust-toolchain` | 1.93.0 | keep | Pinned toolchain version by design. |
| `Swatinem/rust-cache` | v2 | keep | v2 is latest major (v2.9.2). |
| `taiki-e/install-action` | @cargo-llvm-cov | keep | Tool-name tag, intended usage. |

## Files to Change

Pure `uses:` version-string edits (pattern: bump per table above) in:

**Workflows** (`.github/workflows/`):
- `build.yml`, `build-multiplatform.yml`, `playwright.yml` — checkout, download/upload-artifact, test-reporter
- `publish-docker.yml`, `publish-docker-multiplatform.yml`, `publish-docker-mt.yml` — checkout, docker/* trio, upload/download-artifact, gh-release
- `publish-app-bindings.yml` — checkout, rust-cache (no-op), upload/download-artifact, gh-release
- `publish-ts-client.yml` — checkout, gh-release
- `release.yml` — checkout, setup-node, upload/download-artifact, gh-release (tauri-action untouched)
- `deploy-website.yml` — checkout, setup-node, cache, configure-pages, upload-pages-artifact, deploy-pages, repository-dispatch

**Composite actions** (`.github/actions/`):
- `setup-node/action.yml` — setup-node@v7, cache@v6
- `setup-models/action.yml`, `setup-playwright/action.yml` — cache@v6
- `build-and-test/action.yml` — codecov@v7 (×2), upload-artifact@v7
- `build-only/action.yml`, `bodhiserver-dev-build/action.yml`, `napi-build/action.yml` — upload-artifact@v7
- `setup-rust/action.yml`, `setup-rust-docker/action.yml` — no version changes (rust-cache/dtolnay/taiki-e stay)

**Dependabot** — `.github/dependabot.yml`: add

```yaml
 - package-ecosystem: "github-actions"
   directory: "/"
   schedule:
     interval: weekly
```

(Dependabot's `github-actions` ecosystem also scans `.github/actions/*/action.yml` referenced from workflows.)

## Non-goals

- No SHA-pinning migration (keep major tags — current convention).
- No runner label changes (`ubuntu-latest*`, `macos-latest`, `windows-latest` all fine; GitHub-hosted runners already meet the v2.327.1 minimum the Node-24 actions require).
- No tauri-action v1 migration.
- No workflow logic/refactor changes — version bumps only.

## Execution Steps

1. Apply all `uses:` bumps per the table (single pass across the files above).
2. Add the `github-actions` Dependabot ecosystem entry.
3. Sanity-check YAML: `actionlint` if installed (else careful diff review — edits are version strings only).
4. Single focused commit directly on `main` (trunk workflow), e.g. `ci: upgrade GitHub Actions to latest stable majors (Node 24)`; rebase on `origin/main` and push. No app code touched → backend/UI/E2E gates not applicable; CI itself is the test.

## Verification

1. **Push-triggered workflows**: the push to `main` runs `build.yml` and `playwright.yml` — confirm green via `gh run list`/`gh run watch`, and confirm the Node 20 deprecation annotations are gone (`gh api repos/BodhiSearch/BodhiApp/check-runs/<job-id>/annotations`).
2. **Tag-triggered release workflows**: user runs `make release.all` and `make release.docker.*` — this exercises `release.yml`, `publish-ts-client.yml`, `publish-app-bindings.yml`, `publish-docker*.yml`, which cover gh-release@v3, docker trio, artifact v7/v8 round-trips, and setup-node@v7 across macOS/Linux/Windows runners.
3. **deploy-website.yml**: verified at next website tag (`getbodhi.app/v*`) or via its `workflow_dispatch`; watch the Pages deploy step and spot-check https://getbodhi.app after deploy (upload-pages-artifact v3→v5 is the only structural jump; no dotfiles in `out/`, so safe).
4. Watch the first artifact download in `publish-docker*.yml`/`release.yml` for the new v8 digest-mismatch enforcement (expected clean; a failure there indicates a real integrity issue, not a config problem).
