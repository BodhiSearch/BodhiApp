# UI Pages - Tools Feature

> Layer: `crates/bodhi` (Next.js) | Status: ✅ Completed (Phase 8)

## Implementation Summary

Phase 8 implementation completed with the following:
- Tools list page at `/ui/tools` with DataTable display
- Tool configuration page at `/ui/tools/edit?toolid=xxx`
- Setup flow integration as Step 5 (7-step flow)
- Separate form components for config (`ToolConfigForm`) and setup (`SetupToolsForm`)
- Admin controls for app-level enable/disable with confirmation dialogs
- Full test coverage (unit tests + MSW handlers)
- E2E test page objects and setup flow updates

See `ai-docs/specs/20260114-tools-backend/phase-8-implementation.md` for detailed implementation notes.

## Navigation

Tools item added under Settings group (between API Tokens and Manage Users).

### Sidebar Entry

```tsx
// crates/bodhi/src/hooks/use-navigation.tsx
{
  title: 'Tools',
  href: '/ui/tools/',
  description: 'Configure AI tools',
  icon: Wrench,
}
```

## Pages

### /ui/tools - Tools List

Lists all available tools with configuration status. Uses DataTable for display with status badges.

**File:** `crates/bodhi/src/app/ui/tools/page.tsx`

**Status Badges:**
- **Enabled** (green): `app_enabled && configured && enabled`
- **Configured** (yellow): `app_enabled && configured && !enabled`
- **Not Configured** (gray): `app_enabled && !configured`
- **App Disabled** (red): `!app_enabled`

### /ui/tools/edit?toolid=xxx - Tool Configuration

Configuration page for individual tool. Uses query parameter (not dynamic route) for static export compatibility.

**File:** `crates/bodhi/src/app/ui/tools/edit/page.tsx`

**Component:** `ToolConfigForm` (`crates/bodhi/src/app/ui/tools/ToolConfigForm.tsx`)

**Features:**
- Fetches backend state, shows loading skeleton
- Admin section visible only for `resource_admin` users
- App-level enable/disable toggle with confirmation dialogs
- API key input (password field) with external link to exa.ai
- Enable toggle (disabled until API key configured)
- Clear API Key button with confirmation dialog
- Form disabled when app-level is disabled

### /ui/setup/tools - Setup Step 5

Setup flow integration for first-time tool configuration.

**File:** `crates/bodhi/src/app/ui/setup/tools/page.tsx`

**Component:** `SetupToolsForm` (`crates/bodhi/src/app/ui/setup/tools/SetupToolsForm.tsx`)

**Features:**
- Renders immediately with defaults (optimistic rendering)
- App toggle always visible (setup is admin-only context)
- Auto-enables tool when user enters API key (UX improvement)
- Applies backend state when loaded (discards local changes)
- No Clear API Key button (fresh setup)
- Skip option to proceed without configuration

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

## API Hooks

**File:** `crates/bodhi/src/hooks/useTools.ts`

| Hook | Purpose |
|------|---------|
| `useAvailableTools` | Fetch all tools with status |
| `useToolConfig` | Fetch tool configuration (no retry on 404) |
| `useUpdateToolConfig` | Update user's tool config |
| `useDeleteToolConfig` | Delete user's tool config (clear API key) |
| `useSetAppToolEnabled` | Enable tool at app level (admin) |
| `useSetAppToolDisabled` | Disable tool at app level (admin) |

## MSW Mocks (for tests)

**File:** `crates/bodhi/src/test-utils/msw-v2/handlers/tools.ts`

Handlers for:
- `GET /bodhi/v1/tools` - List all tools
- `GET /bodhi/v1/tools/:tool_id/config` - Get tool config
- `PUT /bodhi/v1/tools/:tool_id/config` - Update tool config
- `DELETE /bodhi/v1/tools/:tool_id/config` - Delete tool config
- `PUT /bodhi/v1/tools/:tool_id/app-config` - Enable app tool
- `DELETE /bodhi/v1/tools/:tool_id/app-config` - Disable app tool
