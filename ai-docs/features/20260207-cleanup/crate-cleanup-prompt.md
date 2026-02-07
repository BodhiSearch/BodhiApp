# Crate Cleanup

Clean up the crate specified by `$CRATE_NAME`. Follow each phase in order. Do not make changes until Phase 2, and only after user approval of the Phase 1 plan.

**`$CRATE_NAME` will be provided by the user in a follow-up message** (e.g., `$CRATE_NAME=services`).

Derive the crate path as follows:
- Default: `crates/$CRATE_NAME`
- Special case: if `$CRATE_NAME=bodhi`, the crate path is `crates/bodhi/src-tauri`

## Phase 0 — Pre-checks

1. Verify clean git state (`git status` shows no uncommitted changes relevant to this crate)
2. Run `cargo test -p $CRATE_NAME` and confirm tests pass
3. Read the crate's CLAUDE.md and PACKAGE.md if they exist
4. Read `<crate-path>/Cargo.toml` to understand dependencies and features

## Phase 1 — Analysis & Plan (read-only)

Perform all analysis without making any changes. Present findings to the user for approval before proceeding.

### 1.1 API Surface Audit

- List all `pub` items (types, functions, traits, constants) exported from the crate
- For each `pub` item, check if it is used by any downstream crate (search the workspace)
- Flag items that could be reduced to `pub(crate)` or private
- Note: glob re-exports (`pub use module::*`) stay as-is

### 1.2 Dead Code

- Find unreferenced types, functions, modules, and constants
- List any TODO/FIXME/HACK comments for user decision (do not auto-remove)
- Identify unused imports and dead feature flags

### 1.3 Module Structure

- Review module organization for logical coherence
- Recommend splits only where there are clear logical boundaries (no line-count thresholds)
- Flag modules that mix unrelated concerns

### 1.4 Dependency Audit

- Identify unused dependencies in Cargo.toml
- Flag unnecessary external dependencies that could use std alternatives
- Check feature flags for optimization opportunities (e.g., unused features pulled in)

### 1.5 `use super::*` Audit

- Find all `use super::*` in `#[cfg(test)]` modules
- Determine the explicit imports needed to replace each occurrence
- Reference: project convention is to avoid `use super::*` in test modules

### 1.6 Clippy Suppression Review

- Find all `#[allow(...)]` and `#[cfg_attr(..., allow(...))]` attributes
- For each suppression, assess case-by-case: can the code be refactored to remove it, or is the suppression justified?
- Present recommendations

### 1.7 Test Quality

- Flag low-value tests: trivial constructor tests, derive macro tests, standard serialization round-trips
- Present these to the user for decision — do not auto-remove
- Note tests that lack assertions or test trivial behavior

### Present Plan

Summarize all findings in a structured plan with:
- Items grouped by category (visibility, dead code, deps, etc.)
- Each item marked with proposed action
- Estimated downstream impact for visibility changes
- Ask user for approval before proceeding to Phase 2

## Phase 2 — Execute (after user approval)

Apply changes in this order, running `cargo check -p $CRATE_NAME` after each category:

1. **`use super::*` fixes** — Replace with explicit imports in test modules
2. **Dead code removal** — Remove user-approved dead code items
3. **Visibility reductions** — Change `pub` to `pub(crate)` or private as approved
4. **Dependency cleanup** — Remove unused deps, optimize features
5. **Clippy suppression fixes** — Refactor code to remove justified suppressions
6. **Module restructuring** — Apply approved module splits/reorganizations
7. **User-requested items** — Any additional changes from user feedback

After all changes: run `cargo check -p $CRATE_NAME` to verify the crate compiles.

## Phase 3 — Verify

Run all three checks and fix any issues:

```bash
cargo fmt --all -- --check
cargo test -p $CRATE_NAME
cargo clippy -p $CRATE_NAME -- -D warnings
```

## Phase 4 — Downstream Impact

1. Check all downstream crates compile: `cargo check --workspace`
2. For small downstream breakages (e.g., import path changes): fix them in this session
3. For large downstream breakages: document them in the report for a future session
4. Run `cargo test --workspace` to verify no regressions

## Phase 5 — Commit & Document

### Report

Create a cleanup report at `ai-docs/features/20260207-cleanup/$CRATE_NAME/report.md` containing:
- Summary of changes made
- Items deferred or skipped (with reasons)
- Downstream crates affected
- Any remaining TODOs

### Documentation

- Update CLAUDE.md and PACKAGE.md for the crate using the docs-updater agent
- Run `make docs.context-update` to update symlinks

### Commit

Create a single commit with message:
```
cleanup($CRATE_NAME): <one-line summary of key changes>
```
