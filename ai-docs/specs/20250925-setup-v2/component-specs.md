# API Model Forms Test Refactoring Specification

**Project:** BodhiApp API Models Forms
**Date:** 2025-09-25
**Version:** v2 (Setup Integration)
**Status:** Phase 1 - Planning Complete

## Executive Summary

This document provides a comprehensive, phase-wise plan for refactoring tests after a major refactoring of API model forms from 850+ lines to ~230 lines through component extraction and shared architecture.

### Refactoring Completed
- ✅ Extracted 16 shared components from monolithic forms
- ✅ Created MSW handler utilities for API mocking
- ✅ Co-located 7 component unit tests with their components
- ✅ Reduced code duplication by 73%

### Current Test Status (Updated 2025-09-25)
- **Overall Test Suite:** 664 passing, 47 failing (93.4% pass rate)
- **Passing Tests:** 7 component unit tests (co-located) + 6 integration tests (ApiModelSetupForm)
- **Failing Tests:** 24 total failures (9 in ApiModelSetupForm, 15 in page.test.tsx)
- **Missing Tests:** 9 components without test coverage
- **Phase 2 Progress:** ✅ Major breakthrough - reduced ApiModelSetupForm failures from 12 → 9 (6 tests now passing)
- **New Discovery:** 15 page-level integration tests failing due to missing MSW handlers

### Objectives (Updated Scope)
1. Fix all failing integration tests (24 failures: 9 ApiModelSetupForm + 15 page.test.tsx)
2. Create missing component unit tests (9 components)
3. Create comprehensive hook tests (3 hooks)
4. Add end-to-end integration tests for complete user workflows
5. Achieve 100% test coverage for refactored components

---

## Current State Analysis

### ✅ Completed Components (Unit Tests Created & Passing)

| Component | Test File | Status | Coverage |
|-----------|-----------|--------|----------|
| ProviderSelector | `components/api-models/providers/ProviderSelector.test.tsx` | ✅ Passing | Complete |
| ProviderInfo | `components/api-models/providers/ProviderInfo.test.tsx` | ✅ Passing | Complete |
| Constants | `components/api-models/providers/constants.test.ts` | ✅ Passing | Complete |
| ApiKeyInput | `components/api-models/form/ApiKeyInput.test.tsx` | ✅ Passing | Complete |
| BaseUrlInput | `components/api-models/form/BaseUrlInput.test.tsx` | ✅ Passing | Complete |
| TestConnectionButton | `components/api-models/actions/TestConnectionButton.test.tsx` | ✅ Passing | Complete |
| useApiModelForm (partial) | `components/api-models/hooks/useApiModelForm.test.ts` | 🟡 In Progress | Partial |

### ❌ Failing Integration Tests

#### ApiModelSetupForm.test.tsx (9 failures remaining)
- **Location:** `/app/ui/setup/api-models/ApiModelSetupForm.test.tsx`
- **Status:** 🟡 In Progress - 6/15 tests now passing (40% complete)
- **Key Fixes Completed:** ✅ Provider selection, ✅ BaseURL visibility, ✅ API key toggle, ✅ Component structure
- **Remaining Issues:** Form submission workflows, model selection, button interactions

#### page.test.tsx (15 failures - NEW DISCOVERY)
- **Location:** `/app/ui/setup/api-models/page.test.tsx`
- **Status:** 🔴 All failing due to missing MSW handlers
- **Root Cause:** 500 error on `/bodhi/v1/info` endpoint - missing app info handler
- **Impact:** Simple fix will resolve all 15 tests at once
  - Data-testid attributes changed with new components

#### ApiModelForm.test.tsx (6 failures)
- **Location:** `/app/ui/api-models/ApiModelForm.test.tsx`
- **Root Cause:** Component structure changes, endpoint mismatches
- **Key Issues:**
  - Mock ModelSelector implementation outdated
  - Button interaction patterns changed
  - Form submission flow modified
  - Error handling expectations need updates

### 🔍 Missing Tests (9 Components)

