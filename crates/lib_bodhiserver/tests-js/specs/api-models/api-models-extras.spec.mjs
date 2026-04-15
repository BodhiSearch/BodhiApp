import { ApiModelFixtures } from '@/fixtures/apiModelFixtures.mjs';
import { ApiModelFormPage } from '@/pages/ApiModelFormPage.mjs';
import { LoginPage } from '@/pages/LoginPage.mjs';
import { ModelsListPage } from '@/pages/ModelsListPage.mjs';
import { getAuthServerConfig, getTestCredentials } from '@/utils/auth-server-client.mjs';
import { expect, test } from '@/fixtures.mjs';

// Tests for the extra_headers / extra_body JSON editor fields in the API model form.
// Uses real Anthropic OAuth credentials from .env.test — only fetch-models is
// exercised against the live API (no chargeable completions).

const ANTHROPIC_BASE_URL = 'https://api.anthropic.com/v1';
const REAL_MODEL_ID = 'claude-3-haiku-20240307';
const ANTHROPIC_OAUTH_FORMAT = ApiModelFixtures.API_FORMATS.anthropic_oauth;

test.describe('API Models - Extras Editor (extra_headers / extra_body)', () => {
  let authServerConfig;
  let testCredentials;
  let anthropicOAuthToken;

  test.beforeAll(async () => {
    authServerConfig = getAuthServerConfig();
    testCredentials = getTestCredentials();
    anthropicOAuthToken = process.env[ANTHROPIC_OAUTH_FORMAT.envKey];
    if (!anthropicOAuthToken) {
      throw new Error(
        `${ANTHROPIC_OAUTH_FORMAT.envKey} missing in .env.test — required for api-models-extras spec`
      );
    }
  });

  let loginPage;
  let modelsPage;
  let formPage;

  test.beforeEach(async ({ page, sharedServerUrl }) => {
    loginPage = new LoginPage(page, sharedServerUrl, authServerConfig, testCredentials);
    modelsPage = new ModelsListPage(page, sharedServerUrl);
    formPage = new ApiModelFormPage(page, sharedServerUrl);
  });

  test('anthropic_oauth format shows and pre-fills extras; openai format hides extras', async () => {
    await test.step('Phase 1: login and navigate to new API model form', async () => {
      await loginPage.performOAuthLogin();
      await modelsPage.navigateToModels();
      await modelsPage.clickNewApiModel();
      await formPage.form.waitForFormReady();
    });

    await test.step('Phase 2: selecting openai format does not show extras fields', async () => {
      await formPage.form.selectApiFormat('openai');
      await formPage.form.expectExtrasVisible(false);
    });

    await test.step('Phase 3: selecting anthropic_oauth shows and pre-fills extras', async () => {
      await formPage.form.selectApiFormat('anthropic_oauth');
      await formPage.form.expectExtrasVisible(true);
      await formPage.form.expectExtrasPrefilledFor(ANTHROPIC_OAUTH_FORMAT);
    });

    await test.step('Phase 4: switching back to openai hides extras again', async () => {
      await formPage.form.selectApiFormat('openai');
      await formPage.form.expectExtrasVisible(false);
    });
  });

  test('malformed JSON in extra_headers shows validation error after submit attempt', async ({
    page,
  }) => {
    await test.step('Phase 1: login and navigate to new API model form', async () => {
      await loginPage.performOAuthLogin();
      await modelsPage.navigateToModels();
      await modelsPage.clickNewApiModel();
      await formPage.form.waitForFormReady();
    });

    await test.step('Phase 2: select anthropic_oauth, fill credentials, fetch and select a model', async () => {
      await formPage.form.selectApiFormat('anthropic_oauth');
      await formPage.form.fillBaseUrl(ANTHROPIC_BASE_URL);
      await formPage.form.fillApiKey(anthropicOAuthToken);
      await formPage.form.expectExtrasVisible(true);
      await formPage.form.clickFetchModels();
      await formPage.form.expectFetchSuccess();
      await formPage.form.expectAtLeastOneModelFetched();
      await formPage.form.searchAndSelectModel(REAL_MODEL_ID);
    });

    await test.step('Phase 3: overwrite extra_headers with malformed JSON and attempt submit', async () => {
      await formPage.form.fillExtraHeaders('{ invalid json }');
      await page.click(formPage.form.selectors.createButton);
      await formPage.form.expectExtraHeadersError('must be valid JSON');
    });
  });

  test('malformed JSON in extra_body shows validation error after submit attempt', async ({
    page,
  }) => {
    await test.step('Phase 1: login and navigate to new API model form', async () => {
      await loginPage.performOAuthLogin();
      await modelsPage.navigateToModels();
      await modelsPage.clickNewApiModel();
      await formPage.form.waitForFormReady();
    });

    await test.step('Phase 2: select anthropic_oauth, fill credentials, fetch and select a model', async () => {
      await formPage.form.selectApiFormat('anthropic_oauth');
      await formPage.form.fillBaseUrl(ANTHROPIC_BASE_URL);
      await formPage.form.fillApiKey(anthropicOAuthToken);
      await formPage.form.expectExtrasVisible(true);
      await formPage.form.clickFetchModels();
      await formPage.form.expectFetchSuccess();
      await formPage.form.expectAtLeastOneModelFetched();
      await formPage.form.searchAndSelectModel(REAL_MODEL_ID);
    });

    await test.step('Phase 3: overwrite extra_body with malformed JSON and attempt submit', async () => {
      await formPage.form.fillExtraBody('not-valid-json');
      await page.click(formPage.form.selectors.createButton);
      await formPage.form.expectExtraBodyError('must be valid JSON');
    });
  });

  test('anthropic_oauth create/edit round-trip: extras persist after save', async () => {
    let modelId;

    await test.step('Phase 1: login and navigate to new API model form', async () => {
      await loginPage.performOAuthLogin();
      await modelsPage.navigateToModels();
      await modelsPage.clickNewApiModel();
      await formPage.form.waitForFormReady();
    });

    await test.step('Phase 2: fill form with anthropic_oauth and pre-filled extras', async () => {
      await formPage.form.selectApiFormat('anthropic_oauth');
      await formPage.form.fillBaseUrl(ANTHROPIC_BASE_URL);
      await formPage.form.fillApiKey(anthropicOAuthToken);
      await formPage.form.expectExtrasVisible(true);
      await formPage.form.expectExtrasPrefilledFor(ANTHROPIC_OAUTH_FORMAT);
      await formPage.form.clickFetchModels();
      await formPage.form.expectFetchSuccess();
      await formPage.form.expectAtLeastOneModelFetched();
      await formPage.form.searchAndSelectModel(REAL_MODEL_ID);
    });

    await test.step('Phase 3: create model and verify it appears in list', async () => {
      modelId = await formPage.createModelAndCaptureId();
      await modelsPage.navigateToModels();
      await modelsPage.verifyApiModelInList(modelId, 'anthropic_oauth', ANTHROPIC_BASE_URL);
    });

    await test.step('Phase 4: edit model and verify extras are still pre-filled', async () => {
      await modelsPage.editModel(modelId);
      await formPage.form.waitForFormReady();
      await formPage.form.expectExtrasVisible(true);
      await formPage.form.expectExtrasPrefilledFor(ANTHROPIC_OAUTH_FORMAT);
    });

    await test.step('Phase 5: cleanup - delete model', async () => {
      await modelsPage.navigateToModels();
      await modelsPage.deleteModel(modelId);
    });
  });

  // Backend performs a metadata fetch against upstream on create/update for
  // anthropic_oauth, which requires the OAuth beta headers. Clearing extras
  // makes that fetch return 401 and save is rejected — the UI stays on the
  // edit page and surfaces an error toast.
  test('clearing anthropic_oauth extras causes backend metadata fetch to fail on save', async ({
    page,
  }) => {
    let modelId;

    await test.step('Phase 1: create anthropic_oauth model with valid pre-filled extras', async () => {
      await loginPage.performOAuthLogin();
      await modelsPage.navigateToModels();
      await modelsPage.clickNewApiModel();
      await formPage.form.waitForFormReady();
      await formPage.form.selectApiFormat('anthropic_oauth');
      await formPage.form.fillBaseUrl(ANTHROPIC_BASE_URL);
      await formPage.form.fillApiKey(anthropicOAuthToken);
      await formPage.form.clickFetchModels();
      await formPage.form.expectFetchSuccess();
      await formPage.form.expectAtLeastOneModelFetched();
      await formPage.form.searchAndSelectModel(REAL_MODEL_ID);
      modelId = await formPage.createModelAndCaptureId();
    });

    await test.step('Phase 2: edit, clear extras, attempt update — expect failure toast', async () => {
      await modelsPage.navigateToModels();
      await modelsPage.editModel(modelId);
      await formPage.form.waitForFormReady();
      await formPage.form.expectExtrasVisible(true);
      await formPage.form.fillExtraHeaders('');
      await formPage.form.fillExtraBody('');
      await page.click(formPage.form.selectors.updateButton);
      await expect(
        page.locator('[data-state="open"]:has-text("Failed to Update API Model")')
      ).toBeVisible();
      await expect(page).toHaveURL(/\/ui\/models\/api\/edit/);
    });

    await test.step('Phase 3: cleanup - delete model', async () => {
      await modelsPage.navigateToModels();
      await modelsPage.deleteModel(modelId);
    });
  });
});
