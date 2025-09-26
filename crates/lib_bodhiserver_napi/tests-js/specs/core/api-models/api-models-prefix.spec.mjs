import { test, expect } from '@playwright/test';
import {
  createAuthServerTestClient,
  getAuthServerConfig,
  getTestCredentials,
} from '../../../playwright/auth-server-client.mjs';
import { createServerManager } from '../../../playwright/bodhi-app-server.mjs';
import { randomPort } from '../../../test-helpers.mjs';
import { LoginPage } from '../../../pages/LoginPage.mjs';
import { ModelsListPage } from '../../../pages/ModelsListPage.mjs';
import { ApiModelFormPage } from '../../../pages/ApiModelFormPage.mjs';
import { ChatPage } from '../../../pages/ChatPage.mjs';
import { ApiModelFixtures } from '../../../fixtures/apiModelFixtures.mjs';

test.describe('API Models Prefix Functionality', () => {
  let serverManager;
  let baseUrl;
  let loginPage;
  let modelsPage;
  let formPage;
  let chatPage;
  let testData;

  test.beforeAll(async () => {
    // Verify environment setup
    const { apiKey, openrouterApiKey } = ApiModelFixtures.getRequiredEnvVars();

    // Server setup (existing logic)
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
    testData = { apiKey, openrouterApiKey, authServerConfig, testCredentials };
  });

  test.beforeEach(async ({ page }) => {
    loginPage = new LoginPage(page, baseUrl, testData.authServerConfig, testData.testCredentials);
    modelsPage = new ModelsListPage(page, baseUrl);
    formPage = new ApiModelFormPage(page, baseUrl);
    chatPage = new ChatPage(page, baseUrl);
  });

  test.afterAll(async () => {
    if (serverManager) {
      await serverManager.stopServer();
    }
  });

  test('comprehensive API model prefix lifecycle with multi-api-format management', async ({
    page,
  }) => {
    // Updated to work with auto-generated UUIDs by capturing IDs from API responses
    const baseNoPrefix = ApiModelFixtures.scenarios.BASIC_OPENAI();
    const azureModel = ApiModelFixtures.scenarios.WITH_PREFIX();
    const customModel = ApiModelFixtures.scenarios.CUSTOM_PREFIX();

    // ===== SINGLE SETUP: Login and navigate (done once) =====
    await loginPage.performOAuthLogin();
    await modelsPage.navigateToModels();

    // ===== SCENARIO A: Create basic model without prefix =====
    await modelsPage.clickNewApiModel();
    await formPage.form.waitForFormReady();
    await formPage.form.fillBasicInfo(testData.apiKey, baseNoPrefix.baseUrl);
    await formPage.form.fetchAndSelectModels(['gpt-4']);
    await formPage.form.testConnection();

    // Capture the generated ID when creating the model
    const baseNoPrefixId = await formPage.createModelAndCaptureId();

    // Verify it appears in list without prefix using the captured ID
    await modelsPage.verifyApiModelInList(baseNoPrefixId, 'openai', baseNoPrefix.baseUrl);

    // ===== SCENARIO B: Create model with OpenRouter prefix =====
    await modelsPage.clickNewApiModel();
    await formPage.form.waitForFormReady();

    await formPage.form.fillBasicInfoWithPrefix(
      testData.openrouterApiKey,
      'openrouter/',
      azureModel.baseUrl
    );
    await formPage.form.fetchAndSelectModels(['openai/gpt-3.5-turbo']);
    await formPage.form.testConnection();

    // Capture the generated ID when creating the model with prefix
    const azureModelId = await formPage.createModelAndCaptureId();

    // Verify prefixed model appears correctly and test chat
    await modelsPage.verifyApiModelInList(azureModelId, 'openai', azureModel.baseUrl);
    const openrouterPrefixedModel = 'openrouter/openai/gpt-3.5-turbo';
    await modelsPage.clickChatWithModel(openrouterPrefixedModel);
    await chatPage.expectChatPageWithModel(openrouterPrefixedModel);

    // Test chat functionality with prefixed model
    await chatPage.sendMessage('What is 2+2?');
    await chatPage.waitForResponseComplete();
    const openrouterResponse = await chatPage.getLastAssistantMessage();
    expect(openrouterResponse.length).toBeGreaterThan(0);

    // ===== SCENARIO C: Edit existing model to add prefix =====
    await modelsPage.navigateToModels();
    await modelsPage.editModel(baseNoPrefixId);
    await formPage.form.waitForFormReady();

    // Verify form is pre-filled without prefix
    await formPage.form.verifyFormPreFilledWithPrefix('openai', baseNoPrefix.baseUrl, null);

    // Add openai: prefix to existing model
    await formPage.form.setPrefix('openai:');
    await formPage.updateModel();

    // Test chat with newly prefixed model
    const openaiPrefixedModel = 'openai:gpt-4';
    await modelsPage.clickChatWithModel(openaiPrefixedModel);
    await chatPage.expectChatPageWithModel(openaiPrefixedModel);

    // ===== SCENARIO D: Create third model with custom OpenRouter prefix =====
    await modelsPage.navigateToModels();
    await modelsPage.clickNewApiModel();
    await formPage.form.waitForFormReady();

    await formPage.form.fillBasicInfoWithPrefix(
      testData.openrouterApiKey,
      'custom-',
      customModel.baseUrl
    );
    await formPage.form.fetchAndSelectModels(['openai/gpt-4']);

    // Capture the generated ID when creating the custom model
    const customModelId = await formPage.createModelAndCaptureId();

    // ===== SCENARIO E: Edit OpenRouter model to remove prefix =====
    await modelsPage.editModel(azureModelId);
    await formPage.form.waitForFormReady();

    // Remove the prefix
    await formPage.form.disablePrefix();
    await formPage.updateModel();

    // Test chat still works with unprefixed name
    await modelsPage.clickChatWithModel('openai/gpt-3.5-turbo'); // No prefix now (but still has OpenRouter model format)
    await chatPage.expectChatPageWithModel('openai/gpt-3.5-turbo');

    // ===== SCENARIO F: Multi-model verification =====
    await modelsPage.navigateToModels();

    // Verify all models display correctly in list using captured IDs:
    await modelsPage.verifyApiModelInList(baseNoPrefixId, 'openai', baseNoPrefix.baseUrl); // now has openai: prefix
    await modelsPage.verifyApiModelInList(
      azureModelId,
      'openai', // Shows OpenAI format even when using OpenRouter
      azureModel.baseUrl
    ); // prefix removed (OpenRouter)
    await modelsPage.verifyApiModelInList(
      customModelId,
      'openai', // Shows OpenAI format even when using OpenRouter
      customModel.baseUrl
    ); // has custom- prefix (OpenRouter)

    // Test chat navigation for different prefix types
    await modelsPage.clickChatWithModel('openai:gpt-4'); // OpenAI with colon prefix
    await chatPage.expectChatPageWithModel('openai:gpt-4');

    await modelsPage.navigateToModels();
    await modelsPage.clickChatWithModel('custom-openai/gpt-4'); // OpenRouter with dash prefix
    await chatPage.expectChatPageWithModel('custom-openai/gpt-4');

    // ===== CLEANUP: Delete all created models using captured IDs =====
    await modelsPage.navigateToModels();
    await modelsPage.deleteModel(baseNoPrefixId);
    await modelsPage.deleteModel(azureModelId);
    await modelsPage.deleteModel(customModelId);
  });

  test('prefix form validation, UI behavior and edge cases', async ({ page }) => {
    // Updated to work with auto-generated UUIDs by capturing IDs from API responses
    // This test focuses on validation, UI behavior, and edge cases for prefix functionality
    // combining form validation, api_format persistence, and edge cases

    const modelData = ApiModelFixtures.scenarios.CUSTOM_PREFIX();

    // ===== SINGLE SETUP: Login and navigate =====
    await loginPage.performOAuthLogin();
    await modelsPage.navigateToModels();

    // ===== UI BEHAVIOR TESTING =====
    await modelsPage.clickNewApiModel();
    await formPage.form.waitForFormReady();

    // Test initial state - prefix checkbox unchecked, input disabled
    await expect(
      formPage.page.locator(formPage.form.selectors.usePrefixCheckbox)
    ).not.toBeChecked();
    await expect(formPage.page.locator(formPage.form.selectors.prefixInput)).toBeDisabled();

    // Enable prefix and verify input becomes visible
    await formPage.form.enablePrefix();
    await expect(formPage.page.locator(formPage.form.selectors.usePrefixCheckbox)).toBeChecked();
    await expect(formPage.page.locator(formPage.form.selectors.prefixInput)).toBeVisible();

    // Test prefix input validation with special characters
    await formPage.page.fill(formPage.form.selectors.prefixInput, 'valid-prefix_123');

    // Disable prefix and verify input becomes disabled
    await formPage.form.disablePrefix();
    await expect(
      formPage.page.locator(formPage.form.selectors.usePrefixCheckbox)
    ).not.toBeChecked();
    await expect(formPage.page.locator(formPage.form.selectors.prefixInput)).toBeDisabled();

    // ===== PREFIX FUNCTIONALITY TESTING =====

    // Fill form with OpenRouter API key and URL
    await formPage.form.fillBasicInfoWithPrefix(
      testData.openrouterApiKey,
      'openai-new:',
      modelData.baseUrl
    );
    await formPage.form.fetchAndSelectModels(['openai/gpt-4']);

    // Capture the generated ID when creating the model
    const createdModelId = await formPage.createModelAndCaptureId();

    // ===== PERSISTENCE TESTING =====

    // Edit the created model and verify prefix persists correctly
    await modelsPage.editModel(createdModelId);
    await formPage.form.waitForFormReady();
    await formPage.form.verifyFormPreFilledWithPrefix(
      'openai', // Shows OpenAI format even when using OpenRouter
      'https://openrouter.ai/api/v1', // OpenRouter URL
      'openai-new:'
    );

    // Modify prefix to test update functionality
    await formPage.form.setPrefix('updated-prefix-');
    await formPage.updateModel();

    // Verify the update worked by editing again
    await modelsPage.editModel(createdModelId);
    await formPage.form.waitForFormReady();
    await formPage.form.verifyFormPreFilledWithPrefix(
      'openai', // Shows OpenAI format even when using OpenRouter
      'https://openrouter.ai/api/v1',
      'updated-prefix-'
    );

    // ===== EDGE CASE: Prefix with trailing slash and URL normalization =====
    await modelsPage.navigateToModels();
    await modelsPage.clickNewApiModel();
    await formPage.form.waitForFormReady();

    // Use base URL with trailing slash and prefix with slash
    await formPage.form.fillBasicInfoWithPrefix(
      testData.openrouterApiKey,
      'test/',
      'https://openrouter.ai/api/v1/' // URL with trailing slash
    );
    await formPage.form.fetchAndSelectModels(['openai/gpt-3.5-turbo']);

    // Capture the generated ID for the edge case model
    const edgeCaseId = await formPage.createModelAndCaptureId();

    // Verify model was created and works (URL should be normalized)
    await modelsPage.verifyApiModelInList(edgeCaseId, 'openai', 'https://openrouter.ai/api/v1'); // Normalized URL
    await modelsPage.clickChatWithModel('test/openai/gpt-3.5-turbo');
    await chatPage.expectChatPageWithModel('test/openai/gpt-3.5-turbo');

    // ===== EDGE CASE: Empty prefix =====
    await modelsPage.navigateToModels();
    await modelsPage.clickNewApiModel();
    await formPage.form.waitForFormReady();

    await formPage.form.fillBasicInfo(testData.apiKey);

    // Enable prefix but leave it empty
    await formPage.form.enablePrefix();
    await formPage.page.fill(formPage.form.selectors.prefixInput, ''); // Empty prefix
    await formPage.form.fetchAndSelectModels(['gpt-4']);

    // Capture the generated ID for the empty prefix model
    const emptyPrefixId = await formPage.createModelAndCaptureId();

    // Should work with empty prefix (acts like no prefix)
    await modelsPage.verifyApiModelInList(emptyPrefixId, 'openai', 'https://api.openai.com/v1');
    await modelsPage.clickChatWithModel('gpt-4');
    await chatPage.expectChatPageWithModel('gpt-4');

    // ===== CLEANUP =====
    await modelsPage.navigateToModels();
    await modelsPage.deleteModel(createdModelId);
    await modelsPage.deleteModel(edgeCaseId);
    await modelsPage.deleteModel(emptyPrefixId);
  });
});