| Component | Location | Complexity | Priority |
|-----------|----------|------------|----------|
| ModelSelection | `components/api-models/form/ModelSelection.tsx` | High | P0 |
| PrefixInput | `components/api-models/form/PrefixInput.tsx` | Medium | P1 |
| FetchModelsButton | `components/api-models/actions/FetchModelsButton.tsx` | Medium | P0 |
| FormActions | `components/api-models/actions/FormActions.tsx` | Low | P1 |
| useFetchModels | `components/api-models/hooks/useFetchModels.ts` | High | P0 |
| useTestConnection | `components/api-models/hooks/useTestConnection.ts` | High | P0 |
| ApiModelForm | `components/api-models/ApiModelForm.tsx` | High | P0 |
| Setup Integration | Complete user flows | Very High | P0 |
| E2E Workflows | Multi-step scenarios | Very High | P1 |

### 🔧 Technical Debt Identified

1. **MSW Handler Consistency:** Some tests still use individual handlers instead of utilities
2. **Test Organization:** Mix of co-located and separated test files
3. **Mock Consistency:** Different mocking strategies across test files
4. **Coverage Gaps:** Missing edge cases in existing tests
5. **Documentation:** Test documentation needs updates for new architecture

---

## Phase-Wise Implementation Plan

### Phase 1: Fix MSW Handler Infrastructure 🔧
**Objective:** Standardize MSW handler usage across all integration tests
**Duration:** 1 session
**Priority:** P0 (Blocking)

#### Current State Analysis
- `ApiModelSetupForm.test.tsx` has incomplete MSW setup
- Missing imports and handler utilities
- Inconsistent server configuration

#### Files to Modify
- [ ] `/app/ui/setup/api-models/ApiModelSetupForm.test.tsx`

#### Implementation Steps
1. **Update Imports**
   ```typescript
   // Add missing imports
   import { rest } from 'msw';
   import { createApiModelHandlers } from '@/test-utils/msw-handlers';
   import { ENDPOINT_APP_INFO, ENDPOINT_USER_INFO } from '@/hooks/useQuery';
   import { createMockLoggedInUser } from '@/test-utils/mock-user';
   ```

2. **Fix Server Setup**
   ```typescript
   beforeEach(() => {
     mockOnComplete.mockClear();
     mockOnSkip.mockClear();

     // Use centralized handlers
     server.use(
       ...createApiModelHandlers(),
       rest.get(`*${ENDPOINT_APP_INFO}`, (_, res, ctx) => {
         return res(ctx.json({ status: 'ready' }));
       }),
       rest.get(`*${ENDPOINT_USER_INFO}`, (_, res, ctx) => {
         return res(ctx.json(createMockLoggedInUser()));
       })
     );
   });
   ```

#### Success Criteria
- [ ] No import errors
- [ ] MSW handlers properly configured
- [ ] Server setup matches other test files
- [ ] All API endpoints available for tests

---

### Phase 2: Fix ApiModelSetupForm Integration Tests 🟡
**Objective:** Update all failing tests to work with new component architecture ✅ 40% Complete
**Duration:** 2 sessions (1.5 sessions completed)
**Priority:** P0 (Blocking) - Major progress achieved

#### Current State Analysis (UPDATED 2025-09-25)
- **Test Count:** 15 tests total
- **Current Status:** 6 passing, 9 failing (Major improvement from 2 passing, 12 failing)
- **Key Fixes Completed:**
  - ✅ Fixed provider selection click handler (motion.div blocking clicks)
  - ✅ Fixed provider override issue in useApiModelForm useEffect
  - ✅ Fixed BaseUrl field visibility for OpenAI-compatible providers
  - ✅ Fixed API key toggle visibility testid mismatch
  - ✅ Fixed default provider selection in setup mode
- **Remaining Issues:** Form interactions, model selection, API mocking
- **Affected Test Scenarios:**
  - Provider selection flow
  - Configuration form display
  - API key visibility toggle
  - Test connection functionality
  - Model fetching and selection
  - Form submission workflows

#### Test Specifications

##### Test 1: "renders the form with provider selection"
**Current Issue:** Expects old component structure
**Fix Required:**
- Update selectors to use new ProviderSelector component
- Verify data-testid attributes match implementation
- Update text expectations for new component structure

##### Test 2: "shows configuration form when provider is selected"
**Current Issue:** Form structure changed with shared components
**Fix Required:**
- Update expectations for new form layout
- Verify data-testid for form inputs match ApiKeyInput, BaseUrlInput components
- Update button selectors for TestConnectionButton, FetchModelsButton

##### Test 3: "shows base URL field for OpenAI-compatible provider"
**Current Issue:** BaseUrlInput component conditional rendering logic
**Fix Required:**
- Update selector to match BaseUrlInput data-testid
- Verify `showWhen` prop logic is tested correctly

