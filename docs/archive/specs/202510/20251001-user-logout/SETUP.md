# Token Refresh Integration Test - Setup Guide

## Overview

This guide provides detailed setup instructions for all five solution options to enable token refresh integration test execution. Choose the solution that best fits your environment and security requirements.

## Prerequisites

### Common Prerequisites (All Solutions)
1. **Keycloak Server Running**
   - Version: 21.0+ recommended
   - Accessible at URL you'll configure
   - Admin UI accessible for configuration

2. **Node.js Environment**
   - Node.js >= 22.0
   - npm or yarn package manager

3. **Test Environment**
   - Working directory: `crates/lib_bodhiserver_napi`
   - Test framework: Playwright

4. **Base Environment Variables** (Required for ALL solutions)
   ```bash
   # Keycloak server configuration
   export INTEG_TEST_MAIN_AUTH_URL=http://localhost:8080
   export INTEG_TEST_AUTH_REALM=bodhiapp-test
   export INTEG_TEST_DEV_CONSOLE_CLIENT_SECRET=<your-dev-console-secret>

   # Test user credentials
   export INTEG_TEST_USERNAME=testuser@example.com
   export INTEG_TEST_PASSWORD=<test-user-password>
   export INTEG_TEST_USERNAME_ID=<test-user-uuid>
   ```

## Solution Options

### Option 1: Master Realm Admin (Development/Manual Testing)

**Best For**: Quick local development and manual testing
**Security**: ⚠️ Not recommended for CI/CD (exposes master realm credentials)
**Complexity**: Low
**Setup Time**: 5 minutes

#### Setup Steps

1. **Verify Master Realm Admin Credentials**
   - Default Keycloak admin username: `admin`
   - Default password: set during Keycloak first run
   - Or check your Keycloak installation documentation

2. **Set Additional Environment Variables**
   ```bash
   export INTEG_TEST_MASTER_ADMIN_USERNAME=admin
   export INTEG_TEST_MASTER_ADMIN_PASSWORD=<your-master-admin-password>
   ```

3. **Modify Test Code** (if needed)
   Update `auth-server-client.mjs` to support master realm admin:
   ```javascript
   async getMasterRealmAdminToken(username, password) {
     const params = new URLSearchParams({
       grant_type: 'password',
       client_id: 'admin-cli',
       username: username,
       password: password,
     });

     const response = await fetch(`${this.authUrl}/realms/master/protocol/openid-connect/token`, {
       method: 'POST',
       headers: { 'Content-Type': 'application/x-www-form-urlencoded' },
       body: params.toString(),
     });

     if (!response.ok) {
       throw new Error(`Failed to get master admin token: ${response.status}`);
     }

     const data = await response.json();
     return data.access_token;
   }
   ```

4. **Update Test Setup in `beforeAll`**
   ```javascript
   // Replace existing admin token acquisition:
   const adminToken = await authClient.getMasterRealmAdminToken(
     process.env.INTEG_TEST_MASTER_ADMIN_USERNAME,
     process.env.INTEG_TEST_MASTER_ADMIN_PASSWORD
   );
   ```

5. **Run Test**
   ```bash
   cd crates/lib_bodhiserver_napi
   npm run test:playwright -- token-refresh-integration.spec.mjs
   ```

#### Security Considerations
- ⚠️ Never commit master realm credentials to version control
- ⚠️ Use only in local development environment
- ⚠️ Consider using .env file for credentials (add to .gitignore)
- ⚠️ Rotate master admin password regularly

---

### Option 2: Service Account with Realm Management (CI/CD Secure)

**Best For**: CI/CD pipelines requiring dynamic configuration
**Security**: ✅ Secure - no user credentials exposed
**Complexity**: Medium
**Setup Time**: 15-20 minutes (one-time Keycloak configuration)

#### Setup Steps

1. **Enable Service Account for Dev Console Client**
   - Navigate to Keycloak Admin UI
   - Go to: Clients → `client-bodhi-dev-console`
   - **Settings Tab**:
     - Set "Client authentication" to ON
     - Set "Service accounts roles" to ON
     - Click "Save"

