# Pull (Download Models) Page Coverage Analysis

**File**: `/crates/bodhi/src/app/ui/pull/page.tsx`
**Route**: `/ui/pull/`
**Purpose**: Model download interface with real-time progress tracking and download history

## Page Overview

### Primary Functionality
- **Model Download Interface**: Download models from HuggingFace repositories
- **Real-time Progress Tracking**: Live progress bars with percentage and byte counters
- **Download History**: Paginated table showing all download requests and their status
- **Status Management**: Visual indicators for pending, completed, and error states
- **Auto-refresh**: Polling for updates when downloads are in progress

### Component Hierarchy
```
PullPage (wrapper with AppInitializer)
└── PullPageContent
    ├── UserOnboarding (welcome banner)
    ├── PullForm (download request form)
    ├── DataTable (download history with progress)
    └── Pagination
```

### Key Components Analysis

#### PullForm Component
- **Repository Input**: Text input with autocomplete suggestions
- **Filename Input**: Text input with filename suggestions based on selected repo
- **Dynamic Suggestions**: Auto-populated from existing model data
- **Validation**: Form validation with error handling for duplicate files
- **Reset Functionality**: Clear form and error states

#### Download Progress System
- **Real-time Updates**: Polling-based updates for pending downloads
- **Progress Visualization**: Progress bars with percentage and byte display
- **Status Badges**: Color-coded status indicators (pending, completed, error)
- **Error Display**: Expandable error details for failed downloads

### Data Flow
```typescript
// Polling logic for real-time updates
useEffect(() => {
  const hasPendingDownloads = data?.data.some((download) => download.status === 'pending') ?? false;
  setEnablePolling(hasPendingDownloads);
}, [data]);

// Progress calculation
const computeProgress = (download: DownloadRequest) => {
  if (!download.total_bytes || download.total_bytes === 0) return 0;
  if (!download.downloaded_bytes) return 0;
  return (download.downloaded_bytes / download.total_bytes) * 100;
};
```

## Page Object Model Analysis

### POM Coverage Assessment: **NONE**
**Status**: No dedicated Page Object Model exists for the pull/download page

#### Missing POM Infrastructure
❌ **No PullPage.mjs**: No dedicated page object model
❌ **No Test Coverage**: No existing test specifications
❌ **No Form Integration**: No POM for the PullForm component

### Required POM Structure

