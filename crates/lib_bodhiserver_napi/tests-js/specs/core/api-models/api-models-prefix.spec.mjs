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
      testCredentials.username
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
    const baseNoPrefix = ApiModelFixtures.scenarios.BASIC_OPENAI();
    const azureModel = ApiModelFixtures.scenarios.WITH_PREFIX();
    const customModel = ApiModelFixtures.scenarios.CUSTOM_PREFIX();

    // ===== SINGLE SETUP: Login and navigate (done once) =====
    await loginPage.performOAuthLogin();
    await modelsPage.navigateToModels();

    // ===== SCENARIO A: Create basic model without prefix =====
    await modelsPage.clickNewApiModel();
    await formPage.waitForFormReady();
    await formPage.fillBasicInfo(baseNoPrefix.modelId, testData.apiKey, baseNoPrefix.baseUrl);
    await formPage.fetchAndSelectModels(['gpt-4']);
    await formPage.testConnection();
    await formPage.createModel();

    // Verify it appears in list without prefix
    await modelsPage.verifyApiModelInList(baseNoPrefix.modelId, 'openai', baseNoPrefix.baseUrl);

    // ===== SCENARIO B: Create model with OpenRouter prefix =====
    await modelsPage.clickNewApiModel();
    await formPage.waitForFormReady();

    await formPage.fillBasicInfoWithPrefix(
      azureModel.modelId,
      testData.openrouterApiKey,
      'openrouter/',
      azureModel.baseUrl
    );
    await formPage.fetchAndSelectModels(['openai/gpt-3.5-turbo']);
    await formPage.testConnection();
    await formPage.createModel();

    // Verify prefixed model appears correctly and test chat
    await modelsPage.verifyApiModelInList(azureModel.modelId, 'openai', azureModel.baseUrl);
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
    await modelsPage.editModel(baseNoPrefix.modelId);
    await formPage.waitForFormReady();

    // Verify form is pre-filled without prefix
    await formPage.verifyFormPreFilledWithPrefix(
      baseNoPrefix.modelId,
      'openai',
      baseNoPrefix.baseUrl,
      null
    );

    // Add openai: prefix to existing model
    await formPage.setPrefix('openai:');
    await formPage.updateModel();

    // Test chat with newly prefixed model
    const openaiPrefixedModel = 'openai:gpt-4';
    await modelsPage.clickChatWithModel(openaiPrefixedModel);
    await chatPage.expectChatPageWithModel(openaiPrefixedModel);

    // ===== SCENARIO D: Create third model with custom OpenRouter prefix =====
    await modelsPage.navigateToModels();
    await modelsPage.clickNewApiModel();
    await formPage.waitForFormReady();

    await formPage.fillBasicInfoWithPrefix(
      customModel.modelId,
      testData.openrouterApiKey,
      'custom-',
      customModel.baseUrl
    );
    await formPage.fetchAndSelectModels(['openai/gpt-4']);
    await formPage.createModel();

    // ===== SCENARIO E: Edit OpenRouter model to remove prefix =====
    await modelsPage.editModel(azureModel.modelId);
    await formPage.waitForFormReady();

    // Remove the prefix
    await formPage.disablePrefix();
    await formPage.updateModel();

    // Test chat still works with unprefixed name
    await modelsPage.clickChatWithModel('openai/gpt-3.5-turbo'); // No prefix now (but still has OpenRouter model format)
    await chatPage.expectChatPageWithModel('openai/gpt-3.5-turbo');

    // ===== SCENARIO F: Multi-model verification =====
    await modelsPage.navigateToModels();

    // Verify all models display correctly in list:
    await modelsPage.verifyApiModelInList(baseNoPrefix.modelId, 'openai', baseNoPrefix.baseUrl); // now has openai: prefix
    await modelsPage.verifyApiModelInList(
      azureModel.modelId,
      'openai', // Shows OpenAI format even when using OpenRouter
      azureModel.baseUrl
    ); // prefix removed (OpenRouter)
    await modelsPage.verifyApiModelInList(
      customModel.modelId,
      'openai', // Shows OpenAI format even when using OpenRouter
      customModel.baseUrl
    ); // has custom- prefix (OpenRouter)

    // Test chat navigation for different prefix types
    await modelsPage.clickChatWithModel('openai:gpt-4'); // OpenAI with colon prefix
    await chatPage.expectChatPageWithModel('openai:gpt-4');

    await modelsPage.navigateToModels();
    await modelsPage.clickChatWithModel('custom-openai/gpt-4'); // OpenRouter with dash prefix
    await chatPage.expectChatPageWithModel('custom-openai/gpt-4');

    // ===== CLEANUP: Delete all created models =====
    await modelsPage.navigateToModels();
    await modelsPage.deleteModel(baseNoPrefix.modelId);
    await modelsPage.deleteModel(azureModel.modelId);
    await modelsPage.deleteModel(customModel.modelId);
  });

  test('prefix form validation, UI behavior and edge cases', async ({ page }) => {
    // This test focuses on validation, UI behavior, and edge cases for prefix functionality
    // combining form validation, api_format persistence, and edge cases

    const modelData = ApiModelFixtures.scenarios.CUSTOM_PREFIX();

    // ===== SINGLE SETUP: Login and navigate =====
    await loginPage.performOAuthLogin();
    await modelsPage.navigateToModels();

    // ===== UI BEHAVIOR TESTING =====
    await modelsPage.clickNewApiModel();
    await formPage.waitForFormReady();

    // Test initial state - prefix checkbox unchecked, input disabled
    await expect(formPage.page.locator(formPage.selectors.usePrefixCheckbox)).not.toBeChecked();
    await expect(formPage.page.locator(formPage.selectors.prefixInput)).toBeDisabled();

    // Enable prefix and verify input becomes visible
    await formPage.enablePrefix();
    await expect(formPage.page.locator(formPage.selectors.usePrefixCheckbox)).toBeChecked();
    await expect(formPage.page.locator(formPage.selectors.prefixInput)).toBeVisible();

    // Test prefix input validation with special characters
    await formPage.page.fill(formPage.selectors.prefixInput, 'valid-prefix_123');

    // Disable prefix and verify input becomes disabled
    await formPage.disablePrefix();
    await expect(formPage.page.locator(formPage.selectors.usePrefixCheckbox)).not.toBeChecked();
    await expect(formPage.page.locator(formPage.selectors.prefixInput)).toBeDisabled();

    // ===== PREFIX FUNCTIONALITY TESTING =====

    // Fill form with OpenRouter API key and URL
    await formPage.fillBasicInfoWithPrefix(
      modelData.modelId,
      testData.openrouterApiKey,
      'openai-new:',
      modelData.baseUrl
    );
    await formPage.fetchAndSelectModels(['openai/gpt-4']);
    await formPage.createModel();

    // ===== PERSISTENCE TESTING =====

    // Edit the created model and verify prefix persists correctly
    await modelsPage.editModel(modelData.modelId);
    await formPage.waitForFormReady();
    await formPage.verifyFormPreFilledWithPrefix(
      modelData.modelId,
      'openai', // Shows OpenAI format even when using OpenRouter
      'https://openrouter.ai/api/v1', // OpenRouter URL
      'openai-new:'
    );

    // Modify prefix to test update functionality
    await formPage.setPrefix('updated-prefix-');
    await formPage.updateModel();

    // Verify the update worked by editing again
    await modelsPage.editModel(modelData.modelId);
    await formPage.waitForFormReady();
    await formPage.verifyFormPreFilledWithPrefix(
      modelData.modelId,
      'openai', // Shows OpenAI format even when using OpenRouter
      'https://openrouter.ai/api/v1',
      'updated-prefix-'
    );

    // ===== EDGE CASE: Prefix with trailing slash and URL normalization =====
    await modelsPage.navigateToModels();
    await modelsPage.clickNewApiModel();
    await formPage.waitForFormReady();

    const edgeCaseId = `edge-case-${Date.now()}`;
    // Use base URL with trailing slash and prefix with slash
    await formPage.fillBasicInfoWithPrefix(
      edgeCaseId,
      testData.openrouterApiKey,
      'test/',
      'https://openrouter.ai/api/v1/' // URL with trailing slash
    );
    await formPage.fetchAndSelectModels(['openai/gpt-3.5-turbo']);
    await formPage.createModel();

    // Verify model was created and works (URL should be normalized)
    await modelsPage.verifyApiModelInList(edgeCaseId, 'openai', 'https://openrouter.ai/api/v1'); // Normalized URL
    await modelsPage.clickChatWithModel('test/openai/gpt-3.5-turbo');
    await chatPage.expectChatPageWithModel('test/openai/gpt-3.5-turbo');

    // ===== EDGE CASE: Empty prefix =====
    await modelsPage.navigateToModels();
    await modelsPage.clickNewApiModel();
    await formPage.waitForFormReady();

    const emptyPrefixId = `empty-prefix-${Date.now()}`;
    await formPage.fillBasicInfo(emptyPrefixId, testData.apiKey);

    // Enable prefix but leave it empty
    await formPage.enablePrefix();
    await formPage.page.fill(formPage.selectors.prefixInput, ''); // Empty prefix
    await formPage.fetchAndSelectModels(['gpt-4']);
    await formPage.createModel();

    // Should work with empty prefix (acts like no prefix)
    await modelsPage.verifyApiModelInList(emptyPrefixId, 'openai', 'https://api.openai.com/v1');
    await modelsPage.clickChatWithModel('gpt-4');
    await chatPage.expectChatPageWithModel('gpt-4');

    // ===== CLEANUP =====
    await modelsPage.navigateToModels();
    await modelsPage.deleteModel(modelData.modelId);
    await modelsPage.deleteModel(edgeCaseId);
    await modelsPage.deleteModel(emptyPrefixId);
  });
});
