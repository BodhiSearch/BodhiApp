# Models page — functional spec & context

*Companion doc: `shared-primitives.md` (read first — this doc builds on the six-entity model defined there).*

*Audience: Claude / AI coding agents first, developers second. Wireframe lives in `design/models-page/project/screens/discover.jsx` (filename kept for minimal `index.html` churn; internal function names are `ModelsDesktop`, `ModelsMedium`, `ModelsMobile`).*

---

## 1. What this page is

A single page that lists every "model-shaped thing" in BodhiApp's world, behind one mode toggle. Replaces the earlier split between **Models Hub** (user's owned things) and **Discover** (catalog). Partition is a filter, not a page.

Entry points:
- `app.jsx` → `SCREENS` entry `{key:'models', title:'Models', list:()=>window.ModelsScreens}`
- Renders 3 variants: Desktop, Medium (tablet), Mobile — all now in `discover.jsx`, exposed as `window.ModelsScreens`.

---

## 2. Top-level anatomy (desktop)

```
┌──────────────────────────────────────────────────────────────────────────┐
│  Models                                 [+ ▾ Add model]  [⋯ ▾ Browse]    │  ← page header
├──────────────────────────────────────────────────────────────────────────┤
│  [●] My Models (14)     [ ] All Models (3.1M + 23 directory)             │  ← toolbar row 1: ModeToggle
│  All · Aliases · Files · API models · Providers · HF repos   sort ▾ ▦☰   │  ← toolbar row 2: KindChipRow + sort + view
├──────────┬───────────────────────────────────────────────────────────────┤
│ FILTERS  │  ROWS                                                         │
│          │    [alias]  my-gemma            …   ↗ catalog                 │
│ Special- │    [file]   google/gemma-2:Q5   …   ↗ catalog                 │
│  ization │    [api]    openai/gpt-5-mini   …   ↗ openai                  │
│ Kind     │    [prov]   openai (connected)  …                             │
│ Source   │    [hf]     Qwen/Qwen3.5-9B     …   ✓ 2 local aliases ↗       │
│ Capab.   │    …                                                          │
│ Size·rig │                                                               │
│ Cost·api │                                                               │
│  (dimmed │                                                               │
│   in My) │                                                               │
│ License  │                                                               │
│ Format   │                                                               │
└──────────┴───────────────────────────────────────────────────────────────┘
```

- Clicking a row opens the right-side detail panel (overlays the rows in the wireframe).
- The **left rail** (global, not page-specific) has `Bodhi` brand + `Models` leaf + `↓ Downloads` menu.

---

## 3. The mode toggle

### Semantics

| Mode | Row kinds visible | Default sort | Cost·api filter |
|---|---|---|---|
| `My Models` (default) | `alias`, `file`, `api-model`, `provider-connected` | Recently used | dimmed |
| `All Models` | all 6 kinds (local entities tagged `local`) | Likes | active |

### Counts

- `My Models (N)` — count of **local entities matching all other filters**.
- `All Models (M + D directory)` — `M` = HF catalog matches, `D` = provider directory entries. Keep the `+ D directory` phrasing — it is the one place users learn that `All` draws from multiple sources.

### Default

Default mode is `My Models`. Rationale: most sessions are "what do I have / what am I using", not "let me browse 3M repos". The toggle is persistent across sessions (lives in the local state; no separate localStorage key beyond what the variant stores).

### The caption

A one-line caption (`ModeToggleCaption` primitive) sits under the toggle: *"All Models draws from HuggingFace (catalog) and Bodhi Directory (providers)."* This is a one-time explainer; easy to suppress later with a dismiss affordance.

---

## 4. Filter sidebar (8 groups, unified)

Order is **stable** across modes. Groups that don't apply in the current mode get greyed (`.filter-group-disabled`) with a caption "not applicable in My Models" — never hidden. (See `shared-primitives.md` §5 for the rationale.)

| # | Group | Selection | My | All |
|---|---|---|:---:|:---:|
| 1 | **Specialization** (single-select, `Clear`) | All / Chat / Coding / Agent / Reasoning / Long-ctx / Multiling / Vision / Embed / Small | ✓ | ✓ |
| 2 | **Kind** (multi-select) | Aliases / Files / API models / Providers / HF repos | ✓ | ✓ |
| 3 | **Source** (multi-select) | HuggingFace / Bodhi Directory / OpenAI / Anthropic / Groq / Together / NVIDIA NIM / HF Inference / OpenRouter / Google | ✓ | ✓ |
| 4 | **Capability** (multi-select) | tool-use / vision / structured / embedding / reasoning / speech / image-gen | ✓ | ✓ |
| 5 | **Size · rig** | Fits rig ✓ / <5GB / 5–15GB / >15GB | ✓ (aliases/files only) | ✓ (aliases/files/hf-repo) |
| 6 | **Cost · api** | Free-OSS / $<1/M / $1–5 / $>5 / ≥99% up | dimmed | ✓ |
| 7 | **License** | Apache-2 / MIT / Llama / Gemma / Proprietary | ✓ | ✓ |
| 8 | **Format** | GGUF / openai-responses / openai-completions / anthropic-messages / anthropic-oauth / google-gemini | ✓ | ✓ |

