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
    await formPage.fillBasicInfo(testData.apiKey, modelData.baseUrl);
    await formPage.fetchAndSelectModels(['gpt-4']); // Select only gpt-4 for chat integration
    await formPage.testConnection();

    // Capture the generated ID when creating the model
    const createdModelId = await formPage.createModelAndCaptureId();

    // Step 3: Navigate back to models to verify creation
    await modelsPage.navigateToModels();
    await modelsPage.waitForModelsToLoad();

    // Verify the model appears in the list using the captured ID
    await modelsPage.verifyApiModelInList(createdModelId, 'openai', modelData.baseUrl);

    // Step 4: Test chat integration with model
    const modelName = 'gpt-4'; // Use the exact model we selected during creation
    await modelsPage.clickChatWithModel(modelName);

    // Verify we're on chat page with model pre-selected
    await chatPage.expectChatPageWithModel(modelName);

    // Test chat functionality
    const testMessage = 'What day comes after Monday?';
    await chatPage.sendMessage(testMessage);
    await chatPage.waitForResponseComplete();

    // Verify response contains expected answer
    const response = await chatPage.getLastAssistantMessage();
    expect(response.toLowerCase()).toContain('tuesday');

    // Step 5: Test edit functionality with captured ID
    await modelsPage.navigateToModels();
    await modelsPage.editModel(createdModelId);
    await formPage.waitForFormReady();

    // Verify form is pre-filled correctly
    await formPage.verifyFormPreFilled('openai', modelData.baseUrl);

    // Make a small change and update
    await formPage.setPrefix('test:');
    await formPage.updateModel();

    // Step 6: Clean up by deleting the model
    await modelsPage.navigateToModels();
    await modelsPage.deleteModel(createdModelId);
  });

  test('API model form validation and connection testing', async ({ page }) => {
    const modelData = ApiModelFixtures.scenarios.BASIC_OPENAI();

    await loginPage.performOAuthLogin();
    await modelsPage.navigateToModels();
    await modelsPage.clickNewApiModel();

    // Test step-by-step form validation
    await formPage.waitForFormReady();
    await formPage.fillBasicInfo(testData.apiKey, modelData.baseUrl);
    await formPage.fetchAndSelectModels(['gpt-3.5-turbo']);
    await formPage.testConnection();

    // Capture the generated ID when creating the model
    const validationTestModelId = await formPage.createModelAndCaptureId();

    // Verify creation by checking the model appears in list
    await modelsPage.navigateToModels();
    await modelsPage.waitForModelsToLoad();
    await modelsPage.verifyApiModelInList(validationTestModelId, 'openai', modelData.baseUrl);

    // Clean up by deleting the created model
    await modelsPage.deleteModel(validationTestModelId);
  });
});
