# Plan: Optimize Documentation System via Parallel Sub-Agents

## Context

Project documentation files have grown beyond effective sizes for AI agent context windows. Rather than pre-deciding all changes, we launch specialized sub-agents that autonomously explore each crate, audit existing docs against source code, and produce optimized documentation following progressive disclosure patterns.

**Key insight from best practices**: CLAUDE.md should only contain things Claude can't figure out by reading code. Standard conventions, inferable patterns, dependency listings, and code examples waste context tokens. Use `file_path:line_number` references instead of inline code. Focus on non-obvious behaviors, gotchas, architectural decisions, and commands that can't be guessed.

---

## Step 0: Create MDFILES.md Convention File

Create `/MDFILES.md` (~80 lines) defining the documentation system. This is read by all sub-agents as their instruction set.

### Content of MDFILES.md

```markdown
# Documentation Convention (MDFILES.md)

## Purpose
Define conventions for all *.md documentation files in the project, optimized for
Claude Code AI coding assistant. These docs supplement what Claude can already infer
from reading source code.

## Core Principle: Only Document the Non-Obvious
For each line in a doc file, ask: "Would removing this cause Claude to make mistakes?"
If not, cut it. Claude can read source code — don't duplicate what's already there.

### HIGH-VALUE content (include)
- Commands Claude can't guess (build, test, run incantations)
- Architectural decisions and their rationale that aren't evident from code
- Non-obvious behaviors, gotchas, and foot-guns
- Cross-crate coordination patterns not visible from a single crate
- Testing instructions and preferred patterns that differ from defaults
- Domain-specific rules that override standard conventions

### LOW-VALUE content (exclude)
- Standard language conventions Claude already knows
- Dependency listings (readable from Cargo.toml/package.json)
- Code examples (use `file_path:line_number` references instead)
- Generic advice ("write clean code", "handle errors properly")
- README material (project descriptions, feature lists)
- Information that changes frequently (version numbers, status)
- File-by-file descriptions of the codebase (Claude can read files)
- Aspirational/future plans not yet implemented

## File Roles & Line Limits

| File | Role | Line Limit |
|------|------|-----------|
| Root `CLAUDE.md` | Project entry point: commands, high-level arch, crate peek | 200-300 |
| `crates/CLAUDE.md` | Workspace hub: crate index, shared Rust conventions, cross-crate patterns | 200-300 |
| `crates/<crate>/CLAUDE.md` | Crate entry point with progressive disclosure pointers | 300 max |
| `crates/<crate>/PACKAGE.md` | Implementation details, file index, API surface | No hard limit |
| Deeper `CLAUDE.md` | Sub-module entry point | 200 max |
| Deeper `PACKAGE.md` | Sub-module implementation details | No hard limit |

## Satellite Files (fixed names, per-crate as needed)
- **TESTING.md** — Test patterns, fixtures, helpers, canonical test structure
- **CONVENTION.md** — Naming, style, patterns specific to this crate
- **TECHDEBT.md** — Known issues, planned refactors, stale patterns
- **Exception for `bodhi/src/`**: Also allows COMPONENTS.md, HOOKS.md, FORMS.md

## Progressive Disclosure Header
Every CLAUDE.md must start with a companion docs section:

# <Crate Name> — CLAUDE.md
**Companion docs** (load as needed):
- `PACKAGE.md` — Implementation details and file index
- `TESTING.md` — Test patterns and fixtures
- `CONVENTION.md` — Crate-specific conventions
(list only files that actually exist)

## Writing Style
- Use `file_path:line_number` references instead of code snippets
- Keep sections scannable: short paragraphs, bullet points, tables
- Lead with the most important information
- Use emphasis (IMPORTANT, CRITICAL) sparingly for truly critical rules
- Prefer imperative mood ("Use X" not "You should use X")
```

---

## Step 1: Parallel Crate-Level Sub-Agents

Launch 5 **general-purpose** sub-agents in parallel (in isolated worktrees). Each agent autonomously explores its assigned crates and produces all documentation files.

### Shared Sub-Agent Instructions (included in every agent prompt)