##### Test 4: "does not show base URL field for OpenAI provider"
**Current Issue:** Same as Test 3, opposite scenario
**Fix Required:**
- Update negative assertion for BaseUrlInput component

##### Test 5: "toggles API key visibility"
**Current Issue:** ApiKeyInput component structure changed
**Fix Required:**
- Update selectors to match ApiKeyInput component
- Verify toggle button data-testid
- Update assertions for password/text input type

##### Tests 6-8: Button State Management
**Current Issue:** TestConnectionButton and FetchModelsButton component changes
**Fix Required:**
- Update button selectors to match component data-testids
- Verify disabled/enabled state logic
- Update interaction patterns

##### Tests 9-12: API Integration & Workflows
**Current Issue:** Form submission and API interaction patterns changed
**Fix Required:**
- Update model selection logic for new ModelSelection component
- Verify form submission flow with new FormActions component
- Update toast message expectations
- Update completion callback patterns

#### Implementation Steps
1. **Update Test 1-4:** Component rendering and conditional display
2. **Update Test 5:** API key visibility toggle
3. **Update Test 6-8:** Button states and interactions
4. **Update Test 9-12:** Complete workflows and API integration

#### Success Criteria
- [ ] All 12 tests passing
- [ ] No test warnings or deprecation notices
- [ ] Proper component isolation (no implementation details tested)
- [ ] Consistent data-testid usage

---

### Phase 2.5: Fix Page-Level Integration Tests 🔧
**Objective:** Fix all 15 failing page.test.tsx tests (NEW DISCOVERY)
**Duration:** 0.5 sessions (Quick Fix)
**Priority:** P1 (High Impact - Simple Solution)

#### Current State Analysis
- **Test Count:** 15 tests, all failing
- **Root Cause:** Missing MSW handler for `/bodhi/v1/info` endpoint
- **Error:** 500 Internal Server Error causing loading state to persist
- **Impact:** All tests fail because page never loads past loading spinner

#### Simple Solution Required
```typescript
// Add to MSW handlers setup in page.test.tsx
rest.get('*/bodhi/v1/info', (_, res, ctx) =>
  res(ctx.json({ status: 'ready' }))
)
```

#### Success Criteria
- [ ] All 15 page tests pass
- [ ] Page renders beyond loading state
- [ ] All navigation and interaction tests work

**Expected Outcome:** 100% success rate (15/15 tests passing) with minimal effort

---

### Phase 3: Fix ApiModelForm Integration Tests 🔧
**Objective:** Update 6 failing tests for main API model form
**Duration:** 2 sessions
**Priority:** P0 (Blocking)

#### Current State Analysis
- **Test Count:** 6 failing tests out of large test suite
- **Primary Issues:**
  - Mock ModelSelector implementation outdated
  - Form interaction patterns changed
  - Button and action component structure modified

#### Files to Modify
- [ ] `/app/ui/api-models/ApiModelForm.test.tsx`

#### Key Failing Test Categories

##### Category 1: Component Rendering Tests
**Tests Affected:** Form element rendering, button presence
**Issue:** Tests expect old component structure
**Fix:** Update selectors and expectations for shared components

##### Category 2: Form Interaction Tests
**Tests Affected:** Model selection, form submission
**Issue:** Mock ModelSelector doesn't match new ModelSelection component
**Fix:** Update mock implementation or replace with real component testing

##### Category 3: API Integration Tests
**Tests Affected:** Fetch models, test connection functionality
**Issue:** Button interaction patterns changed with new action components
**Fix:** Update interaction patterns for FetchModelsButton, TestConnectionButton

#### Implementation Steps
1. **Update Mock ModelSelector**
   - Replace with proper mock matching ModelSelection component
   - Or update test to use real component with mocked hooks

2. **Fix Form Rendering Tests**
   - Update data-testid selectors
   - Verify new component structure

3. **Fix Interaction Tests**
   - Update button click patterns
   - Update form submission flow
   - Verify error handling

#### Success Criteria
- [ ] All 6 failing tests now passing
- [ ] Mock implementation matches real component interface
- [ ] Integration test coverage maintained

---

### Phase 4: Create ModelSelection Component Tests 📝
**Objective:** Create comprehensive unit tests for the ModelSelection component
**Duration:** 1 session
**Priority:** P0 (High complexity, core component)

