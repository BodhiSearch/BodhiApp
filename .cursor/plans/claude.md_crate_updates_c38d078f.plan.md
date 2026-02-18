---
name: CLAUDE.md crate updates
overview: "Audit and update CLAUDE.md + PACKAGE.md across 9 workspace crates (plus bodhi frontend and test_utils subdirs) using a tiered approach: deep rewrite for high-traffic crates (services, routes_app, bodhi, lib_bodhiserver_napi), light audit for the rest, fixing drift from 192 commits of recent changes including toolsets, MCP, auth consolidation, and error migration."
todos:
  - id: define-template
    content: Define flexible CLAUDE.md template convention with required (Purpose, Architecture Position with bidirectional refs) and optional sections
    status: completed
  - id: batch1-services
    content: "Deep rewrite: services CLAUDE.md + PACKAGE.md -- add MCP service, toolsets, toolset renaming, auth pattern update, service count verification"
    status: completed
  - id: batch1-routes-app
    content: "Deep rewrite: routes_app CLAUDE.md + PACKAGE.md -- add toolset routes, verify MCP routes, remove old extractors, add access request review, update OpenAPI"
    status: completed
  - id: batch1-bodhi-tauri
    content: "Deep rewrite: bodhi/src-tauri CLAUDE.md + PACKAGE.md -- verify dual-mode arch, update service init, verify file index"
    status: completed
  - id: batch1-bodhi-frontend
    content: "Deep rewrite: bodhi/src CLAUDE.md + PACKAGE.md -- add MCP/toolsets/access-request UI, fix apiClient docs, trim from 676 lines"
    status: completed
  - id: batch1-napi
    content: "Consistency update: lib_bodhiserver_napi CLAUDE.md + PACKAGE.md -- 6 months stale, update binding list, file index, build commands"
    status: completed
  - id: batch2-objs
    content: "Light audit: objs CLAUDE.md + PACKAGE.md -- add ToolsetScope, MCP types, verify error types"
    status: completed
  - id: batch2-server-core
    content: "Light audit: server_core CLAUDE.md + PACKAGE.md -- verify SharedContext/RouterState, remove FTL refs, add cross-refs"
    status: completed
  - id: batch2-auth-middleware
    content: "Light audit: auth_middleware CLAUDE.md + PACKAGE.md -- verify consolidation reflected, confirm no old extractors"
    status: completed
  - id: batch2-server-app
    content: "Light audit: server_app CLAUDE.md + PACKAGE.md -- add ExternalTokenSimulator, pre-configured client, /dev/db-reset"
    status: completed
  - id: batch2-lib-bodhiserver
    content: "Light audit: lib_bodhiserver CLAUDE.md + PACKAGE.md -- verify service count, add MCP/toolset services"
    status: completed
  - id: batch2-test-utils
    content: "Light audit: test_utils subdirs (objs, server_core, services, lib_bodhiserver) -- create missing lib_bodhiserver CLAUDE.md, fix server_core duplicate Commands"
    status: completed
isProject: false
---

# CLAUDE.md + PACKAGE.md Crate Documentation Update

## Context

192 commits in the last month introduced major features and refactors that are not reflected in crate documentation:

