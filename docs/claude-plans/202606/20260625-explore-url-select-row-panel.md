# Plan: URL-synced row selection (detail rail) for the Explore pages

## Context

On both Explore screens — **Models** (`/ui/models/explore/api/`) and **Providers**
(`/ui/models/explore/providers/`) — clicking a list row opens a right-side detail **rail**. Today the
selected row lives in component `useState` (`selectedKey` / `selectedSlug`), so it is lost on refresh
and does not participate in browser Back/Forward. Providers already reads `?select=<slug>` once on
mount (for the Models→Providers cross-link) but never writes it.

We want the **selected row captured in the URL** so the rail state survives Back/Forward, but
**without polluting history**: selecting/switching/closing rows must NOT create history entries.
Practically this means: the URL always reflects the current selection, every selection write uses
`replace: true`, and Back/Forward jumps past row selections to the previous **filter/page** state
(restoring whatever `?select` that URL encodes).

### Confirmed decisions (with the user)

- **Selection == rail-open.** There is no "selected but closed" state. `?select=<id>` present ⇒ row
  highlighted + rail open; absent ⇒ no selection, rail closed. The AppShell rail already auto-opens
  whenever rail slot content is non-null, so we only sync the selected-row id — **no separate
  `rail=open|closed` param**.
- **Always `replace` for selection.** Select / switch / close all use
  `navigate({ search, replace: true })`. Filter/sort/search/page writes keep their existing (push)
  behavior. Mixing is fine.
- **Dedup.** Selecting the already-selected row is a no-op (no redundant navigate).
- **Filter + selection:** keep `?select` across filter/sort/search/page changes; **if the selected
  row is filtered/paged out of the current results, the rail simply closes** (we have no empty-rail
  state). On Models this is automatic (`rows.find(...) ?? null` → null → AppShell closes the rail).
  On Providers, the existing *synthesized-provider fallback* is kept (it powers the cross-link), so a
  filtered-out selection there may keep the rail open — an accepted minor inconsistency (see Providers
  note below).

## Design — URL as the single source of truth (mirror sort/facets)

Make the selected row **derived from `search.select`** instead of mirrored in `useState`, exactly
like sort/facets are derived. This makes Back/Forward restoration automatic (no effect needed) and
removes the read-only mount effect on Providers.

