# docs/archive/ — CLAUDE.md

**⚠️ FROZEN HISTORICAL SNAPSHOT — NOT CURRENT TRUTH.**

Everything under `docs/archive/` is a point-in-time record: it was accurate when written and is deliberately **never updated**. Do **not** load it to learn how the system works today — it will mislead you. Current truth lives in `docs/` (outside `archive/`) and in `crates/<crate>/CLAUDE.md`.

## When to read it
Only for **historical research** — "how did X evolve?", "what was the plan/spec behind this change?", "why was this decision made?". It serves as a changelog/timeline, not a reference.

Assume any technical claim here may be stale (e.g. references to the `objs` crate before it merged into `services`, Next.js before the Vite migration, pre-SeaORM patterns, removed non-authenticated mode). Verify against live code before acting on anything found here.

## Contents
| Folder | What |
|---|---|
| `specs/<yyyymm>/` | Dated feature specs (regrouped by month) |
| `claude-plans/<yyyymm>/` | Claude Code implementation plans + review artifacts |
| `features/` | Completed/implemented/active feature stories (early build-out) |
| `knowledge-transfer/` | Dated completion writeups and migration notes |
| `kiro-specs/` | Salvaged specs from the removed `.kiro/` tooling (all shipped) |
| `misc/` | Old top-level prompts, the former `99-archive/`, and archival research |

New plans are authored under `docs/claude-plans/<yyyymm>/`; once an effort is done, its plan ages into this archive.
