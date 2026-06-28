# Fix keyboard nav + auto-close in the main section-switcher dropdown

## Context

The main navigation dropdown (the section switcher with **Chat / Models / MCP / Tokens / Users / Settings**)
is hand-rolled, not a shadcn component. Two bugs were reported:

1. **Arrow keys don't work** when the dropdown is open — you can't move between options with ↑/↓.
2. **Selecting an option doesn't auto-close** the dropdown.

The component lives in `crates/bodhi/src/components/shell/ShellNav.tsx`. It has **two render paths**:
- **Expanded sidebar** — an inline `{open && <div className="shell-nav-menu">…</div>}` under the trigger button.
- **Collapsed icon-rail** — the same items inside an `AnchoredPopover` (fixed-position, escapes overflow).

Today the only keyboard handling is `Escape` (inside `AnchoredPopover`), there is **no arrow-key handling at
all on the expanded path**, and the items are plain TanStack Router `<Link>`s with no `onClick` to close.
The flaky document-click listener is the only thing that *sometimes* closes the menu — hence the bug.

**Decision (confirmed with user):** patch the existing hand-rolled component — do **not** migrate to shadcn
`DropdownMenu`. Scope is the **nav dropdown only**; leave the avatar/user menu (`UserMenuPop`) untouched.

### Why we don't reuse `useListKeyNav`

`components/shell/useListKeyNav.ts` is a document-level handler for `.l-scroll`/`.l-listrow` master-detail
lists. It is **eager-select** (it `.click()`s the row on every arrow press, navigating immediately) and
**explicitly bails when focus is inside `.shell-sidebar`/`.shell-rail`** (lines 49-52) — exactly where the nav
dropdown lives. For a section switcher we want *focus-only* arrows + an explicit Enter/click to commit, not
navigate-on-every-arrow. So we add a small local roving-focus handler and only borrow its conventions
(no-wrap, Home/End, ignore modifier keys).

### Why we don't change `AnchoredPopover`

`AnchoredPopover` is shared by **4 consumers** (`ShellNav`, `ShellFilterGroup`, `ChatHistorySidebar`, plus the
barrel export). Baking menu-specific roving-focus into it would affect the filter-chip and chat-history
popovers. All new keyboard/focus logic stays in `ShellNav`; `AnchoredPopover` is left as-is (its existing
Escape + document-click close still serves the collapsed path).

## Implementation

### 1. `crates/bodhi/src/components/shell/ShellNav.tsx` (primary change)

Both render paths share one set of item refs + handlers.

- **State/refs** (near the top of the component): add
  `const itemRefs = useRef<(HTMLAnchorElement | null)[]>([])`, `const [focusIndex, setFocusIndex] = useState(0)`,
  and a `const wasOpen = useRef(false)`. `anchorRef` already exists (typed `HTMLButtonElement`); the two paths
  are mutually-exclusive `return`s so a single ref serves both triggers.

- **Active index**: `const activeIndex = Math.max(0, SHELL_NAV.findIndex((n) => n.id === section))`.

- **Focus-into-menu-on-open / return-focus-on-close** — a `useEffect([open, activeIndex])`:
  - on the `false→true` transition: `setFocusIndex(activeIndex)` and focus the active item
    (`requestAnimationFrame(() => itemRefs.current[activeIndex]?.focus())` — rAF covers the collapsed
    `AnchoredPopover` whose position is set in a layout effect; harmless on the expanded path).
  - on the `true→false` transition: `anchorRef.current?.focus()` to return focus to the trigger.
  - update `wasOpen.current = open` at the end.

- **Per-item keydown** `onItemKeyDown(e, idx)`:
  - ignore when `e.ctrlKey || e.metaKey || e.altKey`.
  - `ArrowDown` → `min(last, idx+1)`, `ArrowUp` → `max(0, idx-1)`, `Home` → 0, `End` → last (no wrap).
  - `Escape` → `e.preventDefault(); setOpenPop(null); return` (needed for the expanded path, which has no
    `AnchoredPopover` wrapper).
  - anything else (incl. **Enter/Space**) → `return` and let the native `<Link>` activate. `preventDefault()`
    + `setFocusIndex(next)` + `itemRefs.current[next]?.focus()` for the movement keys.

