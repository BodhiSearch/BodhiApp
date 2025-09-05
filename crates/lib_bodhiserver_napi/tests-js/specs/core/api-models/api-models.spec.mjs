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

test.describe('API Models Integration', () => {
  let serverManager;
  let baseUrl;
  let loginPage;
  let modelsPage;
  let formPage;
  let chatPage;
  let testData;

  test.beforeAll(async () => {
    // Verify environment setup
    const { apiKey } = ApiModelFixtures.getRequiredEnvVars();

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
    testData = { apiKey, authServerConfig, testCredentials };
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

  test('complete API model lifecycle with OpenAI integration and chat testing', async ({
    page,
  }) => {
    const modelData = ApiModelFixtures.createLifecycleTestData();

    // Step 1: Login and navigate to models
    await loginPage.performOAuthLogin();
    await modelsPage.navigateToModels();
    await modelsPage.clickNewApiModel();

    // Step 2: Create API model with validation testing - select only gpt-4 for chat testing
    await formPage.waitForFormReady();
    await formPage.fillBasicInfo(modelData.modelId, testData.apiKey, modelData.baseUrl);
    await formPage.fetchAndSelectModels(['gpt-4']); // Select only gpt-4 for chat integration
    await formPage.testConnection();
    await formPage.createModel();

    // Step 3: Verify model appears in models list
    await modelsPage.verifyApiModelInList(modelData.modelId, modelData.provider, modelData.baseUrl);

    // Step 4: Test chat integration with model
    const modelName = 'gpt-4'; // Use the exact model we selected during creation
    await modelsPage.clickChatWithModel(modelName);

    // Verify we're on chat page with model pre-selected
    await chatPage.expectChatPageWithModel(modelName);

    // Test chat functionality
    const testMessage = 'What day comes after Monday?';
    await chatPage.sendMessage(testMessage);
    await chatPage.waitForAssistantResponse();

    // Verify response contains expected answer
    const response = await chatPage.getLastAssistantMessage();
    await expect(response.toLowerCase()).toContain('tuesday');

    // Navigate back to models for further testing
    await modelsPage.navigateToModels();

    // Step 5: Test edit functionality with pre-filled data validation
    await modelsPage.editModel(modelData.modelId);
    await formPage.waitForFormReady();
    await formPage.verifyFormPreFilled(modelData.modelId, modelData.provider, modelData.baseUrl);
    await formPage.testConnection(); // Test with masked API key
    await formPage.updateModel();

    // Step 5: Verify model is still in the list after edit
    await modelsPage.verifyApiModelInList(modelData.modelId, modelData.provider, modelData.baseUrl);

    // Step 6: Navigate back to models and test delete functionality
    await modelsPage.navigateToModels();
    await modelsPage.deleteModel(modelData.modelId);
  });

  test('API model form validation and connection testing', async ({ page }) => {
    const modelData = ApiModelFixtures.scenarios.BASIC_OPENAI();

    await loginPage.performOAuthLogin();
    await modelsPage.navigateToModels();
    await modelsPage.clickNewApiModel();

    // Test step-by-step form validation
    await formPage.waitForFormReady();
    await formPage.fillBasicInfo(modelData.modelId, testData.apiKey);
    await formPage.fetchAndSelectModels(['gpt-3.5-turbo']);
    await formPage.testConnection();
    await formPage.createModel();

    // Verify creation and cleanup
    await modelsPage.verifyApiModelInList(modelData.modelId);
    await modelsPage.deleteModel(modelData.modelId);
  });
});
