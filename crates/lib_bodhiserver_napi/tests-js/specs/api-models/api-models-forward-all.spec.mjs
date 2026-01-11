import { ApiModelFixtures } from '@/fixtures/apiModelFixtures.mjs';
import { ApiModelFormPage } from '@/pages/ApiModelFormPage.mjs';
import { ChatPage } from '@/pages/ChatPage.mjs';
import { LoginPage } from '@/pages/LoginPage.mjs';
import { ModelsListPage } from '@/pages/ModelsListPage.mjs';
import { randomPort } from '@/test-helpers.mjs';
import {
  createAuthServerTestClient,
  getAuthServerConfig,
  getTestCredentials,
} from '@/utils/auth-server-client.mjs';
import { createServerManager } from '@/utils/bodhi-app-server.mjs';
import { expect, test } from '@playwright/test';

test.describe('API Models Forward All With Prefix', () => {
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

  test('forward_all_with_prefix lifecycle: create, list models, chat, prefix uniqueness, delete', async ({
    page,
  }) => {
    // ===== SETUP: Login and navigate =====
    await loginPage.performOAuthLogin();
    await modelsPage.navigateToModels();

    // ===== SCENARIO A: Create model with forward_all_with_prefix =====
    await modelsPage.clickNewApiModel();
    await formPage.form.waitForFormReady();

    // Fill basic info with OpenAI
    await formPage.form.fillBasicInfoWithPrefix(
      testData.apiKey,
      'fwd/', // prefix for forward all
      'https://api.openai.com/v1'
    );

    // Enable forward all mode (should disable model selection)
    await formPage.form.enableForwardAll();

    // Verify model selection is disabled
    await formPage.form.expectModelSelectionState('disabled');

    // Create model and capture ID
    const forwardAllModelId = await formPage.createModelAndCaptureId();

    // Synchronously populate cache by calling sync-models endpoint
    // Also trigger a /v1/models call to populate frontend cache
    const syncResult = await page.evaluate(async (modelId) => {
      const response = await fetch(`/bodhi/v1/api-models/${modelId}/sync-models`, {
        method: 'POST',
        credentials: 'include',
      });
      if (!response.ok) {
        throw new Error(`Failed to sync models: ${response.status}`);
      }
      const result = await response.json();

      const modelsResponse = await fetch('/v1/models', { credentials: 'include' });
      const modelsData = await modelsResponse.json();

      return {
        sync: result,
        models: modelsData.data,
      };
    }, forwardAllModelId);

    // Assert on sync result
    expect(syncResult.sync.id).toBe(forwardAllModelId);
    expect(syncResult.sync.prefix).toBe('fwd/');
    expect(syncResult.sync.forward_all_with_prefix).toBe(true);
    expect(syncResult.models.some((m) => m.id.startsWith('fwd/'))).toBe(true);

    // ===== SCENARIO B: Verify model in list with prefix and forward_all columns =====
    await modelsPage.navigateToModels();
    const apiModel = await modelsPage.getModelRow(forwardAllModelId);
    expect(apiModel).toMatchObject({
      id: forwardAllModelId,
      api_format: 'openai',
      base_url: 'https://api.openai.com/v1',
      prefix: 'fwd/',
      forward_all: 'Yes',
    });

    // ===== SCENARIO B2: Verify edit page shows correct radio button selection =====
    // Navigate to edit page for the forward_all model
    await modelsPage.editModel(forwardAllModelId);

    // Verify forward_all radio button is selected
    await formPage.form.verifyForwardAllModeSelected();
    // Verify prefix input shows correct value
    await formPage.form.verifyPrefixValue('fwd/');

    // Navigate to chat and verify prefixed models appear
    await chatPage.navigateToChat();
    await chatPage.waitForChatPageLoad();

    // Open model combobox and verify prefixed models appear
    await chatPage.toggleModelCombobox();
    await chatPage.expectModelOptionVisible('fwd/gpt-4');
    await chatPage.toggleModelCombobox();

    // ===== SCENARIO C: Chat with forward_all prefixed model =====
    const prefixedModel = 'fwd/gpt-4';
    await chatPage.selectModel(prefixedModel);

    await chatPage.sendMessage('What is 1+1?');
    await chatPage.waitForResponseComplete();
    const response = await chatPage.getLastAssistantMessage();
    expect(response.length).toBeGreaterThan(0);
    expect(response.toLowerCase()).toContain('2');

    // ===== SCENARIO D: Prefix uniqueness - try creating model with same prefix =====
    await modelsPage.navigateToModels();
    await modelsPage.clickNewApiModel();
    await formPage.form.waitForFormReady();

    await formPage.form.fillBasicInfoWithPrefix(
      testData.apiKey,
      'fwd/', // Same prefix - should fail
      'https://api.openai.com/v1'
    );
    await formPage.form.enableForwardAll();

    // Try to create - should show error
    await page.click('[data-testid="create-api-model-button"]');
    await formPage.form.waitForToast(/prefix.*already exists|prefix_exists/i);

    // ===== SCENARIO E: Delete forward_all model =====
    await modelsPage.navigateToModels();
    await modelsPage.deleteModel(forwardAllModelId);

    // ===== SCENARIO F: Verify models removed from chat =====
    await chatPage.navigateToChat();
    await chatPage.waitForChatPageLoad();

    // Open model combobox and verify prefixed models are no longer visible
    await chatPage.toggleModelCombobox();
    await chatPage.expectModelOptionNotVisible('fwd/gpt-4');
  });

  test('toggle between forward_all and selected_models modes', async ({ page }) => {
    // ===== Test UI behavior of switching modes =====
    await loginPage.performOAuthLogin();
    await modelsPage.navigateToModels();
    await modelsPage.clickNewApiModel();
    await formPage.form.waitForFormReady();

    // Initially forward_all should be disabled (no prefix)
    await formPage.form.expectForwardAllDisabled();

    // Enable prefix
    await formPage.form.setPrefix('test/');

    // Now forward_all should be enabled
    await formPage.form.expectForwardAllEnabled();

    // Enable forward_all mode
    await formPage.form.enableForwardAll();

    // Model selection should be disabled
    await formPage.form.expectModelSelectionState('disabled');

    // Switch back to selected models mode
    await formPage.form.selectModelsMode();

    // Model selection should be enabled again
    await formPage.form.expectModelSelectionState('enabled');

    // Fetch and select models normally
    await formPage.form.fillApiKey(testData.apiKey);
    await formPage.form.fetchAndSelectModels(['gpt-4']);

    // Create with selected models mode
    const selectedModeModelId = await formPage.createModelAndCaptureId();

    // Cleanup
    await modelsPage.navigateToModels();
    await modelsPage.deleteModel(selectedModeModelId);
  });
});
