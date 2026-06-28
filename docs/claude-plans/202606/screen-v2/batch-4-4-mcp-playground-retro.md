# Batch 4-4 — MCP Playground Screen-V2 — Retrospective

Status: **complete, V2-only (no flag), live-verified + RTL + E2E (standalone + multi_tenant).** Migrates the
legacy 2-pane tool-only playground onto the V2 `AppShell` chrome with a 3-pane layout covering five MCP
capabilities (Overview, Tools, Prompts, Resources, Templates) and readable result rendering.

## What landed

**3-pane shell on shared chrome.** `useShellChrome({ sidebar, rail, headerActions, breadcrumb })` drives
the layout:

- **Sidebar (left)** — `InstancePicker` (writes `?id=` and resets feature/item) + `CapabilityNav` with
  per-feature counts and muted-when-empty styling.
- **Rail (right)** — `PlaygroundRail` (config-driven `ShellSearch` + `useListKeyNav`), one config per
  non-Overview capability. Auto-closed on Overview (`railDefaultOpen={feature !== 'overview'}`).
- **Center** — `OverviewView` dashboard or one of `ToolDetail` / `PromptDetail` / `ResourceDetail` /
  `TemplateDetail`. All slot nodes memoised; `ShellSlots` accepts both `sidebar` and `rail`
  simultaneously.

**URL-driven state.** Route `validateSearch` is `{ id, feature, item }` (`feature` enum default
`'overview'`). All non-instance navigation goes through `useViewTransition()` + `replace:true`
(the `select()` pattern from `MyMcpsScreen`).

**Data layer — single hook, all four list methods.** `useMcpClient` keeps its `useRef`-stateful SDK
pattern (NOT TanStack Query). After `listTools`, `connect()` now also runs `listPrompts` /
`listResources` / `listResourceTemplates` **each guarded** — `Method-not-found` / capability-absent
downgrades to an empty list instead of an error. `McpClientTool` gained `annotations` + `title` (mapped
in `toolMapping.ts`). `McpToolCallResult` now exposes `structuredContent`. New surface methods:
`getPrompt(name, args)`, `readResource(uri)`. The hook exposes `counts` (used by `CapabilityNav` +
`OverviewView`) and a unified `refresh()`.

**Readable-by-default results.** `ResultPanel` shows Readable / Raw / Request tabs with a copy button
and a status pill (`data-test-state=success|error|idle|running`). `ReadableResult` ports the prototype
renderers (`ContentBlock`, `ToolResultView`, `Markdownish`, `DataView`, `MessagesView`,
`ResourceContentsView`). Tool result `resource_link` blocks deep-link into Resources via an
`openResource(uri)` callback (`?feature=resources&item=<uri>` + `replace:true` view transition), and
the resource is read live in `ResourceDetail`.

**Per-capability detail views.**

- `ToolDetail` — `BehaviourHints` (read-only / open-world / annotation badges), schema-driven `ArgForm`,
  Run / Reset, full `ResultPanel`.
- `PromptDetail` — `getPrompt` args via field-list `ArgForm`, "Preview messages" renders role-labelled
  chat bubbles through `MessagesView`.
- `ResourceDetail` — name / URI / MIME header + Read button → readable contents in `ResultPanel`.
- `TemplateDetail` — `extractTemplateVars` over `uriTemplate` (RFC 6570 level-1 `{var}`), live
  "Resolves to" preview with `data-filled` toggle, Resolve & Read → `readResource(resolvedUri)`.

**Tests.**

- **Unit (Vitest + MSW)** — `routes/mcps/playground/index.test.tsx` rewritten on the
  `makeRouteRouter` / `RouteHarness` pattern (real in-memory `RouterProvider` context). 16 specs cover
  Overview counts, capability switching, rail listing/selection per feature, Run/Preview/Read flows,
  result panel state transitions, and template URI substitution. `test-utils/msw-v2/handlers/
mcp-protocol.ts` got `prompts/list`, `prompts/get`, `resources/list`, `resources/read`,
  `resources/templates/list`.
- **E2E (Playwright)** — new `tests-js/pages/McpPlaygroundPage.mjs` page object owns V2 selectors and
  flows (instance switch, capability nav, rail search/select, per-capability run+result,
  connection-status). `McpsPage.mjs` delegates playground methods to it; spec call-sites updated to
  V2 selectors. `mcps-crud.spec.mjs` `test.step`s now exercise: connect → Overview counts → Tools run
  (echo) → Prompts preview → Resources read → Templates resolve & read.

**Cleanup.** Deleted `ToolSidebar.tsx`, `ExecutionArea.tsx`, `FormInput.tsx`, `ResultSection.tsx`, and
the old `types.ts` (folded into `playgroundTypes.ts`). Renamed `behaviourHints.ts` → `behaviour-hints.ts`
(macOS case-insensitive FS collision with the `BehaviourHints.tsx` component). All V2 tests/specs and
E2E selectors point exclusively at the new test IDs.

## Decisions worth recording

- **Out of scope (next phase):** Elicitation, Sampling, Completion. **Dropped entirely:** any
  "Use in Chat" / "Send to Chat" hand-off — the playground is for exercising the wire protocol.
- **JSON mode is gone.** Schema-driven `ArgForm` is the only input path; the legacy JSON editor +
  Form/JSON toggle have been removed from the UI, page object, and specs.
- **Guarded capability listing.** Servers without `prompts` / `resources` / `templates` capability
  return JSON-RPC `-32601`; each list call is independently try/catch'd → empty list → muted
  CapabilityNav row + EmptyState in the center.
- **`PlaygroundScreen`'s connect effect is keyed on `mcp.path` only**, not on the `mcpClient` identity
  (the hook re-creates a fresh object each render — including it would loop). Documented inline.
- **Templates use RFC 6570 level-1 only** (`{name}`-style placeholders, percent-encoded values, no
  reserved/operator expansion). Sufficient for the everything-server templates and avoids a runtime
  dependency.
- **Rail key-nav** uses the shared `useListKeyNav` from the shell, mirroring `MyMcpsScreen`.
- **TanStack Router context in unit tests.** `routeApi.useSearch()` cannot be stubbed cleanly — the
  in-memory `makeRouteRouter` / `RouteHarness` + `ShellHarness` combo gives us a real router context
  while keeping setup terse.
- **Resource fixtures are dynamic.** The everything server generates many resources, so the E2E step
  selects "the first" rail item rather than hard-coding a URI.

## Open follow-ups

- Wire `Elicitation` / `Sampling` / `Completion` once the upstream SDK lands stable surface APIs.
- Consider a "recent calls" rail section per capability (kept out of P5 to keep the surface tight).
- The known swallowed view-transition `InvalidStateError` (cross-screen) still logs once on first
  capability switch — tracked in the foundation retro, not regressed by this batch.
