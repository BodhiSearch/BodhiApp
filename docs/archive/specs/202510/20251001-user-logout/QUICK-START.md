# Token Refresh Integration Test - Quick Start Guide

## 5-Minute Setup (Recommended Approach)

This guide provides the fastest path to run the token refresh integration test using **Option 3: Pre-Configured Realm** - no admin credentials needed!

## Prerequisites

- Keycloak server running (localhost:8080 or accessible URL)
- Node.js >= 22 installed
- 5 minutes of your time

## Step 1: Create Realm Configuration (1 minute)

Create file `bodhiapp-test-realm.json`:

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
  "clientSessionMaxLifespan": 600
}
```

## Step 2: Import Realm into Keycloak (2 minutes)

### Option A: Using Keycloak Admin UI (Easiest)
1. Open Keycloak Admin UI: http://localhost:8080/admin
2. Login with admin credentials
3. Click realm dropdown (top left) → "Add realm"
4. Click "Select file" → Choose `bodhiapp-test-realm.json`
5. Click "Create"

### Option B: Using Docker CLI (If using Docker)
```bash
# Copy realm file to container:
docker cp bodhiapp-test-realm.json keycloak:/tmp/

# Import realm:
docker exec -it keycloak /opt/keycloak/bin/kc.sh import \
  --file /tmp/bodhiapp-test-realm.json \
  --override true
```

### Option C: Using Keycloak CLI (Standalone)
```bash
./bin/kc.sh import --file bodhiapp-test-realm.json --override true
```

## Step 3: Create Dev Console Client (1 minute)

In Keycloak Admin UI for `bodhiapp-test` realm:

1. **Create Client**:
   - Go to: Clients → Create client
   - Client ID: `client-bodhi-dev-console`
   - Client Protocol: `openid-connect`
   - Click "Next"

2. **Configure Client**:
   - Client authentication: ON
   - Authorization: OFF
   - Standard flow: ON
   - Direct access grants: ON
   - Click "Save"

3. **Get Client Secret**:
   - Go to "Credentials" tab
   - Copy "Client Secret" value
   - Save for environment variables

## Step 4: Create Test User (1 minute)

1. **Create User**:
   - Go to: Users → Add user
   - Username: `testuser`
   - Email: `testuser@example.com`
   - Email verified: ON
   - Enabled: ON
   - Click "Create"

2. **Set Password**:
   - Go to "Credentials" tab
   - Set password (e.g., `TestPass123!`)
   - Temporary: OFF
   - Click "Set Password"

3. **Get User ID**:
   - Look at browser URL: `...users/{USER-UUID}/...`
   - Copy UUID (looks like: `a1b2c3d4-e5f6-...`)
   - Save for environment variables

## Step 5: Set Environment Variables (30 seconds)

Create `.env` file or add to your shell profile:

```bash
# Keycloak configuration
export INTEG_TEST_MAIN_AUTH_URL=http://localhost:8080
export INTEG_TEST_AUTH_REALM=bodhiapp-test
export INTEG_TEST_DEV_CONSOLE_CLIENT_SECRET=<paste-client-secret-from-step-3>

# Test user credentials
export INTEG_TEST_USERNAME=testuser@example.com
export INTEG_TEST_PASSWORD=TestPass123!
export INTEG_TEST_USERNAME_ID=<paste-user-uuid-from-step-4>
```

Load environment variables:
```bash
source .env  # if using .env file
# OR just export them directly in terminal
```

## Step 6: Modify Test (30 seconds)

Edit `crates/lib_bodhiserver_napi/tests-js/specs/auth/token-refresh-integration.spec.mjs`:

Find this section in `beforeAll`:
```javascript
// Configure client with 5-second token lifespan
await authClient.configureClientTokenLifespan(adminToken, resourceClient.clientId, 5);
```

Comment it out:
```javascript
// Realm already configured with 5-second token lifespan!
// await authClient.configureClientTokenLifespan(adminToken, resourceClient.clientId, 5);
```

Also comment out admin token acquisition (few lines above):
```javascript
// const adminToken = await authClient.getDevConsoleToken(username, password);
```

## Step 7: Run Test (30 seconds)

```bash
cd /Users/amir36/Documents/workspace/src/github.com/BodhiSearch/BodhiApp/crates/lib_bodhiserver_napi
npm run test:playwright -- token-refresh-integration.spec.mjs
```

## Expected Output

```
✓ Token Refresh Integration › should refresh expired access token automatically (15s)

Test Results:
- OAuth login completed successfully
- Initial access token retrieved
- Token expired after 6 seconds
- Token refresh occurred automatically
- New access token different from original
- Token reused correctly without re-refresh