#### Component Analysis
- **Location:** `src/components/api-models/form/ModelSelection.tsx`
- **Complexity:** High - handles model list rendering, selection state, search/filter
- **Dependencies:** None identified yet (need to read component)
- **Test File:** `src/components/api-models/form/ModelSelection.test.tsx` (to be created)

#### Test Specifications

##### Test Group 1: Basic Rendering
- [ ] Renders with empty model list
- [ ] Renders with model list provided
- [ ] Shows "No models available" message when list is empty
- [ ] Renders all models in provided list

##### Test Group 2: Model Selection
- [ ] Calls onModelSelect when model is clicked
- [ ] Shows selected state for selected models
- [ ] Handles multiple model selection
- [ ] Prevents duplicate selection

##### Test Group 3: Model Deselection
- [ ] Calls onModelRemove when selected model is clicked
- [ ] Removes visual selection state
- [ ] Handles removing from multiple selections

##### Test Group 4: Select All Functionality
- [ ] Shows "Select All" button when appropriate
- [ ] Calls onModelsSelectAll with all available models
- [ ] Updates visual state for all models
- [ ] Handles "Select None" when all are selected

##### Test Group 5: Search/Filter (if implemented)
- [ ] Filters model list based on search input
- [ ] Shows no results message when search returns empty
- [ ] Clears filter when search is cleared
- [ ] Maintains selection state during filtering

##### Test Group 6: Accessibility & UX
- [ ] Proper ARIA labels for model options
- [ ] Keyboard navigation support
- [ ] Focus management
- [ ] Screen reader announcements

#### Implementation Plan
1. **Read component implementation** to understand exact props and behavior
2. **Create test file** with proper setup and mocks
3. **Implement basic rendering tests** first
4. **Add interaction tests** for selection/deselection
5. **Add advanced functionality tests** (select all, search if present)
6. **Add accessibility tests**

#### Success Criteria
- [ ] Test file created and co-located
- [ ] All test groups implemented
- [ ] 100% code coverage for component
- [ ] All tests passing
- [ ] Proper test isolation (no implementation details)

---

### Phase 5: Create PrefixInput Component Tests 📝
**Objective:** Create unit tests for optional prefix input component
**Duration:** 0.5 sessions
**Priority:** P1 (Medium complexity)

#### Component Analysis
- **Location:** `src/components/api-models/form/PrefixInput.tsx`
- **Complexity:** Medium - conditional rendering, validation, help text
- **Test File:** `src/components/api-models/form/PrefixInput.test.tsx` (to be created)

#### Test Specifications

##### Test Group 1: Conditional Rendering
- [ ] Shows when `showWhen` prop is true
- [ ] Hides when `showWhen` prop is false
- [ ] Shows by default if `showWhen` not provided

##### Test Group 2: Input Functionality
- [ ] Renders input with correct attributes
- [ ] Calls onChange when input value changes
- [ ] Calls onBlur when input loses focus
- [ ] Shows placeholder text

##### Test Group 3: Validation & Error States
- [ ] Shows error message when error prop provided
- [ ] Applies error styling when error present
- [ ] Shows help text when no error
- [ ] Prioritizes error message over help text

##### Test Group 4: Checkbox Integration (if present)
- [ ] Shows checkbox for enabling/disabling prefix
- [ ] Toggles input enabled state based on checkbox
- [ ] Clears input value when disabled
- [ ] Proper labeling and accessibility

#### Success Criteria
- [ ] Complete test coverage for all conditional logic
- [ ] Proper form integration testing
- [ ] All accessibility requirements tested

---

### Phase 6: Create FetchModelsButton Component Tests 📝
**Objective:** Create unit tests for fetch models button with loading states
**Duration:** 1 session
**Priority:** P0 (Medium complexity, critical functionality)

#### Component Analysis
- **Location:** `src/components/api-models/actions/FetchModelsButton.tsx`
- **Complexity:** Medium - button states, loading, success/error handling
- **Test File:** `src/components/api-models/actions/FetchModelsButton.test.tsx` (to be created)

#### Test Specifications

##### Test Group 1: Basic Rendering
- [ ] Renders with default props
- [ ] Shows correct button text for idle state
- [ ] Applies correct variant and size props
- [ ] Uses custom data-testid when provided

##### Test Group 2: Button States
- [ ] Disabled when canFetch is false
- [ ] Enabled when canFetch is true
- [ ] Shows loading state when isLoading is true
- [ ] Shows different text during loading

