# Remote Access: help-chip affordance + help-aware rail toggle

## Context

On `/ui/tunnels/` (Remote Access), the in-context help links — "How to install it", "It's installed but not detected" — render through `FaqLink` as bare underlined text at `text-sm font-medium`, inheriting full foreground colour. Next to the `outline` "Check again" button they read as heavy, near-primary actions and look unstyled rather than deliberately tertiary. The design prototype treated them as a primary-ish affordance; that is explicitly not wanted.

Two outcomes:

1. Those links become a **quiet help chip**: a borderless rounded pill with a small `?` glyph and muted text, tinted on hover — obviously clickable, obviously below the outline button in rank, and the `?` telegraphs "this opens help".
2. The shell's right-panel toggle in the breadcrumb bar shows a **help icon with an on-state** on this screen, so it is clear the right panel is help (today it is a generic `panel-right` glyph with the tooltip "Toggle detail panel").

Both are built as **reusable components**, not one-off styling: a `HelpChip` primitive any screen can use, and generic rail-toggle slots on the shell that any screen can set.

## 1. `help` button variant + `HelpChip` component

**`crates/bodhi/src/components/ui/button.tsx`** — add to `buttonVariants`:

- variant `help`: `rounded-full font-normal text-muted-foreground hover:bg-muted hover:text-foreground` (transparent at rest; `muted`/`muted-foreground` tokens theme themselves in dark mode).
- size `chip`: `h-7 gap-1.5 rounded-full px-2.5 text-xs [&_svg]:size-3.5` — the `[&_svg]:size-3.5` overrides the base cva's `[&_svg]:size-4`; `cn` is `twMerge(clsx(...))` (`src/lib/utils.ts`), which resolves arbitrary-variant conflicts correctly.

Focus ring, disabled state and icon layout all come from the existing base cva — nothing new.

**New `crates/bodhi/src/components/help-chip/HelpChip.tsx`** (+ `index.ts` barrel, matching the `components/faq-rail/` and `components/detail-rail/` convention):

- `HelpChip` = `forwardRef` wrapper over `Button` with `variant="help" size="chip"`, rendering a leading `CircleHelp` icon before `children`.
- Props: all `ButtonProps` plus optional `icon?: ReactNode` (defaults to `<CircleHelp aria-hidden />`) and `asChild` support, so an external-doc link can be `<HelpChip asChild><a href=…>…</a></HelpChip>`.
- Export from the barrel: `HelpChip`, `HelpChipProps`.

## 2. `FaqLink` composes `HelpChip`

**`crates/bodhi/src/components/faq-rail/FaqRail.tsx:208-232`** — `FaqLink` keeps its exact public API (`id`, `reveal`, `children`, `className`) and its `data-testid={`faq-link-${id}`}`; only its body changes from the hand-rolled underlined `<button>` to `<HelpChip onClick={() => onReveal(id)} …>`. Reveal behaviour (`open` + `flash` + `seq` → `openRail()` + `scrollToEntry`) is untouched.

All 11 call sites in `crates/bodhi/src/routes/tunnels/index.tsx` (lines 170, 171, 174, 175, 263, 348, 493, 511, 555, 599) inherit the new look with no edit — tunnels is `faq-rail`'s only consumer today. No JSX change is needed in `index.tsx` for this part: the existing `flex flex-wrap items-center gap-2` rows already lay the chips out beside "Check again", and the chips sitting inside `<Note>` bodies read correctly as a standalone line.

## 3. Reusable rail toggle with icon + on-state

**New `crates/bodhi/src/components/shell/ShellRailToggle.tsx`** — extracts the inline button at `AppShell.tsx:296-300`:

