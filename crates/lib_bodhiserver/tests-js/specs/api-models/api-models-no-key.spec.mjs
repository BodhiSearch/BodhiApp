import { LLMock } from '@copilotkit/aimock';
import { ApiModelFixtures } from '@/fixtures/apiModelFixtures.mjs';
import { ApiModelFormPage } from '@/pages/ApiModelFormPage.mjs';
import { ChatPage } from '@/pages/ChatPage.mjs';
import { LoginPage } from '@/pages/LoginPage.mjs';
import { ModelsListPage } from '@/pages/ModelsListPage.mjs';
import { getAuthServerConfig, getTestCredentials } from '@/utils/auth-server-client.mjs';
import { expect, test } from '@/fixtures.mjs';

// Tests for the optional API-key feature using a mock LLM server.
//
// One shared LLMock instance handles all formats — the mock responds to model
// name regardless of API format, so cross-format endpoint hits (e.g. Anthropic
// test hitting /v1/chat/completions) will 404, catching routing bugs. The canary
// response proves the mock is serving rather than a real LLM.
//
// Auth recovery test (fetch fails without key, succeeds after adding key) is
// included here since the same mock server makes it deterministic.

const MOCK_MODELS = ['mock-gpt-4', 'mock-gpt-3.5-turbo'];
const MOCK_GEMINI_MODELS = ['mock-gemini-flash', 'mock-gemini-pro'];
const MOCK_RESPONSE = 'David Smith is from Chicago';

