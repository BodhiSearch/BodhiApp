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

## Phase 1: Vertical Migration - Chat Integration ✅ **NEXT PRIORITY**
**Goal: Complete chat functionality end-to-end migration using established patterns**

**Target Completion: 3-4 days**

### Migration Target: Chat Feature Test Suite
**Covers: Message sending, streaming responses, model selection, chat history, conversation management**

Based on existing test files:
- Current tests: None found - greenfield development opportunity
- New structure: `crates/lib_bodhiserver_napi/tests-js/specs/core/chat/chat.spec.mjs`

### Task 1.1: Create Chat Page Objects (Vertical Pattern)
**Files to Create using battle-tested patterns:**

**File: `pages/ChatPage.mjs`** - Main chat interface
```javascript
export class ChatPage extends BasePage {
  selectors = {
    chatContainer: '[data-testid="chat-container"]',
    messageInput: '[data-testid="message-input"]', 
    sendButton: '[data-testid="send-button"]',
    modelSelector: '[data-testid="model-selector"]',
    messageHistory: '[data-testid="message-history"]',
    newChatButton: '[data-testid="new-chat-button"]',
    streamingResponse: '[data-testid="streaming-response"]'
  };
  
  async sendMessage(message) {
    await this.fillTestId('message-input', message);
    await this.clickTestId('send-button');
  }
  
  async waitForResponse(expectedContent) {
    await expect(this.page.locator(this.selectors.messageHistory).last())
      .toContainText(expectedContent);
  }
  
  async selectModel(modelName) {
    await this.clickTestId('model-selector');
    const visibleOption = this.page.locator(`[data-testid="model-option-${modelName}"]`).locator('visible=true').first();
    await expect(visibleOption).toBeVisible();
    await visibleOption.click();
  }
}
```

**File: `fixtures/ChatFixtures.mjs`** - Chat test data
```javascript
export class ChatFixtures {
  static createChatScenarios() {
    return {
      basicQuestions: [
        { input: 'What day comes after Monday?', expectedContains: 'Tuesday' },
        { input: 'What is 2+2?', expectedContains: '4' },
        { input: 'What color is the sky?', expectedContains: 'blue' }
      ],
      conversationFlow: [
        { message: 'Hello, what can you help me with?', expectResponse: true },
        { message: 'Can you explain what an API is?', expectResponse: true }
      ]
    };
  }
}
```

### Task 1.2: Complete Chat Integration Test Suite
**File: `specs/core/chat/chat.spec.mjs`**

**Implementation using proven consolidation approach:**
```javascript
test('complete chat functionality with model integration and conversation flow', async ({ page }) => {
  // Step 1: Login and navigation
  await loginPage.performOAuthLogin();
  
  // Step 2: Navigate from models to chat (proven integration pattern)
  await modelsPage.navigateToModels();
  await modelsPage.clickChatWithModel('gpt-4');
  
  // Step 3: Chat functionality testing
  await chatPage.sendMessage('What day comes after Monday?');
  await chatPage.waitForResponse('Tuesday');
  
  // Step 4: Model switching test
  await chatPage.selectModel('gpt-3.5-turbo');
  await chatPage.sendMessage('What is 2+2?');
  await chatPage.waitForResponse('4');
  
  // Step 5: New conversation test
  await chatPage.startNewConversation();
  await chatPage.verifyEmptyChat();
  
  // Step 6: Streaming response test (if applicable)
  await chatPage.sendMessage('Explain what an API is');
  await chatPage.waitForStreamingComplete();
});

test('chat error handling and edge cases', async ({ page }) => {
  await loginPage.performOAuthLogin();
  await chatPage.navigateToChat();
  
  // Test empty message handling
  await chatPage.verifySendButtonDisabledForEmptyMessage();
  
  // Test maximum message length
  await chatPage.sendMessage('x'.repeat(10000));
  await chatPage.expectValidationError();
  
  // Test network disconnection recovery
  await chatPage.simulateNetworkError();
  await chatPage.sendMessage('Test message');
  await chatPage.expectNetworkErrorHandling();
});
```

### Task 1.3: Add Required data-testid Attributes
**Add missing data-testid attributes to chat components (learned pattern):**
- Chat container, message input, send button
- Model selector dropdown and options
- Message history container and individual messages
- Streaming response indicators
- New chat/clear conversation buttons

