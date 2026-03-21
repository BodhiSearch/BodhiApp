# Multi-Tenancy Decisions Context

## Q&A Session 1: Core Architecture

### Local LLM Features
- **Decision**: Runtime only, no gating
- All code always compiled. Single binary for both hosted and self-hosted
- Runtime config determines behavior (`BODHI_MODE=hosted` vs `standalone`)
- Local-LLM routes return errors/unsupported in hosted mode
- Dead code stays - no Cargo feature flags needed

### Data Isolation Strategy
- **Decision**: Row-level isolation with `org_id` column
- Single PostgreSQL schema, every table gets `org_id`
- PostgreSQL RLS for defense-in-depth
- All orgs in same tables

### Keycloak Organization Model
- **Decision**: Keycloak 26+ Organizations feature
- Single realm, orgs as first-class entities
- Each org has a **separate Keycloak client** with **client-scoped roles**
- Roles already client-scoped (Admin/Manager/PowerUser/User) - works naturally

### Single-Tenant Mode
- **Decision**: Proper org from Keycloak (not shadow)
- Local instance creates KC client during setup → receives org_id
- org_id stored in AppRegInfo (migrated from file → DB organizations table)
- Auth middleware injects org from organizations table for local instance
- For multi-tenant, Traefik injects X-BodhiApp-Org header

---

## Q&A Session 2: Org Scoping & Data

### Cross-Org Resources
- **Decision**: No system defaults, no inheritance
- Each org bootstrapped/seeded on creation
- Toolset seeding will be removed before release

### User-to-Org Membership
- **Decision**: Multi-org via subdomain
- Users can belong to multiple orgs
- Subdomain (my-org.getbodhi.app) determines active org context

### Query Layer
- **Decision**: SQLx for both, using `sqlx::Any` driver
- Single implementation using AnyPool
- Runtime DB selection (SQLite or PostgreSQL)
- Handle divergences with runtime dialect checks
- Rename SqliteDbService → DbServiceImpl

### Org Resolution
- **Decision**: Subdomain only
- Traefik extracts `<org>` from `<org>.getbodhi.app` → injects `X-BodhiApp-Org: <org>`
- Both frontend and API on same subdomain

---

## Q&A Session 3: Security & Tokens

### API Token Scoping
- **Decision**: Org-scoped tokens
- Bound to `(user_id, org_id)`, isolated per org

### Encryption Keys
- **Decision**: Per-org, stored in organizations table
- Simple DB column for now, KMS later

### Session Store
- **Decision**: PostgreSQL with separate connection pool
- Configurable `SESSION_DB_URL` with host/user/password/schema
- Two PgPool instances: app data + sessions
- tower-sessions-sqlx-store PostgresStore (existing crate)
- Org-scoped sessions (different subdomain = different session)

---

## Q&A Session 4: Features & Scope

### Toolset Config
- **Decision**: Org-level config (app_toolset_configs org-scoped)

### Access Requests
- **Decision**: Existing flow extends naturally (client = org)

### Audit Logging
- **Decision**: NATS JetStream, out of current scope
- Define interface now, implement later

### Rate Limiting
- **Decision**: Deferred

---

## Q&A Session 5: Database & Architecture

### Database Abstraction
- **Decision**: `sqlx::Any` (single implementation, runtime DB selection)

### Org ID Propagation
- **Decision**: Both extract + pass
- Middleware injects Extension<OrgContext>
- Handlers extract and pass org_id explicitly to services
- Services stay framework-agnostic

### Organizations Table
- **Decision**: Minimal - merged with AppRegInfo
- Fields: org_id, slug, display_name, kc_client_id, client_secret (encrypted), encryption_key, status, created_at, updated_at
- Status: active | suspended | deleted
- Exists in both SQLite and PostgreSQL

### DbPool / Repository Design
- **Decision**: Add org_id as explicit parameter to every org-scoped repository method
- Keep existing repository trait pattern (AccessRepository, TokenRepository, etc.)
- DbService super-trait composition stays

### Data Migration
- **Decision**: Clean-slate hosted, no backwards compatibility

### Org Provisioning
- **Decision**: External service (new.getbodhi.app), app is consumer only

### KC Client Mapping
- **Decision**: App DB mapping in organizations table

---

## Q&A Session 6: Infrastructure

### Proxy
- **Decision**: Traefik, slug-only injection

### Auth Middleware Org Resolution
- **Decision**: Traefik injects slug only, app resolves credentials from CacheService
- CacheService backed by Redis in multi-tenant, in-memory for single-tenant
- Event-driven cache invalidation

### CacheService
- **Decision**: Refactor to generic cache trait
- `get<T>(key)`, `set<T>(key, val, ttl)`, `invalidate(key)`
- Redis impl for multi-tenant, in-memory for single-instance

### PostgreSQL Topology
- **Decision**: Single shared DB for all orgs

### SecretService
- **Decision**: Remove entirely
- All secrets to DB (per-org) or env vars (platform)

### SettingService
- **Decision**: Platform-level only, no per-org settings

### DataService
- **Decision**: Rethink - becomes LocalModelService for local model management
- AiApi model aliases already in separate service
- Most DataService methods return UnsupportedOperation in multi-tenant mode
- Refactoring bundled with Phase 3

### Session Implementation
- **Decision**: Use tower-sessions-sqlx-store PostgresStore crate

---

## Q&A Session 7: Frontend & Login

### Frontend Org UX
- **Decision**: Org switcher + subdomain
- UI shows current org name + dropdown to switch
- Switching navigates to other org's subdomain
- Frontend fetches org list from API

### Login Flow
- **Decision**: Org-aware login
- Login handler extracts org from X-BodhiApp-Org header
- Looks up org's client_id from cache
- Uses org-specific KC client for OAuth redirect

### NAPI Bindings
- **Decision**: NAPI supports both modes (single-tenant + multi-tenant)

### Plan Format
- **Decision**: Index + phase files
- Overview README with phase summaries
- Detailed per-phase files for implementation

---

## Phase Ordering

1. **Phase 1**: Session PostgreSQL migration (F)
2. **Phase 2**: DB abstraction with sqlx::Any (A)
3. **Phase 3**: Org table + org_id threading + auth middleware + DataService refactor (B+C+D)
4. **Phase 4**: CacheService generic + Redis (E)
5. **Phase 5**: Docker/deployment changes (G)
6. **Phase 6**: Frontend multi-tenant changes

Each grouped phase produces a working app for testing.
