import { AccessRequestReviewPage } from '@/pages/AccessRequestReviewPage.mjs';
import { LoginPage } from '@/pages/LoginPage.mjs';
import { ToolsetsPage } from '@/pages/ToolsetsPage.mjs';
import { OAuth2TestAppPage } from '@/pages/OAuth2TestAppPage.mjs';
import {
  getAuthServerConfig,
  getPreConfiguredAppClient,
  getPreConfiguredResourceClient,
  getTestCredentials,
} from '@/utils/auth-server-client.mjs';
import { createServerManager } from '@/utils/bodhi-app-server.mjs';
import { createStaticServer } from '@/utils/static-server.mjs';
import { expect, test } from '@playwright/test';

/**
 * Toolsets Authentication Restrictions E2E Tests
 *
 * Test Matrix for OAuth + Toolset Scope Combinations:
 * | # | Access Request Config | OAuth Scope Request          | Expected Result                              |
 * |---|----------------------|------------------------------|----------------------------------------------|
 * | 1 | WITH toolsets         | WITH access_request_scope    | Token has scope -> List returns toolset      |
 * | 2 | WITH toolsets         | WITHOUT access_request_scope | Token lacks scope -> List returns empty      |
 * | 3 | WITHOUT toolsets      | WITH fake scope              | Keycloak error (invalid_scope)               |
 * | 4 | WITHOUT toolsets      | WITHOUT extra scope          | Token lacks scope -> List returns empty      |
 *
 * Additional tests:
 * - Session auth returns toolset_types field
 * - OAuth tokens are blocked from config endpoints (session-only)
 *
 * Note: API token blocking tests (bodhiapp_* -> 401) moved to routes_app unit tests
 * (test_toolset_endpoints_reject_api_token in routes_toolsets/tests/toolsets_test.rs)
 */

const TOOLSET_TYPE = 'builtin-exa-search';

test.describe('Session Auth - Toolset Endpoints', () => {
  let authServerConfig;
  let testCredentials;
  let serverManager;
  let baseUrl;

  test.beforeAll(async ({ browser }) => {
    authServerConfig = getAuthServerConfig();
    testCredentials = getTestCredentials();
    const resourceClient = getPreConfiguredResourceClient();
    const port = 51135;

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

    // Configure Exa toolset via session auth
    const context = await browser.newContext();
    const page = await context.newPage();
    const loginPage = new LoginPage(page, baseUrl, authServerConfig, testCredentials);
    await loginPage.performOAuthLogin();

    const toolsetsPage = new ToolsetsPage(page, baseUrl);
    const exaApiKey = process.env.INTEG_TEST_EXA_API_KEY;
    expect(exaApiKey, 'INTEG_TEST_EXA_API_KEY not found in env').toBeDefined();
    expect(exaApiKey, 'INTEG_TEST_EXA_API_KEY not found in env').not.toBeNull();
    await toolsetsPage.configureToolsetWithApiKey(TOOLSET_TYPE, exaApiKey);

    await context.close();
  });

  test.afterAll(async () => {
    if (serverManager) {
      await serverManager.stopServer();
    }
  });

  test('GET /toolsets with session auth returns toolset_types field', async ({ browser }) => {
    const sessionContext = await browser.newContext();
    const sessionPage = await sessionContext.newPage();
    const loginPage = new LoginPage(sessionPage, baseUrl, authServerConfig, testCredentials);
    await loginPage.performOAuthLogin();

    await sessionPage.goto(baseUrl);

    const data = await sessionPage.evaluate(async (baseUrl) => {
      const response = await fetch(`${baseUrl}/bodhi/v1/toolsets`, {
        headers: {
          'Content-Type': 'application/json',
        },
      });
      if (!response.ok) {
        throw new Error(`HTTP ${response.status}`);
      }
      return await response.json();
    }, baseUrl);

    expect(data.toolset_types).toBeDefined();
    expect(Array.isArray(data.toolset_types)).toBe(true);
    const exaType = data.toolset_types.find((t) => t.toolset_type === TOOLSET_TYPE);
    expect(exaType).toBeTruthy();

    await sessionContext.close();
  });
});

