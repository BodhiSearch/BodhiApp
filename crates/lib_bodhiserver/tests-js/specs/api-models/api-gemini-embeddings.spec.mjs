import { ApiModelFixtures } from '@/fixtures/apiModelFixtures.mjs';
import { ApiModelFormPage } from '@/pages/ApiModelFormPage.mjs';
import { LoginPage } from '@/pages/LoginPage.mjs';
import { ModelsListPage } from '@/pages/ModelsListPage.mjs';
import { TokensPage } from '@/pages/TokensPage.mjs';
import { getAuthServerConfig, getTestCredentials } from '@/utils/auth-server-client.mjs';
import { fetchWithBearer, mintApiToken } from '@/utils/api-model-helpers.mjs';
import { expect, test } from '@/fixtures.mjs';

// Live E2E test: create a Gemini embedding alias via the UI, mint a BodhiApp API token,
// then call /v1beta/models/{id}:embedContent to verify the proxy chain works end-to-end.
//
// Requires INTEG_TEST_GEMINI_API_KEY in .env.test.
// Uses gemini-embedding-001 (smallest Gemini embedding model, widely available).

const GEMINI_FORMAT = ApiModelFixtures.API_FORMATS.gemini;
const EMBED_MODEL = 'gemini-embedding-001';
const EMBED_ENDPOINT = `/v1beta/models/${EMBED_MODEL}:embedContent`;
const EMBED_BODY = { content: { parts: [{ text: 'hello world' }] } };

test.describe('API Models - Gemini Embeddings', () => {
  let authServerConfig;
  let testCredentials;
  let geminiApiKey;

  test.beforeAll(async () => {
    authServerConfig = getAuthServerConfig();
    testCredentials = getTestCredentials();
    geminiApiKey = process.env[GEMINI_FORMAT.envKey];
    if (!geminiApiKey) {
      throw new Error(
        `${GEMINI_FORMAT.envKey} missing in .env.test — required for api-gemini-embeddings spec`
      );
    }
  });

  let loginPage;
  let modelsPage;
  let formPage;
  let tokensPage;

  test.beforeEach(async ({ page, sharedServerUrl }) => {
    loginPage = new LoginPage(page, sharedServerUrl, authServerConfig, testCredentials);
    modelsPage = new ModelsListPage(page, sharedServerUrl);
    formPage = new ApiModelFormPage(page, sharedServerUrl);
    tokensPage = new TokensPage(page, sharedServerUrl);
  });

  test('create Gemini embedding alias and call embedContent via app API token', async ({
    page,
    sharedServerUrl,
  }) => {
    await loginPage.performOAuthLogin();
    await modelsPage.navigateToModels();
    await modelsPage.clickNewApiModel();

    await formPage.form.waitForFormReady();
    await formPage.form.selectApiFormat(GEMINI_FORMAT.format);
    await formPage.form.fillBasicInfo(geminiApiKey, GEMINI_FORMAT.baseUrl);
    await formPage.form.fetchAndSelectModels([EMBED_MODEL]);
    // Skip testConnection — the UI probes via :generateContent which embed-only
    // models (gemini-embedding-001) do not support.
    const modelId = await formPage.createModelAndCaptureId();

    try {
      const apiToken = await mintApiToken(
        tokensPage,
        page,
        'gemini-embed-test',
        'scope_token_user'
      );

      const { resp, data } = await fetchWithBearer(
        sharedServerUrl,
        apiToken,
        EMBED_ENDPOINT,
        EMBED_BODY
      );
      expect(resp.ok).toBe(true);
      expect(Array.isArray(data.embedding?.values)).toBe(true);
      expect(data.embedding.values.length).toBeGreaterThan(0);
    } finally {
      await modelsPage.navigateToModels();
      await modelsPage.deleteModel(modelId);
    }
  });
});
