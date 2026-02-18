import { TokenFixtures } from '@/fixtures/tokenFixtures.mjs';
import { ChatPage } from '@/pages/ChatPage.mjs';
import { ChatSettingsPage } from '@/pages/ChatSettingsPage.mjs';
import { LoginPage } from '@/pages/LoginPage.mjs';
import { TokensPage } from '@/pages/TokensPage.mjs';
import { getAuthServerConfig, getTestCredentials } from '@/utils/auth-server-client.mjs';
import { expect, test } from '@/fixtures.mjs';
import { SHARED_SERVER_URL, SHARED_STATIC_SERVER_URL } from '@/test-helpers.mjs';

test.describe('API Tokens - Complete Integration', () => {
  let authServerConfig;
  let testCredentials;

  test.beforeAll(async () => {
    authServerConfig = getAuthServerConfig();
    testCredentials = getTestCredentials();

    // Use shared server started by Playwright webServer
  });

  test('Complete Token Lifecycle and Chat Integration @integration', async ({ page }) => {
    // Initialize page objects
    const loginPage = new LoginPage(page, SHARED_SERVER_URL, authServerConfig, testCredentials);
    const tokensPage = new TokensPage(page, SHARED_SERVER_URL);
    const chatPage = new ChatPage(page, SHARED_SERVER_URL);
    const chatSettings = new ChatSettingsPage(page, SHARED_SERVER_URL);

    const tokenNames = TokenFixtures.getTestTokenNames();

    // Step 1-2: Login and navigate to tokens page
    await loginPage.performOAuthLogin();
    await tokensPage.navigateToTokens();
    await tokensPage.expectTokensPage();

    // Step 3: Create token with name
    await tokensPage.createToken(tokenNames.chat);

    // Step 4: Verify token dialog appears
    await tokensPage.expectTokenDialog();

    // Step 5: Toggle show/hide token visibility
    // Initially hidden
    await tokensPage.expectTokenHidden();

    // Show token
    await tokensPage.toggleShowToken();
    const tokenValue = await tokensPage.getTokenValue();
    expect(tokenValue).toMatch(/^bodhiapp_/);
    await tokensPage.expectTokenVisible(tokenValue);

    // Hide token again
    await tokensPage.toggleShowToken();
    await tokensPage.expectTokenHidden();

    // Step 6: Copy token to clipboard
    const clipboard = await TokenFixtures.mockClipboard(page);
    const copiedToken = await tokensPage.copyTokenFromDialog();
    const clipboardContent = await clipboard.getContent();
    expect(clipboardContent).toBe(copiedToken);
    expect(copiedToken).toBe(tokenValue);

    // Step 7: Close dialog with Done button
    await tokensPage.closeTokenDialog();
    await tokensPage.expectDialogClosed();

    // Step 8: Verify token appears in list with correct name and active status
    await tokensPage.expectTokenInList(tokenNames.chat, 'active');
    await tokensPage.expectTokenName(tokenNames.chat);
    await tokensPage.expectTokenStatus(tokenNames.chat, 'active');

    // Step 9: Navigate to chat page
    await chatPage.navigateToChat();
    await chatPage.waitForChatPageLoad();
    await chatPage.verifyChatEmpty();

    // Step 10-11: Open chat settings and enable API token authentication
    await chatSettings.openSettings();
    await chatSettings.setApiToken(true, copiedToken);
    await chatSettings.verifyApiTokenSettings(true, true);

    // Step 12: Select a model
    await chatSettings.selectModelQwen();

    // Step 13-14: Send a test message and verify successful response
    await chatPage.sendMessage('What is 2+2?');
    await chatPage.waitForResponseComplete();
    await chatPage.waitForResponse('4');
    await chatPage.verifyMessageInHistory('user', 'What is 2+2?');
    await chatPage.verifyMessageInHistory('assistant', '4');

    // Step 15-16: Return to tokens page and toggle token status to inactive
    await tokensPage.navigateToTokens();
    await tokensPage.expectTokensPage();
    await tokensPage.toggleTokenStatus(tokenNames.chat);
    await tokensPage.waitForTokenUpdateSuccess('inactive');
    await tokensPage.expectTokenStatus(tokenNames.chat, 'inactive');

    // Step 17-18: Go back to chat and attempt to send message
    await chatPage.navigateToChat();
    await chatPage.waitForChatPageLoad();

    // Step 19: Send message with inactive token
    await chatPage.sendMessageAndReturn('This should fail');

    // Step 20: Verify authentication error
    await chatPage.expectError();

    // Cleanup: Re-activate token for clean state
    await tokensPage.navigateToTokens();
    await tokensPage.toggleTokenStatus(tokenNames.chat);
    await tokensPage.expectTokenStatus(tokenNames.chat, 'active');
    // Step 21: Go back to chat and attempt to send message
    await chatPage.navigateToChat();
    await chatPage.waitForChatPageLoad();
    await chatPage.sendMessage('What is 4+4?');
    await chatPage.waitForResponseComplete();
    await chatPage.waitForResponse('8');
  });

  test('Multi-User Token Management and Isolation @integration', async ({ browser }) => {
    let adminContext;
    let userContext;

    adminContext = await browser.newContext();
    const adminPage = await adminContext.newPage();
    const adminLogin = new LoginPage(
      adminPage,
      SHARED_SERVER_URL,
      authServerConfig,
      testCredentials
    );
    const adminTokensPage = new TokensPage(adminPage, SHARED_SERVER_URL);

    const tokenNames = TokenFixtures.getTestTokenNames();

    // Step 1-3: Admin creates 2 tokens
    await adminLogin.performOAuthLogin();
    await adminTokensPage.navigateToTokens();

    await TokenFixtures.mockClipboard(adminPage);
    await adminTokensPage.createToken(tokenNames.admin1);
    const adminToken1 = await adminTokensPage.copyTokenFromDialog();
    await adminTokensPage.closeTokenDialog();
    await adminTokensPage.expectTokenInList(tokenNames.admin1);

    await adminTokensPage.createToken(tokenNames.admin2);
    await adminTokensPage.closeTokenDialog();
    await adminTokensPage.expectTokenInList(tokenNames.admin2);

    // Verify admin tokens visible (may include tokens from prior tests in same server)
    const adminTokenCount = await adminTokensPage.getTokenCount();
    expect(adminTokenCount).toBeGreaterThanOrEqual(2);

    // Step 4-6: User login and verify isolation
    const userCredentials = {
      username: process.env.INTEG_TEST_USER_MANAGER,
      userId: process.env.INTEG_TEST_USER_MANAGER_ID,
      password: process.env.INTEG_TEST_PASSWORD,
    };

    userContext = await browser.newContext();
    const userPage = await userContext.newPage();
    const userLogin = new LoginPage(userPage, SHARED_SERVER_URL, authServerConfig, userCredentials);
    const userTokensPage = new TokensPage(userPage, SHARED_SERVER_URL);

    await userLogin.performOAuthLogin();
    await userTokensPage.navigateToTokens();

    // User should see empty list (isolation)
    await userTokensPage.expectEmptyTokensList();

    // Step 7: User creates their own token
    await TokenFixtures.mockClipboard(userPage);
    await userTokensPage.createToken(tokenNames.user);
    const userToken = await userTokensPage.copyTokenFromDialog();
    await userTokensPage.closeTokenDialog();
    await userTokensPage.expectTokenInList(tokenNames.user);

    const userTokenCount = await userTokensPage.getTokenCount();
    expect(userTokenCount).toBe(1);

    // Step 8: User uses their token in chat successfully
    const userChatPage = new ChatPage(userPage, SHARED_SERVER_URL);
    const userChatSettings = new ChatSettingsPage(userPage, SHARED_SERVER_URL);

    await userChatPage.navigateToChat();
    await userChatPage.waitForChatPageLoad();
    await userChatSettings.selectModelQwen();
    await userChatSettings.setApiToken(true, userToken);
    await userChatPage.sendMessage('Hello');
    await userChatPage.waitForResponseComplete();

    // Step 9: API token auth bypasses session — admin's token works in manager's session
    await userChatSettings.setApiToken(true, adminToken1);
    await userChatPage.sendMessage('What is 2+2?');
    await userChatPage.waitForResponseComplete();
    await userChatPage.waitForResponse('4');

    // Step 10: Admin deactivates their token
    await adminTokensPage.navigateToTokens();
    await adminTokensPage.toggleTokenStatus(tokenNames.admin1);
    await adminTokensPage.waitForTokenUpdateSuccess('inactive');
    await adminTokensPage.expectTokenStatus(tokenNames.admin1, 'inactive');

    // Step 11: Manager using deactivated admin token gets error
    await userChatSettings.setApiToken(true, adminToken1);
    await userChatPage.sendMessageAndReturn('This should fail');
    await userChatPage.expectError();

    // Step 12: User refreshes tokens page and still sees only their token
    await userTokensPage.navigateToTokens();
    await userTokensPage.expectTokensPage();
    const finalUserTokenCount = await userTokensPage.getTokenCount();
    expect(finalUserTokenCount).toBe(1);
    await userTokensPage.expectTokenInList(tokenNames.user);
    await userTokensPage.expectTokenNotInList(tokenNames.admin1);
    await userTokensPage.expectTokenNotInList(tokenNames.admin2);

    // Cleanup
    if (userContext) {
      await userContext.close();
    }
    if (adminContext) {
      await adminContext.close();
    }
  });

  test('Token Scope Selection and Display @integration', async ({ page }) => {
    const loginPage = new LoginPage(page, SHARED_SERVER_URL, authServerConfig, testCredentials);
    const tokensPage = new TokensPage(page, SHARED_SERVER_URL);

    const tokenNames = TokenFixtures.getTestTokenNames();

    // Step 1-2: Login and navigate to tokens page
    await loginPage.performOAuthLogin();
    await tokensPage.navigateToTokens();
    await tokensPage.expectTokensPage();

    // Step 3: Create token with User scope
    await tokensPage.createToken(tokenNames.basic, 'scope_token_user');
    await tokensPage.expectTokenDialog();
    await tokensPage.copyTokenFromDialog();
    await tokensPage.closeTokenDialog();
    await tokensPage.expectDialogClosed();

    // Step 4: Verify token appears in list with User scope
    await tokensPage.waitForTokenCreationSuccess();
    const userTokenData = await tokensPage.findTokenByName(tokenNames.basic);
    expect(userTokenData).not.toBeNull();
    expect(userTokenData.scope).toBe('scope_token_user');
    expect(userTokenData.status).toBe('active');

    // Step 5: Create token with PowerUser scope
    await tokensPage.createToken(tokenNames.admin1, 'scope_token_power_user');
    await tokensPage.expectTokenDialog();
    await tokensPage.copyTokenFromDialog();
    await tokensPage.closeTokenDialog();
    await tokensPage.expectDialogClosed();

    // Step 6: Verify token appears in list with PowerUser scope
    await tokensPage.waitForTokenCreationSuccess();
    const powerUserTokenData = await tokensPage.findTokenByName(tokenNames.admin1);
    expect(powerUserTokenData).not.toBeNull();
    expect(powerUserTokenData.scope).toBe('scope_token_power_user');
    expect(powerUserTokenData.status).toBe('active');

    // Step 7: Verify both tokens are displayed in the list with correct scopes
    const tokenCount = await tokensPage.getTokenCount();
    expect(tokenCount).toBeGreaterThanOrEqual(2);
  });

  test('Error Handling and Recovery @integration', async ({ page }) => {
    const loginPage = new LoginPage(page, SHARED_SERVER_URL, authServerConfig, testCredentials);
    const tokensPage = new TokensPage(page, SHARED_SERVER_URL);
    const chatPage = new ChatPage(page, SHARED_SERVER_URL);
    const chatSettings = new ChatSettingsPage(page, SHARED_SERVER_URL);

    const tokenNames = TokenFixtures.getTestTokenNames();
    const invalidTokens = TokenFixtures.getInvalidTokens();
    const errorMessages = TokenFixtures.getErrorMessages();

    // Step 1-2: Login and navigate to chat
    await loginPage.performOAuthLogin();
    await chatPage.navigateToChat();
    await chatPage.waitForChatPageLoad();

    // Step 3-5: Enable API token without entering value and attempt to send message
    await chatSettings.openSettings();
    await chatSettings.setApiToken(true, '');

    // Select model first
    await chatSettings.selectModelQwen();

    // Attempt to send message with no token
    await chatPage.sendMessageAndReturn('Test message');

    // Step 5: Verify error about missing token
    await chatPage.expectError();

    // Reset state by navigating away and back
    await tokensPage.navigateToTokens();
    await chatPage.navigateToChat();
    await chatPage.waitForChatPageLoad();

    // Step 6-8: Enter invalid token format and verify error
    await chatSettings.setApiToken(true, invalidTokens.invalidFormat);
    await chatSettings.selectModelQwen();
    await chatPage.sendMessageAndReturn('Another test');
    await chatPage.expectError();

    // Reset state by navigating away and back
    await tokensPage.navigateToTokens();
    await chatPage.navigateToChat();
    await chatPage.waitForChatPageLoad();

    // Step 9-11: Enter valid format but non-existent token
    await chatSettings.setApiToken(true, invalidTokens.nonExistent);
    await chatSettings.selectModelQwen();
    await chatPage.sendMessageAndReturn('Yet another test');
    await chatPage.expectError();

    // Step 12-13: Navigate to tokens page and create valid token
    await tokensPage.navigateToTokens();
    await tokensPage.expectTokensPage();
    await tokensPage.createToken(tokenNames.basic);
    const validToken = await tokensPage.copyTokenFromDialog();
    await tokensPage.closeTokenDialog();
    await tokensPage.expectTokenInList(tokenNames.basic, 'active');

    // Step 14-16: Return to chat, use valid token, verify successful chat
    await chatPage.navigateToChat();
    await chatPage.waitForChatPageLoad();
    await chatSettings.setApiToken(true, validToken);
    await chatSettings.selectModelQwen();
    await chatPage.sendMessage('What is 3+3?');
    await chatPage.waitForResponseComplete();
    await chatPage.waitForResponse('6');

    // Step 17-19: Simulate network failure and verify network error
    await chatPage.simulateNetworkFailure();
    await chatPage.sendMessageAndReturn('Network test');
    await chatPage.expectError();

    // Step 20-21: Restore network and verify chat works again
    await chatPage.restoreNetworkConnection();
    await chatPage.sendMessage('What is 4+4?');
    await chatPage.waitForResponseComplete();
    await chatPage.waitForResponse('8');

    // Verify message history contains all successful messages
    await chatPage.verifyMessageInHistory('user', 'What is 3+3?');
    await chatPage.verifyMessageInHistory('assistant', '6');
    await chatPage.verifyMessageInHistory('user', 'What is 4+4?');
    await chatPage.verifyMessageInHistory('assistant', '8');
  });
});
