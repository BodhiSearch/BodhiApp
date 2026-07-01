# Implementation prompt — Migrate MCP Playground to Screen-V2 design (Tools · Prompts · Resources · Templates · Overview)

> This is a **build** prompt for an agent working in the BodhiApp repo. It migrates the live MCP
> Playground React screen to its Screen-V2 design. **Elicitation & Sampling are explicitly OUT of
> scope here** — they are the *next* phase (the frontend MCP client does not support them yet). This
> phase ships only the capabilities the SDK already supports: **Overview, Tools, Prompts, Resources,
> Resource Templates**.

---

## 0. Orient yourself first (do this before writing any code)

**Look at the target design (live + source):**
- Open the running prototype in Chrome: **http://localhost:8000/MCP-Playground-Tools.html** — then
  also visit `MCP-Playground-Overview.html`, `MCP-Playground-Prompts.html`,
  `MCP-Playground-Resources.html`, `MCP-Playground-Templates.html` (same host). Click around: pick the
  instance, switch capabilities in the left nav, select items in the right list, run a tool / preview a
  prompt / read a resource / resolve a template, toggle Result/Raw/Request, copy.
- Read the prototype **source** under `design/`:
  - `design/MCP-Playground-*.html` (the five capability pages; each just sets
    `window.PG_PAGE_CAP = "<cap>"`).
  - `design/mcp-playground/` — the shared JS/CSS the pages load:
    `pg-chrome.jsx` (shell wiring, instance picker, capability nav, breadcrumb, header),
    `pg-views.jsx` (the five detail views), `pg-render.jsx` (ArgForm, ResultPanel, readable
    renderers — markdown, structured data, images, resource links, messages — behaviour hints,
    CopyBtn), `pg-tools.jsx` (tool model + hint definitions), `pg-data.jsx` (mock data shapes),
    `pg-shared.jsx` (helpers), `pg-app.css` (the `.pg-*` styling — reference for structure/spacing,
    **not** to be ported verbatim; see §3).

