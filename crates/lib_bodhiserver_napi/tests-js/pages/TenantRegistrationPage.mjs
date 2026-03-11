import { BasePage } from '@/pages/BasePage.mjs';

export class TenantRegistrationPage extends BasePage {
  /**
   * Fill the tenant registration form.
   */
  async fillTenantForm(name, description = '') {
    await this.fillTestId('tenant-name-input', name);
    if (description) {
      await this.fillTestId('tenant-description-input', description);
    }
  }

  /**
   * Submit the tenant creation form.
   */
  async submitTenantForm() {
    await this.clickTestId('create-tenant-button');
  }

  /**
   * Wait for tenant creation to complete (redirects to /ui/chat/).
   */
  async waitForCreated() {
    await this.page.waitForURL(
      (url) => url.origin === this.baseUrl && url.pathname === '/ui/chat/',
      { timeout: 30000 }
    );
    await this.waitForSPAReady();
  }
}
