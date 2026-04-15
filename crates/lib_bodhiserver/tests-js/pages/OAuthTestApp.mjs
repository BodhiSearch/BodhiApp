import { ConfigSection } from '@/pages/sections/ConfigSection.mjs';
import { AccessCallbackSection } from '@/pages/sections/AccessCallbackSection.mjs';
import { OAuthSection } from '@/pages/sections/OAuthSection.mjs';
import { DashboardPage } from '@/pages/test-app/DashboardPage.mjs';
import { ChatPage } from '@/pages/test-app/ChatPage.mjs';
import { RESTPage } from '@/pages/test-app/RESTPage.mjs';

export class OAuthTestApp {
  constructor(page, baseUrl) {
    this.page = page;
    this.baseUrl = baseUrl;
    this.config = new ConfigSection(page);
    this.accessCallback = new AccessCallbackSection(page);
    this.oauth = new OAuthSection(page);
    this.dashboard = new DashboardPage(page);
    this.chat = new ChatPage(page);
    this.rest = new RESTPage(page);
  }

  async navigate() {
    await this.page.goto(this.baseUrl);
    await this.page.waitForLoadState('domcontentloaded');
  }

  async expectLoggedIn() {
    await this.page.locator('[data-testid="header-user-email"]').waitFor();
    await this.page.locator('[data-testid="btn-header-logout"]').waitFor();
  }

  async getHeaderEmail() {
    return await this.page.locator('[data-testid="header-user-email"]').textContent();
  }

  async logout() {
    await this.page.click('[data-testid="btn-header-logout"]');
  }
}
