# Setup Flow V2 - Comprehensive Revamp Plan

## Overview

Revamp the BodhiApp setup flow to showcase and configure new features introduced in the past 6 months:
1. **API Models** - Allow users to configure external API providers with their own API keys
2. **Browser Extension** - Check for installation and guide users to install for enhanced capabilities

## Setup Flow Overview (6 Steps)

### Step 1: Welcome & Benefits (Current)
- **Keep existing implementation** with benefits cards
- Show server name/description form
- User registers the app instance

### Step 2: Resource Admin (Current)
- **Keep existing flow**
- User logs in with OAuth
- User becomes admin for this app instance

### Step 3: Download Models (Current)
- **Keep existing implementation**
- Show recommended models (update list to latest)
- Allow background downloads
- User can skip and download later

### Step 4: Configure API Models (NEW)
New screen to configure external API providers:
- **Header**: "Enhance with Cloud AI Models"
- **Description**: "Connect your API keys to access powerful cloud models alongside local models"
- **API Configuration Form**:
  - Provider dropdown (OpenAI, OpenAI-compatible)
  - API key input (secure, masked display)
  - Base URL (pre-filled for OpenAI, editable for compatible)
  - Test Connection button
  - Success/Error feedback
- **Skip Option**: "Configure Later" button to proceed without API setup
- Save configuration to database using existing API model endpoints

### Step 5: Install Browser Extension (NEW)
New screen to guide browser extension installation:
- **Header**: "Install Browser Extension"
- **Description**: "Access BodhiApp from any website with our secure browser extension"
- **Extension Detection**:
  - Check if extension is already installed
  - If installed: Show "✓ Extension Detected" with green checkmark
  - If not installed: Show installation guide
- **Installation Guide**:
  - Detect user's browser
  - Show appropriate store link:
    - Chrome → Chrome Web Store
    - Edge → Edge Add-ons
    - Brave → Manual installation instructions
  - "Verify Installation" button to re-check
- **Benefits List**:
  - Access AI from any webpage
  - Secure local-first architecture
  - Works with both local and API models
- **Skip Option**: "Install Later" button

### Step 6: Setup Complete (Modified)
- **Header**: "Setup Complete!"
- **Summary Section**:
  - ✓ App registered
  - ✓ Admin account configured
  - ✓ X local models queued/downloaded
  - ✓ API models configured (if applicable)
  - ✓ Browser extension status
- **Action Button**: "Start Chatting" → redirects to `/ui/chat`
- **Secondary Options**: Links to Models page, Settings

## Implementation Plan (Test-Driven Phases)

### Phase 1: Foundation & Progress Updates
**Goal**: Update setup progress component and prepare foundation

#### Implementation Tasks:
1. **Update SetupProgress Component**
   - Modify `SetupProgress.tsx` to show 6 steps instead of 4
   - Update step labels and navigation
   - Add data-testid attributes for testing

#### Testing Tasks:
1. **Component Tests** (`crates/bodhi/`)
   ```bash
   npm run test -- SetupProgress.test.tsx
   ```
   - Test 6-step progress display
   - Test correct step highlighting
   - Test navigation between steps

#### Acceptance Criteria:
- [ ] SetupProgress shows 6 steps correctly
- [ ] Step labels are appropriate
- [ ] All tests pass

---

### Phase 2: API Models Configuration Screen
**Goal**: Create API models configuration page with form and testing

#### Implementation Tasks:
1. **Create API Models Page**
   - New file: `crates/bodhi/src/app/ui/setup/api-models/page.tsx`
   - Form component for API configuration
   - Integration with existing API endpoints

2. **Create Supporting Components**
   - `ApiModelConfigForm.tsx` - Main form component
   - Provider selection dropdown
   - API key input with show/hide toggle
   - Base URL input with defaults
   - Test connection functionality
   - Loading/success/error states

3. **Update Navigation**
   - Modify `download-models/page.tsx` to navigate to API models page
   - Add route constants

#### Testing Tasks:
1. **Component Tests** (`crates/bodhi/`)
   ```bash
   npm run test -- api-models/
   npm run test -- ApiModelConfigForm.test.tsx
   ```
   - Test form validation
   - Test API provider selection
   - Test connection testing functionality
   - Test skip functionality
   - Test navigation flow

#### Acceptance Criteria:
- [ ] API models page renders correctly
- [ ] Form validation works properly
- [ ] Test connection functionality works
- [ ] Skip option works
- [ ] Navigation flows correctly
- [ ] All component tests pass

---

### Phase 3: Browser Extension Setup Screen
**Goal**: Create browser extension detection and installation guide

#### Implementation Tasks:
1. **Create Extension Setup Page**
   - New file: `crates/bodhi/src/app/ui/setup/extension/page.tsx`
   - Browser detection logic
   - Extension detection using `window.bodhiext` check
   - Installation guide component

2. **Create Supporting Components**
   - `ExtensionDetector.tsx` - Browser and extension detection
   - `InstallationGuide.tsx` - Browser-specific installation instructions
   - Browser detection utility functions
   - Store URL mapping
   - Installation status component

3. **Update Navigation**
   - API models page navigates to extension page
   - Add route constants

