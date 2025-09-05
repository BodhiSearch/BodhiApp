# UI Test Refactoring - Task Breakdown

## Phase 0: Vertical Migration - API Models Integration (Pilot) ✅ **COMPLETED**
**Goal: Demonstrate complete vertical migration by refactoring API models test as proof of concept**

**Target Completion: 3-4 days** ✅ **COMPLETED**

### Migration Results:
- ✅ Created complete Page Object Model structure (`BasePage`, `LoginPage`, `ModelsListPage`, `ApiModelFormPage`)
- ✅ Implemented test data factories (`ApiModelFixtures`)  
- ✅ Migrated all 5 test scenarios successfully
- ✅ Fixed button timing issue in test connection flow
- ✅ All tests passing (5/5) with same reliability as original
- ✅ Removed old test file after successful migration

### Current State Analysis: api-models-integration.spec.mjs

The existing test demonstrates:
- **Login Helper**: OAuth flow with auth server integration
- **API Model Creation**: Form-based workflow with real OpenAI API
- **Model Verification**: Table-based validation with data-testid selectors
- **CRUD Operations**: Create, edit, delete with confirmation dialogs
- **Responsive Testing**: Mobile (375px) and tablet (768px) viewports
- **Real Integration**: Uses actual OpenAI API key for testing

### Task 0.1: Create New Directory Structure for API Models
**Files to Create:**
- `crates/lib_bodhiserver_napi/tests-js/specs/core/api-models/` - API models tests
- `crates/lib_bodhiserver_napi/tests-js/pages/` - Page objects directory
- `crates/lib_bodhiserver_napi/tests-js/fixtures/` - Test data directory
- `crates/lib_bodhiserver_napi/tests-js/helpers/` - Enhanced utilities directory

**Actions:**
- [ ] Create `specs/core/api-models/` directory structure
- [ ] Create `pages/` directory for page objects
- [ ] Create `fixtures/` directory for test data
- [ ] Create `helpers/` directory for utilities
- [ ] **Verification:** Directory structure accessible

### Task 0.2: Implement Base Page Object Model
**File: `crates/lib_bodhiserver_napi/tests-js/pages/BasePage.mjs`**

**Implementation:**
```javascript
export class BasePage {
  constructor(page, baseUrl) {
    this.page = page;
    this.baseUrl = baseUrl;
  }
  
  async navigate(path) {
    await this.page.goto(`${this.baseUrl}${path}`);
    await this.waitForSPAReady();
  }
  
  async waitForSPAReady() {
    await this.page.waitForLoadState('domcontentloaded');
    await this.page.waitForTimeout(500);
  }
  
  async clickTestId(testId) {
    await this.page.click(`[data-testid="${testId}"]`);
  }
  
  async fillTestId(testId, value) {
    await this.page.fill(`[data-testid="${testId}"]`, value);
  }
  
  async expectVisible(selector) {
    await expect(this.page.locator(selector)).toBeVisible();
  }
  
  async waitForToast(message) {
    await expect(this.page.locator('[data-state="open"]')).toContainText(message);
  }
}
```

**Actions:**
- [ ] Implement BasePage class with core methods
- [ ] Add SPA ready waiting functionality
- [ ] Add data-testid interaction helpers
- [ ] Add toast notification helpers
- [ ] Add common assertion methods
- [ ] **Test:** Create simple test to validate BasePage

### Task 0.3: Create LoginPage Object
**File: `crates/lib_bodhiserver_napi/tests-js/pages/LoginPage.mjs`**

**Implementation:**
```javascript
import { expect } from '@playwright/test';
import { BasePage } from './BasePage.mjs';

export class LoginPage extends BasePage {
  constructor(page, baseUrl, authServerConfig, testCredentials) {
    super(page, baseUrl);
    this.authServerConfig = authServerConfig;
    this.testCredentials = testCredentials;
  }
  
  async performOAuthLogin() {
    await this.navigate('/ui/login');
    
    // Click login button to initiate OAuth flow
    const loginButton = this.page.locator('button:has-text("Login")');
    await loginButton.first().click();
    
    // Wait for redirect to auth server
    await this.page.waitForURL((url) => url.origin === this.authServerConfig.authUrl);
    
    // Fill credentials
    await this.page.fill('#username', this.testCredentials.username);
    await this.page.fill('#password', this.testCredentials.password);
    
    // Submit and wait for redirect back
    await this.page.click('button:has-text("Sign In")');
    await this.page.waitForURL((url) => 
      url.origin === this.baseUrl && url.pathname === '/ui/chat/'
    );
  }
}
```

