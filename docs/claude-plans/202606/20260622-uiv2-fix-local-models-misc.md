# Plan — Explore · Local Models: chip simplification, drop fake "Recommended", remove footer Pull, add Updated column + sort

## Context

The Explore · Local Models discovery screen (`/ui/models/explore/local/`, Batch 3-6) shipped with a
few rough edges the user spotted in the live UI:

1. **Browse chips ("Trending" / "New") stretch the full sidebar width** and carry icons, looking
   visually different from every other facet chip on the screen. They use the `m-facet-pills.nowrap`
   variant (`flex: 1 1 0`) which forces each pill to grow to fill the row.
2. **The quant ("Download options") panel shows a "Recommended" badge** on one quant. The
   recommended feature was never actually implemented — `quant.recommended` is an upstream heuristic
   we don't trust/own — so the badge is misleading and should be removed.
3. **A footer "Pull <quant> · <size>" button** auto-targets the "recommended" quant. With Recommended
   gone, this CTA has no honest basis. The intended interaction is: the user picks a quant from the
   list and downloads it via that row's own ⬇ button (which already exists).
4. **No "Updated" (last-modified) column or sort.** The reference API already supports
   `sort=last_modified` and returns `last_modified` on the Model DTO; the UI even has the label wired
   (`SORT_LABELS.last_modified = 'Updated'`) but never surfaces the column or makes it sortable.

Outcome: chips look consistent and compact, the quant panel is honest (no fake recommendation, no
auto-pick footer — explicit per-quant download), and the list gains a sortable Updated column.

All changes are **frontend-only** (no backend / API / ts-client regeneration). User decisions:
- **Updated column**: render `model.last_modified` from the list response; show `—` when null. No
  pre-flight API verification.
- **Quant download**: keep the existing per-row ⬇ icon button only; no extra selection UI.

## Files to modify

