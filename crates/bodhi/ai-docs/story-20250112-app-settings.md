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
- [ ] In non-authz mode, show Admin menu to all users
- [ ] When non-admin users try to access settings page:
  - Show warning message "Only Admins can view and modify the settings."
  - Do not display any settings when authz: true and role < Admin

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
  - Clear button (if setting has a user given value)
  - Input field for new value
  - Save button
- [ ] Show validation errors inline
- [ ] Show success message when setting is saved

### Backend Implementation
- [ ] API endpoint GET /v1/bodhi/settings:
  - Return all settings with their current and default values
  - Include setting metadata (type, for number their range)
- [ ] API endpoint PUT /v1/bodhi/settings/{key}:
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
- HF_TOKEN
- BODHI_HOME
- BODHI_EXEC_DEFAULT_VARIANT
- BODHI_EXEC_PATH
- BODHI_EXEC_LOOKUP_PATH
- BODHI_EXEC_VARIANT

Read-only/System Settings (not configurable):
- BODHI_ENV_TYPE
- BODHI_APP_TYPE
- BODHI_VERSION
- BODHI_AUTH_URL
- BODHI_AUTH_REALM

## Technical Implementation Details

### Settings Hierarchy
Settings are read in the following order of precedence:
1. Command line arguments
2. Environment variables
3. settings.yaml in BODHI_HOME directory
4. Default values from codebase

### API Endpoints

#### GET /v1/bodhi/settings
Returns all settings with their current and default values.

Response format:
```typescript
interface SettingResponse {
  settings: Array<{
    key: string;
    current_value: string | number | boolean;
    default_value: string | number | boolean;
    metadata: {
      type: 'string' | 'number' | 'boolean' | 'option';
      options?: string[];      // for type='option' 
      range?: {               // for type='number'
        min: number;
        max: number;
      };
    }
  }>
}
```

#### PUT /v1/bodhi/settings/{key}
Updates individual setting value.

Request format:
```typescript
interface UpdateSettingRequest {
  value: string | number | boolean;
}
```

#### DELETE /v1/bodhi/settings/{key}
Resets setting to default value.

### Available Settings

#### Server Configuration
- BODHI_SCHEME (type: string, default: "http")
- BODHI_HOST (type: string, default: "localhost") 
- BODHI_PORT (type: number, default: 1135)
- BODHI_FRONTEND_URL (type: string)

#### Logging Configuration
- BODHI_LOGS (type: string)
- BODHI_LOG_LEVEL (type: option, default: "warn", options: ["error", "warn", "info", "debug", "trace"])
- BODHI_LOG_STDOUT (type: boolean)

#### Model Configuration  
- HF_HOME (type: string)
- BODHI_HOME (type: string)

#### Execution Configuration
- BODHI_EXEC_PATH (type: string)
- BODHI_EXEC_LOOKUP_PATH (type: string)

### Frontend Implementation

#### Components
1. Settings Page (`crates/bodhi/src/app/ui/settings/page.tsx`)
   - Main layout with settings groups
   - Role-based access control
   - Error handling

2. Setting Groups (`crates/bodhi/src/app/ui/settings/components/SettingGroup.tsx`)
   - Collapsible groups by category
   - Title and description
   - List of settings

3. Setting Item (`crates/bodhi/src/app/ui/settings/components/SettingItem.tsx`)
   - Label and description
   - Current value display
   - Default value display
   - Input field based on type:
     - Text input for strings
     - Number input for numbers
     - Select dropdown for options
     - Toggle for booleans
   - Save and Reset buttons
   - Validation error display

#### State Management
- Use React Query for data fetching and caching
- Simple state management with no real-time updates
- Optimistic updates for better UX

### Backend Implementation

#### Service Layer
Uses existing setting_service.rs methods:
```rust
fn get_setting_value(&self, key: &str) -> Option<Value>;
fn set_setting(&self, key: &str, value: &str) -> Result<()>;
fn delete_setting(&self, key: &str) -> Result<()>;
```