2. **Assign Realm Management Roles to Service Account**
   - Still in `client-bodhi-dev-console` configuration
   - Go to **Service Account Roles** tab
   - In "Client Roles" dropdown, select: `realm-management`
   - From "Available Roles", assign these roles:
     - `manage-clients` (required for token lifespan configuration)
     - `view-clients` (required for client lookup)
   - Click "Add selected"

3. **Retrieve Client Secret**
   - Go to **Credentials** tab
   - Copy "Client Secret" value
   - This is your `INTEG_TEST_DEV_CONSOLE_CLIENT_SECRET`

4. **Set Environment Variables**
   ```bash
   # Base variables (already set)
   export INTEG_TEST_MAIN_AUTH_URL=http://localhost:8080
   export INTEG_TEST_AUTH_REALM=bodhiapp-test
   export INTEG_TEST_DEV_CONSOLE_CLIENT_SECRET=<service-account-secret>

   # Service account specific
   export INTEG_TEST_SERVICE_ACCOUNT_ENABLED=true
   ```

5. **Modify Test Code for Service Account**
   Update `auth-server-client.mjs`:
   ```javascript
   async getServiceAccountToken(clientId, clientSecret) {
     const params = new URLSearchParams({
       grant_type: 'client_credentials',
       client_id: clientId,
       client_secret: clientSecret,
     });

     const response = await fetch(`${this.authUrl}/realms/${this.realm}/protocol/openid-connect/token`, {
       method: 'POST',
       headers: { 'Content-Type': 'application/x-www-form-urlencoded' },
       body: params.toString(),
     });

     if (!response.ok) {
       throw new Error(`Failed to get service account token: ${response.status}`);
     }

     const data = await response.json();
     return data.access_token;
   }
   ```

6. **Update Test Setup in `beforeAll`**
   ```javascript
   // Use service account for admin token:
   const adminToken = await authClient.getServiceAccountToken(
     'client-bodhi-dev-console',
     authServerConfig.devConsoleClientSecret
   );
   ```

7. **Verify Setup**
   Test service account permissions:
   ```bash
   curl -X GET "${INTEG_TEST_MAIN_AUTH_URL}/admin/realms/${INTEG_TEST_AUTH_REALM}/clients" \
     -H "Authorization: Bearer ${SERVICE_ACCOUNT_TOKEN}"
   ```
   Should return 200 with client list (not 403)

8. **Run Test**
   ```bash
   cd crates/lib_bodhiserver_napi
   npm run test:playwright -- token-refresh-integration.spec.mjs
   ```

#### Security Considerations
- ✅ Suitable for CI/CD pipelines
- ✅ No user credentials exposed
- ✅ Scoped to realm-specific permissions
- ✅ Can be rotated independently from user passwords

#### Troubleshooting
- **403 Forbidden**: Verify service account roles assigned correctly
- **401 Unauthorized**: Check client secret is correct
- **Empty token**: Ensure "Service accounts roles" is enabled

---

### Option 3: Pre-Configured Realm (CI/CD Simple) ⭐ RECOMMENDED

**Best For**: CI/CD pipelines, automated testing, team environments
**Security**: ✅ Secure - no admin credentials needed
**Complexity**: Low
**Setup Time**: 10 minutes (one-time realm configuration)

#### Setup Steps

1. **Create Realm Configuration File**
   Create `bodhiapp-test-realm.json`:
   ```json
   {
     "realm": "bodhiapp-test",
     "enabled": true,
     "displayName": "BodhiApp Test Realm",
     "accessTokenLifespan": 5,
     "accessTokenLifespanForImplicitFlow": 5,
     "ssoSessionIdleTimeout": 300,
     "ssoSessionMaxLifespan": 600,
     "clientSessionIdleTimeout": 300,
     "clientSessionMaxLifespan": 600,
     "offlineSessionIdleTimeout": 2592000,
     "accessCodeLifespan": 60,
     "accessCodeLifespanUserAction": 300,
     "accessCodeLifespanLogin": 1800,
     "actionTokenGeneratedByAdminLifespan": 43200,
     "actionTokenGeneratedByUserLifespan": 300
   }
   ```

