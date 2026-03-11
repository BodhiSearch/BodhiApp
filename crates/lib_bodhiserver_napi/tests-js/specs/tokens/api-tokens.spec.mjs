import { ApiModelFixtures } from '@/fixtures/apiModelFixtures.mjs';
import { TokenFixtures } from '@/fixtures/tokenFixtures.mjs';
import { ApiModelFormPage } from '@/pages/ApiModelFormPage.mjs';
import { ChatPage } from '@/pages/ChatPage.mjs';
import { ChatSettingsPage } from '@/pages/ChatSettingsPage.mjs';
import { LoginPage } from '@/pages/LoginPage.mjs';
import { ModelsListPage } from '@/pages/ModelsListPage.mjs';
import { TokensPage } from '@/pages/TokensPage.mjs';
import { registerApiModelViaUI } from '@/utils/api-model-helpers.mjs';
import { getAuthServerConfig, getTestCredentials } from '@/utils/auth-server-client.mjs';
import { expect, test } from '@/fixtures.mjs';
import { SHARED_STATIC_SERVER_URL } from '@/test-helpers.mjs';

test.describe('API Tokens - Complete Integration', () => {
  let authServerConfig;
  let testCredentials;
  let testApiKey;

  test.beforeAll(async () => {
    authServerConfig = getAuthServerConfig();
    testCredentials = getTestCredentials();
    testApiKey = ApiModelFixtures.getRequiredEnvVars().apiKey;
  });

  test('Token Lifecycle, Scopes, and Chat @integration', async ({ page, sharedServerUrl }) => {
    const loginPage = new LoginPage(page, sharedServerUrl, authServerConfig, testCredentials);
    const modelsPage = new ModelsListPage(page, sharedServerUrl);
    const apiModelFormPage = new ApiModelFormPage(page, sharedServerUrl);
    const tokensPage = new TokensPage(page, sharedServerUrl);
    const chatPage = new ChatPage(page, sharedServerUrl);
    const chatSettings = new ChatSettingsPage(page, sharedServerUrl);

    const tokenNames = TokenFixtures.getTestTokenNames();

    await test.step('Setup: Login and register API model', async () => {
      await loginPage.performOAuthLogin();
      await registerApiModelViaUI(modelsPage, apiModelFormPage, testApiKey);
    });

    let copiedToken;
    await test.step('Token CRUD: create, visibility, copy, list', async () => {
      await tokensPage.navigateToTokens();
      await tokensPage.expectTokensPage();

      await tokensPage.createToken(tokenNames.chat);
      await tokensPage.expectTokenDialog();

      // Toggle show/hide
      await tokensPage.expectTokenHidden();
      await tokensPage.toggleShowToken();
      const tokenValue = await tokensPage.getTokenValue();
      expect(tokenValue).toMatch(/^bodhiapp_/);
      await tokensPage.expectTokenVisible(tokenValue);
      await tokensPage.toggleShowToken();
      await tokensPage.expectTokenHidden();

      // Copy token
      const clipboard = await TokenFixtures.mockClipboard(page);
      copiedToken = await tokensPage.copyTokenFromDialog();
      const clipboardContent = await clipboard.getContent();
      expect(clipboardContent).toBe(copiedToken);
      expect(copiedToken).toBe(tokenValue);

      // Close and verify in list
      await tokensPage.closeTokenDialog();
      await tokensPage.expectDialogClosed();
      await tokensPage.expectTokenInList(tokenNames.chat, 'active');
      await tokensPage.expectTokenName(tokenNames.chat);
      await tokensPage.expectTokenStatus(tokenNames.chat, 'active');
    });

    await test.step('Token Scopes: create tokens with different scopes', async () => {
      // Create token with User scope
      await tokensPage.createToken(tokenNames.basic, 'scope_token_user');
      await tokensPage.expectTokenDialog();
      await tokensPage.copyTokenFromDialog();
      await tokensPage.closeTokenDialog();
      await tokensPage.expectDialogClosed();

      await tokensPage.waitForTokenCreationSuccess();
      const userTokenData = await tokensPage.findTokenByName(tokenNames.basic);
      expect(userTokenData).not.toBeNull();
      expect(userTokenData.scope).toBe('scope_token_user');
      expect(userTokenData.status).toBe('active');

      // Create token with PowerUser scope
      await tokensPage.createToken(tokenNames.admin1, 'scope_token_power_user');
      await tokensPage.expectTokenDialog();
      await tokensPage.copyTokenFromDialog();
      await tokensPage.closeTokenDialog();
      await tokensPage.expectDialogClosed();

      await tokensPage.waitForTokenCreationSuccess();
      const powerUserTokenData = await tokensPage.findTokenByName(tokenNames.admin1);
      expect(powerUserTokenData).not.toBeNull();
      expect(powerUserTokenData.scope).toBe('scope_token_power_user');
      expect(powerUserTokenData.status).toBe('active');

      const tokenCount = await tokensPage.getTokenCount();
      expect(tokenCount).toBeGreaterThanOrEqual(3);
    });

    await test.step('Chat Integration: token auth, deactivate, reactivate', async () => {
      await chatPage.navigateToChat();
      await chatPage.waitForChatPageLoad();
      await chatPage.verifyChatEmpty();

      // Model selection BEFORE API token (known UI quirk)
      await chatSettings.selectModel(ApiModelFixtures.OPENAI_MODEL);
      await chatSettings.setApiToken(true, copiedToken);
      await chatSettings.verifyApiTokenSettings(true, true);

      await chatPage.sendMessage('What is 2+2?');
      await chatPage.waitForResponseComplete();
      await chatPage.waitForResponse('4');
      await chatPage.verifyMessageInHistory('user', 'What is 2+2?');
      await chatPage.verifyMessageInHistory('assistant', '4');

      // Deactivate token and verify error
      await tokensPage.navigateToTokens();
      await tokensPage.expectTokensPage();
      await tokensPage.toggleTokenStatus(tokenNames.chat);
      await tokensPage.waitForTokenUpdateSuccess('inactive');
      await tokensPage.expectTokenStatus(tokenNames.chat, 'inactive');

      await chatPage.navigateToChat();
      await chatPage.waitForChatPageLoad();
      await chatPage.sendMessageAndReturn('This should fail');
      await chatPage.expectError();

      // Reactivate token and verify recovery
      await tokensPage.navigateToTokens();
      await tokensPage.toggleTokenStatus(tokenNames.chat);
      await tokensPage.expectTokenStatus(tokenNames.chat, 'active');

      await chatPage.navigateToChat();
      await chatPage.waitForChatPageLoad();
      await chatPage.sendMessage('What is 4+4?');
      await chatPage.waitForResponseComplete();
      await chatPage.waitForResponse('8');
    });
  });

  test('Multi-User Isolation and Error Recovery @integration', async ({
    browser,
    sharedServerUrl,
  }, testInfo) => {
    test.skip(
      testInfo.project.name === 'multi_tenant',
      'Multi-user test requires same-tenant membership setup'
    );

    let adminContext;
    let managerContext;

    try {
      const tokenNames = TokenFixtures.getTestTokenNames();
      const invalidTokens = TokenFixtures.getInvalidTokens();
      let adminToken1;

      await test.step('Admin: login, register model, create tokens', async () => {
        adminContext = await browser.newContext();
        const adminPage = await adminContext.newPage();
        const adminLogin = new LoginPage(
          adminPage,
          sharedServerUrl,
          authServerConfig,
          testCredentials
        );
        const adminModelsPage = new ModelsListPage(adminPage, sharedServerUrl);
        const adminFormPage = new ApiModelFormPage(adminPage, sharedServerUrl);
        const adminTokensPage = new TokensPage(adminPage, sharedServerUrl);

        await adminLogin.performOAuthLogin();
        await registerApiModelViaUI(adminModelsPage, adminFormPage, testApiKey);

        await adminTokensPage.navigateToTokens();
        await TokenFixtures.mockClipboard(adminPage);

        await adminTokensPage.createToken(tokenNames.admin1);
        adminToken1 = await adminTokensPage.copyTokenFromDialog();
        await adminTokensPage.closeTokenDialog();
        await adminTokensPage.expectTokenInList(tokenNames.admin1);

        await adminTokensPage.createToken(tokenNames.admin2);
        await adminTokensPage.closeTokenDialog();
        await adminTokensPage.expectTokenInList(tokenNames.admin2);

        const adminTokenCount = await adminTokensPage.getTokenCount();
        expect(adminTokenCount).toBeGreaterThanOrEqual(2);
      });

      let managerToken;
      await test.step('Manager: login, register model, verify isolation', async () => {
        const managerCredentials = {
          username: process.env.INTEG_TEST_USER_MANAGER,
          userId: process.env.INTEG_TEST_USER_MANAGER_ID,
          password: process.env.INTEG_TEST_PASSWORD,
        };

        managerContext = await browser.newContext();
        const managerPage = await managerContext.newPage();
        const managerLogin = new LoginPage(
          managerPage,
          sharedServerUrl,
          authServerConfig,
          managerCredentials
        );
        const managerModelsPage = new ModelsListPage(managerPage, sharedServerUrl);
        const managerFormPage = new ApiModelFormPage(managerPage, sharedServerUrl);
        const managerTokensPage = new TokensPage(managerPage, sharedServerUrl);

        await managerLogin.performOAuthLogin();
        await registerApiModelViaUI(managerModelsPage, managerFormPage, testApiKey);

        await managerTokensPage.navigateToTokens();
        await managerTokensPage.expectEmptyTokensList();

        await TokenFixtures.mockClipboard(managerPage);
        await managerTokensPage.createToken(tokenNames.user);
        managerToken = await managerTokensPage.copyTokenFromDialog();
        await managerTokensPage.closeTokenDialog();
        await managerTokensPage.expectTokenInList(tokenNames.user);

        const managerTokenCount = await managerTokensPage.getTokenCount();
        expect(managerTokenCount).toBe(1);
      });

      await test.step('Cross-user token and model isolation', async () => {
        const managerPage = managerContext.pages()[0];
        const managerChatPage = new ChatPage(managerPage, sharedServerUrl);
        const managerChatSettings = new ChatSettingsPage(managerPage, sharedServerUrl);
        const managerTokensPage = new TokensPage(managerPage, sharedServerUrl);

        // Manager: chat with own model + own token
        await managerChatPage.navigateToChat();
        await managerChatPage.waitForChatPageLoad();
        await managerChatSettings.selectModel(ApiModelFixtures.OPENAI_MODEL);
        await managerChatSettings.setApiToken(true, managerToken);
        await managerChatPage.sendMessage('Hello');
        await managerChatPage.waitForResponseComplete();

        // Manager: use admin's token (API token auth bypasses session)
        await managerChatSettings.setApiToken(true, adminToken1);
        await managerChatPage.sendMessage('What is 2+2?');
        await managerChatPage.waitForResponseComplete();
        await managerChatPage.waitForResponse('4');

        // Admin: deactivate their token
        const adminPage = adminContext.pages()[0];
        const adminTokensPage = new TokensPage(adminPage, sharedServerUrl);
        await adminTokensPage.navigateToTokens();
        await adminTokensPage.toggleTokenStatus(tokenNames.admin1);
        await adminTokensPage.waitForTokenUpdateSuccess('inactive');
        await adminTokensPage.expectTokenStatus(tokenNames.admin1, 'inactive');

        // Manager: deactivated admin token should error
        await managerChatSettings.setApiToken(true, adminToken1);
        await managerChatPage.sendMessageAndReturn('This should fail');
        await managerChatPage.expectError();

        // Manager: own token still works
        await managerChatSettings.setApiToken(true, managerToken);
        await managerChatPage.sendMessage('What is 3+3?');
        await managerChatPage.waitForResponseComplete();

        // Manager: verify only own tokens visible
        await managerTokensPage.navigateToTokens();
        await managerTokensPage.expectTokensPage();
        const finalManagerTokenCount = await managerTokensPage.getTokenCount();
        expect(finalManagerTokenCount).toBe(1);
        await managerTokensPage.expectTokenInList(tokenNames.user);
        await managerTokensPage.expectTokenNotInList(tokenNames.admin1);
        await managerTokensPage.expectTokenNotInList(tokenNames.admin2);
      });

      await test.step('Error handling and recovery (admin context)', async () => {
        const adminPage = adminContext.pages()[0];
        const adminChatPage = new ChatPage(adminPage, sharedServerUrl);
        const adminChatSettings = new ChatSettingsPage(adminPage, sharedServerUrl);
        const adminTokensPage = new TokensPage(adminPage, sharedServerUrl);

        // Empty token error
        await adminChatPage.navigateToChat();
        await adminChatPage.waitForChatPageLoad();
        await adminChatSettings.openSettings();
        await adminChatSettings.setApiToken(true, '');
        await adminChatSettings.selectModel(ApiModelFixtures.OPENAI_MODEL);
        await adminChatPage.sendMessageAndReturn('Test message');
        await adminChatPage.expectError();

        // Reset and test invalid token format
        await adminTokensPage.navigateToTokens();
        await adminChatPage.navigateToChat();
        await adminChatPage.waitForChatPageLoad();
        await adminChatSettings.setApiToken(true, invalidTokens.invalidFormat);
        await adminChatSettings.selectModel(ApiModelFixtures.OPENAI_MODEL);
        await adminChatPage.sendMessageAndReturn('Another test');
        await adminChatPage.expectError();

        // Reset and test non-existent token
        await adminTokensPage.navigateToTokens();
        await adminChatPage.navigateToChat();
        await adminChatPage.waitForChatPageLoad();
        await adminChatSettings.setApiToken(true, invalidTokens.nonExistent);
        await adminChatSettings.selectModel(ApiModelFixtures.OPENAI_MODEL);
        await adminChatPage.sendMessageAndReturn('Yet another test');
        await adminChatPage.expectError();

        // Create valid token and verify recovery
        await adminTokensPage.navigateToTokens();
        await adminTokensPage.expectTokensPage();
        await adminTokensPage.createToken(tokenNames.basic);
        const validToken = await adminTokensPage.copyTokenFromDialog();
        await adminTokensPage.closeTokenDialog();
        await adminTokensPage.expectTokenInList(tokenNames.basic, 'active');

        await adminChatPage.navigateToChat();
        await adminChatPage.waitForChatPageLoad();
        await adminChatSettings.setApiToken(true, validToken);
        await adminChatSettings.selectModel(ApiModelFixtures.OPENAI_MODEL);
        await adminChatPage.sendMessage('What is 3+3?');
        await adminChatPage.waitForResponseComplete();
        await adminChatPage.waitForResponse('6');

        // Network failure and recovery
        await adminChatPage.simulateNetworkFailure();
        await adminChatPage.sendMessageAndReturn('Network test');
        await adminChatPage.expectError();

        await adminChatPage.restoreNetworkConnection();
        await adminChatPage.sendMessage('What is 4+4?');
        await adminChatPage.waitForResponseComplete();
        await adminChatPage.waitForResponse('8');

        await adminChatPage.verifyMessageInHistory('user', 'What is 3+3?');
        await adminChatPage.verifyMessageInHistory('assistant', '6');
        await adminChatPage.verifyMessageInHistory('user', 'What is 4+4?');
        await adminChatPage.verifyMessageInHistory('assistant', '8');
      });
    } finally {
      if (managerContext) await managerContext.close();
      if (adminContext) await adminContext.close();
    }
  });
});
