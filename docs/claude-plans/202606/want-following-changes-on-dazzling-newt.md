# Explore · API Models — URL filter sync, rail cross-links, row-render fix

## Context

The Explore · API Models page (`/ui/models/explore/api/`) lets users browse the
reference-API model catalog with a faceted sidebar, sortable columns, and a
per-model detail rail. Today all filter/sort/search/page state lives in React
`useState` inside `ExploreApiScreen`, so a page refresh or browser Back/Forward
loses everything — the page has no shareable/bookmarkable URL state. The user
wants this to become a real search page with browser history support.

This change also cleans up two rail issues surfaced in screenshots and finishes
wiring the already-built dual-range price filters into the URL.

Pricing state/debounce status (verified in code):

- The `DualRangeControl` in `ExploreApiSidebar.tsx` already tracks `[min, max]`
  for both Input and Output, wired to `pricing_in_min/max` + `pricing_out_min/max`,
  and the Radix `Slider` fires `onValueCommit` only on release (debounced).
- **BUT** the shared shadcn `Slider` (`src/components/ui/slider.tsx`) rendered a
  single `<Slider.Thumb>`, so only the MIN handle was draggable/visible — the MAX
  handle was missing. This was the real "single slider" bug. Fix: render one
  `Thumb` per value (see item 0 below).

The remaining work:

1. **Browser Back/Forward + URL sync** — track only modified (non-default)
   filters in the URL query string; read them on mount and on history
   navigation, apply them, and write them back when the user changes a filter.
2. **`?provider=<slug>` cross-link** — a rail link that filters the page to all
   models served by one provider.
3. **Row-render bug** — when the detail rail is open the list overflows
   horizontally; the row background + bottom border stop mid-row instead of
   spanning the full scrolled width (see screenshot `06.51.35`).
4. **Remove the big "Configure in Bodhi" CTA** from the rail — the per-provider
   `+` ("Add") button already covers configuring (screenshot `07.06.26`).
5. **Replace the rail's "Documentation" link** with two links: *All Models from
   Provider* and *View*.

## Design summary

Make the TanStack Router **`useSearch()` the single source of truth** for the
page. Derive `sort`/`order`/`page`/`q`/`facets`/query-params directly from the
parsed search each render; write changes via `navigate({ search })` on user
action only. No `useState` mirror of URL state, and **no effect that writes the
URL** — that eliminates the read→write→read loop class entirely. Only
`searchInput` (uncommitted text), `selectedKey` (open rail), and
`hiddenColumns` stay local.

## Files & changes

### 0. `components/ui/slider.tsx` — render one Thumb per value (dual-range fix)

Radix needs one `<Slider.Thumb>` per value. The shared component rendered exactly
one, so range sliders (`value={[min,max]}`) showed only the min handle. Derive
`thumbCount` from `(value ?? defaultValue)?.length ?? 1` and map that many Thumbs.
Single-value sliders (chat settings, provider/context sliders) are unaffected
(length 1 → one thumb).

### 1. `routes/models/explore/api/index.tsx` — add `validateSearch` (Zod)

Add a Zod schema as the typed source of truth. All fields `.optional()`;
defaults (`sort=updated`, `order=desc`, `page=1`) are **never written** to the
URL (the screen strips them before `navigate`), so the URL stays clean.

```ts
export const exploreApiSearchSchema = z.object({
  q: z.string().optional(),
  sort: z.enum(SORT).optional(),          // SORT/CAPABILITY/MODALITY/STATUS tuples
  order: z.enum(['asc', 'desc']).optional(),
  page: z.number().int().positive().optional(),
  capability: z.array(z.enum(CAPABILITY)).optional(),
  modality: z.array(z.enum(MODALITY)).optional(),
  status: z.array(z.enum(STATUS)).optional(),
  provider: z.array(z.string()).optional(),     // provider SLUGs (facet key is slug)
  family: z.array(z.string()).optional(),
  open_weights: z.enum(['open', 'closed']).optional(),
  pricing: z.enum(['free', 'paid']).optional(),
  pricing_in_min: z.number().optional(),
  pricing_in_max: z.number().optional(),
  pricing_out_min: z.number().optional(),
  pricing_out_max: z.number().optional(),
  context_min: z.number().optional(),
});
export type ExploreApiSearch = z.infer<typeof exploreApiSearchSchema>;
```

