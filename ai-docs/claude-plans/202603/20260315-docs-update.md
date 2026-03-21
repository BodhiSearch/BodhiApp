# Documentation Overhaul Plan

## Context

BodhiApp's docs (`crates/bodhi/src/docs/`) were last updated Oct 2025. ~6 months of development has shipped major features: MCPs with OAuth auth, app/user access requests, scope redesign, GGUF metadata, settings→SQLite, BODHI_HOME change, `/apps/` API prefix, CORS, tool calling/thinking in chat, new Docker GPU variants, and the bodhi-js-sdk for third-party integration.

The docs need a comprehensive overhaul: repositioned as AI gateway/hub, restructured into subdirectories, new developer/ section (main USP: third-party integration), bodhi-js-sdk docs copied from downstream repo, and live screenshots captured.

**Rules**: No toolsets. No multi-tenant depth. No feature evolution timeline. No "coming soon". Present current state only.

---

## Final Directory Structure

```
docs/
├── _meta.json (order: 0)
├── intro.md (order: 0) ...................... REWRITE
├── install.md (order: 101) .................. UPDATE
│
├── features/ (order: 200)
│   ├── _meta.json
│   ├── chat/
│   │   ├── _meta.json (order: 200)
│   │   └── chat-ui.md ...................... UPDATE (tool calling, thinking, MCPs)
│   ├── models/
│   │   ├── _meta.json (order: 205)
│   │   ├── model-alias.md .................. UPDATE (consolidated list, GGUF metadata)
│   │   ├── api-models.md ................... UPDATE (forward_all_with_prefix)
│   │   ├── model-downloads.md .............. LIGHT UPDATE
│   │   └── model-files.md .................. LIGHT UPDATE
│   ├── mcps/
│   │   ├── _meta.json (order: 235)
│   │   ├── setup.md ....................... NEW (servers, instances, auth configs)
│   │   └── usage.md ....................... NEW (playground, chat integration)
│   ├── auth/
│   │   ├── _meta.json (order: 240)
│   │   ├── user-access-requests.md ........ RENAME+REWRITE (user onboarding only)
│   │   ├── app-access-management.md ....... NEW (user-facing: review/approve/deny/down-scope)
│   │   ├── api-tokens.md .................. UPDATE (DB-backed, encrypted, scopes)
│   │   └── user-management.md ............. UPDATE (role hierarchy, link both access types)
│   └── settings/
│       ├── _meta.json (order: 230)
│       └── app-settings.md ................ UPDATE (SQLite, source hierarchy)
│
├── developer/ (order: 250) .................. NEW SECTION
│   ├── _meta.json
│   ├── getting-started.md .................. NEW (register→SDK→login→API journey)
│   ├── bodhi-js-sdk/
│   │   ├── _meta.json (order: 252)
│   │   ├── getting-started.md .............. COPY from SDK repo + adapt
│   │   └── advanced.md .................... COPY from SDK repo + adapt
│   ├── app-access-requests.md .............. NEW (resource consent, API flow, scopes)
│   └── openapi-reference.md ................ MOVE from features/ + UPDATE (CORS, /apps/)
│
├── deployment/ (order: 300)
│   ├── _meta.json
│   └── docker.md .......................... MAJOR REWRITE
│
├── FAQ.md (order: 500) ..................... UPDATE
└── Troubleshooting.md (order: 600) ......... UPDATE
```

**Files to delete** (moved/renamed):
- `features/access-requests.md` → replaced by `features/auth/user-access-requests.md`
- `features/openapi-docs.md` → replaced by `developer/openapi-reference.md`

**New _meta.json files** (7):
- `features/chat/_meta.json`
- `features/models/_meta.json`
- `features/mcps/_meta.json`
- `features/auth/_meta.json`
- `features/settings/_meta.json`
- `developer/_meta.json`
- `developer/bodhi-js-sdk/_meta.json`

---

## Execution Phases

### Phase 1: Core Pages (sequential — sets positioning)

**Agent-Core**: `intro.md` + `install.md` + all `_meta.json` files

**intro.md** (order: 0) — Complete rewrite:
- Reposition as "AI gateway/hub" — local models + cloud API proxy + MCP tools + unified auth
- Key features: Unified AI Gateway, MCP Tool Integration, OpenAI + Ollama Compatible APIs, Role-Based Auth, Multi-Platform (Desktop + Docker CPU/CUDA/ROCm/Vulkan/MUSA/Intel/CANN), bodhi-js-sdk for developers, Built-in Chat UI with agentic tool execution
- Quick-start links to install, features, developer, deployment
- Remove all roadmap/coming-soon/evolution content

