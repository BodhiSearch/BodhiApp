# Prompt: Commands & Services Layer Structural Analysis & Consolidation

## Objective

Analyze BodhiApp's `commands` and `services` crates holistically and bottom-up to produce a consolidation plan that removes structural debt, makes code organization uniform and predictable, and aligns module/type/trait boundaries with the application's natural domain boundaries.

The routes layer has already been reorganized into domain-coherent folders (see `ai-docs/claude-plans/20260208-cleanup-routes.md`). This analysis applies the same rigor to the layers beneath: `services` (business logic) and `commands` (CLI orchestration).

## Prior Art

- **Error handling refactoring** (`ai-docs/claude-plans/20260207-error-cleanup.md`, `20260207-error-cleanup-2.md`) established uniform error patterns across the workspace.
- **Route reorganization** (`ai-docs/claude-plans/20260208-cleanup-routes.md`) reorganized `routes_app` from flat files into domain folders (`routes_auth/`, `routes_users/`, `routes_models/`, `routes_api_models/`, `routes_toolsets/`).
- The domain groupings discovered in routes may or may not map cleanly to the services layer — discover the actual boundaries from the code, don't assume.

## Analysis Structure

### Step 1: Map the Current Landscape

Before proposing changes, build a complete picture:

**For `services/`:**
- What modules exist and what does each one do?
- How large is each module (rough line counts, number of types/traits/functions)?
- What are the dependency relationships between modules within the crate?
- How does the `db/` sub-crate relate to the service layer — is it a clean data access layer, or does business logic leak into it?
- What does the `AppService` registry look like and how is it consumed?

**For `commands/`:**
- What modules exist and what does each one do?
- How do commands relate to services — thin wrappers, or significant orchestration logic?
- Are there patterns that repeat across command implementations?
- Where does business logic live — in commands, in services, or split awkwardly between them?

### Step 2: Analyze Module Boundaries

For each module in both crates, assess:

#### Organization & Cohesion
- Does each module have a single clear responsibility?
- Are there modules doing work for multiple unrelated domains?
- Are there modules that are too large and should be split?
- Are there modules that are too small and should merge with neighbors?
- Are related types co-located or scattered across files?

#### Naming Consistency
- Do modules, types, traits, and functions follow consistent naming conventions?
- Are there naming patterns that diverge from what the routes layer now uses?
- Are error types named consistently with the patterns established in the error cleanup?

#### Type Placement
- Are DTOs, request/response types, and domain objects in the right module?
- Are there types in `services` that belong in `objs`? Or vice versa?
- Are there types shared between `commands` and `services` that have unclear ownership?

#### Trait Design
- Are service traits well-scoped or do they have grab-bag interfaces?
- Are mock boundaries well-defined for testing?
- Are there traits with methods that don't belong together?
- Are there patterns of trait methods that always get called together, suggesting a missing abstraction?

#### Error Boundaries
- Do error types follow the domain-specific enum pattern from the error cleanup?
- Are there error types that are too broad (catching too many unrelated failures)?
- Are there error types that are too narrow (unnecessary fragmentation)?
- Is error propagation between services clean or does it involve unnecessary wrapping?

### Step 3: Analyze Cross-Cutting Patterns

Look across both crates for:

- **Re-export hygiene**: Are `lib.rs` and `mod.rs` files organized or flat dumps?
- **Test organization**: Are tests inline, in separate files, or in test directories? Is it consistent?
- **Feature flag usage**: Are `test-utils` and other feature flags used consistently?
- **Module file structure**: flat files vs directory modules — is the choice consistent and appropriate for module size?
- **Import patterns**: Are there circular-feeling dependencies or modules that import too widely?

### Step 4: Classify Findings

For each issue found, classify it:

1. **Misplaced code** — type/function lives in wrong module or crate
2. **Fragmented domain** — one domain's code scattered across unexpected locations
3. **Naming inconsistency** — similar things named differently
4. **Redundant abstractions** — unnecessary indirection or wrapper types
5. **Missing abstractions** — repeated patterns that should be unified
6. **Oversized modules** — files doing too much, should be split
7. **Undersized modules** — files too small, should merge into neighbors
8. **Leaky boundaries** — module exposes internals or depends on things it shouldn't
9. **Inconsistent patterns** — same task done differently in different places

### Step 5: Produce Consolidation Plan

Group findings into phases ordered by:
1. **Foundation first** — changes to `services` before `commands`
2. **Risk-adjusted** — smaller, safer changes before large restructurings
3. **Domain-coherent** — changes that make one area fully clean before moving to the next

For each phase, specify:
- What changes
- Why (which debt pattern it addresses)
- What files move/merge/split
- What renames occur
- What tests need updating
- Verification command

## Constraints

- Do NOT propose changes to `objs` — it was already organized
- Do NOT propose changes to `routes_app`, `routes_oai`, `routes_all` — already reorganized
- Do NOT propose changes to frontend, build system, CI, or Docker
- Do NOT propose new crates — prefer reorganizing within existing crates
- Do NOT propose trait redesigns unless the current design causes concrete problems demonstrated by code examples
- Preserve all public API behavior (CLI commands, service interfaces consumed by routes)
- Each phase must be independently compilable and testable
- Prefer moves and renames over rewrites — the code works, it just needs reorganization

## Output Format

Produce the analysis as a plan document with:
1. Module inventory (table showing modules → responsibility → size → domain)
2. Findings table (issue, classification, severity, affected files)
3. Phased consolidation plan (as described in Step 5)
4. Summary metrics (number of file moves, renames, merges, splits per phase)

## How to Execute This Analysis

Read the following files first to build context:
- `crates/services/CLAUDE.md` and `crates/commands/CLAUDE.md` (if they exist)
- `crates/services/src/lib.rs` and `crates/commands/src/lib.rs` for module structure
- `crates/services/Cargo.toml` and `crates/commands/Cargo.toml` for dependency graph
- Every module file in `services/src/` to understand boundaries and responsibilities
- Every module file in `commands/src/` to understand boundaries and responsibilities
- The `services/src/db/` directory structure for data access layer analysis
- The completed route reorganization plan (`ai-docs/claude-plans/20260208-cleanup-routes.md`) for context on domain boundaries discovered during route cleanup
- The completed error cleanup plans for context on error patterns

Then do targeted reads of specific modules where complexity or fragmentation appears.
