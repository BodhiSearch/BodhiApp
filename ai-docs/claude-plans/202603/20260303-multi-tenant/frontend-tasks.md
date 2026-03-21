# Frontend Tasks for Multi-Tenant Support

> **Purpose**: Deferred frontend work items needed after backend multi-tenancy is complete.
> These tasks are out of scope for the backend implementation plan but must be implemented
> before multi-tenant mode is user-facing.
>
> **Created**: 2026-03-03
> **Status**: Collecting. Not started.
> **Update when**: Backend changes create new frontend requirements.

---

## Priority 1: Must-Have for Multi-Tenant Launch

### F1: Hide LLM Features in Multi-Tenant Mode
- **Depends on**: Backend Phase 5 (deployment mode feature gating)
- **Backend signal**: `GET /bodhi/v1/info` returns `deployment_mode: "multi"` or `"standalone"`
- **UI changes**:
  - Hide "Models" page (model management, downloads, aliases)
  - Hide "Chat" page (direct LLM inference)
  - Hide "Download Models" setup step
  - Hide "LLM Engine" setup step (exec variant selection)
  - Hide model-related sections in Settings
- **Approach**: Check deployment mode from info endpoint, conditionally render navigation items

### F2: Error Handling for 501 (Feature Disabled)
- **Depends on**: Backend Phase 5
- **Backend signal**: LLM routes return 501 Not Implemented with descriptive error body
- **UI changes**: Show friendly message like "This feature is not available in hosted mode"
- **Note**: If routes are conditionally unregistered, frontend gets 404 instead of 501. Handle both.

### F3: TypeScript Client Regeneration
- **Depends on**: All backend API changes complete
- **Action**: Run `make build.ts-client` to regenerate `@bodhiapp/ts-client`
- **Impact**: Type changes for Tenant (was AppInstance), new fields in API responses
- **Note**: Must be done before any frontend component work

### F4: Setup Flow Updates
- **Depends on**: Backend Phase 1 (TenantService rename)
- **UI changes**:
  - API responses use `Tenant` types instead of `AppInstance`
  - Setup endpoint returns tenant_id in response
  - Update API hook types to match new response shapes
- **Files**: `crates/bodhi/src/app/ui/setup/` directory

---

## Priority 2: Multi-Tenant Auth Flow UI

### F5: Tenant Selector UI
- **Depends on**: Backend two-phase auth endpoints (deferred from this plan)
- **Backend signal**: `GET /bodhi/v1/tenants` returns list of tenants user has access to
- **UI changes**:
  - After platform login, show tenant selection dropdown/list
  - Selected tenant triggers tenant-specific auth flow
  - Active tenant stored in cookie/session
  - Tenant name displayed in header/sidebar
- **UX reference**: GitHub org switcher, Vercel team selector

### F6: Two-Phase Auth Flow
- **Depends on**: Backend two-phase auth endpoints
- **Flow**:
  1. User clicks "Login" → redirected to Keycloak via platform client (BODHI_MULTITENANT_CLIENT_ID)
  2. After platform auth → returned to app → see tenant selector (F5)
  3. User selects tenant → app initiates auth against tenant's Keycloak client
  4. After tenant auth → returned to app → normal usage scoped to tenant
- **Session management**: Platform token for tenant listing, tenant token for API calls

### F7: Tenant Context in API Calls
- **Depends on**: Backend Phase 2 (AuthContext with tenant_id)
- **Behavior**: All API calls automatically include the active tenant's auth token
- **Cookie management**: Active tenant stored in cookie, auth token scoped to that tenant
- **Switching tenants**: Re-authenticates against new tenant's client, SSO session reuse means no password re-entry

---

## Priority 3: Nice-to-Have

### F8: App Info Endpoint Updates
- **Depends on**: Backend changes to info endpoint
- **Changes**: Info response includes deployment_mode, tenant_id, tenant display name
- **Usage**: Header shows tenant name, settings show deployment context

### F9: Multi-Tenant Settings UI
- **Depends on**: Future tenant_settings table (if created)
- **UI**: Per-tenant settings page for tenant admins
- **Deferred**: No per-tenant settings exist yet (decision D9)

---

## Technical Notes

- Frontend is Next.js 14 in `crates/bodhi/src/`
- Uses `@bodhiapp/ts-client` for API types (auto-generated from OpenAPI)
- React Query for API state management
- After UI changes: `make build.ui-rebuild` required for E2E tests
- Component tests: `cd crates/bodhi && npm run test`
