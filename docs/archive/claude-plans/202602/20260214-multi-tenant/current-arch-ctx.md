# Current Architecture Context

## Database Layer

### SQLite Structure
- **ORM/Query Layer**: SQLx with runtime queries (NOT compile-time checked)
- **Connection Pool**: `SqlitePool` via `DbPool::connect(url)`
- **Migration System**: sqlx-based migrations in `crates/services/migrations/`
- **Two databases**: `bodhi.db` (app data) and `sessions.db` (tower_sessions)

### Current Tables

#### User-Scoped Tables (already have `user_id`)
- `api_tokens` - Per-user auth tokens with `user_id`, `token_prefix`, `token_hash`, `scopes`, `status`
- `toolsets` - Per-user toolset instances with `user_id`, `toolset_type`, `slug`, `encrypted_api_key`. UNIQUE(user_id, slug)
- `user_aliases` - Model aliases with global UNIQUE(alias) constraint (NOT per-user)
- `app_access_requests` - OAuth app consent with `app_client_id`, `user_id`, `status`

#### Global Tables (NO user_id - need org_id)
- `download_requests` - Model download tracking, NO user/org isolation
- `access_requests` - Legacy user onboarding (user_id present but for requesting user)
- `api_model_aliases` - Remote API configs with global UNIQUE prefix, shared encrypted keys
- `model_metadata` - GGUF + API model metadata, no ownership
- `app_toolset_configs` - Global toolset type enable/disable
- `tower_sessions` - HTTP session storage with user_id column

### Key Constraints That Need Changes
- `user_aliases.alias` UNIQUE → needs `UNIQUE(org_id, alias)` or `UNIQUE(org_id, user_id, alias)`
- `api_model_aliases.prefix` UNIQUE → needs `UNIQUE(org_id, prefix)`
- `app_toolset_configs.toolset_type` UNIQUE → needs `UNIQUE(org_id, toolset_type)`

---

## Auth Architecture

### Authentication Methods
1. **Session-Based** (browser): HTTP session via tower_sessions, access/refresh tokens in session
2. **Bearer Token** (API): `Authorization: Bearer <token>` with SHA-256 digest lookup
3. **External OAuth Apps**: App-to-app resource access via Keycloak

### User Identity Flow
1. User authenticates → `AuthService.exchange_auth_code()`
2. JWT token validated → extract `user_id` from 'sub' claim
3. Session created → `AppSessionStore` with user_id tracking
4. Middleware injects headers: `X-BodhiApp-User-Id`, `X-BodhiApp-Role`, `X-BodhiApp-Scope`
5. Extractors in handlers: `ExtractUserId`, `ExtractRole`

### Role Hierarchy
```
Admin > Manager > PowerUser > User
```
Roles are already **client-scoped** in Keycloak → natural fit for per-org roles.

---

## Service Layer

### Service Registry (AppService trait)
```
SettingService, DataService, HubService, AuthService, DbService,
SessionService, SecretService, CacheService, TimeService, AiApiService,
ConcurrencyService, QueueProducer, ToolService, NetworkService, AccessRequestService
```

### State Characteristics
- **DbService**: SqlitePool + encryption_key + TimeService (key state holder)
- **SessionService**: AppSessionStore wrapping SqliteStore + SqlitePool
- **AuthService**: reqwest::Client + auth_url + realm (global KC config → needs per-org)
- **SecretService**: Platform keyring access + SettingService
- **Others**: Mostly stateless facades wrapping DB/HTTP calls

### Dependency Injection
- All services: `Arc<dyn Trait>` for thread-safe sharing via `RouterState`
- `RouterState` holds `Arc<dyn AppService>` → single service registry

---

## Middleware Chain

### Request Flow (bottom-to-top execution)
1. `TraceLayer` → HTTP tracing
2. `CorsLayer` → CORS headers
3. `canonical_url_middleware` → URL normalization
4. `auth_middleware` → Injects X-BodhiApp-* headers (user_id, role, scope)
5. `api_auth_middleware` → Role-based + scope-based authorization per route group

### Auth Middleware Logic
1. Strip user-provided X-BodhiApp-* headers (security)
2. Check app status (reject if setup pending)
3. Bearer token path: validate via TokenService → inject headers
4. Session token path: same-origin CSRF check → extract from session → auto-refresh

---

## Configuration

### Settings Hierarchy
Priority: System > CommandLine > Environment > User > Default

### Key Settings
- `BODHI_HOME` → global data directory
- `BODHI_AUTH_URL` → Keycloak endpoint (currently single, needs per-org)
- `BODHI_AUTH_REALM` → Keycloak realm
- `BODHI_PORT` → HTTP server port
- `HF_HOME` → HuggingFace cache directory

### Runtime Config
- `SettingService.set_setting()` writes to `settings.yaml`
- Change listeners for dependent services
- NO per-org configuration support currently

---

## Multi-Tenancy Readiness Assessment

### Already Multi-User Ready
- Toolsets: `UNIQUE(user_id, slug)` per user
- API tokens: `user_id` scoped with token digest lookup
- Sessions: `user_id` tracking with `clear_sessions_for_user()`
- Access requests: `user_id` for request + approval tracking
- Roles: Client-scoped in Keycloak

### Single-Tenant Assumptions (BLOCKING)
1. Shared model download state (no user/org isolation)
2. Global alias namespace (UNIQUE constraint not per-org)
3. Shared API model configurations (one encrypted key per prefix)
4. Global model metadata (no ownership)
5. Shared file system layout (BODHI_HOME is global)
6. Global LLM server context (SharedContext manages ONE process)
7. Global configuration (settings.yaml for all)
8. AuthService has single KC client config (needs per-org client resolution)

### Services Needing Per-Org Awareness
- **DbService**: All queries need `org_id` filter
- **AuthService**: Per-org KC client_id resolution
- **SecretService**: Per-org encryption key
- **SessionService**: Org-aware session storage
- **DataService**: Org-scoped model aliases
- **ToolService**: Org-scoped toolset configs
- **AiApiService**: Org-scoped API model aliases