Wire it: `createFileRoute('/models/explore/api/')({ validateSearch: exploreApiSearchSchema, component })`.
Pull the exact `Capability`/`Modality`/status union members from
`@bodhiapp/reference-api-types` (`status` tuple = `stable|alpha|beta|deprecated`).
Using `z.enum` for closed sets means a hand-edited junk param is dropped rather
than sent upstream; keep `provider`/`family` as open `z.array(z.string())`.

> Array encoding: leave the router's default stringifier. The **visible** URL
> array encoding is independent of the catalog API request, which is built
> separately by `buildCatalogModelsQuery` (already emits repeated keys). Do NOT
> add a router-wide `parseSearch`/`stringifySearch` in `main.tsx` — out of scope
> and affects every route.

### 2. New `routes/models/explore/api/-components/explore-api-search.ts` — mappers

Pure, unit-testable mapping between URL search ⇄ facet state ⇄ API params.
Reuse the existing `modelFacetsToQuery` (in `ExploreApiSidebar.tsx`) — the
facet-slice of the URL and the API params have the identical "non-default facet"
shape.

```ts
export const DEFAULT_SORT = 'updated', DEFAULT_ORDER = 'desc', PAGE_SIZE = 30;

searchToFacets(s: ExploreApiSearch): ModelFacetsState   // inverse of modelFacetsToQuery
facetsToSearch(f: ModelFacetsState): Partial<ExploreApiSearch>  // == modelFacetsToQuery
searchToParams(s: ExploreApiSearch): ListCatalogModelsQuery     // applies defaults + page_size
```

### 3. `ExploreApiScreen.tsx` — derive from search, write via `navigate`

Replace the `page/search/sort/order/facets` `useState` with derivations:

```ts
const routeApi = getRouteApi('/models/explore/api/');
const search = routeApi.useSearch();
const navigate = routeApi.useNavigate();

const sort = search.sort ?? DEFAULT_SORT;
const order = search.order ?? DEFAULT_ORDER;
const page = search.page ?? 1;
const committedSearch = search.q ?? '';
const facets = useMemo(() => searchToFacets(search), [search]);
const params = useMemo(() => searchToParams(search), [search]);   // → useCatalogModels(params)
```

Keep local: `searchInput`, `selectedKey`, `hiddenColumns`. (Deliberately NOT
URL-syncing `selectedKey` — the open rail is ephemeral UI; comment this.)

All mutations go through `navigate({ search: (prev) => next })` with a
**functional updater** (always reads latest parsed search). One helper strips
defaults and resets `page` on filter changes:

