# Batch 5 — Chat Page V2 AppShell Migration

> Screen-v2 migration, final batch. Tracker: `screen-v2/tracker.md` (row #1 / Batch 5). Design
> prototype: `http://localhost:8000/Bodhi%20Chat.html` (source mined into
> `chat/bodhi-chat-app.jsx` + `chat/bodhi-chat.css`).

## Context

Chat (`/ui/chat/`) is the **last and highest-risk** screen of the screen-v2 migration. Every other
section (API Keys, Settings, Models, MCP, Tokens, Users) already runs on the V2 `AppShell` — a
3-column shell (left nav+sidebar · main · right rail) that screens feed via `useShellChrome(...)`.
Chat still renders its **own** dual `SidebarProvider` layout *inside* the root `AppShell` (a
double-shell): left = chat history, right = a settings panel, with the MCP tool-picker hidden in a
composer popover.

Goal: bring chat onto the V2 shell and match the prototype — a chat-history sidebar, a conversation
+ composer main panel, and a **two-tab rail** (Parameters · MCP servers) — while keeping the screen's
behavior intact.

### Hard constraints (decided with the user)

1. **V2-only, NO flag, NO new `ChatScreenV2` component.** Migrate the **existing** chat route +
   components **in place**, replacing components one-by-one. **Reason:** ~50 `data-test*` attributes
   and 7 state CSS classes are load-bearing for the E2E suite; in-place migration preserves screen
   continuity so E2E needs minimal rework.
2. **Presentation-only.** Reuse every hook/store/mutation unchanged (`chatStore`,
   `chatSettingsStore`, `agentStore`, `mcpSelectionStore`, `useMcpClients`, `useMcpAgentTools`,
   `useListModels`). No backend change → no binary rebuild for Gate B.
3. **Phase-wise.** Foundation first, then restyle surfaces one at a time. **After every phase:** run
   UI tests + E2E, do the manual Chrome check (below), commit, move on.
4. **Rail = two tabs.** Parameters (model + settings) and MCP servers. The MCP tool-picker **moves
   from the composer popover into the rail's MCP-servers tab** as an accordion; the composer MCP
   button is removed.
5. **Message chrome = real data only.** Restyle bubbles/avatars/meta-strip to V2, but render only
   data the app actually has (model, token usage, copy). **Drop** prototype-only affordances with no
   backing data/mutation: regenerate, branch-from-here, thumbs, tokens/s, latency, cost, the
   context-usage ring. Note each drop in the commit.

### Per-phase manual check (every phase, non-negotiable)

In addition to automated UI + E2E gates, **each phase** ends with a **Claude-in-Chrome live
walkthrough with a side-by-side comparison against the prototype** (`localhost:8000/Bodhi Chat.html`):

- Drive the real app (`make app.run.live`, no rebuild needed — presentation only) **and** open the
  prototype in a second tab; compare the migrated surface against the design for that phase
  (layout, spacing, colors, states).
- Confirm **interactions** work for that phase's surface (send/stream/stop, select model, toggle a
  setting, switch rail tab, toggle MCP tool, create/select/delete chat, collapse sidebar).
- Confirm **light AND dark**, **responsive** (sidebar collapse, mobile drawers, rail wrapping),
  conversation/composer **scroll** behaves (no double scrollbars), and the **console is clean** (0
  errors on load + key interactions).
- No view-transition flicker on the shell grid columns.

## Shared refactor (the one shared change — Phase 0)

The `ShellSlots` seam (`components/shell/ShellSlotsContext.tsx`) currently exposes only
`breadcrumb / headerActions / sidebar / rail / railHeader / railDefaultOpen`. Chat needs the shell
in a non-default layout: `mainScroll=false` + `railScroll=false` (chat owns the conversation/composer
scroll; rail panes scroll internally), `railWidth≈360`, `sidebarWidth≈260`, `contentClass="flush"`,
`resizeKey="chat"`. `AppShell` **already accepts all of these as props** — only the seam and the root
spread need widening. Add these as **optional** fields so every already-migrated screen is unaffected
(omitted → `undefined` → AppShell defaults).

