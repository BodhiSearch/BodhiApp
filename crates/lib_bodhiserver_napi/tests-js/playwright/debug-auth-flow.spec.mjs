import { expect, test } from '@playwright/test';
import { getCurrentPath, randomPort, waitForSPAReady } from '../test-helpers.mjs';
import { createAuthServerTestClient, getAuthServerConfig, getTestCredentials } from './auth-server-client.mjs';
import { createServerManager } from './bodhi-app-server.mjs';

test.describe('Debug Authentication Flow', () => {
  let serverManager;
  let baseUrl;
  let authServerConfig;
  let testCredentials;
  let authClient;
  let resourceClient;
  let port;
  let serverUrl;

  test.beforeAll(async () => {
    authServerConfig = getAuthServerConfig();
    testCredentials = getTestCredentials();
    port = randomPort();
    serverUrl = `http://localhost:${port}`;

    authClient = createAuthServerTestClient(authServerConfig);
    resourceClient = await authClient.createResourceClient(serverUrl);
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
  });

  test.afterAll(async () => {
    if (serverManager) {
      await serverManager.stopServer();
    }
  });

  test('debug authentication and navigation flow', async ({ page }) => {
    // Step 1: Go to login page
    console.log('Step 1: Navigating to login page');
    await page.goto(`${baseUrl}/ui/login`);
    await waitForSPAReady(page);
    
    console.log('Current URL after login page:', page.url());
    console.log('Page title:', await page.title());
    
    // Take a screenshot
    await page.screenshot({ path: 'debug-step1-login.png' });

    // Step 2: Click login button
    console.log('Step 2: Clicking login button');
    const loginButton = page.locator('button:has-text("Login")');
    await expect(loginButton).toBeVisible();
    await loginButton.first().click();

    // Step 3: Wait for auth server redirect
    console.log('Step 3: Waiting for auth server redirect');
    await page.waitForURL((url) => url.origin === authServerConfig.authUrl);
    console.log('Redirected to auth server:', page.url());

    // Step 4: Fill credentials
    console.log('Step 4: Filling credentials');
    const usernameField = page.locator('#username');
    const passwordField = page.locator('#password');
    await usernameField.fill(testCredentials.username);
    await passwordField.fill(testCredentials.password);
    
    const submitButton = page.locator('button:has-text("Sign In")');
    await submitButton.click();

    // Step 5: Wait for redirect back to app
    console.log('Step 5: Waiting for redirect back to app');
    await page.waitForURL((url) => url.origin === baseUrl);
    console.log('Redirected back to app:', page.url());
    await waitForSPAReady(page);
    
    // Take a screenshot after login
    await page.screenshot({ path: 'debug-step5-after-login.png' });

    // Step 6: Check user info API
    console.log('Step 6: Checking user info');
    const userInfoResponse = await page.request.get(`${baseUrl}/api/user/info`);
    console.log('User info response status:', userInfoResponse.status());
    if (userInfoResponse.status() === 200) {
      const userInfo = await userInfoResponse.json();
      console.log('User info:', userInfo);
    } else {
      const errorText = await userInfoResponse.text();
      console.log('User info error:', errorText);
    }

    // Step 7: Navigate to models page
    console.log('Step 7: Navigating to models page');
    await page.goto(`${baseUrl}/ui/models`);
    await waitForSPAReady(page);
    
    console.log('Current URL after models navigation:', page.url());
    
    // Take a screenshot of models page
    await page.screenshot({ path: 'debug-step7-models-page.png' });

    // Step 8: Check if we can see the page content
    console.log('Step 8: Checking page content');
    const pageContent = await page.textContent('body');
    console.log('Page contains "New API Model":', pageContent.includes('New API Model'));
    console.log('Page contains "Login":', pageContent.includes('Login'));
    
    // Try to find the button
    const newApiModelButton = page.locator('button:has-text("New API Model")');
    const buttonCount = await newApiModelButton.count();
    console.log('Number of "New API Model" buttons found:', buttonCount);
    
    if (buttonCount === 0) {
      // Debug what buttons are available
      const allButtons = page.locator('button');
      const buttonTexts = await allButtons.allTextContents();
      console.log('All button texts on page:', buttonTexts);
    }
  });
});
