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
