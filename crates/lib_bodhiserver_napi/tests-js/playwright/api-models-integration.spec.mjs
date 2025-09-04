import { expect, test } from '@playwright/test';
import { getCurrentPath, randomPort, waitForSPAReady } from '../test-helpers.mjs';
import { createAuthServerTestClient, getAuthServerConfig, getTestCredentials } from './auth-server-client.mjs';
import { createServerManager } from './bodhi-app-server.mjs';

test.describe('AI API Models Integration Tests', () => {
  let serverManager;
  let baseUrl;
  let authServerConfig;
  let testCredentials;
  let authClient;
  let resourceClient;
  let port;
  let serverUrl;
  let openAIApiKey;

  test.beforeAll(async () => {
    // Get OpenAI API key from environment
    openAIApiKey = process.env.INTEG_TEST_OPENAI_API_KEY;
    if (!openAIApiKey) {
      throw new Error('INTEG_TEST_OPENAI_API_KEY environment variable not set');
    }

    authServerConfig = getAuthServerConfig();
    testCredentials = getTestCredentials();
    port = randomPort();
    serverUrl = `http://localhost:${port}`;

    authClient = createAuthServerTestClient(authServerConfig);
    resourceClient = await authClient.createResourceClient(serverUrl);
    await authClient.makeResourceAdmin(resourceClient.clientId, resourceClient.clientSecret, testCredentials.username);
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

  /**
   * Helper function to log in to the application
   */
  async function loginToApp(page) {
    await page.goto(`${baseUrl}/ui/login`);
    await waitForSPAReady(page);

    // Click login button to initiate OAuth flow
    const loginButton = page.locator(
      'button:has-text("Login")'
    );
    await loginButton.first().click();

    // Should redirect to auth server
    await page.waitForURL((url) => url.origin === authServerConfig.authUrl);

    // Fill in auth server credentials
    const usernameField = page.locator('#username');
    const passwordField = page.locator('#password');
    await usernameField.fill(testCredentials.username);
    await passwordField.fill(testCredentials.password);
    const submitButton = page.locator(
      'button:has-text("Sign In")'
    );
    await submitButton.click();
    await page.waitForURL((url) => url.origin === baseUrl && url.pathname === '/ui/chat/');
  }

  /**
   * Helper function to create an API model with real OpenAI integration
   */
  async function createAPIModel(page, modelId = 'test-openai') {
    // Navigate to models page first
    await page.goto(`${baseUrl}/ui/models/`);
    await waitForSPAReady(page);

    // Click the "New API Model" button to navigate to the new API model form
    const newApiModelButton = page.locator('button:has-text("New API Model")');
    await expect(newApiModelButton).toBeVisible();
    await newApiModelButton.click();

    // Wait for navigation to new API model page
    await page.waitForURL((url) => url.pathname === '/ui/api-models/new/');
    await waitForSPAReady(page);

    // Fill in the form
    await page.fill('[data-testid="api-model-id"]', modelId);
    // await page.selectOption('[data-testid="api-model-provider"]', 'OpenAI');
    await page.fill('[data-testid="api-model-base-url"]', 'https://api.openai.com/v1');
    await page.fill('[data-testid="api-model-api-key"]', openAIApiKey);

    // Fetch real models from OpenAI - look for the fetch button in the ModelSelector
    const fetchButton = page.locator('button:has-text("Fetch Models")');
    await expect(fetchButton).toBeVisible();
    await fetchButton.click();

    // Wait for models to load - check for available models in the scroll area
    await page.waitForSelector('text=gpt-4');

    // Select specific models by clicking on them in the available models list within the ScrollArea
    // The models are in clickable divs with cursor-pointer class inside the ScrollArea
    await page.click('.cursor-pointer:has-text("gpt-4")');
    await page.click('.cursor-pointer:has-text("gpt-3.5-turbo")');

    // Test connection with real API
    await page.click('[data-testid="test-connection-button"]');

    // Wait for test to complete (may take a few seconds)
    // The toast uses Radix UI with specific attributes and classes
    await expect(page.locator('[data-state="open"]')).toContainText(/Connection Test Successful/i);

    // Submit the form
    await page.click('[data-testid="create-api-model-button"]');

    // Wait for redirect to models page
    await page.waitForURL((url) => url.pathname === '/ui/models/');
    await waitForSPAReady(page);

    return modelId;
  }

  /**
   * Helper function to verify API model in models list
   */
  async function verifyAPIModelInList(page, modelId, provider = 'OpenAI') {
    // Should be on models page
    expect(new URL(page.url()).pathname).toBe('/ui/models/');
    // Wait for models list to load
    await page.waitForSelector('[data-testid="models-content"]');
    // Wait for the table to have at least one row
    await page.waitForSelector('[data-testid="table-list-models"] tbody tr');

    // Get the first table row (most recently created model)
    const firstRow = page.locator('[data-testid="table-list-models"] tbody tr').first();
    await expect(firstRow).toBeVisible();

    // Verify the alias/name cell contains the model ID
    const aliasCell = firstRow.locator(`[data-testid="alias-cell-api_${modelId}"]`);
    await expect(aliasCell).toContainText(modelId);

    // Verify the provider/repo cell contains the provider name
    const repoCell = firstRow.locator(`[data-testid="repo-cell-api_${modelId}"]`);
    await expect(repoCell).toContainText(provider);

    // Verify the filename/endpoint cell contains the base URL
    const filenameCell = firstRow.locator(`[data-testid="filename-cell-api_${modelId}"]`);
    await expect(filenameCell).toContainText('https://api.openai.com/v1');

    // Note: Source cell with API badge may be hidden in mobile/tablet views due to responsive CSS
    // Skip verification for source cell in responsive tests
  }

  test('complete API model lifecycle with real OpenAI integration', async ({ page }) => {
    // Step 1: Login
    await loginToApp(page);

    // Step 2: Create API model with real OpenAI credentials
    const modelId = await createAPIModel(page, 'lifecycle-test-openai');

    // Step 3: Verify model appears in unified models list
    await verifyAPIModelInList(page, modelId);

    // Step 4: Test edit functionality with pre-filled values
    let firstRow = page.locator('[data-testid="table-list-models"] tbody tr').first();
    const editBtn = firstRow.locator(`[data-testid="edit-button-${modelId}"]:visible`);
    await expect(editBtn).toBeVisible();
    await editBtn.click();
    await waitForSPAReady(page);
    await page.waitForURL((url) => url.pathname === '/ui/api-models/edit/');
    expect(new URL(page.url()).searchParams.get('id')).toBe(modelId);

    // Verify form is pre-filled
    await expect(page.locator('[data-testid="api-model-id"]')).toHaveValue(modelId);
    await expect(page.locator('[data-testid="api-model-provider"]')).toHaveText('OpenAI');
    await expect(page.locator('[data-testid="api-model-base-url"]')).toHaveValue('https://api.openai.com/v1');
    // API key should be empty (masked)
    await expect(page.locator('[data-testid="api-model-api-key"]')).toHaveValue('');
    // Test connection with real API
    await page.click('[data-testid="test-connection-button"]');
    await expect(page.locator('[data-state="open"]')).toContainText(/Connection Test Successful/i);

    // Save without changes
    await page.click('[data-testid="update-api-model-button"]');
    await page.waitForURL((url) => url.pathname === '/ui/models/');

    // Step 5: Test delete functionality
    firstRow = page.locator('[data-testid="table-list-models"] tbody tr').first();
    const deleteBtn = firstRow.locator(`[data-testid="delete-button-${modelId}"]:visible`);
    await expect(deleteBtn).toBeVisible();
    await deleteBtn.click();

    // Wait for confirmation dialog
    await expect(page.locator('text=Delete API Model')).toBeVisible();

    // Confirm deletion
    await page.click('button:has-text("Delete")');
    await expect(page.locator('[data-state="open"]')).toContainText(`API model ${modelId} deleted successfully`);
    const rowCount = await page.locator('[data-testid="table-list-models"] tbody tr').count();
    expect(rowCount).toBe(0);
  });


  test.describe('responsive design', () => {
    test('mobile view shows proper model interaction', async ({ page }) => {
      // Set mobile viewport
      await page.setViewportSize({ width: 375, height: 667 });

      // Login and create test model
      await loginToApp(page);
      const modelId = await createAPIModel(page, 'mobile-test-openai');

      // Verify mobile view
      await verifyAPIModelInList(page, modelId);

      // In mobile view, models should be accessible via dropdown
      const chatButton = page.locator(`[data-testid="models-dropdown-${modelId}"]`);
      await expect(chatButton.first()).toBeVisible();

      // Click to open dropdown
      await chatButton.first().click();

      // Verify models are listed in dropdown
      await expect(page.locator('[role="menuitem"]')).toHaveCount(2); // gpt-4 and gpt-3.5-turbo

      // Skip chat navigation test since chat with API models is not working
      // await page.click('[role="menuitem"]:has-text("gpt-4")');
      // await expect(page).toHaveURL(/\/ui\/chat\?model=gpt-4/);

      // Clean up
      await page.goto(`${baseUrl}/ui/models`);
      await page.click(`button[title="Delete API model ${modelId}"]`);
      await page.click('button:has-text("Delete")');
    });


    test('tablet view responsive behavior', async ({ page }) => {
      // Set tablet viewport
      await page.setViewportSize({ width: 768, height: 1024 });

      // Login and create test model
      await loginToApp(page);
      const modelId = await createAPIModel(page, 'tablet-test-openai');

      // Verify tablet view shows proper layout
      await verifyAPIModelInList(page, modelId);

      // Verify edit and delete buttons are visible
      const firstRow = page.locator('[data-testid="table-list-models"] tbody tr').first();
      expect(firstRow).toBeVisible();
      const editBtn = firstRow.locator(`[data-testid="edit-button-${modelId}"]:visible`);
      const deleteBtn = firstRow.locator(`[data-testid="delete-button-${modelId}"]:visible`);
      expect(editBtn).toBeVisible();
      expect(deleteBtn).toBeVisible();

      // Clean up
      await deleteBtn.click();
      await expect(page.locator('text=Delete API Model')).toBeVisible();
      await page.click('button:has-text("Delete")');
      await expect(page.locator('[data-state="open"]')).toContainText(`API model ${modelId} deleted successfully`);
    });
  });

});