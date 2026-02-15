# Phase 5: Docker & Deployment

## Goal
Update Docker build and deployment infrastructure to support multi-tenant hosted mode with PostgreSQL, Redis, and Traefik.

## Prerequisites
- Phase 4 complete (all backend multi-tenancy code working)

---

## Step 1: Update Existing Dockerfiles

### Build Arg Addition
All Dockerfiles in `devops/` get:
```dockerfile
ARG MULTI_TENANT=false
```

### PostgreSQL Client Libraries
```dockerfile
# In the runtime stage of each Dockerfile
ARG MULTI_TENANT
RUN if [ "$MULTI_TENANT" = "true" ]; then \
      apt-get update && apt-get install -y --no-install-recommends libpq5 && \
      rm -rf /var/lib/apt/lists/*; \
    fi
```

Note: `libpq5` is the runtime library (not `libpq-dev` which is for compilation). Since sqlx::Any with the `postgres` feature compiles PostgreSQL support into the binary unconditionally, the PG client library is always needed. Simplification: always include `libpq5` regardless of MULTI_TENANT flag.

### Simplified Approach
```dockerfile
# Always include PostgreSQL client library (tiny footprint)
RUN apt-get update && apt-get install -y --no-install-recommends libpq5 && \
    rm -rf /var/lib/apt/lists/*
```

---

## Step 2: Multi-Tenant Environment Configuration

### defaults.yaml for Multi-Tenant
```yaml
# devops/defaults-hosted.yaml
BODHI_MULTI_TENANT: "true"
BODHI_PORT: "1135"
BODHI_LOG_LEVEL: "info"
# DATABASE_URL, SESSION_DB_URL, REDIS_URL set via docker-compose env
```

### Runtime Configuration
The Docker image determines mode based on environment variables at startup:
- `BODHI_MULTI_TENANT=true` → PostgreSQL + Redis required
- `BODHI_MULTI_TENANT=false` (default) → SQLite + in-memory cache

---

## Step 3: Docker Compose for Multi-Tenant

### docker-compose.multi-tenant.yml
```yaml
version: '3.8'

services:
  traefik:
    image: traefik:v3.2
    command:
      - "--providers.docker=true"
      - "--providers.docker.exposedbydefault=false"
      - "--entryPoints.web.address=:80"
      - "--entryPoints.websecure.address=:443"
      - "--certificatesresolvers.letsencrypt.acme.tlschallenge=true"
      - "--certificatesresolvers.letsencrypt.acme.email=admin@getbodhi.app"
      - "--certificatesresolvers.letsencrypt.acme.storage=/letsencrypt/acme.json"
    ports:
      - "80:80"
      - "443:443"
    volumes:
      - /var/run/docker.sock:/var/run/docker.sock:ro
      - letsencrypt:/letsencrypt
    networks:
      - bodhi

  bodhi-frontend:
    image: ghcr.io/bodhisearch/bodhi-frontend:latest
    labels:
      - "traefik.enable=true"
      - "traefik.http.routers.frontend.rule=HostRegexp(`{org:[a-z0-9-]+}.getbodhi.app`) && PathPrefix(`/ui/`)"
      - "traefik.http.routers.frontend.entrypoints=websecure"
      - "traefik.http.routers.frontend.tls.certresolver=letsencrypt"
      - "traefik.http.routers.frontend.tls.domains[0].main=getbodhi.app"
      - "traefik.http.routers.frontend.tls.domains[0].sans=*.getbodhi.app"
    networks:
      - bodhi

  bodhi-api:
    image: ghcr.io/bodhisearch/bodhiapp:latest-cpu
    environment:
      BODHI_MULTI_TENANT: "true"
      DATABASE_URL: "postgres://bodhi:${DB_PASSWORD}@postgres:5432/bodhi"
      SESSION_DB_URL: "postgres://bodhi:${DB_PASSWORD}@postgres:5432/bodhi_sessions"
      REDIS_URL: "redis://redis:6379"
      BODHI_AUTH_URL: "${AUTH_URL}"
      BODHI_AUTH_REALM: "${AUTH_REALM}"
      BODHI_ENCRYPTION_KEY: "${MASTER_ENCRYPTION_KEY}"
      BODHI_PORT: "1135"
    labels:
      - "traefik.enable=true"
      - "traefik.http.routers.api.rule=HostRegexp(`{org:[a-z0-9-]+}.getbodhi.app`) && !PathPrefix(`/ui/`)"
      - "traefik.http.routers.api.entrypoints=websecure"
      - "traefik.http.routers.api.tls.certresolver=letsencrypt"
      - "traefik.http.middlewares.inject-org.headers.customrequestheaders.X-BodhiApp-Org={{ index .Labels \"traefik.frontend.rule\" }}"
      # Note: Traefik header injection from regex capture requires plugin or middleware
      # Alternative: Use Traefik plugin or custom middleware for subdomain extraction
    deploy:
      replicas: 3
    networks:
      - bodhi
    depends_on:
      postgres:
        condition: service_healthy
      redis:
        condition: service_healthy

  postgres:
    image: postgres:16-alpine
    environment:
      POSTGRES_USER: bodhi
      POSTGRES_PASSWORD: "${DB_PASSWORD}"
      POSTGRES_DB: bodhi
    volumes:
      - pgdata:/var/lib/postgresql/data
      - ./init-db.sql:/docker-entrypoint-initdb.d/init.sql
    healthcheck:
      test: ["CMD-SHELL", "pg_isready -U bodhi"]
      interval: 5s
      timeout: 5s
      retries: 5
    networks:
      - bodhi

  redis:
    image: redis:7-alpine
    command: redis-server --maxmemory 256mb --maxmemory-policy allkeys-lru
    healthcheck:
      test: ["CMD", "redis-cli", "ping"]
      interval: 5s
      timeout: 3s
      retries: 5
    volumes:
      - redisdata:/data
    networks:
      - bodhi

volumes:
  pgdata:
  redisdata:
  letsencrypt:

networks:
  bodhi:
    driver: bridge
```

