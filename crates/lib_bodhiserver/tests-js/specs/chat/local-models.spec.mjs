import { LocalModelFixtures, QWEN_MODEL } from '@/fixtures/localModelFixtures.mjs';
import { ChatPage } from '@/pages/ChatPage.mjs';
import { ChatSettingsPage } from '@/pages/ChatSettingsPage.mjs';
import { LocalModelFormPage } from '@/pages/LocalModelFormPage.mjs';
import { LoginPage } from '@/pages/LoginPage.mjs';
import { ModelsListPageV2 } from '@/pages/ModelsListPageV2.mjs';
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
    modelsPage = new ModelsListPageV2(page, sharedServerUrl);
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

    await test.step('Create local model alias via the catalog quant selector', async () => {
      const aliasName = `local-smoke-${Date.now()}`;

      await modelsPage.navigateToModels();
      await modelsPage.clickNewModelAlias();
      await formPage.waitForFormReady();
      // Type the repo, then pick the GGUF from the reference-catalog quant table (downloaded already).
      await formPage.fillBasicInfo(aliasName, QWEN_MODEL.repo, QWEN_MODEL.filename);
      // Click-to-add a runtime flag from the catalog (lands in the context-params textarea).
      await formPage.addContextFlag('--flash-attn');
      await expect(formPage.page.locator(formPage.selectors.contextParamsTextarea)).toHaveValue(/--flash-attn/);
      await formPage.createAlias();

      await modelsPage.verifyLocalModelInList(aliasName, QWEN_MODEL.repo, QWEN_MODEL.filename, 'user');
    });

    await test.step('Create alias for a not-yet-downloaded quant kicks off a download', async () => {
      const aliasName = `local-undl-${Date.now()}`;

      await modelsPage.navigateToModels();
      await modelsPage.clickNewModelAlias();
      await formPage.waitForFormReady();
      await formPage.fillTestId('alias-input', aliasName);
      await formPage.page.fill(formPage.selectors.repoInput, QWEN_MODEL.repo);

      // Pick a quant that isn't on disk: its row shows "Not downloaded" + the download-on-save note.
      const quantName = await formPage.selectFirstRemoteQuant();
      if (quantName) {
        await formPage.page.locator(formPage.selectors.quantDownloadNote).waitFor();
        await formPage.createAlias();
        // Backend creates the alias and enqueues the file; under test-mode the download lands
        // completed without a real fetch. The alias appears in the list.
        await modelsPage.expectModelInList(aliasName);
      }
    });
  });
});
