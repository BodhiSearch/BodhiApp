import { LoginPage } from '@/pages/LoginPage.mjs';
import { TokensPage } from '@/pages/TokensPage.mjs';
import { ToolsetsPage } from '@/pages/ToolsetsPage.mjs';
import { OAuth2TestAppPage } from '@/pages/OAuth2TestAppPage.mjs';
import { randomPort } from '@/test-helpers.mjs';
import {
  createAuthServerTestClient,
  getAuthServerConfig,
  getTestCredentials,
} from '@/utils/auth-server-client.mjs';
import { createServerManager } from '@/utils/bodhi-app-server.mjs';
import { createStaticServer } from '@/utils/static-server.mjs';
import { expect, test } from '@playwright/test';

/**
 * Toolsets Authentication Restrictions E2E Tests
 *
 * Test Matrix for OAuth + Toolset Scope Combinations:
 * | # | App Client Config    | OAuth Scope Request  | Expected Result                              |
 * |---|----------------------|----------------------|----------------------------------------------|
 * | 1 | WITH toolset scope   | WITH toolset scope   | Token has scope -> List returns toolset      |
 * | 2 | WITH toolset scope   | WITHOUT toolset scope| Token lacks scope -> List returns empty      |
 * | 3 | WITHOUT toolset scope| WITH toolset scope   | Keycloak error (invalid_scope)               |
 * | 4 | WITHOUT toolset scope| WITHOUT toolset scope| Token lacks scope -> List returns empty      |
 *
 * Additional tests:
 * - API tokens (bodhiapp_*) are blocked from ALL toolset endpoints (401)
 * - OAuth tokens are blocked from config endpoints (session-only)
 */

const TOOLSET_ID = 'builtin-exa-web-search';
const TOOLSET_SCOPE = 'scope_toolset-builtin-exa-web-search';

test.describe('API Token Blocking - Toolset Endpoints', () => {
  let authServerConfig;
  let testCredentials;
  let serverManager;
  let baseUrl;
  let authClient;
  let resourceClient;
  let apiToken;
  let toolsetUuid;

  test.beforeAll(async ({ browser }) => {
    authServerConfig = getAuthServerConfig();
    testCredentials = getTestCredentials();
    const port = randomPort();
    const serverUrl = `http://localhost:${port}`;

    authClient = createAuthServerTestClient(authServerConfig);
    resourceClient = await authClient.createResourceClient(serverUrl);
    await authClient.makeResourceAdmin(
      resourceClient.clientId,
      resourceClient.clientSecret,
      testCredentials.userId
    );

    serverManager = createServerManager({
      appStatus: 'ready',
      authUrl: authServerConfig.authUrl,
      authRealm: authServerConfig.authRealm,
      clientId: resourceClient.clientId,
      clientSecret: resourceClient.clientSecret,
      port,
      host: 'localhost',
      logLevel: 'debug',
    });

    baseUrl = await serverManager.startServer();

    // Create an API token via session auth for testing
    const context = await browser.newContext();
    const page = await context.newPage();
    const loginPage = new LoginPage(page, baseUrl, authServerConfig, testCredentials);
    const tokensPage = new TokensPage(page, baseUrl);

    await loginPage.performOAuthLogin();
    await tokensPage.navigateToTokens();
    await tokensPage.createToken('toolset-test-token');
    await tokensPage.expectTokenDialog();
    apiToken = await tokensPage.copyTokenFromDialog();
    await tokensPage.closeTokenDialog();

    // Create a toolset to get its UUID for testing
    const toolsetsPage = new ToolsetsPage(page, baseUrl);
    const exaApiKey = process.env.INTEG_TEST_EXA_API_KEY;
    expect(exaApiKey, 'INTEG_TEST_EXA_API_KEY not found in env').toBeDefined();
    expect(exaApiKey, 'INTEG_TEST_EXA_API_KEY not found in env').not.toBeNull();
    await toolsetsPage.configureToolsetWithApiKey('builtin-exa-web-search', exaApiKey);

    // Get the UUID from the data-test-uuid attribute
    await toolsetsPage.navigateToToolsetsList();
    toolsetUuid = await toolsetsPage.getToolsetUuidByType('builtin-exa-web-search');

    await context.close();
  });

  test.afterAll(async () => {
    if (serverManager) {
      await serverManager.stopServer();
    }
  });

  test('GET /toolsets with API token returns 401 Unauthorized', async () => {
    const response = await fetch(`${baseUrl}/bodhi/v1/toolsets`, {
      headers: {
        Authorization: `Bearer ${apiToken}`,
        'Content-Type': 'application/json',
      },
    });

    // API tokens are blocked at route level - returns 401 (missing auth for this route type)
    expect(response.status).toBe(401);
  });

  test('GET /toolsets/{id} with API token returns 401 Unauthorized', async () => {
    const response = await fetch(`${baseUrl}/bodhi/v1/toolsets/${toolsetUuid}`, {
      headers: {
        Authorization: `Bearer ${apiToken}`,
        'Content-Type': 'application/json',
      },
    });

    expect(response.status).toBe(401);
  });

  test('PUT /toolsets/{id} with API token returns 401 Unauthorized', async () => {
    const response = await fetch(`${baseUrl}/bodhi/v1/toolsets/${toolsetUuid}`, {
      method: 'PUT',
      headers: {
        Authorization: `Bearer ${apiToken}`,
        'Content-Type': 'application/json',
      },
      body: JSON.stringify({
        enabled: true,
        api_key: 'test-key',
      }),
    });

    expect(response.status).toBe(401);
  });

  test('DELETE /toolsets/{id} with API token returns 401 Unauthorized', async () => {
    const response = await fetch(`${baseUrl}/bodhi/v1/toolsets/${toolsetUuid}`, {
      method: 'DELETE',
      headers: {
        Authorization: `Bearer ${apiToken}`,
        'Content-Type': 'application/json',
      },
    });

    expect(response.status).toBe(401);
  });

  test('POST /toolsets/{id}/execute/{method} with API token returns 401 Unauthorized', async () => {
    const response = await fetch(`${baseUrl}/bodhi/v1/toolsets/${TOOLSET_ID}/execute/search`, {
      method: 'POST',
      headers: {
        Authorization: `Bearer ${apiToken}`,
        'Content-Type': 'application/json',
      },
      body: JSON.stringify({
        tool_call_id: 'call_test',
        params: { query: 'test' },
      }),
    });

    expect(response.status).toBe(401);
  });
});

