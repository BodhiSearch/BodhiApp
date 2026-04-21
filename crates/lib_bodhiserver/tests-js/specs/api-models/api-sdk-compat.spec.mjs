// SDK compatibility suite.
//
// Unlike api-live-upstream.spec.mjs (which exercises the same endpoints via
// raw `fetch` to catch middleware bugs like x-api-key rewriting and universal
// /v1/chat/completions routing), this spec drives each BodhiApp proxy endpoint
// through the provider's OFFICIAL SDK client. Goal: guarantee that real 3rd-
// party integrations using `openai`, `@anthropic-ai/sdk`, and `@google/genai`
// continue to work against BodhiApp's /v1, /anthropic/v1, and /v1beta surfaces.
//
// Matrix: 4 formats (openai, openai_responses, anthropic, gemini) × 2 auth
// methods (API token, OAuth app token) × 5 capabilities (non-streaming chat,
// streaming chat, tool use, models list, embeddings). Each combination is its
// own Playwright test() — a failure in one does not block the others.
// Embeddings are skipped for openai_responses (server rejects) and anthropic
// (upstream has no public embeddings API).

import { expect, test } from '@/fixtures.mjs';
import { ApiModelFixtures } from '@/fixtures/apiModelFixtures.mjs';
import { AccessRequestReviewPage } from '@/pages/AccessRequestReviewPage.mjs';
import { ApiModelFormPage } from '@/pages/ApiModelFormPage.mjs';
import { LoginPage } from '@/pages/LoginPage.mjs';
import { ModelsListPage } from '@/pages/ModelsListPage.mjs';
import { OAuthTestApp } from '@/pages/OAuthTestApp.mjs';
import { TokensPage } from '@/pages/TokensPage.mjs';
import { SHARED_STATIC_SERVER_URL } from '@/test-helpers.mjs';
import { mintApiToken } from '@/utils/api-model-helpers.mjs';
import {
  getAuthServerConfig,
  getPreConfiguredAppClient,
  getTestCredentials,
} from '@/utils/auth-server-client.mjs';
import { buildSdkAdapter } from '@/utils/sdk-adapters.mjs';

const SDK_FORMATS = ['openai', 'openai_responses', 'anthropic', 'gemini'];
const AUTH_METHODS = ['apiToken', 'appToken'];
const EMBED_SUPPORTED = new Set(['openai', 'gemini']);

// Shared state lives in MODULE SCOPE so it survives worker behaviour that
// re-evaluates the describe body (e.g. fixture-config-driven worker rotation
// in Playwright v1.57 when multiple nested describes sit under a parent
// `test.use(...)`). If state were inside the describe callback, a re-evaluation
// would replace the object reference, wiping [setup]'s provisioned aliases.
const state = {
  apiToken: null,
  appToken: null,
  models: {}, // formatKey -> { modelId, effectiveModel }
  embeddingAliases: {}, // 'openai' | 'gemini' -> effective embedding model id
};
let dbResetOnce = false;

