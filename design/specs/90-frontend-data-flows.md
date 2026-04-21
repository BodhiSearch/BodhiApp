# 90 · Frontend data flows
*Per-page composition: which endpoints each screen calls, in what order, and how responses are joined.*

This doc is the "reading order" for a frontend engineer implementing a screen. For each page, it lists the endpoints, suggests React Query `queryKey`s, and spells out the client-side join.

Endpoint labels:
- `BS:` → bodhi-server (see `10-bodhi-server-apis.md`)
- `GB:` → api.getbodhi.app (see `20-getbodhi-public-apis.md`)
- `HF:` → huggingface.co (see `30-huggingface-integration.md`)

## 1. Models page — `My Models` mode
**Queries on mount:**
| queryKey | Endpoint | Purpose |
|---|---|---|
| `['models', 'all']` | `BS: GET /bodhi/v1/models?page_size=100` | Fetches aliases, files, api-models (all three kinds) |
| `['specializations']` | `GB: GET /v1/specializations` | Drives sidebar |
| `['licenses']` | `GB: GET /v1/licenses` | Drives License filter |

**Join:**
1. Split `BS: /models` response by `source`:
   - `source=user` → `alias` rows
   - `source=model` → `file` rows
   - `source=api` → `api-model` rows
2. Derive `provider-connected` rows by grouping `source=api` entries by `(api_format, base_url)`. One row per unique tuple.
3. Apply client-side Kind/Capability/License/Format/Size filters.
4. Render with the row shapes from `shared-primitives.md §3`.

**Performance budget:** one parallel fetch group. p50 target < 150 ms for the list.

## 2. Models page — `All Models` mode
Adds two more sources to the My Models flow.

**Additional queries on mount (parallel with `BS: /models`):**
| queryKey | Endpoint | Purpose |
|---|---|---|
| `['providers']` | `GB: GET /v1/providers` | `provider-unconnected` rows |
| `['trending', window]` | `GB: GET /v1/repos/trending?window=7d` | Default HF-repo rows (unless user selects a different source) |

**Join:**
1. Start with the My Models projection (§1).
2. Append `provider-unconnected` rows from `GB: /providers`, **excluding** providers already present as `provider-connected` (match by `provider_code`). See `00-architecture.md §6` "conflict resolution".
3. Append `hf-repo` rows from `GB: /repos/trending` (or equivalent based on sort/source filter).
4. Tag `source=user/model/api` rows with `localBadge: true` (saffron pill).
5. Annotate `hf-repo` rows with `catalogAliases: { count: N }` when any alias/file matches the repo.

## 3. Models page — Ranked mode (Specialization selected)
Triggered when user picks any specialization ≠ `all`. Causes a shape-change of the main list.

**Additional queries:**
| queryKey | Endpoint | Purpose |
|---|---|---|
| `['leaderboard', benchmark]` | `GB: GET /v1/leaderboards/{benchmark}?page_size=200` | Ranked data |

Where `benchmark` comes from `SPECIALIZATIONS.find(s => s.code === spec).benchmark_key`.

**Join (see `shared-primitives.md §3` for row shapes):**
For each `LeaderboardEntry`:
- If `kind=api`: find all `ApiAliasResponse`s from `BS: /models` where the entry's `provider_model` is in `alias.models`. Stack them as `primaries` in the ranked row; identity = `provider-connected` from `GB: /providers/{provider_code}` (`name (provider — connected)`). If no local match, identity is `provider (from api.getbodhi.app)` (unconnected).
- If `kind=local-gguf`: check `BS: /models` for an alias with matching `repo` + `filename`. If found, stack alias(es); identity = `owner/repo:quant (modelfile)`. If repo+filename is on disk but no alias → orphan row with `+ Create alias` CTA. If not on disk, check `GB: /repos/{owner}/{name}` for quant list; render as `pull →` CTA.

**Rank numbers are preserved as-is from the leaderboard response** — the frontend never renumbers.

## 4. Models page — search / filter interactions
### Client-side filter (kind / capability / license / size / cost)
No new network calls. `useMemo` over the joined set.

### Live search (the `⌘K` filter bar)
**On user input:**
- If input length < 3: suggest recent items from local state.
- If input length ≥ 3:
  | queryKey | Endpoint | Purpose |
  |---|---|---|
  | `['search', 'mirror', q]` | `GB: GET /v1/repos/search?q=...` | Curated + enriched hits |
  | `['search', 'hf', q]` (opt-in) | `HF: GET /api/models?search=...&library=gguf&limit=20` | Long-tail |

