# Reference-API contract — Models · Local Models discovery (`api.getbodhi.app`)

> **⚠️ SUPERSEDED — this doc was the design intent; the API has SHIPPED and diverged.** The source
> of truth is now the deployed API's own docs: `…/api-getbodhi-app/docs/functional/`
> (`README.md`, `endpoints.md`, `recipes.md`, `openapi.json`, `BODHIAPP-INTEGRATION-KICKOFF.md`) and
> the published types `@bodhiapp/reference-api-types`. Key shipped deltas vs. the design below
> (verified live 2026-06-21):
> - **Only two endpoints:** `GET /api/v1/models` and `GET /api/v1/models/{source}/{namespace}/{repo}`.
>   **No `/api/v1/taxonomy`, no `/api/v1/orgs`** — the UI hardcodes facet sets and uses a free-text
>   Publisher input.
> - **List query param `namespace` → renamed `author`** (path + DTO field stay `namespace`).
> - **Dropped filters** (returned for display, not filterable): `capability`, `quant_bits`,
>   `quant_method`, `size_min_bytes`/`size_max_bytes`, `max_context`/`min_context`, `curated`. The
>   UI omits those facets. Surviving facets: `q`, `author`, `specialisation`, `pipeline_tag`, `tag`,
>   `language`, `license`, `sort` (incl. `trending`).
> - **`total_estimate` is always null** → "Showing N", never a total. `params_b` is a **string**.
>   Sizes are **bytes**.
> - **List rows have null `max_quant_size_bytes`/`total_size_bytes`/`context_max`/`architecture`** —
>   populated only on the single-model detail. `quant_count`/`quant_bits`/`quant_methods` are on rows.
> - **No README** is surfaced (`?include=readme` is a no-op; not in the schema).
> - **`quants[]` carry a real `filename`** (the actual `.gguf` to pull); **split multi-part GGUFs are
>   NOT supported** (sharded files excluded — each quant is one pullable file). Added in
>   `@bodhiapp/reference-api-types` 0.0.3.
>
> The original design intent is kept below for history.
>
> ~~The HTTP contract the BodhiApp frontend codes against for the **Local Models discovery** screen
> (Batch 3-6) and reuses for **API Models discovery** (3-7). The production service is a **new
> Cloudflare repo**; see `batch-3-6-cloudflare-reference-api-kickoff.md`.~~
>
> `/Users/amir36/Documents/workspace/src/github.com/BodhiSearch/spike-api-getbodhi-app` is a
> **throw-away spike** — consult it only to confirm what upstream data is available (HuggingFace
> endpoints/fields, quant-filename realities). **Do not copy its API schema, endpoint shapes, or
> architecture.** This contract supersedes anything there.

## 0. Stance — an open HuggingFace proxy (enrich + trim)

For v1 the only model `type` is **GGUF** and the only `source` is **HuggingFace**, so this API is, in
effect, **a proxy in front of the HuggingFace API** — enriching it with the fields the discovery UI
needs (parsed quant table, derived capabilities/specialisation, normalized facets) and trimming the
parts the UI doesn't. We are **open about this**, not hiding it: keeping the response close to HF's
own shape (and additive `?include=` expansion, see §4) means that when we later surface more HF detail
the change is small and stays convention-compliant.

`type` and `source` are **orthogonal axes**, and conflating them is the mistake to avoid:
- **`type`** = the *kind/format* of model the client wants. v1: `gguf`. Future: `safetensor`, `mlx`,
  `api`. (We will fetch e.g. *safetensor* models *from HuggingFace* too — so "gguf" ≠ "huggingface".)
- **`source`** = the *registry/provenance* the data came from. v1: `huggingface`. Future: other
  registries (e.g. GitHub now hosts model repos).

## 1. Conventions

- **Base URL**: configurable; the frontend reads it from `AppInfo.reference_api_url` (default
  `https://api.getbodhi.app/`). Tests point it at a mock origin via the app-info mock.
