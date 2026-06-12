# Edit Model Page Coverage Analysis

**File**: `/crates/bodhi/src/app/ui/models/edit/page.tsx`
**Route**: `/ui/models/edit/`
**Purpose**: Model alias editing interface with data loading from query parameters

## Page Overview

### Primary Functionality
- **Model Alias Editing**: Interface for modifying existing local model aliases
- **Data Loading**: Fetches existing model data via `alias` query parameter
- **Error Handling**: Comprehensive loading and error state management
- **Form Delegation**: Uses shared `AliasForm` component in edit mode with pre-populated data

### Component Hierarchy
```
EditAliasPage (wrapper with AppInitializer)
└── EditAliasContent
    ├── Loading Component (while fetching data)
    ├── ErrorPage (on fetch error or missing data)
    └── AliasForm (isEditMode=true, initialData from API)
        ├── Basic Information Section (alias field disabled)
        ├── Context Parameters Section
        └── Request Parameters Section (collapsible)
```

### Data Loading Flow
```typescript
// Query parameter extraction
const alias = searchParams?.get('alias');

// API data fetching
const { data: modelData, isLoading, error } = useModel(alias ?? '');

// State handling
if (isLoading) return <Loading message="Loading model data..." />;
if (error) return <ErrorPage message="Error loading model data" />;
if (!modelData) return <ErrorPage title="Not Found" message="No model data found" />;
```

### Key Features
- **Disabled Alias Field**: Prevents changing the alias identifier during editing
- **Data Validation**: Pre-existing data validation before form population
- **Error Boundaries**: Multiple error states with appropriate messaging
- **Loading States**: User-friendly loading indicators during data fetch

## Page Object Model Analysis

### POM Coverage Assessment: **EXCELLENT**
**File**: `/crates/lib_bodhiserver_napi/tests-js/pages/LocalModelFormPage.mjs`

#### Edit Mode Support: **COMPREHENSIVE**
✅ **Edit Detection**: `isEditMode()` method correctly identifies edit vs create mode
✅ **Update Actions**: `updateAlias()` method handles form submission in edit mode
✅ **Data Verification**: `getFormData()` method can verify pre-populated form state
✅ **Disabled Field Handling**: Properly handles disabled alias input field

#### Key POM Strengths for Edit Mode
```javascript
async isEditMode() {
  const submitBtn = this.page.locator(this.selectors.submitButton);
  const buttonText = await submitBtn.textContent();
  return buttonText?.includes('Update');
}

async updateAlias() {
  const submitBtn = this.page.locator(this.selectors.submitButton);
  await expect(submitBtn).toContainText('Update');
  await submitBtn.click();
  await this.waitForUrl('/ui/models/');
}

async getFormData() {
  // Extracts all form values for verification
  return {
    alias,
    repo: repoValue?.trim() || '',
    filename: filenameValue?.trim() || '',
    snapshot: snapshotValue,
    contextParams,
  };
}
```

## Test Coverage Analysis

### Existing Test Coverage: **EXCELLENT**
**Primary Testing**: Covered comprehensively in `model-alias.spec.mjs`

#### Edit Workflow Testing
✅ **Navigation to Edit**: Tests navigation from models list to edit form
✅ **Form Pre-population**: Verifies existing data loads correctly into form
✅ **Edit Mode Detection**: Confirms form is in edit mode (Update button)
✅ **Data Modification**: Tests updating context and request parameters
✅ **Save Functionality**: Verifies updates are saved and user returns to list