### Task 1.4: Integration with Existing Infrastructure
**Leverage established patterns:**
- Use existing `LoginPage` and `ModelsListPage` from Phase 0
- Extend `BasePage` with chat-specific common methods
- Use established visible button selection pattern for responsive layouts
- Apply proven exact text matching for model selection
- Follow deterministic testing principles with clear assertions

### Success Criteria for Phase 1
- ✅ **Complete Feature Coverage**: Chat sending, receiving, model switching, conversation management
- ✅ **Integration Tested**: Navigation from models page to chat with pre-selected model
- ✅ **Error Handling**: Network errors, validation errors, edge cases covered
- ✅ **Pattern Consistency**: Page Object Model, fixtures, consolidated tests
- ✅ **Reliability**: Tests pass consistently with no flakiness
- ✅ **Maintainability**: Clear, well-structured test code following established conventions

## Phase 2: Vertical Migration - Local Models Management
**Goal: Complete local models functionality end-to-end migration**

**Target Completion: 3-4 days** 

### Migration Target: Local Models Test Suite
**Covers: Model aliases, local model configuration, HuggingFace integration, model downloading**

Based on existing infrastructure:
- Leverage existing `ModelsListPage` and `ApiModelFormPage` from Phase 0
- Extend with local model-specific functionality
- New structure: `specs/core/local-models/local-models.spec.mjs`

### Task 2.1: Extend Page Objects for Local Models (Vertical Pattern)
**Files to Extend using proven patterns:**

**File: `pages/LocalModelFormPage.mjs`** - Local model configuration
```javascript
export class LocalModelFormPage extends BasePage {
  selectors = {
    modelAliasInput: '[data-testid="local-model-alias"]',
    huggingFaceRepoInput: '[data-testid="huggingface-repo"]', 
    filenameInput: '[data-testid="model-filename"]',
    downloadButton: '[data-testid="download-model-button"]',
    createAliasButton: '[data-testid="create-alias-button"]',
    downloadProgress: '[data-testid="download-progress"]'
  };
  
  async fillLocalModelInfo(alias, repo, filename) {
    await this.fillTestId('local-model-alias', alias);
    await this.fillTestId('huggingface-repo', repo);
    await this.fillTestId('model-filename', filename);
  }
  
  async downloadModel() {
    await this.clickTestId('download-model-button');
    await this.waitForDownloadComplete();
  }
  
  async createAlias() {
    await this.clickTestId('create-alias-button');
    await this.page.waitForURL(url => url.pathname === '/ui/models/');
  }
}
```

**File: `fixtures/LocalModelFixtures.mjs`** - Local model test data
```javascript
export class LocalModelFixtures {
  static createLocalModelData(overrides = {}) {
    const timestamp = Date.now();
    return {
      alias: `test-local-${timestamp}`,
      repo: 'microsoft/DialoGPT-small',
      filename: 'pytorch_model.bin',
      provider: 'HuggingFace',
      ...overrides
    };
  }
  
  static getTestModels() {
    return [
      { repo: 'microsoft/DialoGPT-small', filename: 'pytorch_model.bin' },
      { repo: 'gpt2', filename: 'pytorch_model.bin' }
    ];
  }
}
```

### Task 2.2: Complete Local Models Integration Test Suite
**File: `specs/core/local-models/local-models.spec.mjs`**

