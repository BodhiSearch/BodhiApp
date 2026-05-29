import { ApiModelFixtures } from '@/fixtures/apiModelFixtures.mjs';
import { ApiModelFormPage } from '@/pages/ApiModelFormPage.mjs';
import { ChatPage } from '@/pages/ChatPage.mjs';
import { LoginPage } from '@/pages/LoginPage.mjs';
import { ModelRouterFormPage } from '@/pages/ModelRouterFormPage.mjs';
import { ModelsListPage } from '@/pages/ModelsListPage.mjs';
import { getAuthServerConfig, getTestCredentials } from '@/utils/auth-server-client.mjs';
import { expect, test } from '@/fixtures.mjs';

// Black-box E2E: create a model-router via the UI whose single enabled target points at a
// working OpenAI alias, select the router in chat, send a message, and receive a completion.
// Assertions are made through the UI only (no page.evaluate / direct fetch).
//
// Requires INTEG_TEST_OPENAI_API_KEY in .env.test.

const OPENAI = ApiModelFixtures.API_FORMATS.openai;
const OPENAI_MODEL = ApiModelFixtures.OPENAI_MODEL;
const ROUTER_NAME = 'e2e-router-stack';

test.describe('Model Router - pass-through', () => {
  let authServerConfig;
  let testCredentials;
  let openaiKey;

  test.beforeAll(async () => {
    authServerConfig = getAuthServerConfig();
    testCredentials = getTestCredentials();
    openaiKey = process.env[OPENAI.envKey];
    if (!openaiKey) {
      throw new Error(`${OPENAI.envKey} missing in .env.test — required for model-router spec`);
    }
  });

  let loginPage;
  let modelsPage;
  let apiFormPage;
  let routerFormPage;
  let chatPage;

  test.beforeEach(async ({ page, sharedServerUrl }) => {
    loginPage = new LoginPage(page, sharedServerUrl, authServerConfig, testCredentials);
    modelsPage = new ModelsListPage(page, sharedServerUrl);
    apiFormPage = new ApiModelFormPage(page, sharedServerUrl);
    routerFormPage = new ModelRouterFormPage(page, sharedServerUrl);
    chatPage = new ChatPage(page, sharedServerUrl);
  });

  test('create router with one enabled target, then chat through it', async () => {
    await loginPage.performOAuthLogin();

    // 1. Create the underlying OpenAI alias.
    await modelsPage.navigateToModels();
    await modelsPage.clickNewApiModel();
    await apiFormPage.form.waitForFormReady();
    await apiFormPage.form.selectApiFormat(OPENAI.format);
    await apiFormPage.form.fillBasicInfo(openaiKey, OPENAI.baseUrl);
    await apiFormPage.form.fetchAndSelectModels([OPENAI_MODEL]);
    const aliasId = await apiFormPage.createModelAndCaptureId();

    // 2. Create the model-router referencing that alias (reach the form via the UI button).
    await modelsPage.navigateToModels();
    await modelsPage.clickNewModelRouter();
    await routerFormPage.waitForFormReady();
    await routerFormPage.fillName(ROUTER_NAME);
    await routerFormPage.addTarget();
    await routerFormPage.selectTargetAlias(0, aliasId);
    // The model auto-pins to the alias's single model; submit.
    await routerFormPage.submit();
    await routerFormPage.waitForUrl('/ui/models/');

    // 3. Router shows up in the aggregate models list (GET /bodhi/v1/models).
    await modelsPage.waitForModelsToLoad();
    await modelsPage.verifyModelRouterInList(ROUTER_NAME);

    // 4. Select the router in chat and drive a completion.
    await chatPage.navigateToChat();
    await chatPage.selectModel(ROUTER_NAME);
    await chatPage.sendMessage(OPENAI.chatQuestion);
    await chatPage.waitForResponseComplete();
    const reply = await chatPage.getLastAssistantMessage();
    expect(reply.trim().length).toBeGreaterThan(0);
  });
});