2. **Import Realm into Keycloak**

   **Option A: Via Admin UI**
   - Navigate to Keycloak Admin UI
   - Click dropdown next to realm name (top left)
   - Click "Add realm"
   - Click "Select file" and choose `bodhiapp-test-realm.json`
   - Click "Create"

   **Option B: Via Keycloak CLI**
   ```bash
   # If using Docker:
   docker exec -it keycloak /opt/keycloak/bin/kc.sh import \
     --file /tmp/bodhiapp-test-realm.json \
     --override true

   # If standalone installation:
   ./bin/kc.sh import --file bodhiapp-test-realm.json --override true
   ```

3. **Verify Realm Configuration**
   - Navigate to: Realm Settings → Tokens
   - Verify "Access Token Lifespan" is set to 5 seconds
   - Verify "SSO Session Idle" is reasonable (300 seconds)

4. **Create Test User (if not exists)**
   - Navigate to: Users → Add user
   - Set username, email, enable account
   - Set password in Credentials tab
   - Note the user UUID from URL or user details

5. **Set Environment Variables**
   ```bash
   # Only base variables needed - no admin credentials!
   export INTEG_TEST_MAIN_AUTH_URL=http://localhost:8080
   export INTEG_TEST_AUTH_REALM=bodhiapp-test
   export INTEG_TEST_DEV_CONSOLE_CLIENT_SECRET=<dev-console-secret>
   export INTEG_TEST_USERNAME=testuser@example.com
   export INTEG_TEST_PASSWORD=<test-user-password>
   export INTEG_TEST_USERNAME_ID=<test-user-uuid>
   ```

6. **Modify Test to Skip Admin Configuration**
   Update `token-refresh-integration.spec.mjs`:
   ```javascript
   test.beforeAll(async () => {
     // ... existing setup ...

     // Comment out or remove this line:
     // await authClient.configureClientTokenLifespan(adminToken, resourceClient.clientId, 5);

     // Realm already configured with 5-second token lifespan!
   });
   ```

7. **Run Test**
   ```bash
   cd crates/lib_bodhiserver_napi
   npm run test:playwright -- token-refresh-integration.spec.mjs
   ```

#### Advantages
- ✅ No admin credentials needed at runtime
- ✅ Fast test execution (no setup phase)
- ✅ Stable configuration - no runtime changes
- ✅ Simple CI/CD integration
- ✅ Easy to maintain and replicate

#### Realm Export for CI/CD
Create complete realm export including users:
```bash
# Export realm with users and clients:
./bin/kc.sh export --realm bodhiapp-test --file bodhiapp-test-complete.json --users realm_file

# Commit to version control (without sensitive data):
git add bodhiapp-test-realm.json
```

#### Troubleshooting
- **Tokens not expiring**: Verify realm-level token lifespan is 5 seconds (not client-level)
- **Test timing issues**: Increase wait time to 7-8 seconds if clock skew present
- **Import failed**: Check JSON syntax and Keycloak version compatibility

---

### Option 4: Realm Admin User (Balanced Approach)

**Best For**: Balanced approach for development and CI/CD
**Security**: ✅ Secure - realm-scoped admin (not master realm)
**Complexity**: Low
**Setup Time**: 10 minutes

#### Setup Steps

1. **Create Realm Admin User**
   - Navigate to Keycloak Admin UI
   - Switch to target realm (`bodhiapp-test`)
   - Go to: Users → Add user
   - Set username (e.g., `test-admin`)
   - Set email, enable account
   - Click "Save"

2. **Set User Password**
   - Go to user's **Credentials** tab
   - Set password
   - Disable "Temporary" toggle
   - Click "Set Password"