Default is `mirror` only. A "include long-tail" toggle enables the HF fallback. Results are merged, de-duplicated by repo ID, sorted by source-local score.

## 5. `hf-repo` detail panel (HfRepoPanel)
Opens when user clicks an `hf-repo` row or ranked entry with `kind=local-gguf`.

**Query:**
| queryKey | Endpoint |
|---|---|
| `['repo', owner, name]` | `GB: GET /v1/repos/{owner}/{name}` |

**Fallback if 404 or stale:** `HF: GET /api/models/{owner}/{name}` + `HF: GET /api/models/{owner}/{name}/tree/main`. Frontend computes quant list from siblings client-side.

**Additional joins for the panel body:**
- For each quant, check `BS: /models` for a matching alias — display inline as `✓ N local aliases`.
- For the `pull →` button, check `BS: /models/files/pull` for an existing download request (show progress if present).

## 6. `provider-unconnected` detail panel (UnconnectedProviderPanel)
Opens when user clicks a `provider-unconnected` row.

**Query:**
| queryKey | Endpoint |
|---|---|
| `['provider', code]` | `GB: GET /v1/providers/{code}` |

**Body:**
- Pricing table: `ProviderDetail.models[].cost`
- Capability matrix: from `ProviderDetail.models[].capabilities`
- "Connect" CTA → routes to `Create API model` overlay, prefills `api_format` / `default_base_url` from the detail.

## 7. `provider-connected` / `api-model` detail panel (ConnectedProviderPanel)
Opens when user clicks a `provider-connected` row or an `api-model` row.

**Queries (on-demand):**
| queryKey | Endpoint |
|---|---|
| `['api-model', id]` | `BS: GET /bodhi/v1/models/api/{id}` (for the specific api-model) |
| `['provider', code]` | `GB: GET /v1/providers/{code}` (for pricing/capability enrichment) |
| `['api-model-fetch', id]` | `BS: POST /bodhi/v1/models/api/{id}/sync-models` (manually triggered by the user) |

