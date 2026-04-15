import { ApiModelFixtures } from '@/fixtures/apiModelFixtures.mjs';
import { ApiModelFormPage } from '@/pages/ApiModelFormPage.mjs';
import { ChatPage } from '@/pages/ChatPage.mjs';
import { LoginPage } from '@/pages/LoginPage.mjs';
import { ModelsListPage } from '@/pages/ModelsListPage.mjs';
import { getAuthServerConfig, getTestCredentials } from '@/utils/auth-server-client.mjs';
import { expect, test } from '@/fixtures.mjs';

// Live E2E test: create a Gemini alias via the UI, then chat with it.
// Verifies the full pi-ai → google-generative-ai → /v1beta/* proxy chain including
// SENTINEL_API_KEY stripping by gemini_auth_middleware.
//
// Requires INTEG_TEST_GEMINI_API_KEY in .env.test.

const GEMINI_FORMAT = ApiModelFixtures.API_FORMATS.gemini;

test.describe('Chat UI - Gemini format', () => {
  let authServerConfig;
  let testCredentials;
  let geminiApiKey;

  test.beforeAll(async () => {
    authServerConfig = getAuthServerConfig();
    testCredentials = getTestCredentials();
    geminiApiKey = process.env[GEMINI_FORMAT.envKey];
    if (!geminiApiKey) {
      throw new Error(
        `${GEMINI_FORMAT.envKey} missing in .env.test — required for chat-gemini spec`
      );
    }
  });

  let loginPage;
  let modelsPage;
  let formPage;
  let chatPage;

  test.beforeEach(async ({ page, sharedServerUrl }) => {
    loginPage = new LoginPage(page, sharedServerUrl, authServerConfig, testCredentials);
    modelsPage = new ModelsListPage(page, sharedServerUrl);
    formPage = new ApiModelFormPage(page, sharedServerUrl);
    chatPage = new ChatPage(page, sharedServerUrl);
  });

  test('create Gemini alias via UI and chat with it', async ({ page }) => {
    await loginPage.performOAuthLogin();
    await modelsPage.navigateToModels();
    await modelsPage.clickNewApiModel();

    await formPage.form.waitForFormReady();
    await formPage.form.selectApiFormat(GEMINI_FORMAT.format);
    await formPage.form.fillBasicInfo(geminiApiKey, GEMINI_FORMAT.baseUrl);
    await formPage.form.fetchAndSelectModels([GEMINI_FORMAT.model]);
    await formPage.form.testConnection();
    const modelId = await formPage.createModelAndCaptureId();

    try {
      await chatPage.navigateToChat();
      await chatPage.selectModel(GEMINI_FORMAT.model);
      await chatPage.sendMessage(GEMINI_FORMAT.chatQuestion);
      await chatPage.waitForResponseComplete();

      const reply = await chatPage.getLastAssistantMessage();
      expect(reply.toLowerCase()).toContain(GEMINI_FORMAT.chatExpected);
    } finally {
      await modelsPage.navigateToModels();
      await modelsPage.deleteModel(modelId);
    }
  });
});