### Specialization's side-effect sort

Selecting a specialization swaps the sort to its canonical benchmark. Full mapping (source of truth in `SPECIALIZATIONS` constant, `primitives.jsx`):

| Specialization | Sort key |
|---|---|
| Chat | MT-Bench |
| Coding | HumanEval |
| Agent | ToolBench |
| Reasoning | GPQA |
| Long-ctx | RULER |
| Multiling | MMLU-X |
| Vision | MMMU |
| Embed | MTEB |
| Small | (size ascending) |

This sort persists across the mode toggle — `Coding + My Models` means "my things, ranked by HumanEval". `Clear` returns to the mode's default sort (Recently used / Likes).

### Bodhi Directory as a source

`Source` filter lists `Bodhi Directory` alongside HuggingFace and per-provider sources. Filtering to it shows only provider-unconnected rows (directory data). Hiding it suppresses those rows even in `All Models`.

---

## 5. Kind chips (second toolbar row)

`KindChipRow` — multi-select with `All` as the exclusive default state. Click behaviour:

- `All` + click any kind → clears `All`, activates the kind.
- Any kind + click another kind → both active.
- Any subset + click `All` → clears subset, activates `All`.

Why multi-select: users asked for "Aliases + API models" as a subset that means "things I can use in chat right now" — so we keep it composable rather than forcing single-kind views.

---

## 6. Sort (mode-aware defaults)

| Mode | Default sort | Options |
|---|---|---|
| `My Models` | **Recently used** | Recently used · Recently added · Name · Size · Last run |
| `All Models` | **Likes** | Likes · Downloads · Recent · *benchmark-driven* (when Specialization active) |

Sort dropdown lives on toolbar row 2 next to the kind chips and the Cards/List view toggle.

---

## 7. Row shapes (summary — full spec in shared-primitives.md §3)

- **File-first** (`alias` / `file` / `api-model`): one row per user-configured thing.
- **Repo-first** (`hf-repo`): one row per repo; quants shown nested in detail panel.
- **Provider-summary** (`provider-connected` / `provider-unconnected`): one row per provider with model-count.

### Row-level annotations

Every row in the main list supports these optional props (see `ModelListRow` in `primitives.jsx`):

| Prop | Who sets it | Visual |
|---|---|---|
| `localBadge` | `All` mode on any local-kind row | Saffron `local` pill (`.row-local-badge`) |
| `backlink` | `alias` / `file` / `api-model` | Indigo italic `↗ catalog` / `↗ openai (provider)` (`.row-backlink`) |
| `catalogAliases` | `hf-repo` when local aliases reference this repo | Indigo pill `✓ 2 local aliases ↗` (`.row-catalog-aliases-badge`) |
| `directoryAttribution` | `provider-unconnected` | Tiny `from api.getbodhi.app` caption (`.row-directory-attribution`) + dashed row border |

Clicks on the **row body** open the primary detail panel. Clicks on `backlink` / `catalogAliases` / `directoryAttribution` navigate to the linked entity. Do not conflate them — row-body vs. inline-link is the primary interaction contract.

---

## 8. Right-panel dispatch

Single dispatch switch based on `selectedRow.kind`:

| Row kind | Panel |
|---|---|
| `alias` | `AliasPanel` (from `hub.jsx`) |
| `file` | `FilePanel` (from `hub.jsx`) |
| `api-model` | `ConnectedProviderPanel` with the specific model row highlighted *(deferred: dedicated ApiModelPanel)* |
| `provider-connected` | `ConnectedProviderPanel` |
| `provider-unconnected` | `UnconnectedProviderPanel` (connect flow) |
| `hf-repo` | `HfRepoPanel` |
| `↓ Downloads` menu click | `DownloadsPanel` |

All panel definitions still live where they were before the unification. `hub.jsx` keeps `AliasPanel` / `FilePanel` / `ProviderPanel` even though Hub itself is gone; `discover.jsx` imports nothing explicitly, it just uses the `window.*` exports.

---

## 9. Header action menu

**Desktop**: two buttons, `+ ▾ Add model` (primary) + `⋯ ▾ Browse` (secondary).

`+ ▾ Add model` groups:
- Add by HF repo
- Paste URL
- Add API provider
- Add api-model from connected provider

`⋯ ▾ Browse` groups:
- Trending
- New launches
- Leaderboards

**Mobile / medium**: single `+ ▾` button with both Add and Browse sections (`ModelsAddBrowseMenu` primitive handles both layouts — check the `compact` prop).

---

## 10. Responsive deltas

### Medium (tablet)

