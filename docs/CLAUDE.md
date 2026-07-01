# docs/ — CLAUDE.md

Curated, current-truth documentation for BodhiApp, organized for progressive disclosure. Load a sub-area's `CLAUDE.md` only when working in that area.

**Canonical-source rule**: crate-level docs live at `crates/<crate>/CLAUDE.md` and `crates/<crate>/PACKAGE.md` — they are the source of truth for code-level detail and are **not duplicated here**. This tree links to them.

**Archive rule**: `docs/archive/` is a frozen historical snapshot (plans, dated specs, completed feature stories). Do **not** load it for current truth — only for historical research into how something evolved.

## Sub-areas

| Area | Load when | Index |
|---|---|---|
| **architecture/** | Understanding system design, auth, app lifecycle, desktop, or doing security work | `architecture/CLAUDE.md` |
| **guides/** | Integrating with BodhiApp's APIs as an external consumer (OpenAI/Anthropic/native/NAPI) | `guides/CLAUDE.md` |
| **conventions/** | Writing tests, managing deps, CI/CD, setup flow, or other cross-cutting practices | `conventions/CLAUDE.md` |
| **research/** | Working on the model-router / fallback-routing feature | `research/CLAUDE.md` |
| **notes/** | Picking up open tech-debt, deferred fixes, or the roadmap | `notes/CLAUDE.md` |
| **deployments/** | Deploying to Railway (single- or multi-tenant) | `deployments/railway.md`, `deployments/railway-multi-tenant.md` |

## Authoring areas (not docs)
- `docs/claude-plans/<yyyymm>/` — where Claude Code writes new implementation plans.
- `docs/specs/` — where new feature specs are authored.

> Before writing any test, read `conventions/testing.md`. Before any security assessment, read `architecture/security.md`.
