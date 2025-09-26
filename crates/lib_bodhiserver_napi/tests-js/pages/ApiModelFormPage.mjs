import { BasePage } from './BasePage.mjs';
import { ApiModelFormComponent } from './components/ApiModelFormComponent.mjs';

/**
 * Page object for API Model form operations on /ui/models pages
 * Uses composition with ApiModelFormComponent to eliminate duplication
 */
export class ApiModelFormPage extends BasePage {
  constructor(page, baseUrl) {
    super(page, baseUrl);
    this.form = new ApiModelFormComponent(page);
  }

  // Page-specific methods for /ui/models pages (not setup)
  async createModel() {
    await this.page.click(this.form.selectors.createButton);
    await this.waitForToastOptional(/Successfully created/);
    await this.waitForToastToHideOptional();
    await this.waitForUrl('/ui/models/');
    await this.waitForSPAReady();
  }

  async createModelAndCaptureId() {
    await this.page.click(this.form.selectors.createButton);

    // Try to capture the generated ID from the success toast, but don't fail if toast is flaky
    let generatedId = null;
    try {
      generatedId = await this.form.waitForToastAndExtractId(/Successfully created API model/i);
    } catch (error) {
      console.log('Toast ID extraction failed, will use fallback method');
    }

    await this.form.waitForToastToHideOptional();

    // Navigate to models page
    await this.waitForUrl('/ui/models/');
    await this.waitForSPAReady();

    // If we couldn't get ID from toast, get it from the latest model
    if (!generatedId) {
      console.log('Using fallback: getting ID from latest model in list');
      const modelsPage = new (await import('./ModelsListPage.mjs')).ModelsListPage(
        this.page,
        this.baseUrl
      );
      const latestModel = await modelsPage.getLatestModel();
      generatedId = await modelsPage.getModelIdFromRow(latestModel);
    }

    return generatedId;
  }

  async updateModel() {
    await this.page.click(this.form.selectors.updateButton);
    await this.waitForToastOptional(/Successfully updated/);
    await this.waitForToastToHideOptional();
    await this.waitForUrl('/ui/models/');
    await this.waitForSPAReady();
  }

  async expectToBeOnCreatePage() {
    await this.expectToBeOnPage('/ui/models/create');
  }

  async expectToBeOnEditPage(modelId) {
    await this.expectToBeOnPage(`/ui/models/${modelId}/edit`);
  }

  async expectToBeOnModelsListPage() {
    await this.expectToBeOnPage('/ui/models/');
  }
}
