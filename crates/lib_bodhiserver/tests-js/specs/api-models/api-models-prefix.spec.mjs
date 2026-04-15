import { ApiModelFixtures } from '@/fixtures/apiModelFixtures.mjs';
import { ApiModelFormPage } from '@/pages/ApiModelFormPage.mjs';
import { ChatPage } from '@/pages/ChatPage.mjs';
import { LoginPage } from '@/pages/LoginPage.mjs';
import { ModelsListPage } from '@/pages/ModelsListPage.mjs';
import { getAuthServerConfig, getTestCredentials } from '@/utils/auth-server-client.mjs';
import { expect, test } from '@/fixtures.mjs';

// Prefix feature tests — all using OpenAI (prefix behavior is provider-agnostic).
// Each test is self-contained: creates and deletes its own models.

const OPENAI_BASE_URL = 'https://api.openai.com/v1';

test.describe('API Models Prefix Functionality', () => {
  let loginPage;
  let modelsPage;
  let formPage;
  let chatPage;
  let apiKey;
  let authServerConfig;
  let testCredentials;

  test.beforeAll(() => {
    const { apiKey: key } = ApiModelFixtures.getRequiredEnvVars();
    apiKey = key;
    authServerConfig = getAuthServerConfig();
    testCredentials = getTestCredentials();
  });

  test.beforeEach(async ({ page, sharedServerUrl }) => {
    loginPage = new LoginPage(page, sharedServerUrl, authServerConfig, testCredentials);
    modelsPage = new ModelsListPage(page, sharedServerUrl);
    formPage = new ApiModelFormPage(page, sharedServerUrl);
    chatPage = new ChatPage(page, sharedServerUrl);
  });

  test('create model without prefix: appears in list and chat', async ({ page }) => {
    await loginPage.performOAuthLogin();
    await modelsPage.navigateToModels();
    await modelsPage.clickNewApiModel();

    await formPage.form.waitForFormReady();
    await formPage.form.fillBasicInfo(apiKey, OPENAI_BASE_URL);
    await formPage.form.fetchAndSelectModels([ApiModelFixtures.OPENAI_MODEL]);
    await formPage.form.testConnection();
    const modelId = await formPage.createModelAndCaptureId();

    await modelsPage.verifyApiModelInList(modelId, 'openai', OPENAI_BASE_URL);
    await modelsPage.clickChatWithModel(ApiModelFixtures.OPENAI_MODEL);
    await chatPage.expectChatPageWithModel(ApiModelFixtures.OPENAI_MODEL);

    await modelsPage.navigateToModels();
    await modelsPage.deleteModel(modelId);
  });

  test('create model with prefix: prefixed model visible in list and chat', async ({ page }) => {
    await loginPage.performOAuthLogin();
    await modelsPage.navigateToModels();
    await modelsPage.clickNewApiModel();

    await formPage.form.waitForFormReady();
    await formPage.form.fillBasicInfoWithPrefix(apiKey, 'test/', OPENAI_BASE_URL);
    await formPage.form.fetchAndSelectModels([ApiModelFixtures.OPENAI_MODEL]);
    await formPage.form.testConnection();
    const modelId = await formPage.createModelAndCaptureId();

    const prefixedModel = `test/${ApiModelFixtures.OPENAI_MODEL}`;
    await modelsPage.verifyApiModelInList(modelId, 'openai', OPENAI_BASE_URL);
    await modelsPage.clickChatWithModel(prefixedModel);
    await chatPage.expectChatPageWithModel(prefixedModel);

    await modelsPage.navigateToModels();
    await modelsPage.deleteModel(modelId);
  });

  test('edit model to add prefix: chat model name updates', async ({ page }) => {
    await loginPage.performOAuthLogin();
    await modelsPage.navigateToModels();
    await modelsPage.clickNewApiModel();

    // Create without prefix
    await formPage.form.waitForFormReady();
    await formPage.form.fillBasicInfo(apiKey, OPENAI_BASE_URL);
    await formPage.form.fetchAndSelectModels([ApiModelFixtures.OPENAI_MODEL]);
    await formPage.form.testConnection();
    const modelId = await formPage.createModelAndCaptureId();

    // Edit to add prefix
    await modelsPage.editModel(modelId);
    await formPage.form.waitForFormReady();
    await formPage.form.verifyFormPreFilledWithPrefix('openai', OPENAI_BASE_URL, null);
    await formPage.form.setPrefix('openai:');
    await formPage.updateModel();

    // Chat now uses prefixed model name
    const prefixedModel = `openai:${ApiModelFixtures.OPENAI_MODEL}`;
    await modelsPage.clickChatWithModel(prefixedModel);
    await chatPage.expectChatPageWithModel(prefixedModel);

    await modelsPage.navigateToModels();
    await modelsPage.deleteModel(modelId);
  });

  test('edit model to remove prefix: chat reverts to bare model name', async ({ page }) => {
    await loginPage.performOAuthLogin();
    await modelsPage.navigateToModels();
    await modelsPage.clickNewApiModel();

    // Create with prefix
    await formPage.form.waitForFormReady();
    await formPage.form.fillBasicInfoWithPrefix(apiKey, 'myprefix/', OPENAI_BASE_URL);
    await formPage.form.fetchAndSelectModels([ApiModelFixtures.OPENAI_MODEL]);
    await formPage.form.testConnection();
    const modelId = await formPage.createModelAndCaptureId();

    // Edit to remove prefix
    await modelsPage.editModel(modelId);
    await formPage.form.waitForFormReady();
    await formPage.form.disablePrefix();
    await formPage.updateModel();

    // Model appears without prefix in chat
    await modelsPage.clickChatWithModel(ApiModelFixtures.OPENAI_MODEL);
    await chatPage.expectChatPageWithModel(ApiModelFixtures.OPENAI_MODEL);

    await modelsPage.navigateToModels();
    await modelsPage.deleteModel(modelId);
  });

  test('prefix UI behavior: checkbox enable/disable and input state', async ({ page }) => {
    await loginPage.performOAuthLogin();
    await modelsPage.navigateToModels();
    await modelsPage.clickNewApiModel();
    await formPage.form.waitForFormReady();

    // Initially: unchecked, input disabled
    await expect(page.locator(formPage.form.selectors.usePrefixCheckbox)).not.toBeChecked();
    await expect(page.locator(formPage.form.selectors.prefixInput)).toBeDisabled();

    // Enable prefix: input becomes visible
    await formPage.form.enablePrefix();
    await expect(page.locator(formPage.form.selectors.usePrefixCheckbox)).toBeChecked();
    await expect(page.locator(formPage.form.selectors.prefixInput)).toBeVisible();

    // Accepts valid prefix characters
    await page.fill(formPage.form.selectors.prefixInput, 'valid-prefix_123');

    // Disable prefix: input becomes disabled again
    await formPage.form.disablePrefix();
    await expect(page.locator(formPage.form.selectors.usePrefixCheckbox)).not.toBeChecked();
    await expect(page.locator(formPage.form.selectors.prefixInput)).toBeDisabled();
  });

  test('prefix persistence across edit sessions', async ({ page }) => {
    await loginPage.performOAuthLogin();
    await modelsPage.navigateToModels();
    await modelsPage.clickNewApiModel();

    await formPage.form.waitForFormReady();
    await formPage.form.fillBasicInfoWithPrefix(apiKey, 'persist/', OPENAI_BASE_URL);
    await formPage.form.fetchAndSelectModels([ApiModelFixtures.OPENAI_MODEL]);
    const modelId = await formPage.createModelAndCaptureId();

    // Edit 1: verify prefix persists, change it
    await modelsPage.editModel(modelId);
    await formPage.form.waitForFormReady();
    await formPage.form.verifyFormPreFilledWithPrefix('openai', OPENAI_BASE_URL, 'persist/');
    await formPage.form.setPrefix('updated/');
    await formPage.updateModel();

    // Edit 2: verify updated prefix persists
    await modelsPage.editModel(modelId);
    await formPage.form.waitForFormReady();
    await formPage.form.verifyFormPreFilledWithPrefix('openai', OPENAI_BASE_URL, 'updated/');

    await modelsPage.navigateToModels();
    await modelsPage.deleteModel(modelId);
  });

  test('empty prefix acts like no prefix', async ({ page }) => {
    await loginPage.performOAuthLogin();
    await modelsPage.navigateToModels();
    await modelsPage.clickNewApiModel();

    await formPage.form.waitForFormReady();
    await formPage.form.fillBasicInfo(apiKey, OPENAI_BASE_URL);
    await formPage.form.enablePrefix();
    await page.fill(formPage.form.selectors.prefixInput, '');
    await formPage.form.fetchAndSelectModels([ApiModelFixtures.OPENAI_MODEL]);
    const modelId = await formPage.createModelAndCaptureId();

    await modelsPage.verifyApiModelInList(modelId, 'openai', OPENAI_BASE_URL);
    // Model appears with bare name (no prefix)
    await modelsPage.clickChatWithModel(ApiModelFixtures.OPENAI_MODEL);
    await chatPage.expectChatPageWithModel(ApiModelFixtures.OPENAI_MODEL);

    await modelsPage.navigateToModels();
    await modelsPage.deleteModel(modelId);
  });

  test('base URL with trailing slash is normalized', async ({ page }) => {
    await loginPage.performOAuthLogin();
    await modelsPage.navigateToModels();
    await modelsPage.clickNewApiModel();

    await formPage.form.waitForFormReady();
    // URL with trailing slash
    await formPage.form.fillBasicInfoWithPrefix(apiKey, 'norm/', `${OPENAI_BASE_URL}/`);
    await formPage.form.fetchAndSelectModels([ApiModelFixtures.OPENAI_MODEL]);
    const modelId = await formPage.createModelAndCaptureId();

    // Stored URL should have trailing slash stripped
    await modelsPage.verifyApiModelInList(modelId, 'openai', OPENAI_BASE_URL);

    await modelsPage.navigateToModels();
    await modelsPage.deleteModel(modelId);
  });
});
