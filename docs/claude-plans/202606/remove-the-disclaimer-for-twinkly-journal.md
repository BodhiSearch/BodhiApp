# My Models rail: "Chat with Model" actions + reusable alias→model-string utility

## Context

The My Models detail rail (`ModelDetailRail.tsx`) currently lets you only **Edit** an alias. There's no fast path from a model you can see to actually using it. This change adds **Chat** affordances to every rail type so a user can jump straight from a listed alias to a pre-loaded chat, and cleans up the Local File rail (drop the read-only disclaimer and the pointless Edit button).

The chat endpoint resolves a `model` string to an alias on the backend. To keep the frontend links correct and aligned with that resolution, we add a small **tested** utility that mirrors the backend derivation, rather than scattering string-building inline. We also fold in rail polish the user asked for: HuggingFace links on local-file rows and a copy button on the API base URL.

**Scope:** pure frontend (`crates/bodhi/src`). No backend, no ts-client regeneration.

### Backend resolution being mirrored (ground truth)
`DataService::find_alias` matches `request.model` in priority order; the string that selects each alias:
- **user** alias → its `alias` field verbatim
- **model_router** → its `alias` field verbatim
- **model** (auto-discovered local GGUF) → its `alias` field, already pre-derived as `{repo}:{quant}` (e.g. `gpustack/bge-m3-GGUF:Q4_K_M`)
- **api** alias → per-model: `matchable_models()` = `${prefix ?? ''}${modelId(model)}` (`ApiAlias::matchable_models`, `crates/services/src/models/model_objs.rs`)

So the chat URL is always `/ui/chat/?model=<X>` where X = `alias.alias` for local/user/router, and `(prefix ?? '') + modelId(model)` for an API model.

### Confirmed product decisions
1. Local File: remove disclaimer + remove Edit; single **Chat with Model** primary button.
2. User & Router: keep Edit, add **Chat** as the primary (accent) action; Edit becomes secondary (outline).
3. API rail: render each model as a minimal Provider-style card (mono id/name, no pricing/caps) with a chat button.
4. Utility: new tested `src/lib/modelAlias.ts`.

### De-risked during research
- `ModelsScreenV2.tsx` already imports `catalog.css` **and** `list.css`, and CSS classes are global → `.cat-prov-model*` and `.dp-*` are already available on this route. **No new CSS file/import needed.**
- TanStack `<Link to="/chat/" search={{model}}>` renders an href-assertable link even in the single-route test harness (the provider rail already relies on this).
- `ShellIcon` resolves any lucide name (`message-circle`, `external-link`) dynamically.
- E2E dev-server seeds **local-GGUF aliases only** (no API keys) → E2E targets the Local File chat button; API-card hrefs are covered by component tests.

---

## Implementation

### 1. New util — `crates/bodhi/src/lib/modelAlias.ts`
Imports `AliasResponse`, `ApiAliasResponse` from `@bodhiapp/ts-client` and the guards from `@/lib/utils`.

- `modelId(m)` — **move verbatim** from `ModelDetailRail.tsx` (currently inline, lines 160-165): returns `m.id` (OpenAI/Anthropic) or `m.name` (Gemini), JSON fallback.
- `apiModelChatString(alias, model)` → `` `${alias.prefix ?? ''}${modelId(model)}` `` (mirrors `matchable_models()`).
- `chatModelForAlias(alias): string | null` → `null` if `isApiAlias` (resolves per-model), else `alias.alias` (user/model/model_router all carry `.alias`).

### 2. `ModelDetailRail.tsx` edits
Remove inline `modelId`; import `modelId`, `apiModelChatString`, `chatModelForAlias` from `@/lib/modelAlias`; import `{ Link }` from `@tanstack/react-router` and `CopyButton` from `@/components/CopyButton`.

**`Row` component** — add optional props (existing call sites unchanged):
- `href?: string` → render value as `<a className="dp-row-v mono" target="_blank" rel="noopener noreferrer">{v} <ShellIcon name="external-link" size={12}/></a>`
- `copyable?: boolean` → append `<CopyButton text={v} size="icon" variant="ghost" className="dp-row-copy" />`

**Footer** (replace single Edit button, branch on `alias.source`):
- `model`: ONE accent button — `<Link to="/chat/" search={{ model: chatModel! }} className="dp-btn dp-btn-accent" data-testid="model-detail-chat">… Chat with Model</Link>`. No Edit.
- `user` / `model_router`: TWO stacked — Chat (`dp-btn-accent`, `data-testid="model-detail-chat"`, label "Chat with Model" / "Chat with Router") + Edit (`dp-btn-outline`, keep `onEdit` + `data-testid="model-detail-edit"`).
- `api`: keep ONLY the Edit button (`dp-btn-accent`) — chat is per-model in the body.

