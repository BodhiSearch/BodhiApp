import { LocalModelFixtures, QWEN_MODEL } from '@/fixtures/localModelFixtures.mjs';
import { ChatPage } from '@/pages/ChatPage.mjs';
import { ChatSettingsPage } from '@/pages/ChatSettingsPage.mjs';
import { LocalModelFormPage } from '@/pages/LocalModelFormPage.mjs';
import { LoginPage } from '@/pages/LoginPage.mjs';
import { ModelsListPage } from '@/pages/ModelsListPage.mjs';
import { getAuthServerConfig, getTestCredentials } from '@/utils/auth-server-client.mjs';
import { expect, test } from '@/fixtures.mjs';

/**
 * Local GGUF Model Smoke Tests (standalone-only)
 *
 * Verifies local GGUF model discovery and chat functionality.
 * Multi-tenant has no filesystem model discovery, so these tests
 * are excluded via testIgnore in playwright.config.mjs.
 */

test.describe('Local GGUF Models - Standalone Smoke Test', () => {
  let authServerConfig;
  let testCredentials;
  let loginPage;
  let modelsPage;
  let formPage;
  let chatPage;
  let chatSettingsPage;

  test.beforeAll(async () => {
    authServerConfig = getAuthServerConfig();
    testCredentials = getTestCredentials();
  });

  test.beforeEach(async ({ page, sharedServerUrl }) => {
    loginPage = new LoginPage(page, sharedServerUrl, authServerConfig, testCredentials);
    modelsPage = new ModelsListPage(page, sharedServerUrl);
    formPage = new LocalModelFormPage(page, sharedServerUrl);
    chatPage = new ChatPage(page, sharedServerUrl);
    chatSettingsPage = new ChatSettingsPage(page, sharedServerUrl);
  });

  test('GGUF model chat and local alias creation @smoke @integration', async ({ page }) => {
    await loginPage.performOAuthLogin();

    await test.step('Chat with pre-discovered GGUF model', async () => {
      await chatPage.navigateToChat();
      await chatPage.verifyChatEmpty();
      await chatSettingsPage.selectModelQwen();

      await chatPage.sendMessage('What is 2+2?');
      await chatPage.waitForResponseComplete();
      await chatPage.waitForResponse('4');
      await chatPage.verifyMessageInHistory('user', 'What is 2+2?');
      await chatPage.verifyMessageInHistory('assistant', '4');
    });

    await test.step('Create local model alias', async () => {
      const aliasName = `local-smoke-${Date.now()}`;

      await modelsPage.navigateToModels();
      await modelsPage.clickNewModelAlias();
      await formPage.waitForFormReady();
      await formPage.fillBasicInfo(aliasName, QWEN_MODEL.repo, QWEN_MODEL.filename);
      await formPage.createAlias();

      await modelsPage.verifyLocalModelInList(
        aliasName,
        QWEN_MODEL.repo,
        QWEN_MODEL.filename,
        'user'
      );
    });
  });
});
