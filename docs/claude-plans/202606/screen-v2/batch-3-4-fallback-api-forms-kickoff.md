# Kick-off — Batch 3-4: New Fallback Alias + New API Model (forms)

> Load shared context via @common-prompt.md, then run the per-batch loop (@process.md). **Read
> @batch-3-1-models-retro.md + @batch-3-2-new-local-model-retro.md first.** Reuses the `bf-*` form CSS
> ported in 3-2. These two forms can land together (both API-side forms) or split if scope grows.

## Scope — 2 form screens (each new+edit share one design)
| Screen | design source | prod routes | flag |
|---|---|---|---|
| New Fallback Alias | `Create Fallback Model.html` | `models/router/new/` + `…/edit/` | `new-fallback-model` |
| New API Model | `Create API Model.html` | `models/api/new/` + `…/edit/` | `new-api-model` |

## Key real-data facts (confirmed during 3-1 exploration)
- **API Model form** maps ~1:1 to the existing production `ApiModelForm` (`components/api-models/`):
  PROVIDER CONNECTION (API Format selector — **disabled on edit**, server enforces too; Base URL; API
  Key with "Use API key" toggle + reveal), Prefix, Request Forwarding Mode (forward-all vs selected),
  MODEL SELECTION (Fetch Models / Select All / search / checkboxes), Test Connection. **Liberty OAuth**
  uses the paste-in envelope flow (`LlmLibertyEnvelopeInput`). `api_format` taxonomy: openai /
  openai_responses / anthropic / anthropic_oauth / gemini / llm_liberty_oauth.
- **Fallback form** — the production `ModelRouterForm` carries **resilience config** the prototype
  OMITS (`cooldown_secs` / `max_attempts` / `honor_retry_after` in `FallbackConfig`). **Decision
  pending with user at plan time:** keep resilience settings (real backend fields, already wired) vs
  match the prototype and send backend defaults. Recommend **keep** (real-data cuts both ways).
  Steps are `targets[] {alias, model, enabled, weight?}`; strategy is hardcoded `fallback`. API-model
  steps get a "ROUTE TO MODEL (pre-configured only)" sub-select.

## What earlier phases give you
- All Models V2 list (3-1) — its Fallback rail's **"Edit fallback alias"** CTA + the API rail's "Edit
  api model" CTA point at these routes (today they hit V1; this batch makes them V2).
- `bf-*` form CSS (3-2). Real hooks: `useCreateModelRouter`/`useUpdate…`/`useGetModelRouter`;
  `useCreateApiModel`/`useUpdate…`/`useGetApiModel`/`useTestApiModel`/`useFetchApiModels`/
  `useListApiFormats`. Schemas: `apiModel.ts` (`convertFormToCreate/UpdateRequest`,
  `API_FORMAT_PRESETS`), `llmLibertyEnvelope.ts`. The existing `ApiModelForm` + `ModelRouterForm`.

## Loop (abbrev)
0. **GATE A** — walk both prototypes live (api_format change, local-vs-API, model picker, fallback
   step add/remove/reorder/disable, the right-hand info rail on the fallback form).
1. **Explore** — `routes/models/api/` + `router/` (+ their `-components` / `components/api-models/`),
   schemas, hooks, tests, the `ApiModelFormPage`/`ModelRouterFormPage` e2e page objects.
2. **Plan → approve** (`batch-3-4-...-plan.md`) — **resolve the fallback-resilience question**, the
   right-rail info panel (fallback form has a `rail`), `api_format` lock-on-edit, test list, risks.
3. **Implement** behind `new-api-model` + `new-fallback-model` flags. Reuse the real mutations +
   `convert*` helpers; keep `api_format` disabled on edit; keep `data-testid` + ARIA verbatim.
4. **Tests + e2e + GATE B** — create + edit both alias kinds live (incl. a Liberty envelope path if a
   creds env is available, else gate it explicitly — see @reference-api.md / techdebt). `api_format`
   lock-on-edit; fallback target add/remove/reorder/disable; light + dark + responsive; console-clean.
5. **All gates green** → retire both flags + delete V1 forms → commit → retro → 3-5 kickoff.

## Carry-forward gotchas
- **`api_format` is read-only on edit** — gate on `mode==='edit'`, never infer from data; server
  enforces `ApiFormatImmutableOnEdit`.
- **Liberty paste-in** secrets — keep `autoComplete=off/spellCheck=false/data-1p-ignore/data-lpignore`.
- Fallback right-rail is a published `rail` — memoize it; `useViewTransition` for in-page only.
- Scope CSS; strip prototype idioms; `ShellSlotsProvider` RTL harness; real-data-only.