**Actions:**
- [ ] Implement LoginPage extending BasePage
- [ ] Extract OAuth flow from existing test
- [ ] Handle auth server configuration
- [ ] Add error handling for login failures
- [ ] **Test:** Verify login functionality works

### Task 0.4: Create API Models Page Objects
**Files to Create:**
- `pages/ModelsListPage.mjs` - Models listing and management
- `pages/ApiModelFormPage.mjs` - Create/edit API model form

**File: `crates/lib_bodhiserver_napi/tests-js/pages/ModelsListPage.mjs`**
```javascript
import { expect } from '@playwright/test';
import { BasePage } from './BasePage.mjs';

export class ModelsListPage extends BasePage {
  selectors = {
    content: '[data-testid="models-content"]',
    table: '[data-testid="table-list-models"]',
    newApiModelButton: 'button:has-text("New API Model")',
    tableRow: (index = 'first') => `[data-testid="table-list-models"] tbody tr${index === 'first' ? '' : ':nth-child(' + index + ')'}`,
    aliasCell: (modelId) => `[data-testid="alias-cell-api_${modelId}"]`,
    repoCell: (modelId) => `[data-testid="repo-cell-api_${modelId}"]`,
    filenameCell: (modelId) => `[data-testid="filename-cell-api_${modelId}"]`,
    editButton: (modelId) => `[data-testid="edit-button-${modelId}"]:visible`,
    deleteButton: (modelId) => `[data-testid="delete-button-${modelId}"]:visible`,
    modelsDropdown: (modelId) => `[data-testid="models-dropdown-${modelId}"]`
  };
  
  async navigateToModels() {
    await this.navigate('/ui/models/');
    await this.page.waitForSelector(this.selectors.content);
  }
  
  async clickNewApiModel() {
    await this.expectVisible(this.selectors.newApiModelButton);
    await this.page.click(this.selectors.newApiModelButton);
    await this.page.waitForURL((url) => url.pathname === '/ui/api-models/new/');
  }
  
  async verifyApiModelInList(modelId, provider = 'OpenAI') {
    await this.page.waitForSelector(this.selectors.table);
    await this.page.waitForSelector(`${this.selectors.table} tbody tr`);
    
    const firstRow = this.page.locator(this.selectors.tableRow('first'));
    await expect(firstRow).toBeVisible();
    
    await expect(this.page.locator(this.selectors.aliasCell(modelId))).toContainText(modelId);
    await expect(this.page.locator(this.selectors.repoCell(modelId))).toContainText(provider);
    await expect(this.page.locator(this.selectors.filenameCell(modelId))).toContainText('https://api.openai.com/v1');
  }
  
  async editModel(modelId) {
    const editBtn = this.page.locator(this.selectors.editButton(modelId));
    await expect(editBtn).toBeVisible();
    await editBtn.click();
    await this.page.waitForURL((url) => url.pathname === '/ui/api-models/edit/');
  }
  
  async deleteModel(modelId) {
    const deleteBtn = this.page.locator(this.selectors.deleteButton(modelId));
    await expect(deleteBtn).toBeVisible();
    await deleteBtn.click();
    
    // Handle confirmation dialog
    await expect(this.page.locator('text=Delete API Model')).toBeVisible();
    await this.page.click('button:has-text("Delete")');
    await this.waitForToast(`API model ${modelId} deleted successfully`);
  }
}
```