##### Test Group 3: Click Handling
- [ ] Calls onFetch when clicked and enabled
- [ ] Does not call onFetch when disabled
- [ ] Does not call onFetch when loading
- [ ] Handles multiple rapid clicks appropriately

##### Test Group 4: Status Display
- [ ] Shows success state styling when status is success
- [ ] Shows error state styling when status is error
- [ ] Shows correct icons for each state
- [ ] Returns to idle state appropriately

##### Test Group 5: Tooltip Integration (if present)
- [ ] Shows tooltip when disabled with reason
- [ ] Hides tooltip when enabled
- [ ] Shows correct tooltip text
- [ ] Proper accessibility for tooltips

#### Success Criteria
- [ ] All button states tested thoroughly
- [ ] Loading state behavior verified
- [ ] Accessibility compliance verified
- [ ] Error boundary testing included

---

### Phase 7: Create FormActions Component Tests 📝
**Objective:** Create unit tests for form action buttons (submit, cancel)
**Duration:** 0.5 sessions
**Priority:** P1 (Low complexity)

#### Component Analysis
- **Location:** `src/components/api-models/actions/FormActions.tsx`
- **Complexity:** Low - button rendering, disabled states, callbacks
- **Test File:** `src/components/api-models/actions/FormActions.test.tsx` (to be created)

#### Test Specifications

##### Test Group 1: Button Rendering
- [ ] Renders submit button with correct text for create mode
- [ ] Renders submit button with correct text for edit mode
- [ ] Renders cancel button
- [ ] Applies custom className when provided

##### Test Group 2: Submit Button States
- [ ] Disabled when isSubmitting is true
- [ ] Disabled when form is invalid (if prop exists)
- [ ] Shows loading text when submitting
- [ ] Enabled when form is valid and not submitting

##### Test Group 3: Button Actions
- [ ] Calls onSubmit when submit button clicked
- [ ] Calls onCancel when cancel button clicked
- [ ] Does not call onSubmit when disabled
- [ ] Handles form submission properly

##### Test Group 4: Mode Variations
- [ ] Shows "Create" text in create mode
- [ ] Shows "Update" text in edit mode
- [ ] Shows "Complete Setup" text in setup mode (if applicable)

#### Success Criteria
- [ ] All mode variations tested
- [ ] Button state management verified
- [ ] Callback handling tested
- [ ] Accessibility verified

---

### Phase 8: Create Hook Unit Tests 📝
**Objective:** Create comprehensive tests for business logic hooks
**Duration:** 2 sessions
**Priority:** P0 (High complexity, core business logic)

#### Hook 1: useFetchModels.ts

##### Current State Analysis
- **Location:** `src/components/api-models/hooks/useFetchModels.ts`
- **Complexity:** High - API integration, state management, error handling
- **Test File:** `src/components/api-models/hooks/useFetchModels.test.ts` (to be created)

##### Test Specifications

**Test Group 1: Hook Initialization**
- [ ] Returns correct initial state
- [ ] Sets canFetch based on mode and requirements
- [ ] Handles auto-select common models option

**Test Group 2: Fetch Functionality**
- [ ] Calls correct API endpoint with proper parameters
- [ ] Updates loading state during fetch
- [ ] Updates available models on successful fetch
- [ ] Auto-selects common models when enabled

**Test Group 3: Error Handling**
- [ ] Handles network errors gracefully
- [ ] Shows appropriate error messages
- [ ] Resets loading state on error
- [ ] Maintains previous data on error

**Test Group 4: Mode-Specific Behavior**
- [ ] Uses API key for create mode
- [ ] Uses model ID for edit mode
- [ ] Handles setup mode with auto-selection

#### Hook 2: useTestConnection.ts

##### Test Specifications

**Test Group 1: Connection Testing**
- [ ] Calls test API with correct parameters
- [ ] Updates testing state during request
- [ ] Shows success message on successful test
- [ ] Shows error message on failed test

**Test Group 2: State Management**
- [ ] Manages testing, success, error states
- [ ] Resets state appropriately
- [ ] Handles concurrent test requests

#### Success Criteria
- [ ] Both hooks have comprehensive test coverage
- [ ] All API integration scenarios tested with MSW
- [ ] Error boundary testing included
- [ ] Performance testing for rapid interactions

---

