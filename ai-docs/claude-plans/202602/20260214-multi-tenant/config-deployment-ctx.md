# Configuration & Deployment Context

## Current Configuration

### SettingService
- **Hierarchy**: System > CommandLine > Environment > SettingsFile > Default
- **File**: `{BODHI_HOME}/settings.yaml`
- **Auth settings**: `BODHI_AUTH_URL`, `BODHI_AUTH_REALM` (global, same for all orgs)
- **DB paths**: `app_db_path()`, `session_db_path()` (both under BODHI_HOME)

### Environment Variables (Current)
```env
BODHI_HOME=~/.bodhi                    # Data directory
BODHI_AUTH_URL=https://auth.getbodhi.app  # Keycloak URL
BODHI_AUTH_REALM=bodhi                 # KC realm
BODHI_PORT=1135                        # Server port
BODHI_ENCRYPTION_KEY=...               # Master encryption key
HF_HOME=~/.cache/huggingface           # HuggingFace models
BODHI_EXEC_VARIANT=cpu                 # LLM runtime variant
```

---

## Multi-Tenant Configuration Changes

### New Environment Variables
```env
# Database
DATABASE_URL=postgres://user:pass@host:5432/bodhi    # App database (PostgreSQL)
DATABASE_URL=sqlite:{BODHI_HOME}/app.db              # App database (SQLite, single-tenant)

# Session database (separate connection pool)
SESSION_DB_URL=postgres://user:pass@session-host:5432/sessions
SESSION_DB_SCHEMA=sessions

# Cache
REDIS_URL=redis://cache-host:6379                    # Redis for multi-tenant cache
# If not set, uses in-memory cache (single-tenant)

# Runtime mode
BODHI_MULTI_TENANT=true                              # Enables multi-tenant behavior
# When true:
#   - Expects X-BodhiApp-Org header from reverse proxy
#   - Uses PostgreSQL (DATABASE_URL must be postgres://)
#   - Uses Redis for caching (REDIS_URL required)
#   - Local LLM routes return UnsupportedOperation
# When false (default):
#   - Single org from organizations table
#   - Can use SQLite or PostgreSQL
#   - In-memory cache
#   - All features enabled

# Platform (unchanged)
BODHI_AUTH_URL=https://auth.getbodhi.app
BODHI_AUTH_REALM=bodhi
BODHI_PORT=1135
BODHI_ENCRYPTION_KEY=...               # Master key for encrypting org secrets in DB
```

### Removed Configuration
- `BODHI_HOME` - Not needed for multi-tenant (no file storage)
- `HF_HOME` - Not needed for multi-tenant (no local models)
- `BODHI_EXEC_*` - Not needed for multi-tenant (no llama.cpp)
- `secrets_path` - Removed with SecretService

### SettingService Changes
- Remove: `secrets_path()`, `encryption_key()` (move to env vars)
- Remove: `app_db_path()`, `session_db_path()` (replaced by DATABASE_URL, SESSION_DB_URL)
- Keep: Auth URL methods, server config, logging
- Add: `database_url()`, `session_db_url()`, `redis_url()`, `is_multi_tenant()`

---

## Docker Deployment

### Current Dockerfiles (8 variants in devops/)
- `cpu.Dockerfile` - CPU-only (AMD64/ARM64)
- `cuda.Dockerfile` - NVIDIA CUDA GPU
- `rocm.Dockerfile` - AMD ROCm GPU
- `vulkan.Dockerfile`, `cann.Dockerfile`, `intel.Dockerfile`, `musa.Dockerfile`
- `app-binary.Dockerfile` - Binary builder

### Multi-Tenant Docker Strategy
**Decision**: Build args on existing Dockerfiles

```dockerfile
# In existing Dockerfiles, add:
ARG MULTI_TENANT=false

# Conditional PostgreSQL client libs
RUN if [ "$MULTI_TENANT" = "true" ]; then \
      apt-get install -y libpq-dev; \
    fi

# Conditional sqlx features
# At Cargo build time, sqlx::Any connects to whatever URL is provided
# No Cargo feature change needed - sqlx::Any handles both SQLite and PostgreSQL

# Multi-tenant doesn't need llama.cpp but we keep it (same binary, runtime gating)
```

