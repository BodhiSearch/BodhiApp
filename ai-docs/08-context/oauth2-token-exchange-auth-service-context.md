# OAuth 2.0 Token Exchange - Auth Service Technical Specification

## Overview

Technical specification for the OAuth 2.0 token exchange implementation in the AuthService, supporting dynamic audience management for marketplace scenarios. This document has been updated to reflect the dynamic client creation approach implemented for integration testing.

## Current AuthService Methods

### Method: `request_access`

**Purpose**: Enables resource clients to request inclusion in app token audiences for dynamic audience management.

**Signature**:
```rust
async fn request_access(
  &self,
  client_id: &str,
  client_secret: &str,
  app_client_id: &str,
) -> Result<String>;
```

**Implementation**:
- **Endpoint**: `POST /realms/{realm}/bodhi/resources/request-access`
- **Authentication**: Client credentials flow to obtain access token
- **Request Structure**: `RequestAccessRequest` with utoipa annotations
- **Response**: Returns scope string for the resource access
- **Error Handling**: Standard Keycloak error responses

### Method: `register_client`

**Purpose**: Resource client registration with name, description, and validation.

**Signature**:
```rust
async fn register_client(
  &self,
  name: String,
  description: String,
  redirect_uris: Vec<String>,
) -> Result<AppRegInfo>;
```

**Implementation**:
- **Endpoint**: `POST /realms/{realm}/bodhi/resources`
- **Request Structure**: `RegisterClientRequest` with utoipa annotations
- **Validation**: Server name minimum 10 characters, description optional
- **Form Validation**: Backend validation with `BadRequestError` for invalid input
- **Response**: Returns `AppRegInfo` with client credentials

### Method: `exchange_app_token`

**Purpose**: Simplified token exchange for resource-specific access tokens.

**Signature**:
```rust
async fn exchange_app_token(
  &self,
  client_id: &str,
  client_secret: &str,
  subject_token: &str,
  scopes: Vec<String>,
) -> Result<(String, Option<String>)>;
```

**Implementation**:
- **Endpoint**: `POST /realms/{realm}/protocol/openid-connect/token`
- **Grant Type**: `urn:ietf:params:oauth:grant-type:token-exchange`
- **Parameters**: subject_token, client_id, client_secret, audience, scope
- **Response**: Returns access token and optional refresh token
- **Error Handling**: Maps Keycloak errors to AuthServiceError

### Method: `make_resource_admin`

**Purpose**: Assigns a user as administrator for a specific resource client.

**Signature**:
```rust
async fn make_resource_admin(
  &self,
  client_id: &str,
  client_secret: &str,
  email: &str,
) -> Result<()>;
```

**Implementation**:
- **Endpoint**: `POST {auth_api_url}/clients/make-resource-admin`
- **Authentication**: Client credentials flow to obtain access token
- **Request Structure**: JSON with `{"username": email}`
- **Response**: Success or Keycloak error
- **Usage**: Called during dynamic client setup to assign user roles

## Dynamic Client Creation for Integration Testing

### AuthServerTestClient

A comprehensive test client that encapsulates all interactions with the Keycloak test server for dynamic client setup and token operations.

**Location**: `crates/auth_middleware/src/test_utils/auth_server_test_client.rs`

**Key Features**:
- Dynamic app and resource client creation
- Token exchange operations
- User role assignment
- Audience access management
- Follows httpyac script patterns

### Integration Test Flow

The integration tests now follow this dynamic approach:

1. **Load Configuration**: Read admin credentials and auth server URL from `.env.test`
2. **Setup Dynamic Clients**: Create app and resource clients on-demand
3. **Assign User Roles**: Make test user admin of resource client
4. **Request Audience Access**: Enable app client to access resource scope
5. **Perform Token Operations**: Get tokens and test exchange flows
6. **Run Test Logic**: Execute actual test scenarios

### HTTPyac Script Reference

The implementation follows the successful httpyac script pattern:

