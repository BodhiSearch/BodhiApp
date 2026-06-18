# Selectable list rows → real anchor links (Vimium / keyboard a11y)

> **Status: IMPLEMENTED & verified.** Component named `LinkRow` (per user). All gate checks green
> (eslint + prettier clean on changed files; 67 RTL tests pass; Gate-B live validation on Tokens,
> Settings, Manage Users — accent border light+dark = `--c-lotus-text`, controls raised, console clean
> on single click). Two findings during build, noted inline: (a) the access-requests `RoleSelect` is a
> native `<select>` that **already** has its own `onClick`/`onChange` `stopPropagation`, so the flagged
> ⚠ row-bubble risk was a non-issue — no `.ua-role-cell` wrapper needed; (b) `.setting-row` already had
> `position: relative`. Below is the original plan; the component name is `LinkRow` throughout.

## Context

The HEAD commit `65b4242a [Amir] ListRow design` landed a new pattern in the **`design/` hi-fi
prototype** (`design/bodhi-list.jsx` + `design/bodhi-list.css`): a shared `ListRow` that renders an
empty, stretched `<a className="l-rowlink">` (`RowLink`) as the first child of every selectable row.

**Why this matters:** today the migrated screen-v2 list rows are plain `<div onClick=…>` elements.
They are not links, so keyboard-link tools — notably the **Vimium** browser extension's "follow link"
(`f`) hint overlay — never surface them, and screen readers don't announce them as actionable. The
prototype fixes this by laying a real `<a>` over each row (behind the row's own controls), so the row
becomes a link target while its buttons/switches/selects stay clickable.

This plan ports that pattern from the prototype into the **production frontend** (`crates/bodhi/src/`)
for **every selectable row in the screens already migrated to screen-v2**, and documents the
convention so future screen ports apply it automatically.

### Decisions locked with the user

1. **Anchor target = `href="#"` + `onClick`** (mirror the prototype). Selection in all 4 screens is
   local React state (`setSelectedId`) routed through `useViewTransition` — there is **no URL routing**.
   The anchor cannot be a real deep-link; it `preventDefault`s and calls the existing `onSelect`.
2. **One shared `RowLink` component** reused by all rows (and future V2 screens) — not inlined per screen.
3. **Port the selected-row accent border too** (full visual parity with the HEAD design), not just the
   anchor behavior: the selected row gets the app-wide left accent in `--c-lotus-text`, and a
   status-accent row recolors its existing stripe on selection instead of drawing a second bar.

## Scope — the 4 shipped screens with selectable rows

| Screen | File | Row component | Row class | Inline controls to keep clickable |
|---|---|---|---|---|
| API Tokens | `routes/tokens/index.tsx` | `TokenRow` (~L218) | `l-listrow tk-row` | status `<Switch>` (`role=switch`, in `.tk-status` w/ stopPropagation) |
| Manage Users | `routes/users/-components/ManageUsersV2.tsx` | `UserRow` (~L62) | `l-listrow mu-row` | none inline (role Badge is a static `<span>`) |
| Access Requests | `routes/users/access-requests/index.tsx` | `RequestRow` (~L302) | `l-listrow ua-row` | `RoleSelect` combobox (in `.ua-role-cell`, **no stopPropagation today** ⚠), approve/reject buttons (in `.ua-act` w/ stopPropagation) |
| App Settings | `routes/settings/-components/SettingsPageV2.tsx` | `SettingRow` (~L93) | **`setting-row`** (bespoke CSS grid, **not** `l-listrow`; already `position:relative`) | edit pencil `<button>` (`.row-edit-btn`, already stopPropagation) |

⚠ Access-requests `RoleSelect` sits in `.ua-role-cell` with **no** `onClick stopPropagation` wrapper.
Its trigger is a `<button>` so the z-index raise keeps it clickable above the link, but a click on it
still bubbles to the row `<div onClick>` and would select the row. Confirm in Gate B; if selecting a
role spuriously opens/changes the rail, wrap `.ua-role-cell` interactive content in a
`onClick={(e) => e.stopPropagation()}` div (same fix already used by `.tk-status` / `.ua-act`).

Out of scope: the Settings **sidebar group nav** (`onNavigate` scroll-spy buttons, ~L303 of
SettingsPageV2) — those are navigation chips, not master-detail rows; they do not get a RowLink. Old
(non-v2) UI is untouched. Batch 3 (Models) is all forms — no rows — so nothing to do there.

## Implementation

### 1. Shared `RowLink` component — `crates/bodhi/src/components/shell/RowLink.tsx` (new)

```tsx
interface RowLinkProps {
  /** Runs the row's select handler (open detail rail). Same callback the row's onClick uses. */
  onActivate: () => void;
  /** Accessible name announced by SR / shown by link-hint tools. */
  label?: string;
}

/**
 * Empty, stretched <a> that turns a selectable list row into a real link target so
 * keyboard / link-hint tools (e.g. Vimium) surface the whole row. It fills the row but sits
 * BEHIND the row's own controls (see `.l-rowlink` + the control-raising selector in list.css),
 * so buttons / selects / switches keep working and a normal mouse click still selects the row.
 * Selection is local state (no URL), so this is href="#" + preventDefault — not a navigable link.
 */
export function RowLink({ onActivate, label }: RowLinkProps) {
  return (
    <a
      className="l-rowlink"
      href="#"
      aria-label={label ?? 'Open details'}
      data-testid="row-link"
      onClick={(e) => {
        e.preventDefault();
        e.stopPropagation();
        onActivate();
      }}
    />
  );
}
```

Export it from the shell barrel `crates/bodhi/src/components/shell/index.ts` alongside the other shell
primitives.

`stopPropagation` matters: the row `<div>` keeps its own `onClick` (so mouse clicks on dead-zones /
text cells still select). Without `stopPropagation`, an anchor click would also bubble to the div's
`onClick` and fire `onSelect` twice → a double `useViewTransition` on the same id. `onSelect` is
idempotent (sets the same id) so it's not a correctness bug, but the double view-transition can jank;
stop the bubble.

### 2. CSS

**`crates/bodhi/src/components/shell/list.css`** — add near the `.l-listrow` rules (~L344-366). The
control-raising selector is the crux: the rowlink sits ABOVE the static cells (so Vimium finds it at
the row corners) while real controls are raised ABOVE the link so they stay clickable.

```css
.l-listrow { position: relative; /* …existing… (add position:relative) */ }

/* Stretched row link (keyboard / Vimium nav + a11y).
   Sits above static cells; only real controls are raised above it (selector below).
   inset starts 3px from the left so it never overlaps the .accent border-left / .active
   inset accent stripe (no fighting at the left edge). */
.l-rowlink {
  position: absolute;
  inset: 0 0 0 3px;
  z-index: 0;
  border-radius: inherit;
  text-decoration: none;
  -webkit-tap-highlight-color: transparent;
}
.l-rowlink:focus-visible {
  outline: 2px solid hsl(var(--ring));
  outline-offset: -3px;
  border-radius: 4px;
}
/* Raise real CONTROLS above the link so they stay clickable; the rest of the row is the link. */
.l-listrow button, .l-listrow select, .l-listrow input, .l-listrow textarea,
.l-listrow label, .l-listrow a:not(.l-rowlink),
.l-listrow [role='switch'], .l-listrow [role='button'], .l-listrow [tabindex]:not(.l-rowlink) {
  position: relative;
  z-index: 1;
}
```

Selected-row accent border (port from `design/bodhi-list.css` HEAD), light + dark:

```css
/* selected row — app-wide identifiable left accent (--c-lotus-text) */
.l-listrow.active { box-shadow: inset 3px 0 0 var(--c-lotus-text); }       /* add to existing .active */
/* an accent (status-stripe) row that is selected recolors its existing stripe — no double bar */
.l-listrow.accent.active { box-shadow: none; border-left-color: var(--c-lotus-text); }

/* dark (existing block ~L672) */
[data-theme='dark'] .l-listrow.active,
.dark .l-listrow.active { box-shadow: inset 3px 0 0 var(--c-lotus-text); }  /* was --primary/.7 */
[data-theme='dark'] .l-listrow.accent.active,
.dark .l-listrow.accent.active { box-shadow: none; border-left-color: var(--c-lotus-text); }
```

Notes:
- `--primary-ring` from the prototype does **not** exist in production — use `hsl(var(--ring))` (the
  shadcn ring token, defined in `styles/globals.css`). This is the one prototype line to NOT copy verbatim.
- Starting the link at `inset: 0 0 0 3px` (with `outline-offset: -3px`) is the robust way to keep the
  link clear of the `.accent` `border-left: 3px` and the dark `.active` `inset 3px` box-shadow stripe,
  rather than relying on paint order. The dark inset accent is painted by the row `<div>` itself (not a
  child), so it still shows under the layered link — but eyeball it in Gate B.

**`crates/bodhi/src/components/shell/settings.css`** — `SettingRow` uses the bespoke `.setting-row`
CSS grid. It **already has `position: relative`** (no change to that rule). Its selectors are scoped
under `.settings-screen`, so match that convention. Add the `.l-rowlink` rule scoped here too (the
`.setting-row` `.modified` stripe is a real `border-left: 3px`, so an `inset: 0` link is laid out
inside the border box and does **not** cover the stripe — no `0 0 0 3px` adjustment needed here):

```css
.settings-screen .setting-row > .l-rowlink {
  position: absolute;
  inset: 0;
  z-index: 0;
  border-radius: inherit;
  text-decoration: none;
  -webkit-tap-highlight-color: transparent;
}
.settings-screen .setting-row > .l-rowlink:focus-visible {
  outline: 2px solid hsl(var(--ring));
  outline-offset: -2px;
  border-radius: 4px;
}
.settings-screen .setting-row button, .settings-screen .setting-row select,
.settings-screen .setting-row input, .settings-screen .setting-row a:not(.l-rowlink),
.settings-screen .setting-row [role='button'], .settings-screen .setting-row [tabindex]:not(.l-rowlink) {
  position: relative;
  z-index: 1;
}
```

The link `inset: 0` correctly spans both grid rows (key/value row + `.row-desc` row) so the whole
card is the link target. The shared `RowLink` component reuses the `l-rowlink` class; the
`.settings-screen`-scoped block above supplies the matching geometry + control-raise inside settings.

### 3. Per-screen edits — insert `<RowLink>` as the first child of each row

Import `RowLink` from `@/components/shell` and add it immediately inside the row `<div>`, before the
existing cells. Derive a human label from the row entity:

- **`TokenRow`**: `label={`Open token ${token.name || 'Unnamed token'}`}`
- **`UserRow`**: `label={`Open user ${user.username}`}`
- **`RequestRow`**: `label={`Open access request from ${request.username}`}`
- **`SettingRow`**: `label={`Open setting ${setting.key}`}`

(Action-phrased labels read better in Vimium/SR link lists than a bare name.) `onActivate={onSelect}`
in all four — `onSelect` is already the prop wired to the screen's `select…(id)` callback.

Keep the row div's existing `onClick={onSelect}` and `data-testid`. No change to the inner controls'
existing `stopPropagation` wrappers — the z-index selector is what keeps them clickable above the link;
their `stopPropagation` still prevents the row-div `onClick` from firing on control clicks.

### 4. Controls verified clickable above the link

The z-index selector covers every inline control across the 4 screens:
- Tokens — `<Switch>` renders `<button role="switch">` ✔
- Access Requests — `RoleSelect` trigger is a `<button role="combobox">` ✔, approve/reject `<button>` ✔
- Settings — edit `<button>` ✔
- Manage Users — no inline controls; harmless

The raise only ensures controls receive the **click** (above the link). It does **not** stop that
click from bubbling to the row-div `onClick`. Controls already wrapped in `stopPropagation`
(`.tk-status`, `.ua-act`, settings edit button) are fine. The **only** uncovered case is the
access-requests `RoleSelect` in `.ua-role-cell` — see the ⚠ note in Scope; resolve during Gate B by
adding a `.ua-role-cell` stopPropagation wrapper if needed.

## Tests

**RTL (Vitest) — extend the existing v2 test files** (do not add new files):
`routes/tokens/index.v2.test.tsx`, `routes/users/index.v2.test.tsx`,
`routes/users/access-requests/index.v2.test.tsx`, `routes/settings/index.v2.test.tsx`.

Per screen, scope the link query under the row's existing `data-testid` (the shared `data-testid` on
the link is the generic `row-link`; uniqueness comes from the row scope):