```javascript
// Proposed: PullPage.mjs
export class PullPage extends BasePage {
  selectors = {
    content: '.container',

    // Pull form selectors
    pullForm: 'form',
    repoInput: '#repo',
    filenameInput: '#filename',
    submitButton: 'button[type="submit"]',
    resetButton: 'button:has-text("Reset")',

    // Autocomplete
    suggestions: '.suggestions-list',
    suggestionItem: (value) => `[data-suggestion="${value}"]`,

    // Download table
    downloadTable: 'table',
    downloadRow: (id) => `[data-download-id="${id}"]`,
    statusBadge: '.badge',
    progressBar: '.progress-bar',
    progressText: '.progress-text',

    // Expandable error rows
    errorRow: '.error-details',
    errorMessage: '.error-message',

    // Pagination
    pagination: '.pagination',
  };

  async navigateToDownloads() {
    await this.navigate('/ui/pull/');
    await this.waitForSelector(this.selectors.content);
  }

  async submitDownloadRequest(repo, filename) {
    // Fill form
    await this.page.fill(this.selectors.repoInput, repo);
    await this.page.fill(this.selectors.filenameInput, filename);

    // Submit
    await this.page.click(this.selectors.submitButton);

    // Wait for success or error
    await this.waitForToast();
  }

  async verifyDownloadInProgress(repo, filename) {
    await this.waitForSelector(this.selectors.downloadTable);

    // Find the download row
    const rows = await this.page.locator(`${this.selectors.downloadTable} tbody tr`).all();
    for (const row of rows) {
      const repoText = await row.locator('td:nth-child(1)').textContent();
      const filenameText = await row.locator('td:nth-child(2)').textContent();

      if (repoText?.includes(repo) && filenameText?.includes(filename)) {
        // Verify status is pending
        const statusBadge = row.locator(this.selectors.statusBadge);
        await expect(statusBadge).toContainText('pending');

        // Verify progress bar exists
        const progressBar = row.locator(this.selectors.progressBar);
        await expect(progressBar).toBeVisible();

        return row;
      }
    }

    throw new Error(`Download not found: ${repo}/${filename}`);
  }

  async waitForDownloadCompletion(repo, filename, timeout = 30000) {
    const startTime = Date.now();

    while (Date.now() - startTime < timeout) {
      try {
        const downloadRow = await this.verifyDownloadInProgress(repo, filename);
        const statusBadge = downloadRow.locator(this.selectors.statusBadge);
        const status = await statusBadge.textContent();

        if (status?.includes('completed')) {
          return 'completed';
        } else if (status?.includes('error')) {
          return 'error';
        }

        // Wait before checking again
        await this.page.waitForTimeout(1000);
      } catch {
        // Download might not be visible yet
        await this.page.waitForTimeout(1000);
      }
    }

    throw new Error(`Download did not complete within ${timeout}ms`);
  }

  async verifyProgressDisplay(expectedPercentage = null) {
    const progressBars = await this.page.locator(this.selectors.progressBar).all();

    for (const progressBar of progressBars) {
      await expect(progressBar).toBeVisible();

      if (expectedPercentage !== null) {
        const progressText = await progressBar.locator('+ .progress-text').textContent();
        expect(progressText).toContain(`${expectedPercentage}%`);
      }
    }
  }

  async expandErrorDetails(repo, filename) {
    const downloadRow = await this.findDownloadRow(repo, filename);

    // Look for expandable error indicator
    const errorToggle = downloadRow.locator('.error-toggle');
    if (await errorToggle.isVisible()) {
      await errorToggle.click();

      // Wait for error details to expand
      await expect(downloadRow.locator(this.selectors.errorRow)).toBeVisible();

      return await downloadRow.locator(this.selectors.errorMessage).textContent();
    }

    return null;
  }

  async selectSuggestion(inputSelector, value) {
    // Focus the input to trigger suggestions
    await this.page.focus(inputSelector);

    // Wait for suggestions to appear
    await this.waitForSelector(this.selectors.suggestions);

    // Click the specific suggestion
    await this.page.click(this.selectors.suggestionItem(value));
  }

  async resetForm() {
    await this.page.click(this.selectors.resetButton);

    // Verify form is cleared
    await expect(this.page.locator(this.selectors.repoInput)).toHaveValue('');
    await expect(this.page.locator(this.selectors.filenameInput)).toHaveValue('');
  }
}
```

## Test Coverage Analysis

### Existing Test Coverage: **NONE**
**Status**: No test specifications exist for the pull/download page

#### Complete Test Gap
❌ **No Dedicated Tests**: No test files covering download functionality
❌ **No Integration Tests**: No references in existing test suites
❌ **No Real-time Testing**: No polling or progress tracking tests

### Required Test Coverage

#### 1. **Basic Download Functionality**
```javascript
// Proposed: pull-download.spec.mjs
test.describe('Model Download Management', () => {
  test('submits download request successfully', async ({ page }) => {
    await loginPage.performOAuthLogin();
    await pullPage.navigateToDownloads();

    // Submit download request
    await pullPage.submitDownloadRequest(
      'microsoft/DialoGPT-small',
      'pytorch_model.bin'
    );

    // Verify success message
    await pullPage.waitForToast('Model pull request submitted successfully');

    // Verify download appears in table
    await pullPage.verifyDownloadInProgress(
      'microsoft/DialoGPT-small',
      'pytorch_model.bin'
    );
  });

  test('handles duplicate file downloads', async ({ page }) => {
    await loginPage.performOAuthLogin();
    await pullPage.navigateToDownloads();

    // Submit same download twice
    await pullPage.submitDownloadRequest(
      'microsoft/DialoGPT-small',
      'pytorch_model.bin'
    );

    // Second request should show error
    await pullPage.submitDownloadRequest(
      'microsoft/DialoGPT-small',
      'pytorch_model.bin'
    );

    // Verify error message
    await pullPage.waitForToast('File already exists');
  });
});
```