**Look at the current live screen (what you're replacing):**
- App route: `crates/bodhi/src/routes/mcps/playground/index.tsx` (route
  `/mcps/playground/?id=<uuid>`), with `-components/` `ToolSidebar.tsx`, `ExecutionArea.tsx`,
  `FormInput.tsx`, `ResultSection.tsx`, `types.ts`, and `index.test.tsx`.
- It is **tool-only**: a 2-pane custom layout (tool list + execution area), wrapped in
  `useShellChrome({ breadcrumb })` but **not** using the shell's sidebar/rail slots. Result rendering
  is raw syntax-highlighted JSON in Response/Raw/Request tabs.
- The MCP client hook `crates/bodhi/src/hooks/mcps/useMcpClient.ts` wires only `listTools` +
  `callTool`. The `@modelcontextprotocol/sdk` `Client` (v1.29) **natively supports** `listPrompts`,
  `getPrompt`, `listResources`, `readResource`, `listResourceTemplates` — so adding the other four
  capabilities is a **pure frontend extension, no backend / API / ts-client change**.

**Look at how the *other* MCP V2 screens were built (your style guide):**
- `crates/bodhi/src/routes/mcps/index.tsx` (My MCPs) and `routes/mcps/explore/index.tsx` (Explore) are
  the shipped V2 reference: they use `AppShell` via `useShellChrome` + sidebar/rail slots, the shared
  list+rail kit (`components/shell/`: `AppShell`, `useListKeyNav`, `ShellSearch`, `useViewTransition`,
  `LinkRow`), `dp-*`/shell CSS tokens, and URL-synced selection.
- Retros to read for conventions & gotchas: `docs/claude-plans/202606/screen-v2/`
  `batch-4-2-mcp-screens-retro.md`, `batch-4-3-mcp-design-parity-retro.md`, `mcp-techdebt.md`.

**Read the project rules:** root `CLAUDE.md`, `crates/bodhi/src/CLAUDE.md` (frontend conventions,
hook architecture, testids), `crates/lib_bodhiserver/tests-js/CLAUDE.md` + `E2E.md` (E2E patterns).

---

## 1. What we're building (end state)

The playground becomes a **3-pane Screen-V2 screen** matching the prototype, covering **five
capability areas**, all functional:

- **Left sidebar**: an **instance picker** (pick which connected MCP to explore — server glyph +
  name + connection status) and a **capability nav** linking Overview / Tools / Prompts / Resources /
  Templates, each with a **count** (and a muted/disabled state when that capability is empty or
  unsupported by the server).
- **Right rail**: a **searchable list** of the items for the current capability (tools, prompts,
  resources, or templates). Overview has **no rail**.
- **Centre**: the **detail / run panel** for the selected item, or the Overview dashboard.

**Per capability:**

| Capability | List rail | Centre detail | Run action | Result |
| --- | --- | --- | --- | --- |
| **Overview** | (none) | Connection status, meta facts (endpoint / transport / auth), "what you can do here" capability tiles with counts | — | — |
| **Tools** | tool rows (friendly title + `code-name` + behaviour-hint dot) | header with behaviour-hint chips, auto-built **input form** from `inputSchema`, Run tool / Reset | `tools/call` | readable result (markdown / multi-line text / structured data / images / resource links / multi-part), Result / Raw / Request tabs, copy |
| **Prompts** | prompt rows (title + description) | header + **argument form**, "Preview messages" | `prompts/get` | **messages preview** as role-labelled chat bubbles (markdown rendered) |
| **Resources** | resource rows (name + URI subtitle) | header + meta (address / type), "Read resource" | `resources/read` | readable contents (markdown / text / structured / image), raw toggle |
| **Templates** | template rows (name + URI-template subtitle) | header + **fill-in form** + live "**resolves to**" URI preview, "Resolve & read" | `resources/read` on resolved URI | same readable contents as Resources |

**Cross-cutting (from the prototype, keep):**
- **Readable-by-default rendering** with a **Raw / JSON** toggle + copy always available but secondary.
- **Behaviour-hint chips** on tools (Read-only / Makes changes / Can delete / Repeats add up / Reaches
  out / Stays in workspace — only the ones the tool declares).
- **Empty states** per capability (an MCP may have tools but no prompts, resources but no templates).
  The capability nav reflects what's actually available (counts / muted when empty).
- Loading / connecting / error states styled on-brand.

> **No "Use in Chat" / "Send to Chat".** The prototype links Tools/Prompts off to the chat surface;
> **drop that feature entirely** in this migration. Do not render the "Use in Chat" header action or
> the "Send to Chat" form button on any screen, and do not add any chat hand-off navigation.

---

## 2. Routing & URL state

**Constraint (confirmed):** this codebase routes **with query params, never path params**. Every
dynamic route in `crates/bodhi/src/routes/` uses `validateSearch` (28 files); there are **zero**
`$param` path-route files. (TanStack Router *can* do `$id` path segments, but we stay consistent with
the established convention — do **not** introduce `/mcps/playground/$id/...` paths.)

Keep the single route `/mcps/playground/` and extend its search schema:

```ts
validateSearch: z.object({
  id: z.string().optional(),                       // MCP instance uuid (as today)
  feature: z.enum(['overview','tools','prompts','resources','templates']).optional(),  // default 'overview'
  item: z.string().optional(),                     // selected tool name / prompt name / resource uri / template uriTemplate
})
```

- The capability nav writes `?feature=`; the list rail writes `?item=` (URL-synced selection, like the
  shipped MCP rails — write with `replace` so selection doesn't spam history).
- `feature` defaults to `overview` when absent. `item` selects within the current feature's list.
- Resources opened from a tool result's `resource_link` deep-link to
  `?feature=resources&item=<uri>` (the prototype's "open resource" behaviour) — carry just the URI;
  re-read it live rather than threading name/mime/desc through the URL.

---

## 3. Styling approach — rebuild on shell tokens (NOT a verbatim CSS port)

Per decision: **reproduce the prototype structurally using the existing shell / `dp-*` tokens and
shared primitives, writing minimal new CSS** — the same approach Batch 4-3 used for the server-view
page. Do **not** copy `pg-app.css` and its `.pg-*` vocabulary wholesale.

- Use `AppShell` (via `useShellChrome` + the sidebar/rail/breadcrumb/headerActions slots) for the
  shell — same as My MCPs / Explore. Reuse `ShellSearch`, `useListKeyNav`, `useViewTransition`,
  `LinkRow`/list-row patterns from `components/shell/` for the rail.
- The **instance picker**, **capability nav**, the **detail/run panel**, the **readable renderers**,
  and the **behaviour-hint chips** have no shipped equivalent — build them as new playground
  components with a small, scoped CSS file (e.g. `routes/mcps/playground/-components/playground.css`)
  expressed in the existing token system (the `dp-*` / shell CSS variables, leaf/saffron/danger/indigo/
  lotus/muted tone families), **not** new design primitives. Aim for structural parity with the
  prototype, not pixel-verbatim.
- Match the shell's light/dark theming and the responsive rail→drawer behaviour the other MCP screens
  already have.

---

## 4. Data layer — extend the MCP client hook

Extend `hooks/mcps/useMcpClient.ts` (and its `toolMapping.ts`) to cover all five capabilities. **No
backend, OpenAPI, or `@bodhiapp/ts-client` change** — it's all the browser SDK over the existing proxy
(`credentials: 'include'` already in place).

Add, mirroring the existing `listTools`/`callTool`/`mapSdkToolsToClient` shape:
- `listPrompts()` → `getPrompt(name, args)` (`prompts/get` returns `messages[]`).
- `listResources()` → `readResource(uri)` (`resources/read`).
- `listResourceTemplates()` → resolve a template's URI client-side from the fill-in form, then
  `readResource(resolvedUri)`.
- Surface tool **annotations** (`readOnlyHint`/`destructiveHint`/`idempotentHint`/`openWorldHint`) and
  tool **title** through the client tool type so behaviour-hint chips + friendly titles work (the SDK
  exposes `annotations` and `title` on the tool; extend `McpClientTool` + `mapSdkToolsToClient`).

Decide between extending `useMcpClient` vs. adding sibling capability mappers/hooks (e.g.
`promptMapping.ts`, `resourceMapping.ts`) — keep it consistent with the existing single-connection
hook; the connection is shared, so prefer **one connect** that can list all capabilities (guard each
list call so a server that doesn't support a capability yields an empty list, not an error → drives
the muted nav state). Gracefully handle servers that only implement some capabilities (catch
`Method not found` / capability-absent and treat as empty).

Follow the domain-hook conventions in `crates/bodhi/src/CLAUDE.md` (camelCase hook files, query-key
factories where a TanStack Query hook is appropriate; note the MCP client is SDK-stateful, not a
REST query — keep its current `useRef`-based pattern).

---

## 5. Components — replace the old set wholesale

The v2 structure diverges enough that the four old components are **replaced**, not evolved. Once the
new screen reaches parity, **delete** `ToolSidebar.tsx`, `ExecutionArea.tsx`, `FormInput.tsx`,
`ResultSection.tsx` (salvage their schema→form and tab/copy *logic* into the new components where it
maps cleanly, but not their layout).

Suggested new component set under `routes/mcps/playground/-components/` (names indicative; mirror the
prototype's `pg-views`/`pg-render` decomposition):
- `InstancePicker.tsx` — sidebar instance dropdown (list of the user's connected MCPs; pick → `?id=`).
- `CapabilityNav.tsx` — the Overview/Tools/Prompts/Resources/Templates nav with counts + muted-empty.
- `PlaygroundRail.tsx` — the searchable list rail (config-driven per capability; reuses
  `ShellSearch` + `useListKeyNav`).
- Detail views: `OverviewView.tsx`, `ToolDetail.tsx`, `PromptDetail.tsx`, `ResourceDetail.tsx`,
  `TemplateDetail.tsx`.
- Shared render pieces: `ArgForm.tsx` (auto-form from a schema/arg list, required/optional markers,
  inline help, validation), `ResultPanel.tsx` (status pill + meta + Result/Raw/Request tabs + copy),
  `BehaviourHints.tsx`, and the readable renderers (`ReadableResult` dispatching to markdown /
  structured-data / image / resource-link / messages views).
- `playground.css` — the scoped token-based styles (per §3).

Keep the **run simulator → real** mapping faithful: the prototype's `useRunner` fakes a delay then
builds a result model; replace with the **real** SDK call, but keep the same result-model shape so the
readable renderers are reusable. The wire shapes are `tools/call`, `prompts/get`, `resources/read`
(shown in the Request tab).

---

## 6. Phasing — thin vertical slices, commit per phase

Build phase-by-phase; **verify each slice live in Chrome (Claude-in-Chrome), grow ONE playground E2E
spec (many `test.step`s), run gate checks, then commit.** (House style: vertical slices, live-verify,
commit per phase.) Plan the work **phase-wise**, not as one big chunk.

- **P1 — Shell + Overview + Tools.** The 3-pane chrome (instance picker, capability nav with counts,
  tool list rail), Overview dashboard (status + meta + capability tiles), full Tools detail (behaviour
  hints, auto-form, Run/Reset, readable result with Result/Raw/Request + copy). This establishes the
  whole layout + the data-layer extension for listing all capabilities (so counts work) even though
  only Tools' detail is wired this phase. **Live-verify, E2E, commit.**
- **P2 — Prompts.** Prompt list + argument form + messages preview (chat bubbles). **Commit.**
- **P3 — Resources.** Resource list + read + readable contents; resource-link deep-link from tool
  results. **Commit.**
- **P4 — Templates.** Template list + fill-in form + live "resolves to" preview + resolve & read.
  **Commit.**
- **P5 — Polish.** Empty-state polish, light/dark + responsive pass, console-clean check, remove the
  old components. **Commit.**

Follow the layered methodology: frontend-only change, so the loop is unit tests → `npm run test` →
live-verify via `make app.run.live` (or `npm run dev`) → grow E2E → gate checks. Regenerate types only
if (unexpectedly) a backend change is needed — it should not be.

---

## 7. Testing — unit + E2E at every layer

**Unit (Vitest + MSW), `crates/bodhi`:**
- Rewrite `routes/mcps/playground/index.test.tsx` for the new structure and add tests for the new
  components/hook paths. Cover: capability nav switching (URL `?feature=`), rail selection (`?item=`),
  each detail view's run path (mock the MCP SDK list/get/read/call via the existing MCP protocol MSW
  handlers — extend them for prompts/resources/templates), readable rendering (markdown vs structured
  vs messages), Raw/Request toggles, empty states, connecting/error states.
- Keep the suite green: `cd crates/bodhi && npm run test`.

**E2E (Playwright), `crates/lib_bodhiserver/tests-js`:**
- **Revamp the page object**: the page structure changes materially, so **rebuild the Playground
  methods in `pages/McpsPage.mjs`** (or extract a dedicated `McpPlaygroundPage.mjs` if cleaner) to the
  new 3-pane structure — capability nav, rail search/select, per-capability run + result. Keep
  consistency with existing method naming where natural, but it's **not a constraint** — port to the
  new structure. Update the call-sites in `specs/mcps/mcps-crud.spec.mjs` and
  `specs/mcps/mcps-header-auth.spec.mjs` (and any OAuth spec that runs a tool in the playground) to the
  new methods/testids.
- **Grow ONE playground spec** (extend `mcps-crud.spec.mjs`'s playground journey, or a new
  `specs/mcps/mcps-playground.spec.mjs`) with `test.step`s covering: connect → Overview shows counts →
  Tools run (echo) → switch to Prompts and preview a prompt → Resources read → Templates resolve & read.
  The **everything reference server (port 55180)** already exposes the surface to test this — fixtures
  exist: `fixtures/mcpFixtures.mjs` `EVERYTHING_EXPECTED_TOOLS` / `EVERYTHING_EXPECTED_PROMPTS`
  (`simple-prompt`, `args-prompt`, `completable-prompt`, `resource-prompt`) /
  `EVERYTHING_EXPECTED_RESOURCE_TEMPLATES`.
- **Follow the established E2E patterns** (from `tests-js/CLAUDE.md` + `E2E.md` + the existing MCP
  specs): page-object model with `data-testid` selectors; `test.step()` with user-goal phase names;
  **no `waitForTimeout`** — wait on `data-testid` visibility / URL / connection-status; login via
  `LoginPage.performOAuthLogin`; **set `reducedMotion: 'reduce'` before navigating** to a V2 rail
  screen (kills view-transition detach races) and wait for mutation settle before asserting; both
  **standalone AND multi_tenant** projects must stay green.
- **Never `test.skip()` for a missing env/server** — if the everything server isn't up, throw in
  `beforeAll` so the failure is loud (house rule).
- E2E commands: from `crates/lib_bodhiserver`, `make build.dev-server` then
  `npm run test:playwright` (or `:standalone` / `:multi_tenant`); dev-server + live Vite (HMR) — no
  ui-rebuild needed.

**Testids:** define new testids that match the new structure (e.g.
`mcp-playground-capability-<feature>`, `mcp-playground-rail-item-<id>`,
`mcp-playground-run-button`, `mcp-playground-result-tab-<tab>`, `mcp-playground-result-status` with
`data-test-state`, plus `-prompt-*` / `-resource-*` / `-template-*`). Add `data-test-state` to async
elements (connection status, result success/error) per the Playwright testability skill. Reuse the
old testid *names* only where an element maps 1:1 — don't contort the new structure to preserve them.

---

## 8. Gate checks before each commit (do not skip any)

- `cd crates/bodhi && npm run format && npm run lint`
- `cd crates/bodhi && npm run test` (unit suite green)
- E2E for the touched specs green in **both** projects (`make build.dev-server` first).
- Live GATE-B verification in Claude-in-Chrome: light + dark + responsive (rail→drawer), console clean
  apart from the known app-wide view-transition `InvalidStateError` (swallowed by `useViewTransition`).
- Rebase onto `origin/main` (trunk-based; commit directly to `main`, no PR — per root CLAUDE.md).
- Update `crates/bodhi/src/.../CLAUDE.md` / `PACKAGE.md` notes and add a
  `docs/claude-plans/202606/screen-v2/batch-<n>-mcp-playground-retro.md` summarizing decisions,
  deferrals (elicitation/sampling), and verification.

---

## 9. Out of scope (this phase)

- **"Use in Chat" / "Send to Chat"** — dropped from this migration entirely (do not render the buttons
  or add any chat hand-off). The chat surface stays untouched.
- **Elicitation & Sampling** (and Completion) — the live MCP client doesn't support them; they are the
  **next** phase (see `prompt-mcp-playground-advance.md` for that design). Do **not** build the
  Elicitation/Sampling pages or the inbox/auto-switch behaviour now.
- Any **backend / API / ts-client** change — this migration is frontend-only. If you think you need a
  backend change, stop and flag it.

---

## Definition of done

All five capability areas (Overview, Tools, Prompts, Resources, Templates) live and functional on the
V2 3-pane shell, matching the prototype structurally on shell tokens; the old tool-only components
deleted; `useMcpClient` extended for prompts/resources/templates; unit suite green; the playground E2E
journey green in both standalone and multi_tenant with a revamped page object; live-verified in
Chrome; shipped V2-only (no flag, consistent with the rest of Batch 4); retro written.
