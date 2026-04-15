import { ApiModelFixtures } from '@/fixtures/apiModelFixtures.mjs';
import { ApiModelFormPage } from '@/pages/ApiModelFormPage.mjs';
import { ChatPage } from '@/pages/ChatPage.mjs';
import { LoginPage } from '@/pages/LoginPage.mjs';
import { ModelsListPage } from '@/pages/ModelsListPage.mjs';
import { getAuthServerConfig, getTestCredentials } from '@/utils/auth-server-client.mjs';
import { expect, test } from '@/fixtures.mjs';

// Tests for the `forward_all_with_prefix` API model mode.
//
// The sync-models endpoint (POST /bodhi/v1/models/api/{id}/sync-models) requires a
// browser session cookie, so page.evaluate with credentials:'include' is intentional
// here — it reuses the authenticated session rather than a Bearer token.

test.describe('API Models Forward All With Prefix', () => {
  let loginPage;
  let modelsPage;
  let formPage;
  let chatPage;
  let testData;

  test.beforeAll(async () => {
    const { apiKey } = ApiModelFixtures.getRequiredEnvVars();
    testData = {
      apiKey,
      authServerConfig: getAuthServerConfig(),
      testCredentials: getTestCredentials(),
    };
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

  test('forward_all_with_prefix: create, sync models, chat, prefix uniqueness, delete', async ({
    page,
  }) => {
    await loginPage.performOAuthLogin();
    await modelsPage.navigateToModels();

    // Create the forward_all model with prefix 'fwd/'
    await modelsPage.clickNewApiModel();
    await formPage.form.waitForFormReady();
    await formPage.form.fillBasicInfoWithPrefix(
      testData.apiKey,
      'fwd/',
      'https://api.openai.com/v1'
    );
    await formPage.form.enableForwardAll();
    await formPage.form.expectModelSelectionState('disabled');
    const forwardAllModelId = await formPage.createModelAndCaptureId();

    // Synchronously populate the model cache via the sync-models endpoint.
    // Uses page.evaluate with credentials:'include' because this endpoint
    // requires a browser session cookie, not a Bearer token.
    const syncResult = await page.evaluate(async (modelId) => {
      const syncResp = await fetch(`/bodhi/v1/models/api/${modelId}/sync-models`, {
        method: 'POST',
        credentials: 'include',
      });
      if (!syncResp.ok) {
        throw new Error(`sync-models failed: ${syncResp.status}`);
      }
      const sync = await syncResp.json();
      const modelsResp = await fetch('/v1/models', { credentials: 'include' });
      const modelsData = await modelsResp.json();
      return { sync, models: modelsData.data };
    }, forwardAllModelId);

    expect(syncResult.sync.id).toBe(forwardAllModelId);
    expect(syncResult.sync.prefix).toBe('fwd/');
    expect(syncResult.sync.forward_all_with_prefix).toBe(true);
    expect(syncResult.models.some((m) => m.id.startsWith('fwd/'))).toBe(true);

    // Verify the model row in the list
    await modelsPage.navigateToModels();
    const apiModel = await modelsPage.getModelRow(forwardAllModelId);
    expect(apiModel).toMatchObject({
      id: forwardAllModelId,
      api_format: 'openai',
      base_url: 'https://api.openai.com/v1',
      prefix: 'fwd/',
      forward_all: 'Yes',
    });

    // Verify edit page reflects the correct radio selection
    await modelsPage.editModel(forwardAllModelId);
    await formPage.form.verifyForwardAllModeSelected();
    await formPage.form.verifyPrefixValue('fwd/');

    // Chat page shows prefixed models from the synced cache
    await chatPage.navigateToChat();
    await chatPage.waitForChatPageLoad();
    await chatPage.toggleModelCombobox();
    await chatPage.expectModelOptionVisible(`fwd/${ApiModelFixtures.OPENAI_MODEL}`);
    await chatPage.toggleModelCombobox();

    // Send a message via the prefixed model
    const prefixedModel = `fwd/${ApiModelFixtures.OPENAI_MODEL}`;
    await chatPage.selectModel(prefixedModel);
    await chatPage.sendMessage('What is 1+1?');
    await chatPage.waitForResponseComplete();
    const response = await chatPage.getLastAssistantMessage();
    expect(response.length).toBeGreaterThan(0);
    expect(response.toLowerCase()).toContain('2');

    // Prefix uniqueness: a second model with the same prefix should fail
    await modelsPage.navigateToModels();
    await modelsPage.clickNewApiModel();
    await formPage.form.waitForFormReady();
    await formPage.form.fillBasicInfoWithPrefix(
      testData.apiKey,
      'fwd/',
      'https://api.openai.com/v1'
    );
    await formPage.form.enableForwardAll();
    await page.click('[data-testid="create-api-model-button"]');
    await formPage.form.waitForToast(/Prefix.*already.*used/i);

    // Cleanup
    await modelsPage.navigateToModels();
    await modelsPage.deleteModel(forwardAllModelId);

    // Verify prefixed models disappear from chat after deletion
    await chatPage.navigateToChat();
    await chatPage.waitForChatPageLoad();
    await chatPage.toggleModelCombobox();
    await chatPage.expectModelOptionNotVisible(`fwd/${ApiModelFixtures.OPENAI_MODEL}`);
  });

  test('toggle between forward_all and selected_models modes', async ({ page }) => {
    await loginPage.performOAuthLogin();
    await modelsPage.navigateToModels();
    await modelsPage.clickNewApiModel();
    await formPage.form.waitForFormReady();

    // Without prefix, forward_all is disabled
    await formPage.form.expectForwardAllDisabled();

    // Adding a prefix enables the forward_all radio
    await formPage.form.setPrefix('test/');
    await formPage.form.expectForwardAllEnabled();

    // Enable forward_all → model selection disabled
    await formPage.form.enableForwardAll();
    await formPage.form.expectModelSelectionState('disabled');

    // Switch back to selected_models → model selection enabled
    await formPage.form.selectModelsMode();
    await formPage.form.expectModelSelectionState('enabled');

    // Fetch and select models normally
    await formPage.form.fillApiKey(testData.apiKey);
    await formPage.form.fetchAndSelectModels([ApiModelFixtures.OPENAI_MODEL]);

    const selectedModeModelId = await formPage.createModelAndCaptureId();
    await modelsPage.navigateToModels();
    await modelsPage.deleteModel(selectedModeModelId);
  });
});
