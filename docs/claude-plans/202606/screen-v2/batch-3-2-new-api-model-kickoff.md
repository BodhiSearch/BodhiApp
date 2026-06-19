# Kick-off — Batch 3-2: New API Model (form) — NEXT

> Load shared context via @common-prompt.md (framework skills: `tanstack-router`, `tanstack-query`,
> `vercel-react-best-practices`, `web-design-guidelines`), then run the per-batch loop
> (@process.md §"The per-batch loop"). **Read @batch-3-1-models-retro.md first** (carries the full-stack
> recipe + the form-migration learnings + the GATE-B-rebuild-binary rule). Batch 3 is split into
> sub-phases (@tracker.md); 3-1 (My Models list) is ✅.

## Why this is first
Re-sequenced 2026-06-19: **API Model → Fallback → Local Model**, because the API + Fallback forms are
more straightforward than the local-model form (no quant table / flag catalog / download flow). The
API Model form is the **first form port** — it establishes the `bf-*` form-CSS port that 3-3/3-4 reuse.
It is its own sub-phase (commit + retro), then 3-3 (Fallback).

## Scope — the New API Model form (1 screen, 2 routes share it)
- `routes/models/api/new/` **+** `…/edit/` — one form, create vs edit mode. Flag `new-api-model`.
- Design source: `Create API Model.html` (walk it live on http://localhost:8000 — **GATE A**).
- The **My Models list rail's "Edit api model" CTA** + the "New API Model" nav already point here
  (they hit V1 today; this batch makes them V2).

## Real-data map (confirmed in 3-1 exploration — the prod form already has most of it)
The production `ApiModelForm` (`crates/bodhi/src/components/api-models/`) maps ~1:1 to the prototype:
- **PROVIDER CONNECTION**: `ApiFormatSelector` (**disabled on edit** — server enforces
  `ApiFormatImmutableOnEdit`; see `crates/bodhi/src/CLAUDE.md`), `BaseUrlInput`, `ApiKeyInput` with a
  **"Use API key" toggle + reveal** (optional key), `ExtrasSection` (`extra_headers`/`extra_body` JSON).
- **LLM Liberty OAuth** format → `LlmLibertyEnvelopeInput` (paste-in JSON envelope, replaces base_url/
  key/extras). Keep `autoComplete=off/spellCheck=false/data-1p-ignore/data-lpignore` on the textarea.
- **Prefix** (`PrefixInput`) + **Request Forwarding Mode** (`ForwardModeSelector`: forward-all vs
  selected-models). **MODEL SELECTION** (`ModelSelectionSection`: Fetch Models / Select All / search /
  checkboxes / selected chips). **Test Connection** + submit/cancel (`FormActions`).
- `api_format` taxonomy: `openai / openai_responses / anthropic / anthropic_oauth / gemini /
  llm_liberty_oauth`. Real hooks: `useCreateApiModel` / `useUpdateApiModel` / `useGetApiModel` /
  `useTestApiModel` / `useFetchApiModels` / `useListApiFormats`. Schema: `schemas/apiModel.ts`
  (`createApiModelSchema` / `updateApiModelSchema`, `convertFormToCreateRequest` /
  `convertFormToUpdateRequest`, `convertApiToForm`, `API_FORMAT_PRESETS`).

## Prerequisite (the one new shared piece — sets up 3-3/3-4)
- **Port `bodhi-form.css` (`bf-*`)** scoped to a per-screen root (e.g. `.api-model-screen` or a shared
  `.bodhi-form` root reused by all 3 forms). Batch-1/2 CSS discipline: scope generic names, rename
  collisions, drop global `:root`/`mark`. This is the **first form port** — Batches 1/2 + 3-1 skipped
  it (list screens). Decide during planning whether the `bf-*` root is shared across forms or per-screen.
- No backend change expected (the create/update contract is unchanged from 3-1). **Confirm.**

## Loop
0. **GATE A** — walk `Create API Model.html` live: change `api_format` (watch fields swap, esp. Liberty
   envelope vs base_url/key), toggle Use-API-key, forward-all vs selected, Fetch Models, Test
   Connection, the right-rail/info if any. Record behaviors; mark real vs prototype-only.
1. **Explore** — `routes/models/api/` (new/edit) + `components/api-models/` (form/hooks/actions),
   `schemas/apiModel.ts`, the RTL tests (`api/new/index.test.tsx`, `api/edit/index.test.tsx`), the
   `ApiModelFormPage` e2e page object. **Real data only.**
2. **Prereq** — port `bf-*` CSS scoped; confirm no backend change.
3. **Plan → approve** (`batch-3-2-new-api-model-plan.md`) — form structure (dialog→page already done in
   prod), `useShellChrome` (breadcrumb; a `rail`/`sidebar` only if the design has one), the flag-branch
   (`useUiV2Flag('new-api-model')`), `api_format` lock-on-edit, the Liberty path, test list, risks.
4. **Implement** behind `new-api-model`. Reuse the real mutations + `convert*` helpers; keep
   `api_format` disabled on edit; keep `data-testid` + ARIA verbatim. V2 chrome via the shell.
5. **Migrate tests + e2e** — RTL via the `ShellSlotsProvider` harness; e2e via direct
   `navigate('/ui/models/api/new/')` (or `navViaShell('models','new-api-model')` — but **prefer direct
   nav**, the shell-nav dropdown can be intercepted; see 3-1 retro). Set the flag via `addInitScript`.
   `reducedMotion:'reduce'` if any rail/VT. **Gate Liberty e2e explicitly** (needs a creds env — see
   @reference-api.md / techdebt; stub or tag if absent, never silently skip).
6. **GATE B (live)** — **rebuild the binary if any backend changed** (3-1 rule); else HMR is enough.
   Create + edit a real API model live (each format incl. an OpenAI happy-path; `api_format`
   lock-on-edit; Fetch Models; Test Connection); **light + dark + responsive**; **console-clean**.
7. **All gates green** → retire `new-api-model` flag + delete the V1 API-model form (keep the other V1
   form routes) → migrate the API-model E2E specs that drove create/edit via V1 → commit → retro → **3-3
   (Fallback form) kickoff**.

## Carry-forward gotchas (from 3-1 + Batch-1/2 retros)
- **`api_format` read-only on edit** — gate on `mode==='edit'`, never infer from data.
- **Liberty paste-in secrets** — keep the password-manager/spellcheck-suppression attrs.
- Route tests render the page directly → wrap in `ShellSlotsProvider` + slots-consumer.
- `routeTree.gen.ts` is git-ignored; commit route files only.
- Scope generic CSS under a per-screen/per-form root; strip prototype idioms
  (`lucide.createIcons`/`window.*`/`ReactDOM.createRoot`/`data-theme`/`TweaksPanel`).
- Real-data-only — omit prototype elements with no backing field; but **don't drop real fields the
  prototype omits** (3-1: the prototype omitted resilience/size; the backend had them).
- The router-nav `InvalidStateError` on route entry is pre-existing (techdebt.md), not yours.
- `design/` prototype files are **user-owned** — never include them in your commit.
- **Retiring the V1 form means its E2E specs move with it** — the api-models specs drive create/edit
  via the V1 form; migrate them to the V2 form in this batch (this is why 3-1 kept its flag — the list
  alone couldn't retire while forms were V1; now the API form + its specs migrate together).