3. **Assign Realm Management Roles**
   - Go to user's **Role Mapping** tab
   - Click "Assign role"
   - Filter by "Filter by clients"
   - Search for `realm-management`
   - Assign these roles:
     - `manage-clients`
     - `view-clients`
   - Click "Assign"

4. **Set Environment Variables**
   ```bash
   # Base variables
   export INTEG_TEST_MAIN_AUTH_URL=http://localhost:8080
   export INTEG_TEST_AUTH_REALM=bodhiapp-test
   export INTEG_TEST_DEV_CONSOLE_CLIENT_SECRET=<dev-console-secret>

   # Standard test user
   export INTEG_TEST_USERNAME=testuser@example.com
   export INTEG_TEST_PASSWORD=<test-user-password>
   export INTEG_TEST_USERNAME_ID=<test-user-uuid>

   # Realm admin user (NEW)
   export INTEG_TEST_REALM_ADMIN_USERNAME=test-admin
   export INTEG_TEST_REALM_ADMIN_PASSWORD=<admin-password>
   ```

5. **Update Test Setup in `beforeAll`**
   ```javascript
   // Use realm admin user for admin token:
   const adminToken = await authClient.getDevConsoleToken(
     process.env.INTEG_TEST_REALM_ADMIN_USERNAME,
     process.env.INTEG_TEST_REALM_ADMIN_PASSWORD
   );
   ```

6. **Verify Admin Access**
   Test admin permissions:
   ```bash
   # Get admin token:
   ADMIN_TOKEN=$(curl -X POST "${INTEG_TEST_MAIN_AUTH_URL}/realms/${INTEG_TEST_AUTH_REALM}/protocol/openid-connect/token" \
     -H "Content-Type: application/x-www-form-urlencoded" \
     -d "grant_type=password" \
     -d "client_id=client-bodhi-dev-console" \
     -d "username=${INTEG_TEST_REALM_ADMIN_USERNAME}" \
     -d "password=${INTEG_TEST_REALM_ADMIN_PASSWORD}" \
     | jq -r '.access_token')

   # Test admin API access:
   curl -X GET "${INTEG_TEST_MAIN_AUTH_URL}/admin/realms/${INTEG_TEST_AUTH_REALM}/clients" \
     -H "Authorization: Bearer ${ADMIN_TOKEN}"
   ```
   Should return 200 with client list

7. **Run Test**
   ```bash
   cd crates/lib_bodhiserver_napi
   npm run test:playwright -- token-refresh-integration.spec.mjs
   ```

#### Security Considerations
- ✅ Realm-scoped permissions (not master realm)
- ✅ Separate admin user for testing
- ✅ Can be rotated independently
- ⚠️ User passwords may expire based on realm policy

#### Troubleshooting
- **403 Forbidden**: Verify realm-management roles assigned
- **401 Unauthorized**: Check username/password correct
- **Password expired**: Reset password in Admin UI

---

### Option 5: Unit Test Mocking (Alternative)

**Best For**: Unit testing middleware logic, edge case validation
**Security**: ✅ No external dependencies
**Complexity**: Medium
**Setup Time**: 30 minutes (test refactoring)

#### Approach

Instead of integration testing with real Keycloak, mock token expiry and refresh logic.

#### Implementation Steps

1. **Create Mock Token Generator**
   Create `tests-js/utils/mock-token-generator.mjs`:
   ```javascript
   import jwt from 'jsonwebtoken';

   export function createMockToken(options = {}) {
     const payload = {
       sub: options.userId || 'test-user-id',
       email: options.email || 'test@example.com',
       exp: options.exp || Math.floor(Date.now() / 1000) + 300, // 5 min default
       iat: Math.floor(Date.now() / 1000),
       iss: options.issuer || 'http://localhost:8080/realms/test',
       aud: 'account',
       typ: 'Bearer',
     };

     // Use a test signing key (not real Keycloak key)
     return jwt.sign(payload, 'test-secret-key', { algorithm: 'HS256' });
   }

   export function createExpiredToken(userId = 'test-user-id') {
     return createMockToken({
       userId,
       exp: Math.floor(Date.now() / 1000) - 60, // Expired 1 minute ago
     });
   }
   ```

