# claude.ai/design prompt — Explore · API Models (models.dev inside Bodhi App)

> Paste this whole file into claude.ai/design. It edits the existing Bodhi design mockup, you download and sync back. **Treat this as adopting the existing Bodhi design convention, not inventing a new one — match the reference pages, reuse the shell + list kit + tokens, do NOT redesign.**

---

## 0. Read these FIRST, in order (system of record)

1. `design/Bodhi Models API.html` — the current Explore HTML entry (the file-loading convention you will clone).
2. `design/models/bodhi-models-app.jsx` — the single app, parameterized by `window.MODELS_MODE`.
3. `design/models/models-filters.jsx` — `FILTERS` config + `ShellFilterGroup` usage per mode.
4. `design/models/models-rows.jsx` — `ApiRow`, `ProviderLogo`, `Tag`, the `.m-row` markup.
5. `design/models/models-main.jsx` — list head cells, `usePagination`, `useListKeyNav`, `<Pagination>` per mode.
6. `design/models/models-detail.jsx` — `DetailBody` / `DetailHeader` per mode (the rail).
7. `design/models/bodhi-models-data.js` — `window.MODELS_DATA` mock shapes (`API_PROVIDERS`, `LOCAL_MODELS`).
8. `design/models/bodhi-models-local.jsx` — `ColumnsMenu`, `SortHeaderCell`, `BrowseGroup` (the Local polish you must match).
9. `design/models/api-model-form.jsx` — the Configure form the CTA routes to (`ApiModelForm`, `AMF_FORMATS`).
10. `design/shared/shell-chrome.jsx` (`ShellFilterGroup`), `design/shared/bodhi-list.jsx` (`Pagination`, `useListKeyNav`, `RowLink`), `design/shared/shell-core.jsx` (`SHELL_NAV`).
11. `design/shared/colors_and_type.css` + `design/shared/shell-css/shell-tokens.css` — the lotus palette + chip/tag tokens. **Use these tokens; never introduce a new color.**

---

## 1. Goal & information architecture

The Explore section becomes **two cross-linked entry points into ONE shared catalog** — think "models.dev embedded inside Bodhi App". The catalog data comes from **models.dev** (Portkey is a pricing fallback), so every field you render MUST be a real models.dev field (exact names in §4).

**(A) NEW page — `Explore · API Models` (MODELS-FIRST).** Primary list = individual models (thousands, like the models.dev landing grid). Row = one model. Detail rail = full model spec + the providers that serve it + a "Configure in Bodhi" CTA. This is the new work.

**(B) REFRESHED page — `Explore · API Providers` (PROVIDER-FIRST).** Evolves today's `'api'` mode. Row = one provider. Detail rail = that provider's models with mini specs + connection metadata. Cross-links into (A).

**Cross-linking (both directions):**
- On (A)'s model rail, the "Served by" list — each provider row deep-links to (B): `Bodhi Models API.html?select=<providerSlug>`.
- On (B)'s provider rail, a "View all models from {Provider} →" link deep-links to (A) pre-filtered to that provider: `Bodhi Models API Catalog.html?provider=<providerSlug>`.
- From a model in (A), "Configure in Bodhi" routes to the existing `ApiModelForm` (see §5).

**Architecture decision (do this):** add a **new `MODELS_MODE` value `'api-catalog'`** to the same `ModelsApp`, exactly as `'local'` and `'api'` already coexist. Add a `MODE_CFG['api-catalog'] = { subPage:'explore-api-catalog', label:'Explore · API Models' }`, a `FILTERS['api-catalog']` group set, a new `ApiModelRow` renderer, a new list-head branch, and a new `DetailBody`/`DetailHeader` branch. Do **not** fork the app or the list kit.

---

## 2. NEW page — `Explore · API Models` (mode `'api-catalog'`) — full spec

### 2a. Sidebar filters (`FILTERS['api-catalog']`, rendered as `ShellFilterGroup`s)

Mirror the existing `FILTERS` shape in `models-filters.jsx` (each group = `{icon,label,note?,clearable?, chips|range|single, value?, onSelect?}`). Wire **Provider** and **Capability** to state if cheap; the rest may be display-only `ShellFilterGroup`s like today's api mode — but they MUST render and look real.

