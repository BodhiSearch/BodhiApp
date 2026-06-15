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

**Benchmark sort → ranked display mode.** Picking any non-`All` Specialization activates a benchmark sort and automatically switches the main list to **ranked display mode** (see §8). Clearing the Specialization returns to the mode's default sort and the entity-level row shape.

---

## 7. Row shapes (summary — full spec in shared-primitives.md §3)

- **File-first** (`alias` / `file` / `api-model`): one row per user-configured thing.
- **Repo-first** (`hf-repo`): one row per repo; quants shown nested in detail panel.
- **Provider-summary** (`provider-connected` / `provider-unconnected`): one row per provider with model-count.
- **Ranked** (model-level): used only under ranked display mode (§8). Replaces the three entity-level shapes above while a benchmark sort is active.

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

## 8. Ranked display mode

Added in v27. When a benchmark sort is active, the main list swaps from entity-level rows to **model-level** ranked rows. This gives leaderboards a row shape that actually matches their semantic ("model #2 on HumanEval") rather than forcing ranks onto repo/file/provider-level rows.

### Activation

Ranked mode is ON when a **benchmark sort** is active. The benchmark sort is implied by a Specialization selection; each Specialization maps to its canonical benchmark in the `SPECIALIZATIONS` constant (`screens/discover.jsx:305`):

| Specialization | Benchmark |
|---|---|
| Chat · general | Arena Elo |
| Coding | HumanEval |
| Agentic · tool-use | BFCL |
| Reasoning | GPQA |
| Long context | RULER |
| Multilingual | mMMLU |
| Vision + text | MMMU |
| Text embedding | MTEB |
| Multimodal embed | MMEB |
| Small & fast | Open LLM LB |

Ranked mode turns OFF when Specialization returns to `All`. (A future explicit benchmark-sort dropdown would also trigger ranked mode; deferred — see §15.)

### Row anatomy

```
┌──────┬──────────────────────────────────────┬───────────────────────┐
│ #N   │ primary-identifier-line-1            │ 82.1 HumanEval        │
│      │ primary-identifier-line-2 (if stack) │ [use →] / [pull →]    │
│      │ primary-identifier-line-3 (if stack) │                       │
│      │ underlying-identity-line             │                       │
└──────┴──────────────────────────────────────┴───────────────────────┘
```

- `#N` — rank number, large hand-drawn numeral (`.rank-number`).
- `primary-identifier-line-*` — alias name(s), api-alias name(s), or the raw canonical identifier if no local backing. Stacked when multiple configurations exist.
- `underlying-identity-line` — the canonical model identity with a kind tag (`modelfile` / `huggingface` / `provider` / `from-directory`). Subdued; separated by a dashed rule above.
- Right meta — benchmark score + action chip (`use` / `pull` / `Connect to use` / `+ Create alias`).
- Background tint: `.rank-local` (saffron) for local-backed rows; `.rank-directory` (dashed border) for Bodhi Directory entries.

See `primitives.jsx` → `RankedRow`, `RankedModeCaption`, `groupIntoRankedRows`, `RANKED_FIXTURE_CODING`.

### Dedup + stack rules (the core contract)

| Situation | Rendered as |
|---|---|
| Local file downloaded, 1 alias | `alias-name` · `owner/repo:quant (modelfile)` |
| Local file downloaded, N aliases | `alias-1` · `alias-2` · … · `owner/repo:quant (modelfile)` |
| Local file downloaded, 0 aliases (orphan) | `owner/repo:quant (modelfile · orphan)` + `+ Create alias` |
| Local file NOT downloaded | `owner/repo:quant (huggingface)` + `pull →` |
| API model, user has 1+ api-aliases on it, provider connected | all api-aliases stacked · `provider/model (provider — connected)` |
| API model, 0 user aliases, provider connected | `provider/model (provider — connected)` + `use →` |
| API model, provider in Bodhi Directory (unconnected) | `provider/model (from api.getbodhi.app)` + `Connect to use →` |

**The asymmetry in one sentence:** local file dedup collapses the HF entry (downloaded file = shared bits); API configs stack without dedup (each config = distinct user choice).

### Scope across modes

| Mode × Specialization | Ranked list contents |
|---|---|
| `My Models` + `All` | Non-ranked. Entity-level rows. |
| `My Models` + `Coding` (etc.) | Ranked, local items only (`alias` / `file` / `api-model` / `provider-connected`). Rank numbers stay **absolute** — an entry at global #6 shows as `#6` even if it's the 3rd local row. |
| `All Models` + `All` | Non-ranked. Mixed entity-level rows. |
| `All Models` + `Coding` (etc.) | Ranked, full leaderboard with dedup/stack applied. |

### Filter-vs-ranking interaction