**install.md** (order: 101):
- BODHI_HOME default is now `~/.bodhi` (was `~/.cache/bodhi`)
- Document 6 setup wizard steps (SKIP toolsets step):
  1. Welcome 2. Resource Admin (OAuth) 3. Download Models (March 2026 catalog) 4. API Models 5. Browser Extension (minimal) 6. Complete
- Remove "Firefox/Safari coming soon"

**All _meta.json files**: Create directory metadata with order and title fields.

**Sources**: E2E `tests-js/specs/setup/setup-flow.spec.mjs`, `ai-docs/claude-plans/20260219-ai-gateway/`

### Phase 2: Feature Verticals (6 parallel agents)

Each agent explores its E2E tests, UI components, and current doc content before writing.

---

#### Agent-Models: `features/models/` (4 files)

**Explore**: Current docs, E2E `tests-js/specs/models/`, `tests-js/specs/api-models/`, page objects `ModelsListPage.mjs`, `LocalModelFormPage.mjs`, `ApiModelFormPage.mjs`, UI `crates/bodhi/src/app/ui/models/`

**features/models/model-alias.md** (order: 205):
- Consolidated models list (aliases + API models, source badges: "user", "system")
- GGUF model metadata: capabilities auto-extracted from headers (tool calling, thinking). Brief mention with what it looks like in UI
- Create/edit flow at `/ui/models/alias/new`, `/ui/models/alias/edit`

**features/models/api-models.md** (order: 210):
- `forward_all_with_prefix` — forwards unmapped models via prefix
- `prefix` config for custom naming (e.g., "openrouter/")
- Supported providers: OpenAI, OpenRouter, HuggingFace, any OpenAI-compatible
- Remove "Future Support: Anthropic, Grok"

**features/models/model-downloads.md** (order: 212): Light refresh, verify path `/ui/models/files/pull/`

**features/models/model-files.md** (order: 215): Remove "coming soon" about delete, verify path `/ui/models/files/`

---

#### Agent-MCPs: `features/mcps/` (2 new files)

**Explore**: E2E `tests-js/specs/mcps/mcps-crud.spec.mjs`, `mcps-oauth-auth.spec.mjs`, `mcps-oauth-dcr.spec.mjs`, `mcps-header-auth.spec.mjs`, UI `crates/bodhi/src/app/ui/mcps/`, `crates/bodhi/src/app/ui/mcp-servers/`, plans `20260217-mcps/`, `20260220-mcp-auth/`

**features/mcps/setup.md** (order: 236):
- **MCP Servers** (admin, `/ui/mcp-servers/`): URL, name, description, enabled. Auth types: None, Header (encrypted AES-256-GCM), OAuth Pre-Registered, OAuth DCR (RFC 7591/8414)
- **MCP Instances** (user, `/ui/mcps/`): Select server, tool discovery, tool whitelist, OAuth connect/disconnect, enable/disable, delete

**features/mcps/usage.md** (order: 237):
- **Playground** (`/ui/mcps/playground/`): Tool sidebar, form/JSON modes, execute, results tabs, non-whitelisted warnings
- **Chat Integration**: MCPs popover with badge, per-MCP/per-tool toggle, unavailability tooltips, agentic loop (message→tool call→execution→response), tool call UI (status badges, collapsible args/results)

---

#### Agent-UserAccess: `features/auth/` access request pages (2 files)

**Explore**: Current `features/access-requests.md`, E2E `tests-js/specs/request-access/multi-user-request-approval-flow.spec.mjs`, `tests-js/specs/oauth/oauth2-token-exchange.spec.mjs` (app access sections), `tests-js/specs/mcps/mcps-oauth-auth.spec.mjs` (access request approval), page objects `RequestAccessPage.mjs`, `UsersManagementPage.mjs`, `AccessRequestReviewPage.mjs`, UI `crates/bodhi/src/app/ui/request-access/`, `crates/bodhi/src/app/ui/apps/access-requests/review/`, backend `crates/routes_app/src/users/`, `crates/routes_app/src/apps/`

**features/auth/user-access-requests.md** (order: 241):
- Purpose: New users request a role to access the app
- Flow: OAuth login → no role → `/ui/request-access/` → click Request → Pending → admin/manager reviews → approve with role (User/PowerUser/Manager/Admin) or reject → session invalidated → re-auth with role
- Approval hierarchy: Admin assigns all, Manager assigns User/PowerUser/Manager, PowerUser cannot approve

