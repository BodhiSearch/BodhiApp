import { ApiModelFixtures } from '@/fixtures/apiModelFixtures.mjs';
import { ApiModelFormPage } from '@/pages/ApiModelFormPage.mjs';
import { ChatPage } from '@/pages/ChatPage.mjs';
import { LoginPage } from '@/pages/LoginPage.mjs';
import { ModelsListPage } from '@/pages/ModelsListPage.mjs';
import { getAuthServerConfig, getTestCredentials } from '@/utils/auth-server-client.mjs';
import { expect, test } from '@/fixtures.mjs';

for (const [formatKey, formatConfig] of Object.entries(ApiModelFixtures.API_FORMATS)) {
  test.describe(`API Models Integration [${formatConfig.format}]`, () => {
    let loginPage;
    let modelsPage;
    let formPage;
    let chatPage;
    let testData;

    test.beforeAll(async () => {
      // Verify environment setup
      const apiKey = process.env[formatConfig.envKey];
      if (!apiKey) {
        throw new Error(`${formatConfig.envKey} environment variable not set`);
      }

      // Server setup
      const authServerConfig = getAuthServerConfig();
      const testCredentials = getTestCredentials();

      // Use shared server started by Playwright webServer
      testData = { apiKey, authServerConfig, testCredentials };
    });

    test.beforeEach(async ({ page, sharedServerUrl }) => {
      loginPage = new LoginPage(
        page,
        sharedServerUrl,
        testData.authServerConfig,
        testData.testCredentials
      );
      modelsPage = new ModelsListPage(page, sharedServerUrl);
      formPage = new ApiModelFormPage(page, sharedServerUrl);
      chatPage = new ChatPage(page, sharedServerUrl);
    });

    test('complete API model lifecycle with integration and chat testing', async ({ page }) => {
      const modelData = ApiModelFixtures.createModelDataForFormat(formatKey);

      // Step 1: Login and navigate to models
      await loginPage.performOAuthLogin();
      await modelsPage.navigateToModels();
      await modelsPage.clickNewApiModel();

      // Step 2: Create API model - select format, fill form, fetch models, test connection
      await formPage.form.waitForFormReady();
      await formPage.form.selectApiFormat(formatConfig.format);
      await formPage.form.fillBasicInfo(testData.apiKey, modelData.baseUrl);
      await formPage.form.fetchAndSelectModels([formatConfig.model]);
      await formPage.form.testConnection();

      // Capture the generated ID when creating the model
      const createdModelId = await formPage.createModelAndCaptureId();

      // Step 3: Navigate back to models to verify creation
      await modelsPage.navigateToModels();
      await modelsPage.waitForModelsToLoad();

      // Verify the model appears in the list using the captured ID
      await modelsPage.verifyApiModelInList(createdModelId, formatConfig.format, modelData.baseUrl);

      // Step 4: Test chat integration with model
      const modelName = formatConfig.model;
      await modelsPage.clickChatWithModel(modelName);

      // Verify we're on chat page with model pre-selected
      await chatPage.expectChatPageWithModel(modelName);

      // Wait for API format to sync from model config (prevents race with AliasSelector useEffect)
      await chatPage.waitForApiFormat(formatConfig.formatDisplayName);

      // Test chat functionality
      await chatPage.sendMessage(formatConfig.chatQuestion);
      await chatPage.waitForResponseComplete();

      // Verify response contains expected answer
      const response = await chatPage.getLastAssistantMessage();
      expect(response.toLowerCase()).toContain(formatConfig.chatExpected);

      // Step 5: Test edit functionality with captured ID
      await modelsPage.navigateToModels();
      await modelsPage.editModel(createdModelId);
      await formPage.form.waitForFormReady();

      // Verify form is pre-filled correctly
      await formPage.form.verifyFormPreFilled(formatConfig.format, modelData.baseUrl);

      // Make a small change and update
      await formPage.form.setPrefix('test:');
      await formPage.updateModel();

      // Step 6: Clean up by deleting the model
      await modelsPage.navigateToModels();
      await modelsPage.deleteModel(createdModelId);
    });

    test('API model form validation and connection testing', async ({ page }) => {
      const modelData = ApiModelFixtures.createModelDataForFormat(formatKey);

      await loginPage.performOAuthLogin();
      await modelsPage.navigateToModels();
      await modelsPage.clickNewApiModel();

      // Test step-by-step form validation
      await formPage.form.waitForFormReady();
      await formPage.form.selectApiFormat(formatConfig.format);
      await formPage.form.fillBasicInfo(testData.apiKey, modelData.baseUrl);
      await formPage.form.fetchAndSelectModels([formatConfig.model]);
      await formPage.form.testConnection();

      // Capture the generated ID when creating the model
      const validationTestModelId = await formPage.createModelAndCaptureId();

      // Verify creation by checking the model appears in list
      await modelsPage.navigateToModels();
      await modelsPage.waitForModelsToLoad();
      await modelsPage.verifyApiModelInList(
        validationTestModelId,
        formatConfig.format,
        modelData.baseUrl
      );

      // Clean up by deleting the created model
      await modelsPage.deleteModel(validationTestModelId);
    });

    test('authentication error and recovery flow', async ({ page }) => {
      const modelData = ApiModelFixtures.createModelDataForFormat(formatKey);

      await loginPage.performOAuthLogin();
      await modelsPage.navigateToModels();
      await modelsPage.clickNewApiModel();

      await formPage.form.waitForFormReady();

      await formPage.form.selectApiFormat(formatConfig.format);
      await formPage.form.expectBaseUrlValue(modelData.baseUrl);

      await formPage.form.uncheckUseApiKey();

      await formPage.form.clickFetchModels();
      await formPage.form.expectFetchError();

      await formPage.form.checkUseApiKey();
      await formPage.form.fillApiKey(testData.apiKey);

      await formPage.form.fetchAndSelectModels([formatConfig.model]);

      await formPage.form.testConnection();

      const authTestModelId = await formPage.createModelAndCaptureId();

      await modelsPage.navigateToModels();
      await modelsPage.waitForModelsToLoad();
      await modelsPage.verifyApiModelInList(
        authTestModelId,
        formatConfig.format,
        modelData.baseUrl
      );

      await modelsPage.deleteModel(authTestModelId);
    });
  });
}
