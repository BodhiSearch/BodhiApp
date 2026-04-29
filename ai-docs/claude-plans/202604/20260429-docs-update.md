# Bodhi App Docs Overhaul — Progressive-Disclosure Restructure

## Context

The public docs at `getbodhi.app/src/docs/` currently cover ~33 files / 6,300 lines, last meaningfully reorganized before a heavy run of feature work in Q1 2026. Since then BodhiApp has shipped (or evolved): Anthropic Messages API + OAuth, OpenAI Responses API, Gemini compat, MCP proxy + unified MCP auth (Header / OAuth2 preregistered / DCR), MCP playground, MCP store admin, agentic chat with pi-agent-core, Zustand + TanStack Router migration, security hardening (CSP, SSRF, AuthZ), access-request / external-app-access flows, multi-tenant deployment mode, and significant new env vars / runtime settings.

Public docs do not reflect most of this. Several existing pages (api-models, mcps/setup, app-settings, openapi-reference) are accurate but partial, and there is no foundation layer (concepts), no API-compat narrative, no advanced/reference tier, and no progressive-disclosure structure for users who arrive needing to learn vs. users who arrive needing to look something up.

**Outcome:** A restructured docs site that (a) covers every shipped feature, (b) tiers content from "I just installed it" → "I'm building against it" → "I'm self-hosting it" → "I need a reference table," and (c) ships in 4 phases so each phase is independently reviewable.

---

## Decisions (confirmed)

- **API reference style**: functional/narrative only — the docs site already embeds Swagger UI against `openapi.json`, so the docs do **not** duplicate endpoint schemas. Each compat layer gets a usage-oriented page (auth, gotchas, examples) that links into the embedded spec.
- **Scope**: phased rollout (4 phases) — each phase merges independently.
- **Deployment paths**: **Tauri desktop** + **Docker single-tenant** only. Multi-tenant, standalone HTTP, NAPI library — explicitly out of scope for this overhaul.
- **Screenshots**: reuse existing 30 images in `getbodhi.app/public/doc-images/`. Plan inventories the new screenshots needed (with exact route + UI state) so they can be captured manually before content lands.

---

## Framework constraints (do not violate)

- Next.js 14.2 + custom MD routing (`getbodhi.app/src/app/docs/utils.ts`). Markdown only — no MDX, no React components in `.md` files.
- Folders need `_meta.json` with `{ "order": <int>, "title": "..." }`. Files need frontmatter `title`, `description`, `order`. Sidebar is auto-built by `getDocsForSlug` in `utils.ts`.
- Cross-links use `/docs/<slug>` (no `.md` extension).
- Images use raw HTML: `<img src="/doc-images/<file>" alt="..." class="rounded-lg border-2" />`.
- No breadcrumb / TOC / "Related" components exist today — emulate via inline links.

---

## Target Information Architecture