**features/auth/app-access-management.md** (NEW, order: 244) — User-facing guide for managing third-party app access:
- **What it is**: When a third-party app wants to access your MCPs or API, you'll receive an access request to review
- **Review flow** (at `/ui/apps/access-requests/review`):
  - See app name, description, what it's requesting (MCP servers, role)
  - Select which specific MCP instances to grant (not all-or-nothing)
  - Down-scope the role: app requests PowerUser but you can grant User instead
  - Approve or Deny
- **Role limits for apps**: Only User and PowerUser — apps cannot get Manager/Admin
- **Expiry**: Unreviewed requests expire after 10 minutes
- **Privilege escalation protection**: You can only grant roles at or below your own level
- **Flow types**: Popup (auto-closes after decision) or Redirect (sends you back to the app)
- **What happens after approval**: App gets a scoped token that only works for approved resources
- Cross-reference to `developer/app-access-requests.md` for the developer/API perspective
- All Requests page: approved/pending/rejected with reviewer info
- COMPLETELY separate from app access requests

---

#### Agent-Chat: `features/chat/chat-ui.md` (update)

**Explore**: Current doc, E2E `tests-js/specs/chat/chat.spec.mjs`, `chat-mcps.spec.mjs`, `chat-agentic.spec.mjs`, UI `crates/bodhi/src/app/ui/chat/`, `crates/bodhi/src/components/chat/`

**features/chat/chat-ui.md** (order: 201):
- NEW: Tool calling support — models invoke tools during conversation
- NEW: Thinking model support — LLM internal reasoning display
- NEW: MCP integration (cross-ref mcps/usage.md): popover, agentic loops, tool call expansion UI
- Keep: streaming, history, parameter documentation
- Remove outdated screenshots

---

#### Agent-Deploy: `deployment/docker.md` (major rewrite)

**Explore**: Current doc, `devops/CLAUDE.md`, `devops/README.md`, all Dockerfiles, `devops/Makefile`, `devops/.env.example`

**deployment/docker.md** (order: 401):
- All GPU variants: CPU (AMD64+ARM64), CUDA, ROCm, Vulkan, MUSA (NEW), Intel (NEW), CANN (NEW)
- Port: internal 8080 for standalone
- `BODHI_DEPLOYMENT` is immutable build-time property
- Multi-tenant: brief mention only
- PostgreSQL+RLS for production, SQLite for desktop: brief mention
- Encryption key validation on startup
- Remove all broken links: `/docs/developer/configuration` (3x), `/docs/deployment/advanced`, `/docs/deployment/platforms`
- Remove "coming soon" placeholders, Docker Compose "not tested" note

---

#### Agent-Developer: `developer/` (new section — 4 files + 2 copied)

**Explore**: E2E `tests-js/specs/oauth/oauth2-token-exchange.spec.mjs`, `tests-js/specs/mcps/mcps-oauth-auth.spec.mjs`, `tests-js/pages/AccessRequestReviewPage.mjs`, `tests-js/pages/OAuthTestApp.mjs`, UI `crates/bodhi/src/app/ui/apps/access-requests/`, backend `crates/routes_app/src/apps/`, plans `20260210-access-request/impl/`, `20260227-scope-changes/`, current `features/openapi-docs.md`, SDK docs at `/Users/amir36/Documents/workspace/src/github.com/BodhiSearch/bodhi-browser/bodhi-js-sdk/docs/integration/`

**developer/getting-started.md** (NEW, order: 251):
- End-to-end tutorial: Register OAuth app → Install bodhi-js-sdk → Setup BodhiProvider → Login with resources → Call OpenAI-compatible APIs
- Direct HTTP primary, extension secondary (mentioned but not emphasized)
- Cross-references to bodhi-js-sdk/, app-access-requests, openapi-reference pages
- Code examples using `@bodhiapp/bodhi-js-react`

