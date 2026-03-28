# Plan: Rename "Workspace" → "Organization" Across the Codebase

## Context

The term "workspace" is used throughout frontend UI and some documentation to describe what is internally called a "tenant". As the product evolves toward enterprise/paid offerings using Keycloak's organization feature, "organization" is the correct term. "Tenant" remains correct for internal/technical representation (API routes, variable names, hook names, data-testid attributes, etc.) — only "workspace" usages get renamed.

Full audit confirms: the frontend code already uses "tenant" for all technical identifiers (URLs, hook names, query keys, `data-testid`, route constants). The only "workspace" usages are display text strings and documentation — exactly what needs renaming.

No backwards compatibility required.

---

## Scope: What Changes

### Rule
- `workspace` / `workspaces` → `organization` / `organizations` everywhere it refers to BodhiApp's tenant concept
- `tenant` / `tenants` — unchanged (acceptable internal term)
- Cargo `[workspace]` config — unchanged (Rust package manager concept, unrelated)

---

## Files to Modify

### 1. `crates/bodhi/src/app/setup/tenants/page.tsx` — 7 changes

| Line | Before | After |
|------|--------|-------|
| 71 | `"Setting up your workspace..."` | `"Setting up your organization..."` |
| 78 | `Create Workspace` | `Create Organization` |
| 79 | `Set up your new workspace to get started with Bodhi` | `Set up your new organization to get started with Bodhi` |
| 89 | `Workspace Name` | `Organization Name` |
| 95 | `placeholder="My Workspace"` | `placeholder="My Organization"` |
| 108 | `placeholder="A brief description of your workspace"` | `placeholder="A brief description of your organization"` |
| 113 | `'Create Workspace'` | `'Create Organization'` |

### 2. `crates/bodhi/src/app/login/page.tsx` — 5 changes

| Line | Before | After |
|------|--------|-------|
| 146 | `showSuccess('Workspace', 'Already a member of this workspace')` | `showSuccess('Organization', 'Already a member of this organization')` |
| 219 | `Active workspace: <strong>{activeTenant.name}</strong>` | `Active organization: <strong>{activeTenant.name}</strong>` |
| 262 | `title="Connect to Workspace"` | `title="Connect to Organization"` |
| 293 | `title="Select Workspace"` | `title="Select Organization"` |
| 296 | `Choose a workspace to continue` | `Choose an organization to continue` |

### 3. `crates/bodhi/src/app/users/page.tsx` — 1 change

| Line | Before | After |
|------|--------|-------|
| 106 | `Share this link to invite users to your workspace` | `Share this link to invite users to your organization` |

### 4. `crates/bodhi/src/app/login/page.test.tsx` — 2 changes (UI assertion text)

| Line | Before | After |
|------|--------|-------|
| 346 | `'Select Workspace'` | `'Select Organization'` |
| 384 | `'Select Workspace'` | `'Select Organization'` |

> Lines 366–367 (`name: 'Workspace 1'`, `name: 'Workspace 2'`) and 389–390 are **tenant name fixture values**, not UI labels — leave unchanged. The buttons in the select state show `tenant.name` dynamically, so these assertions are checking data, not static UI text.

### 5. `ai-docs/03-crates/routes_app.md` — 2 changes

| Line | Before | After |
|------|--------|-------|
| 75 | `POST /api/create/workspace`: Create new workspace | `POST /api/create/organization`: Create new organization |
| 306 | `Workspace Management: Multi-workspace support` | `Organization Management: Multi-organization support` |

> Note: Line 75 references a non-existent endpoint (actual route is `POST /tenants`) — update the display label but leave as-is structurally since this is existing doc content.

### 6. `getbodhi.app/src/docs/developer/app-access-requests.md` — 1 change

| Line | Before | After |
|------|--------|-------|
| 24 | `Future resource types will include workspaces and agents.` | `Future resource types will include organizations and agents.` |

---

## What Does NOT Change

| Category | Examples | Reason |
|----------|----------|--------|
| API routes/URLs | `/tenants`, `/setup/tenants` | Tenant is the internal term |
| Hook/function names | `useCreateTenant`, `useTenantActivate`, `useListTenants` | Tenant is the internal term |
| Route constants | `ROUTE_SETUP_TENANTS` | Tenant is the internal term |
| Query key factories | `tenantKeys.all` | Tenant is the internal term |
| `data-testid` attributes | `tenant-name-input`, `create-tenant-button` | Internal test selectors |
| Backend Rust code | `CreateTenantRequest`, `TenantListItem` | Tenant is the internal term |
| Test fixture names | `name: 'Workspace 1'` (line 366–367) | Dynamic tenant name data, not UI labels |
| Cargo `[workspace]` config | All `Cargo.toml` files | Rust package manager concept |
| Dev container config | `.devcontainer/*/devcontainer.json` | VS Code infrastructure setting |

---

## Verification

1. `cd crates/bodhi && npm test` — all Vitest component tests pass (especially `login/page.test.tsx`)
2. Visual check in dev mode: `cd crates/bodhi && npm run dev`
   - `/setup/tenants` in multi-tenant mode: see "Create Organization", "Organization Name", "My Organization" placeholder
   - `/login`: see "Select Organization", "Connect to Organization", "Active organization:", toast "Already a member of this organization"
   - `/users`: see invite link helper text "your organization"