1 passed (15s)
```

## Verification Checklist

After test completes, verify:
- [ ] Test status shows "passed"
- [ ] Console logs show "Initial access token (first 20 chars): ..."
- [ ] Console logs show "Refreshed access token (first 20 chars): ..." with different prefix
- [ ] Console logs show "Token refresh verified successfully"
- [ ] No 401 or 403 errors in output
- [ ] Test completed in ~15-20 seconds

## Troubleshooting

### Issue: "Environment variable INTEG_TEST_MAIN_AUTH_URL not set"
**Solution**: Verify environment variables loaded
```bash
env | grep INTEG_TEST
# Should show all 6 variables
```

### Issue: "Connection refused" or "ECONNREFUSED"
**Solution**: Verify Keycloak is running
```bash
curl http://localhost:8080/realms/bodhiapp-test
# Should return JSON with realm metadata
```

### Issue: Test hangs during login
**Solution**: Verify test credentials work
```bash
curl -X POST "http://localhost:8080/realms/bodhiapp-test/protocol/openid-connect/token" \
  -H "Content-Type: application/x-www-form-urlencoded" \
  -d "grant_type=password" \
  -d "client_id=client-bodhi-dev-console" \
  -d "client_secret=${INTEG_TEST_DEV_CONSOLE_CLIENT_SECRET}" \
  -d "username=${INTEG_TEST_USERNAME}" \
  -d "password=${INTEG_TEST_PASSWORD}"
# Should return access_token (not error)
```

### Issue: Token not expiring (test fails on assertion)
**Solution**: Verify realm token lifespan
1. Keycloak Admin UI → Realm Settings → Tokens
2. Check "Access Token Lifespan" = 5 seconds
3. If not, update and save
4. Re-run test

### Issue: Test fails with "User not found" or "Invalid credentials"
**Solution**: Double-check username, password, and user ID
1. Verify username matches exactly (case-sensitive)
2. Verify password is correct (no typos)
3. Verify user ID is correct UUID from Keycloak
4. Check user is enabled in Keycloak

## What This Test Validates

When the test passes, it confirms:

1. **OAuth Flow Works**: User can login via OAuth2 password grant
2. **Session Management**: Tokens stored correctly in session
3. **Token Expiry Detection**: Middleware detects expired access tokens
4. **Automatic Refresh**: Expired tokens refreshed transparently
5. **Token Reuse**: Refreshed tokens reused without unnecessary re-refreshes
6. **No User Disruption**: Token refresh happens without 401 errors to client

## Next Steps

### For CI/CD Integration
1. **Create realm export**: Export realm configuration from Keycloak
2. **Add to pipeline**: Import realm in CI setup phase
3. **Use environment secrets**: Store client secret and credentials securely
4. **Run in pipeline**: Add test to test suite

Example CI configuration (GitHub Actions):
```yaml
jobs:
  test:
    steps:
      - name: Start Keycloak
        run: docker run -d -p 8080:8080 quay.io/keycloak/keycloak:latest start-dev

      - name: Import test realm
        run: |
          docker cp bodhiapp-test-realm.json keycloak:/tmp/
          docker exec keycloak /opt/keycloak/bin/kc.sh import --file /tmp/bodhiapp-test-realm.json

      - name: Run token refresh test
        env:
          INTEG_TEST_MAIN_AUTH_URL: http://localhost:8080
          INTEG_TEST_AUTH_REALM: bodhiapp-test
          INTEG_TEST_DEV_CONSOLE_CLIENT_SECRET: ${{ secrets.DEV_CONSOLE_SECRET }}
          INTEG_TEST_USERNAME: testuser@example.com
          INTEG_TEST_PASSWORD: ${{ secrets.TEST_PASSWORD }}
          INTEG_TEST_USERNAME_ID: ${{ secrets.TEST_USER_ID }}
        run: npm run test:playwright -- token-refresh-integration.spec.mjs
```

### For Local Development
1. **Add to test suite**: Run with other integration tests
2. **Document setup**: Share this guide with team members
3. **Maintain realm**: Keep realm configuration in version control
4. **Update as needed**: Adjust token lifespan for different test scenarios

### For Alternative Approaches
If this approach doesn't work for your environment, see:
- **SETUP.md**: All 5 solution options with detailed instructions
- **README.md**: Complete project overview and decision matrix

## Summary

You've successfully:
- ✅ Created test realm with 5-second token lifespan
- ✅ Configured dev console client
- ✅ Created test user with credentials
- ✅ Set environment variables
- ✅ Modified test to use pre-configured realm
- ✅ Ran test and verified token refresh works

**Total Setup Time**: ~5 minutes
**Ongoing Maintenance**: Minimal (realm configuration persists)

## Support

Having issues? Check:
1. **This guide**: Review troubleshooting section above
2. **SETUP.md**: Detailed setup for all options
3. **README.md**: Complete project documentation
4. **agent-log.md**: Implementation details and discoveries

For questions about implementation details, see `agent-ctx.md` for technical insights.

---

**Quick Start Guide Version**: 1.0
**Last Updated**: 2025-10-02
**Recommended For**: CI/CD pipelines, automated testing, team environments