1. **Provider** — `chips`, multi-select. One chip per seeded provider with its brand color: Anthropic (`lotus`), OpenAI (`leaf`), Google (`indigo`), Groq (`saffron`), DeepSeek (`teal`), Meta (`indigo`). icon `at-sign`.
2. **Capability** — `chips`, multi-select: `Reasoning` (indigo), `Tool use` (indigo), `Vision` (indigo), `Structured output` (muted). icon `sparkles`. (Maps to the boolean fields `reasoning` / `tool_call` / vision-from-`modalities.input` / `structured_output`.)
3. **Modality** — `chips`, multi-select: `Text` / `Image` / `PDF` / `Audio`. icon `shapes`. note `(input & output)`. (Maps to `modalities.input[]` ∪ `modalities.output[]`.)
4. **Pricing** — `range` slider `{min:0, max:75, step:0.5, unit:'/M', prefix:'$'}`. icon `dollar-sign`. note `(input $/Mtok)`. (Maps to the top-level `cost.input`, which is the model's PRIMARY/representative-provider price — authoritative per-provider prices live in `providers[].{in,out}`.)
5. **Context size** — `range` slider `{min:0, max:1000, step:8, unit:'K'}`. icon `ruler`. note `(context window)`. (Maps to `limit.context`, shown in K.)
6. **Status** — `chips`, multi-select, clearable: `Stable` (leaf), `Beta` (saffron), `Deprecated` (muted). icon `activity`. (Maps to optional `status`; its only real models.dev values are `alpha|beta|deprecated` — a stable/GA model has NO `status` field, so the synthesized `Stable` chip means "status absent". `alpha` is deliberately folded into/dropped behind `Beta` for the chip set; if you keep it distinct, add an `Alpha` (saffron) chip.)
7. **Open weights** — `single` (radio: `Any` / `Open` / `Closed`) OR a single clearable chip `Open weights` (leaf). icon `unlock`. (Maps to `open_weights` bool.)

A **Browse** sort control like Local's `BrowseGroup` is optional; if added, offer `Newest` (`release_date` desc) ↔ `Cheapest` (`cost.input` asc) ↔ `Largest context` (`limit.context` desc).

### 2b. List head + columns

Match the Local page's `SortHeaderCell` polish (compact rows, sortable headers with up/down chevrons, colorful capability chips). Columns, left→right:

| cell class | header | content | align |
|---|---|---|---|
| `lh-num` | `#` | rank index | left |
| `lh-model` (padded to clear no logo, ~16px) | `Model` | model `name` **bold** + `family` muted sub-line | left |
| `lh-ctx` | `Context` *(sortable)* | `limit.context` formatted `200K` / `1M` | right |
| `lh-in` | `Input $` *(sortable)* | `$` + `cost.input` `/M` | right |
| `lh-out` | `Output $` *(sortable)* | `$` + `cost.output` `/M` | right |
| `lh-caps` | `Capabilities` | colorful chips: Reasoning/Tool/Vision/Structured (only those true) | left |
| `lh-providers` | `Providers` *(sortable)* | count of providers serving this model, big number + `PROVIDERS` label (reuse `.m-right .m-score` styling) | right |

Use `useListKeyNav` (Arrow/Home/End eager-select, no wrap) and `RowLink` exactly as the other modes. Reuse `Tag` + `TAG_MAP` for capability chips (tool-use/reasoning/vision → `tag-indigo`, structured → `tag-muted`). Free pricing (`cost.input==0 && cost.output==0`) renders as `Free` (mirror `fmtPrice`).

### 2c. Per-model detail rail (`DetailBody` `api-catalog` branch)

`DetailHeader`: a small monogram/logo tile (reuse `ProviderLogo`-style tinted tile keyed off `family` only — `base_model` is not in the served data) + the model `name`, with `family` as a muted sub-line and a `status` badge (DISPLAY labels: `Stable`=leaf when `status` is absent, `Beta`=saffron for `beta`/`alpha`, `Deprecated`=muted for `deprecated`) and an `open_weights` chip (`Open`/`Closed`).

Then a **spec grid** (reuse the `.specs` `{k,v}` pattern) built from the REAL models.dev fields:

- **Pricing** sub-block (label `Cost · $/Mtok`): `Input` = `cost.input`, `Output` = `cost.output`, `Cache read` = `cost.cache_read`, `Cache write` = `cost.cache_write` (omit a row if the field is absent). This top-level `cost` is the PRIMARY/representative-provider price (mirroring how the backend picks a representative serving row); authoritative per-(provider,model) prices are shown per-row in the "Served by" list below via `providers[].{in,out}`.
- **Limits**: `Context` = `limit.context`, `Max output` = `limit.output`. (models.dev `limit` is `{context,output}` only — there is no `input` key.)
- **Modalities**: `Input` = `modalities.input[]` joined, `Output` = `modalities.output[]` joined (text/image/pdf/audio/video).
- **Capabilities**: chips for `reasoning`, `tool_call`, `structured_output`, `attachment`, plus `vision` (derived: `modalities.input` includes `image`). If `reasoning_options` present, show a muted note (e.g. `effort` / `budget_tokens`).
- **Meta**: `Family` = `family`, `Open weights` = yes/no, `Status` = `status`, `Released` = `release_date`, `Updated` = `last_updated`, `Knowledge cutoff` = `knowledge`, `Temperature` = `temperature` yes/no.

Then a **"Served by" provider list** — section header `Served by (N)`. Each row: `ProviderLogo` (size 26) + provider `name` + that provider's `base_url` muted, right-aligned `$in / $out` for this model at that provider. Each row deep-links to (B): `Bodhi Models API.html?select=<slug>`.

**Footer CTA: "Configure in Bodhi"** (icon `plug-zap`). Routes to the Configure form prefilled from the selected model + its primary provider (see §5).

Empty rail state when nothing selected: muted centered hint `Select a model to see its spec, pricing, and providers.`

### 2d. Search, pagination, empty state

- `ShellSearch` (⌘K) wired to the toolbar; placeholder `Search models by name, family, or provider…`.
- Real `usePagination(rows, 8, resetKey)` + `<Pagination total page onPage pageSize unit="models" />` (numbered pager, not minimal — this is a long catalog). Reset to page 1 when filters/search change.
- Empty list state (no matches): centered, muted — title `No models match these filters`, line `Clear a filter or widen the price / context range.`, plus a `Clear filters` ghost button.

---

## 3. REFRESHED page — `Explore · API Providers` (mode `'api'`) — what changes

Keep the row list shape (`ApiRow`: rank, `ProviderLogo`, name, models count, format) and the existing four sidebar groups — but **align filters to real data** and **enrich the rail**.

**Sidebar filters (keep, realign):** Status → single clearable chip `Connected` (leaf) wired to `apiConnectedOnly` (unchanged). Capability → chips Reasoning/Tool use/Vision/Structured (align labels to §2a). Pricing → range `$ /Mtok` (unchanged units). API Format → chips `OpenAI`/`Responses`/`Anthropic`/`Gemini`/`Other` (must mirror the `ApiFormat` values `openai`/`openai_responses`/`anthropic`/`anthropic_oauth`/`gemini`).

**Detail rail (enrich):**
- Header: `ProviderLogo` + provider `name` + a `prov-format-chip` (plug icon + `format`) and Connected/Not-connected badge (unchanged).
- **Provider meta block** (new): `Env var` = `env[]` (e.g. `ANTHROPIC_API_KEY`) shown as monospace chips; `Base URL` = `api` (the `base_url`) monospace; `Docs` = `doc` as a link (`external-link` icon); optional `npm` = `npm`.
- **Models (N)** section — richer than today: each `prov-mrow` shows model `name` + mini specs `ctx` (`limit.context`) + `$in / $out` (`cost.input`/`cost.output`) + capability chips (`caps`). Same `fmtPrice` Free rule.
- **Cross-link** (new): a prominent link/row `View all models from {Provider} →` → `Bodhi Models API Catalog.html?provider=<slug>` (deep-links into page A filtered to this provider).
- Keep the "API models using this provider" deep-link rows into My Models and the `Connect Provider` / `Manage Connection` footer (unchanged).

---

## 4. Mock data shape (`bodhi-models-data.js`) — generate this so the mockup looks real

Add to `window.MODELS_DATA` a new array **`API_CATALOG_MODELS`** of ~30–40 realistic models spanning anthropic / openai / google / groq / deepseek / meta. **Use the EXACT models.dev field names below** (mirror models.dev `catalog.json` + the existing data.js conventions). Keep prices as **$ per 1M tokens** (consistent with the existing `in`/`out`), and dates as `YYYY-MM` or `YYYY-MM-DD`.

```js
// One catalog model (models.dev shape). Add ~30–40 of these.
{
  id: 'claude-sonnet-4-5',            // models.dev id (path stem)
  name: 'Claude Sonnet 4.5',          // display name
  family: 'Claude',                    // family enum-ish (OPTIONAL/sparse — see note below)
  // NOTE: base_model is an INGESTION-ONLY inheritance directive; it is resolved away in
  // the generated api.json/catalog.json and NEVER appears in the served catalog. Do not
  // carry it in the mock object.
  cost: { input: 3, output: 15, cache_read: 0.3, cache_write: 3.75 }, // PRIMARY/representative provider price, $/Mtok; omit absent keys. Authoritative per-provider prices live in providers[].{in,out}
  limit: { context: 200000, output: 64000 },   // models.dev limit is {context,output} ONLY — no input key
  modalities: { input: ['text','image','pdf'], output: ['text'] },    // enum: text|audio|image|video|pdf
  reasoning: true,
  reasoning_options: { budget_tokens: { min: 1024 } },  // optional; toggle|effort|budget_tokens
  tool_call: true,
  structured_output: true,
  attachment: true,
  temperature: true,
  open_weights: false,
  // status is OPTIONAL; only real values are 'alpha'|'beta'|'deprecated'. A stable/GA model
  // has NO status key (status: undefined) — the UI synthesizes the 'Stable' label from its
  // absence. Do NOT emit status: 'stable' (not a valid models.dev value). For stable rows omit it.
  status: undefined,                   // alpha|beta|deprecated, or omit entirely for stable
  release_date: '2025-09-29',
  last_updated: '2025-10-15',
  knowledge: '2025-03',                // knowledge cutoff
  // which seeded providers serve it (slug + this provider's price for this model):
  providers: [
    { slug: 'anthropic', base_url: 'https://api.anthropic.com/v1', in: 3,    out: 15 },
    { slug: 'openrouter', base_url: 'https://openrouter.ai/api/v1', in: 3.3,  out: 16.5 },
  ],
}
```

**Sparse-field realism (important):** `status`, `structured_output`, `reasoning_options`, `knowledge`, `temperature`, and `family` are all OPTIONAL on models.dev and frequently ABSENT. Populate each on only a SUBSET of mock rows (leave the key off otherwise) so the rail's omit-if-absent logic and the synthesized `Stable` label are actually exercised — do NOT give every row a `structured_output` or a `family`.

Coverage to seed realism: Claude (Sonnet 4.5, Opus 4, Haiku 3.5, 3.5 Sonnet); OpenAI (gpt-5, gpt-4o, gpt-4o-mini, o3, o3-mini, o4-mini); Google (gemini-2.5-pro, gemini-2.0-flash, gemini-1.5-pro); Groq-served (llama-3.3-70b-versatile, mixtral-8x7b); DeepSeek (deepseek-v3, deepseek-r1); Meta (llama-3.3-70b, llama-3.1-8b). Vary `cost`, `limit.context` (8K→1M), `modalities`, the capability booleans, `status` (omit on most rows = stable; sprinkle one `deprecated`, one `beta`, optionally one `alpha`), and `open_weights` (Meta/DeepSeek → true).

**Providers array** — extend the existing `API_PROVIDERS` to also carry the real provider fields so page B's meta block is real: add `name` (already `provider`), `env: ['ANTHROPIC_API_KEY']` (string[]), `npm` (e.g. `@anthropic-ai/sdk`), `doc` (URL), `api` (= base_url). Keep `slug`, `format`, `tags`, `models`, `modelRows`, and the derived `connected`/`apiModels`. Reuse the existing `PROV_COLORS` and add `google:'#4285F4'`, `deepseek:'#4D6BFE'`, `meta:'#0866FF'`.

Derive `providersCount` per catalog model from `providers.length`; derive the page-A↔page-B cross-references from `slug`.

---

## 5. "Configure in Bodhi" → existing `ApiModelForm` (prefill contract)

The CTA must route to the existing Create/Configure API Model page (`Create API Model.html` → `ApiModelForm`), **prefilled** from the catalog model + its primary provider. Pass via query string, e.g. `Create API Model.html?provider=<slug>&model=<modelId>`.

Prefill rules (the form already owns these fields — see `AMF_FORMATS`):
- `api_format` and `base_url` come from the **provider**, not the model. Use the provider→format/base_url presets: anthropic → `anthropic` @ `https://api.anthropic.com/v1`; openai → `openai` @ `https://api.openai.com/v1`; google → `gemini` @ `https://generativelanguage.googleapis.com/v1beta`; any OpenAI-compatible (groq/openrouter/together/deepseek) → `openai` @ that provider's `api` base_url.
- Reference the real `ApiFormat` values only: `openai`, `openai_responses`, `anthropic`, `anthropic_oauth`, `gemini`.
- Pre-select the chosen model id in the form's model selection; leave **API Key empty** (user always supplies it).
- In this mock, the CTA just needs to navigate with those query params; no need to re-implement form wiring.

---

## 6. Design constraints (non-negotiable)

- **Reuse, don't fork.** New mode inside `ModelsApp`; reuse `AppShell` (sidebar/main/rail), `ShellFilterGroup`, `ShellSearch`, `bodhi-list.jsx` (`Pagination`/`usePagination`/`useListKeyNav`/`RowLink`), `ProviderLogo`, `Tag`/`TAG_MAP`, `fmtPrice`, `SortHeaderCell`/`ColumnsMenu` patterns from Local.
- **Tokens only, no new palette.** Use lotus/saffron/indigo/teal/leaf + the `--c-<name>-bg/-bd/-text` chip/tag tokens and `fc-*`/`tag-*` classes. Do NOT invent colors (brand hexes in data.js for logos are fine; UI chrome uses tokens).
- **Parity with Local Models polish:** compact rows, sortable columns with chevrons, colorful capability chips, density/showTags behavior consistent with the other Explore pages.
- **Follow the prompt-migration house style** (`design/prompt-migration-v2.md`): match references, surface any divergence rather than redesigning, 56px header, AppShell breadcrumb `[{Bodhi,href},{Models,href},{label,current}]`.
- Add the new page to `SHELL_NAV` under the Models section so nav highlight + breadcrumb resolve.

---

## 7. File checklist (create / edit)

**Create:**
- `design/Bodhi Models API Catalog.html` — clone of `Bodhi Models API.html`; same ordered script loads; set `window.MODELS_MODE = 'api-catalog'`. (Title: `Bodhi · Explore API Models`.)

**Edit:**
- `design/models/bodhi-models-data.js` — add `API_CATALOG_MODELS` (~30–40 models, §4), extend `API_PROVIDERS` with `env`/`npm`/`doc`/`api`, add google/deepseek/meta to `PROV_COLORS`, export the new array on `MODELS_DATA`.
- `design/models/bodhi-models-app.jsx` — add `MODE_CFG['api-catalog']`, default selection + pagination unit for the new mode.
- `design/models/models-filters.jsx` — add `FILTERS['api-catalog']` (§2a); realign `FILTERS['api']` labels (§3).
- `design/models/models-rows.jsx` — add `ApiModelRow` (§2b); export it.
- `design/models/models-main.jsx` — add list-head columns + sortable cells + pagination for `api-catalog`; reuse `useListKeyNav`.
- `design/models/models-detail.jsx` — add `DetailBody`/`DetailHeader` `api-catalog` branch (§2c); enrich the `api` branch with provider meta block + cross-link (§3).
- `design/shared/shell-core.jsx` — add `SHELL_NAV` Models sub-page `{ id:'explore-api-catalog', label:'Explore · API Models', icon:'sparkles', href:'Bodhi Models API Catalog.html' }`; keep `explore-api` relabeled `Explore · API Providers`.

**Definition of done:** both pages load from their HTML entries; page A lists catalog models with working filters/search/sort/pagination and a full spec rail with a "Served by" list + Configure CTA; page B shows enriched provider rails with env/base_url/doc + a "View all models from X" cross-link; nav + breadcrumbs resolve for both; zero new color tokens; all rendered fields use the real models.dev names from §4.
