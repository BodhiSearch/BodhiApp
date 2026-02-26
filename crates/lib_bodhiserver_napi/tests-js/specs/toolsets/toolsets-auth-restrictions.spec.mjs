import { AccessRequestReviewPage } from '@/pages/AccessRequestReviewPage.mjs';
import { LoginPage } from '@/pages/LoginPage.mjs';
import { ToolsetsPage } from '@/pages/ToolsetsPage.mjs';
import { OAuthTestApp } from '@/pages/OAuthTestApp.mjs';
import {
  getAuthServerConfig,
  getPreConfiguredAppClient,
  getTestCredentials,
} from '@/utils/auth-server-client.mjs';
import { expect, test } from '@/fixtures.mjs';
import { SHARED_SERVER_URL, SHARED_STATIC_SERVER_URL } from '@/test-helpers.mjs';

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

test.describe('Session Auth - Toolset Endpoints', { tag: ['@oauth', '@toolsets'] }, () => {
  let authServerConfig;
  let testCredentials;

  test.beforeAll(async ({ browser }) => {
    authServerConfig = getAuthServerConfig();
    testCredentials = getTestCredentials();

    // Use shared server started by Playwright webServer

    // Configure Exa toolset via session auth
    const context = await browser.newContext();
    const page = await context.newPage();
    const loginPage = new LoginPage(page, SHARED_SERVER_URL, authServerConfig, testCredentials);
    await loginPage.performOAuthLogin();

    const toolsetsPage = new ToolsetsPage(page, SHARED_SERVER_URL);
    const exaApiKey = process.env.INTEG_TEST_EXA_API_KEY;
    expect(exaApiKey, 'INTEG_TEST_EXA_API_KEY not found in env').toBeDefined();
    expect(exaApiKey, 'INTEG_TEST_EXA_API_KEY not found in env').not.toBeNull();
    await toolsetsPage.configureToolsetWithApiKey(TOOLSET_TYPE, exaApiKey);

    await context.close();
  });

  test('GET /toolsets with session auth returns toolset_types field', async ({ browser }) => {
    const sessionContext = await browser.newContext();
    const sessionPage = await sessionContext.newPage();
    const loginPage = new LoginPage(
      sessionPage,
      SHARED_SERVER_URL,
      authServerConfig,
      testCredentials
    );
    await loginPage.performOAuthLogin();

    await sessionPage.goto(SHARED_SERVER_URL);

    const data = await sessionPage.evaluate(async (SHARED_SERVER_URL) => {
      const response = await fetch(`${SHARED_SERVER_URL}/bodhi/v1/toolsets`, {
        headers: {
          'Content-Type': 'application/json',
        },
      });
      if (!response.ok) {
        throw new Error(`HTTP ${response.status}`);
      }
      return await response.json();
    }, SHARED_SERVER_URL);

    expect(data.toolset_types).toBeDefined();
    expect(Array.isArray(data.toolset_types)).toBe(true);
    const exaType = data.toolset_types.find((t) => t.toolset_type === TOOLSET_TYPE);
    expect(exaType).toBeTruthy();

    await sessionContext.close();
  });
});

