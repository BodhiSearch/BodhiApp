# Kick-off — Batch 3-5: Models · Local + API Models discovery (reference API)

> Load shared context via @common-prompt.md + **@reference-api.md** (the external `api.getbodhi.app`
> client + id_token mechanism + per-batch reference-API prerequisite), then run the per-batch loop
> (@process.md). **Read @batch-3-1-models-retro.md first.** This is the **first Models reference-API
> consumer** — like the MCP Discover batch, it needs a reference-API spec exported up front so the API
> team can build it in parallel; the migration is never blocked on it.

## Scope — the two non-"My Models" Model-Type sub-views (deferred from 3-1)
The All Models V2 sidebar (3-1) has a **MODEL TYPE** section: **My Models** (shipped in 3-1, real
backend) + **Local Models** (huggingface.co · GGUF discovery) + **API Models** (hosted-API provider
discovery). 3-1 shipped **only My Models** and omitted the other two. 3-5 adds them as
**reference-API-backed discovery** views (catalog of downloadable GGUF models / known API providers),
distinct from the user's configured "My Models" list.

## Reference-API prerequisite (do this FIRST, hand off to the API team)
Per @reference-api.md, before building the UI:
1. **Infer the data shape** from the prototype's sample data (`design/bodhi-models-data.js` +
   `bodhi-models-app.jsx`) — what a "Local Models" discovery row and an "API Models" discovery row
   carry (model name, params, quant options, provider, badges, install/usage counts, …).
2. **Write the interface spec** in this folder (`models-discovery-reference-api.md`): endpoint paths,
   request/response schemas, auth via the OAuth **id_token** (`Authorization: Bearer`). **Export it to
   the planning working dir and report back** so the API team builds it in parallel.
3. **Build + test the UI against an MSW external-origin stub** (mock origin from app-info; assert the
   id_token header) so the batch is self-sufficient. Use the Batch-0 reference-API client scaffold +
   add the domain hook(s).

## What earlier phases give you
- All Models V2 (3-1): the sidebar's MODEL TYPE section + the list/rail shell to extend with the two
  discovery sub-views. The Batch-0 reference-API client + id_token on the current-user response.
- The "create alias from a discovered model" path → the New Local Model form (3-2) / API Model form (3-4).

## Loop (abbrev)
0. **GATE A** — walk the Local Models + API Models sub-views in the prototype; record the discovery
   row shape, badges, and the "configure/add" actions.
1. **Explore** — the Batch-0 reference-API client + hook scaffold; how `id_token` surfaces; the
   prototype sample data; the 3-1 sidebar MODEL TYPE wiring.
2. **Prerequisite** — export the reference-API spec (above); build the MSW external-origin stub.
3. **Plan → approve** (`batch-3-5-...-plan.md`) — the two sub-views, the reference-API hooks, the
   stub-vs-real handling for CI e2e (explicit, per @reference-api.md), test list, risks.
4. **Implement** the discovery views behind a flag; wire "add/configure" into the 3-2/3-4 forms.
5. **Tests + e2e + GATE B** — discovery list renders from the stub (and real API where available);
   light + dark + responsive; console-clean. State the e2e reference-API handling explicitly.
6. **All gates green** → commit → retro → **Batch 4 (MCP) kickoff** (the next nav section; MCP Discover
   is the original first reference-API consumer — carry this spec/stub pattern forward).

## Carry-forward gotchas
- Reference API is called **directly from the frontend** (no backend proxy); id_token = identity only.
- Never block the migration on the real reference API — stub origin keeps the batch self-sufficient.
- Real-data-only for the BodhiApp-owned bits; the discovery bits are reference-API-owned (versioned).
- Scope CSS; `ShellSlotsProvider` RTL harness; settle-waits + `keepPreviousData` for lists (3-1 retro).
