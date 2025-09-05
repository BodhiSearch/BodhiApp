import { expect } from '@playwright/test';
import { BasePage } from './BasePage.mjs';

export class LoginPage extends BasePage {
  constructor(page, baseUrl, authServerConfig, testCredentials) {
    super(page, baseUrl);
    this.authServerConfig = authServerConfig;
    this.testCredentials = testCredentials;
  }
  
  selectors = {
    loginButton: 'button:has-text("Login")',
    usernameField: '#username',
    passwordField: '#password',
    signInButton: 'button:has-text("Sign In")'
  };
  
  async performOAuthLogin(expectedRedirectPath = '/ui/chat/') {
    // Navigate to login page if not already there
    if (!this.page.url().includes('/ui/login')) {
      await this.navigate('/ui/login');
    }
    
    // Click login button to initiate OAuth flow
    const loginButton = this.page.locator(this.selectors.loginButton);
    await loginButton.first().click();
    
    // Wait for redirect to auth server
    await this.page.waitForURL((url) => url.origin === this.authServerConfig.authUrl);
    
    // Fill in credentials on auth server
    await this.page.fill(this.selectors.usernameField, this.testCredentials.username);
    await this.page.fill(this.selectors.passwordField, this.testCredentials.password);
    
    // Submit and wait for redirect back to app
    await this.page.click(this.selectors.signInButton);
    
    // Wait for redirect back to app - allow flexible redirect path
    if (expectedRedirectPath) {
      await this.page.waitForURL((url) => 
        url.origin === this.baseUrl && url.pathname === expectedRedirectPath
      );
    } else {
      // Just wait for any redirect back to the app
      await this.page.waitForURL((url) => url.origin === this.baseUrl);
    }
    
    await this.waitForSPAReady();
  }
  
  async navigateToLogin() {
    await this.navigate('/ui/login');
  }
  
  async expectLoginPage() {
    await this.expectVisible(this.selectors.loginButton);
  }
  
  async expectLoginPageVisible() {
    const loginButton = this.page.locator(
      'button:has-text("Log In"), button:has-text("Login"), button:has-text("Sign In"), button[type="submit"]'
    );
    await expect(loginButton.first()).toBeVisible();
  }
  
  async clickLogin() {
    await this.page.click(this.selectors.loginButton);
  }
  
  async waitForAuthServer() {
    await this.page.waitForURL((url) => url.origin === this.authServerConfig.authUrl);
  }
  
  async fillCredentials(username = null, password = null) {
    const user = username || this.testCredentials.username;
    const pass = password || this.testCredentials.password;
    
    await this.page.fill(this.selectors.usernameField, user);
    await this.page.fill(this.selectors.passwordField, pass);
  }
  
  async submitLogin() {
    await this.page.click(this.selectors.signInButton);
  }
  
  async waitForSuccessfulLogin() {
    await this.page.waitForURL((url) => 
      url.origin === this.baseUrl && url.pathname === '/ui/chat/'
    );
  }
}