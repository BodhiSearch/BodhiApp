import { LoginPage } from '@/pages/LoginPage.mjs';
import { ModelsListPage } from '@/pages/ModelsListPage.mjs';
import { randomPort } from '@/test-helpers.mjs';
import {
  createAuthServerTestClient,
  getAuthServerConfig,
  getTestCredentials,
} from '@/utils/auth-server-client.mjs';
import { createServerManager } from '@/utils/bodhi-app-server.mjs';
import { expect, test } from '@playwright/test';
import path from 'path';
import { fileURLToPath } from 'url';

const __filename = fileURLToPath(import.meta.url);
const __dirname = path.dirname(__filename);

// Expected capabilities for each test model
const MODEL_CAPABILITIES = {
  'llama-plain': {
    repo: 'test/llama-plain',
    filename: 'llama-plain.gguf',
    qualifier: 'plain',
    snapshot: 'test1234567890abcdef1234567890abcdef12345678',
    capabilities: {
      vision: false,
      audio: false,
      thinking: false,
      function_calling: false,
      structured_output: false,
    },
  },
  'qwen-vision': {
    repo: 'test/qwen-vision',
    filename: 'qwen-vision.gguf',
    qualifier: 'vision',
    snapshot: 'test1234567890abcdef1234567890abcdef12345678',
    capabilities: {
      vision: true,
      audio: false,
      thinking: false,
      function_calling: false,
      structured_output: false,
    },
  },
  'phi-tools': {
    repo: 'test/phi-tools',
    filename: 'phi-tools.gguf',
    qualifier: 'tools',
    snapshot: 'test1234567890abcdef1234567890abcdef12345678',
    capabilities: {
      vision: false,
      audio: false,
      thinking: false,
      function_calling: true,
      structured_output: false, // TOOLS_TEMPLATE only has function calling, no JSON schema patterns
    },
  },
  'deepseek-thinking': {
    repo: 'test/deepseek-thinking',
    filename: 'deepseek-thinking.gguf',
    qualifier: 'thinking',
    snapshot: 'test1234567890abcdef1234567890abcdef12345678',
    capabilities: {
      vision: false,
      audio: false,
      thinking: true,
      function_calling: false,
      structured_output: false,
    },
  },
  'mistral-audio': {
    repo: 'test/mistral-audio',
    filename: 'mistral-audio.gguf',
    qualifier: 'audio',
    snapshot: 'test1234567890abcdef1234567890abcdef12345678',
    capabilities: {
      vision: false,
      audio: true,
      thinking: false,
      function_calling: false,
      structured_output: false,
    },
  },
  'llava-multimodal': {
    repo: 'test/llava-multimodal',
    filename: 'llava-multimodal.gguf',
    qualifier: 'multimodal',
    snapshot: 'test1234567890abcdef1234567890abcdef12345678',
    capabilities: {
      vision: true,
      audio: false,
      thinking: false,
      function_calling: true,
      structured_output: false, // TOOLS_TEMPLATE only has function calling, no JSON schema patterns
    },
  },
};

