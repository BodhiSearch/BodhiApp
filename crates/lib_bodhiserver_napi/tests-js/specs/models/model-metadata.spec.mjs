import path from 'node:path';
import { fileURLToPath } from 'node:url';
import { LoginPage } from '@/pages/LoginPage.mjs';
import { ModelsListPage } from '@/pages/ModelsListPage.mjs';
import {
  getAuthServerConfig,
  getPreConfiguredResourceClient,
  getTestCredentials,
} from '@/utils/auth-server-client.mjs';
import { createServerManager } from '@/utils/bodhi-app-server.mjs';
import { expect, test } from '@playwright/test';

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
    const resourceClient = getPreConfiguredResourceClient();
    const port = 51135;

    // Set custom HF_HOME to point to test GGUF fixtures
    const testHfHome = path.resolve(__dirname, '../../data/test-gguf');

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

  test('modal refresh auto-updates metadata from models page', async ({ page }) => {
    // Login and navigate to models page
    await loginPage.performOAuthLogin();
    await modelsPage.navigateToModels();

    // Use a model that hasn't been processed yet (fresh database)
    const modelData = MODEL_CAPABILITIES['qwen-vision'];
    const modelAlias = `${modelData.repo}:${modelData.qualifier}`;

    // Open preview modal
    await modelsPage.clickPreviewButton(modelAlias);

    // Modal should show "No metadata available" initially
    const noMetadataMessage = page.locator('text=No metadata available for this model.');
    await expect(noMetadataMessage).toBeVisible();

    // Click the inline refresh button in the modal (body button since no metadata)
    const refreshButton = page.locator('[data-testid="preview-modal-refresh-button-body"]');
    await expect(refreshButton).toBeVisible();
    await expect(refreshButton).toHaveAttribute('data-teststate', 'ready');
    await refreshButton.click();

    // Wait for loading state
    await expect(refreshButton).toHaveAttribute('data-teststate', 'loading');

    // Wait for ready state (metadata refresh completed)
    await expect(refreshButton).toHaveAttribute('data-teststate', 'ready');

    // Modal should automatically update to show metadata (without reopening)
    await expect(noMetadataMessage).not.toBeVisible();

    // Verify metadata is now displayed with correct values
    await modelsPage.verifyPreviewBasicInfo({
      alias: modelAlias,
      repo: modelData.repo,
      filename: modelData.filename,
      snapshot: modelData.snapshot,
      source: 'MODEL',
    });

    // Verify detailed capability values
    await modelsPage.verifyPreviewCapability('vision', modelData.capabilities.vision);
    await modelsPage.verifyPreviewCapability('audio', modelData.capabilities.audio);
    await modelsPage.verifyPreviewCapability('thinking', modelData.capabilities.thinking);
    await modelsPage.verifyPreviewCapability(
      'function_calling',
      modelData.capabilities.function_calling
    );
    await modelsPage.verifyPreviewCapability(
      'structured_output',
      modelData.capabilities.structured_output
    );

    await modelsPage.closePreviewModal();
  });

  test('modal refresh auto-updates metadata from modelfiles page', async ({ page }) => {
    // Login and navigate to modelfiles page
    await loginPage.performOAuthLogin();
    await page.goto(`${baseUrl}/ui/modelfiles/`);
    await page.waitForSelector('[data-testid="modelfiles-content"]');

    // Use a model that hasn't been processed yet
    const modelData = MODEL_CAPABILITIES['phi-tools'];
    const refreshKey = `${modelData.repo}-${modelData.filename}`;

    // Open preview modal from modelfiles page
    // There are 2 buttons due to responsive design: mobile (sm:hidden) and desktop (hidden sm:table-cell)
    // Use last() to get the desktop version which is visible at default viewport
    const previewButton = page
      .locator(`[data-testid="modelfiles-preview-button-${refreshKey}"]`)
      .last();
    await previewButton.click();
    await expect(page.locator('[data-testid="model-preview-modal"]')).toBeVisible();

    // Modal should show "No metadata available" initially
    const noMetadataMessage = page.locator('text=No metadata available for this model.');
    await expect(noMetadataMessage).toBeVisible();

    // Click the inline refresh button in the modal (body button since no metadata)
    const refreshButton = page.locator('[data-testid="preview-modal-refresh-button-body"]');
    await expect(refreshButton).toBeVisible();
    await expect(refreshButton).toHaveAttribute('data-teststate', 'ready');
    await refreshButton.click();

    // Wait for loading state
    await expect(refreshButton).toHaveAttribute('data-teststate', 'loading');

    // Wait for ready state (metadata refresh completed)
    await expect(refreshButton).toHaveAttribute('data-teststate', 'ready');

    // Modal should automatically update to show metadata (without reopening)
    await expect(noMetadataMessage).not.toBeVisible();

    // Verify metadata is now displayed with correct values
    await modelsPage.verifyPreviewBasicInfo({
      alias: `${modelData.repo}/${modelData.filename}`,
      repo: modelData.repo,
      filename: modelData.filename,
      snapshot: modelData.snapshot,
      source: 'MODEL',
    });

    // Verify detailed capability values
    await modelsPage.verifyPreviewCapability('vision', modelData.capabilities.vision);
    await modelsPage.verifyPreviewCapability('audio', modelData.capabilities.audio);
    await modelsPage.verifyPreviewCapability('thinking', modelData.capabilities.thinking);
    await modelsPage.verifyPreviewCapability(
      'function_calling',
      modelData.capabilities.function_calling
    );
    await modelsPage.verifyPreviewCapability(
      'structured_output',
      modelData.capabilities.structured_output
    );

    await modelsPage.closePreviewModal();
  });

  test('complete flow: login → refresh models → verify metadata → per-row refresh', async ({
    page,
  }) => {
    // Login and navigate to models page
    await loginPage.performOAuthLogin();
    await modelsPage.navigateToModels();

    // Refresh all models first to populate metadata
    await test.step('Refresh metadata for all test models', async () => {
      for (const [, modelData] of Object.entries(MODEL_CAPABILITIES)) {
        const modelAlias = `${modelData.repo}:${modelData.qualifier}`;
        await modelsPage.clickRefreshButton(modelAlias);
        await modelsPage.waitForToast('Metadata refreshed successfully');
      }
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
  });
});
