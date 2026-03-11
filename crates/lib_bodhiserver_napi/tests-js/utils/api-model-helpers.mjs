import { ApiModelFixtures } from '@/fixtures/apiModelFixtures.mjs';

/**
 * Register an API model via the UI form.
 * After calling, the browser is on the models list page.
 *
 * @param {ModelsListPage} modelsPage
 * @param {ApiModelFormPage} formPage
 * @param {string} apiKey - OpenAI API key
 * @returns {{ modelId: string, modelName: string }}
 */
export async function registerApiModelViaUI(modelsPage, formPage, apiKey) {
  const modelData = ApiModelFixtures.createModelData();
  await modelsPage.navigateToModels();
  await modelsPage.clickNewApiModel();
  await formPage.form.waitForFormReady();
  await formPage.form.fillBasicInfo(apiKey, modelData.baseUrl);
  await formPage.form.fetchAndSelectModels([ApiModelFixtures.OPENAI_MODEL]);
  await formPage.form.testConnection();
  const modelId = await formPage.createModelAndCaptureId();
  return { modelId, modelName: ApiModelFixtures.OPENAI_MODEL };
}
