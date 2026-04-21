// This spec validates the /v1, /anthropic/v1, and /v1beta surfaces via RAW
// `fetch` (no SDKs). It complements api-sdk-compat.spec.mjs, which drives the
// same endpoints through the providers' official SDK clients. Keep both:
// this file catches middleware bugs (x-api-key rewriting, universal
// /v1/chat/completions routing, anthropic_oauth headers, chat-UI wiring) that
// SDK calls may not exercise directly; the SDK spec catches client-library
// compatibility regressions.
import { expect, test } from '@/fixtures.mjs';
import { ApiModelFixtures } from '@/fixtures/apiModelFixtures.mjs';
import { AccessRequestReviewPage } from '@/pages/AccessRequestReviewPage.mjs';
import { ApiModelFormPage } from '@/pages/ApiModelFormPage.mjs';
import { ChatPage } from '@/pages/ChatPage.mjs';
import { LoginPage } from '@/pages/LoginPage.mjs';
import { ModelsListPage } from '@/pages/ModelsListPage.mjs';
import { OAuthTestApp } from '@/pages/OAuthTestApp.mjs';
import { TokensPage } from '@/pages/TokensPage.mjs';
import { SHARED_STATIC_SERVER_URL } from '@/test-helpers.mjs';
import {
  fetchWithApiKey,
  fetchWithBearer,
  fetchWithBearerSSE,
  mintApiToken,
} from '@/utils/api-model-helpers.mjs';
import {
  getAuthServerConfig,
  getPreConfiguredAppClient,
  getTestCredentials,
} from '@/utils/auth-server-client.mjs';

// Live upstream tests: validate end-to-end API flow for all supported formats.
//
// Each test:
//   1. Creates one API model per format (openai, openai_responses, anthropic).
//      Format-specific prefixes (multiTestPrefix) disambiguate model IDs when
//      multiple formats share the same upstream model name (e.g. gpt-4.1-nano).
//   2. Uses those models to verify:
//      - Format-native (primary) endpoints with both API token and OAuth app token.
//      - Universal /v1/chat/completions endpoint (all upstream providers support it).
//      - Anthropic's dual BodhiApp endpoints: /v1/messages AND /anthropic/v1/messages.
//   3. Exercises the chat UI for each format.
//
// Adding a new format: add its entry to ApiModelFixtures.API_FORMATS.
// All loops below pick it up automatically.

/**
 * Create all format API models with disambiguation prefixes.
 * Returns a map of formatKey → { modelId, effectiveModel }.
 * effectiveModel = `${multiTestPrefix}${model}` (the ID to use in API calls).
 */
async function createAllFormatModels(modelsPage, formPage) {
  const models = {};
  for (const [formatKey, formatConfig] of Object.entries(ApiModelFixtures.API_FORMATS)) {
    const apiKey = process.env[formatConfig.envKey];
    const prefix = formatConfig.multiTestPrefix ?? '';

    await modelsPage.navigateToModels();
    await modelsPage.clickNewApiModel();
    await formPage.form.waitForFormReady();
    await formPage.form.selectApiFormat(formatConfig.format);
    if (prefix) {
      await formPage.form.fillBasicInfoWithPrefix(apiKey, prefix, formatConfig.baseUrl);
    } else {
      await formPage.form.fillBasicInfo(apiKey, formatConfig.baseUrl);
    }
    await formPage.form.fetchAndSelectModels([formatConfig.model]);
    await formPage.form.testConnection();
    const modelId = await formPage.createModelAndCaptureId();

    models[formatKey] = {
      modelId,
      effectiveModel: prefix ? `${prefix}${formatConfig.model}` : formatConfig.model,
    };
  }
  return models;
}

async function deleteAllModels(modelsPage, models) {
  await modelsPage.navigateToModels();
  for (const { modelId } of Object.values(models)) {
    await modelsPage.deleteModel(modelId);
  }
}