**File: `crates/lib_bodhiserver_napi/tests-js/pages/ApiModelFormPage.mjs`**
```javascript
import { expect } from '@playwright/test';
import { BasePage } from './BasePage.mjs';

export class ApiModelFormPage extends BasePage {
  selectors = {
    modelIdInput: '[data-testid="api-model-id"]',
    providerSelect: '[data-testid="api-model-provider"]',
    baseUrlInput: '[data-testid="api-model-base-url"]',
    apiKeyInput: '[data-testid="api-model-api-key"]',
    fetchModelsButton: 'button:has-text("Fetch Models")',
    testConnectionButton: '[data-testid="test-connection-button"]',
    createButton: '[data-testid="create-api-model-button"]',
    updateButton: '[data-testid="update-api-model-button"]',
    modelOption: (model) => `.cursor-pointer:has-text("${model}")`
  };
  
  async fillBasicInfo(modelId, apiKey, baseUrl = 'https://api.openai.com/v1') {
    await this.fillTestId('api-model-id', modelId);
    await this.fillTestId('api-model-base-url', baseUrl);
    await this.fillTestId('api-model-api-key', apiKey);
  }
  
  async fetchAndSelectModels(models = ['gpt-4', 'gpt-3.5-turbo']) {
    // Fetch models from API
    await this.expectVisible(this.selectors.fetchModelsButton);
    await this.page.click(this.selectors.fetchModelsButton);
    
    // Wait for models to load
    await this.page.waitForSelector('text=gpt-4');
    
    // Select specified models
    for (const model of models) {
      await this.page.click(this.selectors.modelOption(model));
    }
  }
  
  async testConnection() {
    await this.page.click(this.selectors.testConnectionButton);
    await this.waitForToast(/Connection Test Successful/i);
  }
  
  async createModel() {
    await this.page.click(this.selectors.createButton);
    await this.page.waitForURL((url) => url.pathname === '/ui/models/');
  }
  
  async updateModel() {
    await this.page.click(this.selectors.updateButton);
    await this.page.waitForURL((url) => url.pathname === '/ui/models/');
  }
  
  async verifyFormPreFilled(modelId, provider = 'OpenAI', baseUrl = 'https://api.openai.com/v1') {
    await expect(this.page.locator(this.selectors.modelIdInput)).toHaveValue(modelId);
    await expect(this.page.locator(this.selectors.providerSelect)).toHaveText(provider);
    await expect(this.page.locator(this.selectors.baseUrlInput)).toHaveValue(baseUrl);
    await expect(this.page.locator(this.selectors.apiKeyInput)).toHaveValue(''); // masked
  }
}
```

**Actions:**
- [ ] Implement ModelsListPage with table interactions
- [ ] Implement ApiModelFormPage with form operations
- [ ] Extract all selectors to page objects
- [ ] Add responsive design helpers
- [ ] Add validation and error handling
- [ ] **Test:** Verify page objects work independently

### Task 0.5: Create Test Data Fixtures
**File: `crates/lib_bodhiserver_napi/tests-js/fixtures/apiModelFixtures.mjs`**

**Implementation:**
```javascript
export class ApiModelFixtures {
  static createModelData(overrides = {}) {
    const timestamp = Date.now();
    return {
      modelId: `test-model-${timestamp}`,
      provider: 'OpenAI',
      baseUrl: 'https://api.openai.com/v1',
      models: ['gpt-4', 'gpt-3.5-turbo'],
      ...overrides
    };
  }
  
  static createTestSuite(count = 3) {
    return Array.from({ length: count }, (_, i) => 
      this.createModelData({ modelId: `test-model-${Date.now()}-${i}` })
    );
  }
  
  static getRequiredEnvVars() {
    const apiKey = process.env.INTEG_TEST_OPENAI_API_KEY;
    if (!apiKey) {
      throw new Error('INTEG_TEST_OPENAI_API_KEY environment variable not set');
    }
    return { apiKey };
  }
}
```

**Actions:**
- [ ] Create test data factory for API models
- [ ] Add environment variable management
- [ ] Add test data cleanup utilities
- [ ] Add data validation helpers
- [ ] **Test:** Verify fixtures generate valid data

### Task 0.6: Migrate Tests to New Structure
**File: `crates/lib_bodhiserver_napi/tests-js/specs/core/api-models/api-models-integration.spec.mjs`**