All other new pieces (`ChatRailTabs`, `ParametersPane`, `McpServersPane`, `ChatHistorySidebar`,
`ChatTitle`, chat CSS) are **chat-local**, not shared. The collapsed-sidebar history popover reuses
the existing `useShell()` `collapsed`/`openPop`/`setOpenPop` seam (mirror `ModelsScreenV2`) — no new
shared primitive.

---

## Phase 0 — Extend the `ShellSlots` seam (foundation, zero chat change)

**Goal.** Let a publishing screen pass `mainScroll / railScroll / contentClass / railWidth /
sidebarWidth / resizeKey / section` up to the single root `AppShell`. Nothing changes visually for
any existing screen.

**Files.**
- `components/shell/ShellSlotsContext.tsx` — add the optional fields to `ShellSlots`, the destructure,
  and the `useMemo` dep list in `useShellChrome`.
- `routes/__root.tsx` — spread the new fields onto `<AppShell>`. Precedence: `contentClass={slots.contentClass ?? 'flush'}`,
  `section={slots.section ?? resolved.section}`, `resizeKey={slots.resizeKey ?? resolved.section}`
  (guard so unmigrated screens keep `resolveShellRoute` values).
- `components/shell/ShellSlotsContext.test.tsx` — assert the new fields round-trip through publish/clear.

**Tests.** `ShellSlotsContext.test.tsx` + full `npm test` to prove no regression across the ~25
screens using `useShellChrome`. E2E: run full suite unchanged.

**Manual check.** Open every already-migrated section (Models/MCP/Tokens/Users/Settings); rail width,
scroll, breadcrumb unchanged; light+dark; console clean. (No prototype comparison — chat untouched.)

**Commit.** `Extend ShellSlots seam (mainScroll/railScroll/contentClass/railWidth/resizeKey/section), additive, no behavior change`

---

## Phase 1 — Structural swap: chat onto the shell (riskiest)

**Goal.** Remove the chat-owned `SidebarProvider`s. Publish chat history as the AppShell `sidebar`
slot and settings as the `rail`, via `useShellChrome`, with the new layout fields. **Keep styling
close to current** — this phase is the container, not the pixels.

**Files.**
- `routes/chat/index.tsx` — gut the dual-`SidebarProvider` structure. Keep `AppInitializer`,
  `validateSearch`, `ChatUrlSync`, the `loadChats`/hydrate effects, and `?model=/?id=` sync. New
  `ChatScreen` builds `sidebar` (NewChatButton + ChatHistory) and `rail` (settings body), calls
  `useShellChrome({ breadcrumb, sidebar, rail, railDefaultOpen:true, mainScroll:false, railScroll:false,
  railWidth:360, sidebarWidth:260, contentClass:'flush', resizeKey:'chat', section:'chat' })`, and
  renders `<ChatUI/>` as the shell child.
- `routes/chat/-components/ChatUI.tsx` — **remove `useSidebar()`** (`ChatUI.tsx:332`). Replace
  `openSettings/setOpenSettings` with `useShell()` `openRail`/`collapseRail`; the "no model" branch in
  `handleSubmit` calls `openRail()`. Verify the height chain from `.shell-body.is-fill` → ChatUI's
  existing absolute-inset scroll container (`h-full`/`min-h-0`).
- `routes/chat/-components/settings/SettingsSidebar.tsx` — extract the settings body out of the
  shadcn `<Sidebar inner side="right">` into a plain scroll container; **keep `data-testid="settings-sidebar"`**.
- `routes/chat/-components/ChatHistory.tsx` — replace `SidebarMenu*` primitives with plain markup;
  **preserve** `chat-history-container`, `chat-history-item-${id}`, `chat-history-button-${id}`,
  `delete-chat-${id}`, the group texts, and the `bg-muted` active marker.
- `routes/chat/-components/NewChatButton.tsx` — plain button if it used sidebar primitives; keep `new-chat-button`.
- **Toggle continuity:** publish a small `headerActions` node with a history-toggle + settings-toggle
  carrying `data-testid="chat-history-toggle"` and `data-testid="settings-toggle-button"`, driving
  `useShell()` collapse/openRail — so the E2E page objects keep working **verbatim**.

