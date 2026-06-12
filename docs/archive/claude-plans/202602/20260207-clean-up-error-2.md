# Plan: Crate-by-Crate Cleanup Templatized Prompts

## Context

Before starting a new feature, we want to systematically clean up each crate in the BodhiApp workspace. The cleanup proceeds upstream-to-downstream, one crate per Claude Code session. Additionally, we want to optimize the repo for Claude Code by creating reusable skills/commands/agents.

This plan produces **three files** in `ai-docs/features/20260207-cleanup/`:

1. **`crate-cleanup-prompt.md`** - Templatized prompt launched per-crate
2. **`claude-code-optimization-prompt.md`** - One-time prompt for Claude Code optimization
3. **`README.md`** - Processing order and usage instructions

## Crate Processing Order

```
 1. errmeta_derive     -> crates/errmeta_derive
 2. objs               -> crates/objs
 3. llama_server_proc  -> crates/llama_server_proc
 4. services           -> crates/services
 5. commands           -> crates/commands
 6. server_core        -> crates/server_core
 7. auth_middleware     -> crates/auth_middleware
 8. routes_oai         -> crates/routes_oai
 9. routes_app         -> crates/routes_app
10. routes_all         -> crates/routes_all
11. server_app         -> crates/server_app
12. lib_bodhiserver    -> crates/lib_bodhiserver
13. bodhi              -> crates/bodhi/src-tauri
```

**Skipped**: async-openai/*, xtask, lib_bodhiserver_napi, integration-tests, ci_optims

## Key Decisions

- Glob re-exports (`pub use module::*`) stay as-is
- Module splits only on clear logical boundaries (no line-count threshold)
- Downstream breakage: small fixes in same session, large ones documented for later
- Plan-first: agent proposes plan, user approves, then executes
- Breaking changes are fine, no backwards compat requirement
- Single git commit per crate cleanup
- CLAUDE.md / PACKAGE.md updated after each crate via docs-updater agent
- TODOs and low-value tests presented to user for decision, not auto-removed

## Files to Create

### 1. `ai-docs/features/20260207-cleanup/README.md`

Usage guide with:
- Recommended sequence: run optimization prompt first, then crate cleanups in order
- Placeholder substitution table (`{{CRATE_NAME}}`, `{{CRATE_PATH}}`)
- Special case: bodhi has `CRATE_NAME=bodhi`, `CRATE_PATH=crates/bodhi/src-tauri`
- Full processing order with checkboxes for tracking

### 2. `ai-docs/features/20260207-cleanup/crate-cleanup-prompt.md`

Templatized prompt with phases:

**Phase 0 - Pre-checks**: Verify clean git, passing tests, read crate docs and Cargo.toml

**Phase 1 - Analysis & Plan** (read-only, present to user):
- 1.1 API surface audit: list all pub items, check downstream usage, flag candidates for `pub(crate)`/private
- 1.2 Dead code: find unreferenced types/functions/modules, list TODOs for user decision
- 1.3 Module structure: review for logical coherence, recommend splits only with clear boundaries
- 1.4 Dependency audit: unused deps, unnecessary externals, feature flag optimization
- 1.5 `use super::*` audit: find all in test modules, determine explicit replacements
- 1.6 Clippy suppression review: case-by-case assessment (refactor vs keep)
- 1.7 Test quality: flag low-value tests (trivial constructors, derives) without removing

**Phase 2 - Execute** (after user approval):
- Order: super::* fixes -> dead code -> visibility -> deps -> clippy -> module restructure -> user items
- `cargo check` after each category

**Phase 3 - Verify**: `cargo fmt --check`, `cargo test -p`, `cargo clippy -p`

**Phase 4 - Downstream impact**: Check all downstream crates compile, fix small breaks, document large ones

**Phase 5 - Commit & document**:
- Create report at `ai-docs/features/20260207-cleanup/{{CRATE_NAME}}/report.md`
- Update CLAUDE.md and PACKAGE.md via docs-updater agent
- Run `make docs.context-update`
- Single commit: `cleanup({{CRATE_NAME}}): <summary>`

### 3. `ai-docs/features/20260207-cleanup/claude-code-optimization-prompt.md`

One-time exploration prompt with:

**Phase 1 - Discovery**:
- Audit existing commands (plan-md, task-md, next-iter-plan - all deprecated)
- Audit existing agents (docs-updater)
- Analyze Makefile workflows and common command sequences
- Analyze project patterns from CLAUDE.md and crate structure
- Use Haiku sub-agents for exploration

**Phase 2 - Propose** new commands/agents:
- Required: crate-cleanup command, cross-crate-impact command, doc-regen command, test-audit command
- Optional: dependency audit, API surface report, crate health dashboard
- For each: type (command vs agent), name, purpose, trigger, inputs, outputs

**Phase 3 - Create**: Write the actual .claude/commands/ and .claude/agents/ files

**Phase 4 - Deprecated cleanup**: Remove plan-md, task-md, next-iter-plan after confirming replacements

**Phase 5 - Update CLAUDE.md**: Add "Available Commands" section

## Verification

After execution:
- All three files exist in `ai-docs/features/20260207-cleanup/`
- crate-cleanup-prompt.md contains `{{CRATE_NAME}}` and `{{CRATE_PATH}}` placeholders
- README.md has the full processing order
- claude-code-optimization-prompt.md references `.claude/commands/` and `.claude/agents/`
- No actual crate changes made (prompts are templates for future sessions)