**Implementation:**
```javascript
import { expect, test } from '@playwright/test';
import { createAuthServerTestClient, getAuthServerConfig, getTestCredentials } from '../../../playwright/auth-server-client.mjs';
import { createServerManager } from '../../../playwright/bodhi-app-server.mjs';
import { randomPort, waitForSPAReady } from '../../../test-helpers.mjs';
import { LoginPage } from '../../../pages/LoginPage.mjs';
import { ModelsListPage } from '../../../pages/ModelsListPage.mjs';
import { ApiModelFormPage } from '../../../pages/ApiModelFormPage.mjs';
import { ApiModelFixtures } from '../../../fixtures/apiModelFixtures.mjs';

test.describe('API Models Integration', () => {
  let serverManager;
  let baseUrl;
  let loginPage;
  let modelsPage;
  let formPage;
  let testData;

  test.beforeAll(async () => {
    // Verify environment setup
    const { apiKey } = ApiModelFixtures.getRequiredEnvVars();
    
    // Server setup (existing logic)
    const authServerConfig = getAuthServerConfig();
    const testCredentials = getTestCredentials();
    const port = randomPort();
    const serverUrl = `http://localhost:${port}`;

    const authClient = createAuthServerTestClient(authServerConfig);
    const resourceClient = await authClient.createResourceClient(serverUrl);
    await authClient.makeResourceAdmin(resourceClient.clientId, resourceClient.clientSecret, testCredentials.username);
    
    serverManager = createServerManager({
      appStatus: 'ready',
      authUrl: authServerConfig.authUrl,
      authRealm: authServerConfig.authRealm,
      clientId: resourceClient.clientId,
      clientSecret: resourceClient.clientSecret,
      port,
      host: 'localhost',
    });
    
    baseUrl = await serverManager.startServer();
    testData = { apiKey, authServerConfig, testCredentials };
  });

  test.beforeEach(async ({ page }) => {
    loginPage = new LoginPage(page, baseUrl, testData.authServerConfig, testData.testCredentials);
    modelsPage = new ModelsListPage(page, baseUrl);
    formPage = new ApiModelFormPage(page, baseUrl);
  });

  test.afterAll(async () => {
    if (serverManager) {
      await serverManager.stopServer();
    }
  });

  test('complete API model lifecycle', async ({ page }) => {
    const modelData = ApiModelFixtures.createModelData({ modelId: 'lifecycle-test' });
    
    // Step 1: Login
    await loginPage.performOAuthLogin();
    
    // Step 2: Navigate to create form
    await modelsPage.navigateToModels();
    await modelsPage.clickNewApiModel();
    
    // Step 3: Create API model
    await formPage.fillBasicInfo(modelData.modelId, testData.apiKey);
    await formPage.fetchAndSelectModels(modelData.models);
    await formPage.testConnection();
    await formPage.createModel();
    
    // Step 4: Verify in list
    await modelsPage.verifyApiModelInList(modelData.modelId);
    
    // Step 5: Test edit
    await modelsPage.editModel(modelData.modelId);
    await formPage.verifyFormPreFilled(modelData.modelId);
    await formPage.testConnection();
    await formPage.updateModel();
    
    // Step 6: Delete
    await modelsPage.deleteModel(modelData.modelId);
  });

  test.describe('responsive design', () => {
    test('mobile view interactions', async ({ page }) => {
      await page.setViewportSize({ width: 375, height: 667 });
      
      const modelData = ApiModelFixtures.createModelData({ modelId: 'mobile-test' });
      
      await loginPage.performOAuthLogin();
      await modelsPage.navigateToModels();
      await modelsPage.clickNewApiModel();
      
      await formPage.fillBasicInfo(modelData.modelId, testData.apiKey);
      await formPage.fetchAndSelectModels();
      await formPage.testConnection();
      await formPage.createModel();
      
      await modelsPage.verifyApiModelInList(modelData.modelId);
      
      // Mobile-specific interactions
      const dropdown = modelsPage.page.locator(modelsPage.selectors.modelsDropdown(modelData.modelId));
      await expect(dropdown.first()).toBeVisible();
      await dropdown.first().click();
      await expect(modelsPage.page.locator('[role="menuitem"]')).toHaveCount(2);
      
      // Cleanup
      await modelsPage.navigateToModels();
      await modelsPage.deleteModel(modelData.modelId);
    });
  });
});
```

**Actions:**
- [ ] Migrate main test using page objects
- [ ] Replace inline helpers with page object methods  
- [ ] Use fixture data instead of hardcoded values
- [ ] Maintain all existing test scenarios
- [ ] Add proper cleanup and error handling
- [ ] **Test:** Verify migrated test passes

### Task 0.7: Enhanced Test Configuration
**File: `crates/lib_bodhiserver_napi/tests-js/specs/playwright.config.mjs`** (if needed)

**Actions:**
- [ ] Configure test discovery for new specs directory
- [ ] Set up proper test isolation
- [ ] Configure parallel execution for specs
- [ ] Add test categorization (smoke, integration, etc.)
- [ ] **Test:** Verify test execution from new location

### Success Criteria for Phase 0
- [ ] **Structure**: Clean separation of concerns (pages, fixtures, specs)
- [ ] **Maintainability**: Easy to modify selectors and workflows
- [ ] **Reusability**: Page objects can be used across multiple tests
- [ ] **Reliability**: Tests pass consistently with same reliability as original
- [ ] **Coverage**: All original test scenarios preserved and working
- [ ] **Documentation**: Clear examples for future test migrations

### Migration Benefits Demonstrated
1. **Maintainability**: UI changes only require updates in page objects
2. **Reusability**: Login flow, form operations can be reused
3. **Readability**: Tests focus on business logic, not implementation details  
4. **Scalability**: Pattern established for migrating other pages
5. **Debugging**: Clear separation makes issue identification easier

## Phase 0.1: Vertical Migration - Setup Flow Integration ✅ **COMPLETED**
**Goal: Migrate setup flow test using established Page Object Model pattern**

**Target Completion: 1-2 days** ✅ **COMPLETED**

### Migration Results:
- ✅ Created comprehensive setup flow page objects:
  - `SetupBasePage` - Base setup page functionality
  - `SetupWelcomePage` - Welcome and server name setup
  - `SetupResourceAdminPage` - OAuth authentication flow
  - `SetupDownloadModelsPage` - Model download management
  - `SetupCompletePage` - Setup completion flow
- ✅ Implemented `SetupFixtures` with scenarios and validation
- ✅ Migrated all 5 setup test scenarios:
  - Complete setup flow from initial setup to chat page
  - Setup flow navigation between steps  
  - Server setup form validation
  - Step indicator verification throughout flow
  - Authentication error handling gracefully
- ✅ All tests covering OAuth integration, form validation, and multi-step navigation
- ✅ Removed old setup test file after successful migration

### Key Technical Achievements:
- **OAuth Flow Integration**: Complex authentication flow with auth server coordination
- **Multi-Step Navigation**: Step indicator verification and proper flow progression  
- **Form Validation**: Server name validation and error state handling
- **Error Resilience**: Graceful handling of authentication failures
- **Test Data Management**: Flexible setup scenarios with unique server names

## Phase 1: Test Infrastructure Foundation
**Goal: Establish Page Object Model and test organization structure**

**Target Completion: Week 1**

### Task 1.1: Create Directory Structure
**Files to Create:**
- `crates/lib_bodhiserver_napi/tests-js/specs/` - Main specs directory
- `crates/lib_bodhiserver_napi/tests-js/pages/` - Page objects directory
- `crates/lib_bodhiserver_napi/tests-js/fixtures/` - Test data directory
- `crates/lib_bodhiserver_napi/tests-js/helpers/` - Enhanced utilities directory

**Actions:**
- [ ] Create `specs/auth/` directory for authentication tests
- [ ] Create `specs/core/chat/` directory for chat tests
- [ ] Create `specs/core/models/` directory for model tests
- [ ] Create `specs/core/settings/` directory for settings tests
- [ ] Create `specs/admin/` directory for admin features
- [ ] Create `specs/setup/` directory for onboarding tests
- [ ] Create `specs/regression/` directory for edge cases
- [ ] **Test:** Verify directory structure is created and accessible

### Task 1.2: Implement Base Page Object Model
**File: `crates/lib_bodhiserver_napi/tests-js/pages/BasePage.mjs`**

**Actions:**
- [ ] Create `BasePage` class with constructor accepting Playwright page
- [ ] Implement `navigate(path)` method with SPA ready wait
- [ ] Implement `waitForSelector(selector)` with timeout handling
- [ ] Implement `clickTestId(testId)` using data-testid attributes
- [ ] Implement `fillTestId(testId, value)` for form inputs
- [ ] Implement `getTextContent(selector)` for assertions
- [ ] Implement `isVisible(selector)` for visibility checks
- [ ] Implement `takeScreenshot(name)` for visual debugging
- [ ] **Test:** Create unit test for BasePage methods

### Task 1.3: Create Page Objects for Core Pages
**Files to Create:**
- `pages/LoginPage.mjs`
- `pages/ChatPage.mjs`
- `pages/ModelsPage.mjs`
- `pages/SettingsPage.mjs`
- `pages/SetupPage.mjs`

**Actions:**
- [ ] LoginPage: Implement login form interaction methods
- [ ] LoginPage: Add OAuth flow handling methods
- [ ] ChatPage: Implement message sending methods
- [ ] ChatPage: Add model selection methods
- [ ] ChatPage: Add message history navigation
- [ ] ModelsPage: Implement model CRUD operations
- [ ] ModelsPage: Add filtering and sorting methods
- [ ] SettingsPage: Implement settings update methods
- [ ] SetupPage: Add setup flow navigation
- [ ] **Test:** Create example test using each page object

### Task 1.4: Test Data Factories
**File: `crates/lib_bodhiserver_napi/tests-js/fixtures/testData.mjs`**

**Actions:**
- [ ] Create `UserFactory` for test user generation
- [ ] Create `ModelFactory` for local and API models
- [ ] Create `MessageFactory` for chat messages
- [ ] Create `SettingsFactory` for configuration
- [ ] Create `TokenFactory` for API tokens
- [ ] Implement `cleanup()` methods for each factory
- [ ] Add randomization utilities for unique data
- [ ] **Test:** Unit tests for data factory methods

### Task 1.5: Enhanced Test Helpers
**File: `crates/lib_bodhiserver_napi/tests-js/helpers/enhanced-helpers.mjs`**

**Actions:**
- [ ] Create `waitForStreamingResponse()` for chat streaming
- [ ] Create `mockLLMResponse()` for testing without real LLM
- [ ] Create `interceptAPICall()` for API mocking
- [ ] Create `uploadFile()` helper for file operations
- [ ] Create `clearLocalStorage()` for test isolation
- [ ] Create `setAuthToken()` for authenticated requests
- [ ] Create `captureNetworkLogs()` for debugging
- [ ] **Test:** Integration test for helper functions

## Phase 2: Core Feature Tests
**Goal: Implement comprehensive tests for core functionality**

**Target Completion: Week 2**

### Task 2.1: Chat Interface Tests
**File: `crates/lib_bodhiserver_napi/tests-js/specs/core/chat/chat-basic.spec.mjs`**

**Actions:**
- [ ] Test: Send simple message and receive response
- [ ] Test: Select different models
- [ ] Test: Clear chat history
- [ ] Test: Start new chat conversation
- [ ] Test: Navigate between multiple chats
- [ ] Test: Search in chat history
- [ ] Test: Copy message content
- [ ] Test: Delete individual messages
- [ ] **Verification:** All tests pass with real backend

### Task 2.2: Chat Streaming Tests
**File: `crates/lib_bodhiserver_napi/tests-js/specs/core/chat/chat-streaming.spec.mjs`**

**Actions:**
- [ ] Test: Streaming response visualization
- [ ] Test: Stop streaming mid-response
- [ ] Test: Resume after stream interruption
- [ ] Test: Handle network disconnection during stream
- [ ] Test: Multiple concurrent streaming sessions
- [ ] Test: Stream timeout handling
- [ ] Test: Large response streaming
- [ ] **Verification:** Streaming behavior correct

### Task 2.3: Chat Settings Tests
**File: `crates/lib_bodhiserver_napi/tests-js/specs/core/chat/chat-settings.spec.mjs`**

**Actions:**
- [ ] Test: Adjust temperature setting
- [ ] Test: Modify max tokens
- [ ] Test: Set system prompt
- [ ] Test: Configure stop words
- [ ] Test: Toggle streaming on/off
- [ ] Test: Save and load chat preferences
- [ ] Test: Reset to default settings
- [ ] **Verification:** Settings persist correctly

### Task 2.4: Model Management Tests
**File: `crates/lib_bodhiserver_napi/tests-js/specs/core/models/model-crud.spec.mjs`**

**Actions:**
- [ ] Test: Create new local model alias
- [ ] Test: Edit existing model configuration
- [ ] Test: Delete model with confirmation
- [ ] Test: Duplicate model configuration
- [ ] Test: Search and filter models
- [ ] Test: Sort models by different criteria
- [ ] Test: Pagination with many models
- [ ] **Verification:** CRUD operations work correctly

### Task 2.5: API Model Configuration Tests
**File: `crates/lib_bodhiserver_napi/tests-js/specs/core/models/api-models.spec.mjs`**

**Actions:**
- [ ] Test: Add OpenAI API configuration
- [ ] Test: Test API connection
- [ ] Test: Select API models to enable
- [ ] Test: Update API key
- [ ] Test: Handle invalid API key
- [ ] Test: Switch between API providers
- [ ] Test: Delete API configuration
- [ ] **Verification:** API models integrate correctly

### Task 2.6: Settings Management Tests
**File: `crates/lib_bodhiserver_napi/tests-js/specs/core/settings/settings.spec.mjs`**

**Actions:**
- [ ] Test: View all settings
- [ ] Test: Update individual setting
- [ ] Test: Validate setting constraints
- [ ] Test: Reset setting to default
- [ ] Test: Export settings configuration
- [ ] Test: Import settings from file
- [ ] Test: Handle invalid setting values
- [ ] **Verification:** Settings management robust

## Phase 3: User Journey Tests
**Goal: Test complete user workflows end-to-end**

**Target Completion: Week 3**

### Task 3.1: First-Time User Journey
**File: `crates/lib_bodhiserver_napi/tests-js/specs/journeys/first-time-user.spec.mjs`**

**Actions:**
- [ ] Test: Complete setup wizard
- [ ] Test: Configure first model
- [ ] Test: Send first chat message
- [ ] Test: Explore UI features
- [ ] Test: Access help documentation
- [ ] Test: Configure preferences
- [ ] Test: Complete onboarding
- [ ] **Verification:** Smooth first-time experience

### Task 3.2: Power User Workflow
**File: `crates/lib_bodhiserver_napi/tests-js/specs/journeys/power-user.spec.mjs`**

**Actions:**
- [ ] Test: Manage multiple model configurations
- [ ] Test: Switch between models rapidly
- [ ] Test: Use keyboard shortcuts
- [ ] Test: Bulk operations on models
- [ ] Test: Advanced chat features
- [ ] Test: Custom system prompts
- [ ] Test: Performance with many chats
- [ ] **Verification:** Efficient power user features

### Task 3.3: API Developer Journey
**File: `crates/lib_bodhiserver_napi/tests-js/specs/journeys/api-developer.spec.mjs`**

**Actions:**
- [ ] Test: Generate API token
- [ ] Test: Test API endpoints
- [ ] Test: Revoke and regenerate tokens
- [ ] Test: Monitor API usage
- [ ] Test: Configure rate limits
- [ ] Test: Access API documentation
- [ ] Test: Handle API errors
- [ ] **Verification:** Complete API workflow

### Task 3.4: Admin User Journey
**File: `crates/lib_bodhiserver_napi/tests-js/specs/journeys/admin-user.spec.mjs`**

**Actions:**
- [ ] Test: Manage user accounts
- [ ] Test: View system statistics
- [ ] Test: Configure global settings
- [ ] Test: Monitor system health
- [ ] Test: Manage permissions
- [ ] Test: Audit user actions
- [ ] Test: System maintenance tasks
- [ ] **Verification:** Admin capabilities complete

## Phase 4: Quality Assurance Tests
**Goal: Ensure accessibility, performance, and visual quality**

**Target Completion: Week 4**

### Task 4.1: Accessibility Tests
**File: `crates/lib_bodhiserver_napi/tests-js/specs/quality/accessibility.spec.mjs`**

**Actions:**
- [ ] Test: Keyboard navigation throughout app
- [ ] Test: Screen reader compatibility
- [ ] Test: ARIA labels and roles
- [ ] Test: Color contrast compliance
- [ ] Test: Focus management
- [ ] Test: Skip navigation links
- [ ] Test: Form field labels
- [ ] **Verification:** WCAG 2.1 AA compliance

### Task 4.2: Responsive Design Tests
**File: `crates/lib_bodhiserver_napi/tests-js/specs/quality/responsive.spec.mjs`**

**Actions:**
- [ ] Test: Mobile viewport (360px)
- [ ] Test: Tablet viewport (768px)
- [ ] Test: Desktop viewport (1920px)
- [ ] Test: Portrait/landscape orientation
- [ ] Test: Touch interactions on mobile
- [ ] Test: Responsive navigation menu
- [ ] Test: Adaptive layouts
- [ ] **Verification:** Responsive on all devices

### Task 4.3: Performance Tests
**File: `crates/lib_bodhiserver_napi/tests-js/specs/quality/performance.spec.mjs`**

**Actions:**
- [ ] Test: Page load time metrics
- [ ] Test: Time to interactive (TTI)
- [ ] Test: First contentful paint (FCP)
- [ ] Test: Memory usage over time
- [ ] Test: Performance with 100+ models
- [ ] Test: Performance with 1000+ messages
- [ ] Test: Concurrent user simulation
- [ ] **Verification:** Performance within targets

### Task 4.4: Visual Regression Tests
**File: `crates/lib_bodhiserver_napi/tests-js/specs/quality/visual.spec.mjs`**

**Actions:**
- [ ] Setup: Configure screenshot comparison
- [ ] Test: Capture baseline screenshots
- [ ] Test: Compare layout changes
- [ ] Test: Dark/light theme consistency
- [ ] Test: Component visual states
- [ ] Test: Error state appearances
- [ ] Test: Loading state animations
- [ ] **Verification:** Visual consistency maintained

### Task 4.5: Error Handling Tests
**File: `crates/lib_bodhiserver_napi/tests-js/specs/quality/error-handling.spec.mjs`**

**Actions:**
- [ ] Test: Network failure recovery
- [ ] Test: Invalid input handling
- [ ] Test: Session timeout handling
- [ ] Test: Rate limit exceeded
- [ ] Test: Server error responses
- [ ] Test: Malformed data handling
- [ ] Test: Graceful degradation
- [ ] **Verification:** All errors handled gracefully

## Phase 5: Test Infrastructure Enhancement
**Goal: Optimize test execution and maintenance**

**Target Completion: Week 5**

### Task 5.1: Parallel Execution Setup
**File: `playwright.config.mjs` (modifications)**

**Actions:**
- [ ] Configure dynamic port allocation
- [ ] Enable parallel test execution
- [ ] Set up test sharding
- [ ] Configure worker allocation
- [ ] Implement test isolation
- [ ] Add retry mechanisms
- [ ] Configure test timeouts
- [ ] **Verification:** Tests run in parallel successfully

### Task 5.2: CI/CD Integration
**Files to Create:**
- `.github/workflows/ui-tests.yml`
- `scripts/run-test-suite.sh`

**Actions:**
- [ ] Create smoke test suite configuration
- [ ] Create regression test suite
- [ ] Create full test suite
- [ ] Configure test result reporting
- [ ] Set up artifact collection
- [ ] Configure failure notifications
- [ ] Add performance tracking
- [ ] **Verification:** CI/CD pipeline functional

### Task 5.3: Test Documentation
**Files to Create:**
- `tests-js/README.md`
- `tests-js/CONTRIBUTING.md`
- `tests-js/PATTERNS.md`

**Actions:**
- [ ] Document test structure
- [ ] Document page object patterns
- [ ] Document test data management
- [ ] Document debugging procedures
- [ ] Create troubleshooting guide
- [ ] Add example test templates
- [ ] Document best practices
- [ ] **Verification:** Documentation complete and clear

### Task 5.4: Migration of Existing Tests
**Files to Migrate:**
- All files in `tests-js/playwright/*.spec.mjs`

**Actions:**
- [ ] Migrate auth-flow-integration tests
- [ ] Migrate api-models-integration tests
- [ ] Migrate setup flow tests
- [ ] Refactor to use Page Objects
- [ ] Remove duplicate code
- [ ] Update test configurations
- [ ] Verify migrated tests pass
- [ ] **Verification:** All tests migrated successfully

### Task 5.5: Cross-Browser Testing
**File: `playwright.config.mjs` (enhancements)**

**Actions:**
- [ ] Enable Firefox testing
- [ ] Enable WebKit testing (fix bus errors)
- [ ] Configure browser-specific settings
- [ ] Handle browser-specific behaviors
- [ ] Test browser compatibility
- [ ] Document browser issues
- [ ] Create browser matrix
- [ ] **Verification:** Tests pass on all browsers

## Phase 6: Continuous Improvement
**Goal: Establish patterns for ongoing test development**

**Target: Ongoing**

### Task 6.1: Test Metrics and Monitoring
**Actions:**
- [ ] Set up test execution metrics
- [ ] Track test reliability scores
- [ ] Monitor test execution times
- [ ] Identify flaky tests
- [ ] Track code coverage
- [ ] Generate test reports
- [ ] Create dashboards

### Task 6.2: Test Maintenance Process
**Actions:**
- [ ] Establish test review process
- [ ] Create test update procedures
- [ ] Document breaking changes
- [ ] Set up automated alerts
- [ ] Create maintenance schedule
- [ ] Track technical debt
- [ ] Plan refactoring cycles

### Task 6.3: Advanced Testing Capabilities
**Actions:**
- [ ] Explore AI-powered testing
- [ ] Implement chaos testing
- [ ] Add security testing
- [ ] Create load testing
- [ ] Add mutation testing
- [ ] Explore property-based testing
- [ ] Implement contract testing

## Success Criteria

### Phase 1 Success
- [ ] Page Object Model fully implemented
- [ ] Test data factories operational
- [ ] Directory structure established
- [ ] Base infrastructure tested

### Phase 2 Success
- [ ] Core features have >80% coverage
- [ ] All CRUD operations tested
- [ ] Settings management verified
- [ ] Chat functionality comprehensive

### Phase 3 Success
- [ ] Critical user journeys tested
- [ ] End-to-end workflows verified
- [ ] User personas covered
- [ ] Journey documentation complete

### Phase 4 Success
- [ ] Accessibility compliance verified
- [ ] Performance benchmarks met
- [ ] Visual regression baselines set
- [ ] Quality gates established

### Phase 5 Success
- [ ] Parallel execution working
- [ ] CI/CD pipeline operational
- [ ] Documentation comprehensive
- [ ] Migration completed

## Risk Management

### High Risk Items
- [ ] WebKit browser compatibility issues
- [ ] Test flakiness in streaming tests
- [ ] Port conflict in parallel execution
- [ ] LLM dependency for chat tests

### Mitigation Strategies
- [ ] Start with single browser support
- [ ] Implement robust wait strategies
- [ ] Use dynamic port allocation
- [ ] Create mock LLM responses

## Notes

- Each task should produce a working test file
- Tests should be reviewed before merging
- Documentation should be updated with each phase
- Performance metrics should be tracked continuously
- Flaky tests should be fixed immediately or disabled