test.describe('OAuth Token + Toolset Scope Combinations', () => {
  let authServerConfig;
  let testCredentials;
  let serverManager;
  let staticServer;
  let baseUrl;
  let testAppUrl;
  const port = 51135;
  const appPort = 55173;

  test.beforeAll(async () => {
    authServerConfig = getAuthServerConfig();
    testCredentials = getTestCredentials();
  });

  test.beforeEach(async () => {
    const resourceClient = getPreConfiguredResourceClient();

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
   * Access Request: WITH toolsets (approved)
   * OAuth Request: WITH access_request_scope
   * Expected: Token has scope -> List returns toolset + can execute tool
   *
   * This test performs end-to-end verification including live tool execution via OAuth exchanged token.
   * It requires INTEG_TEST_EXA_API_KEY environment variable to be set.
   */
  test('App WITH toolset scope + OAuth WITH toolset scope returns toolset in list and can execute', async ({
    page,
  }) => {
    // Check API key environment variable - fail if not present
    const exaApiKey = process.env.INTEG_TEST_EXA_API_KEY;
    expect(exaApiKey, 'INTEG_TEST_EXA_API_KEY environment variable is required').toBeTruthy();

    // Phase 1: Session login to configure toolset with API key
    const loginPage = new LoginPage(page, baseUrl, authServerConfig, testCredentials);
    await loginPage.performOAuthLogin();

    // Configure Exa toolset with API key via session auth
    const toolsetsPage = new ToolsetsPage(page, baseUrl);
    await toolsetsPage.configureToolsetWithApiKey(TOOLSET_TYPE, exaApiKey);

    // Get the toolset UUID for approval
    await toolsetsPage.navigateToToolsetsList();
    const toolsetId = await toolsetsPage.getToolsetUuidByScope(TOOLSET_TYPE);
    expect(toolsetId).toBeTruthy();

    // Phase 2: Use pre-configured app client for OAuth flow
    const appClient = getPreConfiguredAppClient();
    const redirectUri = `${testAppUrl}/oauth-test-app.html`;

    // Phase 3: Two-step OAuth flow via test app HTML + review page UI
    const oauth2TestAppPage = new OAuth2TestAppPage(page, testAppUrl);
    await oauth2TestAppPage.navigateToTestApp(redirectUri);

    await oauth2TestAppPage.configureOAuthForm(
      baseUrl,
      authServerConfig.authUrl,
      authServerConfig.authRealm,
      appClient.clientId,
      redirectUri,
      'openid profile email scope_user_user',
      JSON.stringify([{ toolset_type: TOOLSET_TYPE }])
    );

    // Submit access request -> draft -> redirects to review page
    await oauth2TestAppPage.submitAccessRequest();
    await oauth2TestAppPage.waitForAccessRequestRedirect(baseUrl);

    // Approve on the review page (browser has session from Phase 1 login)
    const reviewPage = new AccessRequestReviewPage(page, baseUrl);
    await reviewPage.approveWithToolsets([
      { toolsetType: TOOLSET_TYPE, instanceId: toolsetId },
    ]);

    // Wait for callback back to test app with ?id= param
    await oauth2TestAppPage.waitForAccessRequestCallback(testAppUrl);
    // Test app fetches status, populates scopes, shows Login button
    await oauth2TestAppPage.waitForLoginReady();
    await oauth2TestAppPage.clickLogin();
    await oauth2TestAppPage.waitForTokenExchange(testAppUrl);

    const accessToken = await oauth2TestAppPage.getAccessToken();
    expect(accessToken).toBeTruthy();

    // Phase 4: Verification
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
    const exaToolset = data.toolsets.find((t) => t.toolset_type === TOOLSET_TYPE);
    expect(exaToolset).toBeTruthy();

    // Verify toolset_types field exists and contains exa config
    expect(data.toolset_types).toBeDefined();
    expect(Array.isArray(data.toolset_types)).toBe(true);
    const exaType = data.toolset_types.find((t) => t.toolset_type === TOOLSET_TYPE);
    expect(exaType).toBeTruthy();

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
   * Access Request: WITH toolsets (approved)
   * OAuth Request: WITHOUT access_request_scope
   * Expected: Token lacks scope -> List returns empty
   */
  test('App WITH toolset scope + OAuth WITHOUT toolset scope returns empty list', async ({
    page,
  }) => {
    const exaApiKey = process.env.INTEG_TEST_EXA_API_KEY;
    expect(exaApiKey, 'INTEG_TEST_EXA_API_KEY environment variable is required').toBeTruthy();

    // Session login to configure toolset and approve access request
    const loginPage = new LoginPage(page, baseUrl, authServerConfig, testCredentials);
    await loginPage.performOAuthLogin();

    // Configure Exa toolset with API key
    const toolsetsPage = new ToolsetsPage(page, baseUrl);
    await toolsetsPage.configureToolsetWithApiKey(TOOLSET_TYPE, exaApiKey);

    // Get the toolset UUID for approval
    await toolsetsPage.navigateToToolsetsList();
    const toolsetId = await toolsetsPage.getToolsetUuidByScope(TOOLSET_TYPE);
    expect(toolsetId).toBeTruthy();

    // Use pre-configured app client for OAuth flow
    const appClient = getPreConfiguredAppClient();
    const redirectUri = `${testAppUrl}/oauth-test-app.html`;

    // Two-step OAuth flow via test app HTML + review page UI
    const oauth2TestAppPage = new OAuth2TestAppPage(page, testAppUrl);
    await oauth2TestAppPage.navigateToTestApp(redirectUri);

    await oauth2TestAppPage.configureOAuthForm(
      baseUrl,
      authServerConfig.authUrl,
      authServerConfig.authRealm,
      appClient.clientId,
      redirectUri,
      'openid profile email scope_user_user',
      JSON.stringify([{ toolset_type: TOOLSET_TYPE }])
    );

    // Submit access request -> draft -> redirects to review page
    await oauth2TestAppPage.submitAccessRequest();
    await oauth2TestAppPage.waitForAccessRequestRedirect(baseUrl);

    // Approve on the review page (browser has session from earlier login)
    const reviewPage = new AccessRequestReviewPage(page, baseUrl);
    await reviewPage.approveWithToolsets([
      { toolsetType: TOOLSET_TYPE, instanceId: toolsetId },
    ]);

    // Wait for callback back to test app with ?id= param
    await oauth2TestAppPage.waitForAccessRequestCallback(testAppUrl);
    // Test app fetches status, populates scopes, shows Login button
    await oauth2TestAppPage.waitForLoginReady();

    // Remove access_request_scope from the resolved scopes before login
    const arScope = await oauth2TestAppPage.getAccessRequestScope();
    const currentScope = await page.inputValue('#scope');
    const modifiedScope = currentScope.replace(arScope, '').replace(/\s+/g, ' ').trim();
    await oauth2TestAppPage.setScopes(modifiedScope);

    await oauth2TestAppPage.clickLogin();
    await oauth2TestAppPage.waitForTokenExchange(testAppUrl);

    const accessToken = await oauth2TestAppPage.getAccessToken();
    expect(accessToken).toBeTruthy();

    // Test: GET /toolsets with OAuth token (no access_request_scope)
    // The list endpoint returns all toolsets for the user (no scope filtering)
    // Scope enforcement happens at the execute endpoint via toolset_auth_middleware
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

    // User can see their toolsets in the list (list endpoint doesn't filter by scope)
    // The toolset was created via session auth and is owned by this user
    const exaToolset = data.toolsets.find((t) => t.toolset_type === TOOLSET_TYPE);
    expect(exaToolset).toBeTruthy();

    // But executing the toolset should fail without access_request_scope
    const executeResponse = await fetch(
      `${baseUrl}/bodhi/v1/toolsets/${exaToolset.id}/execute/search`,
      {
        method: 'POST',
        headers: {
          Authorization: `Bearer ${accessToken}`,
          'Content-Type': 'application/json',
        },
        body: JSON.stringify({
          params: {
            query: 'test query',
            num_results: 1,
          },
        }),
      }
    );

    // Without access_request_scope, execute should be denied
    expect(executeResponse.status).not.toBe(200);
  });

  /**
   * Test Matrix Case 3:
   * Access Request: WITHOUT toolsets (auto-approved)
   * OAuth Request: WITH non-existent scope
   * Expected: Keycloak error (invalid_scope)
   */
  test('App WITHOUT toolset scope + OAuth WITH toolset scope returns invalid_scope error', async ({
    page,
  }) => {
    // Session login for KC scope wiring
    const loginPage = new LoginPage(page, baseUrl, authServerConfig, testCredentials);
    await loginPage.performOAuthLogin();

    // Use pre-configured app client
    const appClient = getPreConfiguredAppClient();
    const redirectUri = `${testAppUrl}/oauth-test-app.html`;

    // Navigate to test app - test app handles access request (auto-approve)
    const oauth2TestAppPage = new OAuth2TestAppPage(page, testAppUrl);
    await oauth2TestAppPage.navigateToTestApp(redirectUri);

    await oauth2TestAppPage.configureOAuthForm(
      baseUrl,
      authServerConfig.authUrl,
      authServerConfig.authRealm,
      appClient.clientId,
      redirectUri,
      'openid profile email scope_user_user',
      null
    );

    // Two-step flow: submit access request (auto-approve), wait for scopes
    await oauth2TestAppPage.submitAccessRequest();
    await oauth2TestAppPage.waitForLoginReady();

    // Inject a non-existent scope into the resolved scopes
    const currentScope = await page.inputValue('#scope');
    await oauth2TestAppPage.setScopes(currentScope + ' scope_ar_nonexistent');

    await oauth2TestAppPage.clickLogin();

    // Keycloak should reject this with invalid_scope error
    const errorResult = await oauth2TestAppPage.expectOAuthError('invalid_scope');
    expect(errorResult.error).toBe('invalid_scope');
  });

  /**
   * Test Matrix Case 4:
   * Access Request: WITHOUT toolsets (auto-approved)
   * OAuth Request: WITHOUT extra scope
   * Expected: Token lacks scope -> List returns empty
   */
  test('App WITHOUT toolset scope + OAuth WITHOUT toolset scope returns empty list', async ({
    page,
  }) => {
    // Session login for KC scope wiring
    const loginPage = new LoginPage(page, baseUrl, authServerConfig, testCredentials);
    await loginPage.performOAuthLogin();

    // Use pre-configured app client
    const appClient = getPreConfiguredAppClient();
    const redirectUri = `${testAppUrl}/oauth-test-app.html`;

    // OAuth flow WITHOUT toolset scope - test app handles access request
    const oauth2TestAppPage = new OAuth2TestAppPage(page, testAppUrl);
    await oauth2TestAppPage.navigateToTestApp(redirectUri);

    // Basic scopes only - test app will add resourceScope from request-access response
    const fullScopes = `openid profile email scope_user_user`;
    await oauth2TestAppPage.configureOAuthForm(
      baseUrl,
      authServerConfig.authUrl,
      authServerConfig.authRealm,
      appClient.clientId,
      redirectUri,
      fullScopes,
      null
    );

    // Two-step flow: submit access request (auto-approve), wait for scopes, then login
    await oauth2TestAppPage.submitAccessRequest();
    await oauth2TestAppPage.waitForLoginReady();
    await oauth2TestAppPage.clickLogin();
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

    // Without toolset scope in token, should return empty toolsets list
    expect(data.toolsets.length).toBe(0);

    // toolset_types returns app-level enabled types (not filtered by OAuth scope)
    expect(data.toolset_types).toBeDefined();
    expect(Array.isArray(data.toolset_types)).toBe(true);
  });
});

test.describe('OAuth Token - Toolset CRUD Endpoints (Session-Only)', () => {
  let authServerConfig;
  let testCredentials;
  let serverManager;
  let staticServer;
  let baseUrl;
  let testAppUrl;
  let toolsetUuid;
  const port = 51135;
  const appPort = 55173;

  test.beforeAll(async () => {
    authServerConfig = getAuthServerConfig();
    testCredentials = getTestCredentials();
  });

  test.beforeEach(async ({ browser }) => {
    const resourceClient = getPreConfiguredResourceClient();

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
    await toolsetsPage.configureToolsetWithApiKey(TOOLSET_TYPE, exaApiKey);

    // Get the UUID from the data-test-uuid attribute
    await toolsetsPage.navigateToToolsetsList();
    toolsetUuid = await toolsetsPage.getToolsetUuidByScope(TOOLSET_TYPE);

    await sessionContext.close();

    // Setup static server for OAuth test app
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

  test('GET and PUT /toolsets/{id} with OAuth token returns 401 (session-only)', async ({ page }) => {
    // Session login for KC scope wiring and OAuth
    const loginPage = new LoginPage(page, baseUrl, authServerConfig, testCredentials);
    await loginPage.performOAuthLogin();

    // Use pre-configured app client
    const appClient = getPreConfiguredAppClient();
    const redirectUri = `${testAppUrl}/oauth-test-app.html`;

    // Complete OAuth flow via two-step test app flow (auto-approve)
    const oauth2TestAppPage = new OAuth2TestAppPage(page, testAppUrl);
    await oauth2TestAppPage.navigateToTestApp(redirectUri);
    const fullScopes = `openid profile email scope_user_user`;
    await oauth2TestAppPage.configureOAuthForm(
      baseUrl,
      authServerConfig.authUrl,
      authServerConfig.authRealm,
      appClient.clientId,
      redirectUri,
      fullScopes,
      null
    );
    await oauth2TestAppPage.submitAccessRequest();
    await oauth2TestAppPage.waitForLoginReady();
    await oauth2TestAppPage.clickLogin();
    await oauth2TestAppPage.waitForTokenExchange(testAppUrl);
    const accessToken = await oauth2TestAppPage.getAccessToken();

    // Test: OAuth tokens are blocked for GET /toolsets/{id} endpoint (session-only)
    const getResponse = await fetch(`${baseUrl}/bodhi/v1/toolsets/${toolsetUuid}`, {
      headers: {
        Authorization: `Bearer ${accessToken}`,
        'Content-Type': 'application/json',
      },
    });
    expect(getResponse.status).toBe(401);

    // Test: OAuth tokens are blocked for PUT /toolsets/{id} endpoint (session-only)
    const putResponse = await fetch(`${baseUrl}/bodhi/v1/toolsets/${toolsetUuid}`, {
      method: 'PUT',
      headers: {
        Authorization: `Bearer ${accessToken}`,
        'Content-Type': 'application/json',
      },
      body: JSON.stringify({
        slug: 'Updated-OAuth',
        description: 'Updated from OAuth test',
        enabled: false,
        api_key: { action: 'Keep' },
      }),
    });
    expect(putResponse.status).toBe(401);
  });
});