**`LocalRailBody`:**
- repo row → `href={`https://huggingface.co/${local.repo}`}`
- filename row → `href={`https://huggingface.co/${local.repo}/blob/main/${local.filename}`}`
- Disclaimer `.dp-desc` section: render **only when `alias.source === 'user'`** (keeps "User-created alias…"); removed entirely for `source === 'model'`.

**`ApiRailBody`:**
- base URL row → add `copyable`.
- Models section: keep `data-testid="model-detail-models"` on container, switch to `.cat-prov-models`; render each model as a minimal card reusing `.cat-prov-model`, `.cat-prov-model-head`, `.cat-prov-model-name mono`, `.cat-prov-model-head-right`, `.cat-prov-model-add`. The card's action is `<Link to="/chat/" search={{ model: apiModelChatString(alias, m) }} className="cat-prov-model-add" data-testid={`model-detail-chat-${modelId(m)}`}><ShellIcon name="message-circle" size={15}/></Link>`. No price/caps sub-row.

### 3. CSS — `routes/models/-components/models.css` (optional, minor)
- Add `.dp-row-copy` to shrink the icon button to ~24px so it fits `.dp-row` (shadcn `size="icon"` is 36px). Verify visually; skip if acceptable.
- Leave now-unused `.m-model-list`/`.m-model-item` (harmless).

---

## Tests

**Unit — new `crates/bodhi/src/lib/modelAlias.test.ts`** (fixtures from `test-fixtures/models.ts`):
- `chatModelForAlias`: local GGUF passthrough, user passthrough, router passthrough, api → `null`.
- `apiModelChatString`: with prefix (`azure/` + `gpt-4` → `azure/gpt-4`), without prefix (`gpt-4`).
- `modelId`: id branch (OpenAI/Anthropic) and **Gemini** name branch. Add `createMockGeminiModel(name)` to fixtures (or construct inline).

**Component — extend `crates/bodhi/src/routes/models/index.v2.test.tsx`:**
- Local File: disclaimer gone; `model-detail-chat` href contains `/chat/` + `model=<alias>`; NO `model-detail-edit`; repo/filename rows are `<a>` with correct HF hrefs.
- User: both `model-detail-chat` (`model=<alias>`) and `model-detail-edit`; "User-created alias…" still present.
- Router: both Chat (`model=<alias>`) and Edit.
- API: card per model (`model-detail-model-<id>`); chat link href = `model=<prefix><id>` (test prefix + no-prefix variants); base-URL row has copy button.
- Confirm existing tests still pass (model name still rendered in `.cat-prov-model-name`; API rail keeps Edit-nav).

**E2E — extend `crates/lib_bodhiserver/tests-js/specs/models/all-models-v2.spec.mjs`** (black-box, local-GGUF only):
- Add `railChat` selector to the page object. Open first local-file row, assert Chat button visible, href contains `/ui/chat/?model=`, click it, `expect(page).toHaveURL(/\/ui\/chat\/\?.*model=/)`. No API-specific assertions.

---

## Verification

From `crates/bodhi`:
1. `npm test -- modelAlias` — unit green.
2. `npm test -- routes/models/index.v2` — component green.
3. `npm run lint` — clean (watch the new `Row` props and `chatModel!` narrowing).
4. `make test.e2e` (from `crates/lib_bodhiserver/tests-js`) — E2E green.

Chrome MCP on `localhost:1135` (`/ui/models/`):
1. Local File row → single "Chat with Model" accent button, no Edit, no "Auto-discovered…" text; repo/filename are external HF links.
2. Click it → `/ui/chat/?model=<alias>`, selector reflects it.
3. User row → Chat (accent) + Edit (outline); disclaimer still shown.
4. Router row → Chat ("Chat with Router") + Edit.
5. API row → base-URL copy icon works; each model is a card with a `message-circle` chat button → `/ui/chat/?model=<prefix><modelId>`.

## Risks
- `chatModel!` non-null is safe — only the non-api footer branch uses it.
- CopyButton height in `.dp-row` may need the `.dp-row-copy` override; confirm in step 5.
- `prefix` is `string | null | undefined`; `?? ''` covers all.
