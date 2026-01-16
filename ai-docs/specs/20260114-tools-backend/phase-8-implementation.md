# Phase 8 - Tools Configuration UI Implementation

> Completed: January 2026

## Summary

Created `/ui/tools` pages for tool configuration, integrated into setup flow as step 5, and added navigation under Settings group. The setup flow (`/ui/setup/tools`) and config page (`/ui/tools/edit`) use separate components optimized for their different use cases. Includes admin controls for app-level enable/disable and user controls for API key management.

## Architecture

```
Setup Flow (7 steps):
Welcome -> Resource Admin -> Download Models -> API Models -> Tools -> Browser Extension -> Complete
                                                              ^^^^^
                                                          STEP 5

Separate Components (no sharing):
┌─────────────────────────────────────┐  ┌─────────────────────────────────────┐
│  /ui/tools/ToolConfigForm.tsx       │  │  /ui/setup/tools/SetupToolsForm.tsx │
│  - Fetches state, shows skeleton    │  │  - Renders optimistically           │
│  - Admin-only app toggle            │  │  - App toggle always visible        │
│  - Clear API key button             │  │  - Auto-enables on API key entry    │
└─────────────────────────────────────┘  └─────────────────────────────────────┘
         │                                        │
         ▼                                        ▼
┌───────────────────────┐              ┌─────────────────────┐
│ /ui/tools/edit?toolid │              │ /ui/setup/tools     │
└───────────────────────┘              └─────────────────────┘
```

## Files Created/Modified

### Backend (Rust)

| File | Changes |
|------|---------|
| `crates/services/src/db/service.rs` | Added `delete_user_tool_config` trait method and implementation |
| `crates/services/src/tool_service.rs` | Added `delete_user_tool_config` trait method and implementation |
| `crates/services/src/test_utils/db.rs` | Added `delete_user_tool_config` to test wrapper |
| `crates/routes_app/src/routes_tools.rs` | Added `DELETE /tools/:tool_id/config` endpoint; app-level check on PUT/POST endpoints |

### Frontend (TypeScript/React)

| File | Description |
|------|-------------|
| `crates/bodhi/src/hooks/useTools.ts` | API hooks for tool management (no retry on 404) |
| `crates/bodhi/src/app/ui/tools/ToolConfigForm.tsx` | Config page form - fetches state, shows skeleton, admin controls |
| `crates/bodhi/src/app/ui/tools/page.tsx` | Tools list page |
| `crates/bodhi/src/app/ui/tools/edit/page.tsx` | Tool configuration page |
| `crates/bodhi/src/app/ui/setup/tools/SetupToolsForm.tsx` | Setup form - optimistic rendering, auto-enable on API key |
| `crates/bodhi/src/app/ui/setup/tools/page.tsx` | Setup step 5 page |
| `crates/bodhi/src/app/ui/setup/constants.ts` | Updated to 7 steps |
| `crates/bodhi/src/app/ui/setup/components/SetupProvider.tsx` | Added tools path mapping |
| `crates/bodhi/src/app/ui/setup/api-models/page.tsx` | Changed next step to tools |
| `crates/bodhi/src/hooks/use-navigation.tsx` | Added Tools to Settings group |
| `crates/bodhi/src/lib/constants.ts` | Added `ROUTE_SETUP_TOOLS` |

### Tests

| File | Description |
|------|-------------|
| `crates/bodhi/src/hooks/useTools.test.ts` | Hook unit tests |
| `crates/bodhi/src/app/ui/tools/page.test.tsx` | List page tests |
| `crates/bodhi/src/app/ui/tools/edit/page.test.tsx` | Edit page tests |
| `crates/bodhi/src/app/ui/setup/tools/page.test.tsx` | Setup page tests |
| `crates/bodhi/src/test-utils/msw-v2/handlers/tools.ts` | MSW mock handlers |

### E2E Tests