test.describe('Live upstream - API token', () => {
  let loginPage;
  let modelsPage;
  let formPage;
  let chatPage;
  let tokensPage;
  let authServerConfig;
  let testCredentials;

  test.beforeAll(() => {
    for (const [, formatConfig] of Object.entries(ApiModelFixtures.API_FORMATS)) {
      const apiKey = process.env[formatConfig.envKey];
      if (!apiKey) {
        throw new Error(`${formatConfig.envKey} environment variable not set`);
      }
    }
    authServerConfig = getAuthServerConfig();
    testCredentials = getTestCredentials();
  });

  test.beforeEach(async ({ page, sharedServerUrl }) => {
    loginPage = new LoginPage(page, sharedServerUrl, authServerConfig, testCredentials);
    modelsPage = new ModelsListPage(page, sharedServerUrl);
    formPage = new ApiModelFormPage(page, sharedServerUrl);
    chatPage = new ChatPage(page, sharedServerUrl);
    tokensPage = new TokensPage(page, sharedServerUrl);
  });

  test('create all format models + API token, verify primary and universal endpoints, chat UI', async ({
    page,
    sharedServerUrl,
  }) => {
    let models; // formatKey → { modelId, effectiveModel }
    let apiToken;

    await loginPage.performOAuthLogin();

    try {
      // Create one API model per format with disambiguation prefixes
      await test.step('Create API models for all formats', async () => {
        models = await createAllFormatModels(modelsPage, formPage);
      });

      // Mint one BodhiApp API token shared across all format tests
      await test.step('Mint API token', async () => {
        apiToken = await mintApiToken(
          tokensPage,
          page,
          'live-upstream-api-token',
          'scope_token_user'
        );
        expect(apiToken).toMatch(/^bodhiapp_/);
      });

      // Verify each format's native (primary) endpoint(s)
      await test.step('Primary endpoints with API token', async () => {
        for (const [formatKey, formatConfig] of Object.entries(ApiModelFixtures.API_FORMATS)) {
          const { effectiveModel } = models[formatKey];
          const streamingEndpoints = formatConfig.streamingEndpoints?.(effectiveModel) ?? [];
          for (const endpoint of formatConfig.primaryEndpoints(effectiveModel)) {
            const body = formatConfig.buildPrimaryBody(effectiveModel, formatConfig.chatQuestion);
            const fetchFn = streamingEndpoints.includes(endpoint)
              ? fetchWithBearerSSE
              : fetchWithBearer;
            const { resp, data } = await fetchFn(sharedServerUrl, apiToken, endpoint, body);
            expect(resp.status, `${formatKey} ${endpoint}`).toBe(200);
            const content = formatConfig.extractPrimaryResponse(data);
            expect(content.toLowerCase(), `${formatKey} ${endpoint} response`).toContain(
              formatConfig.chatExpected
            );
          }
        }
      });

      // Anthropic SDK clients authenticate with x-api-key instead of Authorization: Bearer.
      // anthropic_auth_middleware (mounted only on the anthropic_apis route group) rewrites
      // x-api-key to Authorization: Bearer before api_auth_middleware validates the token.
      // Only /anthropic/v1/messages goes through that middleware; /v1/messages does not.
      await test.step('Anthropic /anthropic/v1/messages with x-api-key token', async () => {
        const formatConfig = ApiModelFixtures.API_FORMATS.anthropic;
        const { effectiveModel } = models.anthropic;
        const body = formatConfig.buildPrimaryBody(effectiveModel, formatConfig.chatQuestion);
        const { resp, data } = await fetchWithApiKey(
          sharedServerUrl,
          apiToken,
          '/anthropic/v1/messages',
          body
        );
        expect(resp.status, 'anthropic x-api-key /anthropic/v1/messages').toBe(200);
        const content = formatConfig.extractPrimaryResponse(data);
        expect(content.toLowerCase(), 'anthropic x-api-key response').toContain(
          formatConfig.chatExpected
        );
      });

      // Formats where BodhiApp allows routing to /v1/chat/completions.
      // openai_responses is excluded — the handler explicitly rejects that format
      // on this endpoint (use /v1/responses instead).
      await test.step('Universal /v1/chat/completions with API token', async () => {
        for (const [formatKey, formatConfig] of Object.entries(ApiModelFixtures.API_FORMATS)) {
          if (!formatConfig.supportsUniversalChatCompletions) continue;
          const { effectiveModel } = models[formatKey];
          const body = {
            model: effectiveModel,
            messages: [{ role: 'user', content: formatConfig.chatQuestion }],
          };
          const { resp, data } = await fetchWithBearer(
            sharedServerUrl,
            apiToken,
            '/v1/chat/completions',
            body
          );
          expect(resp.status, `${formatKey} /v1/chat/completions`).toBe(200);
          const content = data.choices?.[0]?.message?.content ?? '';
          expect(content.toLowerCase(), `${formatKey} chat completions response`).toContain(
            formatConfig.chatExpected
          );
        }
      });

      // Chat UI integration: verify each format works end-to-end in the chat page
      await test.step('Chat UI for each format', async () => {
        for (const [formatKey, formatConfig] of Object.entries(ApiModelFixtures.API_FORMATS)) {
          const { effectiveModel } = models[formatKey];
          await chatPage.navigateToChat();
          await chatPage.waitForChatPageLoad();
          await chatPage.selectModel(effectiveModel);
          await chatPage.waitForApiFormat(formatConfig.formatDisplayName);
          await chatPage.sendMessage(formatConfig.chatQuestion);
          await chatPage.waitForResponseComplete();
          const response = await chatPage.getLastAssistantMessage();
          expect(response.toLowerCase()).toContain(formatConfig.chatExpected);
        }
      });
    } finally {
      await deleteAllModels(modelsPage, models);
    }
  });

  test('auth error recovery: fetch models without key fails, succeeds after adding key', async ({
    sharedServerUrl,
  }) => {
    await loginPage.performOAuthLogin();
    await modelsPage.navigateToModels();
    await modelsPage.clickNewApiModel();

    // Use OpenAI as representative format (auth error behavior is format-agnostic)
    const formatConfig = ApiModelFixtures.API_FORMATS.openai;
    const apiKey = process.env[formatConfig.envKey];

    await formPage.form.waitForFormReady();
    await formPage.form.selectApiFormat(formatConfig.format);
    await formPage.form.expectBaseUrlValue(formatConfig.baseUrl);

    // Trigger auth error by attempting fetch without API key
    await formPage.form.uncheckUseApiKey();
    await formPage.form.clickFetchModels();
    await formPage.form.expectFetchError();

    // Recovery: provide key and retry
    await formPage.form.checkUseApiKey();
    await formPage.form.fillApiKey(apiKey);
    await formPage.form.fetchAndSelectModels([formatConfig.model]);
    await formPage.form.testConnection();

    const modelId = await formPage.createModelAndCaptureId();
    await modelsPage.navigateToModels();
    await modelsPage.deleteModel(modelId);
  });
});

