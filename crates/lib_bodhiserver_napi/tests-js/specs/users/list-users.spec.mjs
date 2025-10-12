import { expect, test } from '@playwright/test';
import { AllUsersPage } from '@/pages/AllUsersPage.mjs';
import { LoginPage } from '@/pages/LoginPage.mjs';
import { createAuthServerTestClient, getAuthServerConfig } from '@/utils/auth-server-client.mjs';
import { createServerManager } from '@/utils/bodhi-app-server.mjs';
import { randomPort } from '@/test-helpers.mjs';

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

    await adminPage.waitForSelector('tbody tr');

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

    await adminUsersPage.expectNoActionsForUser(testUsers.admin.username);
    await adminUsersPage.expectCurrentUserIndicator(testUsers.admin.username);

    const managerRow = await adminUsersPage.findUserRowByUsername(testUsers.manager.username);
    const managerRemoveBtn = managerRow.locator(
      `[data-testid="remove-user-btn-${testUsers.manager.username}"]`
    );
    await expect(managerRemoveBtn).toBeVisible();

    const powerUserRow = await adminUsersPage.findUserRowByUsername(testUsers.powerUser.username);
    const powerUserRemoveBtn = powerUserRow.locator(
      `[data-testid="remove-user-btn-${testUsers.powerUser.username}"]`
    );
    await expect(powerUserRemoveBtn).toBeVisible();

    const userRow = await adminUsersPage.findUserRowByUsername(testUsers.user.username);
    const userRemoveBtn = userRow.locator(
      `[data-testid="remove-user-btn-${testUsers.user.username}"]`
    );
    await expect(userRemoveBtn).toBeVisible();

    await adminUsersPage.navigateToPendingRequests();
    await adminUsersPage.navigateToAllRequests();
    await adminUsersPage.navigateToUsers();
    await adminUsersPage.expectUsersPage();

    const userCount = await adminUsersPage.getUserCount();
    expect(userCount).toBe(4);

    await adminUsersPage.expectRoleAvailable(testUsers.manager.username, 'Admin');
    await adminUsersPage.expectRoleAvailable(testUsers.powerUser.username, 'Admin');
    await adminUsersPage.expectRoleAvailable(testUsers.user.username, 'Admin');

    await adminUsersPage.selectRoleForUser(testUsers.manager.username, 'User');
    await adminUsersPage.expectRoleChangeDialog();

    const confirmButton = adminPage.locator('[data-testid="role-change-confirm"]');
    await confirmButton.click();

    await adminPage.waitForSelector('[data-testid="role-change-dialog"]', { state: 'hidden' });

    await adminUsersPage.waitForRoleChangeSuccess();

    await adminPage.reload();
    await adminUsersPage.expectUsersPage();
    await adminUsersPage.expectUserRole(testUsers.manager.username, 'User');

    await adminUsersPage.selectRoleForUser(testUsers.manager.username, 'Manager');
    await adminUsersPage.expectRoleChangeDialog();
    const confirmButton2 = adminPage.locator('[data-testid="role-change-confirm"]');
    await confirmButton2.click();
    await adminPage.waitForSelector('[data-testid="role-change-dialog"]', { state: 'hidden' });
    await adminUsersPage.waitForRoleChangeSuccess();

    await adminPage.reload();
    await adminUsersPage.expectUsersPage();
    await adminUsersPage.expectUserRole(testUsers.manager.username, 'Manager');

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

    const managerPowerUserRow = await managerUsersPageNew.findUserRowByUsername(
      testUsers.powerUser.username
    );
    const managerPowerUserActions = managerPowerUserRow.locator('td:last-child');
    const managerPowerUserDropdown = managerPowerUserActions.locator('button[role="combobox"]');
    const managerPowerUserRemoveBtn = managerPowerUserActions.locator('button:has-text("Remove")');

    await expect(managerPowerUserDropdown).toBeVisible();
    await expect(managerPowerUserRemoveBtn).toBeVisible();

    const managerUserRow = await managerUsersPageNew.findUserRowByUsername(testUsers.user.username);
    const managerUserActions = managerUserRow.locator('td:last-child');
    const managerUserDropdown = managerUserActions.locator('button[role="combobox"]');
    const managerUserRemoveBtn = managerUserActions.locator('button:has-text("Remove")');

    await expect(managerUserDropdown).toBeVisible();
    await expect(managerUserRemoveBtn).toBeVisible();

    await managerUsersPageNew.expectRoleNotAvailable(testUsers.powerUser.username, 'Admin');
    await managerUsersPageNew.expectRoleAvailable(testUsers.powerUser.username, 'Manager');
    await managerUsersPageNew.expectRoleAvailable(testUsers.powerUser.username, 'Power User');
    await managerUsersPageNew.expectRoleAvailable(testUsers.powerUser.username, 'User');

    await managerUsersPageNew.expectRoleNotAvailable(testUsers.user.username, 'Admin');
    await managerUsersPageNew.expectRoleAvailable(testUsers.user.username, 'Manager');
    await managerUsersPageNew.expectRoleAvailable(testUsers.user.username, 'Power User');
    await managerUsersPageNew.expectRoleAvailable(testUsers.user.username, 'User');

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

    const currentUrl = powerUserPage.url();
    expect(currentUrl).toBe(`${baseUrl}/ui/login/?error=insufficient-role`);

    await managerUsersPageNew.changeUserRole(testUsers.powerUser.username, 'User');
    await managerUsersPageNew.waitForRoleChangeSuccess();
    await managerUsersPageNew.expectUserRole(testUsers.powerUser.username, 'User');

    await managerUsersPageNew.navigateToPendingRequests();
    await managerUsersPageNew.navigateToUsers();
    await managerUsersPageNew.expectUsersPage();
    await managerUsersPageNew.expectUserRole(testUsers.powerUser.username, 'User');

    await managerUsersPageNew.changeUserRole(testUsers.user.username, 'Power User');
    await managerUsersPageNew.waitForRoleChangeSuccess();
    await managerUsersPageNew.expectUserRole(testUsers.user.username, 'Power User');

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

    await adminUsersPage.navigateToUsers();
    await adminUsersPage.expectUsersPage();

    const powerUserRemoveButton = adminPage.locator(
      `[data-testid="remove-user-btn-${testUsers.powerUser.username}"]`
    );
    await expect(powerUserRemoveButton).toBeVisible();

    await adminUsersPage.removeUser(testUsers.powerUser.username);
    await adminUsersPage.waitForUserRemovalSuccess();

    await adminUsersPage.expectUserNotExists(testUsers.powerUser.username);

    const cancelTestRemoveBtn = adminPage.locator(
      `[data-testid="remove-user-btn-${testUsers.user.username}"]`
    );
    await cancelTestRemoveBtn.click();
    await adminUsersPage.expectRemoveUserDialog();
    await adminUsersPage.cancelRemoveUser();

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
