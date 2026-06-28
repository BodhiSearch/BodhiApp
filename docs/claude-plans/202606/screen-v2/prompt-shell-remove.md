# Prompt — Retire the ShellSlotsContext migration scaffolding

**For:** a coding agent picking up this task fresh.
**Status:** the work is NOT started. This is a brief, not a plan. Explore the codebase, form your own analysis, and produce your own implementation plan before writing code.

---

## Why this exists

During the V2 screen migration, the app needed migrated screens and not-yet-migrated screens to coexist under a single shell while the migration proceeded screen-by-screen. The seam that made that possible is **`ShellSlotsContext`** (`crates/bodhi/src/components/shell/ShellSlotsContext.tsx`). It is explicitly self-described as **temporary migration scaffolding** — read its file header, and read the "Migration scaffolding to REMOVE when the whole migration completes" and "Deferred architectural improvement" sections of `docs/claude-plans/202606/screen-v2/techdebt.md`.

The migration is now effectively complete: every app screen is V2, there are no feature flags left, and there is no longer any V1/V2 coexistence. So the reason the scaffolding existed is gone — but the scaffolding is still here. It works and is harmless, but it is non-obvious indirection that every new screen now has to learn and thread through. The goal of this task is to **remove it and replace it with the idiomatic end-state the migration always intended.**

## How it works today (so you know what you're removing)

One `<AppShell>` is rendered for all app routes in `routes/__root.tsx`'s `RootShell`. It derives the active section from the pathname, and a screen contributes its rich chrome (breadcrumb, header actions, an optional page sidebar, a detail rail, plus a few layout overrides like rail width / scroll behavior) by **publishing** those slots upward through context. A screen publishes via the `useShellChrome(...)` hook; the root shell **consumes** the published slots via `useShellSlots()` and spreads them onto the single `<AppShell>`.

Two other layout modes branch off the same root component: **bare** routes (login / auth / request-access / oauth callbacks / the standalone consent review) render through `BareLayout`, and **fullscreen** routes (the setup wizard) render the bare `<Outlet/>`. The branch is currently a centralized pathname-prefix switch in `components/shell/resolveShellRoute.ts` — `techdebt.md` already names replacing that prefix switch with route-declared layout (e.g. TanStack Router `staticData.layout` read via `useMatches()`, converging to pathless `_shell`/`_bare` layout routes) as the deferred-but-intended follow-up. **Treat the bare/fullscreen layout seam as in-scope to consider** — the cleanest end-state likely addresses the slots seam and the layout-prefix switch together, since both are forms of "the root component decides the layout for everyone."

## What "done" looks like (outcome, not recipe)

- `ShellSlotsContext.tsx` (the value/setter contexts, `ShellSlotsProvider`, `useShellSlots`, and the `useShellChrome` publish hook) is **deleted**, OR reduced to something that is no longer scaffolding — i.e. a screen no longer "publishes upward through a context to a shell it doesn't own." A migrated screen should own (or directly parameterize) its shell.
- Each screen expresses its chrome (breadcrumb / header actions / sidebar / rail / layout overrides) through a **direct, typed, idiomatic** mechanism — the migration's stated intent was screens passing props to a per-route `<AppShell>`, and/or pathless layout routes. You decide which fits this router setup best after exploring it.
- The bare / fullscreen / app-shell decision is made in a way that's at least as clean as today (ideally route-declared rather than a central pathname switch — but don't over-reach if that balloons the change; it's acceptable to land the slots removal first and leave the layout-prefix switch as a smaller follow-up, as long as you say so).
- **No behavior or visual change** for users: same breadcrumbs, same rails opening/closing, same section-scoped column-resize persistence, same reduced-motion view-transition behavior. This is a structural refactor.
- The test suite is **green**, including the screen tests that currently exercise the seam (see below), and the affected Playwright E2E specs pass.

## What makes this non-trivial (scope you must account for)

