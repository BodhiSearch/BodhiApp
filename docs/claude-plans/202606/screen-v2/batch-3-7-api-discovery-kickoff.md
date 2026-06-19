# Kick-off — Batch 3-7: Models · API Models discovery (reference API)

> Load shared context via @common-prompt.md + **@reference-api.md**, then run the per-batch loop
> (@process.md). **Read @batch-3-6-local-discovery-retro.md first** (it establishes the Models
> reference-API client + stub pattern this batch reuses). Final Models sub-phase.

## Scope — the "API Models" discovery sub-view (deferred from 3-1)
The My Models sidebar (3-1) MODEL TYPE section's third entry: **API Models** discovery — a catalog of
known hosted-API providers/models from the reference API (Status/Capability/Pricing/API-Format facets
in the prototype), distinct from the user's configured API aliases. "Add"/"configure" from a discovered
provider → the New API Model form (3-2).

## Reference-API prerequisite (export the spec, hand off — reuse the 3-6 client/stub pattern)
Per @reference-api.md:
1. **Infer the data shape** from `design/bodhi-models-data.js` + `bodhi-models-app.jsx` — an "API
   Models" discovery row (provider, model, capability, pricing, api-format, status, …).
2. **Write the interface spec** (`models-api-discovery-reference-api.md`): endpoints, schemas, id_token
   auth. **Export + report back** for the parallel API build.
3. **Build + test against the MSW external-origin stub** (reuse the 3-6 reference-API client + add the
   API-discovery hook).

## Loop (abbrev — mirror 3-6)
0. **GATE A** — walk the API Models sub-view (Browse/Status/Capability/Pricing/API-Format facets; row
   shape; the "add/configure" action).
1. **Explore** — the 3-6 reference-API client + Models-discovery wiring; prototype sample data.
2. **Prerequisite** — export the spec; build the stub.
3. **Plan → approve** (`batch-3-7-...-plan.md`) — sub-view, hook, e2e stub-vs-real handling, tests, risks.
4. **Implement → tests → GATE B** — discovery list from the stub; "add" wires into the 3-2 API form;
   light + dark + responsive; console-clean.
5. **All gates green** → commit → retro → **Batch 4 (MCP) kickoff** — MCP Discover is the original
   reference-API consumer; carry this Models spec/stub pattern forward.

## Carry-forward gotchas
- Reference API called directly from the frontend; id_token = identity only; never block on the real API.
- State the e2e reference-API handling explicitly. Scope CSS; `ShellSlotsProvider` RTL harness.
- `design/` files user-owned — exclude from commit. This closes Batch 3 (Models) — update @tracker.md.
