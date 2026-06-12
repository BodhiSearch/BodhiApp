# Remove Non-Authenticated Mode Installation Option

## Overview
Simplify the Bodhi App by removing the ability to install and run the application in non-authenticated mode. This change will require all installations to use authenticated mode with OAuth2 integration, eliminating the complexity of supporting two different authentication paradigms and improving overall security posture.

## Current vs. Desired State

### Current State (To Be Removed)
```
Setup Flow:
1. User chooses between "Authenticated Mode" and "Non-Authenticated Mode"
2. Non-Authenticated Mode:
   - No user authentication required
   - All API endpoints publicly accessible
   - No role-based access control
   - No API tokens
   - authz: false stored in secrets

Backend Behavior:
- auth_middleware checks authz flag and skips authentication if false
- api_auth_middleware bypasses authorization if authz is false
- AppInfo API returns authz field indicating current mode
```

### Desired State (Authenticated Only)
```
Setup Flow:
1. User proceeds directly to authenticated mode setup
2. Authenticated Mode (Only Option):
   - OAuth2 authentication required
   - Role-based access control enforced
   - API tokens supported
   - authz concept removed entirely

Backend Behavior:
- All middleware always enforces authentication
- AppInfo API no longer includes authz field
- Setup API no longer accepts authz parameter
```

## Core Features to Remove

### 1. Setup UI Non-Authenticated Option

#### Frontend Changes
- Remove "Non-Authenticated Mode" option from setup page
- Update setup flow to proceed directly to authenticated mode
- Remove authz parameter from setup API calls
- Update setup mode descriptions to focus on authenticated benefits

#### Files to Modify
- `crates/bodhi/src/components/setup/SetupPage.tsx`
- `crates/bodhi/src/components/setup/SetupModeCard.tsx`
- `crates/bodhi/src/hooks/useQuery.ts` (SetupRequest type)

### 2. Backend API Authz References

#### API Endpoint Changes
```
POST /bodhi/v1/setup
- Remove authz field from SetupRequest
- Always setup in authenticated mode
- Always transition to resource-admin status

GET /bodhi/v1/info  
- Remove authz field from AppInfo response
- Simplify response structure
```

#### Files to Modify
- `crates/routes_app/src/routes_setup.rs`
- `crates/bodhi/src/types/models.ts`
- `openapi.json` (auto-generated, will update after code changes)

### 3. Authentication Middleware Simplification

#### Remove Authz Checks
- Remove authz flag checks from auth_middleware
- Remove authz flag checks from api_auth_middleware  
- Always enforce authentication and authorization
- Remove authz_status helper function

#### Files to Modify
- `crates/auth_middleware/src/auth_middleware.rs`
- `crates/auth_middleware/src/api_auth_middleware.rs`

### 4. Secret Service Cleanup

#### Remove Authz Storage
- Remove authz() method from SecretServiceExt trait
- Remove set_authz() method from SecretServiceExt trait
- Remove KEY_APP_AUTHZ constant
- Clean up related test utilities

#### Files to Modify
- `crates/services/src/service_ext.rs`
- `crates/services/src/test_utils/secret.rs`

## Implementation Phases

### Phase 1: Backend API Cleanup
1. Remove authz field from SetupRequest and AppInfo structs
2. Update setup handler to always use authenticated mode
3. Remove authz checks from middleware
4. Remove authz methods from SecretServiceExt
5. Update all backend tests

### Phase 2: Frontend UI Simplification  
1. Remove non-authenticated mode option from setup UI
2. Update setup flow to proceed directly to authenticated setup
3. Remove authz handling from login components
4. Update frontend tests

### Phase 3: Documentation and Testing
1. Update API documentation (OpenAPI)
2. Update architecture documentation
3. Update setup process documentation
4. Comprehensive testing of simplified flow

## Security Considerations

### 1. Enhanced Security Posture
- **Consistent Authentication**: All installations require proper OAuth2 setup
- **No Bypass Paths**: Eliminates potential security bypasses through non-authz mode
- **Role-Based Access**: All users operate within proper RBAC framework
- **Audit Trail**: All actions are properly authenticated and logged

### 2. Code Security
- **Remove Dead Code**: Eliminates unused authentication bypass logic
- **Simplified Attack Surface**: Fewer code paths reduce potential vulnerabilities
- **Clear Security Model**: Single authentication paradigm easier to audit

## API Specifications

### Updated POST /bodhi/v1/setup
**Purpose:** Setup application in authenticated mode (only option)

**Request:**
```json
{}
```

**Response:**
Success (Always transitions to resource-admin):
```json
{
  "status": "resource-admin"
}
```

Error:
```json
{
  "error": {
    "message": "Error description",
    "type": "invalid_request_error",
    "code": "error_code"
  }
}
```

### Updated GET /bodhi/v1/info
**Purpose:** Get application information and status