Sidebar filters (Source / Capability / Kind / License / Format / Size / Cost) narrow the **visible** rows but never renumber ranks. An entry at global `#42` still shows `#42` even when higher-ranked rows are filtered away. This preserves the "#2 in the world on HumanEval" semantic rather than making ranks context-dependent.

### Caption

When ranked mode is active, a `RankedModeCaption` banner sits below the toolbar with the exact copy:

> *Ranked by **HumanEval** (Coding). Local downloads shown as your aliases; API models stack all configurations. Filtering hides rows but keeps rank numbers.*

Dismissible (clicking `×`). Not load-bearing — purely honest-disclosure.

---

## 9. Right-panel dispatch

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

**Ranked-mode selections** use the same dispatch table: the entry's `dispatchKind` (one of `alias` / `file` / `api-model` / `provider-connected` / `provider-unconnected` / `hf-repo`) maps to the panel exactly as above. `sel='rank-${N}'` is the selection key shape; `discover.jsx` resolves this via `RANKED_FIXTURE_CODING.find(e => e.rank === N)` and renders the matching panel.

---

## 10. Header action menu

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

**Why this replaces the old Provider Directory tab**: "Add API provider" in this menu IS the Provider Directory's connect flow; the browse-provider behaviour is reproduced by `Kind=Providers + Source=Bodhi Directory` in the filter sidebar. See §13 for the full absorption rationale.

**What each menu item routes to:**
- `Add by HF repo` → Create local alias overlay (`AliasOverlay`, from `screens/alias.jsx`).
- `Paste URL` → Same alias overlay, pre-filled with a `.gguf` / `hf://` URL.
- `Add API provider` → Create API model overlay (`ApiOverlay`, from `screens/api.jsx`). Primary entry into the consolidated api-create flow.
- `Add API model` (badge "from connected") → Placeholder for a dedicated "pick from connected provider" picker. Currently routes to the same `ApiOverlay` with the provider chip pre-selected. A dedicated picker screen is deferred (see `shared-primitives.md §7`).

---

## 11. Responsive deltas

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

Mobile + medium both gain a **ranked-mode frame** in the variant deck (PhoneFrame "6 · Ranked (Coding)" / TabletFrame "4 · Ranked (Coding · HumanEval)") showing a compact ranked list. The caption is the same text as desktop.

---

## 12. LocalStorage and tab migration

`app.jsx` persists the active tab under `bodhi-wf-tab`. The migration shim:

```js
// Rewrites legacy keys on first load of the new layout
if (cur === 'hub' || cur === 'discover' || cur === 'providers') {
  localStorage.setItem('bodhi-wf-tab', 'models');
}
```

Keep this shim until the wireframe is retired. Users visiting the static-hosted wireframes with stale localStorage land on a non-existent tab otherwise.

---

## 13. Decisions archive (context not visible in the wireframe)

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

11. **Provider Directory absorbed (2026-04-19).** The top-level `Provider directory` tab was removed. Rationale: after the Hub+Discover unification, providers became first-class rows on the Models page; the directory's remaining unique surface (Variant B matrix, Variant C needs-matcher) was not worth the cost of a parallel page. Variant B + Variant C were dropped explicitly; Variant A (logo gallery) is reproduced by Models' Kind=Providers + Source=Bodhi Directory filters + the `+ ▾ Add model` menu's "Add API provider" item. `providers.jsx` is archived (not loaded) rather than deleted.

12. **Ranked display mode chosen over tiny-rank-numbers-on-current-rows.** When benchmark sort is active, the alternatives considered were (a) showing `#N` badges on the existing entity-level rows, or (b) switching to a model-level row shape. (a) was rejected: rows are entity-scoped (repo, file, provider) but leaderboards are model-scoped. Putting `#2` on an hf-repo row is ambiguous — which of its 5 quants is #2? Model-level rows make the rank unambiguous.

13. **Local files dedup; API configs stack — the core asymmetry.** The single most important rule of ranked mode. Captured in the caption text so it's surfaced to users, not buried in docs. The asymmetry exists because local files are *shared* (any user on the system aliases the same bits) while API configs are *distinct choices* (different auth, different overrides).

14. **Absolute rank numbers.** Filters narrow visible rows but never renumber. Rank-within-filter-scope was explicitly rejected: it would undermine the "#2 in the world" semantic and turn rank into a confusing context-dependent integer.

15. **Model Detail tab absorbed (2026-04-19).** The top-level `Model detail` tab was removed. Rationale: all three variants showed only `hf-repo` entities, and two variants (side-drawer, bottom-sheet) were already how the Models page dispatches `HfRepoPanel`. The third variant (full-page) had three unique surfaces — benchmark Bar charts, interactive quant slider, community rating card — all of which were dropped by user decision (not ported). The noted follow-up option: "can be added to HfRepoPanel if required." `detail.jsx` is archived on disk.