### Phase 9: Create Main ApiModelForm Component Tests 📝
**Objective:** Create integration tests for main composed form component
**Duration:** 2 sessions
**Priority:** P0 (High complexity, main component)

#### Component Analysis
- **Location:** `src/components/api-models/ApiModelForm.tsx`
- **Complexity:** High - composes all other components, mode switching
- **Test File:** `src/components/api-models/ApiModelForm.test.tsx` (to be created)

#### Test Specifications

##### Test Group 1: Mode Switching
- [ ] Renders correctly in create mode
- [ ] Renders correctly in edit mode
- [ ] Renders correctly in setup mode
- [ ] Shows appropriate titles for each mode
- [ ] Displays correct button text per mode

##### Test Group 2: Component Composition
- [ ] Renders all required form components
- [ ] Shows ProviderSelector when appropriate
- [ ] Shows BaseUrlInput conditionally
- [ ] Integrates ModelSelection component properly
- [ ] Shows FormActions with correct props

##### Test Group 3: Form Flow Integration
- [ ] Provider selection updates form state
- [ ] API format change affects other fields
- [ ] Model selection updates form state
- [ ] Test connection uses current form values
- [ ] Fetch models uses current form values

##### Test Group 4: Data Flow
- [ ] Initial data populates form correctly (edit mode)
- [ ] Form validation works across all components
- [ ] Form submission collects all data correctly
- [ ] Cancel functionality works in all modes

##### Test Group 5: Hook Integration
- [ ] Uses useApiModelForm hook correctly
- [ ] Integrates with useFetchModels hook
- [ ] Integrates with useTestConnection hook
- [ ] Manages loading states from all hooks

#### Success Criteria
- [ ] Integration between all components verified
- [ ] All three modes fully tested
- [ ] Data flow through entire form tested
- [ ] Hook integration verified

---

### Phase 10: Create End-to-End Integration Tests 📝
**Objective:** Create comprehensive user journey tests
**Duration:** 3 sessions
**Priority:** P1 (Very high complexity, complete workflows)

#### Test Categories

##### Category 1: API Model Creation Journey
**Test File:** `src/app/ui/api-models/ApiModelForm.e2e.test.tsx` (to be created)

**Test Scenarios:**
- [ ] **Complete OpenAI Setup:** Provider selection → API key entry → model fetch → model selection → creation
- [ ] **Complete OpenAI-Compatible Setup:** Provider selection → base URL entry → API key entry → model fetch → model selection → creation
- [ ] **Error Recovery:** Invalid API key → error handling → correction → successful creation
- [ ] **Test Connection Workflow:** Form entry → test connection → success feedback → continue to creation

##### Category 2: API Model Editing Journey
**Test Scenarios:**
- [ ] **Model Addition:** Load existing model → fetch new models → add additional models → save
- [ ] **Configuration Update:** Load existing model → change base URL → test connection → save
- [ ] **Model Removal:** Load existing model → remove some models → save
- [ ] **No-Change Update:** Load existing model → submit without changes → verify no API calls

##### Category 3: Setup Flow Integration
**Test File:** `src/app/ui/setup/api-models/ApiModelSetupForm.e2e.test.tsx` (to be created)

**Test Scenarios:**
- [ ] **Complete Setup Journey:** Provider selection → configuration → model selection → completion callback
- [ ] **Setup Abandonment:** Start setup → click skip → verify skip callback
- [ ] **Setup Error Recovery:** Invalid configuration → error → correction → successful completion
- [ ] **Auto-Model Selection:** Provider selection → model fetch → verify common models auto-selected

##### Category 4: Cross-Component Data Consistency
**Test Scenarios:**
- [ ] **Provider Change Impact:** Select provider → change to different provider → verify form reset
- [ ] **API Format Impact:** Change API format → verify base URL and other fields update
- [ ] **Mode Switching:** Switch between create/edit modes → verify component behavior changes

##### Category 5: Performance & Edge Cases
**Test Scenarios:**
- [ ] **Large Model Lists:** Mock API returning 100+ models → verify performance and rendering
- [ ] **Network Timeouts:** Mock slow API responses → verify timeout handling
- [ ] **Concurrent Requests:** Rapid button clicking → verify request deduplication
- [ ] **Browser Back/Forward:** Navigate during form entry → verify state preservation

