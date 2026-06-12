# SeaORM Migration Review: Consolidated Index

**Commit**: `b33b1a22a` (feat: SeaORM prototype migration for ModelRepository)
**Date**: 2026-02-28
**Scope**: 236 changed files, ~16k lines across 8+ crates

## Executive Summary

The SeaORM migration is well-executed with excellent adherence to established conventions. No Critical findings. 4 Important findings to work on, 1 deferred, and 4 Nice-to-have actionable findings. All verified patterns documented in [verified-patterns.md](verified-patterns.md).

## Important Findings

| # | Finding | Report | Files | Fix Layer |
|---|---------|--------|-------|-----------|
| I-1 | MCP encrypted entities missing Pattern B views | [services-schema-review](services-schema-review.md) | entities/mcp_auth_header.rs, mcp_oauth_config.rs, mcp_oauth_token.rs | entities + service_mcp.rs |
| I-2 | `if_not_exists()` should be removed from all migrations | [services-schema-review](services-schema-review.md) | All 14 migration files | sea_migrations/ |
| I-3 | MCP encrypted fields leak to callers via list operations | [services-repos-review](services-repos-review.md) | service_mcp.rs | service_mcp.rs (depends on I-1) |
| I-4 | Missing encryption roundtrip tests for MCP auth entities | [services-tests-review](services-tests-review.md) | test_mcp_repository.rs | test_mcp_repository.rs |

## Deferred

| # | Finding | Report | Reason |
|---|---------|--------|--------|
| D-1 | Missing transaction wrapping for multi-step MCP operations | [services-repos-review](services-repos-review.md) | Not in current scope |

## Nice-to-have Findings

| # | Finding | Report | Files |
|---|---------|--------|-------|
| N-1 | strum serialize_all inconsistency (ApiFormat uses lowercase) | [objs-review](objs-review.md) | objs/src/db_enums.rs, api_model_alias.rs |
| N-2 | create_toolset returns input clone instead of inserted model | [services-repos-review](services-repos-review.md) | service_toolset.rs:59 |
| N-3 | SqlxError wrapper still present in error.rs | [services-repos-review](services-repos-review.md) | error.rs:3-18 |
| N-4 | Missing tests for unique constraint violations | [services-tests-review](services-tests-review.md) | test_*_repository.rs |

## Cross-Layer Dependencies

### I-1 + I-3: MCP Pattern B (linked findings)

I-1 identifies missing View structs in entities. I-3 is the downstream consequence: list/get methods return encrypted ciphertext.

**Fix order**:
1. Create `McpAuthHeaderView`, `McpOAuthConfigView`, `McpOAuthTokenView` in `entities/` (I-1)
2. Update `service_mcp.rs` list/get methods to use `into_partial_model::<*View>()` (I-3)
3. Update `objs.rs` Row types if domain structs change shape
4. Update service consumer code in `mcp_service/service.rs` if Row types change

**Reference**: `entities/api_model_alias.rs:34-48` (ApiAliasView) and `service_model.rs:145-153` (usage)

### I-4: Test Coverage (independent, do last)

Add tests after I-1/I-3 changes so tests validate the new patterns.

## Fix Order (Layered Development Methodology)

```
Step 1: services/sea_migrations/ ── Remove if_not_exists() (I-2)
Step 2: services/entities/       ── Add MCP View structs (I-1)
Step 3: services/service_mcp.rs  ── Use Views in list/get (I-3)
Step 4: services/test_mcp_*.rs   ── Add encryption roundtrip tests (I-4)
Step 5: cargo test -p services   ── Verify all changes
```

## Files

```
ai-docs/claude-plans/20260227-sea-orm/reviews/
├── index.md                    # This file
├── objs-review.md              # N-1: Foundation layer
├── services-schema-review.md   # I-1, I-2: Entities + migrations
├── services-repos-review.md    # D-1, I-3, N-2, N-3: Repository implementations
├── services-tests-review.md    # I-4, N-4: Test coverage gaps
└── verified-patterns.md        # All confirmed-correct patterns (no action needed)
```