- **~23 screens publish via `useShellChrome`** (route components under `crates/bodhi/src/routes/**`). Migrating them is the bulk of the work; do it incrementally, section by section, keeping the suite green between steps.
- **~16 screen test files mount `ShellSlotsProvider` and/or read `useShellSlots`** — many via a local `SlotsConsumer`/`ShellHarness` test harness that renders the published slots so the test can assert the header pill / detail rail. When the publish mechanism changes, these harnesses must change with it. Plan the test-harness story up front — it is part of the design, not an afterthought. A shared test utility that renders a screen inside whatever the new shell mechanism is (replacing the per-file `SlotsConsumer` copies) is worth considering.
- **`__root.tsx` data wiring**: `RootShell` also fetches the shell user (`useGetUser`) and owns the logout handler, gated to app-shell routes only (so login/setup don't fetch). Preserve that gating wherever the AppShell ends up.
- **Layout overrides**: chat publishes non-default shell layout (its own scroll ownership, wider rail, its own `resizeKey`). Whatever replaces the slots must carry these per-screen overrides through.
- **Re-render discipline**: the current context deliberately splits the setter and the value into two contexts so publishers don't re-render on value changes, and requires stable slot nodes. Whatever you build should be at least as disciplined (see the vercel-react-best-practices skill); don't reintroduce render thrash or the "inline component definition" footgun.

## Suggested shape of the work (an outline, deliberately not a solution)

1. **Explore first.** Read `ShellSlotsContext.tsx`, `__root.tsx`, `AppShell`, `resolveShellRoute.ts`, `BareLayout.tsx`, a couple of representative publishing screens (e.g. a catalog screen, App Settings with its sidebar slot, Chat with its layout overrides), and several of the screen test harnesses. Understand the TanStack Router setup in `main.tsx` (file-based routing, `defaultViewTransition`, basepath). Consult the `tanstack-router` skill for pathless/layout-route idioms.
2. **Decide the target mechanism** — per-route `<AppShell>` props, pathless layout route(s) providing the shell, route `staticData.layout`, or a combination. Write down the trade-offs and pick one. This is the key design decision; justify it.
3. **Prove it on one screen + its tests** end-to-end before fanning out (a vertical slice). Land it, verify in the browser and in tests.
4. **Roll out section by section** (chat, models, mcps, settings, users, tokens, …), gating each with the frontend unit suite. Convert the test harnesses as you go.
5. **Delete the scaffolding** once nothing references it, plus the now-stale `techdebt.md` entries. Decide the layout-prefix-switch question (fold in or explicitly defer).
6. **Run the affected Playwright E2E** (the rail/shell-touching specs) and a final full-suite check.

## Guardrails (house rules — confirm against the repo's current CLAUDE.md)

- Trunk-based: focused commits straight to `main`, linear history, run the formatter before each commit. Commit per coherent step, not one giant commit.
- Don't touch generated code (`routeTree.gen.ts`, `@bodhiapp/ts-client`) or vendored `components/ui/*` unless clearly project-customized.
- Gate every step with the frontend unit suite (`cd crates/bodhi && npm test`) + typecheck; the suite must never be left red between commits. Reserve Playwright E2E for step/phase ends and run only the affected specs, then a full run at the end.
- This is a refactor: **preserve all `data-testid`s** and the existing visual output. If you find yourself changing what the user sees, stop — that's a different task.
- Prefer evolving the abstraction over duplicating it; keep deliberate seams, but this particular seam is explicitly marked for removal — the bar is "is it still scaffolding?", not "does it work?".

## Open questions to resolve (with the user if needed)

- Per-route `<AppShell>` props vs. pathless layout routes vs. a hybrid — if your exploration doesn't make one clearly best, surface the trade-off rather than guessing.
- Whether to fold the bare/fullscreen layout-prefix switch (`resolveShellRoute.ts`) into this change or leave it as a smaller follow-up.
- Whether a shared screen-test harness should replace the per-file `SlotsConsumer` copies as part of this work.
