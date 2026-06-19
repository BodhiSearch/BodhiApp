import { expect } from '@playwright/test';

import { BasePage } from './BasePage.mjs';

/**
 * Page object for the Model Router create/edit form (`/ui/models/router/new` and `/edit`).
 * Black-box: drives the UI only via data-testid attributes + ARIA roles.
 *
 * The V2 rebuild replaced the plain alias <Select> with a searchable cmdk combobox and the
 * step rows with richer step cards. data-testids are preserved verbatim; the combobox options
 * expose their accessible name = the alias identity, so option selection is unchanged.
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

  /**
   * Select the referenced alias for target `idx` from the cmdk combobox. Opens the popover
   * (trigger keeps `target-alias-${idx}`), types the identity into the focused search to filter,
   * then clicks the matching option scoped to THIS popover's listbox. Typing+scoping avoids the
   * strict-mode ambiguity when multiple step cards each render the same alias as an option.
   */
  async selectTargetAlias(idx, aliasIdentity) {
    await this.clickTestId(`target-alias-${idx}`);
    // Scope everything to the popover that just opened ([data-state="open"]); other step cards'
    // comboboxes may still be mounted, so a page-wide role/placeholder query is ambiguous.
    const openPopover = this.page
      .locator('[data-radix-popper-content-wrapper] [data-state="open"]')
      .last();
    await openPopover.getByPlaceholder('Search aliases…').fill(aliasIdentity);
    await openPopover.getByRole('option', { name: aliasIdentity, exact: true }).first().click();
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

  /** Toggle a step's enabled switch (shadcn Switch → `aria-checked`). */
  async setTargetEnabled(idx, enabled) {
    const sw = this.page.locator(`[data-testid="target-enabled-${idx}"]`);
    const checked = (await sw.getAttribute('aria-checked')) === 'true';
    if (checked !== enabled) {
      await sw.click();
    }
  }

  async moveTargetUp(idx) {
    await this.clickTestId(`target-up-${idx}`);
  }

  async moveTargetDown(idx) {
    await this.clickTestId(`target-down-${idx}`);
  }

  async removeTarget(idx) {
    await this.clickTestId(`target-remove-${idx}`);
  }

  async submit() {
    await this.clickTestId('router-submit');
  }

  /** Assert the free-text model input value for a forward-all target. */
  async expectModelOnForm(idx, expectedModel) {
    await expect(this.page.locator(`[data-testid="target-model-${idx}"]`)).toHaveValue(
      expectedModel
    );
  }
}