```javascript
test('complete local model lifecycle with download and chat integration', async ({ page }) => {
  const modelData = LocalModelFixtures.createLocalModelData();
  
  // Step 1: Login and navigation
  await loginPage.performOAuthLogin();
  await modelsPage.navigateToModels();
  
  // Step 2: Create new local model alias
  await modelsPage.clickNewLocalModel();
  await localModelFormPage.fillLocalModelInfo(modelData.alias, modelData.repo, modelData.filename);
  
  // Step 3: Download model (if needed)
  await localModelFormPage.downloadModel();
  await localModelFormPage.waitForDownloadComplete();
  
  // Step 4: Create alias and verify in list
  await localModelFormPage.createAlias();
  await modelsPage.verifyLocalModelInList(modelData.alias, modelData.repo);
  
  // Step 5: Test chat integration with local model
  await modelsPage.clickChatWithModel(modelData.alias);
  await chatPage.sendMessage('Hello');
  await chatPage.waitForResponse();
  
  // Step 6: Edit and delete model
  await modelsPage.navigateToModels();
  await modelsPage.editModel(modelData.alias);
  await localModelFormPage.verifyFormPreFilled(modelData.alias, modelData.repo);
  await localModelFormPage.updateModel();
  
  await modelsPage.deleteModel(modelData.alias);
});

test('local model error handling and validation', async ({ page }) => {
  await loginPage.performOAuthLogin();
  await modelsPage.navigateToModels();
  await modelsPage.clickNewLocalModel();
  
  // Test validation errors
  await localModelFormPage.createAlias(); // Empty form
  await localModelFormPage.expectValidationErrors(['Alias required', 'Repository required']);
  
  // Test invalid repository
  await localModelFormPage.fillLocalModelInfo('test', 'invalid-repo', 'model.bin');
  await localModelFormPage.downloadModel();
  await localModelFormPage.expectDownloadError('Repository not found');
  
  // Test duplicate alias
  const existingAlias = 'existing-model';
  await localModelFormPage.fillLocalModelInfo(existingAlias, 'microsoft/DialoGPT-small', 'model.bin');
  await localModelFormPage.expectAliasValidationError('Alias already exists');
});
```

## Phase 3: Vertical Migration - Settings Management
**Goal: Complete settings functionality end-to-end migration**

**Target Completion: 3-4 days**

### Migration Target: Settings Test Suite
**Covers: Application settings, user preferences, system configuration, OAuth settings**

Based on existing patterns:
- Create new `SettingsPage` using established Page Object Model
- New structure: `specs/core/settings/settings.spec.mjs`
- Follow consolidation pattern from Phase 0 learnings

### Task 3.1: Create Settings Page Objects (Vertical Pattern)
**Files to Create using battle-tested patterns:**

**File: `pages/SettingsPage.mjs`** - Settings management interface
```javascript
export class SettingsPage extends BasePage {
  selectors = {
    settingsContainer: '[data-testid="settings-container"]',
    generalTab: '[data-testid="settings-tab-general"]',
    authTab: '[data-testid="settings-tab-auth"]',
    systemTab: '[data-testid="settings-tab-system"]',
    saveButton: '[data-testid="save-settings-button"]',
    resetButton: '[data-testid="reset-settings-button"]',
    exportButton: '[data-testid="export-settings-button"]'
  };
  
  async navigateToSettings() {
    await this.navigate('/ui/settings/');
    await this.expectVisible(this.selectors.settingsContainer);
  }
  
  async switchToTab(tabName) {
    await this.clickTestId(`settings-tab-${tabName}`);
    await this.page.waitForTimeout(300); // Tab transition
  }
  
  async updateSetting(settingId, value) {
    await this.fillTestId(`setting-${settingId}`, value);
  }
  
  async saveSettings() {
    await this.clickTestId('save-settings-button');
    await this.waitForToast('Settings saved successfully');
  }
}
```

**File: `fixtures/SettingsFixtures.mjs`** - Settings test data
```javascript
export class SettingsFixtures {
  static createSettingsData() {
    return {
      general: {
        theme: 'dark',
        language: 'en',
        autoSave: true
      },
      auth: {
        sessionTimeout: 3600,
        requireReauth: false
      },
      system: {
        maxConcurrentChats: 3,
        logLevel: 'info'
      }
    };
  }
  
  static getValidationScenarios() {
    return [
      { field: 'sessionTimeout', invalid: -1, error: 'Must be positive number' },
      { field: 'maxConcurrentChats', invalid: 0, error: 'Must be at least 1' }
    ];
  }
}
```

### Task 3.2: Complete Settings Integration Test Suite
**File: `specs/core/settings/settings.spec.mjs`**