```tsx
const row = screen.getByTestId('token-row-token-1');
const link = within(row).getByTestId('row-link');
expect(link.tagName).toBe('A');
expect(link).toHaveAccessibleName(/Open token Production API/i);
await user.click(link);
expect(await screen.findByTestId('token-detail-rail')).toBeInTheDocument(); // rail opens via anchor
```

Plus, per screen:
- Existing control tests still pass unchanged (Switch toggle, approve/reject, settings edit) — proves
  controls remain clickable above the link.
- One explicit assertion that activating a control does **not** open the detail rail (stopPropagation
  intact) — e.g. clicking the token status switch leaves the rail closed; for access-requests, clicking
  `role-select-<username>` opens the combobox **without** selecting the row (guards the ⚠ RoleSelect case).

**Optional shared unit test** `components/shell/RowLink.test.tsx`: render `<RowLink onActivate={fn}
label="x" />`, assert it's an `<a>` with `aria-label="x"`, click fires `onActivate` exactly once and
calls `preventDefault`/`stopPropagation`. Pins the load-bearing behavior independent of the screens.

**E2E (Playwright)** — black-box only (memory: no `page.evaluate`/context fetch). Existing
row-selection specs already cover opening the rail via click; optionally add one keyboard-link
assertion driven purely through the UI (focus the row link via Tab, press Enter, assert rail opens).
Set `reducedMotion: 'reduce'` for any spec asserting on rail-open to avoid view-transition detach races
(per the carried-forward retro rule), and wait for mutation settle before asserting.

