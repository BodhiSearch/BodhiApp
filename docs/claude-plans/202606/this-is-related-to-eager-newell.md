# Migrate Explore API Models + Providers lists to semantic `<table>`

## Context

The Explore · API **Models** (`/ui/models/explore/api/`) and **Providers**
(`/ui/models/explore/api-providers/`) screens render tabular data with **CSS Grid
pseudo-tables**, not real HTML tables. The header row and each body row are *separate*
DOM grids that only happen to share a column-width definition:

- **Models**: a computed `gridTemplate` string (`visibleColumns.map(c => c.width).join(' ')`)
  is applied via inline `style={{ gridTemplateColumns }}` to both `.cat-listhead.cat-model-grid`
  and every `.cat-row.cat-model-grid`. When the column picker collapses optional columns,
  the header and the rows can drift out of alignment — this is the reported bug (see the
  screenshot: TEXT / INPUT $ / OUTPUT $ / UPDATED / PROVIDERS headers not lining up over
  their values).
- **Providers**: the header `.cat-listhead.cat-prov-grid` lives **outside** `.l-scroll`,
  so it does not scroll horizontally with the rows (which are inside `.l-scroll`) — a second
  variant of the same "two independent grids" problem.

Both screens carry a pile of bespoke CSS to fight grid layout: `min-width: max-content`
width-sync rules, per-cell `overflow/ellipsis/flex-shrink` hacks, and a long comment block
explaining the fragile width sharing.

**Goal:** Replace the grid divs with a single semantic `<table>` per screen. A shared
`<colgroup>` makes the header `<th>` and body `<td>` widths come from **one source** — they
can never drift. `table-layout: fixed` makes those widths authoritative so per-cell
`text-overflow: ellipsis` "just works", letting us delete the width-sync and most overflow
hacks. This fixes the misalignment, fixes the Providers horizontal-scroll inconsistency, and
simplifies the overflow/truncation rules for labels, icons, and badges — all in one change.

This is a **frontend-only rendering/markup refactor**. No backend, no API, no behavior
change: column picker, sort headers, facet sidebar, detail rail, pagination, search, view
transitions, and keyboard nav all stay identical.

## Scope & key constraints (verified during exploration)

- The grid classes `cat-row`, `cat-listhead`, `cat-model-grid`, `cat-prov-grid` are used
  **only** by these two screens + `catalog.css`. No other V2 screen consumes them — safe to
  rewrite.
- **Do NOT modify shared primitives** in `crates/bodhi/src/components/shell/list.css`
  (`.l-listrow`, `.l-listview`, `.l-scroll`, `.l-rowlink`) — they are used by users, tokens,
  settings, and local-discovery screens. All table overrides are scoped under `.cat-screen`
  in `catalog.css`.
- `useListKeyNav` (`components/shell/useListKeyNav.ts`) finds rows via `querySelectorAll('.l-listrow')`
  and activates a `.l-rowlink` anchor inside each. So each `<tr>` **must keep class `.l-listrow`**
  and contain a `LinkRow` (`<a class="l-rowlink">`).
- All component tests (`api/index.v2.test.tsx`, `api-providers/index.v2.test.tsx`) and E2E
  page objects (`ApiExplorePage.mjs`, `ApiProvidersPage.mjs`) assert via `data-testid`,
  `getAllByRole('option')`, and text content — **none inspect div-vs-table structure**.
  Preserving `data-testid`s and `role="option"` keeps them green.

## Column-sizing decision (Models)

`table-layout: fixed` cannot use the current `minmax(…fr)` tracks. **Chosen approach
(user-confirmed): MODEL absorbs the slack.** All numeric/short columns get fixed px widths;
MODEL is the single width-less `<col>` that absorbs remaining space; FAMILY gets a fixed px
width. This keeps the elastic feel closest to today with one flexible column.

## Horizontal-scroll policy (both screens)

**Avoid horizontal scroll as much as possible without compromising the data shown.** The
table should fit the available width by default; only when the columns genuinely cannot fit
(e.g. the detail rail is open on a narrow viewport, or all optional columns are shown on a
small screen) does the horizontal scrollbar appear.

