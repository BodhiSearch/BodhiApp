# UI Pages - Toolsets Feature

> Layer: `crates/bodhi` (Next.js) | Status: ✅ Complete

## Implementation Summary

- Toolsets list page at `/ui/toolsets` with DataTable display
- Toolset configuration page at `/ui/toolsets/edit?toolsetid=xxx`
- Setup flow integration as Step 5 (7-step flow)
- Separate form components for config (`ToolsetConfigForm`) and setup (`SetupToolsetsForm`)
- Admin controls for app-level enable/disable with confirmation dialogs
- Full test coverage (unit tests + MSW handlers)
- E2E test page objects and setup flow updates

## Navigation

Toolsets item added under Settings group (between API Tokens and Manage Users).

### Sidebar Entry

```tsx
// crates/bodhi/src/hooks/use-navigation.tsx
{
  title: 'Toolsets',
  href: '/ui/toolsets/',
  description: 'Configure AI toolsets',
  icon: Wrench,
}
```

## Pages

### /ui/toolsets - Toolsets List

Lists all available toolsets with their tools and configuration status. Uses DataTable for display with status badges.

**File:** `crates/bodhi/src/app/ui/toolsets/page.tsx`

**Status Badges:**
- **Enabled** (green): `app_enabled && configured && enabled`
- **Configured** (yellow): `app_enabled && configured && !enabled`
- **Not Configured** (gray): `app_enabled && !configured`
- **App Disabled** (red): `!app_enabled`

**Toolset Display:**
- Shows toolset name and description
- Shows count of available tools (e.g., "4 tools")
- Expandable to see individual tool names

### /ui/toolsets/edit?toolsetid=xxx - Toolset Configuration

Configuration page for individual toolset. Uses query parameter (not dynamic route) for static export compatibility.

**File:** `crates/bodhi/src/app/ui/toolsets/edit/page.tsx`

**Component:** `ToolsetConfigForm` (`crates/bodhi/src/app/ui/toolsets/ToolsetConfigForm.tsx`)

**Features:**
- Fetches backend state, shows loading skeleton
- Admin section visible only for `resource_admin` users
- App-level enable/disable toggle with confirmation dialogs
- API key input (password field) with external link to exa.ai
- Enable toggle (disabled until API key configured)
- Clear API Key button with confirmation dialog
- Form disabled when app-level is disabled
- Shows list of tools provided by this toolset

### /ui/setup/toolsets - Setup Step 5

Setup flow integration for first-time toolset configuration.

**File:** `crates/bodhi/src/app/ui/setup/toolsets/page.tsx`

**Component:** `SetupToolsetsForm` (`crates/bodhi/src/app/ui/setup/toolsets/SetupToolsetsForm.tsx`)

**Features:**
- Renders immediately with defaults (optimistic rendering)
- App toggle always visible (setup is admin-only context)
- Auto-enables toolset when user enters API key (UX improvement)
- Applies backend state when loaded (discards local changes)
- No Clear API Key button (fresh setup)
- Skip option to proceed without configuration

## Data Test IDs

| Element | data-testid |
|---------|-------------|
| Toolsets page | `toolsets-page` |
| Toolset edit page | `toolset-edit-page` |
| Toolset config form | `toolset-config-form` |
| API key input | `toolset-api-key-input` |
| Enable toggle | `toolset-enabled-toggle` |
| App enabled toggle | `app-enabled-toggle` |
| Save button | `save-toolset-config` |
| Clear API key button | `clear-api-key-button` |
| App disabled message | `app-disabled-message` |
| Skip button (setup) | `skip-toolsets-setup` |
| Toolsets setup page | `toolsets-setup-page` |
| Tools list | `toolset-tools-list` |

## API Hooks

**File:** `crates/bodhi/src/hooks/useToolsets.ts`

| Hook | Purpose |
|------|---------|
| `useAvailableToolsets` | Fetch all toolsets with status and tools |
| `useToolsetConfig` | Fetch toolset configuration (no retry on 404) |
| `useUpdateToolsetConfig` | Update user's toolset config |
| `useDeleteToolsetConfig` | Delete user's toolset config (clear API key) |
| `useSetAppToolsetEnabled` | Enable toolset at app level (admin) |
| `useSetAppToolsetDisabled` | Disable toolset at app level (admin) |

## MSW Mocks (for tests)

**File:** `crates/bodhi/src/test-utils/msw-v2/handlers/toolsets.ts`

Handlers for:
- `GET /bodhi/v1/toolsets` - List all toolsets with tools
- `GET /bodhi/v1/toolsets/:toolset_id/config` - Get toolset config
- `PUT /bodhi/v1/toolsets/:toolset_id/config` - Update toolset config
- `DELETE /bodhi/v1/toolsets/:toolset_id/config` - Delete toolset config
- `PUT /bodhi/v1/toolsets/:toolset_id/app-config` - Enable app toolset
- `DELETE /bodhi/v1/toolsets/:toolset_id/app-config` - Disable app toolset

## Types

```typescript
// crates/bodhi/src/hooks/useToolsets.ts

export interface ToolDefinition {
  type: string;
  function: {
    name: string;
    description: string;
    parameters: Record<string, unknown>;
  };
}

export interface ToolsetListItem {
  toolset_id: string;
  name: string;
  description: string;
  app_enabled: boolean;
  user_config?: { enabled: boolean; has_api_key: boolean };
  tools: ToolDefinition[];
}

export interface ListToolsetsResponse {
  toolsets: ToolsetListItem[];
}

export interface UserToolsetConfig {
  toolset_id: string;
  enabled: boolean;
  created_at: string;
  updated_at: string;
}

export interface EnhancedToolsetConfigResponse {
  toolset_id: string;
  app_enabled: boolean;
  config: UserToolsetConfig;
}
```