test.describe('OAuth Token + Toolset Scope Combinations', () => {
  let authServerConfig;
  let testCredentials;
  let authClient;
  let serverManager;
  let staticServer;
  let resourceClient;
  let baseUrl;
  let testAppUrl;
  let port;

  test.beforeAll(async () => {
    authServerConfig = getAuthServerConfig();
    testCredentials = getTestCredentials();
    authClient = createAuthServerTestClient(authServerConfig);
  });

  test.beforeEach(async () => {
    port = randomPort();
    const serverUrl = `http://localhost:${port}`;

    // Create resource client directly (no UI setup needed)
    resourceClient = await authClient.createResourceClient(serverUrl);
    await authClient.makeResourceAdmin(
      resourceClient.clientId,
      resourceClient.clientSecret,
      testCredentials.userId
    );

    // Start server in ready state with resource client credentials
    serverManager = createServerManager({
      appStatus: 'ready',
      authUrl: authServerConfig.authUrl,
      authRealm: authServerConfig.authRealm,
      clientId: resourceClient.clientId,
      clientSecret: resourceClient.clientSecret,
      port,
      host: 'localhost',
    });
    baseUrl = await serverManager.startServer();

    // Setup static server for OAuth test app
    const appPort = randomPort();
    staticServer = createStaticServer(appPort);
    testAppUrl = await staticServer.startServer();
  });

  test.afterEach(async () => {
    if (staticServer) {
      await staticServer.stopServer();
    }
    if (serverManager) {
      await serverManager.stopServer();
    }
  });

  /**
   * Test Matrix Case 1:
   * App Client: WITH toolset scope
   * OAuth Request: WITH toolset scope
   * Expected: Token has scope -> List returns toolset + can execute tool
   *
   * This test performs end-to-end verification including live tool execution via OAuth exchanged token.
   * It requires INTEG_TEST_EXA_API_KEY environment variable to be set.
   */
  test('App WITH toolset scope + OAuth WITH toolset scope returns toolset in list and can execute', async ({
    page,
    browser,
  }) => {
    // Check API key environment variable - fail if not present
    const exaApiKey = process.env.INTEG_TEST_EXA_API_KEY;
    expect(exaApiKey, 'INTEG_TEST_EXA_API_KEY environment variable is required').toBeTruthy();

    // Phase 1: Session login to configure toolset with API key
    // Use a new browser context for session login to avoid cookie conflicts
    const sessionContext = await browser.newContext();
    const sessionPage = await sessionContext.newPage();

    const loginPage = new LoginPage(sessionPage, baseUrl, authServerConfig, testCredentials);
    await loginPage.performOAuthLogin();

    // Configure Exa toolset with API key via session auth
    const toolsetsPage = new ToolsetsPage(sessionPage, baseUrl);
    await toolsetsPage.configureToolsetWithApiKey(TOOLSET_ID, exaApiKey);

    // Close session context - we'll use OAuth token from here on
    await sessionContext.close();

    // Get dev console token for client management
    const devConsoleToken = await authClient.getDevConsoleToken(
      testCredentials.username,
      testCredentials.password
    );

    // Create app client WITH toolset scope configured
    const redirectUri = `${testAppUrl}/oauth-test-app.html`;
    const appClient = await authClient.createAppClient(
      devConsoleToken,
      port,
      'toolsets-test-case1-with-scope',
      'Test client for toolset OAuth - Case 1: app with scope, oauth with scope',
      [redirectUri],
      [authServerConfig.toolsetScopeExaWebSearchId] // Pass toolset scope ID
    );

    // Request audience access via Bodhi App API, including toolset_scope_ids to register with resource-client
    const requestAccessResponse = await fetch(`${baseUrl}/bodhi/v1/apps/request-access`, {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({
        app_client_id: appClient.clientId,
        toolset_scope_ids: [authServerConfig.toolsetScopeExaWebSearchId],
      }),
    });
    expect(requestAccessResponse.status).toBe(200);
    const requestAccessData = await requestAccessResponse.json();
    const resourceScope = requestAccessData.scope;

    // Phase 2: OAuth flow via test app
    // Navigate to test app and complete OAuth flow WITH toolset scope
    const oauth2TestAppPage = new OAuth2TestAppPage(page, testAppUrl);
    await oauth2TestAppPage.navigateToTestApp(redirectUri);

    // Include toolset scope in the OAuth request
    const fullScopes = `openid profile email scope_user_user ${resourceScope} ${TOOLSET_SCOPE}`;
    await oauth2TestAppPage.configureOAuthForm(
      authServerConfig.authUrl,
      authServerConfig.authRealm,
      appClient.clientId,
      redirectUri,
      fullScopes
    );

    await oauth2TestAppPage.startOAuthFlow();
    await oauth2TestAppPage.waitForAuthServerRedirect(authServerConfig.authUrl);
    await oauth2TestAppPage.handleLogin(testCredentials.username, testCredentials.password);
    await oauth2TestAppPage.handleConsent();
    await oauth2TestAppPage.waitForTokenExchange(testAppUrl);

    const accessToken = await oauth2TestAppPage.getAccessToken();
    expect(accessToken).toBeTruthy();

    // Phase 3: Verification
    // Test: GET /toolsets with OAuth token returns filtered list containing the toolset
    const response = await fetch(`${baseUrl}/bodhi/v1/toolsets`, {
      headers: {
        Authorization: `Bearer ${accessToken}`,
        'Content-Type': 'application/json',
      },
    });

    expect(response.status).toBe(200);
    const data = await response.json();
    expect(data.toolsets).toBeDefined();
    expect(Array.isArray(data.toolsets)).toBe(true);

    // Should contain the toolset we have the scope for
    const exaToolset = data.toolsets.find((t) => t.toolset_type === TOOLSET_ID);
    expect(exaToolset).toBeTruthy();

    // Execute the toolset using OAuth token
    const executeResponse = await fetch(
      `${baseUrl}/bodhi/v1/toolsets/${exaToolset.id}/execute/search`,
      {
        method: 'POST',
        headers: {
          Authorization: `Bearer ${accessToken}`,
          'Content-Type': 'application/json',
        },
        body: JSON.stringify({
          tool_call_id: 'test_call_oauth',
          params: {
            query: 'latest news about AI from San Francisco',
            num_results: 3,
          },
        }),
      }
    );

    const executeData = await executeResponse.json();
    expect(executeResponse.status).toBe(200);

    // Verify response structure matches ToolsetExecutionResponse
    expect(executeData.tool_call_id).toBe('test_call_oauth');
    expect(executeData.result).toBeDefined();
    expect(executeData.error).toBeUndefined();

    // Verify result contains query-related keywords
    const resultStr = JSON.stringify(executeData.result).toLowerCase();
    expect(
      resultStr.includes('san francisco') ||
        resultStr.includes('ai') ||
        resultStr.includes('artificial intelligence')
    ).toBe(true);
  });

  /**
   * Test Matrix Case 2:
   * App Client: WITH toolset scope
   * OAuth Request: WITHOUT toolset scope
   * Expected: Token lacks scope -> List returns empty
   */
  test('App WITH toolset scope + OAuth WITHOUT toolset scope returns empty list', async ({
    page,
  }) => {
    // Get dev console token for client management
    const devConsoleToken = await authClient.getDevConsoleToken(
      testCredentials.username,
      testCredentials.password
    );

    // Create app client WITH toolset scope configured (but we won't request it in OAuth)
    const redirectUri = `${testAppUrl}/oauth-test-app.html`;
    const appClient = await authClient.createAppClient(
      devConsoleToken,
      port,
      'toolsets-test-case2-no-oauth-scope',
      'Test client for toolset OAuth - Case 2: app with scope, oauth without scope',
      [redirectUri],
      [authServerConfig.toolsetScopeExaWebSearchId] // Pass toolset scope ID
    );

    // Request audience access, including toolset_scope_ids to register with resource-client
    // (even though we won't request it in OAuth, the scope still needs to be on resource-client)
    const requestAccessResponse = await fetch(`${baseUrl}/bodhi/v1/apps/request-access`, {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({
        app_client_id: appClient.clientId,
        toolset_scope_ids: [authServerConfig.toolsetScopeExaWebSearchId],
      }),
    });
    expect(requestAccessResponse.status).toBe(200);
    const requestAccessData = await requestAccessResponse.json();
    const resourceScope = requestAccessData.scope;

    // Complete OAuth flow WITHOUT toolset scope in the request
    const oauth2TestAppPage = new OAuth2TestAppPage(page, testAppUrl);
    await oauth2TestAppPage.navigateToTestApp(redirectUri);

    // Do NOT include toolset scope - just basic scopes
    const fullScopes = `openid profile email scope_user_user ${resourceScope}`;
    await oauth2TestAppPage.configureOAuthForm(
      authServerConfig.authUrl,
      authServerConfig.authRealm,
      appClient.clientId,
      redirectUri,
      fullScopes
    );

    await oauth2TestAppPage.startOAuthFlow();
    await oauth2TestAppPage.waitForAuthServerRedirect(authServerConfig.authUrl);
    await oauth2TestAppPage.handleLogin(testCredentials.username, testCredentials.password);
    await oauth2TestAppPage.handleConsent();
    await oauth2TestAppPage.waitForTokenExchange(testAppUrl);

    const accessToken = await oauth2TestAppPage.getAccessToken();
    expect(accessToken).toBeTruthy();

    // Test: GET /toolsets with OAuth token (no toolset scope) returns empty list
    const response = await fetch(`${baseUrl}/bodhi/v1/toolsets`, {
      headers: {
        Authorization: `Bearer ${accessToken}`,
        'Content-Type': 'application/json',
      },
    });

    expect(response.status).toBe(200);
    const data = await response.json();
    expect(data.toolsets).toBeDefined();
    expect(Array.isArray(data.toolsets)).toBe(true);

    // Without toolset scope in token, should return empty list
    expect(data.toolsets.length).toBe(0);
  });

  /**
   * Test Matrix Case 3:
   * App Client: WITHOUT toolset scope
   * OAuth Request: WITH toolset scope
   * Expected: Keycloak error (invalid_scope)
   */
  test('App WITHOUT toolset scope + OAuth WITH toolset scope returns invalid_scope error', async ({
    page,
  }) => {
    // Get dev console token for client management
    const devConsoleToken = await authClient.getDevConsoleToken(
      testCredentials.username,
      testCredentials.password
    );

    // Create app client WITHOUT toolset scope configured
    const redirectUri = `${testAppUrl}/oauth-test-app.html`;
    const appClient = await authClient.createAppClient(
      devConsoleToken,
      port,
      'toolsets-test-case3-no-app-scope',
      'Test client for toolset OAuth - Case 3: app without scope, oauth with scope',
      [redirectUri]
      // No toolset scope IDs - app client not authorized for toolset scope
    );

    // Request audience access
    const requestAccessResponse = await fetch(`${baseUrl}/bodhi/v1/apps/request-access`, {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({ app_client_id: appClient.clientId }),
    });
    expect(requestAccessResponse.status).toBe(200);
    const requestAccessData = await requestAccessResponse.json();
    const resourceScope = requestAccessData.scope;

    // Navigate to test app and try OAuth flow WITH toolset scope (should fail)
    const oauth2TestAppPage = new OAuth2TestAppPage(page, testAppUrl);
    await oauth2TestAppPage.navigateToTestApp(redirectUri);

    // Try to include toolset scope in the OAuth request (not authorized on app client)
    const fullScopes = `openid profile email scope_user_user ${resourceScope} ${TOOLSET_SCOPE}`;
    await oauth2TestAppPage.configureOAuthForm(
      authServerConfig.authUrl,
      authServerConfig.authRealm,
      appClient.clientId,
      redirectUri,
      fullScopes
    );

    await oauth2TestAppPage.startOAuthFlow();

    // Keycloak should reject this with invalid_scope error
    const errorResult = await oauth2TestAppPage.expectOAuthError('invalid_scope');
    expect(errorResult.error).toBe('invalid_scope');
  });

  /**
   * Test Matrix Case 4:
   * App Client: WITHOUT toolset scope
   * OAuth Request: WITHOUT toolset scope
   * Expected: Token lacks scope -> List returns empty
   */
  test('App WITHOUT toolset scope + OAuth WITHOUT toolset scope returns empty list', async ({
    page,
  }) => {
    // Get dev console token for client management
    const devConsoleToken = await authClient.getDevConsoleToken(
      testCredentials.username,
      testCredentials.password
    );

    // Create app client WITHOUT toolset scope configured
    const redirectUri = `${testAppUrl}/oauth-test-app.html`;
    const appClient = await authClient.createAppClient(
      devConsoleToken,
      port,
      'toolsets-test-case4-no-scope-anywhere',
      'Test client for toolset OAuth - Case 4: no scope anywhere',
      [redirectUri]
      // No toolset scope IDs
    );

    // Request audience access
    const requestAccessResponse = await fetch(`${baseUrl}/bodhi/v1/apps/request-access`, {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({ app_client_id: appClient.clientId }),
    });
    expect(requestAccessResponse.status).toBe(200);
    const requestAccessData = await requestAccessResponse.json();
    const resourceScope = requestAccessData.scope;

    // Complete OAuth flow WITHOUT toolset scope
    const oauth2TestAppPage = new OAuth2TestAppPage(page, testAppUrl);
    await oauth2TestAppPage.navigateToTestApp(redirectUri);

    // No toolset scope - just basic scopes
    const fullScopes = `openid profile email scope_user_user ${resourceScope}`;
    await oauth2TestAppPage.configureOAuthForm(
      authServerConfig.authUrl,
      authServerConfig.authRealm,
      appClient.clientId,
      redirectUri,
      fullScopes
    );

    await oauth2TestAppPage.startOAuthFlow();
    await oauth2TestAppPage.waitForAuthServerRedirect(authServerConfig.authUrl);
    await oauth2TestAppPage.handleLogin(testCredentials.username, testCredentials.password);
    await oauth2TestAppPage.handleConsent();
    await oauth2TestAppPage.waitForTokenExchange(testAppUrl);

    const accessToken = await oauth2TestAppPage.getAccessToken();
    expect(accessToken).toBeTruthy();

    // Test: GET /toolsets with OAuth token (no toolset scope) returns empty list
    const response = await fetch(`${baseUrl}/bodhi/v1/toolsets`, {
      headers: {
        Authorization: `Bearer ${accessToken}`,
        'Content-Type': 'application/json',
      },
    });

    expect(response.status).toBe(200);
    const data = await response.json();
    expect(data.toolsets).toBeDefined();
    expect(Array.isArray(data.toolsets)).toBe(true);

    // Without toolset scope in token, should return empty list
    expect(data.toolsets.length).toBe(0);
  });
});

