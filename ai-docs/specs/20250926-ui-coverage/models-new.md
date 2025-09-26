# New Model Page Coverage Analysis

**File**: `/crates/bodhi/src/app/ui/models/new/page.tsx`
**Route**: `/ui/models/new/`
**Purpose**: Model alias creation interface with pre-population support from query parameters

## Page Overview

### Primary Functionality
- **Model Alias Creation**: Interface for creating new local model aliases
- **Pre-population Support**: Accepts URL parameters to pre-fill form (repo, filename, snapshot)
- **Query Parameter Integration**: Supports `?repo=...&filename=...&snapshot=...` for streamlined workflows
- **Form Delegation**: Delegates to shared `AliasForm` component in create mode

### Component Hierarchy
```
NewModel (wrapper with AppInitializer)
└── NewModelContent
    └── AliasForm (isEditMode=false, initialData from query params)
        ├── Basic Information Section
        │   ├── Alias Input
        │   ├── Repo ComboBox
        │   ├── Filename ComboBox
        │   └── Snapshot ComboBox
        ├── Context Parameters Section
        └── Request Parameters Section (collapsible)
```

### Query Parameter Handling
```typescript
// URL pattern: /ui/models/new/?repo=vendor/model&filename=file.gguf&snapshot=main
const initialData = searchParams?.get('repo') || searchParams?.get('filename')
  ? {
      source: 'user' as const,
      alias: '',
      repo: searchParams?.get('repo') || '',
      filename: searchParams?.get('filename') || '',
      snapshot: searchParams?.get('snapshot') || '',
      request_params: {},
      context_params: [],
    }
  : undefined;
```

### AliasForm Integration
The page relies heavily on the shared `AliasForm` component which provides:
- **Dynamic Field Population**: ComboBox options loaded from existing model files
- **Cascading Selectors**: Repo → Filename → Snapshot dependency chain
- **Validation**: Comprehensive form validation with Zod schema
- **Request Parameters**: Expandable section with OpenAI-compatible parameters
- **Context Parameters**: llama-server parameter configuration

## Page Object Model Analysis

### POM Coverage Assessment: **EXCELLENT**
**File**: `/crates/lib_bodhiserver_napi/tests-js/pages/LocalModelFormPage.mjs`

#### Selector Coverage: **Comprehensive**
✅ **Basic Form Fields**: All primary input fields covered
✅ **ComboBox Interactions**: Sophisticated combobox selection methods
✅ **Request Parameters**: All OpenAI-compatible parameter fields
✅ **Form State Management**: Submit buttons, validation states
✅ **Dynamic Content**: Snapshot loading, combobox population

#### Key POM Strengths
- **ComboBox Mastery**: Robust `selectFromCombobox()` method handles complex dropdown interactions
- **Dynamic Loading**: `waitForSnapshotToLoad()` handles cascading field dependencies
- **Parameter Management**: `fillRequestParams()` with comprehensive parameter support
- **State Awareness**: `isEditMode()` detection for form state validation

#### Advanced Interaction Support
```javascript
// Sophisticated combobox handling
async selectFromCombobox(testId, value) {
  const trigger = this.page.locator(this.selectors.comboboxTrigger(testId));
  await trigger.click();
  const option = this.page.locator(this.selectors.comboboxOption(value));
  await option.click();
}

// Dynamic dependency handling
async waitForSnapshotToLoad() {
  await this.page.waitForFunction(() => {
    const snapshotElement = document.querySelector('[data-testid="snapshot-select"]');
    return snapshotElement && !snapshotElement.disabled;
  });
}
```

## Test Coverage Analysis

### Existing Test Coverage: **EXCELLENT**
**Primary Testing**: Covered comprehensively in `model-alias.spec.mjs`

#### Test Scenario Coverage
✅ **Basic Creation Flow**: Full workflow from navigation to form submission
✅ **Pre-populated Forms**: "Create alias from model" functionality testing
✅ **Form Validation**: Missing fields, invalid data, duplicate names
✅ **Advanced Parameters**: Context parameters and request parameters
✅ **Integration Testing**: End-to-end with navigation and verification

#### Specific Test Examples
```javascript
// Basic creation workflow
test('complete local model lifecycle', async ({ page }) => {
  await modelsPage.clickNewModelAlias();
  await formPage.waitForFormReady();
  await formPage.fillBasicInfo(primaryData.alias, primaryData.repo, primaryData.filename);
  await formPage.fillContextParams(primaryData.contextParams);
  await formPage.fillRequestParams(primaryData.requestParams);
  await formPage.createAlias();
});

// Pre-population testing
test('create alias from existing model', async ({ page }) => {
  await modelsPage.createAliasFromModel(secondaryData.sourceModelAlias);
  const formData = await formPage.getFormData();
  expect(formData.repo).toBe('bartowski/microsoft_Phi-4-mini-instruct-GGUF');
  expect(formData.filename).toBe('microsoft_Phi-4-mini-instruct-Q4_K_M.gguf');
});

// Validation testing
test('comprehensive validation and error handling', async ({ page }) => {
  // Test missing alias validation
  await formPage.fillBasicInfo('', validData.repo, validData.filename);
  await submitButton.click();
  await expect(page.url()).toContain('/ui/models/new');
});
```

## Data-TestId Audit

### Complete Data-TestId Coverage