// Phase-2 black-box E2E: a router whose primary target deterministically fails with a
// retryable error falls through to a working secondary, and the user sees a normal reply.
//
// The broken primary is a forward-all OpenAI alias (real provider + valid key, so it
// passes create-time validation) with a pinned model the upstream does not serve — the
// chat request gets a retryable 404, which the fallback strategy treats as "try next".
// The secondary is the working alias. No mock servers; UI interactions only.
const FALLBACK_ROUTER_NAME = 'e2e-router-fallback';
const BAD_PREFIX = 'bad/';
const BAD_MODEL = `${BAD_PREFIX}nonexistent-model-zzz`;

test.describe('Model Router - in-request fallback', () => {
  let authServerConfig;
  let testCredentials;
  let openaiKey;

  test.beforeAll(async () => {
    authServerConfig = getAuthServerConfig();
    testCredentials = getTestCredentials();
    openaiKey = process.env[OPENAI.envKey];
    if (!openaiKey) {
      throw new Error(`${OPENAI.envKey} missing in .env.test — required for model-router spec`);
    }
  });

  let loginPage;
  let modelsPage;
  let apiFormPage;
  let routerFormPage;
  let chatPage;

  test.beforeEach(async ({ page, sharedServerUrl }) => {
    loginPage = new LoginPage(page, sharedServerUrl, authServerConfig, testCredentials);
    modelsPage = new ModelsListPage(page, sharedServerUrl);
    apiFormPage = new ApiModelFormPage(page, sharedServerUrl);
    routerFormPage = new ModelRouterFormPage(page, sharedServerUrl);
    chatPage = new ChatPage(page, sharedServerUrl);
  });

  test('broken primary target falls through to working secondary', async () => {
    await loginPage.performOAuthLogin();

    // 1. Broken primary: forward-all alias with a prefix (pins are free-text, validated by
    //    prefix only — so a bogus model passes create-time checks but 404s at request time).
    await modelsPage.navigateToModels();
    await modelsPage.clickNewApiModel();
    await apiFormPage.form.waitForFormReady();
    await apiFormPage.form.selectApiFormat(OPENAI.format);
    await apiFormPage.form.fillBasicInfoWithPrefix(openaiKey, BAD_PREFIX, OPENAI.baseUrl);
    await apiFormPage.form.enableForwardAll();
    const primaryId = await apiFormPage.createModelAndCaptureId();

    // 2. Working secondary: a normal selected-models alias.
    await modelsPage.navigateToModels();
    await modelsPage.clickNewApiModel();
    await apiFormPage.form.waitForFormReady();
    await apiFormPage.form.selectApiFormat(OPENAI.format);
    await apiFormPage.form.fillBasicInfo(openaiKey, OPENAI.baseUrl);
    await apiFormPage.form.fetchAndSelectModels([OPENAI_MODEL]);
    const secondaryId = await apiFormPage.createModelAndCaptureId();

    // 3. Router: broken primary first, working secondary second.
    await modelsPage.navigateToModels();
    await modelsPage.clickNewModelRouter();
    await routerFormPage.waitForFormReady();
    await routerFormPage.fillName(FALLBACK_ROUTER_NAME);
    await routerFormPage.addTarget();
    await routerFormPage.selectTargetAlias(0, primaryId);
    await routerFormPage.fillTargetModel(0, BAD_MODEL);
    await routerFormPage.addTarget();
    await routerFormPage.selectTargetAlias(1, secondaryId);
    await routerFormPage.submit();
    await routerFormPage.waitForUrl('/ui/models/');
    await modelsPage.waitForModelsToLoad();
    await modelsPage.verifyModelRouterInList(FALLBACK_ROUTER_NAME);

    // 4. Chat through the router: primary 404s (retryable), secondary serves the reply.
    await chatPage.navigateToChat();
    await chatPage.selectModel(FALLBACK_ROUTER_NAME);
    await chatPage.sendMessage(OPENAI.chatQuestion);
    await chatPage.waitForResponseComplete();
    const reply = await chatPage.getLastAssistantMessage();
    expect(reply.trim().length).toBeGreaterThan(0);
  });
});