How this is achieved:
- `.cat-table { width: 100% }` makes the table fill `.l-scroll`, so at normal widths there is
  **no** overflow and **no** horizontal scrollbar.
- Use `min-width: max-content` instead of a hard `width: max-content`: the table only grows
  past the container — triggering `.l-scroll`'s `overflow-x: auto` scrollbar — when the sum of
  the fixed column widths plus the MODEL column's minimum content exceeds the available width.
- The slack-absorbing MODEL `<col>` (no explicit width) soaks up extra space at wide widths
  (no scroll) and shrinks/ellipsizes first as width tightens, so scroll is the last resort.
- No column is dropped to avoid scroll — all visible/picked columns always render; we only
  scroll when they truly don't fit.

## Implementation

### 1. Models screen — `routes/models/explore/api/-components/ExploreApiScreen.tsx`

- **Keep** the `Column` model (`key`, `label`, `width`, `align`, `sort`, `optional`, `cell`).
  Change `width` **semantics** from "grid track" to "`<col>` width": fixed-px columns
  (`'36px'`, `'64px'`, `'70px'`, `'84px'`) carry over verbatim; `model` becomes width-less
  (e.g. `width: ''` / omitted → the slack-absorbing col), `family` becomes a fixed px width
  (e.g. `'120px'`).
- **Delete** `gridTemplate` (the `useMemo` joining widths) and both
  `style={{ gridTemplateColumns: gridTemplate }}` usages.
- Render the list branch as:
  ```tsx
  <div className="l-scroll" data-testid="cat-model-list">
    <table className="cat-table">
      <colgroup>
        {visibleColumns.map((c) => <col key={c.key} style={c.width ? { width: c.width } : undefined} />)}
      </colgroup>
      <thead className="cat-listhead" data-testid="cat-listhead">
        <tr>
          {visibleColumns.map((col) => col.sort
            ? <th key={col.key} scope="col" className={col.align === 'right' ? 'cat-th--right' : undefined}>
                <ColSort col={col.sort} label={col.label} sort={sort} order={order} align={col.align ?? 'left'} onSort={onSort} />
              </th>
            : <th key={col.key} scope="col" className={`cat-colhead${col.align === 'right' ? ' cat-colhead--right' : ''}`}>{col.label}</th>
          )}
        </tr>
      </thead>
      <tbody className="l-listview">
        {rows.map((m, i) => <ModelRow key={modelKey(m)} … columns={visibleColumns} idx={…} />)}
      </tbody>
    </table>
  </div>
  ```
  Keep the existing `isLoading` / empty-state branches as siblings (render the `<table>` only
  in the rows branch). Skeleton/empty markup is unchanged.
- `ModelRow` becomes a `<tr>` (drop the `gridTemplate` prop):
  ```tsx
  <tr className={`l-listrow cat-row${active ? ' active' : ''}`} onClick={onSelect}
      role="option" aria-selected={active} data-testid={`cat-model-row-${model.slug}-${model.model_id}`}>
    {columns.map((col) => col.key === 'num'
      ? <td className="cat-num-td" key="num">
          <LinkRow onActivate={onSelect} label={`Open ${model.name}`} />
          <span className="cat-num">#{idx}</span>
        </td>
      : <td key={col.key} className={col.align === 'right' ? 'cat-td--right' : undefined}>{col.cell(model)}</td>
    )}
  </tr>
  ```
  `ColSort` is unchanged. Move `data-testid="cat-listhead"` from the old header div onto `<thead>`.

### 2. Providers screen — `routes/models/explore/api-providers/-components/ExploreProvidersScreen.tsx`

- **Delete** the standalone `<div className="cat-listhead cat-prov-grid">…</div>` (currently
  outside `.l-scroll`) and the `cat-resultbar`-adjacent header.
