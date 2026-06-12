# Auth & Keycloak Context

## Current Auth Architecture

### AppRegInfo (to be replaced)
- **Struct**: `{ client_id: String, client_secret: String }`
- **Storage**: Encrypted YAML file via SecretService (AES-256-GCM + PBKDF2)
- **Path**: `{BODHI_HOME}/secrets.yaml`
- **Usage**: OAuth2 token exchange, token refresh, audience validation, login redirect
- **Migration target**: Merged into `organizations` table in DB

### AppStatus (to be migrated)
- **Values**: `NotReady | Setup | ResourceAdmin | Ready`
- **Storage**: Same encrypted secrets.yaml via SecretService
- **Usage**: Controls middleware behavior (reject requests during Setup)
- **Migration target**: Column on `organizations` table or separate state management

### Current Auth Flow
```
1. User visits /ui/login
2. Login handler reads AppRegInfo.client_id from SecretService
3. Constructs OAuth2 authorization URL with PKCE
4. User redirects to Keycloak login
5. KC redirects back to /ui/auth/callback with authorization code
6. Callback handler exchanges code for tokens using AppRegInfo credentials
7. Tokens stored in session (access_token, refresh_token, user_id)
8. Subsequent requests: auth_middleware extracts token from session
9. If expired: refresh using AppRegInfo.client_id + client_secret
```

### Current Token Service Flow
```
Bearer token request:
  1. Extract Authorization header
  2. If bodhiapp_* prefix → DB API token lookup
  3. Else → External client token validation:
     a. Validate issuer against BODHI_AUTH_URL
     b. Validate audience contains AppRegInfo.client_id
     c. Pre-exchange: validate access_request in DB
     d. Token exchange via KC (RFC 8693)
     e. Post-exchange: verify claims
     f. Cache exchanged token by SHA-256 digest
```

---

## Multi-Tenant Auth Changes

### Per-Org Client Resolution
```
Multi-tenant flow:
  1. Traefik extracts org slug from subdomain
  2. Injects X-BodhiApp-Org: <slug> header
  3. Auth middleware reads org slug from header
  4. Looks up org config from CacheService (Redis-backed)
  5. Cache miss → query organizations table → cache result
  6. Uses org's client_id + client_secret for all OAuth operations

Single-tenant flow:
  1. Auth middleware reads org from organizations table (single row)
  2. Cached in CacheService (in-memory)
  3. Same as multi-tenant but no Traefik/subdomain
```

### OrgContext Structure
```rust
pub struct OrgContext {
  pub org_id: String,
  pub org_slug: String,
  pub kc_client_id: String,
  pub client_secret: String,  // Decrypted
  pub encryption_key: Vec<u8>,
  pub status: OrgStatus,
}
```

### Middleware Changes

#### New: Org Resolution Middleware (runs before auth_middleware)
```rust
// Extracts org from X-BodhiApp-Org header
// Resolves full OrgContext from cache/DB
// Injects as Axum Extension<OrgContext>
// Rejects if org not found or suspended
```

#### Modified: auth_middleware
```rust
// Before: reads AppRegInfo from SecretService
// After: reads OrgContext from Extension (already resolved)
// Uses org.kc_client_id and org.client_secret for all OAuth ops
```

#### Modified: Login handler
```rust
// Before: AppRegInfo from SecretService
// After: OrgContext from Extension
// OAuth redirect uses org-specific client_id
// Callback uses org-specific credentials for code exchange
```

### Keycloak Configuration
- **Single realm**: All orgs in one KC realm
- **KC Organizations**: KC 26+ feature for org management
- **Per-org client**: Each org has its own KC client with:
  - Client-scoped roles (Admin, Manager, PowerUser, User)
  - Configured redirect URIs (`https://<slug>.getbodhi.app/ui/auth/callback`)
  - Client secret for confidential access
- **Auth URL**: Same for all orgs (`BODHI_AUTH_URL/realms/<realm>`)
- **Token URL**: Same for all orgs (single realm endpoint)

### KC Client Mapping
```
organizations table:
  slug: "my-org"
  kc_client_id: "bodhi-my-org"

KC:
  Organization: "my-org"
  Client: "bodhi-my-org"
    Roles: admin, manager, power_user, user
    Redirect URIs: https://my-org.getbodhi.app/ui/auth/callback
```

---

## Header Changes

### Current Headers (injected by auth_middleware)
```
X-BodhiApp-Token: <access_token>
X-BodhiApp-User-Id: <sub claim>
X-BodhiApp-Role: <admin|manager|power_user|user>
X-BodhiApp-Username: <preferred_username>
X-BodhiApp-Scope: <resource_scope>
X-BodhiApp-Access-Request-Id: <id>
X-BodhiApp-Azp: <authorized_party>
```

### New Headers
```
X-BodhiApp-Org: <org_slug>       -- Injected by Traefik (multi-tenant) or auth_middleware (single-tenant)
X-BodhiApp-Org-Id: <org_id>     -- Injected by org resolution middleware after DB lookup
```

### New Extractor
```rust
pub struct ExtractOrgId(pub String);
// Extracts from X-BodhiApp-Org-Id header
// Required for all org-scoped routes
```

---

## SecretService Removal Plan

### What SecretService currently stores:
1. `app_reg_info` (AppRegInfo) → Moves to organizations table
2. `app_status` (AppStatus) → Moves to organizations table or config

### What uses SecretService:
- `auth_middleware` → Will use OrgContext from Extension
- `token_service` → Will use OrgContext
- `login.rs` (auth handlers) → Will use OrgContext
- `routes_setup` (initial setup) → Will write to organizations table directly
- `access_request_service` → Will get client_id from OrgContext

### Removal steps:
1. Add organizations table with client credentials
2. Create OrgContext resolution path (cache/DB)
3. Update all SecretService consumers to use OrgContext
4. Remove SecretService trait, impl, and encrypted file handling
5. Remove SecretServiceExt trait
6. Remove from AppService trait and DefaultAppService
7. Remove keyring dependencies from Cargo.toml

### Platform-level secrets (env vars only after removal):
- `BODHI_AUTH_URL` - Keycloak server URL
- `BODHI_AUTH_REALM` - Keycloak realm name
- `DATABASE_URL` - PostgreSQL connection string
- `SESSION_DB_URL` - Session DB connection string
- `BODHI_ENCRYPTION_KEY` - Master encryption key (for encrypting org secrets in DB)

---

## Session Org-Scoping

### Cookie Isolation
- Cookies are naturally scoped to subdomain
- `my-org.getbodhi.app` cookie ≠ `other-org.getbodhi.app` cookie
- SameSite=Strict prevents cross-subdomain cookie sharing
- Each org login creates independent session

### KC SSO Behavior
- All orgs share one KC realm
- User logs into org-alpha → KC SSO session created
- User visits org-beta → KC recognizes SSO session → auto-authenticates
- Result: seamless SSO across orgs, but separate app-level sessions per org
- Each org session has its own access/refresh tokens with org-specific client claims
