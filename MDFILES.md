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

```
# <Crate Name> — CLAUDE.md
**Companion docs** (load as needed):
- `PACKAGE.md` — Implementation details and file index
- `TESTING.md` — Test patterns and fixtures
- `CONVENTION.md` — Crate-specific conventions
(list only files that actually exist)
```

## Writing Style
- Use `file_path:line_number` references instead of code snippets
- Keep sections scannable: short paragraphs, bullet points, tables
- Lead with the most important information
- Use emphasis (IMPORTANT, CRITICAL) sparingly for truly critical rules
- Prefer imperative mood ("Use X" not "You should use X")
