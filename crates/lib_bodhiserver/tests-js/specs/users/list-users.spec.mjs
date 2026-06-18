import { AllUsersPage } from '@/pages/AllUsersPage.mjs';
import { LoginPage } from '@/pages/LoginPage.mjs';
import { randomPort } from '@/test-helpers.mjs';
import { createAuthServerTestClient, getAuthServerConfig } from '@/utils/auth-server-client.mjs';
import { createServerManager } from '@/utils/bodhi-app-server.mjs';
import { expect, test } from '@playwright/test';

test.describe('Enhanced Users Management Flow', () => {
  let serverManager;
  let baseUrl;
  let authServerConfig;
  let authClient;
  let resourceClient;
  let port;
  let serverUrl;

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

  async function setupTestUsersDirectly() {
    resourceClient = await authClient.createResourceClient(
      serverUrl,
      'Test Resource Client',
      'Direct test setup',
      true
    );

    await authClient.makeResourceAdmin(
      resourceClient.clientId,
      resourceClient.clientSecret,
      testUsers.admin.userId
    );

    const adminToken = await authClient.getResourceUserToken(
      resourceClient.clientId,
      resourceClient.clientSecret,
      testUsers.admin.username,
      testUsers.admin.password
    );

    await authClient.assignUserRole(adminToken, testUsers.manager.userId, testUsers.manager.role);
    await authClient.assignUserRole(
      adminToken,
      testUsers.powerUser.userId,
      testUsers.powerUser.role
    );
    await authClient.assignUserRole(adminToken, testUsers.user.userId, testUsers.user.role);
  }

  test.beforeAll(async () => {
    authServerConfig = getAuthServerConfig();
    port = randomPort();
    serverUrl = `http://localhost:${port}`;

    authClient = createAuthServerTestClient(authServerConfig);

    await setupTestUsersDirectly();

    serverManager = createServerManager({
      appStatus: 'ready',
      authUrl: authServerConfig.authUrl,
      authRealm: authServerConfig.authRealm,
      clientId: resourceClient.clientId,
      clientSecret: resourceClient.clientSecret,
      createdBy: testUsers.admin.userId,
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

  test('Enhanced Users Management Flow - All Phases Complete', async ({ browser }) => {
    let adminContext;
    let managerContext;
    let powerUserContext;

    adminContext = await browser.newContext();
    const adminPage = await adminContext.newPage();
    const adminLogin = new LoginPage(adminPage, baseUrl, authServerConfig, testUsers.admin);
    await adminLogin.performOAuthLogin();

    const adminUsersPage = new AllUsersPage(adminPage, baseUrl);
    await adminUsersPage.navigateToUsers();
    await adminUsersPage.expectUsersPage();

    const expectedOrder = [
      testUsers.admin.username,
      testUsers.manager.username,
      testUsers.powerUser.username,
      testUsers.user.username,
    ];
    await adminUsersPage.verifyUsersInHierarchicalOrder(expectedOrder);

    await adminUsersPage.expectUserExists(testUsers.admin.username);
    await adminUsersPage.expectUserRole(testUsers.admin.username, 'Admin');

    await adminUsersPage.expectUserExists(testUsers.manager.username);
    await adminUsersPage.expectUserRole(testUsers.manager.username, 'Manager');

    await adminUsersPage.expectUserExists(testUsers.powerUser.username);
    await adminUsersPage.expectUserRole(testUsers.powerUser.username, 'Power User');

    await adminUsersPage.expectUserExists(testUsers.user.username);
    await adminUsersPage.expectUserRole(testUsers.user.username, 'User');

    // Admin can't act on self (rail shows "You"); can act on every lower-ranked user.
    await adminUsersPage.expectNoActionsForUser(testUsers.admin.username);
    await adminUsersPage.expectCurrentUserIndicator(testUsers.admin.username);
    await adminUsersPage.expectActionsForUser(testUsers.manager.username);
    await adminUsersPage.expectActionsForUser(testUsers.powerUser.username);
    await adminUsersPage.expectActionsForUser(testUsers.user.username);

    const userCount = await adminUsersPage.getUserCount();
    expect(userCount).toBe(4);

    // Admin can promote anyone to Admin.
    await adminUsersPage.expectRoleAvailable(testUsers.manager.username, 'Admin');
    await adminUsersPage.expectRoleAvailable(testUsers.powerUser.username, 'Admin');
    await adminUsersPage.expectRoleAvailable(testUsers.user.username, 'Admin');

    // Demote manager → User, then restore → Manager (rail select + Save).
    await adminUsersPage.changeUserRole(testUsers.manager.username, 'resource_user', 'User');
    await adminPage.reload();
    await adminUsersPage.expectUsersPage();
    await adminUsersPage.expectUserRole(testUsers.manager.username, 'User');

    await adminUsersPage.changeUserRole(testUsers.manager.username, 'resource_manager', 'Manager');
    await adminPage.reload();
    await adminUsersPage.expectUsersPage();
    await adminUsersPage.expectUserRole(testUsers.manager.username, 'Manager');

    // Manager session: hierarchy enforcement.
    managerContext = await browser.newContext();
    const managerPageNew = await managerContext.newPage();
    const managerLoginNew = new LoginPage(
      managerPageNew,
      baseUrl,
      authServerConfig,
      testUsers.manager
    );
    await managerLoginNew.performOAuthLogin();

    const managerUsersPageNew = new AllUsersPage(managerPageNew, baseUrl);
    await managerUsersPageNew.navigateToUsers();
    await managerUsersPageNew.expectUsersPage();

    await managerUsersPageNew.expectNoActionsForUser(testUsers.manager.username);
    await managerUsersPageNew.expectCurrentUserIndicator(testUsers.manager.username);

    await managerUsersPageNew.expectNoActionsForUser(testUsers.admin.username);
    await managerUsersPageNew.expectRestrictedUserIndicator(testUsers.admin.username);

    await managerUsersPageNew.expectActionsForUser(testUsers.powerUser.username);
    await managerUsersPageNew.expectActionsForUser(testUsers.user.username);

    // Manager can't grant Admin (would outrank self).
    await managerUsersPageNew.expectRoleNotAvailable(testUsers.powerUser.username, 'Admin');
    await managerUsersPageNew.expectRoleAvailable(testUsers.powerUser.username, 'Manager');
    await managerUsersPageNew.expectRoleAvailable(testUsers.powerUser.username, 'Power User');
    await managerUsersPageNew.expectRoleAvailable(testUsers.powerUser.username, 'User');

    await managerUsersPageNew.expectRoleNotAvailable(testUsers.user.username, 'Admin');
    await managerUsersPageNew.expectRoleAvailable(testUsers.user.username, 'Manager');

    // Power user can't reach Manage Users at all.
    powerUserContext = await browser.newContext();
    const powerUserPage = await powerUserContext.newPage();
    const powerUserLogin = new LoginPage(
      powerUserPage,
      baseUrl,
      authServerConfig,
      testUsers.powerUser
    );
    await powerUserLogin.performOAuthLogin();

    await powerUserPage.goto(`${baseUrl}/ui/users`);
    await powerUserPage.waitForLoadState('networkidle');
    await powerUserPage.waitForURL(`${baseUrl}/ui/login/?error=insufficient-role`);
    expect(powerUserPage.url()).toBe(`${baseUrl}/ui/login/?error=insufficient-role`);

    // Manager demotes power-user → User, promotes user → Power User.
    await managerUsersPageNew.changeUserRole(testUsers.powerUser.username, 'resource_user', 'User');
    await managerUsersPageNew.navigateToUsers();
    await managerUsersPageNew.expectUsersPage();
    await managerUsersPageNew.expectUserRole(testUsers.powerUser.username, 'User');

    await managerUsersPageNew.changeUserRole(
      testUsers.user.username,
      'resource_power_user',
      'Power User'
    );

    // The (now demoted) power-user's session loses elevated access.
    await powerUserPage.goto(`${baseUrl}/ui/models`);
    await powerUserPage.waitForLoadState('networkidle');
    await powerUserPage.waitForURL(`${baseUrl}/ui/login/`);

    await adminUsersPage.navigateToUsers();
    await adminUsersPage.expectUsersPage();
    await adminUsersPage.expectUserRole(testUsers.powerUser.username, 'User');
    await adminUsersPage.expectUserRole(testUsers.user.username, 'Power User');

    await managerPageNew.reload();
    await managerUsersPageNew.waitForSPAReady();
    await managerUsersPageNew.expectUsersPage();
    await managerUsersPageNew.expectUserRole(testUsers.powerUser.username, 'User');
    await managerUsersPageNew.expectUserRole(testUsers.user.username, 'Power User');

    if (powerUserContext) {
      await powerUserContext.close();
      powerUserContext = null;
    }

    // Admin removes the demoted power-user via the rail's two-click confirm.
    await adminUsersPage.navigateToUsers();
    await adminUsersPage.expectUsersPage();
    await adminUsersPage.removeUser(testUsers.powerUser.username);
    await adminUsersPage.expectUserNotExists(testUsers.powerUser.username);

    // The remaining user is still present (not removed).
    await adminUsersPage.expectUserExists(testUsers.user.username);

    await managerUsersPageNew.expectNoActionsForUser(testUsers.admin.username);
    await managerUsersPageNew.expectRestrictedUserIndicator(testUsers.admin.username);

    await adminUsersPage.expectNoActionsForUser(testUsers.admin.username);
    await adminUsersPage.expectCurrentUserIndicator(testUsers.admin.username);
    await managerUsersPageNew.expectNoActionsForUser(testUsers.manager.username);
    await managerUsersPageNew.expectCurrentUserIndicator(testUsers.manager.username);

    await managerPageNew.reload();
    await managerUsersPageNew.waitForSPAReady();
    await managerUsersPageNew.expectUsersPage();
    await managerUsersPageNew.expectUserNotExists(testUsers.powerUser.username);
    await managerUsersPageNew.expectUserExists(testUsers.user.username);

    const finalUserCount = await adminUsersPage.getUserCount();
    expect(finalUserCount).toBe(3);

    if (powerUserContext) {
      await powerUserContext.close();
    }
    if (managerContext) {
      await managerContext.close();
    }
    if (adminContext) {
      await adminContext.close();
    }
  });
});