```javascript
test('complete settings management lifecycle with persistence and validation', async ({ page }) => {
  const settingsData = SettingsFixtures.createSettingsData();
  
  // Step 1: Login and navigation
  await loginPage.performOAuthLogin();
  await settingsPage.navigateToSettings();
  
  // Step 2: Update general settings
  await settingsPage.switchToTab('general');
  await settingsPage.updateSetting('theme', settingsData.general.theme);
  await settingsPage.updateSetting('language', settingsData.general.language);
  await settingsPage.saveSettings();
  
  // Step 3: Update auth settings
  await settingsPage.switchToTab('auth');
  await settingsPage.updateSetting('session-timeout', settingsData.auth.sessionTimeout);
  await settingsPage.saveSettings();
  
  // Step 4: Verify settings persistence
  await page.reload();
  await settingsPage.navigateToSettings();
  await settingsPage.verifySettingValue('theme', settingsData.general.theme);
  
  // Step 5: Export/import settings
  await settingsPage.exportSettings();
  await settingsPage.verifyExportFile();
  
  // Step 6: Reset settings
  await settingsPage.resetSettings();
  await settingsPage.verifyDefaultSettings();
});

test('settings validation and error handling', async ({ page }) => {
  await loginPage.performOAuthLogin();
  await settingsPage.navigateToSettings();
  
  const validationScenarios = SettingsFixtures.getValidationScenarios();
  
  for (const scenario of validationScenarios) {
    await settingsPage.updateSetting(scenario.field, scenario.invalid);
    await settingsPage.saveSettings();
    await settingsPage.expectValidationError(scenario.error);
  }
  
  // Test connection settings validation
  await settingsPage.switchToTab('auth');
  await settingsPage.updateSetting('oauth-url', 'invalid-url');
  await settingsPage.testConnection();
  await settingsPage.expectConnectionError('Invalid OAuth URL');
});
```

## Phase 4: Vertical Migration - Complete User Journeys
**Goal: End-to-end user journey testing using established feature components**

**Target Completion: 4-5 days**

### Migration Target: User Journey Test Suite
**Covers: First-time setup to productive usage, power user workflows, error recovery journeys**

Based on Phase 0-3 components:
- Leverage all existing page objects (Login, Models, Chat, Settings, Setup)
- Create comprehensive journey flows
- New structure: `specs/journeys/user-journeys.spec.mjs`

### Task 4.1: Complete User Journey Test Suite
**File: `specs/journeys/user-journeys.spec.mjs`**