```
## Your Process
1. READ `/MDFILES.md` for documentation conventions
2. For each assigned crate:
   a. READ all existing *.md files in the crate directory (and subdirectories like src/test_utils/)
   b. READ the crate's lib.rs/main.rs, mod.rs files, and major modules to understand current state
   c. CROSS-REFERENCE every claim in existing docs against actual source code:
      - Fix stale imports (especially `objs::` which was absorbed into `services`)
      - Fix references to removed/renamed types, functions, or crates
      - Remove documentation for features/patterns that no longer exist
      - Add documentation for patterns that exist in code but are undocumented
   d. RESTRUCTURE CLAUDE.md to fit within line limit:
      - Keep: purpose, architecture position, critical non-obvious rules, progressive disclosure pointers
      - Move to satellites: testing details, conventions, verbose patterns
      - Move to PACKAGE.md: implementation details, API surface, file index
      - DELETE: generic advice, inferable-from-code content, README material, code examples
        (replace code examples with file_path:line_number references)
   e. CREATE satellite files only if substantial content (>20 lines) warrants it
   f. UPDATE PACKAGE.md: fix stale references, remove dead content, add missing info
   g. ADD progressive disclosure header to CLAUDE.md

## Quality Checks Before Finishing
- Every file path referenced in docs exists in the crate (verify with Glob/Grep)
- Every type/function mentioned exists in current source code
- No `objs::` imports (absorbed into `services` crate)
- CLAUDE.md line count verified ≤ limit
- No inline code examples — use file:line references instead
- No content that Claude could figure out by reading source code
```

### Agent A: services + server_core (dedicated large crates)

**Crates**: `crates/services/` (CLAUDE.md: 537L, limit: 300), `crates/server_core/` (CLAUDE.md: 455L, limit: 300)

**Additional context**:
- `services` is the domain types + business logic hub. Also has `src/test_utils/` with CLAUDE.md (149L) and PACKAGE.md (588L, limit 200L for CLAUDE.md in deeper folders)
- `server_core` is HTTP infrastructure + LLM coordination. Also has `src/test_utils/` with CLAUDE.md (152L) and PACKAGE.md (558L)
- Both crates are significantly over the 300-line limit
- The `src/test_utils/` PACKAGE.md files may have fabricated code examples — verify every example against actual test source files
- These crates have the richest cross-crate coordination patterns — focus on what's non-obvious

### Agent B: bodhi/src (dedicated large UI crate)

**Crates**: `crates/bodhi/src/` (CLAUDE.md: 944L, limit: 300)

**Additional context**:
- This is the Next.js 14 frontend — the most over-limit file (3x)
- May use UI-specific satellite names: COMPONENTS.md, HOOKS.md, FORMS.md alongside standard satellites
- Also handle `crates/bodhi/src-tauri/` (CLAUDE.md: 211L, limit: 300) — under limit but audit freshness
- The `bodhi/src/PACKAGE.md` (462L) also needs freshness audit
- Focus on: non-obvious Next.js patterns, TypeScript client integration workflow, testing patterns that differ from defaults
- Ruthlessly cut: dependency listings (in package.json), standard React patterns, CSS config, build config

### Agent C: routes_app + auth_middleware

**Crates**: `crates/routes_app/` (CLAUDE.md: 452L, limit: 300), `crates/auth_middleware/` (CLAUDE.md: 182L, limit: 300)

**Additional context**:
- `routes_app` is the API orchestration layer — over limit at 452L
- Key patterns to preserve: AuthScope extractor, domain-specific error handling, OpenAPI registration checklist, handler naming conventions
- `auth_middleware` is under limit but needs freshness audit
- These two crates are tightly coupled — auth_middleware provides the middleware, routes_app consumes it via AuthScope
- Focus on: non-obvious error handling chains, the AuthScope pattern, testing patterns for route handlers

### Agent D: llama_server_proc + errmeta_derive + errmeta + mcp_client (Rust foundations)

**Crates**: `crates/llama_server_proc/` (113L), `crates/errmeta_derive/` (141L), `crates/errmeta/` (141L), `crates/mcp_client/` (check if docs exist)

**Additional context**:
- All under limit — focus on freshness audit and quality improvement
- These are foundation crates with minimal dependencies
- `errmeta` + `errmeta_derive` define the error infrastructure — focus on non-obvious proc-macro behavior, error code generation rules
- `llama_server_proc` manages LLM process lifecycle — focus on non-obvious process management patterns
- `mcp_client` may not have docs yet — create if the crate has meaningful patterns to document
- Pure Rust focus — no UI or HTTP concerns

### Agent E: server_app + lib_bodhiserver + lib_bodhiserver_napi (application + E2E)

**Crates**: `crates/server_app/` (258L), `crates/lib_bodhiserver/` (176L), `crates/lib_bodhiserver_napi/` (163L)