- `onSort` → set `sort`/`order` (strip if equal to defaults), drop `page`.
- `onFacetsChange(next)` / `onClearAllFacets` → rebuild search as
  **non-facet slice (`q`/`sort`/`order`) + `facetsToSearch(next)`** so removing a
  facet actually drops its key (a shallow merge can't delete); `page` dropped → resets to 1.
- `commitSearch(value)` → set/clear `q`; when `q` present switch `sort` to
  `relevance`, else revert to default (preserves today's search-ranks-by-match
  behavior); drop `page`/`order`.
- Pager `onPage(p)` → `navigate` setting `page` (the one place page is set).
- `searchInput`: stays local; one **read-only** effect syncs it down from
  `committedSearch` on Back/Forward (`useEffect([committedSearch])`) — never
  writes the URL, so no loop.

`keepPreviousData` UX is preserved automatically: `params` is referentially
stable per distinct URL via `useMemo([search])`.

### 4. `ExploreApiRail.tsx` — remove CTA, replace Documentation link

- **Remove** the bottom `Configure in Bodhi` CTA block and its
  `base_url_requires_substitution` note, plus the now-dead `bridge` /
  `configureSearch` computation. The per-provider `+` (`cat-servedby-add`) in
  `ServedByRow` is the remaining configure path.
- In `ServedByRow`, **replace** the conditional Documentation `<a>` with two
  `<Link>`s (keep the Base URL / API format / API keys rows above):

```tsx
<Link to="/models/explore/api/" search={{ provider: [served.slug] }}
      className="cat-doc-link" data-testid={`cat-model-servedby-allmodels-${served.slug}`}>
  <ShellIcon name="layers" size={13} /> All Models from Provider
</Link>
<Link to="/models/explore/api-providers/" search={{ q: served.name }}
      className="cat-doc-link" data-testid={`cat-model-servedby-view-${served.slug}`}>
  <ShellIcon name="external-link" size={13} /> View
</Link>
```

`served.slug` is the provider facet value (facet bucket is keyed by slug);
`served.name` is the human search term for the providers page. *All Models from
Provider* targets the **same route**, so the existing URL-sync mechanism (item 3)
filters the page in place. Add a `.cat-servedby-links` flex wrapper in
`catalog.css` for the two links.

### 5. `api-providers/index.tsx` + `ExploreProvidersScreen.tsx` — accept `?q=` (seed only)

Per the user: **do not otherwise change the providers page** — its own
back/forward + field prepop is the next iteration. Minimal change so the *View*
link works:

- Add `q: z.string().optional()` to the existing `providersSearchSchema`
  (alongside `select`).
- In `ExploreProvidersScreen`, read `q` via `useSearch({ strict: false })` and
  **seed once** (`useRef` guard) into the existing local `searchInput`/`search`
  on mount, mirroring the existing `select` effect. One-shot so later typing
  isn't fought by the URL; never writes back.

### 6. `routes/models/explore/-shared/catalog.css` — row-render fix

Root cause: `.l-scroll` (in `components/shell/list.css`) has `overflow-y: auto`
but **no `overflow-x`**, and the model grid template (computed inline, wider than
the container when the rail is open) gets clipped at the container's right edge —
so each row's background and bottom border stop mid-row instead of spanning the
full content width.

Fix (CSS only; column picker stays authoritative):

```css
.l-scroll { overflow-x: auto; }          /* add alongside the existing overflow-y:auto */
.cat-model-grid { min-width: max-content; } /* row stretches to full content width → bg/border span it */
```

Scope `min-width` to `.cat-model-grid` (header + rows) so the provider grid
(`cat-prov-grid`, `1fr`-based) is unaffected. The header already lives inside
`.l-scroll`, so header and rows share one horizontal scrollbar and stay aligned.
(Auto-hiding columns when the rail opens is a possible future UX, but belongs in
the column-picker JS, not here — deferred.)

## Verification

End-to-end, run the app live (`make app.run.live`, no rebuild needed — Vite HMR)
and drive it in Chrome:

1. Apply a capability filter + change sort → confirm URL gains
   `?capability=…&sort=…`; the list updates. Hit browser **Back** → filters and
   list revert; **Forward** → re-applied.
2. Drag the Input/Output min+max sliders, release → one request fires with
   `pricing_in_min/max` / `pricing_out_min/max`; URL carries them; Back reverts.
3. Open a model rail → click **All Models from Provider** → URL becomes
   `…/api/?provider=<slug>`, provider facet active, list filtered. Click **View**
   → lands on `…/api-providers/?q=<name>` with the search box pre-filled.
4. Confirm the **Configure in Bodhi** CTA is gone; the per-provider `+` remains.
5. Open the rail and narrow the viewport → row background + bottom border span
   the full (scrollable) row width; a single horizontal scrollbar appears,
   header stays aligned.

Automated tests (all layers):

- **Unit (Vitest)** — new `explore-api-search.test.ts`: round-trip
  `searchToFacets`/`facetsToSearch`, `searchToParams` defaults + empties; extends
  the existing `catalog-query.test.ts` coverage of `modelFacetsToQuery`.
- **Component (Vitest + MSW)** — migrate `api/index.v2.test.tsx` to a real
  **memory router** (drop the `Link` mock; assert real `href`s): mounting at
  `?provider=anthropic` issues a request with `provider=anthropic` and shows the
  chip; sort/facet writes reset `page` and strip defaults; `history.back()`
  re-applies. New `ServedByRow` tests assert both link `to`+`search`. Providers
  test: `?q=anthropic` seeds the box and requests `q=anthropic`.
- **E2E (Playwright, black-box, UI-only)** — grow the API-explore spec with
  `test.step`s for: Back/Forward re-apply, both cross-links, CTA-removed, and a
  light row-render check (rail open → list horizontally scrollable, model-name
  not clipped to zero). Run with `reducedMotion: 'reduce'` (V2 rail screens) and
  wait for mutation settle before asserting.

## Commit sequence (trunk-based, commit to `main`)

1. `feat(ui): add validateSearch schema + search mappers for Explore API Models` (items 1–2 + unit tests)
2. `feat(ui): drive Explore API Models from URL search (back/forward)` (item 3 + component tests, harness migration)
3. `feat(ui): rail cross-links + remove Configure CTA; seed providers ?q=` (items 4–5 + tests)
4. `fix(ui): Explore API Models row background/border span full width` (item 6)
5. `test(e2e): Explore API Models URL sync + cross-links`