```javascript
test('complete first-time user journey: setup to productive chat', async ({ page }) => {
  // Step 1: First-time setup flow (using Phase 0.1 components)
  const setupData = SetupFixtures.createSetupScenario();
  await setupPage.completeFirstTimeSetup(setupData);
  
  // Step 2: Create first API model (using Phase 0 components)
  const apiModelData = ApiModelFixtures.createModelData();
  await modelsPage.navigateToModels();
  await modelsPage.clickNewApiModel();
  await apiModelFormPage.fillBasicInfo(apiModelData.modelId, process.env.INTEG_TEST_OPENAI_API_KEY);
  await apiModelFormPage.fetchAndSelectModels(['gpt-4']);
  await apiModelFormPage.testConnection();
  await apiModelFormPage.createModel();
  
  // Step 3: First chat experience (using Phase 1 components)
  await modelsPage.clickChatWithModel(apiModelData.modelId);
  await chatPage.sendMessage('Hello! What can you help me with?');
  await chatPage.waitForResponse();
  
  // Step 4: Explore settings (using Phase 3 components)
  await settingsPage.navigateToSettings();
  await settingsPage.switchToTab('general');
  await settingsPage.updateSetting('theme', 'dark');
  await settingsPage.saveSettings();
  
  // Step 5: Create local model (using Phase 2 components)
  const localModelData = LocalModelFixtures.createLocalModelData();
  await modelsPage.navigateToModels();
  await modelsPage.clickNewLocalModel();
  await localModelFormPage.fillLocalModelInfo(localModelData.alias, localModelData.repo, localModelData.filename);
  await localModelFormPage.createAlias();
  
  // Step 6: Multi-model conversation
  await modelsPage.clickChatWithModel(localModelData.alias);
  await chatPage.sendMessage('What kind of model are you?');
  await chatPage.waitForResponse();
  
  await chatPage.selectModel(apiModelData.modelId);
  await chatPage.sendMessage('And what about you?');
  await chatPage.waitForResponse();
});

test('power user workflow: rapid model management and advanced chat', async ({ page }) => {
  await loginPage.performOAuthLogin();
  
  // Rapid model creation workflow
  const models = [
    ApiModelFixtures.createModelData({ modelId: 'gpt-4-power' }),
    ApiModelFixtures.createModelData({ modelId: 'gpt-3.5-power' }),
    LocalModelFixtures.createLocalModelData({ alias: 'local-power' })
  ];
  
  // Batch create models
  for (const model of models.slice(0, 2)) {
    await modelsPage.createApiModelQuick(model);
  }
  await modelsPage.createLocalModelQuick(models[2]);
  
  // Rapid model switching in chat
  await chatPage.navigateToChat();
  for (const model of models) {
    await chatPage.selectModel(model.modelId || model.alias);
    await chatPage.sendMessage(`Testing ${model.modelId || model.alias}`);
    await chatPage.waitForResponse();
  }
  
  // Bulk model operations
  await modelsPage.navigateToModels();
  await modelsPage.selectMultipleModels(models.map(m => m.modelId || m.alias));
  await modelsPage.bulkDelete();
  await modelsPage.confirmBulkOperation();
});

test('error recovery journey: network failures to successful completion', async ({ page }) => {
  await loginPage.performOAuthLogin();
  
  // Simulate network issues during model creation
  await modelsPage.navigateToModels();
  await modelsPage.clickNewApiModel();
  
  const modelData = ApiModelFixtures.createModelData();
  await apiModelFormPage.fillBasicInfo(modelData.modelId, 'invalid-api-key');
  
  // Test connection failure and recovery
  await apiModelFormPage.testConnection();
  await apiModelFormPage.expectConnectionError('Invalid API key');
  
  await apiModelFormPage.fillBasicInfo(modelData.modelId, process.env.INTEG_TEST_OPENAI_API_KEY);
  await apiModelFormPage.testConnection();
  await apiModelFormPage.createModel();
  
  // Network failure during chat and recovery
  await modelsPage.clickChatWithModel(modelData.modelId);
  await chatPage.simulateNetworkFailure();
  await chatPage.sendMessage('This should fail initially');
  await chatPage.expectNetworkError();
  
  await chatPage.restoreNetworkConnection();
  await chatPage.retryLastMessage();
  await chatPage.waitForResponse();
  
  // Session timeout and reauth
  await chatPage.simulateSessionTimeout();
  await chatPage.sendMessage('This should trigger reauth');
  await loginPage.expectLoginRedirect();
  await loginPage.performOAuthLogin('/ui/chat/');
  await chatPage.verifyReturnedToChat();
});
```

### Task 4.2: Integration Quality Tests
**File: `specs/quality/integration-quality.spec.mjs`**

```javascript
test('responsive design integration across all features', async ({ page }) => {
  const viewports = [{ width: 375, height: 667 }, { width: 768, height: 1024 }, { width: 1920, height: 1080 }];
  
  for (const viewport of viewports) {
    await page.setViewportSize(viewport);
    
    // Test responsive behavior across all migrated features
    await loginPage.performOAuthLogin();
    await modelsPage.verifyResponsiveLayout(viewport.width);
    await chatPage.verifyResponsiveLayout(viewport.width);
    await settingsPage.verifyResponsiveLayout(viewport.width);
    
    // Test responsive interactions
    await modelsPage.testResponsiveModelCreation(viewport.width);
    await chatPage.testResponsiveChatting(viewport.width);
  }
});

test('performance integration with realistic data volumes', async ({ page }) => {
  await loginPage.performOAuthLogin();
  
  // Create realistic test data
  const manyModels = Array.from({length: 20}, (_, i) => 
    ApiModelFixtures.createModelData({modelId: `perf-test-${i}`})
  );
  
  // Performance test model list with many items
  for (const model of manyModels.slice(0, 5)) {
    await modelsPage.createApiModelQuick(model);
  }
  
  await modelsPage.navigateToModels();
  await modelsPage.measurePageLoadTime(); // Should be < 2s
  await modelsPage.testPaginationPerformance();
  
  // Performance test chat with long conversations
  await chatPage.navigateToChat();
  await chatPage.simulateLongConversation(50); // 50 messages
  await chatPage.measureResponseTime(); // Should be < 5s for new messages
  
  // Cleanup
  await modelsPage.bulkDeleteAllTestModels();
});
```

