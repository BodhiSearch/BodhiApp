# Shared foundation — Models + Create local alias

*Audience: Claude / AI coding agents first, developers second. This doc captures the conceptual substrate both finalized wireframes share — the entity model, data sources, the row-shape duality pattern, the primitives catalogue, and the conversation context that is not visible in the wireframe itself.*

Finalized wireframes live in `design/models-page/project/`. This spec is an archive of the reasoning.

---

## 1. Framing — why a wireframe even for this

BodhiApp exposes "a model" in at least five different shapes depending on where it lives:

- a **user-configured alias** bound to a local GGUF with runtime params (llama.cpp world)
- a **downloaded GGUF file** sitting on disk that may or may not have an alias
- a **user-configured API model** (e.g. `openai/gpt-5-mini` with per-model overrides)
- a **connected API provider** (with credentials/OAuth)
- an **unconnected API provider** discovered from the hosted Bodhi directory
- a **remote HuggingFace repo** that could become one of the above

The previous UI split these into two pages (Models Hub, Discover) plus four side-flows (alias create, api-model create, provider add, etc.). The two pages disagreed about toolbar shape, filter sets, and row semantics. The wireframes in `project/screens/` are the redesign that collapses this into:

- **one "Models" page** that lists all six kinds behind a mode toggle
- **one "Create local alias" flow** for the subset that maps to llama.cpp

Everything below is the spec for how those two screens think.

---

## 2. The six-entity model (the single most important contract)

Every row in the Models page is one of six typed entities. The entire visual system — row shape, detail panel dispatch, filter applicability, toggle eligibility — is driven by `kind`.