#### Testing Tasks:
1. **Component Tests** (`crates/bodhi/`)
   ```bash
   npm run test -- extension/
   npm run test -- ExtensionDetector.test.tsx
   npm run test -- InstallationGuide.test.tsx
   ```
   - Test browser detection
   - Test extension detection logic
   - Test installation guide display
   - Test store URL generation
   - Test skip functionality
   - Mock `window.bodhiext` for testing

#### Acceptance Criteria:
- [ ] Extension detection works correctly
- [ ] Browser-specific installation guides display
- [ ] Store links are correct for each browser
- [ ] Skip functionality works
- [ ] Navigation flows correctly
- [ ] All component tests pass

---

### Phase 4: Setup Complete Enhancement
**Goal**: Enhance setup complete page with comprehensive summary

#### Implementation Tasks:
1. **Update Setup Complete Page**
   - Modify `crates/bodhi/src/app/ui/setup/complete/page.tsx`
   - Add configuration summary display
   - Update action buttons and navigation

2. **Create Supporting Components**
   - `SetupSummary.tsx` - Configuration summary with checkmarks
   - Status display for each setup component
   - Action buttons for next steps

3. **Update Final Navigation**
   - Extension page navigates to complete page
   - Complete page redirects to chat instead of staying on complete

#### Testing Tasks:
1. **Component Tests** (`crates/bodhi/`)
   ```bash
   npm run test -- complete/
   npm run test -- SetupSummary.test.tsx
   ```
   - Test summary display with different configurations
   - Test action button functionality
   - Test navigation to chat

#### Acceptance Criteria:
- [ ] Setup summary displays correctly
- [ ] Different configuration states show properly
- [ ] Action buttons work correctly
- [ ] Navigation to chat works
- [ ] All component tests pass

---

### Phase 5: Model Data Updates
**Goal**: Update recommended models with latest versions

#### Implementation Tasks:
1. **Update Model Data**
   - Update `download-models/data.ts` with latest models:
     - DeepSeek-R1 models
     - Latest Llama versions
     - Qwen 2.5 models
     - Gemma 2 models
   - Ensure model metadata is accurate

#### Testing Tasks:
1. **Component Tests** (`crates/bodhi/`)
   ```bash
   npm run test -- download-models/
   ```
   - Test model data structure
   - Test model card rendering with new data
   - Verify download functionality still works

#### Acceptance Criteria:
- [ ] Latest models are included in recommendations
- [ ] Model metadata is accurate
- [ ] Download functionality works with new models
- [ ] All tests pass

---

### Phase 6: Integration Testing
**Goal**: Comprehensive end-to-end testing of the complete setup flow

#### Implementation Tasks:
1. **Create Integration Tests**
   - New files in `crates/lib_bodhiserver_napi/tests-js/specs/ui/setup/`
   - `setup-flow-complete.spec.mjs` - Complete flow test
   - `setup-api-models.spec.mjs` - API models configuration test
   - `setup-extension.spec.mjs` - Extension installation test

2. **Test Scenarios**
   ```javascript
   // Complete setup flow
   test('Complete setup flow - all steps', async ({ page }) => {
     // Step 1: Welcome & Benefits
     // Step 2: Resource Admin (login)
     // Step 3: Download Models
     // Step 4: Configure API Models
     // Step 5: Extension Setup
     // Step 6: Setup Complete → Chat
   });

   test('Setup flow with skip options', async ({ page }) => {
     // Test skipping API models and extension
   });

   test('API models configuration and testing', async ({ page }) => {
     // Test API key input, connection testing
   });

   test('Extension detection scenarios', async ({ page }) => {
     // Test with/without extension installed
   });
   ```

#### Testing Tasks:
1. **Playwright Integration Tests**
   ```bash
   cd crates/lib_bodhiserver_napi
   npm run test -- tests-js/specs/ui/setup/
   ```
   - Test complete setup flow end-to-end
   - Test skip functionality at each step
   - Test API configuration and connection testing
   - Test extension detection and installation flow
   - Test error scenarios and recovery
   - Test navigation between all steps

#### Acceptance Criteria:
- [ ] Complete setup flow works end-to-end
- [ ] Skip options work at all appropriate steps
- [ ] API models configuration saves correctly
- [ ] Extension detection works properly
- [ ] Error handling works correctly
- [ ] All integration tests pass
- [ ] Setup completes successfully and navigates to chat

## Benefits

1. **Progressive Enhancement** - Users get both local and cloud options
2. **Flexibility** - Each step is optional/skippable
3. **Clear Value Proposition** - Users understand all features
4. **Seamless Onboarding** - Logical flow from setup to usage
5. **Future-Ready** - Easy to add more API providers

## Migration for Existing Users

- Existing users skip setup entirely
- Can access new features via Settings page
- No breaking changes to current functionality

## Testing Strategy

### Component Testing
- Each phase includes comprehensive component tests
- Use `data-testid` attributes for reliable test selectors
- Mock external dependencies (APIs, browser APIs)
- Test error states and edge cases

### Integration Testing
- Playwright-based UI tests for complete user journeys
- Test realistic user scenarios
- Test skip paths and optional configurations
- Verify data persistence across steps
- Test error recovery and user guidance

## Success Metrics

- [ ] All component tests pass in each phase
- [ ] Integration tests cover complete user journeys
- [ ] Setup flow completion rate improves
- [ ] User adoption of API models and extension features
- [ ] No regression in existing setup functionality