import { SetupBasePage } from '@/pages/SetupBasePage.mjs';
import { expect } from '@playwright/test';

export class SetupDownloadModelsPage extends SetupBasePage {
  selectors = {
    ...this.selectors,
    chatModelsTitle: 'text=Chat Models',
    modelCard: '[data-testid="model-card"]',
    downloadButton: 'button:has-text("Download")',
    downloadingButton: 'button:has-text("Downloading")',
    downloadedButton: 'button:has-text("Downloaded")',
    progressBar: '[role="progressbar"]',
    // Model cards typically contain model names
    llamaModel: 'text=Llama',
    mistralModel: 'text=Mistral',
    gemmaModel: 'text=Gemma',
  };

  async navigateToDownloadModels() {
    await this.navigate('/ui/setup/download-models/');
    await this.waitForSetupPage();
  }

  async expectDownloadModelsPage() {
    await this.page.waitForURL((url) => url.pathname === '/ui/setup/download-models/');
    await this.expectVisible(this.selectors.chatModelsTitle);
    await this.expectStepIndicator(3);
    await this.expectRecommendedModelsDisplayed();
  }

  async expectRecommendedModelsDisplayed() {
    // Check that model cards are visible
    try {
      await expect(this.page.locator(this.selectors.modelCard).first()).toBeVisible({});
    } catch {
      // Fallback: look for model names or download buttons
      const hasModels = await Promise.race([
        this.page.locator(this.selectors.downloadButton).first().isVisible(),
        this.page.locator(this.selectors.llamaModel).first().isVisible(),
        this.page.locator(this.selectors.mistralModel).first().isVisible(),
      ]);
      expect(hasModels).toBeTruthy();
    }
  }

  async downloadModel(modelName = null) {
    let downloadButton;

    if (modelName) {
      // Find download button for specific model
      const modelCard = this.page.locator(`text=${modelName}`).locator('..').locator('..'); // Navigate up to card
      downloadButton = modelCard.locator(this.selectors.downloadButton);
    } else {
      // Download first available model
      downloadButton = this.page.locator(this.selectors.downloadButton).first();
    }

    await downloadButton.click();

    // Wait for download to start (button changes to "Downloading")
    await expect(this.page.locator(this.selectors.downloadingButton).first()).toBeVisible({});
  }

  async waitForDownloadComplete(timeout = 30000) {
    // Wait for at least one download to complete
    await expect(this.page.locator(this.selectors.downloadedButton).first()).toBeVisible({
      timeout,
    });
  }

  async skipDownloads() {
    // Continue without downloading models
    await this.clickContinue();
  }

  async continueAfterDownloads() {
    await this.clickContinue();
  }
}