- Move the header **into** a `<table>` inside `.l-scroll` (fixes the horizontal-scroll
  inconsistency for free):
  ```tsx
  <div className="l-scroll" data-testid="cat-prov-list">
    <table className="cat-table">
      <colgroup>
        <col style={{ width: '36px' }} /><col style={{ width: '30px' }} /><col /><col style={{ width: '1%' }} />
      </colgroup>
      <thead className="cat-listhead" data-testid="cat-listhead">
        <tr><th scope="col">#</th><th scope="col" aria-hidden="true" /><th scope="col">PROVIDER</th><th scope="col" className="cat-th--right">MODELS</th></tr>
      </thead>
      <tbody className="l-listview">{rows.map((p, i) => <ProviderRow … idx={…} />)}</tbody>
    </table>
  </div>
  ```
  Grid `36px 30px 1fr auto` → cols `44px`, `38px`, width-less (absorbs slack ≈ `1fr`), and
  a fixed `88px` for the MODELS count column. NOTE: under `table-layout: fixed` a `1%` col
  really renders at ~1% of the table width (it does NOT shrink-to-content), so give count/score
  columns an explicit px width — confirmed in Chrome where `1%` collapsed the MODELS column to
  ~10px and clipped the count.
- `ProviderRow` becomes a `<tr class="l-listrow cat-row">` with four `<td>`s; `LinkRow` is the
  first child of the first (`#`) `<td>`. Cell content wrappers (`.cat-logo`, `.cat-body`,
  `.cat-score`) are reused verbatim. Keep `role="option"`, `aria-selected`,
  `data-testid={`cat-prov-row-${provider.slug}`}`. The `cat-resultbar` block above the list
  stays as-is.

### 3. Shared CSS — `routes/models/explore/-shared/catalog.css`

**Delete (grid-era, now dead):**
- `.cat-prov-grid { grid-template-columns: … }`
- `.cat-listhead.cat-model-grid { min-width: max-content }`
- `[data-testid='cat-model-list'] .l-listview { min-width: max-content }`
- `.cat-row.cat-model-grid { min-width: 100% }`
- the long width-sync comment block (lines ~273–293)
- the `display: grid; gap; align-items; padding` bodies of `.cat-listhead` and `.cat-row`
  (rewritten below)

**Add / rewrite (all scoped to the catalog):**
```css
.cat-table { width: 100%; min-width: max-content; border-collapse: collapse; table-layout: fixed; }

/* Sticky header — same visual treatment, now on thead/th. */
.cat-listhead { position: sticky; top: 0; z-index: 5; background: hsl(var(--background)); }
.cat-listhead th { padding: 6px 16px; border-bottom: 1px solid hsl(var(--border)); font-size: 11px;
  font-weight: 600; letter-spacing: 0.04em; color: hsl(var(--muted-foreground)); text-align: left;
  vertical-align: middle; white-space: nowrap; }
.cat-th--right { text-align: right; }

/* Body — override the shared .l-listrow/.l-listview flex back to table semantics. */
.cat-screen tr.l-listrow { display: table-row; }          /* keeps inherited position: relative (LinkRow offset parent) */
.cat-screen tbody.l-listview { display: table-row-group; }
.cat-screen .l-listrow > td { padding: 12px 16px; vertical-align: middle; overflow: hidden; text-overflow: ellipsis; }
.cat-td--right { text-align: right; }

/* Lift cell content above the stretched .l-rowlink WITHOUT making any td an offset parent. */
.cat-screen .l-listrow > td > * { position: relative; z-index: 1; }
.cat-num-td { white-space: nowrap; }
```

**Keep AS-IS (content styles, container-independent):** `.cat-model-name`, `.cat-num-cell`(+`.free`),
`.cat-cell-text`, `.cat-body`, `.cat-name`, `.cat-sub`, `.cat-shape`, `.cat-caps`,
`.cat-model-caps`, `.cap-chip`/`.cap-*`, `.cat-score`(+num/lbl), `.cat-logo`/`.cat-tint-*`,
`.cat-status*`, `.cat-colsort`(+label/svg/`--left`), `.cat-colhead`(+`--right`), `.cat-num`.

**Simplification win:** with `table-layout: fixed` + `td { overflow: hidden; text-overflow: ellipsis }`,
each cell clips to its `<col>` width structurally — the deleted `min-width: max-content`
width-sync rules and any per-cell shrink hacks are no longer needed for icons/badges/labels.

### Vimium / link-hint gotcha (the content-lift rule must exclude `.l-rowlink`)