**Response:**
```json
{
  "version": "0.1.0",
  "status": "ready"
}
```

**Removed Fields:**
- `authz` field no longer present in response

## Testing Requirements

### Backend Testing
- Unit tests for simplified setup handler
- Integration tests for authentication middleware
- Security tests ensuring no authentication bypasses
- API contract tests for updated endpoints
- Remove all authz-related test cases

### Frontend Testing
- Unit tests for simplified setup components
- Integration tests for setup flow
- Component tests for updated UI
- Remove non-authz mode test scenarios

### End-to-End Testing
- Complete setup flow validation
- Authentication enforcement verification
- Role-based access control testing
- Remove authz bypass test scenarios

## Documentation Updates Required

### Technical Documentation
- Update authentication architecture documentation
- Update API endpoint documentation (remove authz references)
- Update development conventions documentation
- Update testing strategy documentation

## Success Metrics

### Development Metrics
- Reduced codebase complexity (fewer authentication code paths)
- Simplified testing matrix (single authentication paradigm)
- Improved code maintainability
- Cleaner separation of concerns

### Code Quality Metrics
- Removal of dead/unused code paths
- Elimination of conditional authentication logic
- Reduced cyclomatic complexity in middleware
- Better test coverage for remaining authentication flows

## Detailed Implementation Plan

### Backend Changes

#### 1. Update AppInfo and SetupRequest Types
**File**: `crates/bodhi/src/types/models.ts`
```typescript
// Remove authz field from AppInfo interface
export interface AppInfo {
  status: AppStatus;
  // authz: boolean; // REMOVE THIS LINE
  version: string;
}
```

**File**: `crates/routes_app/src/routes_setup.rs`
```rust
// Remove authz field from SetupRequest
#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct SetupRequest {
  // Remove authz field entirely - no user choice
}

// Update AppInfo struct
#[derive(Debug, Serialize, Deserialize, PartialEq, ToSchema)]
pub struct AppInfo {
  pub version: String,
  // pub authz: bool, // REMOVE THIS LINE
  pub status: AppStatus,
}
```

#### 2. Simplify Setup Handler
**File**: `crates/routes_app/src/routes_setup.rs`
```rust
// Update setup_handler to always use authenticated mode
pub async fn setup_handler(
  State(state): State<Arc<dyn RouterState>>,
  Json(_request): Json<SetupRequest>, // No authz field to process
) -> Result<Json<SetupResponse>, ApiError> {
  let secret_service = &state.app_service().secret_service();
  let auth_service = &state.app_service().auth_service();
  let status = app_status_or_default(secret_service);

  if status != AppStatus::Setup {
    return Err(AppServiceError::AlreadySetup)?;
  }

  // Always setup in authenticated mode
  let setting_service = &state.app_service().setting_service();
  // ... OAuth2 registration logic (existing code)
  secret_service.set_app_status(&AppStatus::ResourceAdmin)?;

  Ok(SetupResponse {
    status: AppStatus::ResourceAdmin,
  })
}
```

#### 3. Remove Authz from Secret Service
**File**: `crates/services/src/service_ext.rs`
```rust
// Remove authz-related constants and methods
// const KEY_APP_AUTHZ: &str = "app_authz"; // REMOVE

pub trait SecretServiceExt {
  // fn authz(&self) -> Result<bool>; // REMOVE
  // fn set_authz(&self, authz: bool) -> Result<()>; // REMOVE

  fn app_reg_info(&self) -> Result<Option<AppRegInfo>>;
  fn set_app_reg_info(&self, app_reg_info: &AppRegInfo) -> Result<()>;
  fn app_status(&self) -> Result<AppStatus>;
  fn set_app_status(&self, app_status: &AppStatus) -> Result<()>;
}

impl<T: AsRef<dyn SecretService>> SecretServiceExt for T {
  // Remove authz() and set_authz() implementations
  // ... keep other methods
}
```

#### 4. Simplify Authentication Middleware
**File**: `crates/auth_middleware/src/auth_middleware.rs`
```rust
pub async fn auth_middleware(
  session: Session,
  State(state): State<Arc<dyn RouterState>>,
  _headers: HeaderMap,
  mut req: Request,
  next: Next,
) -> Result<Response, ApiError> {
  // Remove authz check - always enforce authentication
  // if !authz_status(&secret_service) { // REMOVE THIS BLOCK
  //   return Ok(next.run(req).await);
  // }

  // Always proceed with authentication logic
  // ... existing authentication code
}

// Remove authz_status function entirely
// fn authz_status(secret_service: &Arc<dyn SecretService>) -> bool { // REMOVE
```

