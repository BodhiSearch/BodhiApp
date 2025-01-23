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
- [x] API endpoint GET /v1/bodhi/settings:
  - Return all settings with their current and default values
  - Include setting metadata (type, for number their range)
- [x] API endpoint PUT /v1/bodhi/settings/{key}:
  - Update individual setting
  - Validate setting value
  - Return updated setting value
- [x] Add setting validation rules in backend
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
- BODHI_EXEC_VARIANT
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
- BODHI_EXEC_VARIANT (type: string)
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
- `crates/objs/src/envs.rs`: API endpoints for settings management
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

## API Test Scenarios

### GET /v1/bodhi/settings

1. List Settings - Non-Auth Mode
   - GIVEN the app is in non-auth mode
   - WHEN a GET request is made to /v1/bodhi/settings
   - THEN returns 200 with list of all settings
   - AND includes current values, default values and metadata

2. List Settings - Auth Mode (Admin)
   - GIVEN the app is in auth mode
   - AND user has resource_admin role
   - WHEN a GET request is made to /v1/bodhi/settings
   - THEN returns 200 with list of all settings
   - AND includes current values, default values and metadata

3. List Settings - Auth Mode (Non-Admin)
   - GIVEN the app is in auth mode
   - AND user does not have resource_admin role
   - WHEN a GET request is made to /v1/bodhi/settings
   - THEN returns 401 Unauthorized
   - AND returns appropriate error message

4. List Settings - With Custom Values
   - GIVEN some settings have custom values in settings.yaml
   - WHEN a GET request is made to /v1/bodhi/settings
   - THEN returns 200 with list of all settings
   - AND shows custom values as current_value
   - AND shows system defaults as default_value

5. List Settings - With Environment Overrides
   - GIVEN some settings are overridden by environment variables
   - WHEN a GET request is made to /v1/bodhi/settings
   - THEN returns 200 with list of all settings
   - AND environment values take precedence over settings.yaml
   - AND shows system defaults as default_value

### PUT /v1/bodhi/settings/{key}

1. Update Setting - Valid Key (Non-Auth Mode)
   - GIVEN the app is in non-auth mode
   - WHEN a PUT request is made to /v1/bodhi/settings/BODHI_LOG_LEVEL
   - WITH valid value for the setting type
   - THEN returns 200 with updated setting info
   - AND setting is persisted to settings.yaml

2. Update Setting - Valid Key (Auth Mode Admin)
   - GIVEN the app is in auth mode
   - AND user has resource_admin role
   - WHEN a PUT request is made to /v1/bodhi/settings/BODHI_LOG_LEVEL
   - WITH valid value for the setting type
   - THEN returns 200 with updated setting info
   - AND setting is persisted to settings.yaml

3. Update Setting - Valid Key (Auth Mode Non-Admin)
   - GIVEN the app is in auth mode
   - AND user does not have resource_admin role
   - WHEN a PUT request is made to /v1/bodhi/settings/BODHI_LOG_LEVEL
   - THEN returns 401 Unauthorized
   - AND setting remains unchanged

4. Update Setting - Invalid Key
   - GIVEN any auth mode
   - WHEN a PUT request is made to /v1/bodhi/settings/INVALID_KEY
   - THEN returns 404 Not Found
   - AND returns appropriate error message

5. Update Setting - Invalid Value Type
   - GIVEN any auth mode
   - WHEN a PUT request is made to /v1/bodhi/settings/BODHI_PORT
   - WITH value of wrong type (e.g. string for number)
   - THEN returns 422 Unprocessable Entity
   - AND returns appropriate error message

6. Update Setting - Out of Range Value
   - GIVEN any auth mode
   - WHEN a PUT request is made to /v1/bodhi/settings/BODHI_PORT
   - WITH number outside valid range
   - THEN returns 422 Unprocessable Entity
   - AND returns appropriate error message

7. Update Setting - Invalid Option Value
   - GIVEN any auth mode
   - WHEN a PUT request is made to /v1/bodhi/settings/BODHI_LOG_LEVEL
   - WITH value not in allowed options
   - THEN returns 422 Unprocessable Entity
   - AND returns appropriate error message

8. Update Setting - Read Only Setting
   - GIVEN any auth mode
   - WHEN a PUT request is made to /v1/bodhi/settings/BODHI_VERSION
   - THEN returns 422 Unprocessable Entity
   - AND returns "Setting is read-only" message

