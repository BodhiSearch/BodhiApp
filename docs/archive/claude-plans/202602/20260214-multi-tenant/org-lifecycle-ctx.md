# Org Lifecycle & Audit Context

## Org Provisioning

### External Service: new.getbodhi.app
- Separate application handles org creation
- User submits org creation request
- Platform admin approves (manual approval, out of scope)
- On approval, provisioning service:
  1. Creates KC Organization via Admin API
  2. Creates KC Client for the org
  3. Configures client-scoped roles (admin, manager, power_user, user)
  4. Sets redirect URIs (https://<slug>.getbodhi.app/ui/auth/callback)
  5. Generates per-org encryption key
  6. Inserts row into `organizations` table
  7. Seeds initial org data (toolset configs, etc.)
  8. Makes requesting user the org admin in KC

### BodhiApp's Role
- **Consumer only**: Reads from organizations table
- Zero org management endpoints in BodhiApp
- Org config loaded into CacheService on first request
- Cache invalidated via event-driven mechanism (NATS or Redis pub/sub)

### Organization States
```
active     → Normal operation, all routes work
suspended  → Org temporarily disabled, all routes return 403
deleted    → Soft-deleted, all routes return 404
```

---

## Access Request Flow (Multi-Tenant)

### Current Flow (Single-Tenant)
1. User visits instance
2. If not registered → submits access request
3. Admin reviews and approves/denies
4. On approval → user gets role assigned in KC

### Multi-Tenant Flow
1. User visits `my-org.getbodhi.app`
2. If not a member of `my-org` → submits join request
3. Org admin reviews and approves/denies
4. On approval:
   - User added to KC Organization `my-org`
   - User gets client-scoped role in `bodhi-my-org` client
   - Access request record updated in DB with org_id

### Key Difference
- Access requests are **org-scoped** (already have org_id via the client/org mapping)
- No new endpoints needed - existing flow works because client = org

---

## Audit Logging (Deferred Implementation)

### Decision: NATS JetStream (out of current scope)
- Define event interface now
- Implement NATS integration later
- Initial release: optional structured logging to stdout

### Audit Event Interface
```rust
#[derive(Debug, Serialize)]
pub struct AuditEvent {
  pub timestamp: DateTime<Utc>,
  pub org_id: String,
  pub user_id: String,
  pub action: AuditAction,
  pub resource_type: String,
  pub resource_id: String,
  pub details: serde_json::Value,
  pub ip_address: Option<String>,
}

pub enum AuditAction {
  Create,
  Update,
  Delete,
  Login,
  Logout,
  TokenCreate,
  TokenRevoke,
  RoleChange,
  AccessRequestApprove,
  AccessRequestDeny,
  ToolsetExecute,
}
```

### Audit Trait
```rust
#[async_trait]
pub trait AuditService: Send + Sync + Debug {
  async fn log_event(&self, event: AuditEvent) -> Result<()>;
}

// Initial implementation: no-op or structured log
pub struct LogAuditService;

impl AuditService for LogAuditService {
  async fn log_event(&self, event: AuditEvent) -> Result<()> {
    tracing::info!(audit = true, org_id = %event.org_id, action = ?event.action, "audit event");
    Ok(())
  }
}

// Future: NATS JetStream implementation
pub struct NatsAuditService { /* ... */ }
```

### Where to Emit Audit Events
- Auth middleware: login/logout events
- Token routes: create/revoke API tokens
- Admin routes: role changes, access request decisions
- Toolset routes: toolset creation, execution
- User routes: alias management, model config changes

---

## Frontend Org Features

### Org Switcher UI
- Shows current org name in header/sidebar
- Dropdown lists orgs the user belongs to
- Switching navigates to `<other-org>.getbodhi.app`
- Org list fetched from `GET /api/orgs/user-memberships`

### New API Endpoints for Frontend
```
GET /api/orgs/current
  → Returns: { org_id, slug, display_name, status }
  → Source: OrgContext (already resolved by middleware)

GET /api/orgs/user-memberships
  → Returns: [{ org_id, slug, display_name, role }]
  → Source: Query KC Organizations API for user's org memberships
  → Cached per user session
```

### Frontend Changes Summary
1. **API calls**: No URL changes (same subdomain, same paths)
2. **Org context**: Fetch current org info for display
3. **Org switcher**: New UI component + navigation
4. **Login**: Works per-org automatically (different KC client per subdomain)
5. **Session**: Naturally scoped per subdomain (cookie isolation)
