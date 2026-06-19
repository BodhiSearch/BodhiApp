# Kick-off — Batch 3-6: Models · Local Models discovery (reference API)

> Load shared context via @common-prompt.md + **@reference-api.md** (the external `api.getbodhi.app`
> client + the OAuth id_token mechanism + the per-batch reference-API prerequisite), then run the
> per-batch loop (@process.md). **Read @batch-3-1-models-retro.md first.** This is the **first Models
> reference-API consumer** — like MCP Discover, it needs a reference-API spec exported up front so the
> API team builds it in parallel; the migration is never blocked on it.

## Scope — the "Local Models" discovery sub-view (deferred from 3-1)
The My Models sidebar (3-1) has a **MODEL TYPE** section: My Models (shipped, real backend) + **Local
Models** (huggingface.co · GGUF discovery) + API Models (3-7). 3-6 adds the **Local Models** discovery
view: a catalog of downloadable GGUF models from the reference API, distinct from the user's configured
"My Models". "Add"/"configure" from a discovered model → the New Local Model form (3-4) / files-pull (3-5).

## Reference-API prerequisite (do FIRST — export the spec, hand off to the API team)
Per @reference-api.md, before building the UI:
1. **Infer the data shape** from the prototype's sample data (`design/bodhi-models-data.js` +
   `bodhi-models-app.jsx`) — a "Local Models" discovery row's fields (model name, params, quant
   options, license, capability, size/ctx, install/usage counts, …).
2. **Write the interface spec** in this folder (`models-local-discovery-reference-api.md`): endpoint
   paths, request/response schemas, auth via the OAuth **id_token** (`Authorization: Bearer`). **Export
   it + report back** so the API team builds it in parallel.
3. **Build + test the UI against an MSW external-origin stub** (mock origin from app-info; assert the
   id_token header) so the batch is self-sufficient. Use the Batch-0 reference-API client scaffold +
   add the Local-discovery hook(s).

## Loop (abbrev)
0. **GATE A** — walk the Local Models sub-view in the prototype (Browse/Specialisation/Capability/Size/
   Quant/License facets; discovery row shape; the "add/configure" action).
1. **Explore** — the Batch-0 reference-API client + hook scaffold; how `id_token` surfaces; the
   prototype sample data; the 3-1 sidebar MODEL TYPE wiring.
2. **Prerequisite** — export the reference-API spec (above); build the MSW external-origin stub.
3. **Plan → approve** (`batch-3-6-...-plan.md`) — the sub-view, the reference-API hook, the stub-vs-real
   handling for CI e2e (explicit, per @reference-api.md), test list, risks.
4. **Implement → tests → GATE B** — discovery list renders from the stub (and real API where available);
   "add" wires into the 3-4/3-5 flows; light + dark + responsive; console-clean.
5. **All gates green** → commit → retro → **3-7 (API Models discovery) kickoff**.

## Carry-forward gotchas
- Reference API is called **directly from the frontend** (no backend proxy); id_token = identity only.
- Never block the migration on the real reference API — the stub origin keeps the batch self-sufficient.
- State the e2e reference-API handling explicitly (stub/tag/defer), never silently skip.
- Scope CSS; `ShellSlotsProvider` RTL harness; settle-waits + `keepPreviousData` for lists (3-1 retro).
- `design/` files user-owned — exclude from commit.
