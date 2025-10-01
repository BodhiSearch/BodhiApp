import { test, expect } from '@playwright/test';
import {
  createAuthServerTestClient,
  getAuthServerConfig,
  getTestCredentials,
} from '@/utils/auth-server-client.mjs';
import { createServerManager } from '@/utils/bodhi-app-server.mjs';
import { randomPort } from '@/test-helpers.mjs';
import { LoginPage } from '@/pages/LoginPage.mjs';
import { ModelsListPage } from '@/pages/ModelsListPage.mjs';
import { LocalModelFormPage } from '@/pages/LocalModelFormPage.mjs';
import { ChatPage } from '@/pages/ChatPage.mjs';
import { LocalModelFixtures } from '@/fixtures/localModelFixtures.mjs';

test.describe('Local Model Alias Management - Consolidated User Journeys', () => {
  let serverManager;
  let baseUrl;
  let loginPage;
  let modelsPage;
  let formPage;
  let chatPage;
  let testData;

  test.beforeAll(async () => {
    // Server setup
    const authServerConfig = getAuthServerConfig();
    const testCredentials = getTestCredentials();
    const port = randomPort();
    const serverUrl = `http://localhost:${port}`;

    const authClient = createAuthServerTestClient(authServerConfig);
    const resourceClient = await authClient.createResourceClient(serverUrl);
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
    });

    baseUrl = await serverManager.startServer();
    testData = { authServerConfig, testCredentials };
  });

  test.beforeEach(async ({ page }) => {
    loginPage = new LoginPage(page, baseUrl, testData.authServerConfig, testData.testCredentials);
    modelsPage = new ModelsListPage(page, baseUrl);
    formPage = new LocalModelFormPage(page, baseUrl);
    chatPage = new ChatPage(page, baseUrl);
  });

  test.afterAll(async () => {
    if (serverManager) {
      await serverManager.stopServer();
    }
  });

  test('complete local model lifecycle with chat integration and context parameters', async ({
    page,
  }) => {
    const testData = LocalModelFixtures.createComprehensiveLifecycleData();
    const primaryData = testData.primaryAlias;
    const secondaryData = testData.secondaryAlias;
    const chatData = testData.chatTest;
    const contextData = testData.contextParamsTest;

    // Step 1: Login and navigate to models
    await loginPage.performOAuthLogin();
    await modelsPage.navigateToModels();

    // Step 2: Create primary model alias with full parameters
    await modelsPage.clickNewModelAlias();
    await formPage.waitForFormReady();

    // Fill basic information
    await formPage.fillBasicInfo(primaryData.alias, primaryData.repo, primaryData.filename);

    // Fill context parameters
    await formPage.fillContextParams(primaryData.contextParams);

    // Fill request parameters
    await formPage.fillRequestParams(primaryData.requestParams);

    // Create the alias
    await formPage.createAlias();

    // Step 3: Verify primary alias appears in models list
    await modelsPage.verifyLocalModelInList(
      primaryData.alias,
      primaryData.repo,
      primaryData.filename,
      'user'
    );

    // Step 4: Edit the primary alias
    await modelsPage.editLocalModel(primaryData.alias);
    await formPage.waitForFormReady();

    // Verify we're in edit mode
    expect(await formPage.isEditMode()).toBe(true);

    // Update context parameters and some request parameters
    await formPage.fillContextParams(primaryData.updatedData.contextParams);
    await formPage.fillRequestParams(primaryData.updatedData.requestParams);

    // Update the alias
    await formPage.updateAlias();

    // Step 5: Create alias from existing model file
    await modelsPage.createAliasFromModel(secondaryData.sourceModelAlias);

    // Verify we're on the new alias form with pre-populated data
    await formPage.waitForFormReady();

    const formData = await formPage.getFormData();
    expect(formData.repo).toBe('bartowski/microsoft_Phi-4-mini-instruct-GGUF');
    expect(formData.filename).toBe('microsoft_Phi-4-mini-instruct-Q4_K_M.gguf');

    // Fill in just the alias name and create
    await formPage.fillBasicInfo(secondaryData.alias, '', ''); // Only fill alias, repo/filename pre-filled
    await formPage.createAlias();

    // Verify secondary alias appears in list
    await modelsPage.verifyLocalModelInList(
      secondaryData.alias,
      'bartowski/microsoft_Phi-4-mini-instruct-GGUF',
      'microsoft_Phi-4-mini-instruct-Q4_K_M.gguf',
      'user'
    );

    // Step 6: Test chat integration with primary alias
    await modelsPage.chatWithLocalModel(primaryData.alias);

    // Verify we're on chat page with model pre-selected
    await chatPage.expectChatPageWithModel(primaryData.alias);

    // Send a test message and wait for any response (just verify the model works)
    await chatPage.sendMessage(chatData.message);
    await chatPage.waitForResponseComplete();

    // Verify we got some response (model is working)
    const userMessages = await chatPage.page.locator(chatPage.selectors.userMessage).count();
    const assistantMessages = await chatPage.page
      .locator(chatPage.selectors.assistantMessage)
      .count();
    expect(userMessages).toBeGreaterThan(0); // At least one user message
    expect(assistantMessages).toBeGreaterThan(0); // At least one assistant response

    // Step 7: Test external link functionality
    await modelsPage.navigateToModels();
    const externalButton = await modelsPage.openExternalLink(primaryData.alias);

    // Verify button has correct href (should point to HuggingFace)
    const href = await externalButton.getAttribute('onclick');
    expect(href || (await externalButton.evaluate((el) => el.getAttribute('title')))).toContain(
      'HuggingFace'
    );

    // Step 8: Test advanced context parameters
    await modelsPage.clickNewModelAlias();
    await formPage.waitForFormReady();

    // Fill form with advanced context parameters
    await formPage.fillBasicInfo(contextData.alias, contextData.repo, contextData.filename);
    await formPage.fillContextParams(contextData.advancedParams);

    // Create the alias
    await formPage.createAlias();

    // Edit the alias to verify context parameters persistence
    await modelsPage.editLocalModel(contextData.alias);
    await formPage.waitForFormReady();

    // Verify context parameters are populated correctly
    const contextValue = await page.locator('[data-testid="context-params"]').inputValue();
    expect(contextValue).toBe(contextData.advancedParams);

    // Step 9: Navigate back to models and verify all aliases exist
    await modelsPage.navigateToModels();

    // Verify all created aliases are present
    await modelsPage.verifyLocalModelInList(
      primaryData.alias,
      primaryData.repo,
      primaryData.filename,
      'user'
    );
    await modelsPage.verifyLocalModelInList(
      secondaryData.alias,
      'bartowski/microsoft_Phi-4-mini-instruct-GGUF',
      'microsoft_Phi-4-mini-instruct-Q4_K_M.gguf',
      'user'
    );
    await modelsPage.verifyLocalModelInList(
      contextData.alias,
      contextData.repo,
      contextData.filename,
      'user'
    );

    // Verify source badges show 'user' type for all created aliases
    await modelsPage.verifyModelTypeBadge(primaryData.alias, 'user');
    await modelsPage.verifyModelTypeBadge(secondaryData.alias, 'user');
    await modelsPage.verifyModelTypeBadge(contextData.alias, 'user');
  });

  test('comprehensive validation and error handling', async ({ page }) => {
    const validationData = LocalModelFixtures.createComprehensiveValidationData();
    const validData = validationData.validTest;
    const missingFields = validationData.missingFields;
    const duplicateTest = validationData.duplicateTest;

    // Step 1: Login and navigate to models
    await loginPage.performOAuthLogin();
    await modelsPage.navigateToModels();

    // Step 2: Test missing required fields validation
    await modelsPage.clickNewModelAlias();
    await formPage.waitForFormReady();

    // Test missing alias validation
    await formPage.fillBasicInfo(
      missingFields.missingAlias.alias,
      missingFields.missingAlias.repo,
      missingFields.missingAlias.filename
    );

    const submitButton = page.locator('[data-testid="submit-alias-form"]');
    await submitButton.click();

    // Verify we're still on the form page (didn't submit)
    await expect(page.url()).toContain('/ui/models/new');

    // Clear form and test missing repo
    await page.reload();
    await formPage.waitForFormReady();
    await formPage.fillBasicInfo(
      missingFields.missingRepo.alias,
      missingFields.missingRepo.repo,
      missingFields.missingRepo.filename
    );
    await submitButton.click();
    await expect(page.url()).toContain('/ui/models/new');

    // Clear form and test missing filename
    await page.reload();
    await formPage.waitForFormReady();
    await formPage.fillBasicInfo(
      missingFields.missingFilename.alias,
      missingFields.missingFilename.repo,
      missingFields.missingFilename.filename
    );
    await submitButton.click();
    await expect(page.url()).toContain('/ui/models/new');

    // Step 3: Create valid alias for duplicate testing
    await page.reload();
    await formPage.waitForFormReady();

    await formPage.fillBasicInfo(
      duplicateTest.baseAlias,
      duplicateTest.repo,
      duplicateTest.filename
    );
    await formPage.createAlias();

    // Step 5: Test duplicate alias name validation
    await modelsPage.clickNewModelAlias();
    await formPage.waitForFormReady();

    await formPage.fillBasicInfo(
      duplicateTest.duplicateAlias,
      duplicateTest.repo,
      duplicateTest.filename
    );
    await submitButton.click();

    // Should show error about duplicate alias or stay on form
    await expect(page.url()).toContain('/ui/models/new');

    // Step 6: Create valid alias to verify form works correctly
    await page.reload();
    await formPage.waitForFormReady();

    await formPage.fillBasicInfo(validData.alias, validData.repo, validData.filename);
    await formPage.fillContextParams(validData.contextParams);
    await formPage.fillRequestParams(validData.requestParams);

    // This should succeed
    await formPage.createAlias();

    // Verify successful creation
    await modelsPage.verifyLocalModelInList(
      validData.alias,
      validData.repo,
      validData.filename,
      'user'
    );
  });

  test('advanced features and edge cases', async ({ page }) => {
    const advancedData = LocalModelFixtures.createContextParamsTestData();
    const chatData = LocalModelFixtures.createChatIntegrationTestData();

    // Step 1: Login and navigate to models
    await loginPage.performOAuthLogin();
    await modelsPage.navigateToModels();

    // Step 2: Test different context parameter formats
    await modelsPage.clickNewModelAlias();
    await formPage.waitForFormReady();

    // Test multi-line context parameters
    await formPage.fillBasicInfo(advancedData.alias, advancedData.repo, advancedData.filename);
    await formPage.fillContextParams(advancedData.contextParams);

    // Create alias
    await formPage.createAlias();

    // Step 3: Verify context parameter persistence and formatting
    await modelsPage.editLocalModel(advancedData.alias);
    await formPage.waitForFormReady();

    // Verify context parameters are populated correctly
    const contextValue = await page.locator('[data-testid="context-params"]').inputValue();
    expect(contextValue).toBe(advancedData.contextParams);

    // Step 4: Test chat integration with advanced parameters
    await modelsPage.navigateToModels();
    await modelsPage.chatWithLocalModel(advancedData.alias);

    // Verify we're on chat page with model pre-selected
    await chatPage.expectChatPageWithModel(advancedData.alias);

    // Send a test message and wait for response
    await chatPage.sendMessage(chatData.message);
    await chatPage.waitForResponseComplete();

    // Verify we got some response (model is working)
    const userMessages = await chatPage.page.locator(chatPage.selectors.userMessage).count();
    const assistantMessages = await chatPage.page
      .locator(chatPage.selectors.assistantMessage)
      .count();
    expect(userMessages).toBeGreaterThan(0); // At least one user message
    expect(assistantMessages).toBeGreaterThan(0); // At least one assistant response

    // Step 5: Test empty context parameters (edge case)
    await modelsPage.navigateToModels();
    await modelsPage.clickNewModelAlias();
    await formPage.waitForFormReady();

    const emptyParamsAlias = `empty-params-${Date.now()}`;
    await formPage.fillBasicInfo(emptyParamsAlias, advancedData.repo, advancedData.filename);
    // Leave context parameters empty
    await formPage.fillContextParams('');

    // Create alias - should work with empty context params
    await formPage.createAlias();

    // Verify creation
    await modelsPage.verifyLocalModelInList(
      emptyParamsAlias,
      advancedData.repo,
      advancedData.filename,
      'user'
    );
  });
});
