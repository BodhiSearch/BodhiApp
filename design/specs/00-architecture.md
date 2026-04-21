# 00 · Architecture — three data sources
*Read first. Defines which backend owns what, and how the frontend joins data across them.*

## 1. The three sources
The Models page displays **six entity kinds** (see `design/models-page/specs/shared-primitives.md §2`). Those entities live across three backends:

| Source | URL | Owns |
|---|---|---|
| **bodhi-server** | `http://<host>/bodhi/v1/*` | user's own data: `alias`, `file`, `api-model`, `provider-connected` — everything in the tenant DB + local disk |
| **api.getbodhi.app** | `https://api.getbodhi.app/v1/*` | public, non-user-specific catalog: `provider-unconnected` (Bodhi Directory), supported AI providers (`API_FORMATS`), curated leaderboards + benchmark scores, specialization taxonomy, trending/new-launch cuts, capability/license/cost normalization |
| **huggingface.co** | `https://huggingface.co/api/*` | `hf-repo` (the entire HF catalog), repo siblings, GGUF metadata, downloads/likes counters |

The frontend calls all three directly. Bodhi-server does **not** proxy api.getbodhi.app or HF.

### Why not proxy through bodhi-server
Three reasons, in order of weight:
1. **Public data has no tenant scope.** Proxying forces every frontend to round-trip through the user's local server just to read a public provider list. Wastes latency + local CPU.
2. **Provenance is load-bearing.** The UI *shows* `from api.getbodhi.app` on unconnected providers. Obscuring that provenance behind a proxy would force extra attribution metadata in the proxy response, adding complexity for no gain.
3. **Cache locality.** HF and api.getbodhi.app both have CDN cache. A local server proxy would double-cache with no hit-rate benefit.

The only case where bodhi-server *must* talk to an external source is **downloading GGUF files** (existing `HubService.download()` path), because that side-effects the local disk. This stays as-is.

## 2. Entity ↔ source mapping
| `kind` | Source | Where in this spec folder |
|---|---|---|
| `alias` | bodhi-server | `10-bodhi-server-apis.md §2` |
| `file` | bodhi-server | `10-bodhi-server-apis.md §3` |
| `api-model` | bodhi-server | `10-bodhi-server-apis.md §4` |
| `provider-connected` | bodhi-server (derived from `api-model` rows) | `10-bodhi-server-apis.md §4` |
| `provider-unconnected` | api.getbodhi.app | `20-getbodhi-public-apis.md §2` |
| `hf-repo` | huggingface.co (direct) + api.getbodhi.app (enriched metadata) | `30-huggingface-integration.md` |

## 3. Mode toggle ↔ source mapping
| Mode | Rows | Calls |
|---|---|---|
| `My Models` | `alias` · `file` · `api-model` · `provider-connected` | bodhi-server only |
| `All Models` | all six kinds | bodhi-server + api.getbodhi.app + HF |

This split keeps `My Models` fast (single local API round trip) and defers the multi-source fan-out to `All Models`.

## 4. Ranked mode data flow
When a Specialization is selected, ranked rows are **model-level** (not entity-level — see `models.md §8`). Source-of-truth:
- **Ranking / score / benchmark** → `api.getbodhi.app /v1/leaderboards/{benchmark}` (curated leaderboard data).
- **Local backing** (is this alias on my disk? is this api-model configured?) → join against `bodhi-server /bodhi/v1/models`.
- **HF catalog backing** (if nothing local, is it pullable from HF?) → HF `/api/models/{repo}` metadata fetched on-demand.

The frontend does the join. api.getbodhi.app's leaderboard endpoint returns canonical model identifiers; the frontend matches those against local entities via repo/filename/alias name.

## 5. Filter sidebar ↔ source mapping
| Filter group | Driven by |
|---|---|
| Specialization | static list served by `api.getbodhi.app /v1/specializations` |
| Kind | client-side (filters rows by `kind`) |
| Source | client-side (filters rows by provenance) |
| Capability | client-side, backed by `ModelMetadata.capabilities` from bodhi-server (local) + capability tags from api.getbodhi.app (catalog) |
| Size · rig | client-side — requires `size_bytes` on every row (local from bodhi-server, catalog from HF+api.getbodhi.app) |
| Cost · api | client-side — requires `cost_per_mtok_in/out` on `api-model` / `provider-connected` (bodhi-server) + `provider-unconnected` (api.getbodhi.app) |
| License | client-side — metadata field normalized in api.getbodhi.app |
| Format | client-side — `api_format` for api rows, `GGUF` implied for local |

## 6. Composition rules
- **The frontend fetches all three sources in parallel** on page load (My mode skips api.getbodhi.app + HF). React Query cache TTLs:
  - bodhi-server user data: 5 s (invalidate on mutation)
  - api.getbodhi.app public data: 1 h (background refresh)
  - HF repo listings: 5 min (background refresh on focus)
- **Cross-source backlinks** (`↗ catalog`, `✓ N local aliases`) are computed client-side by matching canonical identifiers:
  - `hf-repo.id = "Qwen/Qwen3.5-9B-GGUF"` ↔ `alias.repo + alias.filename` (prefix match)
  - `provider-unconnected.provider_code = "openai"` ↔ `provider-connected.provider_code`
  - `api-model.provider_code = "openai"` ↔ `provider-connected.provider_code`
- **Conflicts are resolved "my data wins"**: if a provider is both in api.getbodhi.app directory *and* the user has a connection to it, render only the connected version with `connected` state. The directory entry is suppressed.

## 7. Error / partial-failure behaviour
- Each source fetches independently — one failing does not block the others.
- If `api.getbodhi.app` is down, `All Models` renders without the directory section and the banner `Bodhi Directory unavailable — retry` sits above the rows.
- If HF is rate-limited (429), HF repo rows fall back to the enriched cache from api.getbodhi.app (which has a HF mirror — see `30-huggingface-integration.md`). Staleness chip appears on affected rows.
- If `bodhi-server` is unreachable, the whole page shows the single-source error (app isn't usable without the local server).

## 8. Pagination
All three sources expose pagination differently:
| Source | Pagination style |
|---|---|
| bodhi-server | `?page=N&page_size=M` (1-indexed), response envelope `{ data, total, page, page_size }` |
| api.getbodhi.app | same shape as bodhi-server (matching style for consistency) |
| huggingface.co | `?limit=M&full=true` + cursor via `next` link header |

The frontend normalizes HF's cursor into a page-like abstraction in its data layer.

## 9. Caching + freshness
- **api.getbodhi.app is the mirror layer.** It periodically (weekly or nightly) scrapes HF for the top-N repos by downloads/trending + computes enriched fields (license normalization, capability inference from tags, architecture extraction from GGUF metadata via the dedicated endpoint — see `30-huggingface-integration.md §3`). The frontend can then page through a consistent catalog without hammering HF for every browse.
- **HF is called directly only for deep / fresh / long-tail queries** (exact search, raw tree listings, on-demand repo detail when the user clicks a not-yet-mirrored repo).
- **bodhi-server has no catalog cache of its own** — it owns user data only.

## 10. Versioning
- bodhi-server endpoints are under `/bodhi/v1/` (existing).
- api.getbodhi.app endpoints are under `/v1/`.
- Backward-compat rule: additive changes (new fields, new endpoints) require no version bump; breaking changes require `/v2`.