test.describe('OAuth Token - Toolset CRUD Endpoints (Session-Only)', () => {
  let authServerConfig;
  let testCredentials;
  let authClient;
  let serverManager;
  let staticServer;
  let resourceClient;
  let baseUrl;
  let testAppUrl;
  let port;
  let toolsetUuid;

  test.beforeAll(async () => {
    authServerConfig = getAuthServerConfig();
    testCredentials = getTestCredentials();
    authClient = createAuthServerTestClient(authServerConfig);
  });

  test.beforeEach(async ({ browser }) => {
    port = randomPort();
    const serverUrl = `http://localhost:${port}`;

    // Create resource client directly (no UI setup needed)
    resourceClient = await authClient.createResourceClient(serverUrl);
    await authClient.makeResourceAdmin(
      resourceClient.clientId,
      resourceClient.clientSecret,
      testCredentials.userId
    );

    // Start server in ready state with resource client credentials
    serverManager = createServerManager({
      appStatus: 'ready',
      authUrl: authServerConfig.authUrl,
      authRealm: authServerConfig.authRealm,
      clientId: resourceClient.clientId,
      clientSecret: resourceClient.clientSecret,
      port,
      host: 'localhost',
    });
    baseUrl = await serverManager.startServer();

    // Create a real toolset via session auth to get its UUID
    const sessionContext = await browser.newContext();
    const sessionPage = await sessionContext.newPage();
    const loginPage = new LoginPage(sessionPage, baseUrl, authServerConfig, testCredentials);
    await loginPage.performOAuthLogin();

    // Configure Exa toolset to create an instance
    const toolsetsPage = new ToolsetsPage(sessionPage, baseUrl);
    const exaApiKey = process.env.INTEG_TEST_EXA_API_KEY;
    expect(exaApiKey, 'INTEG_TEST_EXA_API_KEY not found in env').toBeDefined();
    expect(exaApiKey, 'INTEG_TEST_EXA_API_KEY not found in env').not.toBeNull();
    await toolsetsPage.configureToolsetWithApiKey(TOOLSET_ID, exaApiKey);

    // Get the UUID from the data-test-uuid attribute
    await toolsetsPage.navigateToToolsetsList();
    toolsetUuid = await toolsetsPage.getToolsetUuidByType('builtin-exa-web-search');

    await sessionContext.close();

    // Setup static server for OAuth test app
    const appPort = randomPort();
    staticServer = createStaticServer(appPort);
    testAppUrl = await staticServer.startServer();
  });

  test.afterEach(async () => {
    if (staticServer) {
      await staticServer.stopServer();
    }
    if (serverManager) {
      await serverManager.stopServer();
    }
  });

  test('GET /toolsets/{id} with OAuth token returns 401 (session-only)', async ({ page }) => {
    // Get dev console token
    const devConsoleToken = await authClient.getDevConsoleToken(
      testCredentials.username,
      testCredentials.password
    );

    // Create app client WITH toolset scope (so OAuth flow succeeds)
    const redirectUri = `${testAppUrl}/oauth-test-app.html`;
    const appClient = await authClient.createAppClient(
      devConsoleToken,
      port,
      'toolsets-crud-test-get',
      'Test client for GET /toolsets/{id} endpoint',
      [redirectUri],
      [authServerConfig.toolsetScopeExaWebSearchId]
    );

    // Request audience access, including toolset_scope_ids to register with resource-client
    const requestAccessResponse = await fetch(`${baseUrl}/bodhi/v1/apps/request-access`, {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({
        app_client_id: appClient.clientId,
        toolset_scope_ids: [authServerConfig.toolsetScopeExaWebSearchId],
      }),
    });
    const requestAccessData = await requestAccessResponse.json();
    const resourceScope = requestAccessData.scope;

    // Complete OAuth flow
    const oauth2TestAppPage = new OAuth2TestAppPage(page, testAppUrl);
    await oauth2TestAppPage.navigateToTestApp(redirectUri);
    const fullScopes = `openid profile email scope_user_user ${resourceScope} ${TOOLSET_SCOPE}`;
    await oauth2TestAppPage.configureOAuthForm(
      authServerConfig.authUrl,
      authServerConfig.authRealm,
      appClient.clientId,
      redirectUri,
      fullScopes
    );
    await oauth2TestAppPage.startOAuthFlow();
    await oauth2TestAppPage.waitForAuthServerRedirect(authServerConfig.authUrl);
    await oauth2TestAppPage.handleLogin(testCredentials.username, testCredentials.password);
    await oauth2TestAppPage.handleConsent();
    await oauth2TestAppPage.waitForTokenExchange(testAppUrl);
    const accessToken = await oauth2TestAppPage.getAccessToken();

    // Test: OAuth tokens are blocked for /toolsets/{id} endpoint (session-only)
    const response = await fetch(`${baseUrl}/bodhi/v1/toolsets/${toolsetUuid}`, {
      headers: {
        Authorization: `Bearer ${accessToken}`,
        'Content-Type': 'application/json',
      },
    });

    // OAuth tokens should be rejected
    expect(response.status).toBe(401);
  });

  test('PUT /toolsets/{id} with OAuth token returns 401 (session-only)', async ({ page }) => {
    const devConsoleToken = await authClient.getDevConsoleToken(
      testCredentials.username,
      testCredentials.password
    );

    // Create app client WITH toolset scope (so OAuth flow succeeds)
    const redirectUri = `${testAppUrl}/oauth-test-app.html`;
    const appClient = await authClient.createAppClient(
      devConsoleToken,
      port,
      'toolsets-crud-test-put',
      'Test client for PUT /toolsets/{id} endpoint',
      [redirectUri],
      [authServerConfig.toolsetScopeExaWebSearchId]
    );

    // Request audience access, including toolset_scope_ids to register with resource-client
    const requestAccessResponse = await fetch(`${baseUrl}/bodhi/v1/apps/request-access`, {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({
        app_client_id: appClient.clientId,
        toolset_scope_ids: [authServerConfig.toolsetScopeExaWebSearchId],
      }),
    });
    const requestAccessData = await requestAccessResponse.json();
    const resourceScope = requestAccessData.scope;

    const oauth2TestAppPage = new OAuth2TestAppPage(page, testAppUrl);
    await oauth2TestAppPage.navigateToTestApp(redirectUri);
    const fullScopes = `openid profile email scope_user_user ${resourceScope} ${TOOLSET_SCOPE}`;
    await oauth2TestAppPage.configureOAuthForm(
      authServerConfig.authUrl,
      authServerConfig.authRealm,
      appClient.clientId,
      redirectUri,
      fullScopes
    );
    await oauth2TestAppPage.startOAuthFlow();
    await oauth2TestAppPage.waitForAuthServerRedirect(authServerConfig.authUrl);
    await oauth2TestAppPage.handleLogin(testCredentials.username, testCredentials.password);
    await oauth2TestAppPage.handleConsent();
    await oauth2TestAppPage.waitForTokenExchange(testAppUrl);
    const accessToken = await oauth2TestAppPage.getAccessToken();

    // Test: OAuth tokens are blocked for /toolsets/{id} endpoint (session-only)
    const response = await fetch(`${baseUrl}/bodhi/v1/toolsets/${toolsetUuid}`, {
      method: 'PUT',
      headers: {
        Authorization: `Bearer ${accessToken}`,
        'Content-Type': 'application/json',
      },
      body: JSON.stringify({
        name: 'Updated-OAuth',
        description: 'Updated from OAuth test',
        enabled: false,
        api_key: { action: 'Keep' },
      }),
    });

    // OAuth tokens should be rejected
    expect(response.status).toBe(401);
  });
});