test.describe('Live upstream - OAuth app token', () => {
  let loginPage;
  let modelsPage;
  let formPage;
  let authServerConfig;
  let testCredentials;

  test.beforeAll(() => {
    for (const [, formatConfig] of Object.entries(ApiModelFixtures.API_FORMATS)) {
      const apiKey = process.env[formatConfig.envKey];
      if (!apiKey) {
        throw new Error(`${formatConfig.envKey} environment variable not set`);
      }
    }
    authServerConfig = getAuthServerConfig();
    testCredentials = getTestCredentials();
  });

  test.beforeEach(async ({ page, sharedServerUrl }) => {
    loginPage = new LoginPage(page, sharedServerUrl, authServerConfig, testCredentials);
    modelsPage = new ModelsListPage(page, sharedServerUrl);
    formPage = new ApiModelFormPage(page, sharedServerUrl);
  });

  test('OAuth app token: verify primary and universal endpoints for all formats', async ({
    page,
    sharedServerUrl,
  }) => {
    let models;
    let oauthToken;

    // Login establishes the KC session needed for the OAuth approval step later
    await loginPage.performOAuthLogin();

    try {
      // Create one API model per format with disambiguation prefixes
      await test.step('Create API models for all formats', async () => {
        models = await createAllFormatModels(modelsPage, formPage);
      });

      // Obtain a 3rd-party OAuth Bearer token via the OAuthTestApp (full PKCE flow).
      // The existing KC session (from performOAuthLogin above) means Keycloak skips re-login.
      await test.step('Obtain OAuth app token via OAuthTestApp', async () => {
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

        // Approve the access request as the logged-in BodhiApp user
        const reviewPage = new AccessRequestReviewPage(page, sharedServerUrl);
        await reviewPage.approve();

        await app.oauth.waitForAccessRequestCallback(SHARED_STATIC_SERVER_URL);
        await app.accessCallback.waitForLoaded();
        await app.accessCallback.clickLogin();
        await app.oauth.waitForTokenExchange(SHARED_STATIC_SERVER_URL);

        await app.dashboard.navigateTo();
        oauthToken = await app.dashboard.getAccessToken();
        expect(oauthToken).toBeTruthy();
        expect(oauthToken.length).toBeGreaterThan(50);
      });

      // Verify each format's native (primary) endpoint(s)
      await test.step('Primary endpoints with OAuth app token', async () => {
        for (const [formatKey, formatConfig] of Object.entries(ApiModelFixtures.API_FORMATS)) {
          const { effectiveModel } = models[formatKey];
          const streamingEndpoints = formatConfig.streamingEndpoints?.(effectiveModel) ?? [];
          for (const endpoint of formatConfig.primaryEndpoints(effectiveModel)) {
            const body = formatConfig.buildPrimaryBody(effectiveModel, formatConfig.chatQuestion);
            const fetchFn = streamingEndpoints.includes(endpoint)
              ? fetchWithBearerSSE
              : fetchWithBearer;
            const { resp, data } = await fetchFn(sharedServerUrl, oauthToken, endpoint, body);
            expect(resp.status, `${formatKey} ${endpoint}`).toBe(200);
            const content = formatConfig.extractPrimaryResponse(data);
            expect(content.toLowerCase(), `${formatKey} ${endpoint} response`).toContain(
              formatConfig.chatExpected
            );
          }
        }
      });

      // Formats where BodhiApp allows routing to /v1/chat/completions.
      await test.step('Universal /v1/chat/completions with OAuth app token', async () => {
        for (const [formatKey, formatConfig] of Object.entries(ApiModelFixtures.API_FORMATS)) {
          if (!formatConfig.supportsUniversalChatCompletions) continue;
          const { effectiveModel } = models[formatKey];
          const body = {
            model: effectiveModel,
            messages: [{ role: 'user', content: formatConfig.chatQuestion }],
          };
          const { resp, data } = await fetchWithBearer(
            sharedServerUrl,
            oauthToken,
            '/v1/chat/completions',
            body
          );
          expect(resp.status, `${formatKey} /v1/chat/completions`).toBe(200);
          const content = data.choices?.[0]?.message?.content ?? '';
          expect(content.toLowerCase(), `${formatKey} chat completions response`).toContain(
            formatConfig.chatExpected
          );
        }
      });
    } finally {
      // Navigate back to BodhiApp to clean up models
      await deleteAllModels(modelsPage, models);
    }
  });
});