#### 2. **Progress Tracking Tests**
```javascript
test.describe('Download Progress Tracking', () => {
  test('displays real-time progress updates', async ({ page }) => {
    await loginPage.performOAuthLogin();
    await pullPage.navigateToDownloads();

    // Start download
    await pullPage.submitDownloadRequest(
      'microsoft/DialoGPT-medium',
      'config.json'  // Small file for testing
    );

    // Verify progress bar appears
    await pullPage.verifyProgressDisplay();

    // Wait for completion (small file should complete quickly)
    const finalStatus = await pullPage.waitForDownloadCompletion(
      'microsoft/DialoGPT-medium',
      'config.json'
    );

    expect(finalStatus).toBe('completed');
  });

  test('handles download errors gracefully', async ({ page }) => {
    await loginPage.performOAuthLogin();
    await pullPage.navigateToDownloads();

    // Submit download for non-existent file
    await pullPage.submitDownloadRequest(
      'nonexistent/repository',
      'nonexistent_file.bin'
    );

    // Wait for error status
    const finalStatus = await pullPage.waitForDownloadCompletion(
      'nonexistent/repository',
      'nonexistent_file.bin'
    );

    expect(finalStatus).toBe('error');

    // Verify error details can be expanded
    const errorMessage = await pullPage.expandErrorDetails(
      'nonexistent/repository',
      'nonexistent_file.bin'
    );

    expect(errorMessage).toBeTruthy();
  });
});
```

#### 3. **Form Interaction Tests**
```javascript
test.describe('Download Form Interactions', () => {
  test('autocomplete suggestions work correctly', async ({ page }) => {
    await loginPage.performOAuthLogin();
    await pullPage.navigateToDownloads();

    // Type in repo field to trigger suggestions
    await page.type('#repo', 'microsoft');

    // Verify suggestions appear
    await expect(page.locator('.suggestions-list')).toBeVisible();

    // Select suggestion
    await pullPage.selectSuggestion('#repo', 'microsoft/DialoGPT-medium');

    // Verify filename suggestions update
    await page.focus('#filename');
    await expect(page.locator('.suggestions-list')).toBeVisible();
  });

  test('form reset functionality works', async ({ page }) => {
    await loginPage.performOAuthLogin();
    await pullPage.navigateToDownloads();

    // Fill form
    await page.fill('#repo', 'microsoft/DialoGPT-medium');
    await page.fill('#filename', 'config.json');

    // Reset form
    await pullPage.resetForm();

    // Verify form is cleared
    await expect(page.locator('#repo')).toHaveValue('');
    await expect(page.locator('#filename')).toHaveValue('');
  });
});
```

## Data-TestId Audit

### Current Data-TestId Coverage

#### ❌ Completely Missing Data-TestIds
The pull page lacks any data-testid attributes, making automated testing extremely difficult.

#### Required Data-TestId Implementation