// Tests run in file order on a single worker (playwright.config.mjs workers:1).
// Do NOT use test.describe.configure({ mode: 'serial' }): serial mode skips all
// subsequent tests in the group on first failure, which is the opposite of
// what this matrix needs — a broken tool-call in anthropic×appToken must not
// block gemini×apiToken. Instead we rely on default mode + disable auto-reset
// so the setup test's provisioned aliases/tokens survive across the capability
// tests in this file.
test.describe('SDK compatibility — provider endpoints × auth tokens', () => {
  // Reset DB once (for the [setup] test), then no-op so each capability test
  // still sees the aliases + tokens created by [setup].
  test.use({
    autoResetDb: async ({}, use, testInfo) => {
      if (!dbResetOnce) {
        const { resetDatabase } = await import('@/test-helpers.mjs');
        const { getServerUrl } = await import('@/utils/db-config.mjs');
        await resetDatabase(getServerUrl(testInfo.project.name));
        dbResetOnce = true;
      }
      await use();
    },
  });

  test.beforeAll(() => {
    for (const f of SDK_FORMATS) {
      const { envKey } = ApiModelFixtures.API_FORMATS[f];
      if (!process.env[envKey]) {
        throw new Error(`${envKey} environment variable not set`);
      }
    }
  });

  test('[setup] login, provision aliases, mint API + OAuth app tokens', async ({
    page,
    sharedServerUrl,
  }) => {
    const authServerConfig = getAuthServerConfig();
    const testCredentials = getTestCredentials();
    const loginPage = new LoginPage(page, sharedServerUrl, authServerConfig, testCredentials);
    const modelsPage = new ModelsListPage(page, sharedServerUrl);
    const formPage = new ApiModelFormPage(page, sharedServerUrl);
    const tokensPage = new TokensPage(page, sharedServerUrl);

    await loginPage.performOAuthLogin();

    for (const formatKey of SDK_FORMATS) {
      const cfg = ApiModelFixtures.API_FORMATS[formatKey];
      const apiKey = process.env[cfg.envKey];
      const prefix = cfg.multiTestPrefix ?? '';
      // For openai + gemini we register the embedding model alongside the
      // chat model on the same alias so a single upstream key covers both.
      const modelsToRegister = cfg.embeddingModel ? [cfg.model, cfg.embeddingModel] : [cfg.model];

      await modelsPage.navigateToModels();
      await modelsPage.clickNewApiModel();
      await formPage.form.waitForFormReady();
      await formPage.form.selectApiFormat(cfg.format);
      if (prefix) {
        await formPage.form.fillBasicInfoWithPrefix(apiKey, prefix, cfg.baseUrl);
      } else {
        await formPage.form.fillBasicInfo(apiKey, cfg.baseUrl);
      }
      await formPage.form.fetchAndSelectModels(modelsToRegister);
      await formPage.form.testConnection();
      const modelId = await formPage.createModelAndCaptureId();

      state.models[formatKey] = {
        modelId,
        effectiveModel: prefix ? `${prefix}${cfg.model}` : cfg.model,
      };
      if (cfg.embeddingModel) {
        state.embeddingAliases[formatKey] = prefix
          ? `${prefix}${cfg.embeddingModel}`
          : cfg.embeddingModel;
      }
    }

    state.apiToken = await mintApiToken(
      tokensPage,
      page,
      'sdk-compat-api-token',
      'scope_token_user'
    );
    expect(state.apiToken).toMatch(/^bodhiapp_/);

    // OAuth app token via full PKCE flow (same pattern as api-live-upstream.spec.mjs).
    const appClient = getPreConfiguredAppClient();
    const redirectUri = `${SHARED_STATIC_SERVER_URL}/callback`;
    const app = new OAuthTestApp(page, SHARED_STATIC_SERVER_URL);

    await app.navigate();
    await app.config.configureOAuthForm({
      bodhiServerUrl: sharedServerUrl,
      authServerUrl: authServerConfig.authUrl,
      realm: authServerConfig.authRealm,
      clientId: appClient.clientId,
      redirectUri,
      scope: 'openid profile email',
      requested: JSON.stringify({ version: '1' }),
    });
    await app.config.submitAccessRequest();
    await app.oauth.waitForAccessRequestRedirect(sharedServerUrl);

    const reviewPage = new AccessRequestReviewPage(page, sharedServerUrl);
    await reviewPage.approve();

    await app.oauth.waitForAccessRequestCallback(SHARED_STATIC_SERVER_URL);
    await app.accessCallback.waitForLoaded();
    await app.accessCallback.clickLogin();
    await app.oauth.waitForTokenExchange(SHARED_STATIC_SERVER_URL);

    await app.dashboard.navigateTo();
    state.appToken = await app.dashboard.getAccessToken();
    expect(state.appToken).toBeTruthy();
    expect(state.appToken.length).toBeGreaterThan(50);
  });

  for (const format of SDK_FORMATS) {
    for (const auth of AUTH_METHODS) {
      test.describe(`${format} × ${auth}`, () => {
        const tokenFor = () => (auth === 'apiToken' ? state.apiToken : state.appToken);
        const adapterFor = (serverUrl) => buildSdkAdapter(format, serverUrl, tokenFor(), state);

        test('non-streaming chat', async ({ sharedServerUrl }) => {
          const out = await adapterFor(sharedServerUrl).chat({ stream: false });
          expect(out.text.toLowerCase()).toContain('tuesday');
        });

        test('streaming chat', async ({ sharedServerUrl }) => {
          const out = await adapterFor(sharedServerUrl).chat({ stream: true });
          expect(out.chunkCount).toBeGreaterThan(0);
          expect(out.text.toLowerCase()).toContain('tuesday');
        });

        test('tool use (get_weather)', async ({ sharedServerUrl }) => {
          const out = await adapterFor(sharedServerUrl).toolCall();
          expect(out.toolName).toBe('get_weather');
          expect(String(out.toolArgs.location ?? '').toLowerCase()).toContain('san francisco');
        });

        test('models list', async ({ sharedServerUrl }) => {
          const ids = await adapterFor(sharedServerUrl).listModels();
          const effective = state.models[format].effectiveModel;
          // Providers return models as "models/<id>" (Gemini) or "<id>" (OpenAI/Anthropic).
          // Accept either form so the assertion stays SDK-agnostic.
          expect(ids.some((id) => id === effective || id.endsWith(`/${effective}`))).toBe(true);
        });

        if (EMBED_SUPPORTED.has(format)) {
          test('embeddings', async ({ sharedServerUrl }) => {
            const out = await adapterFor(sharedServerUrl).embed('hello world');
            expect(Array.isArray(out.vector)).toBe(true);
            expect(out.dimensions).toBeGreaterThan(0);
          });
        } else {
          test.skip(`embeddings (not supported by ${format})`, () => {});
        }
      });
    }
  }

  test('[teardown] delete provisioned aliases', async ({ page, sharedServerUrl }) => {
    // Each test gets a fresh browser context → session cookies from [setup]
    // are gone. Log in again before touching the admin UI.
    const authServerConfig = getAuthServerConfig();
    const testCredentials = getTestCredentials();
    const loginPage = new LoginPage(page, sharedServerUrl, authServerConfig, testCredentials);
    await loginPage.performOAuthLogin();

    const modelsPage = new ModelsListPage(page, sharedServerUrl);
    await modelsPage.navigateToModels();
    for (const { modelId } of Object.values(state.models)) {
      await modelsPage.deleteModel(modelId);
    }
  });
});