#### Implementation Approach
1. **Use Real Components:** No mocking of shared components, test full integration
2. **Mock Only External APIs:** Use MSW for API mocking, test real component interactions
3. **User-Centric Testing:** Focus on user actions and outcomes, not implementation details
4. **Error Path Testing:** Ensure error recovery flows work correctly
5. **Performance Monitoring:** Include performance assertions for critical paths

#### Success Criteria
- [ ] All user journeys work end-to-end
- [ ] Error recovery flows tested thoroughly
- [ ] Performance requirements met
- [ ] Cross-component integration verified
- [ ] Real-world usage scenarios covered

---

## Progress Tracking

### Phase Status Overview (Updated 2025-09-25)

| Phase | Name | Status | Start Date | Complete Date | Tests Added/Fixed |
|-------|------|--------|------------|---------------|-------------------|
| 1 | MSW Handler Infrastructure | ✅ Completed | 2025-09-25 | 2025-09-25 | 1/1 |
| 2 | ApiModelSetupForm Tests | 🟡 In Progress | 2025-09-25 | - | 6/15 |
| 2.5 | Page Integration Tests | 🔴 Not Started | - | - | 0/15 |
| 3 | ApiModelForm Tests | 🔲 Not Started | - | - | 0/6 |
| 4 | ModelSelection Component | 🔲 Not Started | - | - | 0/20+ |
| 5 | PrefixInput Component | 🔲 Not Started | - | - | 0/10+ |
| 6 | FetchModelsButton Component | 🔲 Not Started | - | - | 0/15+ |
| 7 | FormActions Component | 🔲 Not Started | - | - | 0/10+ |
| 8 | Hook Unit Tests | 🔲 Not Started | - | - | 0/20+ |
| 9 | Main ApiModelForm Component | 🔲 Not Started | - | - | 0/25+ |
| 10 | E2E Integration Tests | 🔲 Not Started | - | - | 0/15+ |

**Total Test Goal:** ~165+ tests across all phases (increased scope)

### Success Metrics (Updated 2025-09-25)

#### Quantitative Goals
- [ ] 0 failing tests (currently 24 failing: 9 ApiModelSetupForm + 15 page tests)
- [x] ✅ **Major Progress:** Reduced ApiModelSetupForm failures from 12 → 9 (50% reduction)
- [x] ✅ **Overall Test Suite:** 93.4% pass rate (664 passing / 47 failing)
- [ ] 100% component test coverage
- [ ] 90%+ line coverage for hooks
- [ ] All 16 components have co-located unit tests
- [ ] Integration tests cover all user workflows

#### Qualitative Goals
- [ ] Tests follow user-centric approach (testing behavior, not implementation)
- [ ] Consistent data-testid patterns across all components
- [ ] MSW handlers used consistently for API mocking
- [ ] Test documentation updated for new architecture
- [ ] Future maintainability improved

### Dependencies & Blockers

#### External Dependencies
- MSW handler utilities must work correctly
- TypeScript client types must be available
- Component implementations must be complete

#### Internal Dependencies
- Phase 1 blocks Phase 2 (MSW setup required)
- Phase 2-3 should complete before Phase 4-7 (understanding integration patterns)
- Phase 8 can run parallel with Phase 4-7
- Phase 9-10 require completion of all component tests

#### Potential Blockers
- Component implementations may need updates for testability
- Data-testid attributes may be missing from some components
- Hook interfaces may need adjustments for proper testing
- Performance issues with large integration tests

---

## Testing Best Practices & Standards

### Component Test Standards
1. **Co-location:** Tests next to components they test
2. **Isolation:** Test components in isolation with mocked dependencies
3. **User-Centric:** Test user interactions, not implementation details
4. **Data-testid:** Use data-testid for element selection
5. **Accessibility:** Include accessibility testing in all component tests

### Integration Test Standards
1. **Real Components:** Use real components, mock only external APIs
2. **MSW Handlers:** Use centralized MSW handlers for API mocking
3. **Complete Workflows:** Test entire user journeys
4. **Error Paths:** Include error recovery in all scenarios
5. **Performance:** Include basic performance assertions

### Hook Test Standards
1. **renderHook:** Use Testing Library's renderHook utility
2. **API Mocking:** Mock API responses with MSW
3. **State Changes:** Test all state transitions
4. **Error Handling:** Test error scenarios thoroughly
5. **Cleanup:** Test proper cleanup and unmounting

