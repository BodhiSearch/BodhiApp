import { MultiTenantLoginPage } from '@/pages/MultiTenantLoginPage.mjs';
import { RequestAccessPage } from '@/pages/RequestAccessPage.mjs';
import { TenantRegistrationPage } from '@/pages/TenantRegistrationPage.mjs';
import { UsersManagementPage } from '@/pages/UsersManagementPage.mjs';
import { getAuthServerConfig, getTestCredentials } from '@/utils/auth-server-client.mjs';
import { expect, test } from '@/fixtures.mjs';

test.describe('Multi-Tenant Lifecycle', () => {
  let authServerConfig;
  let userCredentials;
  let managerCredentials;

  test.beforeAll(async () => {
    authServerConfig = getAuthServerConfig();
    userCredentials = getTestCredentials(); // user@email.com — resource admin on pre-seeded tenant
    managerCredentials = {
      username: process.env.INTEG_TEST_USER_MANAGER,
      password: process.env.INTEG_TEST_PASSWORD,
    };
  });

  test('register tenant, invite via link, request access, approve, switch, logout', async ({
    browser,
    sharedServerUrl,
  }) => {
    let managerContext;
    let userContext;
    let managerTenantClientId; // manager's Test Tenant client_id — used for invite link

    try {
      // ── Step 1: Cleanup — manager@email.com cleans stale tenants ──
      await test.step('Cleanup stale tenants for manager', async () => {
        managerContext = await browser.newContext();
        const managerPage = await managerContext.newPage();
        const managerLogin = new MultiTenantLoginPage(
          managerPage,
          sharedServerUrl,
          authServerConfig,
          managerCredentials
        );

        // Dashboard login as manager@email.com
        await managerLogin.navigate('/ui/login');
        await managerLogin.performDashboardLogin();

        // Cleanup stale tenants + reset DB to clean state
        const cleanupResponse = await managerPage.request.get(
          `${sharedServerUrl}/dev/tenants/cleanup`
        );
        expect(cleanupResponse.status()).toBeLessThan(400);
        const resetResponse = await managerPage.request.get(`${sharedServerUrl}/dev/db-reset`);
        expect(resetResponse.status()).toBeLessThan(400);

        // db-reset clears sessions → manager must re-login
        // After login, manager has 0 memberships → status='setup' → auto-redirect to tenant setup
        await managerLogin.navigate('/ui/login');
        await managerLogin.performDashboardLogin();
        await managerLogin.waitForTenantSetup();
      });

      // ── Step 2: Register tenant — manager creates "Test Tenant" ──
      let managerPage = managerContext.pages()[0];
      await test.step('Register new tenant as manager', async () => {
        const tenantReg = new TenantRegistrationPage(managerPage, sharedServerUrl);
        await tenantReg.fillTenantForm('Test Tenant');
        await tenantReg.submitTenantForm();
        // SSO auto-completes tenant OAuth → redirects to /ui/chat/
        await tenantReg.waitForCreated();
      });

      // ── Step 3: Get manager's Test Tenant client_id ──
      await test.step('Get Test Tenant client_id', async () => {
        managerPage = managerContext.pages()[0];
        const infoResponse = await managerPage.request.get(`${sharedServerUrl}/bodhi/v1/info`);
        expect(infoResponse.ok()).toBeTruthy();
        const appInfo = await infoResponse.json();
        managerTenantClientId = appInfo.client_id;
        expect(managerTenantClientId).toBeTruthy();
      });

      // ── Step 4: user@email.com logs in — single pre-seeded tenant → auto-login ──
      await test.step('Login as user — auto-login to pre-seeded tenant', async () => {
        userContext = await browser.newContext();
        const userPage = await userContext.newPage();
        const userLogin = new MultiTenantLoginPage(
          userPage,
          sharedServerUrl,
          authServerConfig,
          userCredentials
        );

        await userLogin.navigate('/ui/login');
        await userLogin.performDashboardLogin();
        // user@email.com has exactly 1 tenant (pre-seeded) → auto-login
        await userLogin.waitForAutoLogin();
      });

      // ── Step 5: User navigates to invite link for manager's Test Tenant ──
      await test.step('User navigates to invite link', async () => {
        const userPage = userContext.pages()[0];
        // Navigate to invite URL — this stores client_id in sessionStorage
        // and triggers the invite flow
        await userPage.goto(`${sharedServerUrl}/ui/login/?invite=${managerTenantClientId}`);
        await userPage.waitForLoadState('domcontentloaded');
      });

      // ── Step 6: User completes OAuth for manager's Test Tenant → lands on request-access ──
      await test.step('User OAuth flow leads to request-access page', async () => {
        const userPage = userContext.pages()[0];

        // The invite flow processes: dashboard session exists → initiates OAuth for target tenant
        // User has no role in manager's Test Tenant → redirected to /ui/request-access/
        await userPage.waitForURL(
          (url) => url.origin === sharedServerUrl && url.pathname === '/ui/request-access/',
          { timeout: 30000 }
        );

        const requestAccessPage = new RequestAccessPage(userPage, sharedServerUrl);
        await requestAccessPage.expectRequestAccessPage();
      });

      // ── Step 7: User submits access request ──
      await test.step('User submits access request', async () => {
        const userPage = userContext.pages()[0];
        const requestAccessPage = new RequestAccessPage(userPage, sharedServerUrl);

        await requestAccessPage.expectRequestButtonVisible(true);
        await requestAccessPage.clickRequestAccess();
        await requestAccessPage.expectPendingState();
      });

      // ── Step 8: Manager approves user's access request ──
      await test.step('Manager approves user access request', async () => {
        managerPage = managerContext.pages()[0];
        const managerUsersPage = new UsersManagementPage(managerPage, sharedServerUrl);

        await managerUsersPage.navigateToPendingRequests();
        await managerUsersPage.expectRequestExists(userCredentials.username);
        await managerUsersPage.approveRequest(userCredentials.username, 'Manager');
      });

      // ── Step 9: User session was invalidated by approval → re-auth ──
      await test.step('User re-authenticates after approval', async () => {
        const userPage = userContext.pages()[0];

        // Try to navigate to a protected page — session invalidated → redirects to login
        await userPage.goto(`${sharedServerUrl}/ui/chat`);
        await userPage.waitForLoadState('networkidle');
        await userPage.waitForURL((url) => url.pathname === '/ui/login/');
      });

      // ── Step 10: User now has 2 tenants → switch to manager's Test Tenant ──
      await test.step('User switches to manager Test Tenant', async () => {
        const userPage = userContext.pages()[0];
        const userLogin = new MultiTenantLoginPage(
          userPage,
          sharedServerUrl,
          authServerConfig,
          userCredentials
        );

        // User now has 2 tenants: pre-seeded tenant + manager's Test Tenant
        // Dashboard session still active → should see tenant selection
        await userLogin.performDashboardLogin();
        await userLogin.waitForTenantSelection();

        // Select the manager's Test Tenant
        const testTenantButton = userPage.locator('[data-test-action]:has-text("Test Tenant")');
        await testTenantButton.click();

        // SSO auto-completes → /ui/chat/
        await userLogin.waitForAutoLogin();
      });

      // ── Step 11: Verify user's role on manager's Test Tenant ──
      await test.step('Verify user role is resource_manager', async () => {
        const userPage = userContext.pages()[0];
        const userInfoResponse = await userPage.request.get(`${sharedServerUrl}/bodhi/v1/user`);
        expect(userInfoResponse.ok()).toBeTruthy();
        const userInfo = await userInfoResponse.json();
        expect(userInfo.role).toBe('resource_manager');
      });

      // ── Step 12: Logout — verify State A ──
      await test.step('Logout and verify login state', async () => {
        const userPage = userContext.pages()[0];
        const userLogin = new MultiTenantLoginPage(
          userPage,
          sharedServerUrl,
          authServerConfig,
          userCredentials
        );

        await userLogin.navigate('/ui/login');
        await userLogin.logout();
        await userLogin.expectStateA();
      });
    } finally {
      if (managerContext) await managerContext.close();
      if (userContext) await userContext.close();
    }
  });
});