#### ✅ Well-Covered Elements
```typescript
// Form identification
"form-create-alias"                // Form container (auto-generated)

// Basic information fields
"alias-input"                      // Model alias name
"repo-select"                      // Repository combobox
"filename-select"                  // Filename combobox
"snapshot-select"                  // Snapshot/branch combobox
"context-params"                   // Context parameters textarea

// Request parameters (expandable section)
"request-params-toggle"            // Expand/collapse button
"request-param-temperature"        // Temperature input
"request-param-max_tokens"         // Max tokens input
"request-param-top_p"             // Top-p input
"request-param-seed"              // Seed input
"request-param-stop"              // Stop sequences input
"request-param-frequency_penalty" // Frequency penalty input
"request-param-presence_penalty"  // Presence penalty input
"request-param-user"              // User field input

// Form actions
"submit-alias-form"               // Submit button

// ComboBox internals
"combobox-option-{value}"         // Dropdown options
```

#### ⚠️ Potential Issues
- **No Loading State IDs**: Form loading states lack specific test IDs
- **No Error Message IDs**: Validation error messages lack targeted test IDs
- **No Success State IDs**: Success notifications lack specific test IDs

## Gap Analysis

### Coverage Assessment: **MINIMAL GAPS**

The new model page has exceptional test coverage through the comprehensive `model-alias.spec.mjs` test suite. Most gaps are minor edge cases or advanced scenarios.

#### Minor Missing Scenarios

##### 1. **Direct URL Navigation with Query Parameters** (LOW PRIORITY)
**Business Impact**: Edge case but important for integration flows
**Missing Coverage**:
- Direct navigation with `?repo=...&filename=...&snapshot=...`
- Invalid query parameter handling
- Malformed URL parameter validation

##### 2. **ComboBox Edge Cases** (LOW PRIORITY)
**Business Impact**: User experience edge cases
**Missing Coverage**:
- Empty combobox lists (no models available)
- ComboBox loading states
- ComboBox error states (API failures)

##### 3. **Form State Persistence** (LOW PRIORITY)
**Business Impact**: User experience enhancement
**Missing Coverage**:
- Browser refresh with partial form data
- Navigation away and back to form
- Auto-save functionality (if implemented)

##### 4. **Advanced Parameter Validation** (LOW PRIORITY)
**Business Impact**: Parameter validation robustness
**Missing Coverage**:
- Invalid numeric parameter ranges
- Malformed context parameter syntax
- Complex stop sequence validation

## Recommendations

### High-Value Test Additions

#### 1. **Query Parameter Integration** (Priority: MEDIUM)
```javascript
// New test scenario in model-alias.spec.mjs
test('direct navigation with query parameters', async ({ page }) => {
  const repo = 'microsoft/DialoGPT-medium';
  const filename = 'pytorch_model.bin';
  const snapshot = 'main';

  await page.goto(`/ui/models/new/?repo=${repo}&filename=${filename}&snapshot=${snapshot}`);
  await formPage.waitForFormReady();

  const formData = await formPage.getFormData();
  expect(formData.repo).toBe(repo);
  expect(formData.filename).toBe(filename);
  expect(formData.snapshot).toBe(snapshot);
});

test('invalid query parameter handling', async ({ page }) => {
  // Test with malformed or invalid parameters
  await page.goto('/ui/models/new/?repo=invalid%20repo&filename=');
  await formPage.waitForFormReady();
  // Verify graceful handling
});
```

#### 2. **ComboBox Edge Case Testing** (Priority: LOW)
```javascript
test('combobox error states and edge cases', async ({ page }) => {
  // Mock empty model data
  await formPage.waitForFormReady();

  // Test behavior with no available options
  await expect(page.locator('[data-testid="repo-select"]'))
    .toContainText('No options available');
});
```

### POM Enhancements

#### 1. **Query Parameter Support**
```javascript
// Add to LocalModelFormPage.mjs
async navigateWithParams(params = {}) {
  const searchParams = new URLSearchParams(params);
  const url = `/ui/models/new/?${searchParams.toString()}`;
  await this.navigate(url);
  await this.waitForFormReady();
}

async verifyPrePopulatedData(expectedData) {
  const formData = await this.getFormData();
  for (const [key, value] of Object.entries(expectedData)) {
    expect(formData[key]).toBe(value);
  }
}
```

#### 2. **Enhanced Error Handling**
```javascript
async verifyValidationError(fieldName, expectedMessage) {
  const errorElement = this.page.locator(`[data-testid="${fieldName}"] + .error-message`);
  await expect(errorElement).toContainText(expectedMessage);
}

async verifyFormSubmissionBlocked() {
  const currentUrl = this.page.url();
  await this.page.locator(this.selectors.submitButton).click();
  await this.page.waitForTimeout(1000);
  expect(this.page.url()).toBe(currentUrl); // Should stay on same page
}
```

### Data-TestId Enhancements

#### Optional Additions
```typescript
// Add to AliasForm component for enhanced testing
"form-loading-indicator"          // Form loading state
"form-error-message"              // Global form errors
"field-error-{fieldName}"         // Individual field errors
"form-success-message"            // Success notifications
"combobox-loading-{testId}"       // ComboBox loading states
"combobox-empty-{testId}"         // Empty ComboBox states
```

## Summary

The New Model Page demonstrates **exceptional test coverage** and POM quality. The comprehensive `model-alias.spec.mjs` test suite covers all primary workflows and most edge cases. The `LocalModelFormPage` POM is sophisticated and handles complex interactions gracefully.

**Strengths**:
- Comprehensive end-to-end testing coverage
- Sophisticated Page Object Model with advanced interaction support
- Excellent data-testid coverage for all interactive elements
- Strong validation and error handling test scenarios
- Pre-population workflow testing

**Minor Gaps**:
- Direct URL navigation with query parameters
- ComboBox edge cases and error states
- Advanced parameter validation scenarios

**Priority Actions**:
1. Add query parameter integration tests (medium priority)
2. Consider ComboBox edge case testing (low priority)
3. Optional data-testid enhancements for error states

The page is in excellent testing condition with minimal gaps that are primarily edge cases rather than core functionality issues. The existing test coverage provides strong confidence in the page's reliability and user experience.