**File**: `crates/auth_middleware/src/api_auth_middleware.rs`
```rust
pub async fn _impl(
  required_role: Role,
  required_scope: Option<TokenScope>,
  State(state): State<Arc<dyn RouterState>>,
  req: Request,
  next: Next,
) -> Result<Response, ApiAuthError> {
  // Remove authz check - always enforce authorization
  // let authz = &state.app_service().secret_service().authz()?; // REMOVE
  // if !authz { // REMOVE THIS BLOCK
  //   return Ok(next.run(req).await);
  // }

  // Always proceed with authorization logic
  // ... existing authorization code
}
```

### Frontend Changes

#### 5. Simplify Setup Page
**File**: `crates/bodhi/src/components/setup/SetupPage.tsx`
```typescript
// Remove setupModes array with non-authenticated option
const setupModes = [
  {
    title: 'Authenticated Mode',
    description: 'Secure setup with user authentication',
    benefits: [
      'User authentication',
      'Multi-user support with RBAC',
      'Secure API endpoints',
      'API Tokens',
      'Resource usage tracking (coming soon)',
      'User/token level usage quotas (coming soon)',
    ],
    icon: '🔐',
    recommended: true,
  },
  // Remove non-authenticated mode object entirely
];

function SetupContent() {
  // Update handleSetup to always use authenticated mode
  const handleSetup = () => {
    setup({ authz: true }); // Or remove authz parameter entirely
  };

  // ... rest of component
}
```

#### 6. Update Setup Mode Card
**File**: `crates/bodhi/src/components/setup/SetupModeCard.tsx`
```typescript
export const SetupModeCard = ({
  setupModes,
  onSetup,
  isLoading,
}: SetupModeCardProps) => {
  return (
    <motion.div variants={itemVariants}>
      <Card>
        <CardHeader>
          <CardTitle className="text-center">Setup Your Bodhi App</CardTitle>
        </CardHeader>
        <CardContent className="space-y-6">
          {/* Show single authenticated mode */}
          <div className="space-y-4">
            {/* Display authenticated mode benefits */}
          </div>

          <div className="pt-6">
            <Button
              className="w-full relative"
              size="lg"
              onClick={() => onSetup(true)} // Always authenticated
              disabled={isLoading}
            >
              {isLoading ? (
                <>
                  <Loader2 className="mr-2 h-4 w-4 animate-spin" />
                  Setting up your secure instance...
                </>
              ) : (
                'Setup Secure Instance →'
              )}
            </Button>
          </div>
        </CardContent>
        {/* Remove warning about not being able to switch */}
      </Card>
    </motion.div>
  );
};
```

#### 7. Update Frontend Types
**File**: `crates/bodhi/src/hooks/useQuery.ts`
```typescript
// Update SetupRequest type
type SetupRequest = {
  // Remove authz field entirely or make it always true
};

// Update useSetupApp to not require authz parameter
export function useSetupApp(options?: {
  onSuccess?: (appInfo: AppInfo) => void;
  onError?: (message: string) => void;
}): UseMutationResult<
  AxiosResponse<AppInfo>,
  AxiosError<ErrorResponse>,
  {} // Empty request object
> {
  const queryClient = useQueryClient();
  return useMutationQuery<AppInfo, {}>(ENDPOINT_APP_SETUP, 'post', {
    // ... existing implementation
  });
}
```

#### 8. Clean Up Login Components
**File**: `crates/bodhi/src/components/LoginMenu.tsx`
```typescript
export function LoginMenu() {
  const { data: userInfo, isLoading: userLoading } = useUser();
  // Remove appInfo query since authz field no longer exists
  // const { data: appInfo, isLoading: appLoading } = useAppInfo();

  // Remove isNonAuthz check
  // const isNonAuthz = appInfo && !appInfo.authz;

  // Remove non-authz rendering block
  // if (isNonAuthz) { ... }

  // Always show login/logout based on user status
}
```

### Test Updates

#### 9. Update Backend Tests
**File**: `crates/routes_app/src/routes_setup.rs`
```rust
// Remove all authz-related test cases
// Update remaining tests to not use authz parameter

#[rstest]
#[tokio::test]
async fn test_setup_handler_success() -> anyhow::Result<()> {
  // Test setup without authz parameter
  let request = SetupRequest {}; // Empty request
  // ... test authenticated setup flow
}

// Remove test_setup_handler_success_for_non_authz
// Remove authz-related test parameters
```

#### 10. Update Frontend Tests
**File**: `crates/bodhi/src/components/setup/SetupPage.test.tsx`
```typescript
// Update tests to not include non-authenticated mode
// Remove tests for authz parameter handling
// Update setup flow tests for simplified UI
```

## Out of Scope

### Not Included in This Story
- Changes to OAuth2 flow implementation
- New authentication features or enhancements
- Performance optimizations
- UI/UX redesign beyond removing non-authz option
- Deployment strategies or release planning
- User migration tools or backwards compatibility
- Production rollout considerations

### Future Development Considerations
- Enhanced setup wizard improvements
- Advanced role management features
- Additional authentication integrations