| File | Description |
|------|-------------|
| `tests-js/pages/SetupToolsPage.mjs` | Setup tools page object |
| `tests-js/pages/ToolsPage.mjs` | Tools list/edit page object |
| `tests-js/pages/SetupBasePage.mjs` | Updated step numbers (6 for browser extension, 7 for complete) |
| `tests-js/pages/SetupBrowserExtensionPage.mjs` | Updated step number to 6 |
| `tests-js/specs/setup/setup-flow.spec.mjs` | Added Tools step to flow |
| `tests-js/specs/setup/setup-tools.spec.mjs` | Setup tools E2E spec |
| `tests-js/specs/tools/tools-config.spec.mjs` | Tools config E2E spec |

## Key Implementation Details

### URL Pattern

Used query parameter instead of dynamic route for static export compatibility:
- `/ui/tools/edit?toolid=builtin-exa-web-search` (NOT `/ui/tools/[toolId]`)
- Follows existing pattern from `/ui/models/edit?alias=testalias`

### Two Separate Form Components

**ToolConfigForm** (`/ui/tools/ToolConfigForm.tsx`):
- Used on `/ui/tools/edit` page
- Fetches backend state, shows loading skeleton
- Admin section visible only for `resource_admin` users
- Clear API Key button with confirmation dialog
- Form disabled when app-level is disabled

**SetupToolsForm** (`/ui/setup/tools/SetupToolsForm.tsx`):
- Used on `/ui/setup/tools` page
- Renders immediately with defaults (optimistic)
- App toggle always visible (setup is admin-only)
- Auto-enables tool when user enters API key (UX improvement)
- Applies backend state when loaded (discards local changes)
- No Clear API Key button (fresh setup)

### API-Level Validation

Backend rejects tool config updates when app-level is disabled:
- `PUT /tools/:tool_id/config` returns 400 if `app_enabled = false`
- `POST /tools/:tool_id/execute` returns 400 if `app_enabled = false`
- `DELETE /tools/:tool_id/config` is always allowed (cleanup)

### 404 Handling

`useToolConfig` hook doesn't retry on 404 errors - this is a valid state meaning "no config exists yet":
```typescript
retry: (failureCount, error) => {
  if (error?.response?.status === 404) return false;
  return failureCount < 3;
}
```

### Status Badges

Four status states with colors on list page:
- **Enabled** (green): `app_enabled && configured && enabled`
- **Configured** (yellow): `app_enabled && configured && !enabled`
- **Not Configured** (gray): `app_enabled && !configured`
- **App Disabled** (red): `!app_enabled`

### Setup Flow Changes

- Total steps: 6 → 7
- Step 5: Tools (new)
- Step 6: Browser Extension (was 5)
- Step 7: Complete (was 6)

### Navigation

Tools item added under Settings group:
```tsx
{
  title: 'Tools',
  href: '/ui/tools/',
  description: 'Configure AI tools',
  icon: Wrench,
}
```

## Data Test IDs

| Element | data-testid |
|---------|-------------|
| Tools page | `tools-page` |
| Tool edit page | `tool-edit-page` |
| Tool config form | `tool-config-form` |
| API key input | `tool-api-key-input` |
| Enable toggle | `tool-enabled-toggle` |
| App enabled toggle | `app-enabled-toggle` |
| Save button | `save-tool-config` |
| Clear API key button | `clear-api-key-button` |
| App disabled message | `app-disabled-message` |
| Skip button (setup) | `skip-tools-setup` |
| Tools setup page | `tools-setup-page` |

## API Endpoints Used

| Endpoint | Method | Purpose |
|----------|--------|---------|
| `/bodhi/v1/tools` | GET | List all tools with status |
| `/bodhi/v1/tools/:tool_id/config` | GET | Get user's tool config |
| `/bodhi/v1/tools/:tool_id/config` | PUT | Update user's tool config (blocked if app disabled) |
| `/bodhi/v1/tools/:tool_id/config` | DELETE | Delete user's tool config |
| `/bodhi/v1/tools/:tool_id/execute` | POST | Execute tool (blocked if app disabled) |
| `/bodhi/v1/tools/:tool_id/app-config` | PUT | Enable tool at app level (admin) |
| `/bodhi/v1/tools/:tool_id/app-config` | DELETE | Disable tool at app level (admin) |