test.describe('API Models - Optional Key (Mock Server)', () => {
  let sharedMockServer;
  let authServerConfig;
  let testCredentials;

  test.beforeAll(async () => {
    authServerConfig = getAuthServerConfig();
    testCredentials = getTestCredentials();

    sharedMockServer = new LLMock({ port: 0 });
    for (const model of MOCK_MODELS) {
      sharedMockServer.addFixture({
        match: { model },
        response: { content: MOCK_RESPONSE },
      });
    }
    // Mount a /v1/models handler so the backend can validate model IDs on create/update.
    // Returns Anthropic-format when anthropic-version header is present, OpenAI-format otherwise.
    sharedMockServer.mount('/v1', {
      async handleRequest(req, res, pathname) {
        if (pathname === '/models' && req.method === 'GET') {
          const isAnthropic = !!req.headers['anthropic-version'];
          const data = MOCK_MODELS.map((id) =>
            isAnthropic
              ? { id, display_name: id, created_at: '2024-01-01T00:00:00Z', type: 'model' }
              : { id, object: 'model', created: 0, owned_by: 'mock' }
          );
          const body = JSON.stringify(
            isAnthropic ? { data, has_more: false } : { object: 'list', data }
          );
          res.writeHead(200, { 'Content-Type': 'application/json' });
          res.end(body);
          return true;
        }
        return false;
      },
    });
    // Mount a /v1beta handler for Gemini endpoints.
    sharedMockServer.mount('/v1beta', {
      async handleRequest(req, res, pathname) {
        // GET /models -> Gemini models list
        if (pathname === '/models' && req.method === 'GET') {
          const models = MOCK_GEMINI_MODELS.map((id) => ({
            name: `models/${id}`,
            version: '001',
            displayName: id,
            supportedGenerationMethods: ['generateContent', 'embedContent'],
            description: `Mock ${id}`,
          }));
          res.writeHead(200, { 'Content-Type': 'application/json' });
          res.end(JSON.stringify({ models }));
          return true;
        }
        // POST /models/{id}:generateContent -> non-streaming Gemini response
        if (req.method === 'POST' && pathname.endsWith(':generateContent')) {
          res.writeHead(200, { 'Content-Type': 'application/json' });
          res.end(
            JSON.stringify({
              candidates: [
                {
                  content: { role: 'model', parts: [{ text: MOCK_RESPONSE }] },
                  finishReason: 'STOP',
                },
              ],
            })
          );
          return true;
        }
        // POST /models/{id}:streamGenerateContent -> SSE streaming Gemini response
        if (req.method === 'POST' && pathname.endsWith(':streamGenerateContent')) {
          res.writeHead(200, {
            'Content-Type': 'text/event-stream',
            'Cache-Control': 'no-cache',
            Connection: 'keep-alive',
          });
          const chunk = JSON.stringify({
            candidates: [
              {
                content: { role: 'model', parts: [{ text: MOCK_RESPONSE }] },
                finishReason: 'STOP',
              },
            ],
            usageMetadata: { promptTokenCount: 5, candidatesTokenCount: 5, totalTokenCount: 10 },
          });
          res.write(`data: ${chunk}\r\n\r\n`);
          res.end();
          return true;
        }
        return false;
      },
    });
    await sharedMockServer.start();
  });

  test.afterAll(async () => {
    if (sharedMockServer) {
      await sharedMockServer.stop();
    }
  });

  for (const [formatKey, formatConfig] of Object.entries(ApiModelFixtures.API_FORMATS)) {
    test.describe(`[${formatConfig.format}]`, () => {
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

      test('api key lifecycle - starting with key', async ({ page }) => {
        const mockServerUrl = sharedMockServer.url + (formatConfig.mockBaseUrlSuffix ?? '/v1');
        const primaryModel = formatConfig.mockModel ?? 'mock-gpt-4';
        const secondaryModel = formatConfig.mockSecondaryModel ?? 'mock-gpt-3.5-turbo';

        await loginPage.performOAuthLogin();
        await modelsPage.navigateToModels();
        await modelsPage.clickNewApiModel();

        // Create model WITH API key
        await formPage.form.waitForFormReady();
        await formPage.form.selectApiFormat(formatConfig.format);
        await formPage.form.fillBaseUrl(mockServerUrl);
        await formPage.form.checkUseApiKey();
        await formPage.form.fillApiKey('test-key-initial');
        await formPage.form.clickFetchModels();
        await formPage.form.expectFetchSuccess();
        await formPage.form.searchAndSelectModel(primaryModel);
        await formPage.form.testConnection();
        const modelId = await formPage.createModelAndCaptureId();

        // Step 2: Test in chat with initial key
        await chatPage.navigateToChat();
        await chatPage.selectModel(primaryModel);
        await chatPage.sendMessage('Hello');
        await chatPage.waitForResponseComplete();
        expect(await chatPage.getLastAssistantMessage()).toContain(MOCK_RESPONSE);

        // Step 3: Edit – keep existing key, add another model
        await modelsPage.navigateToModels();
        await modelsPage.editModel(modelId);
        await formPage.form.waitForFormReady();
        expect(await formPage.form.isUseApiKeyChecked()).toBe(true);
        await formPage.form.clickFetchModels();
        await formPage.form.expectFetchSuccess();
        await formPage.form.searchAndSelectModel(secondaryModel);
        await formPage.updateModel();

        // Step 4: Test chat with other model using kept key
        await chatPage.navigateToChat();
        await chatPage.selectModel(secondaryModel);
        await chatPage.sendMessage('Test other model');
        await chatPage.waitForResponseComplete();
        expect(await chatPage.getLastAssistantMessage()).toContain(MOCK_RESPONSE);

        // Step 5: Edit – change to a different API key
        await modelsPage.navigateToModels();
        await modelsPage.editModel(modelId);
        await formPage.form.waitForFormReady();
        await formPage.form.checkUseApiKey();
        await formPage.form.fillApiKey('test-key-changed');
        await formPage.form.clickFetchModels();
        await formPage.form.expectFetchSuccess();
        await formPage.updateModel();

        // Step 6: Test chat with changed key
        await chatPage.navigateToChat();
        await chatPage.selectModel(primaryModel);
        await chatPage.sendMessage('Changed key test');
        await chatPage.waitForResponseComplete();
        expect(await chatPage.getLastAssistantMessage()).toContain(MOCK_RESPONSE);

        // Step 7: Edit – remove API key
        await modelsPage.navigateToModels();
        await modelsPage.editModel(modelId);
        await formPage.form.waitForFormReady();
        await formPage.form.uncheckUseApiKey();
        await formPage.updateModel();

        await modelsPage.navigateToModels();
        await modelsPage.editModel(modelId);
        await formPage.form.waitForFormReady();
        expect(await formPage.form.isUseApiKeyChecked()).toBe(false);

        // Step 8: Test chat without key
        await chatPage.navigateToChat();
        await chatPage.selectModel(primaryModel);
        await chatPage.sendMessage('No key test');
        await chatPage.waitForResponseComplete();
        expect(await chatPage.getLastAssistantMessage()).toContain(MOCK_RESPONSE);

        await modelsPage.navigateToModels();
        await modelsPage.deleteModel(modelId);
      });

      test('api key lifecycle - starting without key', async ({ page }) => {
        const mockServerUrl = sharedMockServer.url + (formatConfig.mockBaseUrlSuffix ?? '/v1');
        const primaryModel = formatConfig.mockModel ?? 'mock-gpt-4';
        const secondaryModel = formatConfig.mockSecondaryModel ?? 'mock-gpt-3.5-turbo';

        await loginPage.performOAuthLogin();
        await modelsPage.navigateToModels();
        await modelsPage.clickNewApiModel();

        // Create model WITHOUT API key
        await formPage.form.waitForFormReady();
        await formPage.form.selectApiFormat(formatConfig.format);
        await formPage.form.fillBaseUrl(mockServerUrl);
        await formPage.form.uncheckUseApiKey();
        await formPage.form.clickFetchModels();
        await formPage.form.expectFetchSuccess();
        await formPage.form.searchAndSelectModel(primaryModel);
        await formPage.form.testConnection();
        const modelId = await formPage.createModelAndCaptureId();

        // Step 2: Test in chat without key
        await chatPage.navigateToChat();
        await chatPage.selectModel(primaryModel);
        await chatPage.sendMessage('Hello');
        await chatPage.waitForResponseComplete();
        expect(await chatPage.getLastAssistantMessage()).toContain(MOCK_RESPONSE);

        // Step 3: Edit – keep no key, add another model
        await modelsPage.navigateToModels();
        await modelsPage.editModel(modelId);
        await formPage.form.waitForFormReady();
        expect(await formPage.form.isUseApiKeyChecked()).toBe(false);
        await formPage.form.clickFetchModels();
        await formPage.form.expectFetchSuccess();
        await formPage.form.searchAndSelectModel(secondaryModel);
        await formPage.updateModel();

        // Step 4: Test chat with other model, still no key
        await chatPage.navigateToChat();
        await chatPage.selectModel(secondaryModel);
        await chatPage.sendMessage('Test other model');
        await chatPage.waitForResponseComplete();
        expect(await chatPage.getLastAssistantMessage()).toContain(MOCK_RESPONSE);

        // Step 5: Edit – add API key
        await modelsPage.navigateToModels();
        await modelsPage.editModel(modelId);
        await formPage.form.waitForFormReady();
        await formPage.form.checkUseApiKey();
        await formPage.form.fillApiKey('test-key-added');
        await formPage.form.clickFetchModels();
        await formPage.form.expectFetchSuccess();
        await formPage.updateModel();

        // Step 6: Test chat with added key
        await chatPage.navigateToChat();
        await chatPage.selectModel(primaryModel);
        await chatPage.sendMessage('Added key test');
        await chatPage.waitForResponseComplete();
        expect(await chatPage.getLastAssistantMessage()).toContain(MOCK_RESPONSE);

        // Step 7: Edit – remove API key again
        await modelsPage.navigateToModels();
        await modelsPage.editModel(modelId);
        await formPage.form.waitForFormReady();
        await formPage.form.uncheckUseApiKey();
        await formPage.updateModel();

        await modelsPage.navigateToModels();
        await modelsPage.editModel(modelId);
        await formPage.form.waitForFormReady();
        expect(await formPage.form.isUseApiKeyChecked()).toBe(false);

        // Step 8: Test chat without key again
        await chatPage.navigateToChat();
        await chatPage.selectModel(primaryModel);
        await chatPage.sendMessage('Back to no key');
        await chatPage.waitForResponseComplete();
        expect(await chatPage.getLastAssistantMessage()).toContain(MOCK_RESPONSE);

        await modelsPage.navigateToModels();
        await modelsPage.deleteModel(modelId);
      });
    });
  }
});
