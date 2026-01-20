# Kickoff Prompt: Toolset Multi-Instance - objs + services Layer

Copy the prompt below to start a new chat session.

---

## Prompt

```
We are implementing multi-instance toolset support for BodhiApp. Planning is complete - context files are in @ai-docs/claude-plans/toolset-instances-*.md

Read these files in order:
1. @ai-docs/claude-plans/toolset-context.md - Implementation constraints & preferences
2. @ai-docs/claude-plans/toolset-objs.md - Domain objects layer spec
3. @ai-docs/claude-plans/toolset-services.md - Services layer spec

## Your Task

Replan and implement the **objs + services** layers only. This is a foundational layer - routes and UI will be done in subsequent sessions.

## Approach

1. **Analyze current code** - Read the existing files referenced in the specs
2. **Ask focused questions** - Clarify any ambiguities specific to these layers
3. **Create implementation plan** - Layer-specific, test-driven
4. **Implement with tests** - Each change should have corresponding tests

## Key Constraints (from context file)

- No backwards compatibility - modify in-place
- Modify migration 0007 directly, don't create new migration files
- Rename `toolset_id` → `toolset_type` in schema and code
- `id` changes from INTEGER to TEXT (UUID)
- Add `name`, `description` columns
- Unique constraint: `(user_id, name)` not `(user_id, toolset_id)`

## Files in Scope

**objs crate:**
- `crates/objs/src/toolsets.rs` - New types, validation
- `crates/objs/src/errors.rs` - New error variants

**services crate:**
- `crates/services/migrations/0007_toolsets_config.up.sql`
- `crates/services/migrations/0007_toolsets_config.down.sql`
- `crates/services/src/db/objs.rs` - Row struct
- `crates/services/src/db/service.rs` - CRUD methods
- `crates/services/src/tool_service.rs` - Business logic

## Out of Scope (for this session)

- Routes/API endpoints (routes_app crate)
- Auth middleware (auth_middleware crate)
- Frontend (bodhi crate)

## Questions to Consider

Before implementing, clarify:
1. Should validation functions live in objs or services?
2. How should ToolService trait changes be structured for incremental testing?
3. What's the test strategy for DbService methods with new schema?

## Output Expectations

- Focused plan for objs + services only
- Test-first approach where possible
- Run `cargo fmt` after Rust changes
- Run `cargo test -p objs` and `cargo test -p services` to verify

Start by reading the context files, then ask me questions or present your implementation plan.
```

---

## Notes for User

- The AI will read the context files and understand constraints
- It should ask focused questions about objs/services layer specifics
- Implementation will be isolated to foundational layers
- Routes layer session can assume objs/services are complete and tested