- **All endpoints are prefixed `/api/`**, versioned under `/api/v1`.
- **Content type**: `application/json`, UTF-8.
- **Identity**: a model is keyed `(source, namespace, repo)`. `namespace` = the HF org/user.
- **v1 scope guard**: `type=gguf` and `source=huggingface` are the only supported values. A request
  for any other `source` (or `type`) returns a **422** (`unsupported_source` / `unsupported_type`) —
  the surface is forward-shaped but only HF/GGUF is wired.
- **Numeric precision**: fields needing exact decimals (`params_b`, any `*_usd_mtok`) serialize as
  **strings**; counts (`downloads`, `likes`, `context_max`, byte sizes) as numbers.
- **Timestamps**: ISO 8601 UTC strings, nullable where upstream may omit.

## 2. Auth & CORS

- **Auth**: the frontend sends the user's Keycloak **OIDC `id_token`** as `Authorization: Bearer
  <id_token>` (the BodhiApp client scaffold already does this — `/Users/amir36/Documents/workspace/src/github.com/BodhiSearch/BodhiApp/crates/bodhi/src/lib/referenceApiClient.ts`).
  The server validates signature (RS256) against the Keycloak JWKS + `iss` + `exp`.
  - Issuer (prod) `https://id.getbodhi.app/realms/bodhi`; (staging) `https://main-id.getbodhi.app/realms/bodhi`.
  - JWKS `{issuer}/protocol/openid-connect/certs`.
  - **`aud`** validated only when an expected-audience binding is configured (the API is not yet a
    registered Keycloak client) — until then unchecked; a documented, accepted gap.
  - **Posture**: catalog reads are **public read-through** (missing/anonymous Bearer still serves
    data, so the catalog is browsable pre-login); a **present-but-invalid** token ⇒ `401`. The `sub`
    claim is extracted for future use.
- **No rate limiting in v1** (future requirement — do not implement).
- **CORS**: **allow all origins (`*`)**. BodhiApp is installed anywhere (Tauri desktop, self-hosted
  Docker on any domain, PWA), so there is no fixed origin allowlist. Methods `GET, POST, OPTIONS`;
  allow `authorization, content-type`.

## 3. `GET /api/v1/models` — catalog list (filter + search + sort)

The list/search endpoint. **All query params optional except `limit`.** Because everything is in the
query string, a response is cacheable by normalizing (sorting) the params into the cache key.

### Query params

