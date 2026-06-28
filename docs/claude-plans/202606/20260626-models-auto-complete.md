# Repo autocomplete for the New Local Model form

## Context

The reference API (`api-getbodhi-app`, HEAD commit `a22505e`) just shipped
`GET /api/v1/repos?search=&filter=gguf&limit=` — a typeahead returning lean
`{ items: [{ id: "<author>/<repo>" }] }` suggestions for full HuggingFace repo ids, sized for a
dropdown. The chosen `id` feeds directly into the single-model endpoint
(`GET /api/v1/models/huggingface/{ns}/{repo}`) that the form already calls to list quants.

Today the **New Local Model** form (`/ui/models/alias/new/`) has a plain free-text `Repo` input
(`AliasForm.tsx:194`) whose hint already promises "Suggestions shown" — but nothing is wired. We want
to turn that field into a searchable combobox backed by **two sources**, merged local-first:

1. **BodhiApp** — `GET /bodhi/v1/models/files` (already-downloaded repos), filtered client-side. This is
   the primary source on an empty field (the reference endpoint rejects empty `search`).
2. **Reference API** — `GET /api/v1/repos?search=<typed>&filter=gguf` for live HF suggestions as the user
   types.

The two are concatenated **local-first, then remote (deduped)**: a remote `id` already present as a
local repo is dropped. The combobox stays **free-text** — the user can still type any `<org>/<repo>`
not in the list, preserving today's behavior and letting `QuantSelector` resolve quants for it.

### Why this is a small change

The form's reactivity is already correct: the `repo` field is just a string, and `QuantSelector`
(`QuantSelector.tsx`) watches it and resolves quants via `useModelDetail`. So **selecting a repo only
needs to set that string** — no other wiring changes. All the reference-API plumbing
(`referenceApiClient`, `useAnonymousReferenceApi`, `referenceKeys`, `REF_ENDPOINT_*`) and a cmdk +
Popover combobox pattern (`AliasCombobox.tsx`) already exist and are reused verbatim.

## Decisions (from the user)

- **Merge order:** local first, then remote, deduped (remote id already shown as local is dropped).
- **Empty input:** show only local downloaded repos (reference `/api/v1/repos` requires non-empty `search`).
- **Free-text:** keep the free-text fallback — arbitrary typed `<org>/<repo>` still allowed.

## Dependency bump

`crates/bodhi/package.json` pins `@bodhiapp/reference-api-types: ^0.0.9`. The repo types
(`RepoSuggestion`, `ListReposQuery`, `ListReposResponse`) ship in published **0.0.10** (verified against
the npm tarball). The caret already permits 0.0.10, so:

- Run `cd crates/bodhi && npm install @bodhiapp/reference-api-types@^0.0.10` to bump the pin to `^0.0.10`
  and refresh `package-lock.json` to the 0.0.10 resolution.

(No reference-API repo changes are needed — the endpoint and types are deployed and published.)

## Implementation

### 1. New reference hook — `useSearchRepos`

Add to `crates/bodhi/src/hooks/reference/`:

- **`constants.ts`** — add endpoint + query key:
  - `export const REF_ENDPOINT_REPOS = '/api/v1/repos';`
  - extend `referenceKeys`: `repos: (paramsKey: string) => [...referenceKeys.all, 'repos', paramsKey]`.
- **`useSearchRepos.ts`** — mirror `useDiscoverModels.ts`:
  - Types from `@bodhiapp/reference-api-types`: `ListReposQuery`, `ListReposResponse`, `RepoSuggestion`.
  - `useAnonymousReferenceApi()` client (public read, no id_token — same rationale as `useDiscoverModels`).
  - Build query with `search`, `filter=gguf` (always), `limit` (default 10) — reuse the
    `buildModelsQuery` serialization idiom (repeatable `filter`).
  - `enabled: !!client && search.trim().length > 0` (server 422s on empty `search`), `keepPreviousData`,
    a short `staleTime`. Return `RepoSuggestion[]` (the `.items`).
  - Export from `crates/bodhi/src/hooks/reference/index.ts`.

### 2. New `RepoCombobox` component

`crates/bodhi/src/routes/models/alias/-components/RepoCombobox.tsx` — adapt `AliasCombobox.tsx`
(shadcn `Command` + `Popover`, cmdk) for free-text repo ids:

- **Props:** `value: string`, `onChange: (repo: string) => void` (plus the controlled `open`/`onOpenChange`
  + a `testId`, matching the AliasCombobox conventions).
- **Local source:** `useListModelFiles(1, 100, 'repo', 'asc')` → distinct `repo` strings (the form already
  fetches the same list in `QuantSelector`; React Query dedupes the request by key).