test.describe('OAuth Token + Toolset Scope Combinations', { tag: ['@oauth', '@toolsets'] }, () => {
  let authServerConfig;
  let testCredentials;

  test.beforeAll(async () => {
    authServerConfig = getAuthServerConfig();
    testCredentials = getTestCredentials();
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

    let toolsetId;

    await test.step('Phase 1: Session login and configure toolset', async () => {
      const loginPage = new LoginPage(page, SHARED_SERVER_URL, authServerConfig, testCredentials);
      await loginPage.performOAuthLogin();

      // Configure Exa toolset with API key via session auth
      const toolsetsPage = new ToolsetsPage(page, SHARED_SERVER_URL);
      await toolsetsPage.configureToolsetWithApiKey(TOOLSET_TYPE, exaApiKey);

      // Get the toolset UUID for approval
      await toolsetsPage.navigateToToolsetsList();
      toolsetId = await toolsetsPage.getToolsetUuidByScope(TOOLSET_TYPE);
      expect(toolsetId).toBeTruthy();
    });

    const appClient = getPreConfiguredAppClient();
    const redirectUri = `${SHARED_STATIC_SERVER_URL}/callback`;

    const app = new OAuthTestApp(page, SHARED_STATIC_SERVER_URL);

    await test.step('Phase 2: Navigate and configure OAuth form', async () => {
      await app.navigate();

      await app.config.configureOAuthForm({
        bodhiServerUrl: SHARED_SERVER_URL,
        authServerUrl: authServerConfig.authUrl,
        realm: authServerConfig.authRealm,
        clientId: appClient.clientId,
        redirectUri,
        scope: 'openid profile email',
        requested: JSON.stringify({ toolset_types: [{ toolset_type: TOOLSET_TYPE }] }),
      });
    });

    await test.step('Phase 3: Submit access request and approve with toolsets', async () => {
      // Submit access request -> draft -> redirects to review page
      await app.config.submitAccessRequest();
      await app.oauth.waitForAccessRequestRedirect(SHARED_SERVER_URL);

      // Approve on the review page (browser has session from Phase 1 login)
      const reviewPage = new AccessRequestReviewPage(page, SHARED_SERVER_URL);
      await reviewPage.approveWithToolsets([{ toolsetType: TOOLSET_TYPE, instanceId: toolsetId }]);

      // Wait for callback back to test app with ?id= param
      await app.oauth.waitForAccessRequestCallback(SHARED_STATIC_SERVER_URL);
      // Access callback page: wait for terminal state, then click Login
      await app.accessCallback.waitForLoaded();
      await app.accessCallback.clickLogin();
      await app.oauth.waitForTokenExchange(SHARED_STATIC_SERVER_URL);
    });

    await test.step('Phase 4: Verify toolset access via API', async () => {
      await app.rest.navigateTo();

      // Test: GET /toolsets with OAuth token returns filtered list containing the toolset
      await app.rest.sendRequest({
        method: 'GET',
        url: '/bodhi/v1/toolsets',
      });

      expect(await app.rest.getResponseStatus()).toBe(200);
      const data = await app.rest.getResponse();
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
      await app.rest.sendRequest({
        method: 'POST',
        url: `/bodhi/v1/toolsets/${exaToolset.id}/tools/search/execute`,

        body: JSON.stringify({
          params: {
            query: 'latest news about AI from San Francisco',
            num_results: 3,
          },
        }),
      });

      expect(await app.rest.getResponseStatus()).toBe(200);
      const executeData = await app.rest.getResponse();

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
  });

  /**
   * Test Matrix Case 2:
   * Access Request: WITH toolsets (approved)
   * OAuth Request: WITHOUT access_request_scope
   * Expected: Token lacks access_request_scope -> ExternalApp auth fails with 401
   * (Without access_request_scope, the server cannot link the token to an approved access request
   * and cannot determine the ExternalApp role, resulting in authentication failure)
   */
  test('App WITH toolset scope + OAuth WITHOUT toolset scope returns empty list', async ({
    page,
  }) => {
    const exaApiKey = process.env.INTEG_TEST_EXA_API_KEY;
    expect(exaApiKey, 'INTEG_TEST_EXA_API_KEY environment variable is required').toBeTruthy();

    let toolsetId;

    await test.step('Session login and configure toolset', async () => {
      const loginPage = new LoginPage(page, SHARED_SERVER_URL, authServerConfig, testCredentials);
      await loginPage.performOAuthLogin();

      // Configure Exa toolset with API key
      const toolsetsPage = new ToolsetsPage(page, SHARED_SERVER_URL);
      await toolsetsPage.configureToolsetWithApiKey(TOOLSET_TYPE, exaApiKey);

      // Get the toolset UUID for approval
      await toolsetsPage.navigateToToolsetsList();
      toolsetId = await toolsetsPage.getToolsetUuidByScope(TOOLSET_TYPE);
      expect(toolsetId).toBeTruthy();
    });

    const appClient = getPreConfiguredAppClient();
    const redirectUri = `${SHARED_STATIC_SERVER_URL}/callback`;

    const app = new OAuthTestApp(page, SHARED_STATIC_SERVER_URL);

    await test.step('Navigate and configure OAuth form with toolsets', async () => {
      await app.navigate();

      await app.config.configureOAuthForm({
        bodhiServerUrl: SHARED_SERVER_URL,
        authServerUrl: authServerConfig.authUrl,
        realm: authServerConfig.authRealm,
        clientId: appClient.clientId,
        redirectUri,
        scope: 'openid profile email',
        requested: JSON.stringify({ toolset_types: [{ toolset_type: TOOLSET_TYPE }] }),
      });
    });

    await test.step('Submit access request and approve with toolsets', async () => {
      await app.config.submitAccessRequest();
      await app.oauth.waitForAccessRequestRedirect(SHARED_SERVER_URL);

      // Approve on the review page (browser has session from earlier login)
      const reviewPage = new AccessRequestReviewPage(page, SHARED_SERVER_URL);
      await reviewPage.approveWithToolsets([{ toolsetType: TOOLSET_TYPE, instanceId: toolsetId }]);

      // Wait for callback back to test app with ?id= param
      await app.oauth.waitForAccessRequestCallback(SHARED_STATIC_SERVER_URL);
      // Access callback page: wait for terminal state
      await app.accessCallback.waitForLoaded();
    });

    await test.step('Remove access_request_scope and login', async () => {
      // Remove access_request_scope from the resolved scopes before login
      const arScope = await app.accessCallback.getAccessRequestScope();
      const currentScope = await app.accessCallback.getScopeValue();
      const modifiedScope = currentScope.replace(arScope, '').replace(/\s+/g, ' ').trim();
      await app.accessCallback.setScopes(modifiedScope);

      await app.accessCallback.clickLogin();
      await app.oauth.waitForTokenExchange(SHARED_STATIC_SERVER_URL);
    });

    await test.step('Verify toolset list is rejected without access_request_scope', async () => {
      await app.rest.navigateTo();

      // Test: GET /toolsets with OAuth token that lacks access_request_scope
      // Without access_request_scope, the server cannot link the token to an approved access request,
      // so ExternalApp authentication fails with 401.
      await app.rest.sendRequest({
        method: 'GET',
        url: '/bodhi/v1/toolsets',
      });

      expect(await app.rest.getResponseStatus()).toBe(401);
    });
  });

  /**
   * Test Matrix Case 3:
   * Access Request: WITHOUT toolsets (draft → review → approve)
   * OAuth Request: WITH non-existent scope injected
   * Expected: Keycloak error (invalid_scope)
   */
  test('App WITHOUT toolset scope + OAuth WITH toolset scope returns invalid_scope error', async ({
    page,
  }) => {
    await test.step('Session login for KC scope wiring', async () => {
      const loginPage = new LoginPage(page, SHARED_SERVER_URL, authServerConfig, testCredentials);
      await loginPage.performOAuthLogin();
    });

    const appClient = getPreConfiguredAppClient();
    const redirectUri = `${SHARED_STATIC_SERVER_URL}/callback`;

    const app = new OAuthTestApp(page, SHARED_STATIC_SERVER_URL);

    await test.step('Navigate and configure OAuth form without toolsets', async () => {
      await app.navigate();

      await app.config.configureOAuthForm({
        bodhiServerUrl: SHARED_SERVER_URL,
        authServerUrl: authServerConfig.authUrl,
        realm: authServerConfig.authRealm,
        clientId: appClient.clientId,
        redirectUri,
        scope: 'openid profile email',
        requested: null,
      });
    });

    await test.step('Submit access request and approve via review page', async () => {
      await app.config.submitAccessRequest();
      await app.oauth.waitForAccessRequestRedirect(SHARED_SERVER_URL);

      const reviewPage = new AccessRequestReviewPage(page, SHARED_SERVER_URL);
      await reviewPage.approve();

      await app.oauth.waitForAccessRequestCallback(SHARED_STATIC_SERVER_URL);
      await app.accessCallback.waitForLoaded();
    });

    await test.step('Inject invalid scope and verify Keycloak rejects it', async () => {
      const currentScope = await app.accessCallback.getScopeValue();
      await app.accessCallback.setScopes(currentScope + ' scope_ar_nonexistent');
      await app.accessCallback.clickLogin();

      const errorResult = await app.oauth.expectOAuthError('invalid_scope');
      expect(errorResult.error).toBe('invalid_scope');
    });
  });

  /**
   * Test Matrix Case 4:
   * Access Request: WITHOUT toolsets (draft → review → approve)
   * OAuth Request: WITHOUT extra scope
   * Expected: Token lacks scope -> List returns empty
   */
  test('App WITHOUT toolset scope + OAuth WITHOUT toolset scope returns empty list', async ({
    page,
  }) => {
    await test.step('Session login for KC scope wiring', async () => {
      const loginPage = new LoginPage(page, SHARED_SERVER_URL, authServerConfig, testCredentials);
      await loginPage.performOAuthLogin();
    });

    const appClient = getPreConfiguredAppClient();
    const redirectUri = `${SHARED_STATIC_SERVER_URL}/callback`;

    const app = new OAuthTestApp(page, SHARED_STATIC_SERVER_URL);

    await test.step('Navigate and configure OAuth form without toolsets', async () => {
      await app.navigate();

      // Basic scopes only - test app will add resourceScope from request-access response
      await app.config.configureOAuthForm({
        bodhiServerUrl: SHARED_SERVER_URL,
        authServerUrl: authServerConfig.authUrl,
        realm: authServerConfig.authRealm,
        clientId: appClient.clientId,
        redirectUri,
        scope: 'openid profile email',
        requested: null,
      });
    });

    await test.step('Submit access request, approve, and complete login', async () => {
      await app.config.submitAccessRequest();
      await app.oauth.waitForAccessRequestRedirect(SHARED_SERVER_URL);

      const reviewPage = new AccessRequestReviewPage(page, SHARED_SERVER_URL);
      await reviewPage.approve();

      await app.oauth.waitForAccessRequestCallback(SHARED_STATIC_SERVER_URL);
      await app.accessCallback.waitForLoaded();
      await app.accessCallback.clickLogin();
      // KC session already exists from performOAuthLogin, so Keycloak auto-redirects
      await app.oauth.waitForTokenExchange(SHARED_STATIC_SERVER_URL);
    });

    await test.step('Verify empty toolsets list', async () => {
      await app.rest.navigateTo();

      // Test: GET /toolsets with OAuth token (no toolset scope) returns empty list
      await app.rest.sendRequest({
        method: 'GET',
        url: '/bodhi/v1/toolsets',
      });

      expect(await app.rest.getResponseStatus()).toBe(200);
      const data = await app.rest.getResponse();
      expect(data.toolsets).toBeDefined();
      expect(Array.isArray(data.toolsets)).toBe(true);

      // Without toolset scope in token, should return empty toolsets list
      expect(data.toolsets.length).toBe(0);

      // toolset_types returns app-level enabled types (not filtered by OAuth scope)
      expect(data.toolset_types).toBeDefined();
      expect(Array.isArray(data.toolset_types)).toBe(true);
    });
  });
});

test.describe(
  'OAuth Token - Toolset CRUD Endpoints (Session-Only)',
  { tag: ['@oauth', '@toolsets'] },
  () => {
    let authServerConfig;
    let testCredentials;
    let toolsetUuid;

    test.beforeAll(async ({ browser }) => {
      authServerConfig = getAuthServerConfig();
      testCredentials = getTestCredentials();

      // Create a real toolset via session auth to get its UUID
      const sessionContext = await browser.newContext();
      const sessionPage = await sessionContext.newPage();
      const loginPage = new LoginPage(
        sessionPage,
        SHARED_SERVER_URL,
        authServerConfig,
        testCredentials
      );
      await loginPage.performOAuthLogin();

      // Configure Exa toolset to create an instance
      const toolsetsPage = new ToolsetsPage(sessionPage, SHARED_SERVER_URL);
      const exaApiKey = process.env.INTEG_TEST_EXA_API_KEY;
      expect(exaApiKey, 'INTEG_TEST_EXA_API_KEY not found in env').toBeDefined();
      expect(exaApiKey, 'INTEG_TEST_EXA_API_KEY not found in env').not.toBeNull();
      await toolsetsPage.configureToolsetWithApiKey(TOOLSET_TYPE, exaApiKey);

      // Get the UUID from the data-test-uuid attribute
      await toolsetsPage.navigateToToolsetsList();
      toolsetUuid = await toolsetsPage.getToolsetUuidByScope(TOOLSET_TYPE);

      await sessionContext.close();
    });

    test('GET and PUT /toolsets/{id} with OAuth token returns 401 (session-only)', async ({
      page,
    }) => {
      await test.step('Session login and complete OAuth flow', async () => {
        const loginPage = new LoginPage(page, SHARED_SERVER_URL, authServerConfig, testCredentials);
        await loginPage.performOAuthLogin();
      });

      const appClient = getPreConfiguredAppClient();
      const redirectUri = `${SHARED_STATIC_SERVER_URL}/callback`;

      const app = new OAuthTestApp(page, SHARED_STATIC_SERVER_URL);

      await test.step('Navigate and configure OAuth form', async () => {
        await app.navigate();

        await app.config.configureOAuthForm({
          bodhiServerUrl: SHARED_SERVER_URL,
          authServerUrl: authServerConfig.authUrl,
          realm: authServerConfig.authRealm,
          clientId: appClient.clientId,
          redirectUri,
          scope: 'openid profile email',
          requested: null,
        });
      });

      await test.step('Submit access request, approve, and complete login', async () => {
        await app.config.submitAccessRequest();
        await app.oauth.waitForAccessRequestRedirect(SHARED_SERVER_URL);

        const reviewPage = new AccessRequestReviewPage(page, SHARED_SERVER_URL);
        await reviewPage.approve();

        await app.oauth.waitForAccessRequestCallback(SHARED_STATIC_SERVER_URL);
        await app.accessCallback.waitForLoaded();
        await app.accessCallback.clickLogin();
        // KC session already exists from performOAuthLogin, so Keycloak auto-redirects
        await app.oauth.waitForTokenExchange(SHARED_STATIC_SERVER_URL);
      });

      await test.step('Verify OAuth token is blocked for GET /toolsets/{id}', async () => {
        await app.rest.navigateTo();

        await app.rest.sendRequest({
          method: 'GET',
          url: `/bodhi/v1/toolsets/${toolsetUuid}`,
        });
        expect(await app.rest.getResponseStatus()).toBe(401);
      });

      await test.step('Verify OAuth token is blocked for PUT /toolsets/{id}', async () => {
        await app.rest.sendRequest({
          method: 'PUT',
          url: `/bodhi/v1/toolsets/${toolsetUuid}`,

          body: JSON.stringify({
            slug: 'Updated-OAuth',
            description: 'Updated from OAuth test',
            enabled: false,
            api_key: { action: 'Keep' },
          }),
        });
        expect(await app.rest.getResponseStatus()).toBe(401);
      });
    });
  }
);
