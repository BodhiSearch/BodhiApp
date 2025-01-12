# App Settings Management Feature

## User Story
As an Admin of Bodhi App  
I want to view and configure application settings  
So that I can manage the app's behavior without editing configuration files

## Background
- App settings are stored in settings.yaml and environment variables
- Settings page is accessible via Admin > Settings menu
- In non-authz mode, settings are accessible to all users
- In authz mode, only users with "resource_admin" role can access settings
- Settings can be configured individually and saved

## Acceptance Criteria

### Access Control
- [ ] Add "Settings" menu item under "Admin" dropdown
- [ ] Only show Admin menu to users with "resource_admin" role in authz mode
- [ ] In non-authz mode, show Admin menu to all users
- [ ] When non-admin users try to access settings page:
  - Show warning message "Only administrators can access application settings"
  - Do not display any settings

### Settings Page Layout
- [ ] Group settings into categories:
  - Server Configuration (scheme, host, port, frontend URL)
  - Logging (log level, log directory, stdout logging)
  - Model Configuration (HF home directory)
  - Execution (exec path, exec lookup path)
- [ ] For each setting show:
  - Setting name
  - Current value
  - Default value (if applicable)
  - Clear button (if setting has default value)
  - Input field for new value
  - Save button
- [ ] Show validation errors inline
- [ ] Show success message when setting is saved

### Backend Implementation
- [ ] API endpoint GET /api/settings:
  - Return all settings with their current and default values
  - Include setting metadata (type, validation rules)
- [ ] API endpoint PUT /api/settings/{key}:
  - Update individual setting
  - Validate setting value
  - Return updated setting value
- [ ] Add setting validation rules in backend
- [ ] Add role check middleware for settings APIs

## Available Settings Analysis
Settings from env_service.rs:

Server Settings:
- BODHI_SCHEME (default: "http")
- BODHI_HOST (default: "localhost")
- BODHI_PORT (default: 1135)
- BODHI_FRONTEND_URL

Logging Settings:
- BODHI_LOGS
- BODHI_LOG_LEVEL (default: "warn")
- BODHI_LOG_STDOUT

Data Settings:
- HF_HOME
- BODHI_HOME
- BODHI_EXEC_DEFAULT_VARIANT
- BODHI_EXEC_PATH
- BODHI_EXEC_LOOKUP_PATH

Read-only/System Settings (not configurable):
- BODHI_ENV_TYPE
- BODHI_APP_TYPE
- BODHI_VERSION
- BODHI_AUTH_URL
- BODHI_AUTH_REALM

## Technical Implementation Steps

### Database/Backend Changes
1. Add role check middleware for settings routes
2. Implement settings validation logic
3. Add settings API endpoints

### Frontend Changes
1. Add settings page in `crates/bodhi/src/app/ui/admin/settings`:
   - page.tsx for main layout
   - components/SettingGroup.tsx for grouped settings
   - components/SettingItem.tsx for individual setting
2. Add settings-related hooks in `crates/bodhi/src/hooks`:
   - useSettings
   - useUpdateSetting
3. Add settings types in `crates/bodhi/src/types`
4. Update navigation to include Settings menu item
5. Add role-based menu visibility

## Not In Scope
- App restart after settings change
- Settings history tracking
- Settings export/import
- Settings file location display
- Backup mechanism
- Frontend validation (use backend validation)
- Batch settings updates

## Technical Considerations
- Settings are stored in settings.yaml
- Environment variables take precedence over settings.yaml
- Some settings may have system-level impact
- Backend validation is required for all settings
- Settings can be cleared to use default values
- Individual settings can be updated independently

## Testing Requirements
- [ ] Unit tests for settings validation
- [ ] API endpoint tests
- [ ] UI component tests
- [ ] Role-based access tests
- [ ] Test in both authz and non-authz modes
- [ ] Test setting clear functionality
- [ ] Test validation error display