#### Specific Test Examples
```javascript
// Edit workflow from models list
test('complete local model lifecycle', async ({ page }) => {
  // ... create model first ...

  // Step 4: Edit the primary alias
  await modelsPage.editLocalModel(primaryData.alias);
  await formPage.waitForFormReady();

  // Verify we're in edit mode
  expect(await formPage.isEditMode()).toBe(true);

  // Update context parameters and some request parameters
  await formPage.fillContextParams(primaryData.updatedData.contextParams);
  await formPage.fillRequestParams(primaryData.updatedData.requestParams);

  // Update the alias
  await formPage.updateAlias();
});

// Context parameter persistence testing
test('advanced features and edge cases', async ({ page }) => {
  // Edit the alias to verify context parameters persistence
  await modelsPage.editLocalModel(contextData.alias);
  await formPage.waitForFormReady();

  // Verify context parameters are populated correctly
  const contextValue = await page.locator('[data-testid="context-params"]').inputValue();
  expect(contextValue).toBe(contextData.advancedParams);
});
```

## Data-TestId Audit

### Inherited Data-TestId Coverage

#### ✅ Well-Covered Elements (from AliasForm)
```typescript
// Form identification
"form-edit-alias"                  // Form container (auto-generated for edit mode)

// Basic information fields (same as create mode)
"alias-input"                      // Model alias name (disabled in edit mode)
"repo-select"                      // Repository combobox
"filename-select"                  // Filename combobox
"snapshot-select"                  // Snapshot/branch combobox
"context-params"                   // Context parameters textarea

// Request parameters section
"request-params-toggle"            // Expand/collapse button
"request-param-*"                  // All request parameter fields

// Form actions
"submit-alias-form"               // Submit button (shows "Update Model Alias")
```

#### ❌ Missing Page-Specific Data-TestIds
```typescript
// Page-specific states that need test IDs
"loading-model-data"              // Loading indicator
"error-loading-model"             // Error state
"model-not-found"                 // Not found error
"edit-page-container"             // Page wrapper
```

#### ⚠️ Edit Mode Considerations
- **Disabled Field Testing**: Alias input is disabled but should still be testable
- **Loading State Gaps**: No specific test IDs for edit page loading states
- **Error State Gaps**: Generic error pages lack edit-specific context

## Gap Analysis

### Coverage Assessment: **MINOR GAPS**

The edit page leverages excellent coverage from the shared `AliasForm` component and comprehensive testing in `model-alias.spec.mjs`. Gaps are primarily edge cases and error scenarios.

#### Missing Test Scenarios

##### 1. **Error State Handling** (MEDIUM PRIORITY)
**Business Impact**: Error handling robustness unclear
**Missing Coverage**:
- Invalid alias parameter (non-existent model)
- API error during data loading
- Network failure scenarios
- Permission denied errors

##### 2. **Loading State Verification** (LOW PRIORITY)
**Business Impact**: User experience validation
**Missing Coverage**:
- Loading indicator behavior verification
- Loading timeout scenarios
- Multiple rapid navigation scenarios

##### 3. **Data Integrity Edge Cases** (LOW PRIORITY)
**Business Impact**: Data consistency validation
**Missing Coverage**:
- Corrupted model data handling
- Missing required field scenarios
- Data type mismatch handling

##### 4. **Browser Navigation Edge Cases** (LOW PRIORITY)
**Business Impact**: Navigation robustness
**Missing Coverage**:
- Direct URL access with invalid parameters
- Browser back/forward button behavior
- Page refresh during editing

## Recommendations

### High-Value Test Additions

#### 1. **Error Scenario Testing** (Priority: MEDIUM)
```javascript
// New test scenarios in model-alias.spec.mjs
test.describe('Edit Model Error Handling', () => {
  test('handles non-existent model gracefully', async ({ page }) => {
    await page.goto('/ui/models/edit/?alias=non-existent-model');

    // Should show appropriate error message
    await expect(page.locator('[data-testid="error-message"]'))
      .toContainText('No model data found');
  });

  test('handles API errors during data loading', async ({ page }) => {
    // Mock API failure
    await page.route('**/bodhi/v1/models/*', route =>
      route.fulfill({ status: 500, body: 'Server Error' })
    );

    await modelsPage.editLocalModel('existing-alias');

    // Should show error state
    await expect(page.locator('[data-testid="error-loading-model"]'))
      .toBeVisible();
  });
});
```