```typescript
// Add to PullForm component
<form data-testid="pull-form">
  <Input
    id="repo"
    data-testid="repo-input"
    placeholder="Enter repository"
  />
  <Input
    id="filename"
    data-testid="filename-input"
    placeholder="Enter filename"
  />
  <Button
    type="submit"
    data-testid="submit-download"
  >
    Pull Model
  </Button>
  <Button
    type="button"
    data-testid="reset-form"
  >
    Reset
  </Button>
</form>

// Add to AutocompleteInput component
<div className="suggestions-list" data-testid="autocomplete-suggestions">
  {suggestions.map(suggestion => (
    <div
      key={suggestion}
      data-testid={`suggestion-${suggestion}`}
      data-suggestion={suggestion}
    >
      {suggestion}
    </div>
  ))}
</div>

// Add to download table rows
<TableCell data-testid={`download-repo-${download.id}`}>
  {download.repo}
</TableCell>
<TableCell data-testid={`download-filename-${download.id}`}>
  {download.filename}
</TableCell>
<TableCell data-testid={`download-status-${download.id}`}>
  <StatusBadge status={download.status} />
</TableCell>
<TableCell data-testid={`download-progress-${download.id}`}>
  <ProgressDisplay download={download} />
</TableCell>

// Add to progress components
<div
  className="progress-bar"
  data-testid={`progress-bar-${download.id}`}
  style={{ width: `${progress}%` }}
/>
<span
  className="progress-text"
  data-testid={`progress-text-${download.id}`}
>
  {progress.toFixed(1)}%
</span>

// Add to error details
<div
  className="error-details"
  data-testid={`error-details-${download.id}`}
>
  <p data-testid={`error-message-${download.id}`}>
    {download.error}
  </p>
</div>
```

## Gap Analysis

### Coverage Assessment: **CRITICAL GAPS**

The pull/download page has **zero test coverage** and lacks all testing infrastructure.

#### Critical Missing Infrastructure

##### 1. **Complete Test Infrastructure** (HIGH PRIORITY)
**Business Impact**: Core download functionality with no verification
**Missing Components**:
- Page Object Model for download management
- Basic download functionality tests
- Real-time progress tracking tests
- Error handling verification

##### 2. **Data-TestId System** (HIGH PRIORITY)
**Business Impact**: Cannot implement reliable automated testing
**Missing Elements**:
- Form input identification
- Download row identification
- Progress component identification
- Error state identification

##### 3. **Real-time Testing Framework** (HIGH PRIORITY)
**Business Impact**: Progress tracking and polling behavior untested
**Missing Coverage**:
- Polling mechanism verification
- Progress bar updates
- Status change detection
- Auto-refresh behavior

##### 4. **Integration Testing** (MEDIUM PRIORITY)
**Business Impact**: Download workflow integration untested
**Missing Coverage**:
- End-to-end download process
- Integration with model files page
- Integration with models management

## Recommendations

### High-Value Test Additions

#### 1. **Complete Test Infrastructure** (Priority: HIGH)
```javascript
// Create comprehensive test suite
// - Basic download functionality
// - Progress tracking
// - Error handling
// - Form interactions
// - Real-time updates
```

#### 2. **Enhanced PullForm Component** (Priority: HIGH)
```typescript
// Add comprehensive data-testids to PullForm
export function PullForm() {
  return (
    <Form data-testid="download-form">
      <Card data-testid="download-card">
        <CardHeader>
          <CardTitle data-testid="download-title">Download Model</CardTitle>
        </CardHeader>
        <CardContent className="space-y-4">
          <FormField
            control={form.control}
            name="repo"
            render={({ field }) => (
              <FormItem>
                <FormLabel htmlFor="repo">Repository</FormLabel>
                <FormControl>
                  <Input
                    {...field}
                    id="repo"
                    data-testid="repo-input"
                    placeholder="Enter repository"
                  />
                </FormControl>
                <AutocompleteInput
                  data-testid="repo-autocomplete"
                  suggestions={repos}
                />
                <FormMessage data-testid="repo-error" />
              </FormItem>
            )}
          />

          <FormField
            control={form.control}
            name="filename"
            render={({ field }) => (
              <FormItem>
                <FormLabel htmlFor="filename">Filename</FormLabel>
                <FormControl>
                  <Input
                    {...field}
                    id="filename"
                    data-testid="filename-input"
                    placeholder="Enter filename"
                  />
                </FormControl>
                <AutocompleteInput
                  data-testid="filename-autocomplete"
                  suggestions={filenames}
                />
                <FormMessage data-testid="filename-error" />
              </FormItem>
            )}
          />

          <div className="flex justify-end gap-2">
            <Button
              type="button"
              variant="outline"
              data-testid="reset-button"
              onClick={handleReset}
            >
              Reset
            </Button>
            <Button
              type="submit"
              data-testid="submit-button"
              disabled={isLoading}
            >
              {isLoading ? 'Pulling...' : 'Pull Model'}
            </Button>
          </div>
        </CardContent>
      </Card>
    </Form>
  );
}
```

