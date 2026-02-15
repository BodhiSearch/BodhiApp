# Middleware & Routing Context

## Current Middleware Chain

### Route Composition (crates/routes_app/src/routes.rs)
```
Router::new()
  .merge(public_apis)           // /ping, /health - no middleware
  .merge(optional_auth)         // /ui/user - inject_optional_auth_info
  .merge(protected_apis)        // All authenticated routes
    .merge(user_apis)           // /v1/chat/completions, etc. (role=User)
    .merge(user_session_apis)   // Session-only routes (no API tokens)
    .merge(toolset_exec_apis)   // Toolset execution (custom middleware)
    .merge(power_user_apis)     // /api/models POST, etc. (role=PowerUser)
    .merge(admin_session_apis)  // /api/users, etc. (role=Admin)
    .route_layer(auth_middleware)  // Shared auth for all protected
  .layer(session_layer)         // tower-sessions management
  .layer(canonical_url_middleware)
  .layer(trace_layer)
```

### Auth Middleware Flow (auth_middleware.rs)
1. `remove_app_headers(&mut req)` - Strip injected headers (security)
2. Check AppStatus - reject if Setup
3. Bearer token path:
   - `bodhiapp_*` prefix → DB API token lookup
   - Other → External client token exchange
   - Inject: X-BodhiApp-Token, X-BodhiApp-Scope, X-BodhiApp-User-Id
4. Session token path:
   - Same-origin check (CSRF protection)
   - Extract access_token from session
   - Auto-refresh if expired (with distributed lock)
   - Inject: X-BodhiApp-Token, X-BodhiApp-Role, X-BodhiApp-Username, X-BodhiApp-User-Id
5. No auth → AuthError::InvalidAccess

### API Auth Middleware (api_auth_middleware)
- Role-based check: `user_role.has_access_to(required_role)`
- Scope-based check: TokenScope or UserScope matching
- Applied per route group with different role/scope requirements

---

## Multi-Tenant Middleware Changes

### New Middleware: Org Resolution
**Position**: Before auth_middleware, after trace/session layers

```rust
pub async fn org_resolution_middleware(
  State(state): State<Arc<dyn RouterState>>,
  mut req: Request,
  next: Next,
) -> Result<Response, ApiError> {
  // 1. Extract org slug from X-BodhiApp-Org header (injected by Traefik)
  //    OR from organizations table (single-tenant: single row)
  let org_slug = extract_org_slug(&req, &state)?;

  // 2. Resolve OrgContext from CacheService
  let org_context = state.app_service()
    .cache_service()
    .get::<OrgContext>(&format!("org:{}", org_slug))
    .or_else(|| {
      // Cache miss: query DB, populate cache
      let org = state.app_service().db_service().get_org_by_slug(&org_slug)?;
      state.app_service().cache_service().set(&format!("org:{}", org_slug), &org_context);
      org_context
    })?;

  // 3. Validate org status
  if org_context.status != OrgStatus::Active {
    return Err(ApiError::OrgSuspended);
  }

  // 4. Inject as Extension and header
  req.extensions_mut().insert(org_context.clone());
  req.headers_mut().insert("X-BodhiApp-Org-Id", org_context.org_id.parse()?);

  Ok(next.run(req).await)
}
```

### Modified Middleware Chain
```
Router::new()
  .merge(public_apis)           // /ping, /health - no org resolution needed
  .merge(org_scoped_routes)     // Everything org-scoped
    .merge(optional_auth)
    .merge(protected_apis)
    .route_layer(auth_middleware)           // Uses OrgContext for credentials
    .route_layer(org_resolution_middleware)  // NEW: resolves org from header
  .layer(session_layer)
  .layer(canonical_url_middleware)
  .layer(trace_layer)
```

### Modified Auth Middleware
```rust
pub async fn auth_middleware(
  session: Session,
  org_ctx: Extension<OrgContext>,    // NEW: from org_resolution_middleware
  State(state): State<Arc<dyn RouterState>>,
  mut req: Request,
  next: Next,
) -> Result<Response, ApiError> {
  remove_app_headers(&mut req);

  // Use org_ctx.kc_client_id instead of SecretService.app_reg_info().client_id
  // Use org_ctx.client_secret instead of SecretService.app_reg_info().client_secret

  // Bearer token: validate audience against org_ctx.kc_client_id
  // Session token: refresh using org_ctx credentials
  // ...
}
```

### Single-Tenant Org Injection
For single-tenant mode (no Traefik, no X-BodhiApp-Org header):
- org_resolution_middleware queries organizations table (single row)
- Caches the result
- Injects same Extension<OrgContext>
- No behavioral difference in downstream middleware/handlers

---

## New Extractors

### ExtractOrgId
```rust
pub struct ExtractOrgId(pub String);

impl<S> FromRequestParts<S> for ExtractOrgId {
  type Rejection = ApiError;

  async fn from_request_parts(parts: &mut Parts, _state: &S) -> Result<Self, Self::Rejection> {
    let org_id = parts.headers
      .get("X-BodhiApp-Org-Id")
      .ok_or(HeaderExtractionError::Missing { header: "X-BodhiApp-Org-Id" })?
      .to_str()?;
    Ok(ExtractOrgId(org_id.to_string()))
  }
}
```

### ExtractOrgContext (for handlers needing full org info)
```rust
pub struct ExtractOrgContext(pub OrgContext);

impl<S> FromRequestParts<S> for ExtractOrgContext {
  type Rejection = ApiError;

  async fn from_request_parts(parts: &mut Parts, _state: &S) -> Result<Self, Self::Rejection> {
    let org_ctx = parts.extensions
      .get::<OrgContext>()
      .ok_or(ApiError::OrgContextMissing)?
      .clone();
    Ok(ExtractOrgContext(org_ctx))
  }
}
```

---

## Route Changes for Multi-Tenant

### Routes That Need Org Scoping
All routes behind `protected_apis` automatically get org context from middleware.

### Routes That Stay Global
- `GET /ping` - Health check
- `GET /health` - Health check
- Setup routes (initial org registration) - special handling

### New Routes
- `GET /api/orgs/current` - Returns current org info (from OrgContext)
- `GET /api/orgs/user-memberships` - Returns orgs the current user belongs to (for org switcher)

### Modified Routes
All existing routes: handlers extract `ExtractOrgId` and pass to services.
No URL structure changes (org determined by subdomain, not path).
