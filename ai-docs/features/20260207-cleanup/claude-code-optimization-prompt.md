# Claude Code Optimization for BodhiApp

One-time exploration and setup of reusable Claude Code commands and agents optimized for the BodhiApp project. Run this before starting crate cleanups.

## Phase 1 — Discovery

Audit existing tooling and project patterns to understand what exists and what's needed.

### 1.1 Existing Commands

Audit the commands in `.claude/commands/`:
- `plan-md` — generates plan.md files (deprecated)
- `task-md` — generates task documentation (deprecated)
- `next-iter-plan` — iteration planning (deprecated)

Determine if any functionality from these should be preserved in new commands.

### 1.2 Existing Agents

Audit agents in `.claude/agents/`:
- `docs-updater` — generates/updates CLAUDE.md and PACKAGE.md for crates

This agent is active and should be kept.

### 1.3 Makefile Workflows

Analyze the Makefile for common command sequences that could benefit from automation:
- Test commands (`make test`, `make test.backend`, etc.)
- Build commands (`make build.ui`, `make ci.build`)
- Format/lint commands (`make format`, `make format.all`)
- Documentation commands (`make docs.context-update`)
- Release commands

### 1.4 Project Patterns

Analyze CLAUDE.md and the crate structure to identify recurring development workflows:
- Cross-crate dependency patterns
- Testing patterns per crate type
- Common refactoring sequences
- API change workflows (OpenAPI regen, TypeScript client update)

Use Haiku sub-agents for parallel exploration where appropriate.

## Phase 2 — Propose

Design new commands and agents. For each, specify: type (command vs agent), name, purpose, trigger conditions, inputs, and outputs.

### Required

1. **crate-cleanup** (command)
   - Purpose: Launch a crate cleanup session using the templatized prompt
   - Inputs: crate name, crate path
   - Output: executes the cleanup workflow

2. **cross-crate-impact** (command)
   - Purpose: Analyze the downstream impact of changes to a specific crate
   - Inputs: crate name
   - Output: list of dependent crates, pub items used downstream, potential breakage

3. **doc-regen** (command)
   - Purpose: Regenerate CLAUDE.md and PACKAGE.md for a crate, then run `make docs.context-update`
   - Inputs: crate name or path
   - Output: updated documentation files

4. **test-audit** (command)
   - Purpose: Analyze test quality for a crate — flag low-value tests, missing coverage areas
   - Inputs: crate name
   - Output: test quality report

### Optional (propose if valuable)

5. **dependency-audit** (command)
   - Purpose: Check for unused, outdated, or duplicate dependencies
   - Inputs: crate name or workspace-wide
   - Output: dependency report

6. **api-surface** (command)
   - Purpose: Generate a report of all public items in a crate and their downstream consumers
   - Inputs: crate name
   - Output: API surface report

7. **crate-health** (command)
   - Purpose: Dashboard combining test results, clippy warnings, dependency status, doc coverage
   - Inputs: crate name
   - Output: health summary

Present all proposals to the user for approval before creating files.

## Phase 3 — Create

After user approval, create the command and agent files:

- Commands go in `.claude/commands/<name>.md`
- Agents go in `.claude/agents/<name>.md`

Follow the existing patterns observed in the discovered files. Each file should include:
- Clear description of purpose and when to use
- Input parameters
- Step-by-step instructions
- Expected outputs

## Phase 4 — Deprecated Cleanup

After confirming the new commands adequately replace deprecated ones:

1. Remove `.claude/commands/plan-md.md`
2. Remove `.claude/commands/task-md.md`
3. Remove `.claude/commands/next-iter-plan.md`

Only remove after user confirms the replacements are adequate.

## Phase 5 — Update CLAUDE.md

Add an "Available Commands" section to the project root CLAUDE.md documenting:
- Each new command with its name, purpose, and usage
- Each agent with its name and purpose
- Any removed/deprecated commands