### DELETE /v1/bodhi/settings/{key}

1. Delete Setting - Valid Key (Non-Auth Mode)
   - GIVEN the app is in non-auth mode
   - AND setting has custom value
   - WHEN a DELETE request is made to /v1/bodhi/settings/BODHI_LOG_LEVEL
   - THEN returns 200
   - AND setting reverts to default value

2. Delete Setting - Valid Key (Auth Mode Admin)
   - GIVEN the app is in auth mode
   - AND user has resource_admin role
   - AND setting has custom value
   - WHEN a DELETE request is made to /v1/bodhi/settings/BODHI_LOG_LEVEL
   - THEN returns 200
   - AND setting reverts to default value

3. Delete Setting - Valid Key (Auth Mode Non-Admin)
   - GIVEN the app is in auth mode
   - AND user does not have resource_admin role
   - WHEN a DELETE request is made to /v1/bodhi/settings/BODHI_LOG_LEVEL
   - THEN returns 401 Unauthorized
   - AND setting remains unchanged

4. Delete Setting - Invalid Key
   - GIVEN any auth mode
   - WHEN a DELETE request is made to /v1/bodhi/settings/INVALID_KEY
   - THEN returns 404 Not Found
   - AND returns appropriate error message

5. Delete Setting - Read Only Setting
   - GIVEN any auth mode
   - WHEN a DELETE request is made to /v1/bodhi/settings/BODHI_VERSION
   - THEN returns 422 Unprocessable Entity
   - AND returns "Setting is read-only" message

6. Delete Setting - No Custom Value
   - GIVEN any auth mode
   - AND setting has no custom value
   - WHEN a DELETE request is made to /v1/bodhi/settings/BODHI_LOG_LEVEL
   - THEN returns 200
   - AND setting remains at default value

## Next Steps

### API Integration Tasks

1. API Response Types
   - Create/update API response types in `crates/objs/src/api.rs` to match SettingInfo structure
   - Ensure proper serialization of all fields including metadata
   - Add response examples to documentation

2. Settings Routes Update
   - Modify `crates/routes_app/src/routes_settings.rs` to use SettingInfo
   - Update GET `/settings` endpoint to return full setting information
   - Update PUT `/settings/{key}` endpoint to validate values against metadata
   - Add validation error responses

3. Frontend Integration
   - Add TypeScript types matching new API response format
   - Update settings state management for richer setting information
   - Modify settings UI to display:
     - Setting sources (env, file, default)
     - Default values
     - Validation rules
   - Add input validation based on metadata:
     - Number ranges
     - Option lists
     - Boolean toggles
     - String inputs

4. Validation Implementation
   - Create validation functions using SettingMetadata
   - Add validation to setting updates
   - Implement proper error messages for validation failures
   - Add tests for validation logic

5. Documentation
   - Update API documentation with new response format
   - Document validation rules and setting metadata
   - Update user documentation for new settings UI
   - Add examples of different setting types and sources

### Settings Value Management

#### Setting Sources
Settings can come from different sources, in order of precedence:
1. Command line arguments (highest priority)
2. Environment variables
3. settings.yaml file
4. Default values (lowest priority)

The source of each setting is tracked in the `SettingSource` enum to help with:
- Debugging configuration issues
- Understanding where a setting value comes from
- Determining if a value can be modified

#### Setting Types and Validation
Each setting has associated metadata that defines:
1. Type information:
   - String: Basic text values
   - Number: Integer or float values with optional range constraints
   - Boolean: true/false values
   - Option: Value must be one of predefined options

2. Validation rules:
   - Number ranges (min/max)
   - Option lists (allowed values)
   - Type-specific parsing rules

3. Value Parsing:
   - All values start as strings (from env vars or yaml)
   - Values are parsed according to their metadata type
   - Failed parsing preserves original value for debugging
   - Successful parsing converts to appropriate type:
     - Numbers: parsed as integers or floats
     - Booleans: handles "true"/"false" strings
     - Options: validated against allowed list
     - Strings: preserved as-is

#### Setting Updates
When updating settings:
1. Values are validated against metadata rules
2. Updates are only allowed for configurable settings
3. System settings (version, env type, etc.) are read-only
4. Updates are persisted to settings.yaml
5. Environment variables still take precedence over saved values