### init-db.sql
```sql
-- Create session database
CREATE DATABASE bodhi_sessions OWNER bodhi;
```

---

## Step 4: Traefik Org Header Injection

### Challenge
Traefik doesn't natively extract regex capture groups from Host header into custom headers. Options:

### Option A: Traefik Plugin
Use a custom Traefik plugin or the `rewriteHeaders` middleware to extract subdomain.

### Option B: App-Level Extraction
Instead of Traefik injecting `X-BodhiApp-Org`, the app extracts org from the `Host` header directly:

```rust
// In org_resolution_middleware
fn extract_org_from_host(headers: &HeaderMap) -> Option<String> {
  let host = headers.get("Host")?.to_str().ok()?;
  // host = "my-org.getbodhi.app" or "my-org.getbodhi.app:443"
  let parts: Vec<&str> = host.split('.').collect();
  if parts.len() >= 3 && parts[1..].join(".").starts_with("getbodhi.app") {
    Some(parts[0].to_string())
  } else {
    None
  }
}
```

**Recommended**: Option B (app-level extraction). Simpler, no Traefik plugin dependency. The app already strips injected headers for security anyway.

### Configuration
```env
# Domain pattern for org extraction
BODHI_ORG_DOMAIN=getbodhi.app
# In single-tenant, this is not set → fall back to DB lookup
```

---

## Step 5: Makefile Targets

### New Targets
```makefile
# Makefile.docker.mk additions

docker.dev.hosted: ## Build multi-tenant hosted image
	@$(MAKE) -C devops dev.hosted BUILD_VARIANT=$${BUILD_VARIANT:-development}

docker.compose.hosted: ## Run multi-tenant stack locally
	docker compose -f docker-compose.multi-tenant.yml up -d

docker.compose.hosted.down: ## Stop multi-tenant stack
	docker compose -f docker-compose.multi-tenant.yml down

docker.compose.hosted.logs: ## View multi-tenant stack logs
	docker compose -f docker-compose.multi-tenant.yml logs -f
```

---

## Step 6: Health Check Updates

### Enhanced Health Endpoint
```rust
// /health endpoint should verify all backends
pub async fn health_handler(
  State(state): State<Arc<dyn RouterState>>,
) -> Result<impl IntoResponse, ApiError> {
  let mut status = json!({ "status": "ok" });

  // Check database
  if let Err(e) = state.app_service().db_service().health_check().await {
    status["database"] = json!({ "status": "error", "message": e.to_string() });
  } else {
    status["database"] = json!({ "status": "ok" });
  }

  // Check Redis (if multi-tenant)
  if state.app_service().setting_service().is_multi_tenant() {
    // Redis health via cache service
    status["cache"] = json!({ "status": "ok" });
  }

  Ok(Json(status))
}
```

---

## Step 7: Environment File Template

### .env.hosted.template
```env
# Database
DB_PASSWORD=change-me-in-production
DATABASE_URL=postgres://bodhi:${DB_PASSWORD}@postgres:5432/bodhi
SESSION_DB_URL=postgres://bodhi:${DB_PASSWORD}@postgres:5432/bodhi_sessions

# Cache
REDIS_URL=redis://redis:6379

# Auth
AUTH_URL=https://auth.getbodhi.app
AUTH_REALM=bodhi

# Security
MASTER_ENCRYPTION_KEY=generate-a-secure-key

# App
BODHI_MULTI_TENANT=true
BODHI_PORT=1135
BODHI_ORG_DOMAIN=getbodhi.app
BODHI_LOG_LEVEL=info
```

---

## Deliverable
- Updated Dockerfiles with PostgreSQL client library
- docker-compose.multi-tenant.yml with full stack
- Traefik routing for wildcard subdomains
- App-level org extraction from Host header
- Health check endpoint verifying all backends
- Makefile targets for hosted development
- Environment template for hosted deployment
- Working local multi-tenant stack via docker-compose

## Testing Checklist
- [ ] Docker image builds with `--build-arg MULTI_TENANT=true`
- [ ] docker-compose stack starts (postgres, redis, traefik, app)
- [ ] Health endpoint returns status for all backends
- [ ] Subdomain routing resolves correct org
- [ ] API requests with org subdomain return org-scoped data
- [ ] Frontend served via Traefik at /ui/ paths
- [ ] Horizontal scaling: 3 replicas share state correctly
