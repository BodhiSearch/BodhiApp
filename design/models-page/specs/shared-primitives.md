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

**AI-agent note:** When adding new primitives, follow the same pattern — define locally in `primitives.jsx`, export via `Object.assign(window, {...})` at the bottom. Do not use ES imports; the wireframe is babel-standalone / no bundler.

---

## 5. Interaction conventions that apply across both screens

These are baseline behaviours that recur and should be preserved.

### Filter grey-out vs hide

When a filter group does not apply in the current mode (e.g. `Cost · api` in `My Models` mode), **grey it out** (`.filter-group-disabled` — opacity 0.5, `pointer-events: none`, sub-caption "not applicable in My Models"). Do not hide it. Rationale: layout stability matters more than compactness; hiding groups causes the sidebar to jump every time the user toggles.

### Kind chip multi-select

`KindChipRow` is multi-select with `All` as the only exclusive state. Selecting any specific kind(s) clears `All`; clicking `All` clears the specific kinds. This mirrors the filter-set pattern of "Aliases + API models" (i.e. "things I can use in chat") as a common subset.

### localStorage migration

Tab state persists in `localStorage.bodhi-wf-tab`. A migration shim in `app.jsx` rewrites legacy `hub` / `discover` values to `models` on first load. Do not remove this shim without a plan for what happens to users on old browsers — the wireframe is hosted on a static server and users may have stale localStorage.

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
- **Actually deleting `screens/hub.jsx`.** The file is slimmed to just the three panels. Deleting it fully requires a pass over `index.html` `<script>` tag ordering. Deferred.
- **Real data fetching for the provider directory.** Wireframe uses static samples. Integration with `api.getbodhi.app` is a build task, not a wireframe task.
- **Removing the `Provider directory` tab from `SCREENS`.** It's still in `app.jsx`'s nav list as a "just providers" view. Removing it was out of scope for the Hub/Discover unification pass.
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
│       ├── providers.jsx   # Provider directory (kept as a separate tab)
│       └── detail.jsx      # Model detail (side-drawer / page / sheet)
└── specs/                  # This folder — archival spec docs
    ├── shared-primitives.md  (this file)
    ├── models.md
    └── alias.md
```

Cache-buster: every `<script type="text/babel">` tag in `index.html` ends with `?v=N`. Last bumped to `?v=26` after PhoneFrame re-addition. Bump on every jsx/css change.