```http
### Step 1: Get Dev Console User Token (Direct Access Grant)
POST {{keycloak_url}}/realms/{{realm}}/protocol/openid-connect/token
Authorization: Basic {{dev_console_basic_auth}}
Content-Type: application/x-www-form-urlencoded

grant_type=password&username={{user_regular}}&password={{user_regular_password}}

### Step 2: Create App Client (Public, No Secrets)
POST {{keycloak_url}}/realms/{{realm}}/bodhi/apps
Authorization: Bearer {{dev_console_token}}
Content-Type: application/json

{
  "name": "Unit Test App Client",
  "description": "App client for unit testing",
  "redirect_uris": ["http://localhost:3000/callback"]
}

### Step 3: Create Resource Server Client (Confidential, With Secrets)
POST {{keycloak_url}}/realms/{{realm}}/bodhi/resources
Content-Type: application/json

{
  "name": "Unit Test Resource Server",
  "description": "Resource client for unit testing",
  "redirect_uris": ["http://localhost:8080/callback"]
}

### Step 4: Get Resource Client Service Account Token
POST {{keycloak_url}}/realms/{{realm}}/protocol/openid-connect/token
Authorization: Basic {{resource_client_basic_auth}}
Content-Type: application/x-www-form-urlencoded

grant_type=client_credentials&scope=service_account

### Step 5: Make First Resource Admin
POST {{make_admin_endpoint}}
Authorization: Bearer {{resource_service_token}}
Content-Type: application/json

{
  "username": "{{user_regular}}"
}

### Step 6: Resource Client Requests Audience Access
POST {{keycloak_url}}/realms/{{realm}}/bodhi/resources/request-access
Authorization: Bearer {{resource_service_token}}
Content-Type: application/json

{
  "app_client_id": "{{app_client_id}}"
}

### Step 7: App User Token via Direct Access Grant (with Resource Scope)
POST {{keycloak_url}}/realms/{{realm}}/protocol/openid-connect/token
Content-Type: application/x-www-form-urlencoded

grant_type=password&client_id={{app_client_id}}&username={{user_regular}}&password={{user_regular_password}}&scope=openid email profile roles scope_user_user {{resource_scope_name}}
```

## Test Environment Configuration

### Environment Variables (.env.test)

```bash
INTEG_TEST_AUTH_URL=https://test-id.getbodhi.app
INTEG_TEST_AUTH_REALM=bodhi
INTEG_TEST_ADMIN_USERNAME=admin
INTEG_TEST_ADMIN_PASSWORD=admin_password
INTEG_TEST_DEV_CONSOLE_CLIENT_ID=client-bodhi-dev-console
INTEG_TEST_DEV_CONSOLE_CLIENT_SECRET=dev_console_secret
INTEG_TEST_USER=user@email.com
INTEG_TEST_PASSWORD=pass
```

### Test Scenarios

1. **Offline Token Exchange Success**
   - Creates resource client with `scope_token_user`
   - Tests offline token creation with session
   - Validates token scopes: `["basic", "offline_access", "openid", "scope_token_user"]`

2. **Cross Client Token Exchange Success**
   - Creates app and resource clients dynamically
   - Gets app user token with resource scope
   - Tests token exchange between clients
   - Validates user scope mapping: `scope_user_user`

3. **Cross Client Token Exchange No User Scope**
   - Creates clients with resource scope but no user scopes
   - Tests rejection for missing user privileges
   - Validates error: "user does not have any privileges on this system"

4. **Cross Client Token Exchange Auth Service Error**
   - Tests authentication service error handling
   - Validates error responses and status codes

## Scope and Role Mapping

### User Scopes (External Clients)
- `scope_user_user` → UserScope::User → Role::User
- `scope_user_power_user` → UserScope::PowerUser → Role::PowerUser  
- `scope_user_manager` → UserScope::Manager → Role::Manager
- `scope_user_admin` → UserScope::Admin → Role::Admin

### Token Scopes (Internal Tokens)
- `scope_token_user` → TokenScope::User
- `scope_token_power_user` → TokenScope::PowerUser
- `scope_token_manager` → TokenScope::Manager
- `scope_token_admin` → TokenScope::Admin

### Resource Scopes
Dynamic resource scopes follow the pattern: `scope_resource-{client_id}`

## Security Considerations

1. **Dynamic Client Isolation**: Each test creates unique clients to avoid conflicts
2. **Scope Validation**: Tokens must contain appropriate user scopes for authorization
3. **Audience Validation**: Tokens must have correct audience for the target resource
4. **Role Assignment**: Users must be explicitly made admins of resource clients
5. **Token Expiration**: All tokens have appropriate expiration times
6. **Error Handling**: Clear error messages for authentication and authorization failures

## Implementation Benefits

1. **Test Reliability**: No dependency on pre-configured clients
2. **Parallel Execution**: Tests can run concurrently without conflicts  
3. **Realistic Scenarios**: Tests mirror actual client registration flows
4. **Maintainability**: Centralized client setup logic in `AuthServerTestClient`
5. **Debugging**: Clear separation of setup vs. test logic
6. **Documentation**: Tests serve as executable documentation of the flow 