Key reference patterns already in these files (reuse, don't reinvent):
- `routeApi.useSearch()` / `routeApi.useNavigate()` + functional `navigate({ search: (prev) => next })`.
- The "exactly one effect, writes LOCAL state only" discipline (so no render loop).
- `withViewTransition` wrapping the open/close so the rail animates.

### Both screens

1. **Derive selection from the URL.** Replace `const [selectedKey/Slug, setSelected...] = useState`
   with `const selected = search.select ?? null`. Remove `setSelected...` calls.
2. **`select(id | null)` callback** → wrap in `withViewTransition` and write the URL with replace +
   dedup:
   ```
   const select = useCallback((id: string | null) => {
     if ((id ?? undefined) === search.select) return;            // dedup
     withViewTransition(() =>
       navigate({ search: (prev) => ({ ...prev, select: id ?? undefined }), replace: true })
     );
   }, [navigate, search.select, withViewTransition]);
   ```
   Row click → `select(id)`; rail header X (`onClose`) → `select(null)`.
   `openRail()` is no longer needed (AppShell auto-opens on content); drop the `useShell().openRail`
   usage if nothing else needs it.
3. **Preserve `select` across filter/sort/search/page writes.** `commitSearch` / `onSort` / `onPage`
   already do `{ ...prev }` so they keep `select` for free — **no change**. The facet writes
   (`onFacetsChange` / `onClearAllFacets`) go through `nonFacetSlice(prev)`, which rebuilds from
   scratch (only q/sort/order) — **add `if (prev.select) base.select = prev.select;`** to
   `nonFacetSlice` so facet/clear-all changes also keep the selection. Facet writes keep their current
   **push** behavior; only `select()` writes use `replace` (per-navigate, mixing is fine).
4. **Rail derivation already closes on filter-out.** The rail/railHeader useMemos derive from the
   selected row found in `rows`; when the row isn't present they yield `null` and AppShell closes the
   rail. No change needed beyond the Providers fallback note.
5. **Drop `openRail()`.** It is the only `useShell()` consumer on each screen after this change
   (AppShell auto-opens from rail content). Remove the `openRail` destructure / `useShell()` import if
   nothing else uses it, to keep lint clean.

### Models specifics (`api/`)

- **Add `select` to the route schema** `exploreApiSearchSchema` (`crates/bodhi/src/routes/models/explore/api/index.tsx`):
  `select: z.string().optional()`. The id is the composite `modelKey(m)` = `` `${slug}/${model_id}` ``
  (contains `/`; TanStack Router URL-encodes it — fine).
- `selectedModel = rows.find((m) => modelKey(m) === search.select) ?? null` (swap `selectedKey` →
  `search.select`). **Slash-safety:** keep deriving `selectedRef = { slug: selectedModel.slug,
  modelId: selectedModel.model_id }` from the FOUND ROW's real fields — never parse the composite key
  (a `model_id` can itself contain `/`, e.g. `anthropic/claude-sonnet-4.5`). The detail fetch is
  unchanged.

### Providers specifics (`providers/`)

- Schema already has `select`. **Remove** the read-only mount effect (lines ~351-356) and the
  `selectedSlug` useState; derive `const selectedSlug = search.select ?? null`. Removing that effect
  preserves the "exactly one effect, writes LOCAL only" discipline (the remaining `searchInput` sync
  effect is untouched).
- **Synthesized-provider fallback — keep as-is.** It powers the deep-link / Models→Providers
  `?select=<slug>` cross-link (a provider that may not be on page 1). Trade-off vs the "filter out →
  close" rule: on Providers a selected-but-filtered-out provider can still render via the gated
  `detail` fetch, so its rail may stay open after a filter excludes it — unlike Models, which closes.
  This is the accepted price of keeping the cross-link working; do NOT add heuristics to suppress it.
  (If this proves annoying in practice, revisit later — out of scope here.)

## Files to change

| Concern | Path |
|---|---|
| Models route schema (`select`) | `crates/bodhi/src/routes/models/explore/api/index.tsx` |
| Models screen (derive selection, select/close, nonFacetSlice) | `crates/bodhi/src/routes/models/explore/api/-components/ExploreApiScreen.tsx` |
| Providers screen (derive, drop mount effect + useState, gate synth, nonFacetSlice) | `crates/bodhi/src/routes/models/explore/providers/-components/ExploreProvidersScreen.tsx` |
| Component tests | `…/api/index.v2.test.tsx`, `…/providers/index.v2.test.tsx` |
| E2E | `crates/lib_bodhiserver/tests-js/specs/models/{api-explore,providers}.spec.mjs` + page objects |

## Testing

- **Component (Vitest + memory-router harness):** for BOTH pages —
  - clicking a row writes `?select=<id>` and opens the rail (rail content + header present);
  - clicking the rail X (onClose) drops `?select` and closes the rail;
  - deep-link `?select=<id>` restores the rail on mount;
  - `router.history.back()` after a selection returns to the prior filter state and the rail
    reflects that URL's `?select` (absent → closed);
  - **replace-not-push (robust assertion):** apply a facet (push) so search = `{capability:['reasoning']}`;
    select a row (replace); select a *different* row (replace); a single `router.history.back()` lands
    back on the **facet state** `{capability:['reasoning']}` — skipping BOTH selections. (Assert on the
    restored search, not a brittle `history.length` getter.);
  - selecting the same row twice issues no extra navigate (dedup) — assert URL/search unchanged;
  - changing a facet keeps `?select` in the URL; if the selected row leaves the result set the rail
    closes (rail content null).
  - Providers: the Models→Providers `?select=<slug>` cross-link still opens the rail (gated synth).
- **E2E (Playwright, black-box):** grow each spec with a `test.step`: open a row → URL gains
  `?select`; reload → rail restored; close → `?select` gone; Back/Forward behaves (one Back closes
  the rail / returns to the unselected list); selecting multiple rows then one Back returns to the
  list (proves replace). Use `reducedMotion:'reduce'`; wait for rail settle via the existing
  `cat-*-detail-*` / `railPanel` testids.

## Verification (manual, Chrome)

On `make app.run.live`, for both `/ui/models/explore/api/` and `/ui/models/explore/providers/`:
1. Click a row → rail opens, URL shows `?select=…`. Refresh → rail still open on the same row.
2. Close (X) → `?select` removed, rail closes. Browser Back after a few selections → lands on the
   unselected list (no stepping through each selected row).
3. With a row open, toggle a filter that excludes it → rail closes; toggle one that keeps it → rail
   stays. The `?select` from the Models "Served by · View/Add" cross-link still opens the Providers rail.

## Gate
`cd crates/bodhi && npm run test` (explore tests green), `npm run lint`, `tsc --noEmit`; run the two
E2E specs standalone. Commit per page or as one focused commit (feature rollout) on `main`.