### New: docker-compose for multi-tenant
```yaml
# docker-compose.multi-tenant.yml
services:
  traefik:
    image: traefik:v3
    # Wildcard subdomain routing
    # SSL termination
    # X-BodhiApp-Org header injection

  app:
    image: ghcr.io/bodhisearch/bodhiapp:latest-cpu
    environment:
      BODHI_MULTI_TENANT: "true"
      DATABASE_URL: postgres://bodhi:password@postgres:5432/bodhi
      SESSION_DB_URL: postgres://bodhi:password@postgres:5432/sessions
      REDIS_URL: redis://redis:6379
      BODHI_AUTH_URL: https://auth.getbodhi.app
      BODHI_AUTH_REALM: bodhi
      BODHI_ENCRYPTION_KEY: ${MASTER_KEY}
    deploy:
      replicas: 3  # Horizontal scaling

  postgres:
    image: postgres:16
    volumes:
      - pgdata:/var/lib/postgresql/data

  redis:
    image: redis:7-alpine

  # Keycloak is external (managed separately)
```

### Traefik Configuration
```yaml
# Dynamic routing for wildcard subdomains
http:
  routers:
    bodhi-ui:
      rule: "HostRegexp(`{org:[a-z0-9-]+}.getbodhi.app`) && PathPrefix(`/ui/`)"
      service: bodhi-frontend

    bodhi-api:
      rule: "HostRegexp(`{org:[a-z0-9-]+}.getbodhi.app`)"
      service: bodhi-backend
      middlewares:
        - inject-org-header

  middlewares:
    inject-org-header:
      headers:
        customRequestHeaders:
          X-BodhiApp-Org: "{{ .org }}"  # Extracted from subdomain
```

---

## NAPI Bindings Impact

### Current NAPI (lib_bodhiserver_napi)
- Exposes Rust server to Node.js
- Uses same AppService/DbService infrastructure
- Currently SQLite-only

### Multi-Tenant NAPI Support
- **Decision**: NAPI supports both modes
- NAPI initialization accepts DATABASE_URL (can be sqlite: or postgres:)
- Same sqlx::Any abstraction works for NAPI
- Session DB URL configurable
- Cache backend configurable

### NAPI Changes
- Update connection initialization to use AnyPool
- Pass DATABASE_URL and SESSION_DB_URL as config
- CacheService: default to in-memory for NAPI (single-process)

---

## Startup Flow Changes

### Current Startup (serve.rs)
```
1. Load SettingService from env + settings.yaml
2. Build AppService via AppServiceBuilder:
   a. Connect to SQLite (app.db)
   b. Run migrations
   c. Connect to SQLite (session.db)
   d. Run session migrations
   e. Create SecretService with encryption key
   f. Create all other services
3. Build SharedContext (LLM server management)
4. Build routes with middleware
5. Start HTTP server
```

### New Startup (multi-tenant)
```
1. Load SettingService from env (no settings.yaml for multi-tenant)
2. Build AppService via AppServiceBuilder:
   a. Connect to PostgreSQL via DATABASE_URL (AnyPool)
   b. Run PostgreSQL migrations
   c. Connect to PostgreSQL via SESSION_DB_URL (separate AnyPool)
   d. Run session migrations
   e. Connect to Redis via REDIS_URL
   f. Create CacheService (Redis-backed)
   g. Skip SecretService (removed)
   h. Create all other services
3. Build SharedContext (no-op in multi-tenant, or skip)
4. Build routes with org_resolution_middleware + auth_middleware
5. Start HTTP server
```

### New Startup (single-tenant, backwards-compatible)
```
1. Load SettingService from env + settings.yaml
2. Build AppService via AppServiceBuilder:
   a. Connect to SQLite via app_db_path (AnyPool with sqlite: URL)
   b. Run SQLite migrations
   c. Connect to SQLite via session_db_path
   d. Run session migrations
   e. Create CacheService (in-memory)
   f. Skip SecretService (removed, org info in DB)
   g. Create all other services
3. Build SharedContext (LLM server management)
4. Build routes with org_resolution_middleware + auth_middleware
5. Start HTTP server
```