**Preserved testids.** `chat-ui`, `chat-input`, `chat-form`(+`data-test-state`), `send-button`,
`stop-button`, `message-list`, `empty-chat-state`, `chat-input-panel`/`-container`,
`new-chat-inline-button`, `settings-sidebar`, `settings-toggle-button`, `chat-history-toggle`,
`chat-history-*`, `delete-chat-*`, `new-chat-button`, all setting/model ids. MCP popover testids stay
(popover unchanged this phase).

**Tests.** `routes/chat/index.test.tsx` — wrap render in `ShellSlotsProvider`; assert mount without a
`SidebarProvider`. `SettingsSidebar.test.tsx` / `ChatHistory.test.tsx` / `NewChatButton.test.tsx` —
drop `SidebarProvider` wrappers, keep testid/role assertions. E2E: add `test.use({ reducedMotion:
'reduce' })` to all four chat specs now (locks out view-transition detach races). Page objects
ideally **unchanged** thanks to the preserved toggle testids.

**Manual check.** App ↔ prototype side-by-side: history sidebar shows/collapses; settings rail
shows/collapses via preserved toggles; conversation scrolls independently of the pinned composer;
`?model=/?id=` sync works; streaming works; light+dark; mobile drawers + scrim; **no nested
scrollbars / no grid-column flicker**; console clean.

**Commit.** `Chat: migrate onto V2 AppShell via useShellChrome; remove chat-owned SidebarProviders (history→sidebar, settings→rail); preserve testids`
**Rollback.** Single focused commit → revert restores the dual-shell. Fallback while diagnosing
height chains: temporarily `mainScroll={true}`.

---

## Phase 2 — Message chrome restyle (`ChatMain` / `ChatMessage` / composer)

**Goal.** Restyle conversation + composer to the V2 `.conv`/`.msg`/`.meta-strip`/`.composer` design,
**real data only**.

**Files.**
- `routes/chat/-components/ChatMessage.tsx` — V2 bubbles (user = muted rounded; assistant = name +
  model-tag + markdown + inline tool-call cards + hover-reveal `.meta-strip`). Show **only** model,
  token usage, copy. **Drop** regenerate/branch/thumbs/t-s/latency/cost/context-ring (note in commit).
  **Preserve** `user-message`/`assistant-message`/`streaming-message`, `*-message-content`,
  `data-served-model`, and the state classes the streaming waits depend on: `.chat-user-message`,
  `.chat-user-message-archive`, `.chat-ai-message`, `.chat-ai-archive`, `.chat-ai-streaming`,
  `.message-completed`, `.message-streaming` (restyle the look, keep the classnames).
- `routes/chat/-components/ChatUI.tsx` — V2 composer (`max-w-760` inner, autosize textarea, ⌘↵ send,
  Send button). **Preserve** `chat-input`, `chat-form`+`data-test-state`, `send-button`, `stop-button`,
  `new-chat-inline-button`, `empty-chat-state` ("Welcome to Chat"). MCP popover button untouched here.
- `routes/chat/-components/ToolCallMessage.tsx` — inline tool-call cards; **preserve** `tool-call-message`,
  `tool-call-expand`, `tool-call-status` (still contains "Completed"/"Calling..."), `tool-call-content`,
  the `<pre>` args structure.
- `routes/chat/-components/ThinkingBlock.tsx` — restyle only; keep testids.
- **New (chat-local):** `routes/chat/-components/chat.css` (or co-located) for `.conv`/`.msg`/`.meta-strip`/`.composer`.

**Tests.** `ChatMessage.test.tsx`, `ThinkingBlock.test.tsx` — update structure assertions, keep testid
assertions; add a test that regenerate/branch/thumbs are NOT rendered (locks the decision); confirm
token-usage + copy render when metadata present. E2E: none expected (state classes preserved);
streaming class-based waits are the canary.

**Manual check.** App ↔ prototype: send (streaming + non-streaming); hover meta-strip shows
tokens+copy only; tool-call card; thinking block; light+dark; responsive; console clean.

**Commit.** `Chat: restyle messages + composer to V2 (real-data meta-strip; drop regenerate/branch/thumbs/speed/cost); preserve state classes + testids`

---

## Phase 3 — History sidebar restyle + collapsed-rail popover