**Additional context**:
- All under CLAUDE.md limits but need freshness audit
- `server_app` has live integration tests with OAuth2 test infra — document non-obvious test patterns
- `lib_bodhiserver` has `src/test_utils/` (PACKAGE.md: 372L) that needs audit
- `lib_bodhiserver_napi` has `tests-js/` with:
  - `CLAUDE.md` (175L, limit: 200) — under limit
  - `E2E.md` (452L, limit: 200) — **over limit, must be trimmed**
  - Focus E2E.md on: non-obvious patterns, anti-patterns condensed to brief summaries, decision trees, flakiness patterns
  - Cut from E2E.md: verbose WRONG/RIGHT code examples (use file references instead), standard Playwright patterns

---

## Step 2: Consolidation Sub-Agent (after Step 1 completes)

A single **general-purpose** agent (in isolated worktree) that reads all crate-level docs and produces root-level documentation.

### Consolidation Agent Instructions

```
## Your Task
Read ALL crates/*/CLAUDE.md and crates/*/PACKAGE.md files produced by the crate-level agents.
Then create/update:

1. `/MDFILES.md` — Verify the conventions file is accurate and complete
2. `crates/CLAUDE.md` — Workspace-level hub (200-300 lines)
3. `/CLAUDE.md` — Project root entry point (200-300 lines)

## Process

### crates/CLAUDE.md (200-300 lines)
Read every crates/*/CLAUDE.md. Produce:
- Crate keyword index table: one row per crate with path + 1-line purpose + keywords
  (extract from each crate's CLAUDE.md purpose section)
- Shared Rust conventions (module organization, mod.rs rules from existing content)
- Cross-crate architecture patterns synthesized from individual crate docs:
  - Error layer separation (errmeta -> services -> routes_app)
  - AuthScope extractor flow
  - Service initialization order
- Shared testing conventions (rstest patterns, test file organization, assertion style)
  - Extract from individual crate TESTING.md files, keep only what's shared across crates

### /CLAUDE.md (200-300 lines)
Read the current root CLAUDE.md. Restructure to keep ONLY:
- Development Commands (essential — build, test, format, run commands)
- Layered Development Methodology (the upstream-to-downstream workflow)
- Technology Stack (brief: 5 lines max)
- Key Crates Structure (brief tier listing with dependency arrows, NOT the full table — that's in crates/CLAUDE.md)
- Reference to MDFILES.md for documentation conventions
- Important Notes: Development Guidelines, Architectural Patterns (brief, non-obvious only)
- Critical UI Development Workflow (the build.ui-rebuild requirement)
- Backwards Compatibility notes

REMOVE from root CLAUDE.md:
- Detailed crate keyword index table (moved to crates/CLAUDE.md)
- Architectural Decision Patterns section (moved to crates/CLAUDE.md)
- Testing Practices details (moved to crates/CLAUDE.md)
- Frontend Structure (redundant with bodhi/src/CLAUDE.md)
- Key Features (README material)
- Development Patterns (duplicated by Architectural Patterns)
- Security and Deployment Considerations (duplicated by crate docs)

## Quality Checks
- Root CLAUDE.md + crates/CLAUDE.md together provide complete navigation to any crate
- No content duplicated between root and crates/ CLAUDE.md
- Every crate listed in index table has a valid CLAUDE.md at referenced path
- Line counts within limits
```

---

## Step 3: Verification

After consolidation, run directly (not via sub-agent):
1. `wc -l` all CLAUDE.md files — verify limits
2. `grep -r "objs::" crates/*/CLAUDE.md crates/*/PACKAGE.md` — catch stale imports
3. Verify all progressive disclosure file references resolve to existing files
4. Spot-check: read 2-3 crate CLAUDE.md files, follow every pointer, confirm satellites exist and have content

---

## Execution Summary

| Step | What | Agent Type | Count | Parallelism |
|------|------|-----------|-------|-------------|
| 0 | Create MDFILES.md | Direct edit | — | — |
| 1A | services + server_core | general-purpose | 1 | Parallel with 1B-1E |
| 1B | bodhi/src + bodhi/src-tauri | general-purpose | 1 | Parallel with 1A,1C-1E |
| 1C | routes_app + auth_middleware | general-purpose | 1 | Parallel with 1A-1B,1D-1E |
| 1D | llama_server_proc + errmeta* + mcp_client | general-purpose | 1 | Parallel with 1A-1C,1E |
| 1E | server_app + lib_bodhiserver* + E2E | general-purpose | 1 | Parallel with 1A-1D |
| 2 | Root CLAUDE.md + crates/CLAUDE.md | general-purpose | 1 | After Step 1 |
| 3 | Verification | Direct | — | After Step 2 |
