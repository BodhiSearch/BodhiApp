# Plan for Refactoring setup/api-models Page

## Overview
Refactor the API models setup page to reuse the form from ui/api-models, implementing maximum code reuse while maintaining setup-specific behavior.

## Phase-wise Implementation Plan

### Phase 1: Delete Current setup/api-models Folder ✅ COMPLETED
- ✅ Remove `crates/bodhi/src/app/ui/setup/api-models/` completely
- ✅ This includes: page.tsx, page.test.tsx, and ApiModelSetupForm.tsx
- ✅ Cleaned up orphaned schema references in `schemas/apiModels.ts`
- ✅ Fixed TypeScript compilation errors
- ✅ Verified build success

### Phase 2: Move and Refactor ApiModelForm ✅ COMPLETED
- ✅ Move `crates/bodhi/src/app/ui/api-models/ApiModelForm.tsx` to `crates/bodhi/src/components/api-models/ApiModelForm.tsx`
- ✅ Move its test file `ApiModelForm.test.tsx` accordingly
- ✅ Make it configurable to support three modes: 'create', 'edit', and 'setup'
- ✅ Refactor interface from `isEditMode` boolean to `mode` prop with navigation callbacks
- ✅ Update test file imports and prop usage

### Phase 3: Update Existing Pages ✅ COMPLETED
- ✅ Update `crates/bodhi/src/app/ui/api-models/new/page.tsx` import path and use `mode="create"`
- ✅ Update `crates/bodhi/src/app/ui/api-models/edit/page.tsx` import path and use `mode="edit"`
- ✅ Verified build success - all imports working correctly

### Phase 4: Update Setup Constants ✅ COMPLETED
- ✅ Change "Cloud AI" to "API Models" in SETUP_STEP_LABELS
- ✅ Update corresponding test expectations
- ✅ Verified all tests pass (25/25 tests)
- ✅ Verified build success
- ✅ Confirmed no remaining 'Cloud AI' references

### Phase 5: Create New Setup Page ✅ COMPLETED
- ✅ Create new `crates/bodhi/src/app/ui/setup/api-models/page.tsx`
- ✅ Setup progress header showing "API Models" (step 4 of 6)
- ✅ Simplified welcome section without benefit cards (clean card with emoji and description)
- ✅ Directly display the ApiModelForm in setup mode with navigation callbacks
- ✅ Skip functionality to proceed without configuration
- ✅ Navigation to setup/complete on completion or skip
- ✅ Help section with reassuring message about adding models later
- ✅ Verified build success - new route `/ui/setup/api-models` generated

### Phase 6: Configure ApiModelForm for Setup Mode ❌ CANCELLED
- ❌ Pre-selecting OpenAI is not required per user feedback
- ❌ Current setup mode behavior (empty provider selection) is preferred

## Implementation Summary

The new setup page (`/ui/setup/api-models`) includes:
- **Page Structure**: AppInitializer wrapper with authenticated access requirement
- **Progress Header**: SetupProgress component showing step 4 of 6
- **Logo**: BodhiLogo component for branding consistency
- **Welcome Card**: Clean introduction without benefit cards (as requested)
- **Main Form**: ApiModelForm component in setup mode with callbacks to setup/complete
- **Skip Button**: Allows users to proceed without configuring API models
- **Help Section**: Muted card with reassuring message about adding models later
- **Motion Animations**: Consistent with other setup pages using framer-motion
- **Navigation Flow**: Both form completion and cancellation lead to setup/complete

## Final Implementation Status: ✅ ALL PHASES COMPLETED

This refactoring successfully achieved maximum code reuse while meeting all setup flow requirements:

### Key Achievements
- ✅ **Single Source of Truth**: ApiModelForm component used across create, edit, and setup modes
- ✅ **Consistent Form Behavior**: Same validation, testing, fetching logic across all use cases
- ✅ **Setup-Specific UI**: Clean setup page without card selection, direct form interaction
- ✅ **Flexible Navigation**: Configurable success/cancel routes based on context
- ✅ **User Choice**: No pre-selection in setup mode - users choose their preferred provider
- ✅ **Skip Option**: Users can proceed without configuring API models
- ✅ **Complete Integration**: Navigation flow works seamlessly (download-models → api-models → complete)

### Code Reuse Metrics
- **Before**: 2 separate form implementations (ApiModelForm + ApiModelSetupForm)
- **After**: 1 shared form component with mode-based configuration
- **Lines Saved**: ~186 lines of duplicate code eliminated
- **Maintenance**: Single component to maintain instead of multiple variants

## Testing Implementation Plan

### Phase 7: Test Setup and Initial Structure ✅ COMPLETED (2 tests)
**Test 7.1**: Basic setup, imports, and mocks ✅
- ✅ Create `src/app/ui/setup/api-models/page.test.tsx`
- ✅ Import necessary dependencies from api-model test utilities
- ✅ Set up mocks for router, toast, and MSW server
- ✅ Mock SetupProgress and BodhiLogo components
- ✅ Write basic test setup