#### Error Handling
- 401 Unauthorized: Non-admin access attempt
- 404 Not Found: Invalid setting key
- 422 Unprocessable Entity: Validation failure
- 500 Internal Server Error: Setting update failure

### Testing Requirements

#### Backend Tests
1. Unit Tests
   - Setting validation
   - Role-based access
   - Setting service methods
   - API endpoints

2. Frontend Tests
   - Component rendering
   - User interactions
   - Error handling
   - Role-based display
   - Input validation

## Implementation Tasks

### Backend Tasks
1. API Implementation
   - [ ] Add settings controller
   - [ ] Implement GET /v1/bodhi/settings endpoint
   - [ ] Implement PUT /v1/bodhi/settings/{key} endpoint
   - [ ] Implement DELETE /v1/bodhi/settings/{key} endpoint
   - [ ] Add role-based authorization
   - [ ] Add input validation
   - [ ] Add error handling

2. Service Layer
   - [ ] Add settings metadata
   - [ ] Add validation rules
   - [ ] Add error messages
   - [ ] Add logging

### Frontend Tasks
1. Components
   - [ ] Create settings page
   - [ ] Create setting group component
   - [ ] Create setting item component
   - [ ] Add role-based access control
   - [ ] Add error handling
   - [ ] Add loading states

2. API Integration
   - [ ] Add settings API client
   - [ ] Add React Query hooks
   - [ ] Add error handling
   - [ ] Add loading states

3. UI/UX
   - [ ] Add responsive design
   - [ ] Add validation feedback
   - [ ] Add success/error notifications
   - [ ] Add loading indicators

### Testing Tasks
1. Backend Tests
   - [ ] Add API endpoint tests
   - [ ] Add service layer tests
   - [ ] Add validation tests
   - [ ] Add role-based access tests

2. Frontend Tests
   - [ ] Add component tests
   - [ ] Add API integration tests
   - [ ] Add validation tests
   - [ ] Add role-based display tests

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

## File Overview

### Frontend (React/TypeScript)
- `crates/bodhi/src/app/ui/settings/page.tsx`: Main settings page component
- `crates/bodhi/src/app/ui/settings/components/SettingGroup.tsx`: Settings group component
- `crates/bodhi/src/app/ui/settings/components/SettingItem.tsx`: Individual setting component
- `crates/bodhi/src/hooks/useSettings.ts`: React Query hooks for settings management
- `crates/bodhi/src/types/settings.ts`: TypeScript types for settings
- `crates/bodhi/src/components/navigation/AppNavigation.tsx`: Navigation with settings menu

### Backend (Rust)
- `crates/routes_app/src/routes_settings.rs`: API endpoints for settings management
- `crates/services/src/setting_service.rs`: Core settings service implementation
- `crates/services/src/env_service.rs`: Environment and settings defaults
- `crates/auth_middleware/src/api_auth_middleware.rs`: Role-based authorization
- `crates/routes_all/src/routes.rs`: Route registration for settings endpoints
- `crates/server_app/src/serve.rs`: Server setup and configuration
- `crates/bodhi/src-tauri/src/app.rs`: App initialization and configuration
- `crates/bodhi/src-tauri/src/convert.rs`: Command line argument handling

### Tests
- `crates/bodhi/src/app/ui/settings/*.test.tsx`: Frontend component tests
- `crates/bodhi/src/hooks/useSettings.test.ts`: Hook tests with MSW
- `crates/services/src/test_utils/settings.rs`: Test utilities for settings
- `crates/routes_app/src/test_utils/settings.rs`: API endpoint tests

### Documentation
- `crates/bodhi/ai-docs/story-20250112-app-settings.md`: Main story and implementation details
- `crates/bodhi/ai-docs/story-20250116-api-authorization.md`: Authorization implementation details
- `crates/auth_middleware/src/resources/en-US/messages.ftl`: Error messages for authorization
- `crates/routes_app/src/resources/en-US/messages.ftl`: API endpoint error messages
- `crates/services/src/resources/en-US/messages.ftl`: Service layer error messages