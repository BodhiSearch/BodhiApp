import { expect } from '@playwright/test';

import { BasePage } from './BasePage.mjs';

/**
 * Page object for the model-router create/edit form (`/ui/models/router/new` and `/edit`).
 * Black-box: drives the UI only via data-testid attributes.
 */
export class ModelRouterFormPage extends BasePage {
  constructor(page, baseUrl) {
    super(page, baseUrl);
  }

  async navigateToNew() {
    await this.navigate('/ui/models/router/new/');
    await this.page.waitForSelector('[data-testid="model-router-form"]');
    await this.waitForSPAReady();
  }

  async waitForFormReady() {
    await this.page.waitForSelector('[data-testid="model-router-form"]');
  }

  async fillName(alias) {
    await this.fillTestId('router-alias-input', alias);
  }

  async addTarget() {
    await this.clickTestId('add-target');
  }

  /** Select the referenced alias for target `idx` from the Radix combobox. */
  async selectTargetAlias(idx, aliasIdentity) {
    await this.clickTestId(`target-alias-${idx}`);
    await this.page.getByRole('option', { name: aliasIdentity, exact: true }).click();
  }

  /** Select a pinned model from the dropdown for a selected-subset API target. */
  async selectTargetModel(idx, modelName) {
    await this.clickTestId(`target-model-${idx}`);
    await this.page.getByRole('option', { name: modelName, exact: true }).click();
  }

  /** Type a free-text pinned model for a forward-all API target. */
  async fillTargetModel(idx, modelName) {
    await this.fillTestId(`target-model-${idx}`, modelName);
  }

  async setTargetEnabled(idx, enabled) {
    const sw = this.page.locator(`[data-testid="target-enabled-${idx}"]`);
    const checked = (await sw.getAttribute('aria-checked')) === 'true';
    if (checked !== enabled) {
      await sw.click();
    }
  }

  async submit() {
    await this.clickTestId('router-submit');
  }

  async expectModelOnForm(idx, expectedModel) {
    await expect(this.page.locator(`[data-testid="target-model-${idx}"]`)).toHaveValue(expectedModel);
  }
}