**Test 7.2**: Page renders with authentication requirement ✅
```typescript
it('renders page with correct authentication and app status requirements')
```
- ✅ Verify page renders inside AppInitializer
- ✅ Check that `api-models-setup-page` container is present
- ✅ Verify SetupProgress shows "Step 4 of 6 - API Models"
- ✅ Verify BodhiLogo is rendered
- ✅ Test passes successfully

### Phase 8: Page Structure and Initial State ✅ COMPLETED (1 merged test)
**Test 8.1**: Complete page structure verification ✅
```typescript
it('displays complete setup page structure with form in setup mode')
```
- ✅ Verify SetupProgress shows step 4 of 6 with "API Models" label
- ✅ Check BodhiLogo is rendered
- ✅ Verify form has `data-testid="setup-api-model-form"`
- ✅ Verify submit button shows "Create API Model" text (not "Complete Setup")
- ✅ Verify "Skip for Now" button exists (NO cancel button in form)
- ✅ Check initial field states (empty provider selection in setup mode)
- ✅ Fixed ApiModelForm to not show cancel button in setup mode
- ✅ Test passes successfully

### Phase 9: Skip Functionality ✅ COMPLETED (1 test)
**Test 9.1**: Skip button navigates to setup/complete ✅
```typescript
it('navigates to setup complete when skip button is clicked')
```
- ✅ Find and click skip button (`data-testid="skip-api-setup"`)
- ✅ Verify router.push called with `/ui/setup/complete`
- ✅ Verify no form submission occurs (no toast calls)
- ✅ Test passes successfully

### Phase 10: Form Initial State Validation ✅ COMPLETED (1 test)
**Test 10.1**: Form validation and initial button states ✅
```typescript
it('form shows correct initial field values and button states for setup mode')
```
- ✅ Verify API format selector is empty (setup mode starts with no provider)
- ✅ Check base URL and API key are empty
- ✅ Verify test connection button is disabled initially
- ✅ Verify fetch models button is disabled initially
- ✅ Check that submit button text is "Create API Model"
- ✅ Verify API key field is password type for security
- ✅ Test passes successfully

### Phase 11: Form Submission Success ✅ COMPLETED (1 test)
**Test 11.1**: Successful form submission navigates to setup/complete ✅
```typescript
it('successfully creates API model and redirects to setup complete')
```
- ✅ Select OpenAI API format (auto-populates base URL)
- ✅ Fill in API key
- ✅ Test connection (verify success)
- ✅ Fetch models (verify models loaded)
- ✅ Select a model (gpt-4)
- ✅ Submit form
- ✅ Verify success toast with "API Model Created"
- ✅ Verify navigation to `/ui/setup/complete` (not `/ui/models`)
- ✅ Test passes successfully

### Phase 12: Error Handling ✅ COMPLETED (1 test)
**Test 12.1**: Server error keeps user on setup page ✅
```typescript
it('handles server error during API model creation and stays on setup page')
```
- ✅ Fill valid form data (API format, key, test connection, fetch models, select model)
- ✅ Mock server error (500) for API model creation endpoint
- ✅ Submit form
- ✅ Verify error toast shown "Failed to Create API Model"
- ✅ Verify NO navigation occurs (stays on setup page)
- ✅ Verify form and skip button still visible for user recovery
- ✅ Test passes successfully

### Testing Implementation Strategy
1. **Reuse test utilities**: Import all helpers from `@/test-utils/api-model-test-utils`
2. **Mock setup components**: Mock SetupProgress and BodhiLogo to keep tests focused
3. **Use existing handlers**: Reuse `createApiModelHandlers` and `createTestHandlers`
4. **One test at a time**: Write, run, and fix each test before moving to next
5. **Follow new/page.test.tsx patterns**: Use same testing patterns but adapt for setup context

### Total: 6 Tests across 6 phases ✅ ALL COMPLETED

**Test Suite Summary:**
1. **Phase 7**: Authentication and initial render (1 test) ✅
2. **Phase 8**: Complete page structure verification (1 test) ✅
3. **Phase 9**: Skip button navigation (1 test) ✅
4. **Phase 10**: Form initial state validation (1 test) ✅
5. **Phase 11**: Successful form submission flow (1 test) ✅
6. **Phase 12**: Error handling and recovery (1 test) ✅

**Key Setup Mode Behaviors Tested:**
- **NO Cancel Button**: In setup mode, the form displays "Skip for Now" instead of Cancel ✅
- **Navigation Target**: All navigation goes to `/ui/setup/complete` not `/ui/models` ✅
- **Initial State**: Setup mode starts with empty provider selection (no OpenAI pre-selected) ✅
- **Button Text**: Submit button shows "Create API Model" (same as regular create mode) ✅
- **Error Recovery**: Users can retry or skip after server errors ✅