import { ApiModelFixtures } from '@/fixtures/apiModelFixtures.mjs';

/**
 * Register an API model via the UI form.
 * After calling, the browser is on the models list page.
 *
 * @param {ModelsListPage} modelsPage
 * @param {ApiModelFormPage} formPage
 * @param {string} apiKey - OpenAI API key
 * @param {object} [formatConfig] - Format config from ApiModelFixtures.API_FORMATS (defaults to openai)
 * @returns {{ modelId: string, modelName: string }}
 */
export async function registerApiModelViaUI(modelsPage, formPage, apiKey, formatConfig = null) {
  const config = formatConfig || ApiModelFixtures.API_FORMATS.openai;
  const modelData = ApiModelFixtures.createModelDataForFormat(
    Object.keys(ApiModelFixtures.API_FORMATS).find(
      (k) => ApiModelFixtures.API_FORMATS[k] === config
    ) || 'openai'
  );
  await modelsPage.navigateToModels();
  await modelsPage.clickNewApiModel();
  await formPage.form.waitForFormReady();
  await formPage.form.selectApiFormat(config.format);
  await formPage.form.fillBasicInfo(apiKey, modelData.baseUrl);
  await formPage.form.fetchAndSelectModels([config.model]);
  await formPage.form.testConnection();
  const modelId = await formPage.createModelAndCaptureId();
  return { modelId, modelName: config.model };
}