The content-lift rule lives at `.cat-screen .l-listrow > td > *:not(.l-rowlink)`. The `:not(.l-rowlink)`
is **load-bearing**: `LinkRow` renders its empty `<a class="l-rowlink">` as a direct child of the first
`<td>`, so a naive `> td > *` rule re-applies `position: relative` to the anchor and collapses it to a
0-width point. Vimium (and other link-hint / keyboard tools) only hint elements with a real clickable
area, so the per-row link hint disappears — losing the fast row navigation the old grid layout had.
Excluding `.l-rowlink` lets it keep list.css's `position: absolute; inset: 0 0 0 3px`, which stretches
the anchor across the whole `<tr>` (the offset parent) → one full-row link target per row, exactly as
before. This mirrors the `:not(.l-rowlink)` exclusion already used by list.css's control-raising rule.

### LinkRow inside a `<td>` — why this is robust

A bare `<a>` is not valid as a direct child of `<tr>` (the parser hoists it out), so `LinkRow`
goes inside the first `<td>`. `.l-rowlink` is `position: absolute; inset: 0 0 0 3px`; with the
`<tr>` `position: relative` (inherited from `.l-listrow`) and the host `<td>` left
**unpositioned**, the anchor resolves to the `<tr>` and fills the **whole row** (not just the
cell) — correct in all evergreen browsers. The `> td > *` rule lifts each cell's content above
the link so text stays visible and controls stay clickable. We must **not** add
`position: relative` to any `<td>`, or the anchor would only cover that one cell.

## Sequencing

1. **Models first** (the reported bug). Edit `ExploreApiScreen.tsx` + the table CSS. Run
   `cd crates/bodhi && npm test -- api/index.v2.test.tsx`.
2. **Providers second.** Edit `ExploreProvidersScreen.tsx`. Run
   `npm test -- api-providers/index.v2.test.tsx`.
3. **Full UI suite:** `cd crates/bodhi && npm test` (+ `npm run lint`, `npm run format`).
4. **E2E:** `make build.dev-server` then run `api-explore.spec.mjs` from
   `crates/lib_bodhiserver/tests-js` (per the e2e workflow — dev-server + Vite HMR, no
   ui-rebuild needed).

## Verification (Chrome, against live app at :1135)

Use `make app.run.live` (live Vite, no rebuild). For each screen:

**Models — `http://localhost:1135/ui/models/explore/api/`:**
- Toggle optional columns via the column picker → **header and body stay aligned** (the bug).
- Open the detail rail → list narrows, one shared horizontal scrollbar appears, header
  scrolls horizontally *with* the body and sticks on vertical scroll.
- Sort headers toggle direction; MODEL column absorbs slack and ellipsizes long names;
  numeric columns right-aligned.
- Keyboard ↑/↓/Home/End move + open the rail; row click selects.

**Providers — `http://localhost:1135/ui/models/explore/api-providers/`:**
- Header now scrolls horizontally consistently with rows; sticks on vertical scroll.
- Sort buttons, rail open/close, keyboard nav, row selection all work.

## Test impact

- Component tests pass **unchanged** (selector-based: `data-testid`, `getAllByRole('option')`,
  text). Watch `within(list).getAllByRole('option')` — `<tr role="option">` inside `<table>`
  is valid for ARIA role queries in jsdom/testing-library; this is the one assertion to check
  if anything regresses.
- E2E page objects unchanged (`data-testid` + `[data-testid^="cat-model-row-"]` prefix +
  `getByRole('option')`).
- `LinkRow` keeps `data-testid="row-link"`; its unit test is structure-agnostic.

## Critical files

- `crates/bodhi/src/routes/models/explore/api/-components/ExploreApiScreen.tsx`
- `crates/bodhi/src/routes/models/explore/api-providers/-components/ExploreProvidersScreen.tsx`
- `crates/bodhi/src/routes/models/explore/-shared/catalog.css`
- `crates/bodhi/src/components/shell/list.css` — **read-only reference**, do not modify
- `crates/bodhi/src/components/shell/useListKeyNav.ts` — **read-only reference**, verify
  `.l-listrow` + `.l-rowlink` contract holds