## Docs — make future ports apply this automatically

1. **`docs/claude-plans/202606/screen-v2/process.md`** — in the per-screen migration recipe (§ around
   L59-75, between the "strip prototype idioms" step and the "preserve `data-testid` + ARIA" step), add:

   > **Selectable rows are anchor links.** Any master-detail list row (a row that opens the detail
   > rail) must render the shared `RowLink` (`components/shell/RowLink.tsx`) as its first child:
   > `<RowLink onActivate={onSelect} label={<human name>} />`. This makes the row a real `<a>` target
   > so keyboard/link-hint tools (Vimium) and screen readers can reach it. The row keeps its own
   > `onClick`; inner controls (buttons/selects/switches) stay clickable because the
   > control-raising z-index selector in `list.css` (and `.setting-row` in `settings.css`) lifts them
   > above the stretched link. Selection is local state, so the link is `href="#"` + `preventDefault`.

2. **`docs/claude-plans/202606/screen-v2/common-prompt.md`** — add a one-line operating rule pointing
   at the recipe step: "Selectable list rows must use the shared `RowLink` anchor (see @process.md
   per-screen recipe) — keyboard/Vimium accessibility."

3. **`docs/claude-plans/202606/screen-v2/batch-3-models-kickoff.md` / future kickoffs** — no row work
   for Models (forms only); note that Batch 4 MCP Discover's instance list **will** need RowLink.

## Verification (run after implementing)

1. `cd crates/bodhi && npm run lint && npm run format` — strict TS, no `any`.
2. `cd crates/bodhi && npm test` — RTL incl. the 4 updated v2 test files green.
3. **Gate B — live in Claude-in-Chrome** (`make app.run.live`): on each of `/tokens`,
   `/users`, `/users/access-requests`, `/settings`:
   - Install/enable Vimium, press `f`, confirm each row now gets a link hint.
   - Click a row's static cell → rail opens (mouse path intact).
   - Click an inline control (token Switch, approve/reject, settings edit, role select) → the control
     acts and the rail does **not** open (controls above the link, stopPropagation intact).
   - Tab to a row link, Enter → rail opens (keyboard path).
   - Selected row shows the left accent border in **light AND dark**; a status-accent row recolors its
     stripe instead of drawing two bars.
   - Console clean (0 errors) on load and interactions; responsive (narrow viewport) unaffected.
4. `make test.e2e` (from `crates/lib_bodhiserver/tests-js`) — existing + any new black-box specs green.
5. Commit per screen-v2 convention (single focused commit; `make format` + gate checks first).
