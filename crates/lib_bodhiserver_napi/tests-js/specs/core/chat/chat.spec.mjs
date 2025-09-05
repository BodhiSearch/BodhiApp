import { test, expect } from '@playwright/test';
import {
  createAuthServerTestClient,
  getAuthServerConfig,
  getTestCredentials,
} from '../../../playwright/auth-server-client.mjs';
import { createServerManager } from '../../../playwright/bodhi-app-server.mjs';
import { randomPort } from '../../../test-helpers.mjs';
import { join } from 'path';
import { fileURLToPath } from 'url';
import { dirname } from 'path';

const __filename = fileURLToPath(import.meta.url);
const __dirname = dirname(__filename);
const projectRoot = join(__dirname, '..', '..', '..', '..', '..', '..');
const hfHomePath = join(projectRoot, 'hf-home');

import { LoginPage } from '../../../pages/LoginPage.mjs';
import { ModelsListPage } from '../../../pages/ModelsListPage.mjs';
import { ApiModelFormPage } from '../../../pages/ApiModelFormPage.mjs';
import { ChatPage } from '../../../pages/ChatPage.mjs';
import { ChatHistoryPage } from '../../../pages/ChatHistoryPage.mjs';
import { ChatSettingsPage } from '../../../pages/ChatSettingsPage.mjs';
import { ApiModelFixtures } from '../../../fixtures/ApiModelFixtures.mjs';
import { ChatFixtures } from '../../../fixtures/ChatFixtures.mjs';

test.describe('Chat Interface - Core Functionality', () => {
  let authServerConfig;
  let testCredentials;
  let serverManager;
  let baseUrl;
  let authClient;
  let resourceClient;
  let loginPage;
  let modelsPage;
  let apiModelFormPage;
  let chatPage;
  let chatHistoryPage;
  let chatSettingsPage;
  let testApiKey;

  test.beforeAll(async () => {
    // Environment and server setup
    testApiKey = ChatFixtures.getEnvironmentData().getApiKey();
    authServerConfig = getAuthServerConfig();
    testCredentials = getTestCredentials();
    const port = randomPort();
    const serverUrl = `http://localhost:${port}`;

    authClient = createAuthServerTestClient(authServerConfig);
    resourceClient = await authClient.createResourceClient(serverUrl);
    await authClient.makeResourceAdmin(
      resourceClient.clientId,
      resourceClient.clientSecret,
      testCredentials.username
    );

    serverManager = createServerManager({
      appStatus: 'ready',
      authUrl: authServerConfig.authUrl,
      authRealm: authServerConfig.authRealm,
      clientId: resourceClient.clientId,
      clientSecret: resourceClient.clientSecret,
      port,
      host: 'localhost',
      envVars: { HF_HOME: hfHomePath },
    });

    baseUrl = await serverManager.startServer();
  });

  test.beforeEach(async ({ page }) => {
    loginPage = new LoginPage(page, baseUrl, authServerConfig, testCredentials);
    modelsPage = new ModelsListPage(page, baseUrl);
    apiModelFormPage = new ApiModelFormPage(page, baseUrl);
    chatPage = new ChatPage(page, baseUrl);
    chatHistoryPage = new ChatHistoryPage(page, baseUrl);
    chatSettingsPage = new ChatSettingsPage(page, baseUrl);
  });

  test.afterAll(async () => {
    if (serverManager) {
      await serverManager.stopServer();
    }
  });

  test('basic chat functionality with simple Q&A @smoke @integration', async ({ page }) => {
    // Complete flow: login -> select model -> simple questions -> verify responses

    await loginPage.performOAuthLogin();
    await chatPage.navigateToChat();
    await chatPage.verifyChatEmpty();

    // Select model and ask two simple, direct questions
    await chatSettingsPage.selectModel('bartowski/microsoft_Phi-4-mini-instruct-GGUF:Q4_K_M');

    // Question 1: Simple factual question
    await chatPage.sendMessage('What is 2+2?');
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
    await chatSettingsPage.selectModel('bartowski/microsoft_Phi-4-mini-instruct-GGUF:Q4_K_M');

    // Create multiple chats with simple conversations
    const testMessages = ['Hello first chat', 'Hello second chat', 'Hello third chat'];

    for (const message of testMessages) {
      await chatPage.startNewChat();
      await chatPage.sendMessage(message);
      await chatPage.waitForResponseComplete(); // Wait for complete response
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
    await chatPage.sendMessage('This should fail');
    await chatPage.expectNetworkError();

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

  test('settings persistence and model switching @integration', async ({ page }) => {
    await loginPage.performOAuthLogin();
    await chatPage.navigateToChat();

    // Configure custom settings
    const customSettings = ChatFixtures.getSettingsConfigurations().customized;
    await chatSettingsPage.configureSettings({
      model: customSettings.model,
      streaming: customSettings.streaming,
      apiToken: { enabled: customSettings.apiToken.enabled, value: testApiKey },
      systemPrompt: customSettings.systemPrompt,
      temperature: customSettings.temperature,
      maxTokens: customSettings.maxTokens,
    });

    // Test with configured settings
    await chatPage.sendMessage('Hi');
    if (customSettings.streaming) {
      await chatPage.waitForStreamingComplete();
    } else {
      await chatPage.waitForNonStreamingResponse();
    }

    // Verify settings persist after reload
    await page.reload();
    await chatPage.waitForChatPageLoad();
    await chatSettingsPage.verifySettings({
      model: customSettings.model,
      streaming: customSettings.streaming,
      systemPrompt: customSettings.systemPrompt,
    });

    // Test model switching
    await chatSettingsPage.selectModel('bartowski/microsoft_Phi-4-mini-instruct-GGUF:Q4_K_M');
    await chatPage.sendMessage('Test after switch');
    await chatPage.waitForResponseComplete();

    // Reset for cleanup
    await chatSettingsPage.configureSettings({
      model: 'bartowski/microsoft_Phi-4-mini-instruct-GGUF:Q4_K_M',
      streaming: true,
      systemPrompt: { enabled: false, content: '' },
    });
  });
});