**Goal.** V2 history: "New chat" primary button, History header with collapsible search, grouped
list, per-item hover `⋯` menu (Delete is real; Rename uses the store title if trivially backed;
Pin/Duplicate/Export are inert placeholders — note which). Collapsed mode: icon buttons +
`AnchoredPopover` history list via `useShell()`.

**Files.**
- `routes/chat/-components/ChatHistory.tsx` — V2 grouped list + hover `⋯` menu. **Preserve**
  `chat-history-container`/`-item-${id}`/`-button-${id}` and keep `delete-chat-${id}` on the Delete
  action (now inside the `⋯` menu). Keep `bg-muted` active marker + group header texts.
- `routes/chat/-components/NewChatButton.tsx` — primary styling; keep `new-chat-button`.
- **New:** `routes/chat/-components/ChatHistorySidebar.tsx` — slot wrapper (New chat + collapsible
  search + ChatHistory). Reads `useShell()` `collapsed`; when collapsed, render icon buttons +
  `AnchoredPopover` (mirror `ModelsScreenV2` popover, keyed by stable `openPop` id). Search filters
  the in-memory `useChatStore` chat list client-side.

**Tests.** `ChatHistory.test.tsx`, `NewChatButton.test.tsx` — new structure; assert Delete still fires
`deleteChat`; collapsed-mode test with a `ShellContext` value `collapsed=true` asserting the popover
trigger renders; mark Rename/Pin/Duplicate/Export inert. E2E: `ChatHistoryPage.mjs` —
`deleteChatById`/`deleteChatWithConfirmation` open the `⋯` menu before clicking `delete-chat-${id}`
(the one allowed page-object touch this phase); selector names kept.

**Manual check.** App ↔ prototype: create/select/delete chats; grouped buckets correct; search
filters; collapse sidebar → icon rail + history popover; light+dark; responsive; console clean.

**Commit.** `Chat: V2 history sidebar (search, ⋯ menu, collapsed-rail popover via useShell)`

---

## Phase 4 — Rail tabs scaffold + Parameters pane restyle

**Goal.** Two-tab rail (Parameters · MCP servers) as `railHeader`, with tab state in the chat route
and the active pane published as `rail`. Restyle Parameters to V2. The MCP-servers tab renders a
**placeholder** this phase (real accordion in Phase 5) so the tab structure is testable before the
popover move.

**Files.**
- **New:** `routes/chat/-components/ChatRailTabs.tsx` — segmented `railHeader` (Parameters | MCP
  servers + count badge); props `value/onChange/mcpCount`. Tab state (`useState<'parameters'|'mcps'>`)
  lives in `ChatScreen` (`index.tsx`); route publishes `railHeader={<ChatRailTabs/>}` and
  `rail={tab==='parameters' ? <ParametersPane/> : <McpServersPane/>}`. Wrap **only the pane swap** in
  `useViewTransition()` (reduced-motion aware) — never the grid columns.
- **New:** `routes/chat/-components/settings/ParametersPane.tsx` — V2 Parameters layout. Reuses
  `AliasSelector`, `SettingSlider`, `SystemPrompt`, `StopWords`, `HelpTooltip` and `useChatSettingsStore`
  unchanged; each override row uses a `Switch` (off → control hidden). **Preserve** `#stream-mode`,
  `#api-token-enabled`/`#api-token`, `#seed-enabled`/`#seed`, `max-tool-iterations-input`,
  `system-prompt-enabled`/`-textarea`, `stop-words-enabled`/`-input`, `setting-max-tokens-toggle`,
  `temperature-slider`/`max-tokens-slider`/`top-p-slider`, `model-selector-loaded`/`-trigger`,
  `combobox-option-*`, `api-format-label`. Keep `data-testid="settings-sidebar"` on the pane **root**
  (E2E `openSettingsPanel`). Add a real "Reset to defaults" button.
- `routes/chat/-components/settings/SettingsSidebar.tsx` — folded into `ParametersPane` (or deleted in
  its favor).
- `routes/chat/index.tsx` — wire tab state; publish `railHeader` + tabbed `rail`.

**Tests.** `SettingsSidebar.test.tsx` → `ParametersPane.test.tsx` (keep control assertions; add
tab-switch test; wrap with `ShellSlotsProvider`). `AliasSelector/SettingSlider/SystemPrompt/StopWords`
tests unchanged unless wrappers changed. New `ChatRailTabs` test (toggle + badge). E2E: none required
if `settings-sidebar` + control ids preserved; re-run all chat specs.