**developer/bodhi-js-sdk/** (COPY from SDK repo, order: 252):
- Copy `getting-started.md` and `advanced.md` from `/Users/amir36/Documents/workspace/src/github.com/BodhiSearch/bodhi-browser/bodhi-js-sdk/docs/integration/`
- Add YAML frontmatter (title, description, order)
- Fix relative links: `../authentication.md` etc. → either remove or point to SDK repo GitHub URLs
- Create `_meta.json` with title "Bodhi JS SDK"

**developer/app-access-requests.md** (NEW, order: 253):
- Resource consent model: apps request MCPs + API access, users review/approve
- API flow:
  1. `POST /bodhi/v1/apps/request-access` (unauthenticated) → draft + review_url
  2. User reviews → selects role (User/PowerUser only) + MCP instances
  3. App polls `GET /bodhi/v1/apps/access-requests/{id}` → gets `scope_access_request:<id>`
  4. Token exchange → access `/bodhi/v1/apps/mcps/` endpoints
- 10-minute draft expiry, privilege escalation protection
- Flow types: popup + redirect
- Coming: workspaces, agents as resource types

**developer/openapi-reference.md** (MOVE from features/openapi-docs.md, order: 254):
- Swagger UI access
- `/apps/` API prefix for external apps
- CORS policy: session endpoints (restrictive) vs API/external (permissive)
- OpenAI-compatible endpoints: `/v1/chat/completions`, `/v1/models`, `/v1/embeddings`
- `/bodhi/v1/` prefix for Bodhi-specific endpoints

---

### Phase 3: Support + Remaining Pages (after Phase 2)

**Agent-Support**: `FAQ.md` + `Troubleshooting.md` + `features/auth/user-management.md` + `features/settings/app-settings.md` + `features/auth/api-tokens.md`

**FAQ.md** (order: 500):
- Update "What is Bodhi App?" → AI gateway/hub
- Add MCP Q&As, user access Q&A, app access Q&A
- Update platform support (MUSA, Intel, CANN)
- Remove "Firefox/Safari coming soon", outdated entries

**Troubleshooting.md** (order: 600):
- Add MCP troubleshooting, access request troubleshooting
- Update BODHI_HOME → `~/.bodhi`
- Update Issue Resolution Flowchart

**features/auth/user-management.md** (order: 242):
- Role hierarchy, Admin+Manager approve
- Link to BOTH access request pages (user in auth/, app in developer/)

**features/settings/app-settings.md** (order: 231):
- Settings now in SQLite database
- Source hierarchy: System > CLI > Env > **Database** > SettingsFile > Default
- Add "Database" badge

**features/auth/api-tokens.md** (order: 243):
- DB-backed, encrypted storage
- Scopes: `scope_token_user`, `scope_token_power_user`, `scope_token_admin`
- Lifecycle: create→show/hide→copy→activate/deactivate
- Multi-user isolation

### Phase 4: Cleanup (after all content)

**Agent-Cleanup**: Global pass across ALL doc files:
- Delete old files: `features/access-requests.md`, `features/openapi-docs.md`
- Remove any "toolsets"/"toolset" references
- Remove all "coming soon" / "future" / "planned" placeholders
- Verify all internal `/docs/...` links resolve to new paths
- Remove broken links to non-existent pages
- Remove feature evolution language
- Verify all `<img src="/doc-images/...">` references
- Ensure YAML frontmatter on every .md file

### Phase 5: Screenshots (sequential, after content)

**Agent-Screenshots**: Browser automation against localhost:1135
1. Check app state, seed data if needed
2. Capture new screenshots:
   - `mcp-servers-list.jpg`, `mcp-server-new.jpg`, `mcp-instances-list.jpg`, `mcp-instance-new.jpg`
   - `mcp-playground.jpg`, `chat-mcps-popover.jpg`, `chat-tool-call.jpg`
   - `app-access-review.jpg`, `user-access-request.jpg`
3. Re-capture stale existing screenshots
4. Store in `crates/bodhi/public/doc-images/`

---

## Global Rules

- **No toolsets**: Remove all references. Do not document.
- **No multi-tenant depth**: Brief mention only.
- **No feature evolution**: Present features as current state. No "we added", "new in v1.2".
- **No "coming soon"**: Exception: app-access-requests can mention "workspaces, agents" as future resource types.
- **User vs App access requests**: COMPLETELY separate features. Different docs, different sections, different agents.
- **Direct HTTP primary**: Extension mode documented as secondary/alternative.
- **E2E tests as source of truth**: Each agent reads relevant specs before writing.
- **YAML frontmatter**: Every .md file needs `title`, `description`, `order`.
- **SDK docs**: Copy from downstream repo, add frontmatter, fix relative links.
- **Screenshots**: Capture from running app at localhost:1135.

---

## Verification

After each phase:
1. `cd crates/bodhi && npm run dev` → navigate `http://localhost:3000/docs`
2. Verify sidebar shows new structure: Features (Chat, Models, MCPs, Auth, Settings) → Developer → Deployment
3. Verify all internal links resolve
4. Verify all images load

Final:
1. `make build.ui-rebuild`
2. Navigate `http://localhost:1135/docs`
3. Click every page, verify rendering, links, screenshots