```
src/docs/
├── intro.md                                       (order 0)        — refresh
├── install.md                                     (order 100)      — refresh
├── concepts/                                      (order 150)      — NEW
│   ├── overview.md
│   ├── deployment-modes.md                          (Tauri vs Docker only — no MT)
│   ├── models-aliases-files.md                      (alias vs file vs API model)
│   ├── api-compatibility.md                         (OpenAI/Anthropic/Gemini/Ollama mental model)
│   ├── auth-and-roles.md                            (User/PowerUser/Manager/Admin + scopes)
│   └── mcp-overview.md                              (MCP, tools, auth methods)
├── features/                                      (order 200)
│   ├── chat/
│   │   ├── chat-ui.md                               (refresh)
│   │   ├── tool-calling.md                          (NEW — agentic / MCP in chat)
│   │   └── parameters-and-system-prompt.md          (NEW)
│   ├── models/
│   │   ├── overview.md                              (NEW — disambiguate alias/file/API)
│   │   ├── model-alias.md                           (refresh)
│   │   ├── model-files.md                           (refresh)
│   │   ├── model-downloads.md                       (refresh)
│   │   ├── api-models.md                            (refresh — OpenAI/Anthropic/Gemini/Groq, test, fetch-models)
│   │   └── anthropic-oauth.md                       (NEW — ApiFormat::AnthropicOAuth)
│   ├── mcps/
│   │   ├── overview.md                              (NEW — concepts entry)
│   │   ├── setup.md                                 (refresh)
│   │   ├── auth-methods.md                          (NEW — Header / OAuth2 Preregistered / DCR)
│   │   ├── playground.md                            (NEW — tool testing UI)
│   │   ├── pre-registered-servers.md                (NEW — admin catalog)
│   │   └── usage.md                                 (refresh — usage in chat)
│   ├── auth/
│   │   ├── overview.md                              (NEW — roles + scopes mental model)
│   │   ├── user-management.md                       (refresh)
│   │   ├── user-access-requests.md                  (refresh)
│   │   ├── api-tokens.md                            (refresh)
│   │   └── app-access-management.md                 (refresh)
│   └── settings/
│       └── app-settings.md                          (refresh — note editable subset)
├── deployment/                                    (order 300)
│   ├── overview.md                                  (NEW — desktop vs Docker decision)
│   ├── desktop.md                                   (NEW — Tauri tray, autostart, paths)
│   ├── docker.md                                    (refresh — variants, env vars, compose)
│   └── reverse-proxy.md                             (NEW — TLS, rate limit at proxy)
├── developer/                                     (order 400)      — bumped from 250
│   ├── getting-started.md                           (refresh)
│   ├── building-apps.md                             (refresh)
│   ├── app-access-requests.md                       (refresh)
│   ├── browser-extension.md                         (NEW)
│   ├── openapi-reference.md                         (refresh — point at embedded Swagger UI)
│   └── bodhi-js-sdk/
│       ├── getting-started.md                       (refresh)
│       └── advanced.md                              (refresh)
├── api-compatibility/                             (order 500)      — NEW (functional, not endpoint dumps)
│   ├── overview.md                                  (entry + link to embedded Swagger UI)
│   ├── openai-chat-completions.md                   (/v1/chat/completions usage)
│   ├── openai-responses.md                          (/v1/responses async polling)
│   ├── openai-embeddings.md                         (/v1/embeddings usage)
│   ├── anthropic-messages.md                        (/anthropic/v1/messages, x-api-key)
│   ├── gemini.md                                    (/v1beta/*, x-goog-api-key)
│   ├── ollama.md                                    (/api/* — flag deprecated)
│   ├── mcp-proxy.md                                 (/bodhi/v1/apps/mcps/{id}/mcp)
│   └── error-format.md                              (BodhiErrorResponse vs OAI error envelope)
├── advanced/                                      (order 600)      — NEW
│   ├── architecture.md                              (crate chain + request lifecycle, user-friendly)
│   ├── security-model.md                            (public-friendly summary of func-specs/security)
│   ├── inference-stack.md                           (llama.cpp variants, BODHI_LLAMACPP_ARGS, keep-alive)
│   ├── performance-tuning.md                        (variants × hardware, concurrency, NTFS/macOS notes)
│   └── observability.md                             (logs, queue, settings page, log levels)
├── reference/                                     (order 700)      — NEW
│   ├── env-vars.md                                  (alphabetical matrix from settings/constants.rs)
│   ├── settings.md                                  (DB > YAML > Env > Default; editable subset)
│   ├── roles-and-scopes.md                          (matrix: AuthContext × endpoints)
│   ├── error-codes.md                               (errmeta error codes index)
│   └── glossary.md
└── support/                                       (order 800)
    ├── faq.md                                       (refresh — relocated from FAQ.md)
    ├── troubleshooting.md                           (significant expansion)
    └── whats-new.md                                 (NEW — feature highlights, not full changelog)
```

**Progressive-disclosure mechanics (apply to every page):**
- Lead with a one-paragraph "what this is / who it's for" hook.
- Body covers the common path. Anything power-user / self-hoster goes into a clearly-labeled "Advanced" section near the end, OR moves to `/docs/advanced/` or `/docs/reference/`.
- "Skip ahead" callouts at top of long pages: e.g. *"If you just want X, jump to Y."*
- Inline `/docs/<slug>` links to deeper pages instead of inlining advanced content.
- Use the existing `<img class="rounded-lg border-2" />` pattern; no new components.

---

## Authoritative source map (for sub-agents to mine, not duplicate)

Sub-agents synthesize **public-facing** prose from these — never paste raw spec content.