- **Remote source:** `useSearchRepos({ search: typed, limit: 10 })`, debounced ~200ms on the cmdk
  `CommandInput` value (use a small local `useState` + `useEffect`/timeout, mirroring the catalog
  search-as-you-type recipe).
- **Merge:** locals matching the typed text first, then remote `items` with any id already in the local set
  filtered out. Tag each option with its origin so the list can render a small "Downloaded" vs "HuggingFace"
  affordance (badge/icon), echoing AliasCombobox's badge layout.
- **Free-text:** because cmdk filters its own items, keep an always-present "Use \"<typed>\"" affordance (or
  commit the raw input on blur/Enter) so a typed repo with no suggestion still sets `value`. The trigger
  shows the current `value` (monospace) or a placeholder.
- **Empty input:** with no typed text, render only the local repos (remote query disabled).
- Reuse the existing `splitRepo` notion only where needed; the combobox deals in full `<org>/<repo>` ids.

### 3. Wire into the form

`crates/bodhi/src/routes/models/alias/-components/AliasForm.tsx` (lines 187–209):

- Replace the `<Input data-testid="repo-input" .../>` inside the `repo` `FormField` with
  `<RepoCombobox value={field.value} onChange={field.onChange} testId="repo-input" ... />`.
- Keep `data-testid="repo-input"` on the combobox trigger so existing selectors/tests keep working.
- The existing `useEffect` that clears `filename` on `repo` change (lines 128–135) and the `QuantSelector`
  wiring (line 246) are unchanged — they already react to the new `repo` value.

No schema, no `convertFormToApi`, no backend changes.

## Files touched

- `crates/bodhi/package.json` + `package-lock.json` — bump `@bodhiapp/reference-api-types` to `^0.0.10`.
- `crates/bodhi/src/hooks/reference/constants.ts` — `REF_ENDPOINT_REPOS`, `referenceKeys.repos`.
- `crates/bodhi/src/hooks/reference/useSearchRepos.ts` (new) + `index.ts` export.
- `crates/bodhi/src/routes/models/alias/-components/RepoCombobox.tsx` (new).
- `crates/bodhi/src/routes/models/alias/-components/AliasForm.tsx` — swap Input → RepoCombobox.

Reused as-is: `referenceApiClient.ts`, `useAnonymousReferenceApi`, `useDiscoverModels`/`useModelDetail`
(quant resolution), `useListModelFiles`, `AliasCombobox` (pattern), `components/ui/command.tsx`,
`components/ui/popover.tsx`.

## Tests

### Component tests (Vitest + MSW v2)

- **New MSW handler** for the external `/api/v1/repos` under
  `crates/bodhi/src/test-utils/msw-v2/handlers/` (mirror `reference-models.ts`; same
  `DEFAULT_BASE = 'https://api.getbodhi.app'`, matching `mockAppInfo({ reference_api_url })`). Returns a
  `ListReposResponse` fixture; assert it honors `search`/`filter=gguf`.
- **`useSearchRepos` hook test** — disabled on empty `search`; returns merged `items`; passes
  `filter=gguf`.
- **`RepoCombobox` test** — on empty input shows local downloaded repos; on typing, shows local matches
  first then remote (deduped); a typed-but-unmatched repo still commits as free text; selecting an option
  calls `onChange` with the full id.
- **`AliasForm` test** — selecting a repo in the combobox drives `QuantSelector` to fetch quants (extend
  the existing AliasForm test to mock both `/api/v1/repos` and `/api/v1/models/huggingface/{ns}/{repo}`).

### E2E (Playwright, `lib_bodhiserver/tests-js`)

- Extend the existing New-Local-Model spec (the Batch 3-4 flow). Black-box only (no `page.evaluate`):
  type into the repo combobox, assert suggestions render, pick one by its accessible name (id), assert the
  quant table populates and the alias saves. Set `reducedMotion: 'reduce'` per the V2 view-transition
  convention. The reference API is external — point it at a mock/dev base or stub at the network layer
  consistently with how the Explore-screen E2E handles the reference API (do **not** `test.skip` if the
  base is unset; throw in `beforeAll`).

## Verification (manual, in Chrome)

1. `cd crates/bodhi && npm install @bodhiapp/reference-api-types@^0.0.10`, then `npm run test` for the new
   unit tests.
2. `make app.run.live` (full stack + live Vite HMR), open `http://localhost:1135/ui/models/alias/new/`.
3. Focus the **Repo** field empty → confirm locally-downloaded repos appear.
4. Type `qwen` → confirm local matches lead, then HuggingFace `…-GGUF` suggestions follow (deduped),
   debounced.
5. Pick a suggestion → confirm the quant table populates (existing `useModelDetail` path) and download
   status renders; save the alias.
6. Type a repo **not** in any list → confirm free-text still sets the value and quant resolution/manual
   filename entry still works.