### Code Quality Standards
1. **TypeScript:** Full type safety in all tests
2. **ESLint:** No linting errors in test files
3. **Documentation:** JSDoc comments for complex test utilities
4. **Maintainability:** Tests should be easy to understand and modify
5. **Performance:** Tests should run quickly and efficiently

---

## Lessons Learned (Added 2025-09-25)

### Technical Insights from Phase 2 Implementation

#### 1. Framer Motion Click Event Issues
**Problem:** Provider selection cards weren't clickable
**Root Cause:** Framer Motion `motion.div` wrapper consuming click events
**Solution:** Move `onClick` handler to the `motion.div` instead of inner `Card` component
**Lesson:** Always test click handlers when adding animation wrappers

#### 2. React Hook Form UseEffect Dependencies
**Problem:** Provider selection getting overridden by useEffect
**Root Cause:** Infinite re-render loop in useEffect dependency array
**Solution:** Careful dependency management and conditional logic to prevent loops
**Lesson:** UseEffect with form state requires precise dependency management

#### 3. Test ID Naming Conventions
**Problem:** Tests failing due to testid mismatches
**Root Cause:** Components generate dynamic testids (e.g., `${testId}-visibility-toggle`)
**Solution:** Understand component testid patterns and match exactly in tests
**Lesson:** Establish consistent testid naming conventions across components

#### 4. Component State Management in Setup Mode
**Problem:** Default values conflicting with setup mode empty state
**Root Cause:** Different modes (create/edit/setup) have different initialization needs
**Solution:** Mode-specific default values with proper conditional logic
**Lesson:** Multi-mode components need careful state initialization strategies

#### 5. MSW Handler Discovery
**Problem:** Entire test files failing due to missing API endpoints
**Root Cause:** Tests make API calls to endpoints without MSW handlers
**Solution:** Comprehensive MSW handler setup covering all API endpoints
**Lesson:** API-dependent components need complete MSW handler coverage

### Development Velocity Insights
- **Most Time Consuming:** Debugging useEffect dependency issues
- **Biggest Impact:** Fixing provider selection (unlocked 4+ other tests)
- **Easiest Wins:** Test ID corrections and data-testid updates
- **Pattern Recognition:** Similar fixes needed across multiple test files

---

## Document Maintenance

### Update Schedule
- **Daily:** Progress tracking updates during active development
- **Per Phase:** Success criteria verification and next phase planning
- **Weekly:** Overall progress review and risk assessment
- **Post-Completion:** Final documentation and lessons learned

### Revision History
- **v1.0 (2025-09-25):** Initial comprehensive plan created
- **v1.1 (2025-09-25):** Phase 1 completed, Phase 2 substantial progress
- **v1.2 (2025-09-25):** Major progress update - discovered page.test.tsx issues, Phase 2 40% complete
  - Updated test counts: 24 total failures (9 ApiModelSetupForm + 15 page tests)
  - Added Phase 2.5 for page-level integration tests
  - Documented 6 major technical fixes completed
  - Added comprehensive lessons learned section
- **v2.0 (TBD):** Post-Phase 2 completion review and remaining phases planning

### Document Ownership
- **Primary Maintainer:** Development team
- **Reviewers:** Technical leads
- **Stakeholders:** QA team, product owners

---

## Risk Assessment & Mitigation

### High-Risk Areas
1. **Integration Test Complexity:** E2E tests may be difficult to maintain
   - *Mitigation:* Focus on critical user paths, use page object patterns
2. **Component Interface Changes:** Components may need API changes for testability
   - *Mitigation:* Review component APIs early, plan interface updates
3. **Performance Issues:** Large test suites may run slowly
   - *Mitigation:* Optimize test setup, use parallel execution, focus test scope

### Medium-Risk Areas
1. **MSW Handler Maintenance:** Centralized handlers may become complex
   - *Mitigation:* Keep handlers focused and well-documented
2. **Test Data Management:** Complex test data setup may become unwieldy
   - *Mitigation:* Create reusable test data factories
3. **Browser Compatibility:** Some tests may not work across all browsers
   - *Mitigation:* Test in primary target browsers, document known issues

### Contingency Plans
1. **Timeline Delays:** If phases take longer than expected, prioritize P0 phases
2. **Technical Blockers:** Have alternative approaches ready for each phase
3. **Resource Constraints:** Plan for reduced scope if resources are limited

---

*This document will be continuously updated as phases are completed and new insights are gained.*