| Topic | Source(s) |
|---|---|
| Auth, roles, OAuth, tokens | `ai-docs/01-architecture/authentication.md`, `ai-docs/02-features/implemented/authentication.md`, `ai-docs/02-features/active-stories/api-tokens.md` |
| Security model summary | `ai-docs/func-specs/security/security.md` (public-friendly summary only — do not leak threat model details that aren't already implicit in shipped behavior) |
| MCP auth | `ai-docs/02-features/mcp/prompt.md` + code in `crates/services/src/db/objs/mcp_*` |
| API model / provider strategy | `ai-docs/claude-plans/202604/20260410-provider-model.md` |
| Anthropic API + OAuth | `ai-docs/claude-plans/202604/20260408-anthropic/`, `20260411-anthropic-oauth/` |
| Responses API | `ai-docs/claude-plans/202604/20260407-responses-api/` |
| Tauri desktop | `ai-docs/01-architecture/tauri-desktop.md` |
| Env vars / settings | `crates/services/src/settings/constants.rs` (canonical; no spec) |
| Endpoints | `crates/routes_app/src/`, `crates/routes_oai/src/`, plus the live `/openapi.json` |
| Frontend routes | `crates/bodhi/src/routes/` |

---

## Phased Execution

Each phase = one branch, one merge. Run gate checks (`cd getbodhi.app && npm run build && npm run dev` smoke + cross-link audit) before merging each phase.

### Phase 1 — Foundation refresh + Concepts layer

**Goal:** create the progressive-disclosure spine; refresh the four "front door" docs.

**Sub-agents:** 1 agent (small enough to be coherent in one pass).

**Tasks:**
1. Create new top-level `_meta.json` files for `concepts/` (150), `api-compatibility/` (500), `advanced/` (600), `reference/` (700), `support/` (800). Bump `developer/_meta.json` order from 250 → 400. Bump `deployment/_meta.json` from 300 → 300 (unchanged). Adjust feature subgroup orders if they collide.
2. Move `FAQ.md` → `support/faq.md`, `Troubleshooting.md` → `support/troubleshooting.md`. Add stub redirects only if `getbodhi.app/src/app/docs/[...slug]/` does not 404 gracefully (verify first; do not invent redirect logic).
3. Refresh `intro.md` — current "Key Features" list is shipped, but add: Anthropic + Gemini compat, Responses API, MCP proxy, multi-platform Docker variants. Keep "Quick Start" links pointed at new IA.
4. Refresh `install.md` — verify against current install flow (confirm screenshots `setup-*.jpg` still match by reading `crates/bodhi/src/routes/setup/`).
5. Write `concepts/` pages (6 files). Each ≤ 250 lines. Tone: friendly, mental-model-first.
6. Cross-link audit: every `/docs/...` link in refreshed pages must resolve.

**Verification:**
- `cd getbodhi.app && npm install && npm run build` succeeds.
- `npm run dev` and click through every nav item; no 404s.
- Sidebar order matches the IA above.

---

### Phase 2 — Feature page expansion

**Goal:** every UI feature in `crates/bodhi/src/routes/` is documented with the user-facing tier in `features/`.

**Sub-agents:** 4 in parallel, scoped to disjoint folders.

- **Agent A — Chat:** refresh `chat-ui.md`; create `tool-calling.md` (agentic flow, MCP tool selection in chat, tool call/result display) and `parameters-and-system-prompt.md` (temperature/top-p/max-tokens/stop, system prompt override). Cite `crates/bodhi/src/routes/chat/`.
- **Agent B — Models:** refresh 4 existing files; create `models/overview.md` and `models/anthropic-oauth.md`. The api-models refresh must cover all four providers (OpenAI/Anthropic/Gemini/Groq) and document `test` + `fetch-models` actions. Cite `crates/routes_app/src/routes_api_models.rs` and the api-models route in `crates/bodhi/src/routes/models/api/`.
- **Agent C — MCPs:** refresh `setup.md` and `usage.md`; create `overview.md`, `auth-methods.md` (Header / OAuth2 preregistered / OAuth2 DCR — explain when to pick which), `playground.md`, `pre-registered-servers.md` (admin-only). Cite `crates/services/src/db/objs/mcp_*` and `crates/bodhi/src/routes/mcps/`.
- **Agent D — Auth + Settings:** refresh 4 auth pages + `app-settings.md`; create `auth/overview.md` (single page that maps role → capabilities). Cite `crates/auth_middleware`, `crates/services/src/auth_service.rs`.

**Cross-cutting rules:**
- Each agent produces a screenshot-gap list: file path, route to capture, UI state required (e.g. "logged in as Manager, on `/ui/users/access-requests` with 1+ pending row").
- No agent edits another agent's section.
- All four converge on the same docs voice — Phase 1's refreshed `intro.md` and `install.md` are the style reference.

**Verification:**
- Build + cross-link audit.
- Spot-check by running `make app.run.live` and clicking through each feature while reading the refreshed page side-by-side.

---

### Phase 3 — API compatibility narrative

**Goal:** functional, narrative pages for each compat layer. **No endpoint schemas** — defer to embedded Swagger UI.

**Sub-agents:** 2 in parallel.

- **Agent E — OpenAI family + Ollama:** `api-compatibility/overview.md`, `openai-chat-completions.md`, `openai-responses.md`, `openai-embeddings.md`, `ollama.md`. Each ≤ 200 lines. Each ends with "**Full schema:**" link to the embedded Swagger UI on the deployed site (confirm the URL by reading `getbodhi.app/src/app/api-reference/` or wherever Swagger is mounted — DO NOT GUESS).
- **Agent F — Anthropic / Gemini / MCP proxy / errors:** `anthropic-messages.md`, `gemini.md`, `mcp-proxy.md`, `error-format.md`. Document header rewriting (`x-api-key`, `x-goog-api-key`), the MCP proxy auth model, and the two error envelopes.

**Both agents must:**
- Provide one curl example per page that a developer can paste and run.
- Refresh `developer/openapi-reference.md` to be a concise pointer to (a) embedded Swagger UI, (b) the `api-compatibility/` narrative pages.

**Verification:**
- Run each curl example against `make app.run` to confirm it works.
- Build + cross-link audit.

---

### Phase 4 — Deployment expansion + Advanced + Reference + Support

**Goal:** complete the self-hoster and power-user tier; finish support pages.

**Sub-agents:** 3 in parallel.

- **Agent G — Deployment:** refresh `docker.md`; create `deployment/overview.md` (decision page: desktop vs Docker), `desktop.md` (Tauri specifics: tray, autostart, `~/.bodhi` layout, native-server architecture from `ai-docs/01-architecture/tauri-desktop.md`), `reverse-proxy.md` (TLS termination, rate-limit-at-proxy per `func-specs/security`, `BODHI_PUBLIC_*` vars).
- **Agent H — Advanced:** `architecture.md` (request lifecycle: middleware → route → service → llama.cpp / API model — user-friendly, not crate-internal), `security-model.md` (public-safe summary; cross-link to relevant features), `inference-stack.md` (variants × hardware, `BODHI_EXEC_VARIANT`, `BODHI_LLAMACPP_ARGS`, `BODHI_KEEP_ALIVE_SECS`), `performance-tuning.md`, `observability.md`.
- **Agent I — Reference + Support:** `reference/env-vars.md` (alphabetical matrix from `crates/services/src/settings/constants.rs`), `reference/settings.md` (precedence + editable subset), `reference/roles-and-scopes.md` (matrix), `reference/error-codes.md` (index from errmeta `error_code` attributes — grep `crates/` for them), `reference/glossary.md`. Major expansion of `support/troubleshooting.md` (collect from real GitHub issues + the FAQ). Create `support/whats-new.md` (highlights of Q1 2026 features, ~1 page).

**Verification:**
- Reference matrices spot-checked against current code (env-vars.md vs `constants.rs`; roles vs `auth_middleware`).
- Build + final cross-link audit.
- Final read-through: walk the sidebar top-to-bottom as a "new user," then again as a "power user," then again as a "developer" — each path should feel coherent.

---

## Sub-agent prompt template (consistent across phases)

When spawning each agent, supply:
1. **Scope:** exact files to create/edit, with target slugs.
2. **Style anchor:** point to refreshed `intro.md` + `developer/getting-started.md` as voice reference.
3. **Authoritative sources:** the relevant rows from the source map above + specific code paths to verify against.
4. **Constraints:** Markdown-only (no MDX), `<img>` HTML pattern, frontmatter format, `_meta.json` shape, max ~300 lines per page.
5. **Deliverables:** files + screenshot-gap list + a one-paragraph note flagging any spec/code mismatch encountered.
6. **Forbidden:** inventing endpoints, duplicating `openapi.json` schemas, referencing multi-tenant / NAPI / standalone HTTP modes.

---

## Critical files & utilities (reuse, do not reinvent)

- `getbodhi.app/src/app/docs/utils.ts` — sidebar builder. Adding a folder + `_meta.json` is sufficient; no code change needed.
- `getbodhi.app/src/app/docs/[...slug]/page.tsx` — Markdown renderer. Confirms supported MD features (gfm, prism syntax highlighting). No MDX.
- `getbodhi.app/public/doc-images/` — image directory. Reuse: `chat-ui.jpg`, `models-page.jpg`, `users.jpg`, `api-tokens.jpg`, `app-settings.jpg`, `mcp-*.jpg`, `setup-*.jpg`. Capture for Phase 2/3 (final list emerges from each agent's gap report).
- Embedded Swagger UI mount point — locate before Phase 3 (likely under `getbodhi.app/src/app/`); each api-compatibility page links to it.

---

## End-to-end verification

Per phase:
1. `cd getbodhi.app && npm install && npm run build` — must succeed.
2. `npm run dev` — open every new/changed page in the browser; sidebar order, frontmatter title/description, image references all render.
3. Cross-link audit — `grep -r '/docs/' getbodhi.app/src/docs/` and confirm each target slug exists.
4. For Phase 2/3 user-flow pages: `make app.run.live` in parallel and walk the documented flow against a live instance.
5. For reference matrices (Phase 4): diff against source-of-truth files (`constants.rs`, `auth_middleware`) — flag drift.

Final acceptance: walk the sidebar three times (new user / power user / developer); each persona's path is coherent and self-contained, with explicit "skip ahead" links wherever a beginner would otherwise be dumped into advanced material.