**Manual check.** App ↔ prototype: rail tabs switch (view-transition on pane only, reduced-motion
respected); Parameters override switches hide/show controls; sliders/model combo/api-format; Reset to
defaults; light+dark; responsive (rail as mobile drawer); console clean.

**Commit.** `Chat: V2 rail tabs (Parameters | MCP servers) + Parameters pane restyle; MCP tab placeholder`

---

## Phase 5 — Move MCP tool-picker into the rail's MCP-servers tab (biggest E2E change)

**Goal.** Relocate MCP tool selection from the composer `McpsPopover` into the rail MCP-servers
accordion; remove the composer MCP button. Isolated last so the prior green suite de-risks it.

**Files.**
- **New:** `routes/chat/-components/settings/McpServersPane.tsx` — "Add an MCP server…" combobox +
  accordion (status dot, on/total count, chevron; expanded → All/None quick links + per-tool
  checkboxes; trash to remove) + empty state. Reuses the exact `McpsPopover` selection logic
  (`useMcpSelectionStore`, `useListMcps`, the `mcpTools`/`mcpConnectionStatus` maps). **Preserve**
  `mcp-row-${id}`, `mcp-item-${id}`, `mcp-expand-${id}`, `mcp-checkbox-${id}`,
  `mcp-tool-row-${mcpId}-${tool}`, `mcp-tool-checkbox-${mcpId}-${tool}`, `mcps-empty-state`. The badge
  count moves to `ChatRailTabs` (repurpose/keep `mcps-badge` there).
- `routes/chat/-components/ChatUI.tsx` — remove `<McpsPopover>` + the composer MCP button (and
  `mcps-popover-trigger`/`-content`). **Lift** the MCP client wiring (`useMcpClients`, `connectAll`,
  `agentTools`, the `mcps`/`mcpTools`/`mcpConnectionStatus` maps) from `ChatUI` up to
  `ChatScreen`/`index.tsx` so both the composer (agent) and the rail (picker) share one source — hooks
  unchanged, only the call site moves. Keep the `connectAll` effect keyed on `mcps` exactly as today.
- **Delete:** `routes/chat/-components/McpsPopover.tsx`.

**Preserved / removed testids.** All `mcp-*` selection testids preserved (moved, same names).
**Intentionally removed:** `mcps-popover-trigger`, `mcps-popover-content` (popover gone).

**Tests.** Move `McpsPopover.test.tsx` coverage into `McpServersPane.test.tsx` (toggle MCP/tool,
indeterminate, empty, disabled reasons). Update `index.test.tsx` MCP mocks for the lifted call site.
E2E: `ChatPage.mjs` — replace `openMcpsPopover`/`expectMcpsPopover*` with `openMcpServersTab` +
`expectMcp*InRail`; keep `mcpRow/mcpExpand/mcpCheckbox/mcpToolCheckbox/mcpItem` selectors verbatim;
move `mcpsBadge` to the tab. `chat-mcps.spec.mjs` — open the MCP rail tab instead of the popover;
behavior assertions stay.

**Manual check.** App ↔ prototype: open MCP servers tab; add server; expand accordion; All/None;
per-tool checkboxes; tab badge count; send a message that triggers a tool call → tool-call card
executes; empty state; light+dark; responsive; console clean.

**Commit.** `Chat: relocate MCP tool-picker from composer popover into rail MCP-servers accordion; remove composer MCP button; update E2E page objects`

---

## Phase 6 — `ChatTitle` breadcrumb + dead-flag cleanup

**Goal.** Editable conversation-title breadcrumb; delete the dead V2-flag scaffolding (chat was the
last flag).

**Files.**
- **New:** `routes/chat/-components/ChatTitle.tsx` — crumb (`message-circle · Chat · timestamp`) +
  editable title bound to `useChatStore` current-chat title (Rename = real store mutation). Publish the
  crumb as `breadcrumb` items and the editable title via `headerActions` (avoid widening the breadcrumb
  slot contract).