- **Toolsets/Tools feature**: domain model (`objs`), services, routes, UI -- complete lifecycle
- **MCP server management**: CRUD, tool discovery, OpenAPI integration (today's commit fb390126)
- **Auth consolidation**: `ExtractUserId`/`ExtractRole`/`ExtractToken` removed, replaced by `Extension<AuthContext>` (edaa9c97, f0492608)
- **FTL-to-thiserror migration**: error handling pattern changed across all crates (0350a55e)
- **ExternalTokenSimulator**: new OAuth testing without Keycloak (0ba7e77d)
- **Migration consolidation**: unreleased migrations 0006-0013 collapsed to 0006-0009
- **Toolset renaming**: `tool_type` -> `toolset_type`, `scope`/`scope_uuid` -> `toolset_type`

## Approach

**Tiered + Batched execution:**

- **Batch 1** (high-traffic, deep rewrite): `services`, `routes_app`, `bodhi/src-tauri`, `bodhi/src` (frontend), `lib_bodhiserver_napi`
- **Batch 2** (light audit + consistency): `objs`, `server_core`, `auth_middleware`, `server_app`, `lib_bodhiserver`, plus all `test_utils/` subdirectories

**For every crate, update BOTH CLAUDE.md and PACKAGE.md.**

## Flexible Template Convention

All CLAUDE.md files must have these **required** sections:

- **Purpose** -- 2-3 sentences on what this crate does
- **Architecture Position** -- where it sits in the dependency chain, with both **upstream** (crates it depends on) and **downstream** (crates that depend on it) references to their CLAUDE.md files

**Optional sections** (based on crate complexity):

- Key Domain Concepts / Architecture
- Cross-Crate Integration Patterns
- Testing Conventions
- Important Constraints
- Extension Guidelines

## Cross-Cutting Changes (apply to ALL crates where relevant)

1. **Remove old extractor references**: All mentions of `ExtractUserId`, `ExtractRole`, `ExtractToken`, `ExtractAuthContext` header-based extractors -- replace with `Extension<AuthContext>` pattern
2. **FTL-to-thiserror**: Verify no references to FTL localization files, `fluent` error patterns, or `.ftl` file paths remain. Error handling uses `#[derive(ErrorMeta)]` + thiserror
3. **Cross-crate refs**: Each CLAUDE.md must reference both upstream dependencies and downstream consumers by linking to their CLAUDE.md
4. **Rust 1.93.0**: Update any references to older Rust versions

---

## Batch 1: High-Traffic Crates (Deep Rewrite)

### 1. [crates/services/CLAUDE.md](crates/services/CLAUDE.md) + [PACKAGE.md](crates/services/PACKAGE.md)

**Current state**: 347 + 327 lines, last updated Feb 10 (before MCP, auth consolidation, toolset renaming)

**Required changes**:

- Add **MCP service** (`McpService`) -- CRUD operations, tool discovery, admin enable flow
- Add **toolsets** feature in services layer -- `ToolsetService`, dual-auth model, `ToolsetScope` struct
- Document **toolset renaming**: `name` -> `slug`, `tool_type` -> `toolset_type`
- Update auth patterns: remove references to old header-based extraction
- Add **ExternalApp** role changes (now optional) and `ResourceScope` move
- Verify service count in `AppService` (was "13+", likely higher now)
- Update `PACKAGE.md` file index for new service files (`mcp_service.rs`, toolset changes)
- Add cross-crate refs: upstream (`objs`, `server_core`), downstream (`routes_app`, `server_app`, `lib_bodhiserver`)

**Verification**: Read `src/lib.rs`, `src/mcp_service.rs` (or equivalent), toolset-related files, `AppService` struct

### 2. [crates/routes_app/CLAUDE.md](crates/routes_app/CLAUDE.md) + [PACKAGE.md](crates/routes_app/PACKAGE.md)

**Current state**: 259 + 255 lines, last updated today (fb390126) but may have been auto-updated with MCP addition only

**Required changes**:

- Add **toolset routes** documentation -- dual-auth model, `ToolsetScope`, toolset-type-based access
- Verify MCP routes are properly documented (may already be from today's update)
- Remove old extractor pattern references (`ExtractUserId`, `ExtractRole`, `ExtractToken`)
- Document `Extension<AuthContext>` as the canonical auth pattern
- Add access request review routes (JSON response, redirect flow, partial approve)
- Update OpenAPI section with new endpoints (MCP, toolsets, access request review)
- Add cross-crate refs: upstream (`services`, `auth_middleware`, `server_core`, `objs`), downstream (`server_app`, `bodhi/src-tauri`, `lib_bodhiserver`)
- Update `PACKAGE.md` file index for new route modules

**Verification**: Read `src/shared/openapi.rs` for current endpoint list, route modules, handler signatures

### 3. [crates/bodhi/src-tauri/CLAUDE.md](crates/bodhi/src-tauri/CLAUDE.md) + [PACKAGE.md](crates/bodhi/src-tauri/PACKAGE.md)

**Current state**: 178 + 377 lines, last updated Feb 7 (before auth consolidation, MCP, toolsets)

**Required changes**:

- Verify dual-mode (Tauri desktop + CLI) architecture description is still accurate
- Update service initialization if it changed with new services (MCP, toolsets)
- Add any new Tauri commands or events
- Verify `PACKAGE.md` file index against current source files
- Add cross-crate refs: upstream (all foundation crates), downstream (none -- leaf crate)

**Verification**: Read `src/lib.rs`, `src/native_init.rs`, `Cargo.toml` for current dependencies

### 4. [crates/bodhi/src/CLAUDE.md](crates/bodhi/src/CLAUDE.md) + [PACKAGE.md](crates/bodhi/src/PACKAGE.md)

**Current state**: 676 + 364 lines, last updated Feb 10

**Required changes**:

- Add **MCP UI** components -- MCP management pages, hooks (`useMcps.ts`), MSW handlers
- Add **toolsets UI** components -- toolset pages, hooks (`useToolsets.ts`, `use-toolset-selection.ts`)
- Add **access request review UI** -- partial approve, redirect flow
- Fix `apiClient.baseURL` inconsistency noted in audit (docs say `isTest ? 'http://localhost:3000' : ''`, rules say always `''`)
- Verify component directory structure against actual `src/app/ui/` layout
- Verify hook list against actual `src/hooks/` directory
- Add cross-crate refs: upstream (uses `routes_app` API), downstream (embedded in `bodhi/src-tauri` and `lib_bodhiserver`)
- Trim if possible -- 676 lines is notably longer than other CLAUDE.md files

**Verification**: Read `src/app/ui/` directory structure, `src/hooks/` directory, key component files

### 5. [crates/lib_bodhiserver_napi/CLAUDE.md](crates/lib_bodhiserver_napi/CLAUDE.md) + [PACKAGE.md](crates/lib_bodhiserver_napi/PACKAGE.md)

**Current state**: 161 + 354 lines, CLAUDE.md last updated **Aug 2025** (6 months ago!)

**Required changes**:

- Major staleness -- 6 months of changes not reflected
- Update NAPI binding list (new bindings for MCP, toolsets, dev endpoints)
- Update `PACKAGE.md` file index and code examples
- Verify build commands and test infrastructure docs
- Add cross-crate refs: upstream (`lib_bodhiserver`), downstream (none -- leaf crate)
- Note: `tests-js/` subdirectory docs already handled in previous session

**Verification**: Read `src/lib.rs`, `src/server.rs`, `Cargo.toml`, `package.json`

---

## Batch 2: Light Audit Crates

### 6. [crates/objs/CLAUDE.md](crates/objs/CLAUDE.md) + [PACKAGE.md](crates/objs/PACKAGE.md)

- Add `ToolsetScope` struct, toolset domain model types
- Add MCP-related types (if any new types were added to objs)
- Verify error types list is current (thiserror migration)
- Add cross-crate refs (upstream: none, downstream: all crates)

### 7. [crates/server_core/CLAUDE.md](crates/server_core/CLAUDE.md) + [PACKAGE.md](crates/server_core/PACKAGE.md)

- Verify SharedContext / RouterState descriptions are current
- Remove any stale FTL error references
- Add cross-crate refs

### 8. [crates/auth_middleware/CLAUDE.md](crates/auth_middleware/CLAUDE.md) + [PACKAGE.md](crates/auth_middleware/PACKAGE.md)

- Recently updated (Feb 17) during consolidation -- likely most current
- Verify old extractor references are fully removed
- Confirm `Extension<AuthContext>` is documented as THE pattern
- Add cross-crate refs

### 9. [crates/server_app/CLAUDE.md](crates/server_app/CLAUDE.md) + [PACKAGE.md](crates/server_app/PACKAGE.md)

- Add `ExternalTokenSimulator` in testing section
- Update live server test patterns (pre-configured resource client migration)
- Add `/dev/db-reset` endpoint mention
- Add cross-crate refs

### 10. [crates/lib_bodhiserver/CLAUDE.md](crates/lib_bodhiserver/CLAUDE.md) + [PACKAGE.md](crates/lib_bodhiserver/PACKAGE.md)

- Verify "all 10 business services" count is accurate (likely higher now)
- Update for new services (MCP, toolsets)
- Add cross-crate refs

### 11. test_utils Subdirectories

- **[crates/objs/src/test_utils/](crates/objs/src/test_utils/CLAUDE.md)**: Light audit
- **[crates/server_core/src/test_utils/](crates/server_core/src/test_utils/CLAUDE.md)**: Light audit, fix duplicate Commands section in PACKAGE.md
- **[crates/services/src/test_utils/](crates/services/src/test_utils/CLAUDE.md)**: Light audit
- **[crates/lib_bodhiserver/src/test_utils/](crates/lib_bodhiserver/src/test_utils/PACKAGE.md)**: **Create missing CLAUDE.md** (PACKAGE.md exists but CLAUDE.md does not)

---

## Execution Process (per crate)

1. **Read source code** -- `src/lib.rs`, key modules, `Cargo.toml` to understand current state
2. **Diff against existing docs** -- identify stale content, missing features, wrong patterns
3. **Update CLAUDE.md** -- apply template, fix accuracy, add missing sections
4. **Update PACKAGE.md** -- fix file index, update code examples, verify commands
5. **Cross-reference** -- ensure upstream/downstream links are correct and bidirectional

## What Is NOT In Scope

- `crates/mcp_client/` -- skipped, actively evolving
- `xtask/`, `crates/errmeta_derive/`, `crates/ci_optims/`, `crates/llama_server_proc/` -- not in selected 10
- `devops/`, `.github/` -- not in selected scope
- `tests-js/` docs -- already handled in previous session
- Creating new SKILL.md files -- existing CLAUDE.md + PACKAGE.md two-layer model is sufficient

