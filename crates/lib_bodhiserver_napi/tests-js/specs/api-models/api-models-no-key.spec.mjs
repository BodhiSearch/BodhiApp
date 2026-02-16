import { ApiModelFormPage } from '@/pages/ApiModelFormPage.mjs';
import { ChatPage } from '@/pages/ChatPage.mjs';
import { LoginPage } from '@/pages/LoginPage.mjs';
import { ModelsListPage } from '@/pages/ModelsListPage.mjs';
import {
  getAuthServerConfig,
  getPreConfiguredResourceClient,
  getTestCredentials,
} from '@/utils/auth-server-client.mjs';
import { createServerManager } from '@/utils/bodhi-app-server.mjs';
import { createMockOpenAIServer } from '@/utils/mock-openai-server.mjs';
import { expect, test } from '@playwright/test';

test.describe('API Models - Optional Key (Mock Server)', () => {
  let serverManager;
  let baseUrl;
  let loginPage;
  let modelsPage;
  let formPage;
  let chatPage;
  let authServerConfig;
  let testCredentials;
  let mockOpenAIServer;

  test.beforeAll(async () => {
    authServerConfig = getAuthServerConfig();
    testCredentials = getTestCredentials();
    const resourceClient = getPreConfiguredResourceClient();
    const port = 51135;

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

    mockOpenAIServer = createMockOpenAIServer({ requiresAuth: false });
    await mockOpenAIServer.start();
  });

  test.beforeEach(async ({ page }) => {
    loginPage = new LoginPage(page, baseUrl, authServerConfig, testCredentials);
    modelsPage = new ModelsListPage(page, baseUrl);
    formPage = new ApiModelFormPage(page, baseUrl);
    chatPage = new ChatPage(page, baseUrl);
  });

  test.afterAll(async () => {
    if (mockOpenAIServer) {
      await mockOpenAIServer.stop();
    }
    if (serverManager) {
      await serverManager.stopServer();
    }
  });

  test('complete api key lifecycle - starting with key', async ({ page }) => {
    const mockServerUrl = mockOpenAIServer.getBaseUrl();

    await loginPage.performOAuthLogin();
    await modelsPage.navigateToModels();
    await modelsPage.clickNewApiModel();

    await formPage.form.waitForFormReady();
    await formPage.form.selectApiFormat('openai');
    await formPage.form.fillBaseUrl(mockServerUrl);
    await formPage.form.checkUseApiKey();
    await formPage.form.fillApiKey('test-key-initial');

    mockOpenAIServer.clearRequestLog();
    await formPage.form.clickFetchModels();
    await formPage.form.expectFetchSuccess();
    const fetchRequest = mockOpenAIServer.getLastRequest();
    expect(fetchRequest.headers.authorization).toBe('Bearer test-key-initial');

    await formPage.form.searchAndSelectModel('mock-gpt-4');

    mockOpenAIServer.clearRequestLog();
    await formPage.form.testConnection();
    const testRequest = mockOpenAIServer.getLastRequest();
    expect(testRequest.headers.authorization).toBe('Bearer test-key-initial');

    const modelId = await formPage.createModelAndCaptureId();

    // Step 2: Test in chat with initial key
    await chatPage.navigateToChat();
    await chatPage.selectModel('mock-gpt-4');
    await chatPage.sendMessage('Hello');
    await chatPage.waitForResponseComplete();

    const response = await chatPage.getLastAssistantMessage();
    expect(response).toContain('Mock response to: Hello');

    mockOpenAIServer.clearRequestLog();
    await chatPage.sendMessage('verify key');
    await chatPage.waitForResponseComplete();

    const chatRequests = mockOpenAIServer
      .getRequestLog()
      .filter((r) => r.path === '/v1/chat/completions');
    const lastChatRequest = chatRequests[chatRequests.length - 1];
    expect(lastChatRequest.headers.authorization).toBe('Bearer test-key-initial');

    // Step 3: Edit - Keep existing key, add another model
    await modelsPage.navigateToModels();
    await modelsPage.editModel(modelId);
    await formPage.form.waitForFormReady();

    const isChecked = await formPage.form.isUseApiKeyChecked();
    expect(isChecked).toBe(true);

    mockOpenAIServer.clearRequestLog();
    await formPage.form.clickFetchModels();
    await formPage.form.expectFetchSuccess();
    const fetchWithStoredKey = mockOpenAIServer.getLastRequest();
    expect(fetchWithStoredKey.headers.authorization).toBeDefined();

    await formPage.form.searchAndSelectModel('mock-gpt-3.5-turbo');
    await formPage.updateModel();

    // Step 4: Test chat with other model using kept key
    await chatPage.navigateToChat();
    await chatPage.selectModel('mock-gpt-3.5-turbo');

    mockOpenAIServer.clearRequestLog();
    await chatPage.sendMessage('Test other model');
    await chatPage.waitForResponseComplete();

    const otherModelRequests = mockOpenAIServer
      .getRequestLog()
      .filter((r) => r.path === '/v1/chat/completions');
    const otherModelRequest = otherModelRequests[otherModelRequests.length - 1];
    expect(otherModelRequest.headers.authorization).toBe('Bearer test-key-initial');
    expect(otherModelRequest.body.model).toBe('mock-gpt-3.5-turbo');

    // Step 5: Edit - Change to different API key
    await modelsPage.navigateToModels();
    await modelsPage.editModel(modelId);
    await formPage.form.waitForFormReady();

    await formPage.form.checkUseApiKey();
    await formPage.form.fillApiKey('test-key-changed');

    mockOpenAIServer.clearRequestLog();
    await formPage.form.clickFetchModels();
    await formPage.form.expectFetchSuccess();
    const fetchWithNewKey = mockOpenAIServer.getLastRequest();
    expect(fetchWithNewKey.headers.authorization).toBe('Bearer test-key-changed');

    await formPage.updateModel();

    // Step 6: Test chat with changed key
    await chatPage.navigateToChat();
    await chatPage.selectModel('mock-gpt-4');

    mockOpenAIServer.clearRequestLog();
    await chatPage.sendMessage('Changed key test');
    await chatPage.waitForResponseComplete();

    const changedKeyRequests = mockOpenAIServer
      .getRequestLog()
      .filter((r) => r.path === '/v1/chat/completions');
    const changedKeyRequest = changedKeyRequests[changedKeyRequests.length - 1];
    expect(changedKeyRequest.headers.authorization).toBe('Bearer test-key-changed');

    // Step 7: Edit - Remove API key (uncheck)
    await modelsPage.navigateToModels();
    await modelsPage.editModel(modelId);
    await formPage.form.waitForFormReady();

    await formPage.form.uncheckUseApiKey();
    await formPage.updateModel();

    await modelsPage.navigateToModels();
    await modelsPage.editModel(modelId);
    await formPage.form.waitForFormReady();
    const isCheckedAfterRemove = await formPage.form.isUseApiKeyChecked();
    expect(isCheckedAfterRemove).toBe(false);

    // Step 8: Test chat without key
    await chatPage.navigateToChat();
    await chatPage.selectModel('mock-gpt-4');

    mockOpenAIServer.clearRequestLog();
    await chatPage.sendMessage('No key test');
    await chatPage.waitForResponseComplete();

    const noKeyRequests = mockOpenAIServer
      .getRequestLog()
      .filter((r) => r.path === '/v1/chat/completions');
    const noKeyRequest = noKeyRequests[noKeyRequests.length - 1];
    expect(noKeyRequest.headers.authorization).toBeUndefined();

    await modelsPage.navigateToModels();
    await modelsPage.deleteModel(modelId);
  });

  test('complete api key lifecycle - starting without key', async ({ page }) => {
    const mockServerUrl = mockOpenAIServer.getBaseUrl();

    // Step 1: Create model WITHOUT API key
    await loginPage.performOAuthLogin();
    await modelsPage.navigateToModels();
    await modelsPage.clickNewApiModel();

    await formPage.form.waitForFormReady();
    await formPage.form.selectApiFormat('openai');
    await formPage.form.fillBaseUrl(mockServerUrl);
    await formPage.form.uncheckUseApiKey();

    mockOpenAIServer.clearRequestLog();
    await formPage.form.clickFetchModels();
    await formPage.form.expectFetchSuccess();
    const fetchRequest = mockOpenAIServer.getLastRequest();
    expect(fetchRequest.headers.authorization).toBeUndefined();

    await formPage.form.searchAndSelectModel('mock-gpt-4');

    mockOpenAIServer.clearRequestLog();
    await formPage.form.testConnection();
    const testRequest = mockOpenAIServer.getLastRequest();
    expect(testRequest.headers.authorization).toBeUndefined();

    const modelId = await formPage.createModelAndCaptureId();

    // Step 2: Test in chat without key
    await chatPage.navigateToChat();
    await chatPage.selectModel('mock-gpt-4');
    await chatPage.sendMessage('Hello');
    await chatPage.waitForResponseComplete();

    const response = await chatPage.getLastAssistantMessage();
    expect(response).toContain('Mock response to: Hello');

    mockOpenAIServer.clearRequestLog();
    await chatPage.sendMessage('verify no key');
    await chatPage.waitForResponseComplete();

    const chatRequests = mockOpenAIServer
      .getRequestLog()
      .filter((r) => r.path === '/v1/chat/completions');
    const lastChatRequest = chatRequests[chatRequests.length - 1];
    expect(lastChatRequest.headers.authorization).toBeUndefined();

    // Step 3: Edit - Keep no key, add another model
    await modelsPage.navigateToModels();
    await modelsPage.editModel(modelId);
    await formPage.form.waitForFormReady();

    const isChecked = await formPage.form.isUseApiKeyChecked();
    expect(isChecked).toBe(false);

    mockOpenAIServer.clearRequestLog();
    await formPage.form.clickFetchModels();
    await formPage.form.expectFetchSuccess();
    const fetchWithoutKey = mockOpenAIServer.getLastRequest();
    expect(fetchWithoutKey.headers.authorization).toBeUndefined();

    await formPage.form.searchAndSelectModel('mock-gpt-3.5-turbo');
    await formPage.updateModel();

    // Step 4: Test chat with other model, still no key
    await chatPage.navigateToChat();
    await chatPage.selectModel('mock-gpt-3.5-turbo');

    mockOpenAIServer.clearRequestLog();
    await chatPage.sendMessage('Test other model');
    await chatPage.waitForResponseComplete();

    const otherModelRequests = mockOpenAIServer
      .getRequestLog()
      .filter((r) => r.path === '/v1/chat/completions');
    const otherModelRequest = otherModelRequests[otherModelRequests.length - 1];
    expect(otherModelRequest.headers.authorization).toBeUndefined();
    expect(otherModelRequest.body.model).toBe('mock-gpt-3.5-turbo');

    // Step 5: Edit - Add API key (check checkbox)
    await modelsPage.navigateToModels();
    await modelsPage.editModel(modelId);
    await formPage.form.waitForFormReady();

    await formPage.form.checkUseApiKey();
    await formPage.form.fillApiKey('test-key-added');

    mockOpenAIServer.clearRequestLog();
    await formPage.form.clickFetchModels();
    await formPage.form.expectFetchSuccess();
    const fetchWithAddedKey = mockOpenAIServer.getLastRequest();
    expect(fetchWithAddedKey.headers.authorization).toBe('Bearer test-key-added');

    await formPage.updateModel();

    // Step 6: Test chat with added key
    await chatPage.navigateToChat();
    await chatPage.selectModel('mock-gpt-4');

    mockOpenAIServer.clearRequestLog();
    await chatPage.sendMessage('Added key test');
    await chatPage.waitForResponseComplete();

    const addedKeyRequests = mockOpenAIServer
      .getRequestLog()
      .filter((r) => r.path === '/v1/chat/completions');
    const addedKeyRequest = addedKeyRequests[addedKeyRequests.length - 1];
    expect(addedKeyRequest.headers.authorization).toBe('Bearer test-key-added');

    // Step 7: Edit - Remove API key again
    await modelsPage.navigateToModels();
    await modelsPage.editModel(modelId);
    await formPage.form.waitForFormReady();

    await formPage.form.uncheckUseApiKey();
    await formPage.updateModel();

    await modelsPage.navigateToModels();
    await modelsPage.editModel(modelId);
    await formPage.form.waitForFormReady();
    const isCheckedAfterRemove = await formPage.form.isUseApiKeyChecked();
    expect(isCheckedAfterRemove).toBe(false);

    // Step 8: Test chat without key again
    await chatPage.navigateToChat();
    await chatPage.selectModel('mock-gpt-4');

    mockOpenAIServer.clearRequestLog();
    await chatPage.sendMessage('Back to no key');
    await chatPage.waitForResponseComplete();

    const finalNoKeyRequests = mockOpenAIServer
      .getRequestLog()
      .filter((r) => r.path === '/v1/chat/completions');
    const finalNoKeyRequest = finalNoKeyRequests[finalNoKeyRequests.length - 1];
    expect(finalNoKeyRequest.headers.authorization).toBeUndefined();

    await modelsPage.navigateToModels();
    await modelsPage.deleteModel(modelId);
  });
});