16. **Create API Model uses one flat form, not accordions.** Rationale: API creation has fewer decisions than local alias (no runtime config, no llama.cpp flags). Accordions would add ceremony for a 3-section form. Kept Option A's flat layout; enriched the fields to match the production screenshot (toggles, radio, full Model Selection UI); extended to 4 variants matching alias.jsx.

17. **Conditional Model Selection in api.jsx breaks our normal grey-out convention.** Elsewhere (e.g. `Cost · api` filter in My Models) we grey-out inapplicable groups to keep layout stable. For api.jsx's Model Selection section, when `Forwarding = "Forward all"` we **hide** the section entirely. Rationale: it's a form-specific affordance (not a filter on the shared list), and keeping it visible-but-greyed would imply users should interact with it "once they understand the mode" — which is the wrong mental model. Forward-all means "no selection needed"; hiding the section makes that obvious.

---

## 14. Verification checklist (for smoke-testing after changes)

When modifying `discover.jsx` / `primitives.jsx` / `wireframes.css`:

1. Reload `http://localhost:8000/` in Chrome (cache-buster already set via `?v=N`).
2. In console: `['providers','detail','hub','discover'].forEach(k => { localStorage.setItem('bodhi-wf-tab', k); location.reload(); })` — each stale key lands on `Models` after reload with stored value rewritten to `models`. Shim working.
3. **Top nav**: 3 tabs only — `Models | Create local alias | Create API model`. No `Provider directory`. No `Model detail`.
4. **Desktop variant (ranked mode ON — Specialization = Coding by default)**:
   - Ranked-mode caption visible below toolbar with HumanEval/Coding copy.
   - Rows render as `.rank-row` (numbered #1…#9). Fixture population.
   - Entry #1: `claude-sonnet-4.5 (api-alias)` + `anthropic (provider — connected)` identity, `82.1 HumanEval`.
   - Entry #2: TWO stacked api-aliases (`cc/opus-4-6`, `cc-oauth/opus-4-6`) + `anthropic` provider identity. Stack test.
   - Entry #4: TWO stacked aliases (`my-qwen-coder`, `my-qwen-long`) + `Qwen/Qwen3-Coder-32B:Q4_K_M (modelfile)` identity. Local multi-alias stack test.
   - Entry #5: single HF-repo row (`Qwen3-Coder-32B:Q8_0`) with `pull →` action. Not-downloaded test.
   - Entry #7: orphan file (`DeepSeek-V3:Q4_K_M`) with `+ Create alias` action.
   - Entry #9: `mixtral-8x7b-instruct` with dashed border, `from-directory` tag, `api.getbodhi.app` attribution, `Connect to use →`.
5. **Toggle to `My Models`**: ranked rows shrink to 6 entries numbered `#1 #2 #3 #4 #6 #7`. Note the gaps — rank numbers stay absolute.
6. **Click Specialization → All (Clear)**: ranked rows disappear, normal `.model-card` grid returns, caption gone.
7. **Sidebar filter order**: Specialization → Kind → Source → Capability → Size·rig → Cost·api → License → Format. `Cost · api` dimmed in My Models.
8. **Medium variant**: 4 TabletFrames — grid, filter sheet, header-action menu, ranked (Coding · HumanEval).
9. **Mobile variant**: 6 PhoneFrames — browse, breadcrumb menu, filters sheet, repo detail, header-action, ranked (Coding). Breadcrumb `Bodhi › Models` (single segment).
10. **+ ▾ menu**: grouped with Add + Browse sections. "Add API provider" present (replaces Provider Directory's primary action).
11. **No console errors.**
12. **Tweaks panel** (annotations / texture / hand / color): still works.

---

## 15. Out of scope / deferred

See `shared-primitives.md` §7 for the full deferred list. Specifically for this page:
- `ApiModelPanel` extraction.
- Real `api.getbodhi.app` fetching.
- Deleting `hub.jsx`, `providers.jsx`, and `detail.jsx` entirely from disk (all three archived).
- **Matrix comparison view** (was Provider Directory Variant B) — explicit drop. Do not re-introduce.
- **Needs-based matcher** (was Provider Directory Variant C) — explicit drop.
- **Benchmark Bar charts, interactive quant slider, community rating card** (were Model Detail Variant B) — explicit drop. User noted they can land in `HfRepoPanel` later if required, but not in this pass.
- **Full-page detail layout** — dropped with Model Detail. Right-drawer + bottom-sheet dispatch is the canonical detail UX.
- **Explicit benchmark-sort dropdown** (benchmark sort without Specialization as the trigger) — deferred.
- **Ranked fixtures beyond Coding/HumanEval** — single fixture for the demo; per-specialization data deferred.
- **Rank-within-filter-scope** — explicitly rejected. Ranks stay absolute under filters.
- New entity kinds (custom endpoints, local OpenAI-proxy).
