import { expect, test } from '@playwright/test';
import { randomPort } from '../../test-helpers.mjs';
import {
  createAuthServerTestClient,
  getAuthServerConfig,
} from '../../playwright/auth-server-client.mjs';
import { createServerManager } from '../../playwright/bodhi-app-server.mjs';
import { LoginPage } from '../../pages/LoginPage.mjs';
import { AllUsersPage } from '../../pages/AllUsersPage.mjs';

test.describe('All Users Page Management', () => {
  let serverManager;
  let baseUrl;
  let authServerConfig;
  let authClient;
  let resourceClient;
  let port;
  let serverUrl;

  // Test user credentials from environment
  const testUsers = {
    admin: {
      username: process.env.INTEG_TEST_USER_ADMIN,
      userId: process.env.INTEG_TEST_USER_ADMIN_ID,
      password: process.env.INTEG_TEST_PASSWORD,
      role: 'resource_admin',
    },
    manager: {
      username: process.env.INTEG_TEST_USER_MANAGER,
      userId: process.env.INTEG_TEST_USER_MANAGER_ID,
      password: process.env.INTEG_TEST_PASSWORD,
      role: 'resource_manager',
    },
    powerUser: {
      username: process.env.INTEG_TEST_USER_POWER_USER,
      userId: process.env.INTEG_TEST_USER_POWER_USER_ID,
      password: process.env.INTEG_TEST_PASSWORD,
      role: 'resource_power_user',
    },
    user: {
      username: process.env.INTEG_TEST_USERNAME,
      userId: process.env.INTEG_TEST_USERNAME_ID,
      password: process.env.INTEG_TEST_PASSWORD,
      role: 'resource_user',
    },
  };

  /**
   * Setup test users directly via auth server assign-role endpoint
   */
  async function setupTestUsersDirectly() {
    console.log('Setting up test users directly via assign-role endpoint');

    // Create resource client with liveTest=true for Direct Access Grants
    resourceClient = await authClient.createResourceClient(
      serverUrl,
      'Test Resource Client',
      'Direct test setup',
      true
    );

    // Make admin user the first resource admin
    await authClient.makeResourceAdmin(
      resourceClient.clientId,
      resourceClient.clientSecret,
      testUsers.admin.userId
    );

    // Get admin token via Direct Access Grant
    const adminToken = await authClient.getResourceUserToken(
      resourceClient.clientId,
      resourceClient.clientSecret,
      testUsers.admin.username,
      testUsers.admin.password
    );

    // Assign roles to other users directly
    await authClient.assignUserRole(adminToken, testUsers.manager.userId, testUsers.manager.role);
    await authClient.assignUserRole(
      adminToken,
      testUsers.powerUser.userId,
      testUsers.powerUser.role
    );
    await authClient.assignUserRole(adminToken, testUsers.user.userId, testUsers.user.role);

    console.log('Test users setup completed');
  }

  test.beforeAll(async () => {
    // Setup auth server and credentials
    authServerConfig = getAuthServerConfig();
    port = randomPort();
    serverUrl = `http://localhost:${port}`;

    // Create auth server client
    authClient = createAuthServerTestClient(authServerConfig);

    // Setup test users directly via assign-role endpoint
    await setupTestUsersDirectly();

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

  test('All Users Page Shows Direct Created Users', async ({ browser }) => {
    let adminContext;

    try {
      console.log('=== Testing All Users Page with Direct Created Users ===');

      // Admin logs in to access All Users page
      adminContext = await browser.newContext();
      const adminPage = await adminContext.newPage();
      const adminLogin = new LoginPage(adminPage, baseUrl, authServerConfig, testUsers.admin);
      await adminLogin.performOAuthLogin();

      // Navigate to All Users page
      const adminAllUsersPage = new AllUsersPage(adminPage, baseUrl);
      await adminAllUsersPage.navigateToUsers();
      await adminAllUsersPage.expectUsersPage();

      // Verify all test users are visible with correct roles
      console.log('Verifying admin user');
      await adminAllUsersPage.expectUserExists(testUsers.admin.username);
      await adminAllUsersPage.expectUserRole(testUsers.admin.username, 'Admin');
      await adminAllUsersPage.expectUserStatus(testUsers.admin.username, 'Active');

      console.log('Verifying manager user');
      await adminAllUsersPage.expectUserExists(testUsers.manager.username);
      await adminAllUsersPage.expectUserRole(testUsers.manager.username, 'Manager');
      await adminAllUsersPage.expectUserStatus(testUsers.manager.username, 'Active');

      console.log('Verifying power user');
      await adminAllUsersPage.expectUserExists(testUsers.powerUser.username);
      await adminAllUsersPage.expectUserRole(testUsers.powerUser.username, 'Power User');
      await adminAllUsersPage.expectUserStatus(testUsers.powerUser.username, 'Active');

      console.log('Verifying regular user');
      await adminAllUsersPage.expectUserExists(testUsers.user.username);
      await adminAllUsersPage.expectUserRole(testUsers.user.username, 'User');
      await adminAllUsersPage.expectUserStatus(testUsers.user.username, 'Active');

      // Verify total user count
      const userCount = await adminAllUsersPage.getUserCount();
      expect(userCount).toBe(4);

      console.log(
        '✓ All Users page correctly displays all 4 directly created users with proper roles'
      );
    } finally {
      if (adminContext) {
        await adminContext.close();
      }
    }
  });
});
