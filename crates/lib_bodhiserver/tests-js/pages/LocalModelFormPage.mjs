import { BasePage } from '@/pages/BasePage.mjs';
import { expect } from '@playwright/test';

export class LocalModelFormPage extends BasePage {
  selectors = {
    aliasInput: '[data-testid="alias-input"]',
    repoInput: '[data-testid="repo-input"]',
    snapshotInput: '[data-testid="snapshot-input"]',
    quantTable: '[data-testid="quant-table"]',
    quantManual: '[data-testid="quant-manual"]',
    filenameInput: '[data-testid="filename-input"]',
    quantRow: (name) => `[data-testid="quant-row-${name}"]`,
    quantStatus: (name) => `[data-testid="quant-status-${name}"]`,
    quantDownloadNote: '[data-testid="quant-download-note"]',
    contextParamsTextarea: '[data-testid="context-params"]',
    systemPromptTextarea: '[data-testid="system-prompt"]',
    requestParamsTextarea: '[data-testid="request-params"]',
    submitButton: '[data-testid="submit-alias-form"]',
  };

  async waitForFormReady() {
    await this.waitForSelector(this.selectors.aliasInput);
    await this.waitForSPAReady();
  }

  async fillBasicInfo(alias, repo, filename, snapshot = null) {
    await this.fillTestId('alias-input', alias);

    if (repo) {
      await this.page.fill(this.selectors.repoInput, repo);
    }

    if (filename) {
      await this.selectQuantOrType(filename);
    }

    if (snapshot) {
      await this.page.fill(this.selectors.snapshotInput, snapshot);
    }
  }

  /**
   * Choose the GGUF file. Prefers the catalog quant row matching `filename`; if the repo has no
   * catalog quants (fallback manual input), types the filename directly.
   */
  async selectQuantOrType(filename) {
    const manual = this.page.locator(this.selectors.quantManual);
    const table = this.page.locator(this.selectors.quantTable);
    // Wait until either the quant table or the manual input has resolved.
    await expect(manual.or(table)).toBeVisible();

    if (await manual.isVisible()) {
      await this.page.fill(this.selectors.filenameInput, filename);
      return;
    }
    // Quant rows are keyed by quant name; match the row whose filename equals `filename`.
    const quantName = this.quantNameForFilename(filename);
    const row = this.page.locator(this.selectors.quantRow(quantName));
    await expect(row).toBeVisible();
    await row.click();
    await expect(row).toHaveAttribute('data-test-state', 'selected');
  }

  /** Best-effort quant-name from a GGUF filename (e.g. ...-Q4_K_M.gguf → Q4_K_M). */
  quantNameForFilename(filename) {
    const m = filename.match(/-([A-Za-z0-9_]+)\.gguf$/);
    return m ? m[1] : filename;
  }

  async selectQuantRow(quantName) {
    const row = this.page.locator(this.selectors.quantRow(quantName));
    await expect(row).toBeVisible();
    await row.click();
    await expect(row).toHaveAttribute('data-test-state', 'selected');
  }

  async getQuantStatus(quantName) {
    return (await this.page.locator(this.selectors.quantStatus(quantName)).textContent())?.trim();
  }

  /**
   * Select the first quant whose status reads "Not downloaded" and return its quant name. Returns
   * null when the catalog has no remote quant for the repo (every variant already on disk / no table)
   * — the caller skips the download assertion in that case.
   */
  async selectFirstRemoteQuant() {
    const table = this.page.locator(this.selectors.quantTable);
    if (!(await table.isVisible().catch(() => false))) return null;
    const rows = this.page.locator('[data-testid^="quant-row-"]');
    const count = await rows.count();
    for (let i = 0; i < count; i++) {
      const row = rows.nth(i);
      const testId = await row.getAttribute('data-testid');
      const quantName = testId.replace('quant-row-', '');
      const status = await this.getQuantStatus(quantName);
      if (status && /not downloaded/i.test(status)) {
        await row.click();
        await expect(row).toHaveAttribute('data-test-state', 'selected');
        return quantName;
      }
    }
    return null;
  }

  async fillContextParams(contextParams) {
    if (contextParams) {
      await this.page.fill(this.selectors.contextParamsTextarea, contextParams);
    }
  }

  /** Append a llama-server flag via the click-to-add catalog (e.g. '--flash-attn'). */
  async addContextFlag(flagKey) {
    await this.page.locator(`[data-testid="context-flag-add-${flagKey}"]`).click();
  }

  async fillSystemPrompt(prompt) {
    if (prompt) await this.page.fill(this.selectors.systemPromptTextarea, prompt);
  }

  /** Fill the request-params textarea from a {key: value} map as `key=value` lines. */
  async fillRequestParams(params = {}) {
    const lines = Object.entries(params).map(([k, v]) => `${k}=${Array.isArray(v) ? v.join(',') : v}`);
    if (lines.length) await this.page.fill(this.selectors.requestParamsTextarea, lines.join('\n'));
  }

  /** Append a request param via the click-to-add catalog (e.g. 'top_p'). */
  async addRequestParam(paramKey) {
    await this.page.locator(`[data-testid="request-param-add-${paramKey}"]`).click();
  }

  async createAlias() {
    const submitBtn = this.page.locator(this.selectors.submitButton);
    await expect(submitBtn).toBeVisible();
    await expect(submitBtn).toBeEnabled();
    await submitBtn.click();

    // Wait for navigation back to models list
    await this.waitForUrl('/ui/models/');
    await this.waitForSPAReady();
  }

  async updateAlias() {
    const submitBtn = this.page.locator(this.selectors.submitButton);
    await expect(submitBtn).toBeVisible();
    await expect(submitBtn).toBeEnabled();
    await expect(submitBtn).toContainText('Update');
    await submitBtn.click();

    // Wait for navigation back to models list
    await this.waitForUrl('/ui/models/');
    await this.waitForSPAReady();
  }

  async isEditMode() {
    const submitBtn = this.page.locator(this.selectors.submitButton);
    const buttonText = await submitBtn.textContent();
    return buttonText?.includes('Update');
  }

  async getFormData() {
    const alias = await this.page.locator(this.selectors.aliasInput).inputValue();
    const contextParams = await this.page.locator(this.selectors.contextParamsTextarea).inputValue();
    const repo = await this.page.locator(this.selectors.repoInput).inputValue();
    const snapshot = await this.page.locator(this.selectors.snapshotInput).inputValue();

    return { alias, repo, snapshot, contextParams };
  }
}