All under `crates/bodhi/src/routes/models/explore/local/-components/`:
- `LocalDiscoverySidebar.tsx` — Browse chips (#1)
- `LocalDiscoveryRail.tsx` — drop Recommended badge + footer Pull (#2, #3)
- `LocalDiscoveryScreen.tsx` — add Updated column + sort header (#4)
- `local-discovery.css` — column grid + Updated cell styling, remove now-dead `.ld-rec-badge` /
  `.ld-quant-row.rec` / `.dp-foot`/`.ld-pull-*` rules (#2, #3, #4)

## Change 1 — Browse chips: compact, no icons, wrap like other chips

In `LocalDiscoverySidebar.tsx`:
- Drop the `icon` field from the `BROWSE` array (remove `trending-up` / `sparkles`) and remove the
  `<ShellIcon name={b.icon} …/>` from the Browse pill render — leave just the label, matching the
  Specialisation / Tag / Language chips which are label-only.
- Remove the `nowrap` modifier on the Browse `m-facet-pills` container (`<div className="m-facet-pills nowrap">` → `<div className="m-facet-pills">`). This drops the `flex: 1 1 0` stretch so each
  pill sizes to its content and wraps, identical to the other facet groups.

No CSS change needed — `.m-facet-pill` already styles a content-width pill; we just stop opting into
the `.nowrap` stretch variant. (The `.m-facet-pills.nowrap` rule in `models.css` stays — it's still
used by the Task group.)

## Change 2 — Remove the fake "Recommended" badge + highlight

In `LocalDiscoveryRail.tsx`, inside `QuantsTab`:
- Remove the `q.recommended && (<span className="ld-rec-badge">…Recommended</span>)` block.
- Remove the `${q.recommended ? ' rec' : ''}` conditional class on `ld-quant-row` (no special
  highlight for a quant we don't actually recommend).

In `local-discovery.css`: delete the now-unused `.ld-quant-row.rec` and `.ld-rec-badge` rules.

> We do **not** touch the `Quant` type's `recommended` field (it's owned by `@bodhiapp/reference-api-types`); we simply stop rendering it.

## Change 3 — Remove the footer "Pull <quant>" button

In `LocalDiscoveryRail.tsx`:
- Delete the entire `{tab === 'quants' && recommended && (<div className="dp-foot">…</div>)}` footer
  block and the `MidEllipsis`-based `ld-pull-*` label markup inside it.
- Delete the now-unused `const recommended = quants.find((q) => q.recommended) ?? quants[0];` line.
- Remove the now-unused `fmtSize` import usage **only if** it becomes dead — it's still used by
  `QuantsTab` for `ld-quant-size`, so keep `fmtSize`. `MidEllipsis` is still used for the quant name,
  so keep it too. Verify no other unused imports remain (e.g. nothing else referenced `recommended`).

In `local-discovery.css`: delete the `.dp-foot .dp-btn`, `.ld-pull-label`, `.ld-pull-verb`,
`.ld-pull-size`, `.ld-pull-name` rules (footer-only).

The per-quant ⬇ button (`.ld-quant-pull`, wired to `onPull(q)`) is the sole download path and stays
unchanged — this satisfies "have user select quant and download it."

## Change 4 — Add a sortable "Updated" column to the list

The API supports `sort=last_modified` and the DTO carries `last_modified`; `SortKey` already includes
it and `SORT_LABELS.last_modified = 'Updated'` exists. Reuse the existing `SortHeader` component and
the `fmtDate` helper.

In `LocalDiscoveryScreen.tsx`:
- **Lift `fmtDate` so it's shared.** `fmtDate` currently lives in `LocalDiscoveryRail.tsx`. Export it
  from there and import it into the screen, OR (cleaner) move `fmtDate` + `MONTHS` into a small shared
  spot. Simplest: `export function fmtDate` in `LocalDiscoveryRail.tsx` and import it in the screen.
  (Avoid duplicating the formatter.)
- **Row**: in `LocalRow`, add an "Updated" cell to the stats group:
  ```tsx
  <div className={`ld-stat${sort === 'last_modified' ? ' sorted' : ''}`}>
    <div className="ld-stat-num ld-stat-date">{fmtDate(model.last_modified)}</div>
    <div className="ld-stat-lbl">UPDATED</div>
  </div>
  ```
  Placed after the Likes stat. `fmtDate` already returns `—` for null, matching the chosen behavior.
- **Header**: in `.ld-listhead` → `.ld-lh-stats`, add
  `<SortHeader label="Updated" col="last_modified" sort={sort} onSort={onSort} />` after Likes.

In `local-discovery.css`:
- The date string is wider than a number; add a `.ld-stat-date` rule with a smaller font / its own
  `min-width` and `white-space: nowrap` so the column doesn't jump. Bump `.ld-stat { min-width }` only
  for the date variant (keep numeric stats compact). Confirm the row + header grids stay aligned
  (both use `grid-template-columns: 40px 1fr auto` with the stats group as the `auto` column, so
  adding a third stat just widens the auto column equally in both — no grid edit needed).

No change to sort state plumbing: `onSort('last_modified')` already calls `setSort` + `resetPaging`
exactly like Downloads/Likes. The result bar ("sorted by **Updated**") already renders correctly via
`SORT_LABELS`.

## Verification

1. **Unit/RTL**: `cd crates/bodhi && npm test` — run the existing Local Discovery test suite. Update
   any test asserting the removed footer (`ld-pull-recommended`), the Recommended badge
   (`ld-quant-rec-*`), or the Browse-chip icon/nowrap; add a test that clicking the Updated sort
   header sets `sort=last_modified` (assert `data-testid="ld-sort-last_modified"` goes `active` and
   the request carries `sort=last_modified`) and that an Updated cell renders the formatted date (and
   `—` for a null-`last_modified` fixture).
2. **Typecheck + lint**: `cd crates/bodhi && npm run lint` — confirm no unused imports left behind
   (`recommended`, any dropped icon).
3. **GATE B (live)**: `make app.run.live`, open `http://localhost:1135/ui/models/explore/local/`:
   - Browse chips ("Trending"/"New") are compact, label-only, and wrap — visually matching the other
     facet pills.
   - Open a model → Download options tab: no "Recommended" badge, no green highlighted row, no footer
     Pull button; each quant row's ⬇ downloads that quant (watch the Downloads panel for progress).
   - List shows an Updated column with formatted dates (or `—`); clicking the Updated header re-sorts
     (rows reorder, header highlights, result bar reads "sorted by Updated").
   - Light + dark + responsive widths; console clean.
4. **E2E** (if the Local Discovery Playwright spec touches the footer/recommended/sort): update
   `crates/lib_bodhiserver/tests-js/` accordingly and run `make test.e2e`. Black-box only;
   `reducedMotion:'reduce'` for the V2 rail.
