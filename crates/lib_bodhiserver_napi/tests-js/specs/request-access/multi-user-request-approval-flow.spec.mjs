import { AllAccessRequestsPage } from '@/pages/AllAccessRequestsPage.mjs';
import { LoginPage } from '@/pages/LoginPage.mjs';
import { RequestAccessPage } from '@/pages/RequestAccessPage.mjs';
import { UsersManagementPage } from '@/pages/UsersManagementPage.mjs';
import { randomPort } from '@/test-helpers.mjs';
import { createAuthServerTestClient, getAuthServerConfig } from '@/utils/auth-server-client.mjs';
import { createServerManager } from '@/utils/bodhi-app-server.mjs';
import { expect, test } from '@playwright/test';

test.describe('Multi-User Request and Approval Flow', () => {
  let serverManager;
  let baseUrl;
  let authServerConfig;
  let authClient;
  let resourceClient;
  let port;
  let serverUrl;

  // Test user credentials from environment
  const managerCredentials = {
    username: process.env.INTEG_TEST_USER_MANAGER,
    password: process.env.INTEG_TEST_PASSWORD,
  };

  const adminCredentials = {
    username: process.env.INTEG_TEST_USER_ADMIN,
    password: process.env.INTEG_TEST_PASSWORD,
    userId: process.env.INTEG_TEST_USER_ADMIN_ID,
  };

  const powerUserCredentials = {
    username: process.env.INTEG_TEST_USER_POWER_USER,
    password: process.env.INTEG_TEST_PASSWORD,
  };

  const userCredentials = {
    username: 'user@email.com', // Fixed user for rejection testing
    password: process.env.INTEG_TEST_PASSWORD,
  };

  test.beforeAll(async () => {
    // Setup auth server and credentials
    authServerConfig = getAuthServerConfig();
    port = randomPort();
    serverUrl = `http://localhost:${port}`;

    // Create auth server client and resource client
    authClient = createAuthServerTestClient(authServerConfig);
    resourceClient = await authClient.createResourceClient(serverUrl);

    // Make admin user a resource admin using user ID
    await authClient.makeResourceAdmin(
      resourceClient.clientId,
      resourceClient.clientSecret,
      adminCredentials.userId
    );

    // Create and start server
    serverManager = createServerManager({
      appStatus: 'ready',
      authUrl: authServerConfig.authUrl,
      authRealm: authServerConfig.authRealm,
      clientId: resourceClient.clientId,
      clientSecret: resourceClient.clientSecret,
      port,
      host: 'localhost',
      envVars: {},
    });
    baseUrl = await serverManager.startServer();
  });

  test.afterAll(async () => {
    if (serverManager) {
      await serverManager.stopServer();
    }
  });

  test('Multi-User Access Request Flow', async ({ browser }) => {
    let adminContext;
    let managerContext;
    let powerUserContext;
    let userContext;

    try {
      // ========== Phase 1: Multiple Users Request Access ==========
      console.log('=== Phase 1: Multiple Users Request Access ===');

      // 1.1 Manager requests access
      console.log('1.1 Manager requests access');
      managerContext = await browser.newContext();
      const managerPage = await managerContext.newPage();

      const managerLogin = new LoginPage(
        managerPage,
        baseUrl,
        authServerConfig,
        managerCredentials
      );
      const managerRequestPage = new RequestAccessPage(managerPage, baseUrl);

      await managerLogin.performOAuthLogin('/ui/request-access/');
      await managerRequestPage.expectRequestAccessPage();
      await managerRequestPage.expectRequestButtonVisible(true);

      await managerRequestPage.clickRequestAccess();
      await managerRequestPage.expectPendingState();

      const managerSubmittedDate = await managerRequestPage.getSubmittedDate();
      await managerRequestPage.expectSubmittedDateFormat(managerSubmittedDate);

      // Test protected pages redirect
      await managerRequestPage.testProtectedPageRedirect('/ui/chat');
      await managerRequestPage.expectPendingState();
      await managerRequestPage.testProtectedPageRedirect('/ui/models');
      await managerRequestPage.expectPendingState();
      await managerRequestPage.testProtectedPageRedirect('/ui/settings');
      await managerRequestPage.expectPendingState();

      // Test persistence across reload
      await managerPage.reload();
      await managerRequestPage.waitForSPAReady();
      await managerRequestPage.expectPendingState();

      const currentDate = await managerRequestPage.getSubmittedDate();
      expect(currentDate).toBe(managerSubmittedDate);

      console.log('Manager request completed - keeping context open for session invalidation test');

      // 1.2 PowerUser requests access
      console.log('1.2 PowerUser requests access');
      powerUserContext = await browser.newContext();
      const powerUserPage = await powerUserContext.newPage();

      const powerUserLogin = new LoginPage(
        powerUserPage,
        baseUrl,
        authServerConfig,
        powerUserCredentials
      );
      const powerUserRequestPage = new RequestAccessPage(powerUserPage, baseUrl);

      await powerUserLogin.performOAuthLogin('/ui/request-access/');
      await powerUserRequestPage.expectRequestAccessPage();
      await powerUserRequestPage.expectRequestButtonVisible(true);

      await powerUserRequestPage.clickRequestAccess();
      await powerUserRequestPage.expectPendingState();

      console.log('PowerUser request completed - keeping context open for later validation');

      // 1.3 Regular User requests access
      console.log('1.3 Regular User requests access');
      userContext = await browser.newContext();
      const userPage = await userContext.newPage();

      const userLogin = new LoginPage(userPage, baseUrl, authServerConfig, userCredentials);
      const userRequestPage = new RequestAccessPage(userPage, baseUrl);

      await userLogin.performOAuthLogin('/ui/request-access/');
      await userRequestPage.expectRequestAccessPage();
      await userRequestPage.expectRequestButtonVisible(true);

      await userRequestPage.clickRequestAccess();
      await userRequestPage.expectPendingState();

      console.log('User request completed - keeping context open for rejection flow test');

      // ========== Phase 2: Admin Reviews and Approves Manager Only ==========
      console.log('=== Phase 2: Admin Reviews and Approves Manager Only ===');

      // 2.1 Admin logs in and sees all 3 requests
      console.log('2.1 Admin sees 3 pending requests');
      adminContext = await browser.newContext();
      const adminPage = await adminContext.newPage();

      const adminLogin = new LoginPage(adminPage, baseUrl, authServerConfig, adminCredentials);
      const adminUsersPage = new UsersManagementPage(adminPage, baseUrl);

      await adminLogin.performOAuthLogin();
      await adminUsersPage.navigateToPendingRequests();

      // Verify all 3 requests are visible
      await adminUsersPage.expectRequestExists(managerCredentials.username);
      await adminUsersPage.expectRequestExists(powerUserCredentials.username);
      await adminUsersPage.expectRequestExists(userCredentials.username);

      // 2.2 Admin approves only manager with "Manager" role
      console.log('2.2 Admin approves manager with Manager role');
      await adminUsersPage.approveRequest(managerCredentials.username, 'Manager');

      // Verify manager request is gone but others remain
      await adminUsersPage.expectRequestNotInList(managerCredentials.username);
      await adminUsersPage.expectRequestExists(powerUserCredentials.username);
      await adminUsersPage.expectRequestExists(userCredentials.username);

      // ========== Phase 3: Manager Session Invalidation & Admin Access ==========
      console.log('=== Phase 3: Manager Session Invalidation & Admin Access ===');

      // 3.1 Test manager session invalidation after role approval
      console.log('3.1 Testing manager session invalidation');

      // Try to navigate to protected page - should redirect to login due to session invalidation
      await managerPage.goto(`${baseUrl}/ui/chat`);
      await managerPage.waitForLoadState('networkidle');
      await managerPage.waitForURL((url) => url.pathname === '/ui/login/');
      console.log('Confirmed: Manager session invalidated, redirected to login');

      // Manager re-authenticates using Keycloak session (no username/password needed)
      await managerLogin.performOAuthLoginFromSession();

      // Should now successfully reach chat page with new role
      await managerPage.waitForURL((url) => url.pathname === '/ui/chat/');
      console.log('Manager successfully re-authenticated and reached chat page');

      // 3.2 Manager accesses admin pages with new role
      console.log('3.2 Manager tests admin access with new role');
      const managerUsersPage = new UsersManagementPage(managerPage, baseUrl);

      // Navigate to admin pages
      await managerUsersPage.navigateToPendingRequests();

      // Should see 2 remaining requests
      await managerUsersPage.expectRequestExists(powerUserCredentials.username);
      await managerUsersPage.expectRequestExists(userCredentials.username);

      // 3.3 Manager role hierarchy validation - verify Admin role not available
      console.log('3.3 Manager role hierarchy validation - testing Admin role restriction');

      // Verify that Admin role is not available in dropdown for manager
      console.log('3.3a Manager verifies Admin role not available in dropdown');
      await managerUsersPage.expectRoleNotAvailable(powerUserCredentials.username, 'Admin');

      // 3.3b Now assign the correct PowerUser role (should succeed)
      console.log('3.3b Manager assigns correct Power User role to PowerUser');
      await managerUsersPage.approveRequest(powerUserCredentials.username, 'Power User');

      // Verify poweruser request is gone after successful assignment
      await managerUsersPage.expectRequestNotInList(powerUserCredentials.username);
      await managerUsersPage.expectRequestExists(userCredentials.username);

      // 3.3c Manager verifies all requests page shows correct data after PowerUser approval
      console.log('3.3c Manager verifies all requests page with approved/pending requests');
      const allRequestsPage = new AllAccessRequestsPage(managerPage, baseUrl);
      await allRequestsPage.navigateToAllRequests();
      await allRequestsPage.expectAllRequestsPage();

      // Verify total count
      await allRequestsPage.verifyRequestCount(3);

      // Verify each request with detailed assertions
      await allRequestsPage.verifyAllRequests([
        {
          username: managerCredentials.username,
          status: 'approved',
          reviewer: adminCredentials.username,
        },
        {
          username: powerUserCredentials.username,
          status: 'approved',
          reviewer: managerCredentials.username,
        },
        {
          username: userCredentials.username,
          status: 'pending',
          reviewer: null,
        },
      ]);

      // Verify date columns (approved show updated_at, pending shows created_at)
      await allRequestsPage.verifyDateDisplay(managerCredentials.username, false);
      await allRequestsPage.verifyDateDisplay(powerUserCredentials.username, false);
      await allRequestsPage.verifyDateDisplay(userCredentials.username, true);

      console.log(
        '✓ All requests page correctly displays 2 approved, 1 pending with proper metadata'
      );

      // 3.4 Manager navigates between admin pages
      console.log('3.4 Manager tests admin page navigation');
      await managerUsersPage.navigateToUsers();
      // Should access users page

      // 3.5 Manager rejects last request
      console.log('3.5 Manager rejects user request');
      await managerUsersPage.navigateToPendingRequests();
      await managerUsersPage.rejectRequest(userCredentials.username);
      await managerUsersPage.expectRequestNotInList(userCredentials.username);

      // 3.6 Manager verifies final state with all requests processed
      console.log('3.6 Manager verifies all requests processed');
      await allRequestsPage.navigateToAllRequests();

      // Verify all 3 requests are processed
      await allRequestsPage.verifyRequestCount(3);

      await allRequestsPage.verifyAllRequests([
        {
          username: managerCredentials.username,
          status: 'approved',
          reviewer: adminCredentials.username,
        },
        {
          username: powerUserCredentials.username,
          status: 'approved',
          reviewer: managerCredentials.username,
        },
        {
          username: userCredentials.username,
          status: 'rejected',
          reviewer: managerCredentials.username,
        },
      ]);

      // Verify no pending requests remain
      await managerUsersPage.navigateToPendingRequests();
      await managerUsersPage.expectNoRequests();

      console.log('✓ All requests processed: 2 approved, 1 rejected, 0 pending');

      console.log('All phases completed successfully');
    } finally {
      if (userContext) {
        await userContext.close();
      }
      if (powerUserContext) {
        await powerUserContext.close();
      }
      if (managerContext) {
        await managerContext.close();
      }
      if (adminContext) {
        await adminContext.close();
      }
    }
  });
});