**Body:**
- Model table: union of `ApiAliasResponse.models` (user's selected set) + `ProviderDetail.models` (full catalog) — flag overlaps, let user add more.
- Cost per model: from `DirectoryModel.cost` (or masked as `—` if the directory doesn't have it).
- Status chip: from bodhi-server health check (future endpoint) or derived from last successful `test`.

## 8. Create local alias flow
### Initial load
| queryKey | Endpoint | Purpose |
|---|---|---|
| `['files']` | `BS: GET /bodhi/v1/models/files?page_size=100` | For QuantPicker reuse (already-downloaded quants) |
| `['repo', owner, name]` | `GB: GET /v1/repos/{owner}/{name}` | When arrived with a repo preselected (overlay path from Models page) |
| `['llama-args']` | `BS: GET /bodhi/v1/models/alias/llama-args` **(NEW — see §5 of bodhi spec)** | For ArgsPalette; can fall back to static fixture |
| `['presets']` | none — `PRESET_CATALOGUE` is a frontend constant | |

### On quant selection
If selected quant is not yet downloaded: `BS: POST /bodhi/v1/models/files/pull` fires immediately, the DownloadProgressStrip opens, and the alias can still be created in parallel (`BS: POST /bodhi/v1/models/alias`). The download finishes async in the background.

### On fit check (optional FitCheckCard)
`BS: POST /bodhi/v1/models/alias/fit-check` **(NEW — see `10-bodhi-server-apis.md §5`)**. Non-blocking; card can show a static hint if the endpoint is missing.

### On save
`BS: POST /bodhi/v1/models/alias` with `UserAliasRequest`. 201 redirects to the Models page with the new alias selected.

## 9. Create API model flow
### Initial load
| queryKey | Endpoint | Purpose |
|---|---|---|
| `['api-formats']` | `GB: GET /v1/api-formats` (preferred) or `BS: GET /bodhi/v1/models/api/formats` | Populate ApiFormatPicker |
| `['providers']` | `GB: GET /v1/providers` | For "pre-select provider from Models directory" path |

### On user entering creds → "Fetch Models"
`BS: POST /bodhi/v1/models/api/fetch-models` with `{ base_url, api_format, creds: { type: api_key, value: ... } }`. Renders the returned model IDs in ModelMultiSelect's Available list.

### Cross-enrichment
When `forwarding_mode=selected`, the UI shows a cost hint per model if `GB: /providers/{code}` has them. This is a nice-to-have overlay; not required for save.

### On "Test connection"
`BS: POST /bodhi/v1/models/api/test` with `{ base_url, api_format, model, prompt: "ping" }`. Shows success/failure toast.

### On save
`BS: POST /bodhi/v1/models/api` with `ApiModelRequest`.

## 10. Downloads panel
Opens from the sidebar ↓ Downloads menu.

| queryKey | Endpoint | Refresh |
|---|---|---|
| `['downloads']` | `BS: GET /bodhi/v1/models/files/pull?page_size=50` | poll 1 s while any `status=downloading`; poll 30 s otherwise |
| `['download', id]` | `BS: GET /bodhi/v1/models/files/pull/{id}` | poll 500 ms while active (or subscribe to SSE if `10-bodhi-server-apis.md §3 CHANGE` lands) |

## 11. Add-model menu routing (from Models page header)
| Menu item | Action | Endpoints involved |
|---|---|---|
| Add by HF repo | Opens Create-alias overlay | `GB: /repos/search` (autocomplete) → overlay flow |
| Paste URL | Opens Create-alias overlay with prefilled `hf://...` or `.gguf` URL | Same |
| Add API provider | Opens Create-API-Model overlay | See §9 |
| Add API model (from connected) | Opens Create-API-Model overlay with `provider_code` preset | `BS: /models` to list connected → `GB: /providers/{code}` for model list |
| ↑ Trending | Navigates to Models page with preset `sort=trending` | `GB: /repos/trending` |
| ★ New launches | Same | `GB: /repos/new` |
| 🏆 Leaderboards › | Navigates to Models page with preset `specialization=coding` (default) | Enters ranked mode (§3) |

## 12. Cache invalidation rules
| Mutation | Invalidates |
|---|---|
| `BS: POST /models/alias` (create) | `['models', 'all']`, `['files']` (if the pull was also triggered) |
| `BS: PUT /models/alias/{id}` | `['models', 'all']`, `['alias', id]` |
| `BS: DELETE /models/alias/{id}` | `['models', 'all']`, `['alias', id]` |
| `BS: POST /models/api` | `['models', 'all']` |
| `BS: POST /models/files/pull` | `['downloads']`, `['files']` (on completion) |
| `BS: POST /models/refresh` | `['models', 'all']` (after queue drains) |

api.getbodhi.app and HF caches are immutable from the frontend's perspective — no invalidation triggers.

## 13. Error matrix
| Source | Failure | UX |
|---|---|---|
| BS unreachable | hard — full-page error, retry | no partial rendering |
| GB unreachable | soft — page renders without directory / leaderboards; banner explains | `My Models` still fully functional |
| HF 429 | soft — fall back to GB mirror | stale chip on HF repo rows |
| HF 404 on a mirrored repo | soft — render from GB mirror | mark as `archived on HF` |
| GB 404 on a repo | fall through to HF live | (no banner) |

## 14. Query key conventions
Use tuples for namespacing. Examples:
- `['models', 'all']`, `['models', 'byId', id]`
- `['models', 'api', id]`, `['models', 'api', 'fetch-models', base_url, api_format]`
- `['providers']`, `['provider', code]`
- `['leaderboard', benchmark]`
- `['repo', owner, name]`, `['repos', 'trending', window]`, `['repos', 'search', q, filters]`
- `['downloads']`, `['download', id]`
- `['llama-args']`, `['api-formats']`, `['specializations']`, `['licenses']`

## 15. SWR / React Query configuration suggestions
| Class | staleTime | gcTime |
|---|---|---|
| Local user data (`BS: /models`, `/files`, `/downloads`) | 5 s | 5 min |
| Public catalog (`GB: /providers`, `/repos/*`, `/leaderboards`) | 1 h | 24 h |
| Enum / taxonomy (`/specializations`, `/licenses`, `/benchmarks`, `/api-formats`) | 24 h | 7 d |
| HF direct | 5 min | 1 h |

Refetch on window focus: only for `local user data` class. Public catalogs rely on their long staleTime.
