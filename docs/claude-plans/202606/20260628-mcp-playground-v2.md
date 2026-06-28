# Plan — Migrate MCP Playground to Screen-V2 (Overview · Tools · Prompts · Resources · Templates)

## Context

The live MCP Playground (`/ui/mcps/playground/?id=<uuid>`) is the last MCP screen still on its pre-V2,
tool-only, 2-pane custom layout (tool sidebar + execution area, raw JSON results). Every other MCP
screen (My MCPs, Explore, forms, server-config) was migrated to the Screen-V2 shell in Batches 4-2 and
4-3. This batch migrates the Playground to its hi-fi prototype: a **3-pane Screen-V2 screen** covering
**five capability areas** — Overview, Tools, Prompts, Resources, Templates — with readable-by-default
result rendering.

The `@modelcontextprotocol/sdk` (v1.29, already installed) natively exposes `listPrompts`/`getPrompt`/
`listResources`/`readResource`/`listResourceTemplates`, so the other four capabilities are a **pure
frontend extension** — no backend / OpenAPI / `@bodhiapp/ts-client` change. SDK return shapes were
verified against `node_modules/@modelcontextprotocol/sdk/dist/esm/client/index.d.ts`.

**Explicitly out of scope** (next phase, `prompt-mcp-playground-advance.md`): Elicitation, Sampling,
Completion (the frontend MCP client doesn't support them yet). **Also dropped entirely:** the prototype's
"Use in Chat" / "Send to Chat" hand-off (verified present on the live prototype Prompts/Tools pages — do
not render those buttons or any chat-navigation).

Source of truth read for this plan: the impl prompt
(`docs/claude-plans/202606/screen-v2/prompt-mcp-playground-impl.md`), the live prototype at
`http://localhost:8000/MCP-Playground-*.html` (all five pages + empty state visually confirmed in
Chrome), the prototype source under `design/mcp-playground/`, and the shipped V2 reference screens.

## Confirmed decisions (from user)

1. **Instance picker = full switcher.** The left-sidebar picker lists all connected MCP instances
   (`useListMcps`); selecting one rewrites `?id=` and reconnects in place. **Only one playground MCP
   connection is open at a time** — on switch, the old connection is **properly closed before** the new
   one opens (the existing `useMcpClient.connect()` already calls `disconnect()` first). These
   connections are **separate** from chat's MCP connections (chat uses `useMcpClients`).
2. **Extend `useMcpClient`** in place (one connection lists all capabilities, guarded per-capability).
3. **Dedicated `McpPlaygroundPage.mjs`** E2E page object; `McpsPage` delegates to it.

## End state (matches prototype, on shell tokens)

3-pane Screen-V2 screen via `AppShell` (`useShellChrome` sidebar/rail/breadcrumb/headerActions slots):

- **Left sidebar**: MCP section nav + an **instance picker** ("ACTIVE MCP": glyph + name + connection
  dot, dropdown to switch) + a **capability nav** (Overview / Tools / Prompts / Resources / Templates)
  each with a **count** and a muted/disabled state when empty/unsupported.
- **Header actions**: instance name + a **connection-status pill** (`Connected` / `Connecting` /
  `Error`, with `data-test-state`) + a **refresh** button.
- **Right rail**: a **searchable list** (`ShellSearch` + `useListKeyNav`) of the current capability's
  items. **Overview has no rail.** Empty state per capability (e.g. "No prompts — This MCP doesn't
  publish any ready-made prompts.").
- **Centre**: the detail/run panel for the selected item, or the Overview dashboard.

| Capability | Rail rows | Centre detail | Run | Result |
|---|---|---|---|---|
| **Overview** | (none) | hero (glyph + name + status pill + desc) · meta facts (Endpoint / Transport / Authentication) · "What you can do here" capability tiles w/ counts | — | — |
| **Tools** | friendly title + `code_name` + behaviour-hint dot | header + behaviour-hint chips + auto-form from `inputSchema` + Run/Reset | `tools/call` | readable (markdown / text / structured / image / resource_link / multi-part) · Result/Raw/Request tabs · copy |
| **Prompts** | title + description | header + arg form + "Preview messages" | `prompts/get` | role-labelled chat bubbles (markdown) · Messages/Raw/Request · copy |
| **Resources** | name + URI subtitle | header + meta (Address / Type) + "Read resource" | `resources/read` | readable contents · Raw toggle · copy |
| **Templates** | name + URI-template subtitle | header + fill-in form + live "**Resolves to**" URI preview + "Resolve & read" | `resources/read` on resolved URI | same readable contents as Resources |

Behaviour-hint vocabulary (only the ones the tool declares; from `annotations`): Read-only / Makes
changes (`readOnlyHint`), Can delete / Non-destructive (`destructiveHint`), Safe to repeat / Repeats add
up (`idempotentHint`), Reaches out / Stays in workspace (`openWorldHint`). Order:
`[readOnlyHint, destructiveHint, idempotentHint, openWorldHint]`.

## Routing & URL state

Keep the single route `/mcps/playground/`. Extend `validateSearch` (mirror `myMcpsSearchSchema` in
`crates/bodhi/src/routes/mcps/index.tsx`):

```ts
validateSearch: z.object({
  id: z.string().optional(),                                                 // MCP instance uuid
  feature: z.enum(['overview','tools','prompts','resources','templates']).optional(), // default 'overview'
  item: z.string().optional(),                                               // tool name / prompt name / resource uri / template uriTemplate
})
```

- Capability nav writes `?feature=`; rail selection writes `?item=` with `replace: true` (no history
  spam) wrapped in `useViewTransition()` — exactly the `select()` pattern in `ExploreMcpScreen.tsx`.
- Instance picker writes `?id=` (full switcher) and resets `feature`/`item` to defaults.
- A tool result's `resource_link` deep-links to `?feature=resources&item=<uri>` (carry just the URI;
  re-read live). Wire via a `PgNavContext`-style `openResource(uri)` callback (prototype `pg-shared.jsx`).

## Data layer — extend `useMcpClient` (`crates/bodhi/src/hooks/mcps/useMcpClient.ts`)

No backend / OpenAPI / ts-client change. Add to the existing single-connection `useRef` hook, mirroring
`listTools`/`callTool`:

- `listPrompts()` → `getPrompt(name, args)` (SDK `getPrompt` returns `{ messages: [{role, content}], description? }`).
- `listResources()` → `readResource(uri)` (SDK `readResource` returns `{ contents: [{uri, text|blob, mimeType?}] }`).
- `listResourceTemplates()` (SDK returns `{ resourceTemplates: [{uriTemplate, name, ...}] }`); resolve a
  template's URI client-side from the fill-in form (RFC 6570 **level-1** `{var}` substitution — no lib
  needed; covers the everything server's `demo://resource/dynamic/text/{resourceId}`), then `readResource(resolvedUri)`.
- On `connect()`, after `listTools`, also call the four list methods **each guarded** (catch
  `Method not found` / capability-absent → empty list, not error) so counts populate and the nav muting
  is data-driven. Store `prompts` / `resources` / `resourceTemplates` arrays + the counts in hook state.
- Surface tool **`annotations`** + **`title`** through `McpClientTool` (extend the type) and
  `mapSdkToolsToClient` (`toolMapping.ts`) so behaviour-hint chips + friendly titles work.

Keep the hook's `useRef` SDK-stateful pattern (not a TanStack Query hook) — it's a live connection, and
the existing connect/disconnect already enforces the single-connection + close-before-reconnect contract
the user requires.

## Components — replace the old set wholesale

New components under `routes/mcps/playground/-components/` (mirror prototype `pg-views`/`pg-render`
decomposition):

- `InstancePicker.tsx` — sidebar instance dropdown (`useListMcps`; pick → `?id=`).
- `CapabilityNav.tsx` — Overview/Tools/Prompts/Resources/Templates nav with counts + muted-when-empty.
- `PlaygroundRail.tsx` — config-driven searchable list (`ShellSearch` + `useListKeyNav`), one config
  per capability (rail title / search keys / row renderer).
- Detail views: `OverviewView.tsx`, `ToolDetail.tsx`, `PromptDetail.tsx`, `ResourceDetail.tsx`,
  `TemplateDetail.tsx`.
- Shared render pieces: `ArgForm.tsx` (auto-form from schema/arg list — required/optional markers, inline
  help; salvage schema→form + `buildDefaultParams`/`cleanParams` logic from the old `FormInput.tsx`/
  `types.ts`), `ResultPanel.tsx` (status pill w/ `data-test-state` + meta + Result/Raw/Request tabs +
  copy — salvage tab/copy logic from old `ResultSection.tsx`), `BehaviourHints.tsx`, and
  `ReadableResult.tsx` (dispatches to markdown / structured-data / image / resource-link / messages
  renderers — port the prototype's `pg-render.jsx` renderers: Markdownish, DataView, MessagesView,
  content-block views).
- `playground.css` — scoped token-based styles (per §3 of impl prompt: reproduce `.pg-*` structurally on
  existing `dp-*`/shell tokens + leaf/saffron/danger/indigo/lotus/muted tone families; **not** a verbatim
  `pg-app.css` port). Reuse shared `EmptyState` (`crates/bodhi/src/components/EmptyState.tsx`) and
  `DetailRail`/`DetailRailRow` (`components/shell/detail-rail/`) where they fit (Overview meta rows,
  Resource Address/Type rows).

Keep the result-model shape from the prototype (`{ content: [block…], structuredContent?, isError? }`)
so the readable renderers are reusable; the **real SDK call replaces the prototype's `useRunner`
simulator**, but the rendered model shape stays the same.

After parity: **delete** `ToolSidebar.tsx`, `ExecutionArea.tsx`, `FormInput.tsx`, `ResultSection.tsx`
(and fold the old `types.ts` into the new components).

## Phasing — thin vertical slices, live-verify in Chrome, grow ONE E2E spec, commit per phase

- **P1 — Shell + Overview + Tools.** 3-pane chrome (instance picker, capability nav w/ counts, tool
  rail), Overview dashboard, full Tools detail (hints, auto-form, Run/Reset, readable result + tabs +
  copy). Establishes the layout + the data-layer extension that lists **all** capabilities (so counts
  work) even though only Tools' detail is wired. Live-verify, E2E, commit.
- **P2 — Prompts.** Prompt list + arg form + messages-preview chat bubbles. Commit.
- **P3 — Resources.** Resource list + read + readable contents; resource-link deep-link from tool results. Commit.
- **P4 — Templates.** Template list + fill-in form + live "Resolves to" preview + resolve & read. Commit.
- **P5 — Polish + cleanup.** Empty-state polish, light/dark + responsive (rail→drawer) pass, console-clean
  check, delete the four old components, write retro. Commit.

Loop per phase: unit tests → `cd crates/bodhi && npm run test` → live-verify via `make app.run.live`
(or `npm run dev`) → grow E2E → gate checks → commit. Frontend-only; do not regenerate types.

## Testing

**Unit (Vitest + MSW), `crates/bodhi`:**
- Rewrite `routes/mcps/playground/index.test.tsx` for the 3-pane structure + add component/hook tests.
  Cover: capability nav switching (`?feature=`), rail selection (`?item=`), each detail view's run path,
  readable rendering (markdown vs structured vs messages vs image vs resource_link), Raw/Request toggles,
  empty states, connecting/error states, instance switch (close-then-reconnect).
- **Extend the MCP protocol MSW handler** (`src/test-utils/msw-v2/handlers/mcp-protocol.ts`, today only
  `tools/list` + `tools/call`) to add `prompts/list`, `prompts/get`, `resources/list`, `resources/read`,
  `resources/templates/list`. Honor the existing handler-ordering rule.

**E2E (Playwright), `crates/lib_bodhiserver/tests-js`:**
- **New `pages/McpPlaygroundPage.mjs`** (3-pane: instance switch, capability nav, rail search/select,
  per-capability run + result, connection-status). `McpsPage` delegates its playground methods to it.
  Update call-sites in `specs/mcps/mcps-crud.spec.mjs`, `specs/mcps/mcps-header-auth.spec.mjs`, and any
  OAuth spec that runs a tool in the playground, to the new methods/testids.
- **Grow ONE playground spec** (extend `mcps-crud.spec.mjs`'s playground journey or a new
  `specs/mcps/mcps-playground.spec.mjs`) with `test.step`s: connect → Overview shows counts → Tools run
  (echo) → Prompts preview → Resources read → Templates resolve & read. The **everything reference
  server (port 55180)** covers the surface; fixtures exist in `fixtures/mcpFixtures.mjs`
  (`EVERYTHING_EXPECTED_TOOLS` / `EVERYTHING_EXPECTED_PROMPTS` / `EVERYTHING_EXPECTED_RESOURCE_TEMPLATES`).
- Established patterns: page-object + `data-testid`; `test.step()` with user-goal names; **no
  `waitForTimeout`** (wait on testid/URL/connection-status); login via `LoginPage.performOAuthLogin`;
  **`reducedMotion: 'reduce'` before navigating** to the V2 rail screen + wait for mutation settle;
  **never `test.skip()` for a missing server** — throw in `beforeAll`. Both **standalone AND
  multi_tenant** projects must stay green.

**Testids** (new, structure-matching): `mcp-playground-instance-picker`,
`mcp-playground-capability-<feature>`, `mcp-playground-rail-item-<id>`, `mcp-playground-run-button`,
`mcp-playground-result-tab-<tab>`, `mcp-playground-result-status` (`data-test-state` success/error),
`mcp-playground-connection-status` (`data-test-state` connected/connecting/error), plus
`-prompt-*` / `-resource-*` / `-template-*` variants. Reuse old testid names only where an element maps
1:1; don't contort the new structure.

## Gate checks before each commit (do not skip any)

- `cd crates/bodhi && npm run format && npm run lint`
- `cd crates/bodhi && npm run test` (unit suite green)
- E2E for touched specs green in **both** projects (`cd crates/lib_bodhiserver && make build.dev-server`
  then `npm run test:playwright` / `:standalone` / `:multi_tenant`; dev-server + live Vite, no ui-rebuild).
- Live **GATE B** in Claude-in-Chrome: light + dark + responsive (rail→drawer); console clean apart from
  the known app-wide view-transition `InvalidStateError` (swallowed by `useViewTransition`).
- Rebase onto `origin/main` and commit directly to `main` (trunk-based, no PR).

## Files

**Modify:**
- `crates/bodhi/src/routes/mcps/playground/index.tsx` — extend `validateSearch`; render the 3-pane screen.
- `crates/bodhi/src/hooks/mcps/useMcpClient.ts` — add prompts/resources/templates list+get/read, counts,
  guarded capability listing; extend `McpClientTool` with `annotations` + `title`.
- `crates/bodhi/src/hooks/mcps/toolMapping.ts` — map `annotations` + `title`.
- `crates/bodhi/src/test-utils/msw-v2/handlers/mcp-protocol.ts` — add prompts/resources/templates methods.
- `crates/bodhi/src/routes/mcps/playground/index.test.tsx` — rewrite for new structure.
- `crates/lib_bodhiserver/tests-js/pages/McpsPage.mjs` — delegate playground methods to the new page object.
- `crates/lib_bodhiserver/tests-js/specs/mcps/mcps-crud.spec.mjs`, `mcps-header-auth.spec.mjs` (+ OAuth
  playground spec) — point at new methods/testids.
- `crates/bodhi/src/CLAUDE.md` / `PACKAGE.md` — note the extended hook + new playground structure.

**Create:**
- `crates/bodhi/src/routes/mcps/playground/-components/`: `InstancePicker.tsx`, `CapabilityNav.tsx`,
  `PlaygroundRail.tsx`, `OverviewView.tsx`, `ToolDetail.tsx`, `PromptDetail.tsx`, `ResourceDetail.tsx`,
  `TemplateDetail.tsx`, `ArgForm.tsx`, `ResultPanel.tsx`, `BehaviourHints.tsx`, `ReadableResult.tsx`,
  `playground.css` (names indicative).
- `crates/lib_bodhiserver/tests-js/pages/McpPlaygroundPage.mjs`.
- `docs/claude-plans/202606/screen-v2/batch-<n>-mcp-playground-retro.md` (P5).

**Delete (after P5 parity):** `ToolSidebar.tsx`, `ExecutionArea.tsx`, `FormInput.tsx`,
`ResultSection.tsx`, old `types.ts` (folded in).

## Verification (end-to-end)

1. `make app.run.live`, open `http://localhost:1135/ui/mcps/playground/?id=<everything-instance-uuid>`.
2. Sidebar instance picker switches MCPs (old connection closes, new opens). Capability nav shows live
   counts; empty capabilities are muted. Connection pill reflects connected/connecting/error.
3. Tools: pick a tool, see behaviour-hint chips + auto-form, Run, see readable result + Result/Raw/Request
   + copy; a `resource_link` in a result deep-links to Resources.
4. Prompts: fill args → Preview messages → chat bubbles. Resources: Read → readable contents. Templates:
   fill → live "Resolves to" → Resolve & read → contents.
5. No "Use in Chat" / "Send to Chat" anywhere. Light/dark + mobile drawer behave.
6. `npm run test` green; playground E2E green in standalone + multi_tenant; GATE B console clean.