| Param | Type | Notes |
|---|---|---|
| `type` | `string` | Model kind. v1: `gguf` (only). Other values ⇒ 422. |
| `source` | `string` | Registry. v1: `huggingface` (only). Other values ⇒ 422. |
| `q` | `string` (1–200) | Full-text + fuzzy search over name/tags/license. When set, cursor pagination is disabled (`next_cursor` null); raise `limit` for more. |
| `namespace` | `string` (repeatable) | Publisher/org (HF org/user), e.g. `Qwen`. **Free-text** (not an enum) — accepts any org. OR across values. Powers the clickable-org filter + "More from <org>". Org suggestions come from `GET /api/v1/orgs`. |
| `curated` | `boolean` | When `true`, restrict to editorially curated / "Staff Picks" models. Drives the Staff-Picks browse chip. |
| `pipeline_tag` | `string` (repeatable) | e.g. `text-generation`. OR across values. |
| `library` | `string` (repeatable) | e.g. `gguf`. OR. |
| `license` | `string` (repeatable) | OR. Values from `/api/v1/taxonomy`. |
| `language` | `string` (repeatable) | Overlap — matches if it shares ANY. |
| `tag` | `string` (repeatable) | Containment — must have ALL. |
| `capability` | `string` (repeatable) | Containment (AND). Enum: `tool-use \| vision \| reasoning \| structured \| embedding`. |
| `specialisation` | `string` (repeatable) | Containment (AND). Enum: `coding \| reasoning \| long-ctx \| vision \| small`. (`agentic` reserved — classifier, later.) |
| `quant_bits` | `number` (repeatable) | **Bit-width** of the quant — the number in the GGUF label. Values `2 \| 3 \| 4 \| 5 \| 6 \| 8 \| 16`. OR across values. Parsed from the filename (`Q4_K_M`→4, `Q8_0`→8, `F16`/`BF16`→16). Enumerated by `/api/v1/taxonomy`. |
| `quant_method` | `string` (repeatable) | **Quant method / mix variant** — the suffix part, *orthogonal* to bit-width. Values e.g. `K \| K_M \| K_S \| K_L \| 0 \| 1 \| XS \| XXS \| NL \| IQ \| F16 \| BF16 \| F32`. OR. Parsed from the filename (`Q4_K_M`→`K_M`, `Q8_0`→`0`, `IQ4_XS`→`IQ`+`XS`). Enumerated by `/api/v1/taxonomy`. |
| `size_min_bytes` / `size_max_bytes` | `number` | **Double-sided size range** over `total_size_bytes` (`>= size_min`, `<= size_max`). Replaces the old fixed buckets — the UI renders a **two-thumb GB slider**. Rows with `total_size_bytes = null` never match (size is populated when a repo's files are first read). |
| `max_context` | `number` | `context_max <= max_context`. The UI renders a **max-only** context slider; `min_context` also accepted if ever needed. |
| `min_context` | `number` | `context_max >= min_context`. (Accepted but the v1 UI uses max-only.) |
| `max_input_usd_mtok` | `number` | API-model pricing filter (3-7); n/a for GGUF. |
| `sort` | `string` | One of `downloads \| likes \| last_modified \| trending \| created_at`. Default `downloads`. ("New" = `created_at`; "Trending" = `trending`.) **No benchmark/score sort** (no source). **Impl note:** HF's list API does **not** accept `sort=trending`/`sort=trendingScore` (400) — it only exposes `trendingScore` as a *field*. So the reference API **ingests `trending_score` into its own store and sorts on that column itself**; it does not delegate trending-sort to HF. |
| `order` | `string` | `asc \| desc`. Default `desc`. |
| `cursor` | `string` | Opaque, from a prior `next_cursor`; filters + sort must match across calls. |
| `limit` | `number` (1–200) | Default 50. |

> **Quant: two orthogonal axes (do not merge).** A GGUF filename token like `Q4_K_M` encodes **bit-width**
> (`4`) *and* a **method/mix** (`K_M`) — different concepts that the filename happens to concatenate.
> The UI exposes them as **two separate filters** (`quant_bits` + `quant_method`) so users don't conflate
> "how many bits" with "which quant method". Both are parsed per file and projected (see §5).
>
> **Range filters, not buckets.** **Size** is a **two-thumb GB range** (`size_min_bytes` /
> `size_max_bytes`) — the old small/medium/large buckets are gone. **Context** is a **single max** slider
> (`max_context`). For both sliders the client should derive the **min/max bounds + step** from the
> current result set (e.g. from the largest `total_size_bytes` / `context_max` returned), not hardcode
> them — so the increments adapt to what's actually in the catalog.
>
> The Specialisation facet's **`agentic`** chip has no v1 backing (classifier deferred) and is **dropped
> from the v1 UI**; the rest map to the `specialisation` enum. The prototype's "Score"/HUMAN-eval
> column is **dropped** (no source) — rows sort by **downloads / likes** (+ recent / trending / new).

### Response 200

```jsonc
{
  "items": [ /* Model[] — see §5 */ ],
  "next_cursor": "…|null",   // opaque keyset cursor; null = last page (or q set)
  "total_estimate": null      // reserved; may stay null
}
```

### How the list row is composed (so a list response renders a full row with no per-row fetch)

The discovery row shows `<org>/<repo>`, a meta line, and tags; everything comes from the **list
response** item:
- **meta line** `"<N> quants · up to <size> · <license>"` → `N = quant_count`, `<size>` formatted
  from **`max_quant_size_bytes`**, `<license>` from `license`. (`quant_count` + `max_quant_size_bytes`
  are on the list item so the row renders without a detail call.)
- **rank `#1..#N`** is the **client-side list ordinal** (position in the sorted, paginated result),
  **not** a server field. It re-numbers per page/sort; the API does not assign a global rank.
- The list does **not** include the per-quant `quants[]` table — that's only on the single-model
  response (§4); the row only needs the count + max size, both present above.

## 4. `GET /api/v1/models/{source}/{namespace}/{repo}` — single model

One model by its full key. `source` is in the **path** here (we know the provenance of a specific
repo). v1: `source` must be `huggingface` (else 422). Path segments are URL-decoded server-side;
callers may `encodeURIComponent` a `repo` containing a slash. A cold/unknown repo triggers a live HF
fetch; a repo with no GGUF artifact ⇒ `404`.

Returns the **Model DTO (§5)** plus, for GGUF, the parsed **`quants`** array:

```jsonc
{
  /* …all Model fields (§5)… */
  "quants": [
    { "name": "Q4_K_M", "size": 18253611008, "bits": 4, "method": "K_M", "recommended": true },
    { "name": "Q8_0",   "size": 34489280512, "bits": 8, "method": "0",   "recommended": false }
  ]
}
```

- `quants[]` is derived from the repo's file listing: each GGUF artifact → `{ name, size, bits?,
  method?, recommended? }`, deduped by name (keeping the max size for split/multi-file quants). Sizes
  in bytes.
  - `bits` — the quant's **bit-width** parsed from the label (`Q4_K_M`→4, `Q8_0`→8, `F16`/`BF16`→16);
    `null` if not derivable. (One of the two orthogonal quant axes — see §3.)
  - `method` — the quant **method/mix variant** (`Q4_K_M`→`K_M`, `Q8_0`→`0`, `IQ4_XS`→`XS` on the
    I-quant family, `F16`→`F16`); `null` if not derivable. The *other* axis.
  - `recommended` — marks the suggested default quant (the "Pull best quant" target). Heuristic
    (e.g. a balanced `Q4_K_M`); at most one per repo.
  - **No per-quant `capabilities`.** Capabilities are a **repo-level** property (`capabilities[]` in
    §5) — quantization changes weight precision, not what the model can do. (Rare edge case: a
    *text-only* GGUF of a multimodal repo genuinely drops a modality; **out of v1 scope**.)
  - **No hardware-fit field.** Whether a quant fits the user's GPU/RAM is **BodhiApp-local** (computed
    from host RAM/VRAM + `size`), not a catalog property — never in this contract.
  - The agent should **research current GGUF filename conventions** (quant labels like `Q*_K_M`, `IQ*`,
    `F16/BF16`, split-file suffixes) and derive the parser — do **not** assume the spike's regex is
    correct or complete; confirm against real HF repos.

### 4.1 Additive expansion via `?include=` (GraphQL-ish)

Extra detail is requested **additively** on the same endpoint (no separate sub-paths), so the
response is the same model object **plus** the requested sections. Repeatable / comma-separated.

| `include` value | Adds | Applicability |
|---|---|---|
| `files` | `files: [ { path, type, size, lfs_oid } ]` — the full repo file tree | HF only |
| `readme` | `readme: { markdown, fetched_at }` — the model card markdown | HF only |

```jsonc
// GET /api/v1/models/huggingface/Qwen/Qwen3-Coder-32B?include=files,readme
{
  /* …Model + quants… */,
  "files":  [ { "path": "…-Q4_K_M.gguf", "type": "file", "size": 18253611008, "lfs_oid": "…|null" } ],
  "readme": { "markdown": "…", "fetched_at": "…" }
}
```

> README/files organization takes its cue from HuggingFace's own API (HF serves the card and the file
> tree as sub-resources of the model). We fold them into `?include=` so the discovery UI fetches the
> model once and asks for exactly the sections it needs. The agent may refine the `include` vocabulary
> after reviewing HF's current API organization.

## 5. Model DTO

Shared by `/models` list items and the single-model response (which adds `quants`, and `files`/
`readme` when `?include=`'d). All fields nullable unless marked non-null.

| Field | Type | Notes |
|---|---|---|
| `source` | `string` | non-null. v1 always `huggingface`. **Provenance, returned in the response.** |
| `type` | `string` | non-null. v1 always `gguf`. The model kind. |
| `namespace` | `string` | non-null. HF org/user. |
| `repo` | `string` | non-null. |
| `pipeline_tag` | `string\|null` | e.g. `text-generation`. |
| `domain` | `string\|null` | optional normalized model domain for a clean chip (e.g. `llm` / `embedding` / `vision`), derived from `pipeline_tag`. May be added later. |
| `library` | `string\|null` | e.g. `gguf`. |
| `license` | `string\|null` | e.g. `apache-2.0`. |
| `languages` | `string[]\|null` | |
| `tags` | `string[]\|null` | raw HF tags. |
| `capabilities` | `string[]\|null` | `tool-use\|vision\|reasoning\|structured\|embedding` (derived). |
| `specialisation` | `string[]\|null` | `coding\|reasoning\|long-ctx\|vision\|small` (derived). |
| `quant_count` | `number\|null` | number of distinct GGUF quant variants in the repo — the "N quants" in the row meta. |
| `quant_bits` | `number[]\|null` | distinct **bit-widths** present in the repo (e.g. `[4, 8]`) — powers the bit-width facet + chips. |
| `quant_methods` | `string[]\|null` | distinct **quant methods/variants** present (e.g. `["K_M", "0"]`) — powers the method facet + chips. (Orthogonal to `quant_bits`; both parsed from filenames.) |
| `max_quant_size_bytes` | `number\|null` | size of the **largest** GGUF quant in the repo, in bytes. **Returned on the list response** so the row can render the "up to <size>" meta string without a per-row detail fetch. |
| `total_size_bytes` | `number\|null` | total size of all GGUF artifacts, in bytes. Powers the **size range** filter (`size_min_bytes`/`size_max_bytes`). |
| `architecture` | `string\|null` | model architecture for the detail "specs" row (e.g. `Qwen3-MoE`, `Llama 3.3`), from HF `gguf.architecture` / card data. |
| `context_max` | `number\|null` | tokens. From HF `gguf.context_length` (verified). |
| `params_b` | `string\|null` | billions; string for precision. **Not in HF's `gguf` object** (which only has `gguf.total` = *bytes*) — derive from `safetensors.parameters.<dtype>` when present, else a repo-name regex (`…-32B…`→32). May be null for some GGUF-only repos. |
| `input_usd_mtok` / `output_usd_mtok` / `cached_in_usd_mtok` | `string\|null` | API-model pricing (3-7); null for GGUF. |
| `provider` | `string\|null` | |
| `downloads` | `number\|null` | HF lifetime. |
| `likes` | `number\|null` | |
| `trending_score` | `number\|null` | HF `trendingScore` (a real field on HF model objects). Ingested + stored so the ref API can `sort=trending` on its own column — HF's API exposes the field but **rejects it as a sort key**. |
| `created_at` | `string\|null` | repo creation time, from HF `createdAt` (powers `sort=created_at` = "New"). |
| `last_modified` | `string\|null` | HF `lastModified`. |
| `curated` | `boolean` | non-null. Editorially curated / "Staff Picks". **Our own data — no HF signal.** Drives the Staff-Picks chip + `curated=true` filter. Default `false`. |
| `owner_verified` | `boolean` | non-null. Publisher/org is trusted/verified — renders a ✓ badge. **Not from HF** (HF exposes no verification flag via API) → **self-curated org allow-list** (e.g. `meta-llama`, `Qwen`, `mistralai`, `google`, `bartowski`, …). Default `false`. |
| `fetched_at` | `string` | non-null; when this API last refreshed the row. |

> **No `score`/HUMAN/Arena field.** No upstream provides a benchmark score; the UI sorts by
> `downloads` and omits the score column. A benchmark-score workstream is future work.
>
> **No hardware-fit field.** Whether a model/quant runs on the user's machine is **BodhiApp-local**
> (computed from host RAM/VRAM), never a catalog property — it is intentionally absent here.

## 6. `GET /api/v1/taxonomy` — facet enum values

Returns the allowed values for the facet chips so the UI never hardcodes them:

```jsonc
{
  "source":         [ { "id": "huggingface", "label": "Hugging Face" } ],
  "type":           [ { "id": "gguf", "label": "GGUF" } ],
  "pipeline_tag":   [ { "id": "text-generation" }, … ],
  "domain":         [ { "id": "llm" }, { "id": "embedding" }, … ],   // optional, when projected
  "library":        [ { "id": "gguf", "label": "GGUF" }, … ],
  "license":        [ { "id": "apache-2.0" }, … ],
  "language":       [ … ],
  "capability":     [ { "id": "tool-use" }, … ],
  "specialisation": [ { "id": "coding" }, … ],
  "quant_bits":     [ { "id": "2" }, { "id": "4" }, { "id": "8" }, … ],   // bit-width axis
  "quant_method":   [ { "id": "K_M" }, { "id": "0" }, { "id": "XS" }, … ] // method/variant axis
}
```

**Size and context are range sliders, not enums** — they're not listed here; the client derives each
slider's min/max bound + step from the result set's `total_size_bytes` / `context_max`. **Publishers/
orgs are NOT enumerated here** either — there are too many and the filter accepts free text; use
`/api/v1/orgs` (§7) for autocomplete suggestions. Everything else above enumerates here.

## 7. `GET /api/v1/orgs` — publisher autocomplete

Populates the **org/publisher filter** (a first-class `namespace` filter on `/api/v1/models`). The UI
filter is an **autocomplete that also accepts free text**, so this endpoint *suggests* orgs but the
`namespace` filter is not limited to them.

### Query params

| Param | Type | Notes |
|---|---|---|
| `q` | `string` | Prefix/substring match over org names. Omitted ⇒ top orgs (by model count). |
| `type` / `source` | `string` | v1: `gguf` / `huggingface`. Scopes the org list to the catalog slice. |
| `limit` | `number` (1–100) | Default 20. |

### Response 200

```jsonc
{
  "items": [
    { "id": "Qwen",       "label": "Qwen",       "model_count": 142, "verified": true },
    { "id": "meta-llama", "label": "meta-llama", "model_count":  88, "verified": true }
  ]
}
```

- `id` = the `namespace` value to pass to `/api/v1/models?namespace=<id>`.
- `verified` mirrors the org's `owner_verified` trust signal.
- `model_count` = number of catalog models by that org (for ranking/labeling).

## 8. Errors

| Status | Body | When |
|---|---|---|
| 400 | `{ "error": "validation", … }` | request schema fails |
| 401 | `{ "error": "unauthorized", "message": "…" }` | Bearer present but invalid |
| 404 | `{ "error": "not_found" }` | model doesn't exist / has no GGUF artifact |
| 422 | `{ "error": "unsupported_source" \| "unsupported_type" }` | a `source`/`type` other than `huggingface`/`gguf` (v1 scope guard) |
| 500 | `{ "error": "internal" }` | unhandled |

## 9. What BodhiApp's MSW stub mirrors (Phase 2)

For Phase-2 self-sufficiency the BodhiApp tests stub the external origin (from app-info) for:
`GET /api/v1/models` (list), `GET /api/v1/models/huggingface/{ns}/{repo}` (incl. `quants`, and
`files`/`readme` under `?include=`), `GET /api/v1/taxonomy`, and `GET /api/v1/orgs` (publisher
autocomplete) — asserting the `Authorization: Bearer <id_token>` header is present. Shapes per §3–§7.
If the contract shifts during implementation, the stub follows the contract (which leads), not the
other way round.