- Props: `{ icon?: string; title?: string; open: boolean; onToggle: () => void }`; `icon` defaults to `'panel-right'`, `title` to `'Toggle detail panel'`.
- Renders `<button className={cn('shell-icon-btn shell-rail-toggle', icon === 'panel-right' && 'is-panel', open && 'is-on')} aria-pressed={open} aria-label={title} title={title} data-testid="shell-rail-toggle">` with `<ShellIcon name={icon} size={16} />`. `ShellIcon` resolves any kebab lucide name, so `'circle-help'` needs no registry change.
- Export from `components/shell/index.ts`.

**`crates/bodhi/src/components/shell/AppShell.tsx`** — add `railToggleIcon?: string` / `railToggleTitle?: string` to `AppShellProps`, render `<ShellRailToggle … open={isMobile ? railOpen : !railCollapsed} onToggle={toggleRail} />` in `shell-head-actions`.

**`crates/bodhi/src/components/shell/shell.css`** — add an on-state on `.shell-icon-btn.is-on` (`background: hsl(var(--muted)); color: hsl(var(--foreground));`) and scope the existing collapsed flip at `:1215` to `.shell-rail-toggle.is-panel svg`, so a `?` glyph is never mirrored.

**`crates/bodhi/src/components/shell/ShellChromeContext.tsx`** — add `railToggleIcon?` / `railToggleTitle?` to `ShellSlots` and to `useShellChrome`'s destructure, memo body and dep array. `__root.tsx` already does `{...slots}` onto `<AppShell>`, so no change there.

## 4. Tunnels screen publishes the help toggle

**`crates/bodhi/src/routes/tunnels/index.tsx:614-630`** — add to the `useShellChrome({…})` call:

```ts
railToggleIcon: 'circle-help',
railToggleTitle: 'Help & debugging',
```

While here, fix the one hygiene deviation the screen carries: it is the only screen passing `rail` / `railHeader` inline instead of memoized, so `useShellChrome`'s memo re-publishes on every render. Memoize the `<FaqRail …>` node on `faqProps` and hoist `<FaqRailHeader />` to a module-scope constant, matching e.g. `routes/settings/-components/SettingsPageV2.tsx:201-215`.

## Tests

- **New `components/help-chip/HelpChip.test.tsx`** — renders label + icon, fires `onClick`, and `asChild` renders the child element (anchor) carrying the chip classes.
- **`components/faq-rail/FaqRail.test.tsx`** — existing assertions key off `faq-link-*` testids only, so they keep passing; add one case asserting the link renders the `?` icon and is not underlined at rest.
- **`test-utils/shell-harness.tsx`** — `ChromeProbe` renders `<div data-testid="harness-rail-toggle-icon">{railToggleIcon}</div>` so screens can assert their published toggle.
- **`routes/tunnels/index.test.tsx`** — one assertion that the screen publishes `circle-help`; existing rail tests (`index.test.tsx:206-241`) are unaffected.
- No E2E change: nothing greps `shell-rail-toggle` or the old link markup in `crates/lib_bodhiserver/tests-js`, and every `faq-link-*` / `faq-entry-*` testid is preserved.
- No backend change → no `xtask openapi` / `ts-client` regeneration.

## Verification

1. `cd crates/bodhi && npm test -- HelpChip FaqRail tunnels` then the full `npm test`; `npm run lint` and `npm run format`.
2. `make app.run.live` (Vite HMR — no `build.ui-rebuild` needed) and open `http://localhost:11135/ui/tunnels/` in Chrome. Confirm:
   - Step 1's two links render as muted `?` chips beside the "Check again" outline button, tint on hover, and show a focus ring on Tab.
   - Clicking one still opens the rail, expands + scrolls to that entry, and flashes it.
   - The header toggle shows a `?` icon, tooltip "Help & debugging", tinted while the panel is open, untinted when collapsed, and not mirrored in either state.
   - Repeat in dark mode via the footer theme switch.
3. Open any other V2 screen with a detail rail (e.g. `/ui/models/`) and confirm its toggle is unchanged: `panel-right` glyph, still mirrored when collapsed, now tinted while open.