## Phase 5: Test Infrastructure & Documentation
**Goal: Finalize test infrastructure and create comprehensive documentation**

**Target Completion: 2-3 days**

### Task 5.1: Test Suite Organization & Configuration
**Based on proven patterns from Phases 0-4**

**Actions:**
- ✅ Directory structure established and proven
- ✅ Page Object Model battle-tested across all features
- ✅ Test data fixtures validated in production use
- [ ] Configure test suite categories (smoke, integration, full)
- [ ] Optimize parallel execution using lessons learned
- [ ] Establish test tagging for selective execution
- [ ] Document test isolation patterns that work

**File: Enhanced `playwright.config.mjs`**
```javascript
export default {
  testDir: './tests-js',
  projects: [
    {
      name: 'smoke',
      grep: /@smoke/,
      testMatch: '**/specs/**/*.spec.mjs'
    },
    {
      name: 'integration', 
      grep: /@integration/,
      testMatch: '**/specs/core/**/*.spec.mjs'
    },
    {
      name: 'journeys',
      grep: /@journey/,
      testMatch: '**/specs/journeys/**/*.spec.mjs'
    }
  ]
};
```

### Task 5.2: Comprehensive Test Documentation
**Capture all lessons learned from vertical migration approach**

**File: `tests-js/README.md`** - Complete implementation guide
```markdown
# BodhiApp UI Test Suite

## Proven Architecture (Battle-Tested)

### Page Object Model Structure
- `pages/BasePage.mjs` - Common functionality across all pages
- `pages/LoginPage.mjs` - OAuth authentication flow (proven reliable)
- `pages/ModelsListPage.mjs` - Model CRUD operations with responsive layout handling
- `pages/ApiModelFormPage.mjs` - API model configuration with exact text matching
- `pages/ChatPage.mjs` - Chat functionality with streaming support
- `fixtures/` - Test data factories with cleanup patterns

### Proven Patterns
1. **Vertical Migration**: Complete feature migration vs partial infrastructure
2. **Test Consolidation**: Fewer comprehensive tests vs many small tests
3. **Visible Element Selection**: Handle responsive layouts with duplicate elements
4. **Exact Text Matching**: Prevent selector ambiguity issues
5. **Deterministic Testing**: Clear assertions, no try-catch patterns
```

**File: `tests-js/PATTERNS.md`** - Implementation patterns
```markdown
# Battle-Tested Test Patterns

## Selector Strategy (Learned from Production Issues)

### data-testid Usage
- Use `[data-testid="element-name"]` for reliable element selection
- Add data-testid to all interactive elements during implementation

### Responsive Layout Handling
Problem: Responsive layouts create duplicate elements
Solution: Select visible elements

```javascript
// ❌ Fails with duplicate elements
const button = this.page.locator('[data-testid="chat-button"]');

// ✅ Works with responsive layouts  
const visibleButton = this.page.locator('[data-testid="chat-button"]').locator('visible=true').first();
```

### Exact Text Matching
Problem: Partial text matches cause ambiguity
Solution: Use exact text selectors

```javascript
// ❌ "gpt-4" matches "gpt-4-0613" 
has-text("${model}")

// ✅ Exact match only
>> text="${model}"
```
```

### Task 5.3: CI/CD Integration Templates
**File: Scripts for different test execution modes**

```bash
# scripts/test-smoke.sh
npm run test -- --grep="@smoke"

# scripts/test-integration.sh  
npm run test -- --grep="@integration"

# scripts/test-full.sh
npm run test
```

### Task 5.4: Final Migration Cleanup
**Actions:**
- [ ] Remove old test files from `tests-js/playwright/` directory
- [ ] Update package.json test scripts to use new structure
- [ ] Verify all environment variables documented
- [ ] Create troubleshooting guide for common issues

### Success Criteria for Phase 5
- ✅ **Documentation Complete**: All patterns and lessons learned documented
- ✅ **Test Organization**: Clear categorization and execution modes
- ✅ **Migration Complete**: All old test files removed, new structure validated
- ✅ **Knowledge Transfer**: Future developers can extend tests using established patterns
- ✅ **CI/CD Ready**: Test suite ready for automated execution

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