#### 3. **Progress Component Enhancement** (Priority: HIGH)
```typescript
function ProgressDisplay({ download }: { download: DownloadRequest }) {
  if (download.status !== 'pending') {
    return <span data-testid={`progress-static-${download.id}`}>-</span>;
  }

  return (
    <div data-testid={`progress-container-${download.id}`}>
      <div className="flex items-center space-x-2">
        <div
          className="w-full bg-muted rounded-full h-2"
          data-testid={`progress-track-${download.id}`}
        >
          <div
            className="bg-primary h-2 rounded-full transition-all duration-300"
            data-testid={`progress-bar-${download.id}`}
            style={{ width: `${Math.min(computeProgress(download), 100)}%` }}
          />
        </div>
        <span
          className="text-sm font-medium min-w-[3rem]"
          data-testid={`progress-percentage-${download.id}`}
        >
          {computeProgress(download).toFixed(1)}%
        </span>
      </div>
      {download.total_bytes && (
        <div
          className="text-xs text-muted-foreground"
          data-testid={`progress-bytes-${download.id}`}
        >
          {formatBytes(download.downloaded_bytes)} / {formatBytes(download.total_bytes)}
        </div>
      )}
    </div>
  );
}
```

### Integration Considerations

#### 1. **Model Files Page Integration**
```javascript
// Tests should verify downloaded files appear in modelfiles page
test('downloaded files appear in model files list', async ({ page }) => {
  // Download a model
  await pullPage.submitDownloadRequest('microsoft/DialoGPT-small', 'config.json');
  await pullPage.waitForDownloadCompletion('microsoft/DialoGPT-small', 'config.json');

  // Navigate to model files
  await modelFilesPage.navigateToModelFiles();

  // Verify file appears in list
  await modelFilesPage.verifyModelFileInList('microsoft/DialoGPT-small', 'config.json');
});
```

#### 2. **Models List Integration**
```javascript
// Tests should verify downloaded models can be used for aliases
test('downloaded models available for alias creation', async ({ page }) => {
  // Download a model
  await pullPage.submitDownloadRequest('microsoft/DialoGPT-small', 'pytorch_model.bin');
  await pullPage.waitForDownloadCompletion('microsoft/DialoGPT-small', 'pytorch_model.bin');

  // Create alias using downloaded model
  await modelsPage.navigateToModels();
  await modelsPage.clickNewModelAlias();

  // Verify downloaded model appears in repo/filename options
  await formPage.verifyModelAvailableForAlias('microsoft/DialoGPT-small', 'pytorch_model.bin');
});
```

## Summary

The Pull (Download Models) Page represents the **most critical testing gap** in the models management system. This page handles essential functionality for downloading models from HuggingFace, with real-time progress tracking and complex state management, yet has zero test coverage.

**Critical Issues**:
- No automated testing whatsoever
- No Page Object Model
- No data-testid implementation
- No real-time behavior verification
- No error handling testing

**Business Impact**:
Download functionality is core to the model management workflow. Users rely on this page to acquire models for use in the application. The lack of testing creates significant risk for:
- Download failures going undetected
- Progress tracking malfunctions
- Form validation bypassing
- Error states not displaying properly
- Performance regressions in polling behavior

**Immediate Actions Required** (All HIGH PRIORITY):
1. Implement comprehensive data-testid system
2. Create PullPage Page Object Model
3. Develop complete test suite covering all download scenarios
4. Add real-time progress tracking verification
5. Integrate with existing model management test framework

**Recommended Implementation Order**:
1. **Week 1**: Data-testid implementation and basic POM
2. **Week 2**: Core download functionality testing
3. **Week 3**: Real-time progress and polling tests
4. **Week 4**: Integration testing with other model management pages

This page should be the **highest priority** for testing implementation due to its critical role in the model management ecosystem and complete lack of verification coverage.