- `routes/chat/index.tsx` — publish breadcrumb/title.
- **Delete:** `lib/uiV2Flags.ts`, `lib/uiV2Flags.test.ts`, `hooks/useUiV2Flag.ts` (grep-confirm no
  other importers first); remove the `chat` `UiV2Screen` type.
- Update `screen-v2/tracker.md` (Batch 5 → done; chat flag retired) + write `batch-5-chat-retro.md`.

**Tests.** New `ChatTitle.test.tsx`; remove `uiV2Flags.test.ts`; grep for dangling `useUiV2Flag`
imports. E2E: none (title additive); optional rename assertion.

**Manual check.** App ↔ prototype: title shows + edits + persists; breadcrumb correct; full smoke
(send, history, settings, MCP, streaming); light+dark; responsive; console clean.

**Commit.** `Chat: editable ChatTitle breadcrumb; delete dead uiV2Flags scaffolding (chat was the last flag)`

---

## Top risks + mitigations

1. **Scroll-region breakage from `mainScroll=false` (Phase 1).** Removing the SidebarProviders can
   collapse the conversation/composer height chain (floating composer, double scrollbars). *Mitigate:*
   keep ChatUI's absolute-inset scroll container; verify the `h-full`/`min-h-0` chain from
   `.shell-body.is-fill` down; single-commit phase for clean revert; fallback `mainScroll={true}` while
   diagnosing.
2. **`useSidebar()` removal cascade (Phase 1).** ChatUI/SettingsSidebar/ChatHistory/NewChatButton use
   shadcn sidebar primitives that throw outside a provider. *Mitigate:* convert all four to
   provider-free markup in the same phase; rewire ChatUI to `useShell().openRail`; update test wrappers
   (drop SidebarProvider, add ShellSlotsProvider).
3. **E2E testid/state-class drift (all phases, peaks Phase 5).** ~50 testids + 7 state classes encode
   the suite's streaming/tool/history waits. *Mitigate:* per-phase preserved-testid checklist (above);
   restyle look, never classnames; restrict page-object edits to Phase 3 (delete-under-⋯) and Phase 5
   (popover→rail); `reducedMotion:'reduce'` on chat specs from Phase 1.
4. **Toggle-button continuity (Phase 1).** Settings/history were toggled via
   `settings-toggle-button`/`chat-history-toggle`; the shell has its own toggles. *Mitigate:* publish
   `headerActions` buttons carrying those exact testids that drive `useShell()`.
5. **MCP wiring lift + view-transition races (Phases 4–5).** Lifting `useMcpClients`/`agentTools` to
   `ChatScreen` risks reconnect loops; the rail-swap transition can race E2E. *Mitigate:* keep the
   `connectAll` effect keyed on `mcps` exactly as today, just relocated; wrap only the pane swap in
   `useViewTransition`; force reduced-motion in chat E2E; Gate-B verifies a tool-call still executes
   end-to-end.

## Verification (whole batch)

- Per phase: `cd crates/bodhi && npm test` (UI) → `make test.e2e` (from `crates/lib_bodhiserver/tests-js`)
  → Claude-in-Chrome app↔prototype manual check → commit.
- Final: full `npm test` + full E2E in both standalone and multi_tenant modes; grep for dead
  `useUiV2Flag`/`uiV2Flags` importers; tracker + retro updated.

## Critical files

- `crates/bodhi/src/components/shell/ShellSlotsContext.tsx` · `routes/__root.tsx` (Phase 0 shared seam)
- `crates/bodhi/src/routes/chat/index.tsx` (shell wiring + tab state, every phase)
- `crates/bodhi/src/routes/chat/-components/ChatUI.tsx` (useSidebar removal, composer, MCP lift)
- `crates/bodhi/src/routes/chat/-components/{ChatMessage,ToolCallMessage,ChatHistory,NewChatButton}.tsx`
- `crates/bodhi/src/routes/chat/-components/settings/{SettingsSidebar→ParametersPane,McpServersPane}.tsx` · new `ChatRailTabs.tsx` · `ChatHistorySidebar.tsx` · `ChatTitle.tsx`
- `crates/lib_bodhiserver/tests-js/pages/{ChatPage,ChatHistoryPage,ChatSettingsPage}.mjs` + `specs/chat/*.spec.mjs`