#### 2. **Loading State Verification** (Priority: LOW)
```javascript
test('loading states during edit data fetch', async ({ page }) => {
  // Intercept API call to add delay
  await page.route('**/bodhi/v1/models/*', async route => {
    await new Promise(resolve => setTimeout(resolve, 2000));
    route.continue();
  });

  await modelsPage.editLocalModel('existing-alias');

  // Verify loading indicator appears
  await expect(page.locator('[data-testid="loading-model-data"]'))
    .toBeVisible();

  // Wait for form to load
  await formPage.waitForFormReady();

  // Verify loading indicator disappears
  await expect(page.locator('[data-testid="loading-model-data"]'))
    .not.toBeVisible();
});
```

### POM Enhancements

#### 1. **Error State Handling**
```javascript
// Add to LocalModelFormPage.mjs or create EditModelPage.mjs
async waitForEditPageReady() {
  // Wait for either form ready or error state
  await Promise.race([
    this.waitForSelector(this.selectors.aliasInput),
    this.waitForSelector('[data-testid="error-message"]'),
    this.waitForSelector('[data-testid="loading-indicator"]')
  ]);
}

async verifyEditPageError(expectedMessage) {
  const errorElement = this.page.locator('[data-testid="error-loading-model"]');
  await expect(errorElement).toContainText(expectedMessage);
}

async verifyLoadingState() {
  const loadingElement = this.page.locator('[data-testid="loading-model-data"]');
  await expect(loadingElement).toBeVisible();
}
```

#### 2. **Enhanced Navigation Support**
```javascript
async navigateToEditModel(alias) {
  await this.navigate(`/ui/models/edit/?alias=${encodeURIComponent(alias)}`);
  await this.waitForEditPageReady();
}

async verifyFormPrePopulation(expectedData) {
  await this.waitForFormReady();
  const formData = await this.getFormData();

  for (const [key, expectedValue] of Object.entries(expectedData)) {
    expect(formData[key]).toBe(expectedValue);
  }
}
```

### Data-TestId Enhancements

#### Recommended Additions
```typescript
// Add to EditAliasContent component
"edit-page-container"             // Page wrapper
"loading-model-data"              // Loading state indicator
"error-loading-model"             // Error state container
"model-not-found-error"           // Specific not found error

// Add to Loading component (if used)
"loading-message"                 // Loading message text

// Add to ErrorPage component (if used in edit context)
"error-title"                     // Error title
"error-description"               // Error description
```

### Code Enhancement Suggestions

#### 1. **Enhanced Error Messages**
```typescript
// More specific error handling in EditAliasContent
if (error) {
  const errorMessage = error.response?.status === 404
    ? `Model alias "${alias}" not found`
    : 'Error loading model data';
  return <ErrorPage message={errorMessage} data-testid="error-loading-model" />;
}
```

#### 2. **Loading State Improvements**
```typescript
// Enhanced loading component
if (isLoading) {
  return (
    <Loading
      message="Loading model data..."
      data-testid="loading-model-data"
    />
  );
}
```

## Summary

The Edit Model Page demonstrates **strong test coverage** through comprehensive integration with the `model-alias.spec.mjs` test suite and excellent POM support. The page effectively leverages the shared `AliasForm` component while providing appropriate edit-specific functionality.

**Strengths**:
- Comprehensive edit workflow testing
- Excellent POM support for edit mode operations
- Strong form pre-population and update testing
- Good integration with existing test infrastructure

**Minor Gaps**:
- Error state handling scenarios
- Loading state verification
- Edge case navigation scenarios

**Priority Actions**:
1. Add error scenario testing (medium priority)
2. Implement page-specific data-testids for loading/error states
3. Consider enhanced POM methods for error handling

The page is in excellent testing condition with strong coverage of core functionality. The identified gaps are primarily edge cases that would enhance robustness rather than address critical missing coverage.