- Same toolbar row 1 (mode toggle) + row 2 (kind chips + sort + view).
- Sidebar collapses to a `Filters` button opening a sheet.
- Active-filters strip below the toolbar shows currently-applied filters as removable chips, starting with the Specialization pill.

### Mobile

- Breadcrumb is `Bodhi › Models` (single segment — `MobileMenu` no longer has a `Models → { My Models, Discover }` sub-tree; it is a **leaf**).
- Toolbar row 1: compact mode pill (just `My · All` with the active highlighted).
- Toolbar row 2: kind chip row (horizontal scroll) + `Filters` button + sort.
- Mobile filter sheet leads with Specialization + Kind groups.
- Header `+ ▾` is one grouped menu (Add + Browse).

---

## 11. LocalStorage and tab migration

`app.jsx` persists the active tab under `bodhi-wf-tab`. The migration shim:

```js
// Rewrites legacy keys on first load of the new layout
if (cur === 'hub' || cur === 'discover') {
  localStorage.setItem('bodhi-wf-tab', 'models');
}
```

Keep this shim until the wireframe is retired. Users visiting the static-hosted wireframes with stale localStorage land on a non-existent tab otherwise.

---

## 12. Decisions archive (context not visible in the wireframe)

These are the design decisions made during the Hub+Discover → Models unification that the wireframe alone doesn't tell you. A future agent picking this up should know these to avoid re-opening settled questions.

1. **Partition as filter, not page.** See `shared-primitives.md` §6 for the "why". The single biggest design move.

2. **Default to `My Models`.** Explicitly chosen. Users opening the page 90% of the time want to see their own things; the toggle is a one-click escape to catalog.

3. **Counts stay disaggregated.** `All Models (M + D directory)` is deliberately honest that the number is a sum across sources. We do not hide multi-source provenance.

4. **Three row shapes, not one.** Rejected a forced unification. Rationale in `shared-primitives.md` §3.

5. **Filter groups grey out, never hide.** Layout stability beats compactness. `Cost · api` in `My Models` mode is the canonical example.

6. **Specialization is the only single-select filter with a `Clear`.** All other groups are multi-select. The single-select is because the benchmark-driven sort is one-at-a-time by nature.

7. **Bodhi Directory is first-class.** Not hidden behind flags, not collapsed into "other". Shown with dashed border, attribution chip, and as a `Source` filter option. This mirrors the product intent: users should know there's a hosted directory and opt in/out of it visibly.

8. **`hub.jsx` is slimmed, not deleted.** Kept for panel exports; removing it touches `index.html` `<script>` ordering and has more blast radius than its single file suggests.

9. **The earlier `Browse by Task | Capability | Family` view is gone.** It was a dedicated view that changed the main content shape dramatically. Moved to the `Specialization` filter (single-select, default All, with Clear). The benchmark-driven sort is the remaining signature of the old view.

10. **Detail panels were not redesigned.** The unification is a list-layer change only. `AliasPanel`, `FilePanel`, `HfRepoPanel`, etc. remain exactly as they were. Re-designing them was out of scope; if an AI agent is tempted to re-style them, check with the user first.

---

## 13. Verification checklist (for smoke-testing after changes)

When modifying `discover.jsx` / `primitives.jsx` / `wireframes.css`:

1. Reload `http://localhost:8000/` in Chrome (cache-buster already set via `?v=N`).
2. In console: `localStorage.setItem('bodhi-wf-tab','models'); location.reload();`
3. **Top nav**: no `Models Hub` tab. `Models` present. Other tabs untouched.
4. **Desktop variant**:
   - Toolbar row 1 is the mode toggle with two counts.
   - Toolbar row 2 is kind chips + sort + view toggle.
   - Sidebar filter order: Specialization → Kind → Source → Capability → Size·rig → Cost·api → License → Format.
   - In `My Models`: at least one `alias`, `file`, `api-model`, `provider-connected`. No `hf-repo` or `provider-unconnected`. `Cost · api` dimmed.
   - In `All Models`: all kinds appear; local entities carry `local` saffron badge.
   - `hf-repo` rows with local aliases display `✓ N local aliases ↗`.
   - `alias` rows display `↗ catalog · …`.
   - `provider-unconnected` rows show dashed border + `from api.getbodhi.app`.
5. **Medium variant**: mode pill at top, `Filters` button, active-filters strip.
6. **Mobile variant**: breadcrumb `Bodhi › Models` only (no third segment). `MobileMenu` shows `Models` as leaf.
7. **+ ▾ menu**: grouped with Add + Browse sections.
8. **Tweaks panel** (annotations / texture / hand / color toggles): still works.

---

## 14. Out of scope / deferred

See `shared-primitives.md` §7 for the full deferred list. Specifically for this page:
- `ApiModelPanel` extraction
- Real `api.getbodhi.app` fetching
- Removing the `Provider directory` top-level tab
- Deleting `hub.jsx` entirely
- New entity kinds (custom endpoints, local OpenAI-proxy)
