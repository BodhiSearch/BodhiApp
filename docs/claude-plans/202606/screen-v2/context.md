# UI V2 Migration — Context

Self-contained context for the UI V2 migration task. This folder (`screen-v2/`) is the single
source of truth for the work; these docs are written to be loaded at the start of any session via
[common-prompt.md](@common-prompt.md). They are **orientation, not prescription** — point at where
to look and what the goal is, and let each batch explore the current code fresh, because the code
mutates as the migration progresses.

## The task in one paragraph

The team produced a complete, high-fidelity redesign of all 14 app screens. We are migrating the
production frontend (`crates/bodhi`) from its current **sticky top-header + dropdown nav** to the V2
design: a **left-sidebar `AppShell`** with 5 sibling nav sections — **Chat · Models · MCP · API Keys
· Settings** — each supporting a collapse-to-icon-rail, hover column-resize, an optional 3rd detail
"rail" column, and mobile overlay drawers. The work is **overwhelmingly presentation-layer**: the
data layer (domain hooks over `/bodhi/v1/*`) already exists and is reused; we replace chrome and
per-screen UI.

## Goal & end state

Migrate all 14 screens to AppShell V2. **Done means:** every screen on the new shell, every
per-screen migration flag removed, all old chrome/dead code deleted, and all tests (unit + e2e)
green.

## How we work (the shape, details in [process.md](@process.md))

- **In-place, flag-gated, per screen.** AppShell is the root layout; each screen renders old or new
  at the **same route** based on a per-screen flag. Both coexist while a batch is in flight; the
  flag is retired and the old version deleted when the new screen is accepted.
- **One nav section per batch**, roadmap order: `Foundation → API Keys → Settings → Models → MCP →
  Chat` (Foundation first and Chat last are fixed; the rest can be re-ranked).
- **Iterative & manually coordinated.** No frozen master plan. Each batch is **exploratory**: it
  inspects what's already migrated, ensures its prerequisites, then proposes and gets approval for
  its own plan before coding. Hand-over happens through this folder.

## Why this approach

Migrations evolve and the code changes underneath us. A frozen end-to-end plan goes stale and
removes the human coordination checkpoints we want between batches. Keeping kick-offs exploratory —
"here's the goal and where to look, now go read the current code and propose" — produces better
plans than binding future batches to a recipe written against today's code.

## The other context docs in this folder

- [design-reference.md](@design-reference.md) — the `design/` hi-fidelity prototype: how to view it
  visually (server on :8000 + Claude-in-Chrome), and the rule that we reference it **visually** but
  build to **our** conventions (it is not code to copy).
- [architecture.md](@architecture.md) — orientation to the production frontend + backend: where
  routes, hooks, theming, tests live, and the directional pointers for the AppShell port and the
  backend id_token / reference_api_url work. Pointers, not file:line maps.
- [screen-coverage.md](@screen-coverage.md) — the live coverage checklist: app screen ↔ V2 design,
  consolidations (e.g. model files→alias page, MCP servers→Discover, /new/+/edit/ share one design),
  and the design-pending surfaces the user will provide (login, setup, auth callbacks, Keycloak).
- [reference-api.md](@reference-api.md) — the external `api.getbodhi.app` reference API, the
  id_token identity mechanism, and how reference-API needs are handled as **per-batch
  prerequisites** (interface + spec + MSW stub).
- [process.md](@process.md) — the per-batch loop, the per-screen migration recipe, e2e migration
  context, batch sequence, Batch 0 foundation scope, and the carry-forward risks/watch-outs.
- [glossary.md](@glossary.md) — shared vocabulary (AppShell, rail, section/subPage, batch, flag,
  reference API, id_token, prototype).
- [techdebt.md](@techdebt.md) — tracked failing tests + deferred fixes carried across batches
  (Batch 0: 2 pre-existing backend live-tests; 3 E2E specs to classify — the chat spec to verify
  before the Chat batch).

## Working-folder convention

All intermediate output lives here: per-batch plans (`batch-N-<section>-plan.md`), retrospectives
(`batch-N-<section>-retro.md`), next-batch kick-offs (`batch-(N+1)-<section>-kickoff.md`), and any
reference-API interface specs a batch produces (e.g. `mcp-discover-reference-api.md`). The first
kick-off, [batch-0-foundation-kickoff.md](@batch-0-foundation-kickoff.md), is already here.
