import {
  getAuthServerConfig,
  getTestCredentials,
} from '@/utils/auth-server-client.mjs';
import { expect, test } from '@/fixtures.mjs';
import { SHARED_SERVER_URL, SHARED_STATIC_SERVER_URL } from '@/test-helpers.mjs';

import { ChatFixtures } from '@/fixtures/ChatFixtures.mjs';
import { ApiModelFormPage } from '@/pages/ApiModelFormPage.mjs';
import { ChatHistoryPage } from '@/pages/ChatHistoryPage.mjs';
import { ChatPage } from '@/pages/ChatPage.mjs';
import { ChatSettingsPage } from '@/pages/ChatSettingsPage.mjs';
import { LoginPage } from '@/pages/LoginPage.mjs';
import { ModelsListPage } from '@/pages/ModelsListPage.mjs';

test.describe('Chat Interface - Core Functionality', () => {
  let authServerConfig;
  let testCredentials;
  let loginPage;
  let modelsPage;
  let apiModelFormPage;
  let chatPage;
  let chatHistoryPage;
  let chatSettingsPage;
  let testApiKey;

  test.beforeAll(async () => {
    // Environment setup
    testApiKey = ChatFixtures.getEnvironmentData().getApiKey();
    authServerConfig = getAuthServerConfig();
    testCredentials = getTestCredentials();

    // Use shared server
    // Note: DB reset will be addressed in PR3 when we solve the dev routes availability issue
  });

  test.beforeEach(async ({ page }) => {
    loginPage = new LoginPage(page, SHARED_SERVER_URL, authServerConfig, testCredentials);
    modelsPage = new ModelsListPage(page, SHARED_SERVER_URL);
    apiModelFormPage = new ApiModelFormPage(page, SHARED_SERVER_URL);
    chatPage = new ChatPage(page, SHARED_SERVER_URL);
    chatHistoryPage = new ChatHistoryPage(page, SHARED_SERVER_URL);
    chatSettingsPage = new ChatSettingsPage(page, SHARED_SERVER_URL);
  });

  test('basic chat functionality with simple Q&A @smoke @integration', async ({ page }) => {
    // Complete flow: login -> select model -> simple questions -> verify responses

    await loginPage.performOAuthLogin();
    await chatPage.navigateToChat();
    await chatPage.verifyChatEmpty();

    // Select model and ask two simple, direct questions
    await chatSettingsPage.selectModelQwen();

    // Question 1: Simple factual question
    await chatPage.sendMessage('What is 2+2?');
    await chatPage.waitForResponseComplete();
    await chatPage.waitForResponse('4');
    await chatPage.verifyMessageInHistory('user', 'What is 2+2?');
    await chatPage.verifyMessageInHistory('assistant', '4');

    // Question 2: Another simple factual question
    await chatPage.sendMessage('What day comes after Monday?');
    await chatPage.waitForResponse('Tuesday');
    await chatPage.verifyMessageInHistory('user', 'What day comes after Monday?');
    await chatPage.verifyMessageInHistory('assistant', 'Tuesday');
  });

  test('multi-chat management and error handling @integration', async ({ page }) => {
    await loginPage.performOAuthLogin();
    await chatPage.navigateToChat();
    await chatSettingsPage.selectModelQwen();

    // Create multiple chats with simple conversations
    const testMessages = ['Hello first chat', 'Hello second chat', 'Hello third chat'];

    for (const message of testMessages) {
      await chatPage.startNewChat();
      await chatPage.sendMessage(message);
      await chatPage.waitForResponseComplete();
    }

    // Verify chats in history and navigation
    await chatHistoryPage.verifyChatsInHistory(['Hello first', 'Hello second', 'Hello third']);

    // Test navigation between chats
    await chatHistoryPage.selectChatByTitle('Hello second');
    await chatPage.verifyMessageInHistory('user', 'Hello second chat');

    // Test error handling - empty message validation
    await chatPage.verifySendButtonDisabledForEmpty();

    // Test special characters and edge cases
    const edgeCases = ChatFixtures.createChatScenarios().edgeCases;
    await chatPage.sendMessage(edgeCases.specialCharacters);
    await chatPage.waitForResponseComplete();

    await chatPage.sendMessage(edgeCases.unicodeCharacters);
    await chatPage.waitForResponseComplete();

    // Test network error simulation
    await chatPage.simulateNetworkFailure();
    await chatPage.sendMessageAndReturn('This should fail');
    await chatPage.expectError();

    // Recovery
    await chatPage.restoreNetworkConnection();
    await chatPage.sendMessage('Recovery test');
    await chatPage.waitForResponseComplete();

    // Cleanup - delete test chats
    for (const message of testMessages) {
      const partialTitle = message.slice(0, 10);
      await chatHistoryPage.deleteChatByTitle(partialTitle);
    }
    await chatHistoryPage.verifyHistoryEmpty();
  });
});