test.describe('Model Metadata Refresh and Preview', () => {
  let serverManager;
  let baseUrl;
  let loginPage;
  let modelsPage;
  let testData;

  test.beforeAll(async () => {
    // Server setup with custom HF_HOME
    const authServerConfig = getAuthServerConfig();
    const testCredentials = getTestCredentials();
    const port = randomPort();
    const serverUrl = `http://localhost:${port}`;

    // Set custom HF_HOME to point to test GGUF fixtures
    const testHfHome = path.resolve(__dirname, '../../data/test-gguf');

    const authClient = createAuthServerTestClient(authServerConfig);
    const resourceClient = await authClient.createResourceClient(serverUrl);
    await authClient.makeResourceAdmin(
      resourceClient.clientId,
      resourceClient.clientSecret,
      testCredentials.userId
    );

    serverManager = createServerManager({
      appStatus: 'ready',
      authUrl: authServerConfig.authUrl,
      authRealm: authServerConfig.authRealm,
      clientId: resourceClient.clientId,
      clientSecret: resourceClient.clientSecret,
      port,
      host: 'localhost',
      hfHomePath: testHfHome,
      logLevel: 'info',
    });

    baseUrl = await serverManager.startServer();
    testData = { authServerConfig, testCredentials };
  });

  test.beforeEach(async ({ page }) => {
    loginPage = new LoginPage(page, baseUrl, testData.authServerConfig, testData.testCredentials);
    modelsPage = new ModelsListPage(page, baseUrl);
  });

  test.afterAll(async () => {
    if (serverManager) {
      await serverManager.stopServer();
    }
  });

  test('complete flow: login → refresh all models → verify metadata for each model → per-row refresh', async ({
    page,
  }) => {
    // Login and navigate to models page
    await loginPage.performOAuthLogin();
    await modelsPage.navigateToModels();

    await test.step('Refresh All - button disabled while processing', async () => {
      await modelsPage.clickRefreshAll();
      await modelsPage.verifyRefreshButtonState('disabled');

      await modelsPage.waitForQueueIdle();
      await modelsPage.verifyRefreshButtonState('enabled');
    });

    await test.step('Verify llama-plain model (no capabilities)', async () => {
      const modelData = MODEL_CAPABILITIES['llama-plain'];
      const modelAlias = `${modelData.repo}:${modelData.qualifier}`;

      await modelsPage.clickPreviewButton(modelAlias);

      await modelsPage.verifyPreviewBasicInfo({
        alias: modelAlias,
        repo: modelData.repo,
        filename: modelData.filename,
        snapshot: modelData.snapshot,
        source: 'MODEL',
      });

      for (const [capability, expectedValue] of Object.entries(modelData.capabilities)) {
        await modelsPage.verifyPreviewCapability(capability, expectedValue);
      }

      await modelsPage.closePreviewModal();
    });

    await test.step('Verify qwen-vision model (vision capability)', async () => {
      const modelData = MODEL_CAPABILITIES['qwen-vision'];
      const modelAlias = `${modelData.repo}:${modelData.qualifier}`;

      await modelsPage.clickPreviewButton(modelAlias);

      await modelsPage.verifyPreviewBasicInfo({
        alias: modelAlias,
        repo: modelData.repo,
        filename: modelData.filename,
        snapshot: modelData.snapshot,
        source: 'MODEL',
      });

      await modelsPage.verifyPreviewCapability('vision', true);
      await modelsPage.verifyPreviewCapability('audio', false);
      await modelsPage.verifyPreviewCapability('thinking', false);
      await modelsPage.verifyPreviewCapability('function_calling', false);
      await modelsPage.verifyPreviewCapability('structured_output', false);

      await modelsPage.closePreviewModal();
    });

    await test.step('Verify phi-tools model (function calling capability)', async () => {
      const modelData = MODEL_CAPABILITIES['phi-tools'];
      const modelAlias = `${modelData.repo}:${modelData.qualifier}`;

      await modelsPage.clickPreviewButton(modelAlias);

      await modelsPage.verifyPreviewBasicInfo({
        alias: modelAlias,
        repo: modelData.repo,
        filename: modelData.filename,
        snapshot: modelData.snapshot,
        source: 'MODEL',
      });

      await modelsPage.verifyPreviewCapability('vision', false);
      await modelsPage.verifyPreviewCapability('audio', false);
      await modelsPage.verifyPreviewCapability('thinking', false);
      await modelsPage.verifyPreviewCapability('function_calling', true);
      await modelsPage.verifyPreviewCapability('structured_output', false);

      await modelsPage.closePreviewModal();
    });

    await test.step('Verify deepseek-thinking model (thinking capability)', async () => {
      const modelData = MODEL_CAPABILITIES['deepseek-thinking'];
      const modelAlias = `${modelData.repo}:${modelData.qualifier}`;

      await modelsPage.clickPreviewButton(modelAlias);

      await modelsPage.verifyPreviewBasicInfo({
        alias: modelAlias,
        repo: modelData.repo,
        filename: modelData.filename,
        snapshot: modelData.snapshot,
        source: 'MODEL',
      });

      await modelsPage.verifyPreviewCapability('vision', false);
      await modelsPage.verifyPreviewCapability('audio', false);
      await modelsPage.verifyPreviewCapability('thinking', true);
      await modelsPage.verifyPreviewCapability('function_calling', false);
      await modelsPage.verifyPreviewCapability('structured_output', false);

      await modelsPage.closePreviewModal();
    });

    await test.step('Verify mistral-audio model (audio capability)', async () => {
      const modelData = MODEL_CAPABILITIES['mistral-audio'];
      const modelAlias = `${modelData.repo}:${modelData.qualifier}`;

      await modelsPage.clickPreviewButton(modelAlias);

      await modelsPage.verifyPreviewBasicInfo({
        alias: modelAlias,
        repo: modelData.repo,
        filename: modelData.filename,
        snapshot: modelData.snapshot,
        source: 'MODEL',
      });

      await modelsPage.verifyPreviewCapability('vision', false);
      await modelsPage.verifyPreviewCapability('audio', true);
      await modelsPage.verifyPreviewCapability('thinking', false);
      await modelsPage.verifyPreviewCapability('function_calling', false);
      await modelsPage.verifyPreviewCapability('structured_output', false);

      await modelsPage.closePreviewModal();
    });

    await test.step('Verify llava-multimodal model (vision + function calling)', async () => {
      const modelData = MODEL_CAPABILITIES['llava-multimodal'];
      const modelAlias = `${modelData.repo}:${modelData.qualifier}`;

      await modelsPage.clickPreviewButton(modelAlias);

      await modelsPage.verifyPreviewBasicInfo({
        alias: modelAlias,
        repo: modelData.repo,
        filename: modelData.filename,
        snapshot: modelData.snapshot,
        source: 'MODEL',
      });

      await modelsPage.verifyPreviewCapability('vision', true);
      await modelsPage.verifyPreviewCapability('audio', false);
      await modelsPage.verifyPreviewCapability('thinking', false);
      await modelsPage.verifyPreviewCapability('function_calling', true);
      await modelsPage.verifyPreviewCapability('structured_output', false);

      await modelsPage.closePreviewModal();
    });

    await test.step('Per-row refresh button updates metadata synchronously', async () => {
      const modelData = MODEL_CAPABILITIES['qwen-vision'];
      const modelAlias = `${modelData.repo}:${modelData.qualifier}`;

      // Click per-row refresh button
      await modelsPage.clickRefreshButton(modelAlias);

      // Wait for success toast
      await modelsPage.waitForToast('Metadata refreshed successfully');

      // Verify button is enabled again after refresh
      await modelsPage.verifyRefreshButtonStateForModel(modelAlias, 'enabled');

      // Verify metadata is updated by checking preview
      await modelsPage.clickPreviewButton(modelAlias);

      await modelsPage.verifyPreviewBasicInfo({
        alias: modelAlias,
        repo: modelData.repo,
        filename: modelData.filename,
        snapshot: modelData.snapshot,
      });

      await modelsPage.verifyPreviewCapability('vision', true);
      await modelsPage.verifyPreviewCapability('audio', false);

      await modelsPage.closePreviewModal();
    });

    await test.step('Per-row refresh for different model type', async () => {
      const modelData = MODEL_CAPABILITIES['llama-plain'];
      const modelAlias = `${modelData.repo}:${modelData.qualifier}`;

      const refreshBtn = page.locator(`[data-testid="refresh-button-${modelAlias}"]`);
      await expect(refreshBtn).toBeVisible();

      await refreshBtn.click();

      // Wait for success toast
      await modelsPage.waitForToast('Metadata refreshed successfully');

      // Verify button is enabled again after refresh
      await modelsPage.verifyRefreshButtonStateForModel(modelAlias, 'enabled');
    });
  });
});