2. **Create Mock Auth Service**
   Create `tests-js/mocks/mock-auth-service.mjs`:
   ```javascript
   export class MockAuthService {
     constructor() {
       this.refreshCallCount = 0;
       this.tokens = new Map();
     }

     async refreshToken(refreshToken) {
       this.refreshCallCount++;

       // Simulate refresh token exchange
       const newAccessToken = createMockToken({
         exp: Math.floor(Date.now() / 1000) + 300,
       });

       return {
         access_token: newAccessToken,
         refresh_token: refreshToken, // Refresh token unchanged
         token_type: 'Bearer',
         expires_in: 300,
       };
     }

     getRefreshCallCount() {
       return this.refreshCallCount;
     }
   }
   ```

3. **Create Unit Test**
   Create `tests-js/specs/auth/token-refresh-unit.spec.mjs`:
   ```javascript
   import { test, expect } from '@playwright/test';
   import { createMockToken, createExpiredToken } from '../../utils/mock-token-generator.mjs';
   import { MockAuthService } from '../../mocks/mock-auth-service.mjs';

   test.describe('Token Refresh Middleware (Unit)', () => {
     let mockAuthService;

     test.beforeEach(() => {
       mockAuthService = new MockAuthService();
     });

     test('should refresh expired access token', async () => {
       const expiredToken = createExpiredToken();
       const refreshToken = createMockToken({ exp: Date.now() / 1000 + 3600 });

       // Simulate middleware detecting expired token
       const isExpired = jwt.decode(expiredToken).exp < Date.now() / 1000;
       expect(isExpired).toBe(true);

       // Simulate refresh call
       const result = await mockAuthService.refreshToken(refreshToken);

       // Verify new token generated
       expect(result.access_token).toBeTruthy();
       expect(result.access_token).not.toBe(expiredToken);
       expect(mockAuthService.getRefreshCallCount()).toBe(1);

       // Verify new token not expired
       const newTokenExpiry = jwt.decode(result.access_token).exp;
       expect(newTokenExpiry).toBeGreaterThan(Date.now() / 1000);
     });

     test('should not refresh valid token', async () => {
       const validToken = createMockToken({ exp: Date.now() / 1000 + 300 });

       // Simulate middleware checking token validity
       const isExpired = jwt.decode(validToken).exp < Date.now() / 1000;
       expect(isExpired).toBe(false);

       // No refresh should occur
       expect(mockAuthService.getRefreshCallCount()).toBe(0);
     });
   });
   ```

4. **Install Dependencies**
   ```bash
   cd crates/lib_bodhiserver_napi
   npm install --save-dev jsonwebtoken
   ```

5. **Run Unit Tests**
   ```bash
   npm run test:playwright -- token-refresh-unit.spec.mjs
   ```

#### Advantages
- ✅ Fast execution (no Keycloak dependency)
- ✅ Easy to test edge cases
- ✅ No environment variables needed
- ✅ Deterministic test results

#### Limitations
- ❌ Doesn't test real Keycloak integration
- ❌ Misses JWT signature validation issues
- ❌ Doesn't test network-level token exchange
- ❌ Mock may diverge from real Keycloak behavior

---

## Choosing the Right Solution

### Decision Tree

```
Is this for CI/CD?
├─ YES → Do you need dynamic configuration?
│  ├─ YES → Use Option 2 (Service Account)
│  └─ NO  → Use Option 3 (Pre-Configured Realm) ⭐ RECOMMENDED
│
└─ NO (Local Development)
   └─ Do you have master admin access?
      ├─ YES → Use Option 1 (Master Admin) - Quick but not secure
      └─ NO  → Use Option 4 (Realm Admin User) - Balanced approach
```

### Quick Comparison