| # | `kind` | Source of truth | Default row shape | Visible in `My Models` | Visible in `All Models` |
|---|---|---|---|:---:|:---:|
| 1 | `alias` | Bodhi server DB (user's alias table) | file-first | ✓ | ✓ (tagged `local`) |
| 2 | `file` | Bodhi server DB (disk scan of `~/.bodhi/models`) | file-first | ✓ | ✓ (tagged `local`) |
| 3 | `api-model` | Bodhi server DB (user's configured API model rows with overrides) | file-first (alias-like) | ✓ | ✓ (tagged `local`) |
| 4 | `provider-connected` | Bodhi server DB (user's provider connections w/ keys/OAuth) | provider-summary | ✓ | ✓ |
| 5 | `provider-unconnected` | `api.getbodhi.app` hosted provider directory | provider-summary (dashed border + attribution) | — | ✓ |
| 6 | `hf-repo` | HuggingFace API | repo-first (nested quants) | — | ✓ |

> **Naming note.** The kind is spelled `api-model` in code even though users (and this doc in prose) sometimes call it "API Alias". The synonym is intentional — a user-configured api-model IS an alias in the product sense, but `api-model` stays as the `kind` key for backward-compat. Do not rename without a migration.

### Provenance rules

- `alias` / `file` / `api-model` / `provider-connected` all come from the **same Bodhi server DB**. The user's own data.
- `provider-unconnected` is **always** a read-through from the hosted directory at `api.getbodhi.app`. This is real data in the final product — the wireframe uses static samples but the attribution chip (`from api.getbodhi.app`) makes the source explicit so the eventual fetching feels natural.
- `hf-repo` is a read-through from HuggingFace. The wireframe treats it as always-available catalog.

### Cross-kind computed indicators

Rows are not silos — they carry back-links that reflect relationships computed across the full entity set:

- `hf-repo` gains a `✓ N local aliases ↗` badge when any quant of that repo is already aliased. Click jumps to those aliases.
- `provider-connected` gains a `✓ N api-models configured ↗` badge when the user has api-model rows referencing that provider.
- `alias` / `file` show `↗ catalog · owner/repo` — clicking opens the HfRepoPanel scoped to the source repo.
- `api-model` shows `↗ openai (provider)` — clicking opens the connected ProviderPanel.

**Why this matters:** users landing on a local alias want to know "what else does this repo have?"; users landing on a catalog entry want to know "did I already download this?". The back-links are the duality glue.

---

## 3. Row-shape duality — and why we refused to collapse it

Two row shapes coexist in the same list.

**File-first** (used by `alias`, `file`, `api-model`): one row per user-configured thing.

```
[alias]  my-gemma                     [● ready] [text→text] [tool-use]
         google/gemma-2-9b:Q4_K_M     ↗ catalog
         preset: chat · last run 12m ago
```

**Repo-first** (used by `hf-repo`): one row per repo with quants nested. The full quant list lives in `HfRepoPanel`; individual quants get `+ alias` CTAs there.

```
[hf-repo] Qwen/Qwen3.5-9B              [● ~38 t/s] [text2text] [tool-use]
          9B · Apache-2 · HuggingFace
          default :Q4_K_M · 5.6 GB · 5 quants · ↓ 443k · ♥ 3.1k
          ✓ 2 local aliases ↗
```

**Provider** rows are their own third shape (summary with model-count), and expand fully in their detail panels.

**Ranked rows** (model-level) — a fourth shape added in v27 to handle benchmark leaderboards. See `models.md §8` for the full spec. Triggered when a benchmark sort is active (Specialization ≠ All). Collapses local files to their alias representation, stacks api-aliases above a provider identity line. Each ranked row carries an absolute rank number that never renumbers even under filters.

```
#4.  my-qwen-coder            [alias · preset: coding]
     my-qwen-long             [alias · preset: long-ctx]
     Qwen/Qwen3-Coder-32B:Q4_K_M  [modelfile · 20.3 GB]
                                                 75.8 HumanEval  [use →]
```

### Dedup + stack rules (ranked shape)

The one asymmetry to remember:

- **Local file dedup collapses the HF entry.** Once a modelfile is on disk, all aliases on it share the same bits — listing the local aliases AND the upstream HF repo entry would double-count. The HF entry collapses away.
- **API configs stack without dedup.** Each config (api-key / oauth / override-set) is a distinct user choice. Listing only one would hide the others.

### Why three shapes and not one

An earlier iteration considered unifying everything into a single row component. Rejected because:

- File-first optimises for **"my things"**: the alias name / local identity is what the user recognises. Downloading the repo 2 months ago is irrelevant; what matters is the preset, the last-run time, the ↗ catalog escape hatch.
- Repo-first optimises for **"what's available"**: a Qwen repo has 5 quants; showing them as 5 separate rows buries the repo. Showing one row that expands into quants matches how users actually browse HF.
- A forced unification would either bury aliases under their upstream repo (bad for "my things") or explode HF repos into quant-rows (bad for browsing).

The cost of coexistence is that the page has two row layouts in one list. The mitigation: the `kind` prefix chip (`[alias]`, `[file]`, `[hf-repo]`, …) makes shape selection obvious at a glance. The back-links ensure neither shape feels like a dead end.

---

## 4. Shared primitives catalogue

Primitives are exported from `screens/primitives.jsx` via `Object.assign(window, {...})` and picked up by the other `screens/*.jsx` files through `<script>` tag ordering in `index.html`.

### Models-page primitives

| Name | Used by | What it does |
|---|---|---|
| `ModeToggle` | `discover.jsx` (Models desktop) | Top-of-toolbar segmented radio: `[●] My Models (N) · [ ] All Models (M + D directory)` with caption |
| `ModeToggleCaption` | Models desktop | One-line explainer that `All Models` draws from HuggingFace + Bodhi Directory |
| `KindChipRow` | Models all variants | Second toolbar row: `All · Aliases · Files · API models · Providers · HF repos` multi-select chips |
| `ModelsAddBrowseMenu` | Models all variants | Grouped dropdown: Add section (Add by repo / Paste URL / Add provider / Add api-model) + Browse section (Trending / New launches / Leaderboards) |
| `ModelListRow` | `discover.jsx` | Extended with `localBadge`, `backlink`, `catalogAliases`, `directoryAttribution` props |
| `PhoneFrame` | Models mobile variant, also older screens | Phone chrome wrapping mobile variants (re-added after hub.jsx slimming) |
| `MobileMenu` | all mobile variants | Site-wide menu — `Models` is now a **leaf** (no `expanded: { My Models, Discover }` sub-tree) |
| `RankedRow` | `discover.jsx` desktop/medium/mobile | Model-level row for ranked mode. Takes `{entry, selected, onClick}`. Entry has `rank`, `primaries[]`, `identity`, `score`, `benchmark`, `actions[]`, `dispatchKind`, `isLocal`, `isDirectory`. |
| `RankedModeCaption` | `discover.jsx` | Honest-disclosure banner below the toolbar when ranked mode is active. Explains dedup/stack rules. Dismissible. |
| `groupIntoRankedRows(benchmark, mode)` | `discover.jsx` | Pure function. Returns ordered `RankedRow` props for the given benchmark + mode. Wireframe uses the static `RANKED_FIXTURE_CODING`; production aggregator lives here. |
| `RANKED_FIXTURE_CODING` | primitives.jsx | Static 9-entry leaderboard for the HumanEval/Coding demo. Covers every dedup/stack case. |

### API-create primitives (added v29)

| Name | What it does |
|---|---|
| `ApiFormatPicker` | Dropdown picker over `API_FORMATS`. Field-styled closed state with ▾. |
| `ApiKeyField` | "Use API key" checkbox toggle + masked input + 👁 eye. When toggle off, input disabled. |
| `PrefixField` | "Enable prefix" checkbox toggle + text input + example helper. |
| `ForwardingModeRadio` | Two-option radio: `all` / `selected`. Drives the conditional Model Selection section in `api.jsx`. |
| `ModelMultiSelect` | Selected chips strip (× to remove) + search input + scrollable available list with checkboxes + action footer (`Fetch Models` / `Select All (N)` / `Clear All`). |
| `ApiRail` | Sticky section nav for `ApiStandalone`. 3 anchors: Provider / Routing / Models. Mirrors `AliasRail`. |
| `ApiMediumAnchors` | Top-of-page jump chips for `ApiMedium`. Mirrors `AliasMediumAnchors`. |
| `API_FORMATS` constant | 10 formats: openai-responses, openai-completions, anthropic-messages, anthropic-oauth, google-gemini, openrouter, hf-inference, nvidia-nim, groq, together. Each entry: `{code, label, defaultBaseUrl}`. |
| `FIXTURE_OPENAI_MODELS` constant | 12 representative OpenAI models used by the demo (gpt-5 family, gpt-5.1-codex-mini/max, gpt-5.2-codex, gpt-4-turbo, gpt-5.3-codex, text-embedding-3-large). |

### Create-alias primitives

| Name | What it does |
|---|---|
| `ParamSection` | Collapsible section wrapper used for each of the 4 alias sections |
| `QuantPicker` | Grid-of-chips over repo quants; selecting a chip implies the filename (no separate Filename field) |
| `FitCheckCard` | "Fits your rig" gauge below Model file section |
| `LiveConfigJson` | Rolling JSON summary of the alias config as fields change |
| `DownloadProgressStrip` | Streaming-download progress used when a not-yet-downloaded quant is picked |
| `SliderWithMarks` | Annotated slider with llama.cpp-aware guidance marks |
| `PresetGrid` | 18 preset tiles for Section 3 (Preset & Runtime args) |
| `PresetChipRow` | Condensed preset chip row used in medium/mobile variants |
| `ArgsEditor` | Plain textarea with per-line `ArgLine` rendering — flag chips + value spans + hover tooltip + wavy underline for unknown flags |
| `ArgsPalette` | Right-side list of known `llama-server --help` flags; hover shows `+ append` affordance; click appends to editor |
| `ArgsHelpPop` | Tooltip content for hover over a flag in the editor |
| `ARGS_HELP` / `ARGS_PRESETS` / `PRESET_CATALOGUE` | Static data tables |
| `PresetAndArgsSection` | The merged Section 3 wrapper = PresetGrid + ArgsEditor + ArgsPalette |
| `OverlayShell` | Overlay-variant chrome |
| `AliasRail` / `AliasMediumAnchors` | Sticky section nav used by standalone/medium variants |

### Catalog / discover primitives

| Name | What it does |
|---|---|
| `BrowseBySelector` | (Deprecated as a dedicated view — kept as primitive; superseded by the `Specialization` filter group.) |
| `TaskCategoryGrid` | Category tiles grid — also now behind the Specialization filter. |
| `SPECIALIZATIONS` constant | Source of truth for the 11 specialization entries + their benchmark ref (`HumanEval`, `GPQA`, `MMMU`, …) |

### MCP primitives (added v30)

| Name | What it does |
|---|---|
| `McpCategoryChipRow` | Horizontal chip row over `MCP_CATEGORIES`; filters the MCP Discover grid by catalog category (9 categories + All). |
| `McpStatusFilter` | Pill group that filters by derived card-state: `All · Approved · Connected · Not connected · Pending`. |
| `McpCatalogCard` | Discover grid card. Reads the catalog entry + viewer role and renders the five-state CTA (`One-click Add` / `Submit for Approval` / `+ Add MCP Server` / `View instance ↗` / `Re-enable`). When `state==='connected'`, renders an inline mini-instance summary on the card. |
| `McpCatalogDrawer` | Right-side detail pane opened when a Discover card is clicked. Tabbed: About / Capabilities / Connection / Metadata / Performance. Capabilities lists fixture tools with descriptions. |
| `McpServerForm` | Server-registry form. Takes `mode='blank'|'prefilled'|'edit'` and `initial`. Auth-type-aware: renders `McpAuthOAuthConfig` or `McpAuthHeaderConfig` conditionally. Body usable standalone (Admin full-page) or inside an `OverlayShell` (Discover `Add MCP Server` overlay demo). |
| `McpInstanceForm` | Instance-create form. Picks from approved servers list, auto-defaults slug/name, renders OAuth-connect step (shows Connected state with Client ID + Disconnect) or header-credentials field. Same form body works full-page (My MCPs edit) or inside `OverlayShell` (Discover overlay). |
| `McpAuthHeaderConfig` | Header/Query auth sub-block: list of `{placement, name, hint}` rows + `+ Add Key`. |
| `McpAuthOAuthConfig` | OAuth2 auth sub-block: registration type picker + Authorization/Token/Registration endpoints + Scopes input. Matches production download (24).png. |
| `McpServerListRow` | Admin registry table row. Columns: auth icon / name+date / URL / status chip / instance count / actions edit/disable/delete. |
| `McpApprovalRow` | Inbox row (warn-tone background) showing user-submitted request with Reject / Approve actions. |
| `McpInstanceListRow` | My-MCPs instance row: auth icon / name+lastUsed / URL / status chip (active/needs_reauth) / actions play/edit/delete. |
| `McpInstancePendingBanner` | Yellow dashed banner at top of My MCPs when the user has one or more pending approval requests. |
| `McpToolSidebar` | Playground left sidebar. Instance pill + search + scrollable tool list with name + truncated description. |
| `McpToolExecutor` | Playground main pane. Tool header · description · Form/JSON tabs · parameter fields · Execute button · tabbed response (Success/Response/Raw JSON/Request) with JSON preview. |
| `McpRail` | Sticky section nav for MCP Admin Desktop (Registered / Approvals). Mirrors `AliasRail` / `ApiRail`. |
| `McpMediumAnchors` | Top-of-page jump chips for MCP Admin Medium. Mirrors `AliasMediumAnchors` / `ApiMediumAnchors`. |
| `MCP_CATEGORIES` constant | 9 catalog categories (+ `all`): Productivity, Search & Web, Browser, Dev Tools, Data, AI & Content, Memory, Comms, Finance. Each entry `{code, label, icon}`. |
| `MCP_CATALOG_FIXTURE` constant | 12 curated entries spanning all 5 card-states and all auth types (oauth2/header/none). Includes defaultBaseUrl, transport, authConfig, stats, links. |
| `MCP_SERVERS_FIXTURE` constant | 6 admin registry rows (1 disabled). |
| `MCP_INSTANCES_FIXTURE` constant | 4 user instances including one `needs_reauth` state. |
| `MCP_APPROVAL_FIXTURE` constant | 2 pending user-submitted requests. |
| `MCP_TOOLS_FIXTURE` constant | Keyed by server slug; each value is a list of tool objects with `{name, desc, parameters[]}`. Drives Playground + drawer Capabilities tab. |
| `mcpCardCta({state, role})` | Pure helper returning `{label, tone, disabled?}` for the card CTA given derived state and viewer role. This is the codified "five-state × role" contract. |

### Access-request primitives (added v31)

| Name | What it does |
|---|---|
| `AccessRequestHeader` | App identity card at the top of the review page. Logo, display-name, verified chip, `3rd-party app` chip, truncated client-id, description, homepage link. Also renders the wireframe-only User/Admin role toggle pinned top-right. |
| `AccessCapsEnvelope` | Indigo-soft pill row of the app's declared capabilities: required + preferred + min-ctx + max-cost per-MTok. Hint line: "we'll pre-check models that match". |
| `AccessModelRow` | Per-model checkbox row. Shows up to 3 capability chips + reason chip (`★ app-suggested` / `matches` / `below envelope`) + context size + cost-or-origin meta. Disabled when below envelope (opacity 0.58 + pointer-events none). |
| `AccessModelGroup` | Grouped list (Aliases / API models / Provider models) with `select all` header. Renders `AccessModelRow` for each model with state from `accessModelState()`. |
| `AccessMcpRow` | Per-MCP row driven by 6-state matrix (`has-instance` / `needs-reauth` / `needs-instance` / `oauth-in-progress` / `needs-server` / `pending-admin`). Role-adaptive CTA. Optionally opens `AccessMcpInlineAddServer` in admin `needs-server` case. |
| `AccessMcpInlineAddServer` | Accordion expanded under a `needs-server` row when admin clicks `One-click Add`. Wraps the v30 `McpServerForm` with `mode='prefilled'` + compact layout + Cancel/Save footer. Reuses `McpServerForm` unchanged. |
| `AccessOAuthHint` | Pulsing indigo-dot strip: `⏳ Waiting for OAuth confirmation in popup…` (waiting state) / `✓ Connected · instance created` (connected state). Drives the `oauth-in-progress` row visual. |
| `AccessRoleSelect` | Simple role dropdown (User / PowerUser / Admin) + helper. Mirrors production. |
| `AccessActionBar` | Sticky footer with Deny + `Approve N of M resources` primary. Shows a warn-tone hint line above the primary when disabled (explains why — blocker MCPs count or no-resources-selected). |
| `ACCESS_REQUEST_FIXTURE` constant | One complete request: app identity + capability envelope + suggested-models list + 4 requested MCPs covering `has-instance` / `needs-reauth` / `needs-instance` / `needs-server`. |
| `ACCESS_MODELS_FIXTURE` constant | 8 user models spanning alias / api / provider kinds with capabilities + context + cost. At least one model fails the envelope to demo the `below-envelope` disabled row. |
| `accessModelState(model, caps, suggested)` | Pure helper returning one of 5 row states: `app-suggested` / `matches-envelope` / `user-config` / `below-envelope` / `unavailable`. Codified row-state contract. |

**AI-agent note:** When adding new primitives, follow the same pattern — define locally in `primitives.jsx`, export via `Object.assign(window, {...})` at the bottom. Do not use ES imports; the wireframe is babel-standalone / no bundler.

---

## 5. Interaction conventions that apply across both screens

These are baseline behaviours that recur and should be preserved.

### Filter grey-out vs hide

When a filter group does not apply in the current mode (e.g. `Cost · api` in `My Models` mode), **grey it out** (`.filter-group-disabled` — opacity 0.5, `pointer-events: none`, sub-caption "not applicable in My Models"). Do not hide it. Rationale: layout stability matters more than compactness; hiding groups causes the sidebar to jump every time the user toggles.

### Kind chip multi-select

`KindChipRow` is multi-select with `All` as the only exclusive state. Selecting any specific kind(s) clears `All`; clicking `All` clears the specific kinds. This mirrors the filter-set pattern of "Aliases + API models" (i.e. "things I can use in chat") as a common subset.

### localStorage migration

Tab state persists in `localStorage.bodhi-wf-tab`. A migration shim in `app.jsx` rewrites legacy `hub` / `discover` / `providers` values to `models` on first load. Do not remove this shim without a plan for what happens to users on old browsers — the wireframe is hosted on a static server and users may have stale localStorage.

### Benchmark sort triggers ranked mode

Selecting any non-`All` Specialization → benchmark sort → **ranked display mode**. In ranked mode the main list uses `RankedRow` instead of `ModelListRow` / `DiscoverCard`, rank numbers are absolute across the leaderboard (not renumbered by filters), and local items dedup while API configs stack. Clearing Specialization (back to `All`) exits ranked mode. See `models.md §8` for the full spec.

### Specialization filter (single-select + Clear)

`Specialization` is the one filter group that is **single-select** with a `Clear` affordance (returns to `All`). Selecting a specialization also mutates the sort: Coding → HumanEval, Reasoning → GPQA, etc. This "side-effect sort" is intentional — users picking "Coding" want the leaderboard, not alphabetical order.

### Hand-drawn aesthetic

Wireframes use Kalam/Caveat. Shadcn-style cleanliness is NOT the aim here; these are wireframes, not a component library. Do not replace the sketchy font or color-accent scheme without an explicit user ask — it is load-bearing for signalling "this is a wireframe, not prod" to reviewers.

---

## 6. Conversation context — decisions made during design that shaped this

AI agents picking this up should know these, because the wireframes only show the "after". They don't show what was tried, tested, and rejected.

### Models page

- **Hub + Discover were originally two tabs.** A sub-nav pill (`My Models · Discover`) already admitted they were the same destination. Collapsing them was motivated by: filter sets had drifted out of sync (Discover had Specialization, license, source; Hub didn't); toolbars disagreed (Hub had kind counts; Discover had sort); users paid for ceremony of switching.
- **The decision was to make the partition a filter, not a page.** The mode toggle `[●] My Models · [ ] All Models` collapses the page-split into a radio. Everything else — filters, row shapes, detail panels — stays identical across the toggle.
- **Counts are shown honestly.** `My Models (14)` and `All Models (3.1M + 23 directory)` — the `+ 23 directory` makes it explicit that `All` draws from two sources (HF + Bodhi Directory). Do not collapse these into a single count.
- **Hub's detail panels survived the collapse.** `AliasPanel`, `FilePanel`, `ProviderPanel` still live in `hub.jsx`. That file was slimmed (removed `HubB`, `HubMedium`, `HubMobile`, `ModelCard`, `window.HubScreens`) but kept loaded. Don't move the panels; `index.html` `<script>` ordering will break.
- **Provider directory provenance is first-class.** `api.getbodhi.app` is a real hosted directory. The wireframe's dashed border + `from api.getbodhi.app` attribution chip + `Bodhi Directory` source filter option are all signalling that this is multi-source data. When implementation happens, this attribution pattern should carry through.
- **Provider Directory tab absorbed (2026-04-19).** The top-level `Provider directory` tab is gone. Its three variants (logo gallery, matrix comparison, needs-based matcher) were all either covered by existing Models affordances or dropped by user decision:
  - **Variant A (logo gallery + connect)** → replicated via Models' `Kind=Providers` + `Source=Bodhi Directory` filter combo, and the `+ ▾ Add model` menu's "Add API provider" entry.
  - **Variant B (matrix comparison)** → dropped. Not ported. Do not reintroduce.
  - **Variant C (needs-based matcher with priority chips + match %)** → dropped. Specialization + Capability filters cover ≈ the same intent.
  - `screens/providers.jsx` is left on disk with an archival header comment; not loaded by `index.html`.
- **Ranked display mode added (2026-04-19).** When a benchmark sort is active (Specialization ≠ All), the main list swaps from entity-level rows to model-level `RankedRow`. The core asymmetry (local files dedup; API configs stack) emerged from how users reason about each kind: a downloaded file is a shared asset (aliases on it are facets), while each API config is a distinct user choice. Rank numbers are absolute across the whole leaderboard and never renumber under filters — the rank means "#2 in the world on HumanEval", not "#2 in your filtered subset".
- **Model Detail tab absorbed (2026-04-19).** The top-level `Model detail` tab is gone. Its three variants (side-drawer, full-page, bottom-sheet) were all for `hf-repo` entities, and two were already reproduced by the Models page:
  - **Variant A (side-drawer)** → identical to Models' desktop right-panel dispatch of `HfRepoPanel` when an hf-repo row is clicked.
  - **Variant C (bottom sheet)** → identical to Models' mobile/medium sheet dispatch of `HfRepoPanel`.
  - **Variant B (full-page with benchmark bars + quant slider + community rating)** → dropped by user decision. The unique features (Bar chart benchmarks, interactive quant slider, stars + review count) were explicitly dropped, not ported. They can be added into `HfRepoPanel` later if required, but not in this pass.
  - `screens/detail.jsx` is archived on disk; not loaded by `index.html`. No file depended on it (grep-verified before removal).
- **Create API Model consolidated (2026-04-19).** The original `api.jsx` had three speculative variants (A: one-form minimal / B: stepper / C: provider-aware editor). User direction: keep A's flat one-form UX but enrich with production-parity fields and give it the 4-variant responsive deck used by alias.jsx.
  - **Option A kept + extended.** Fields added to match production: "Use API key" toggle in front of the masked input, "Enable prefix" toggle in front of the prefix text, Forwarding Mode as a **radio** (not dropdown), and the full Model Selection UI (selected chips with ×, available list with checkboxes + search, `Fetch Models` / `Select All (N)` / `Clear All` actions).
  - **Conditional Model Selection.** The Model Selection section is **hidden** (not greyed) when `Forwarding mode = "Forward all requests with prefix"`. This breaks our usual filter-grey-out convention intentionally: the section isn't a filter group on the shared list, it's a form-specific affordance that has no meaning in "forward all" mode. Matches production behavior.
  - **Four variants.** `ApiStandalone` (full page with `ApiRail` sticky nav) / `ApiOverlay` (reached from Models' `+ ▾ Add model → Add API provider`) / `ApiMedium` (tablet, demos Forward-all short form) / `ApiMobile` (two PhoneFrames covering both forwarding states).
  - **Variants B + C dropped.** Stepper UX (B) and provider-aware editor (C) are gone. Their novel bits (OAuth lifecycle card, per-model cost inline) did not survive the consolidation — deferred to `ConnectedProviderPanel` work.
  - Primitives added: `ApiFormatPicker`, `ApiKeyField`, `PrefixField`, `ForwardingModeRadio`, `ModelMultiSelect`, `ApiRail`, `ApiMediumAnchors` + constants `API_FORMATS`, `FIXTURE_OPENAI_MODELS`.

### MCP wireframes (2026-04-19)

- **Four screens added as separate top-level tabs** — `MCP Discover`, `My MCPs`, `MCP Admin`, `MCP Playground`. Each has 3 responsive variants (Desktop / Medium / Mobile). The **overlay treatment is reserved for the two MCP forms only** (server form + instance form) and demonstrated as two extra variants on the Discover tab, because Discover is where both forms are most often launched from at runtime. Discover therefore has 5 variants total; the other three have 3 each.
- **Four MCP entities** — `mcp-catalog-entry` (hosted at `api.getbodhi.app`), `mcp-server-registry` (admin-owned, Bodhi DB), `mcp-instance` (user-owned, Bodhi DB), `mcp-tool` (live from connected instance). Discover cards derive their state by joining entities 1+2+3 into five states: `catalog-only` / `pending-approval` / `approved` / `connected` / `disabled`. This five-state collapse is the central UX contract — one card shape, five CTAs, all derivable.
- **Catalog source is Bodhi-curated only for this pass.** Rich metadata (logo, publisher, category, stats, screenshots-deferred) hosted at `api.getbodhi.app`. Read-through from the official MCP registry / Smithery / mcp.so is deferred; research during design did browse all 7 reference registries via Claude-in-Chrome and their schemas inform our card/detail shape.
- **Admin gate preserved with one-click pre-fill.** Admins can click a catalog card and open `McpServerForm` with every field pre-filled from the catalog entry — one review click and Save creates the registry row for the whole app instance. Users land on the same cards and see `Submit for Approval`, which files a request into the Admin Inbox. Once a server is approved, users see `+ Add MCP Server` on the same card → opens `McpInstanceForm` overlay with credentials entry (OAuth/Header) and a Connect step. Already-connected instances surface inline on the card with a `View instance ↗` chip.
- **Overlay reusability — by design.** Both `McpServerForm` and `McpInstanceForm` are plain form bodies. They render as-is in standalone pages (Admin, My MCPs) and inside `OverlayShell` when launched from Discover. No duplication, no forked code paths.
- **Card visuals encode state.** `.mcp-card.state-connected` uses leaf-soft background + leaf border; `.state-pending-approval` uses dashed + warn-soft; `.state-disabled` fades to 58% opacity. All other state-bearing signals (CTA label, inline-instance strip, pre-footer hint text) derive from the same `state` field. Do not add a sixth state without updating `mcpCardCta()` and the five-state matrix in `specs/mcp.md`.
- **Primitives added (16):** `McpCategoryChipRow`, `McpStatusFilter`, `McpCatalogCard`, `McpCatalogDrawer`, `McpServerForm`, `McpInstanceForm`, `McpAuthHeaderConfig`, `McpAuthOAuthConfig`, `McpServerListRow`, `McpApprovalRow`, `McpInstanceListRow`, `McpInstancePendingBanner`, `McpToolSidebar`, `McpToolExecutor`, `McpRail`, `McpMediumAnchors` + 6 constants (`MCP_CATEGORIES`, `MCP_CATALOG_FIXTURE`, `MCP_SERVERS_FIXTURE`, `MCP_INSTANCES_FIXTURE`, `MCP_APPROVAL_FIXTURE`, `MCP_TOOLS_FIXTURE`) + helper `mcpCardCta`.

### Access-request review (2026-04-20)

- **Replaced the minimal production review page** (dead-end when MCP not configured → see `download (26).png`) with a **self-contained 4-section card** that can resolve missing MCP prerequisites without leaving the page. New tab in the deck as the 8th position.
- **Per-model allow-list** replaces the old "Approve All" coarse grant. The 3rd-party app declares a **capability envelope** (`required`, `preferred`, `minContext`, `maxCostUsdPerMTok`) plus an optional **suggestedModels** list; the UI pre-checks matching rows and disables below-envelope rows. User retains sovereignty — they can grant rows outside the envelope (but can't grant rows that fail hard constraints like `tool-use`).
- **Per-MCP row contract: 6 states × role** (`has-instance` / `needs-reauth` / `needs-instance` / `oauth-in-progress` / `needs-server` / `pending-admin`) × (user / admin). Centralised in `AccessMcpRow`. Missing server → admin gets **one-click inline mini-overlay** (inline accordion wrapping v30 `McpServerForm` pre-filled from catalog); user gets **Request admin to add** filing into v30 Admin Inbox. Missing instance → **OAuth in popup window**; TanStack Query `refetchOnWindowFocus:true` surfaces the new instance automatically on tab refocus — this is the user-friendly magic.
- **Inline mini-overlay, not a layered modal.** Admin's `Add MCP Server` expansion lives *inside* the review card as an accordion. Rationale: keep the reviewer's approval context + the new-server config visible together; a floating modal would occlude the rest of the review.
- **OAuth popup, not same-window redirect.** Overrides the v30 MCP same-window default specifically for this flow. Rationale: same-window would destroy the review context (lose the request-id + filled checkboxes). Popup + on-focus refetch gives the seamless round-trip.
- **Role-adaptive UI, not role-hidden.** Both user and admin see every MCP row; the CTA changes per role. Users get a `Request admin to add` affordance instead of being stuck.
- **Approve button label reflects reality.** `Approve N of M resources` live-updates from checkboxes. Disabled when any checked MCP is not `has-instance` OR when zero resources selected — always with a warn-tone hint explaining why.
- **Primitives added (9):** `AccessRequestHeader`, `AccessCapsEnvelope`, `AccessModelRow`, `AccessModelGroup`, `AccessMcpRow`, `AccessMcpInlineAddServer`, `AccessOAuthHint`, `AccessRoleSelect`, `AccessActionBar` + 2 fixtures (`ACCESS_REQUEST_FIXTURE`, `ACCESS_MODELS_FIXTURE`) + helper `accessModelState`.
- **`Btn` primitive extended with `onClick`.** Was `({variant, size, children, style, title})`; now accepts `onClick` too. Enables interactive demos (inline-overlay toggle, role switch). Backward-compatible — all existing call sites that omit `onClick` keep working.

### Create local alias

- **The form started as a holistic "typed field per llama.cpp knob" design.** This was rejected after user feedback: keeping a typed form in sync with llama.cpp releases is a high-maintenance treadmill — new flags appear, semantics change, and we did not want to own that.
- **Pivot to raw cmdline args.** The editor is a **plain textarea** where each line is a `--flag value` pair. Helpers sit alongside: `ArgsPalette` (parsed `llama-server --help`) for discovery; hover tooltip (`ArgsHelpPop`) for semantics; squiggly wavy underline (`args-line-warn`) for unknown flags. User types/edits freely; the app helps without owning the shape.
- **Snapshot moved out of Identity.** Was originally in Section 1 next to alias name. Moved to Section 2 (Model file) because snapshot is a property of the underlying model file, not of the user's naming/identity. Keep it in Section 2.
- **Filename field was removed entirely.** The `QuantPicker` grid IS the file selector — picking `:Q4_K_M` selects the file. A separate Filename input would double-input the same decision. Do not re-add a Filename field.
- **Presets and Server-args were merged into one section (Section 3 now).** Originally they were two separate collapsible sections. Merging them creates a feedback loop: user picks a preset → the args editor updates live; user tweaks args → the preset chip shows "Custom" automatically. Keep them merged.
- **The preset catalogue has 18 entries (not the earlier 5).** Final set: Default, Chat, Coding, Agent, Reasoning, RAG (short), RAG (long), Vision, Embed, Max Performance, Max Context, Parallel — Medium, Parallel — Max, Hardware Use — Medium, Hardware Use — Max, Long-ctx, Small, Custom. Adding presets is fine; removing any without user discussion is not, because each maps to a specific user intent we explicitly validated.

### Specialization filter

- **Was originally "Browse by Task | Capability | Family" as a dedicated view.** This was rejected: the view changed the main content drastically, and users reported it was confusing because the page shape changed when they thought they were just filtering.
- **Now it's the first filter group** (single-select, default `All`, with `Clear`). The benchmark-driven sort kicks in as a side-effect of selection. This is strictly more composable — users can combine `Specialization: Coding` with any other filter.

---

## 7. Deferred / out-of-scope items

These were explicitly punted during design. A future agent proposing work in this area should check here first before assuming something is missing.

- **`ApiModelPanel`** — api-model rows currently dispatch to `ConnectedProviderPanel` with the specific model row highlighted. Extracting a dedicated `ApiModelPanel` with per-model override UI is deferred.
- **Unifying Hub's `ProviderPanel` with Discover's `ConnectedProviderPanel`.** Two similar panels still exist. Collapsing them is deferred; the duplication is not load-bearing but the refactor has more surface than it looks.
- **Actually deleting `screens/hub.jsx` and `screens/providers.jsx`.** Both are archived but still on disk. Fully deleting them requires a pass over `index.html` `<script>` tag ordering and downstream cross-references. Deferred.
- **Real data fetching for the provider directory.** Wireframe uses static samples. Integration with `api.getbodhi.app` is a build task, not a wireframe task.
- **Matrix-comparison view (was Provider Directory Variant B).** Dropped explicitly in the 2026-04-19 pass. Do not re-introduce as if novel.
- **Needs-based matcher (was Provider Directory Variant C).** Dropped explicitly in the 2026-04-19 pass. Specialization + Capability filters are the replacement. Do not re-introduce.
- **Benchmark visualization bars, quant slider, community rating card (was Model Detail Variant B).** Dropped explicitly in the 2026-04-19 pass. These three unique surfaces do not exist anywhere in the wireframe today. The user noted they can be added to `HfRepoPanel` later if required — but are not considered missing work.
- **Full-page detail layout** (two-column breathing-room variant). Dropped with Model Detail. Current dispatch (right-drawer on desktop, bottom-sheet on mobile) is the canonical detail UX.
- **Separate "Add API model from connected provider" screen.** The `+ ▾ Add model` menu has a distinct "Add API model" entry (badge "from connected"). For this pass it routes to the same `api.jsx` form with provider pre-picked; a dedicated picker screen (list connected providers → choose model) is deferred.
- **Per-API-model override UI** (temperature defaults, system prompt defaults at the API-model level, matching what `AliasPanel` shows for local aliases) — not wireframed. Deferred.
- **OAuth lifecycle UI** (anthropic-oauth PKCE, google-gemini OAuth tokens, expiry/re-auth flows) — `API_FORMATS` lists these formats but the form assumes key-based auth. A future pass will handle OAuth in both `api.jsx` and `ConnectedProviderPanel`.
- **Stepper UX for API create** (was api.jsx Variant B). Dropped. Do not reintroduce — user rejected.
- **Provider-aware editor with per-model cost inline** (was api.jsx Variant C). Dropped. The cost-per-model surface belongs in `ConnectedProviderPanel` if it lands at all.
- **Explicit benchmark-sort dropdown (non-Specialization trigger).** Ranked mode is currently only activated via a Specialization selection. A future explicit "Sort by benchmark X" dropdown is deferred.
- **Benchmarks beyond HumanEval/Coding.** The `RANKED_FIXTURE_CODING` fixture covers one specialization end-to-end. Other specializations (Chat/MT-Bench, Reasoning/GPQA, etc.) show ranked-mode UI but reuse the same fixture for demo. Real per-benchmark fixtures are deferred.
- **Re-ranking under filter scope.** Explicitly rejected: when filters narrow visible rows, the rank numbers stay absolute. A "rank within filtered subset" mode would undermine the "#2 in the world" semantic.

### MCP deferred (added v30)

- **External registry read-through** — official MCP registry (`registry.modelcontextprotocol.io`) / Smithery / mcp.so / Docker Hub MCP / mcpmarket. For this pass, Discover is Bodhi-curated only. A second read-through tier (long-tail search across external registries) is a future pass.
- **Community-submission flow** for new catalog entries (user proposes a server → Bodhi admins vet it into the curated catalog). Not wireframed.
- **Per-tool live analytics** (uptime / latency / call-count time-series charts in the Capabilities drawer). Research showed Smithery does this; our drawer renders static numbers for MVP.
- **Resources and Prompts** — MCP spec defines three capability classes; current wireframe is tools-only, matching the existing MCP decisions log.
- **stdio-based MCP transport** — HTTP-streamable only for MVP, matching the existing MCP decisions log.
- **Needs-reauth refresh overlay** — `mcp-instance` rows show a `⚠ reauth` chip when `authState === 'needs_reauth'`; the OAuth refresh overlay itself is not wireframed. Clicking the chip in production should re-run the OAuth flow.
- **Audit log / activity history.** Admin has an Approvals inbox but no full timeline of approve/reject/disable actions. Deferred.
- **Playground history / saved invocations.** `McpToolExecutor` resets on tool switch; re-run history and named saved examples are a future pass.
- **Role-switcher UI.** The Desktop Discover variant has a "Role: User ▾" chip so the wireframe can demonstrate both admin and user CTAs; a real role switch in the product would be automatic from auth context.
- **Bulk actions** — no multi-select on server, instance, or approval lists.
- **Deep-link to Playground with tool + params encoded.** My MCPs `▷` jumps to Playground with instance pre-selected; encoding the tool name + params in the URL is deferred.

### Access-request deferred (added v31)

- **Per-tool MCP scoping.** Grants are whole-server today; per-tool (OAuth-scope-like) grants deferred. Catalog already carries the tool list, so the wiring exists.
- **Temporal grants / expiry.** Approvals permanent until revoked. Time-boxed grants (30d / 90d) deferred.
- **Rate-limit overrides per app.** No per-app rpm / tpm throttles.
- **Cost ceilings per app.** No hard budget caps in this screen.
- **Header-auth MCP inline form.** Today's `needs-instance` CTA assumes OAuth popup. For header-auth servers the inline mini-overlay should render `McpInstanceForm` inline instead of opening a popup — deferred.
- **Request audit log / history** (My approved apps view) — separate screen, out of scope.
- **Batch review across multiple requests.** One request at a time for now.
- **Popup-blocked / OAuth-failed recovery UX.** Wireframe shows `oauth-in-progress`; failure-retry flow deferred.
- **App icon upload / directory.** Fixture uses letter glyph; production renders a URL. A Bodhi 3rd-party app directory is far-future.
- **Role auto-detection.** Wireframe has a manual User/Admin toggle chip in the header for demo purposes; production uses auth context.
- **New entity kinds.** Custom endpoints, local OpenAI-proxy, etc. — not in this iteration. If a 7th kind is added, it goes into the `kind`-dispatch pattern; see `discover.jsx` for the switch.

---

## 8. File layout reference

```
design/models-page/
├── project/                # The wireframes themselves
│   ├── index.html          # Babel-standalone runtime, script tag ordering matters
│   ├── app.jsx             # SCREENS array + Root component + tab persistence
│   ├── wireframes.css      # All styles
│   └── screens/
│       ├── primitives.jsx  # Shared components (exported via window.*)
│       ├── hub.jsx         # SLIMMED to just AliasPanel/FilePanel/ProviderPanel
│       ├── discover.jsx    # The unified Models page (3 variants)
│       ├── alias.jsx       # Create local alias (4 variants)
│       ├── api.jsx         # Create API model
│       ├── providers.jsx   # ARCHIVED — not loaded (absorbed into Models)
│       └── detail.jsx      # ARCHIVED — not loaded (absorbed into Models right-drawer dispatch)
└── specs/                  # This folder — archival spec docs
    ├── shared-primitives.md  (this file)
    ├── models.md
    └── alias.md
```

Cache-buster: every `<script type="text/babel">` tag in `index.html` ends with `?v=N`. Last bumped to `?v=29` for the Create API Model consolidation pass. Bump on every jsx/css change.

**Loaded scripts** (in order, from `index.html`): `primitives.jsx` → `hub.jsx` → `discover.jsx` → `alias.jsx` → `api.jsx` → `app.jsx`. `providers.jsx` and `detail.jsx` are intentionally NOT in the loaded list — both are archived. Do not re-add them.

**Top-level tabs** in the wireframe: `Models | Create local alias | Create API model` (three tabs; all detail-level browsing happens as dispatched panels inside `Models`).