- **Rebuild `menuItems`** (currently lines 47-58) to be index-aware. Each `<Link>` gets:
  `ref={(el) => { itemRefs.current[idx] = el; }}`, `role="menuitem"`,
  `tabIndex={idx === focusIndex ? 0 : -1}` (roving tabindex), `onClick={() => setOpenPop(null)}` (this fixes
  bug #2 — and because a focused anchor fires `click` on Enter, it also commits keyboard selection, so **no
  Enter branch is needed**), and `onKeyDown={(e) => onItemKeyDown(e, idx)}`. Keep existing `data-testid`,
  `className` (incl. the `.on` active marker), icon, label, badge.

- **Triggers + menu containers** (a11y, minimal):
  - Expanded `.shell-nav-trigger` button (line 100): add `ref={anchorRef}`, `aria-haspopup="menu"`,
    `aria-expanded={open}`; add `role="menu"` to the `.shell-nav-menu` div (line 117).
  - Collapsed `.shell-railbtn` trigger (line 63): add `aria-haspopup="menu"`, `aria-expanded={open}`; wrap the
    popover's `{menuItems}` in a `<div role="menu">`.
  - Do **not** wire `aria-activedescendant`/ids — roving tabindex + real focus is enough.

Keep the existing document-click `useEffect` (lines 40-45) and `AnchoredPopover`'s listeners. `setOpenPop(null)`
is idempotent, so an item-click close followed by a document-click close is a no-op — no race.

### 2. `crates/bodhi/src/components/shell/shell.css`

`.shell-nav-item` has no focus style, so arrow movement would be invisible. Add after the `:hover` rule
(~line 775):

```css
.shell-nav-item:focus-visible {
  outline: none;
  background: hsl(var(--muted));
  color: hsl(var(--foreground));
  box-shadow: 0 0 0 2px hsl(var(--ring));
}
```

### 3. `crates/bodhi/src/components/shell/ShellChrome.tsx` — no change

Left untouched so `ShellFilterGroup` and `ChatHistorySidebar` are unaffected.

## Verification

### Unit tests — new `crates/bodhi/src/components/shell/ShellNav.test.tsx`

Model on `AppShell.test.tsx` (render via `createWrapper()`, render `<AppShell section=…>` for the expanded
path). **Extend the `Link` mock to forward `ref`** (the existing mock drops it) so focus assertions work:
`Link: React.forwardRef((props, ref) => <a ref={ref} href={props.to} {...rest}>…</a>)`.

Cover: opens & focuses the active item; ArrowDown/Up move focus with **no wrap** at the ends; Home/End jump to
first/last; clicking an item closes the menu (bug #2); Enter (simulated as native click on focused anchor)
navigates + closes; Escape closes and returns focus to the trigger; modified arrows (`metaKey`) are ignored;
a11y — trigger `aria-expanded` toggles, items are `role="menuitem"`, only the focused item has `tabIndex=0`.

Run: `cd crates/bodhi && npm test -- ShellNav`.

### Manual — `make app.run.live` + Chrome

1. **Expanded sidebar**: open the section trigger → active row focused (visible ring); ↓/↑ move and stop at
   ends; Enter navigates + closes; reopen, Esc closes + focus returns to trigger; click a row → navigates +
   closes.
2. **Collapsed rail**: collapse the sidebar, repeat on the icon-rail (`AnchoredPopover` "Go to section") —
   ↓/↑/Enter/Esc/click behave identically.
3. **Regression**: open chat-history popover (`ChatHistorySidebar`) and a filter group (`ShellFilterGroup`) —
   confirm unchanged (Esc/click-out still close, no new focus stealing), proving `AnchoredPopover` is intact.

## Critical files

- `crates/bodhi/src/components/shell/ShellNav.tsx` — all keyboard/focus/auto-close logic.
- `crates/bodhi/src/components/shell/shell.css` — focus-visible style for `.shell-nav-item`.
- `crates/bodhi/src/components/shell/ShellNav.test.tsx` — new unit tests.
- `crates/bodhi/src/components/shell/AppShell.test.tsx` — test-harness pattern to copy (Link mock, wrapper).
- `crates/bodhi/src/components/shell/ShellChrome.tsx` / `useListKeyNav.ts` — read-only references, unchanged.