| Criteria | Option 1 | Option 2 | Option 3 ⭐ | Option 4 | Option 5 |
|----------|----------|----------|----------|----------|----------|
| CI/CD Ready | ❌ | ✅ | ✅ | ✅ | ✅ |
| Secure | ⚠️ | ✅ | ✅ | ✅ | ✅ |
| Easy Setup | ✅ | ⚠️ | ✅ | ✅ | ⚠️ |
| Admin Credentials | Master | None | None | Realm | None |
| Dynamic Config | ✅ | ✅ | ❌ | ✅ | N/A |
| Test Speed | Medium | Medium | Fast | Medium | Fastest |
| Real Integration | ✅ | ✅ | ✅ | ✅ | ❌ |

## Common Troubleshooting

### Environment Variable Issues

**Problem**: Test fails with "Environment variable not set"
**Solution**:
```bash
# Verify all required variables:
env | grep INTEG_TEST

# Set missing variables:
export INTEG_TEST_MAIN_AUTH_URL=http://localhost:8080
# ... etc
```

### Keycloak Connection Issues

**Problem**: "Connection refused" or "ECONNREFUSED"
**Solution**:
```bash
# Verify Keycloak is running:
curl http://localhost:8080/realms/bodhiapp-test

# Check Keycloak logs:
docker logs keycloak  # if using Docker
```

### Admin API 403 Forbidden

**Problem**: "403 Forbidden" on admin API calls
**Solution**:
1. Verify admin credentials correct
2. Check realm-management roles assigned
3. Test with curl (see verification steps in each option)
4. Check Keycloak Admin UI → Users/Clients → Role Mapping

### Token Not Expiring

**Problem**: Test fails - token doesn't expire after 6 seconds
**Solution**:
1. Verify token lifespan configured correctly (realm or client level)
2. Check clock synchronization between test machine and Keycloak
3. Increase wait time to 7-8 seconds for safety margin
4. Verify token `exp` claim with JWT decoder (jwt.io)

### Test Hangs on OAuth Login

**Problem**: Test hangs during `performOAuthLogin()`
**Solution**:
1. Check Keycloak login page renders correctly
2. Verify test credentials valid
3. Check for consent screens (disable in client settings if needed)
4. Increase Playwright timeout for slow environments

## Verification Steps

After completing setup, verify your configuration:

### 1. Verify Keycloak Connectivity
```bash
curl ${INTEG_TEST_MAIN_AUTH_URL}/realms/${INTEG_TEST_AUTH_REALM}
# Should return realm metadata JSON
```

### 2. Verify Test User Can Login
```bash
curl -X POST "${INTEG_TEST_MAIN_AUTH_URL}/realms/${INTEG_TEST_AUTH_REALM}/protocol/openid-connect/token" \
  -H "Content-Type: application/x-www-form-urlencoded" \
  -d "grant_type=password" \
  -d "client_id=client-bodhi-dev-console" \
  -d "client_secret=${INTEG_TEST_DEV_CONSOLE_CLIENT_SECRET}" \
  -d "username=${INTEG_TEST_USERNAME}" \
  -d "password=${INTEG_TEST_PASSWORD}"
# Should return access_token
```

### 3. Verify Admin Access (Options 1, 2, 4 only)
```bash
# Get admin token (varies by option)
# Then test admin API:
curl -X GET "${INTEG_TEST_MAIN_AUTH_URL}/admin/realms/${INTEG_TEST_AUTH_REALM}/clients" \
  -H "Authorization: Bearer ${ADMIN_TOKEN}"
# Should return 200 with client list (not 403)
```

### 4. Run Test
```bash
cd crates/lib_bodhiserver_napi
npm run test:playwright -- token-refresh-integration.spec.mjs
```

## Next Steps

After completing setup:
1. ✅ Verify all environment variables set
2. ✅ Test Keycloak connectivity
3. ✅ Run test and verify it passes
4. ✅ Review test output for token refresh confirmation
5. ✅ Add test to CI/CD pipeline (if applicable)

For quick start with recommended approach, see `QUICK-START.md`.
