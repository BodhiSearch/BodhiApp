# Phase 8: Frontend Component Tests

## Purpose

Implement component tests for the review/approve page using React Testing Library and MSW v2 for API mocking.

## Dependencies

- **Phase 6**: Frontend review page implemented

## Testing Strategy

Follow existing frontend test patterns:
- **MSW v2** for API mocking (see existing hooks tests)
- **React Testing Library** for component rendering and interaction
- **Vitest** as test runner
- **data-testid** attributes for element selection (see playwright skill)

## Test Coverage

### 8a. Review Page Component Tests

**File**: `crates/bodhi/src/app/ui/apps/review-access/[id]/page.test.tsx` (new)

**Mock Handlers** (MSW v2):
```typescript
function mockAccessRequestReview(overrides?: Partial<AccessRequestReview>) {
  return http.get(
    '/bodhi/v1/apps/access-request/:id/review',
    () => HttpResponse.json({
      id: 'test-uuid',
      app_client_id: 'app-test-123',
      flow_type: 'popup',
      tools_requested: [
        {
          tool_type: 'builtin-exa-search',
          display_name: 'Exa Web Search',
          user_instances: [
            { id: 'instance-1', name: 'My Exa', enabled: true, has_api_key: true }
          ]
        }
      ],
      expires_at: Date.now() + 600000,
      ...overrides,
    })
  );
}

function mockApproveAccessRequest() {
  return http.post(
    '/bodhi/v1/apps/access-request/:id/approve',
    () => HttpResponse.json({
      resource_scope: 'scope_resource-xyz',
      access_request_scope: 'scope_access_request:test-uuid',
    })
  );
}

function mockDenyAccessRequest() {
  return http.post(
    '/bodhi/v1/apps/access-request/:id/deny',
    () => HttpResponse.json({ success: true })
  );
}
```

**Test Cases**:

1. **Rendering and Data Display**
   - `test_renders_app_client_id` — Shows app client ID
   - `test_renders_requested_tools` — Shows tool display names
   - `test_renders_user_instances` — Shows user's available instances in dropdown
   - `test_loading_state` — Shows loading skeleton during fetch

2. **User Interaction**
   - `test_user_selects_tool_instance` — Can select instance from dropdown
   - `test_approve_button_calls_api` — Approve button triggers API with selected instances
   - `test_deny_button_calls_api` — Deny button triggers API
   - `test_buttons_disabled_during_submission` — Loading state disables buttons

3. **Flow-Specific Behavior**
   - `test_popup_flow_closes_window` — After approve, calls window.close()
   - `test_redirect_flow_navigates` — After approve, sets window.location.href
   - (Mock window.close and window.location for testing)

4. **Error States**
   - `test_expired_request_shows_error` — Shows expired message
   - `test_not_found_shows_error` — Shows 404 error
   - `test_validation_error` — Shows validation error (e.g., no instance selected)
   - `test_network_error_shows_message` — Shows network error with retry option

5. **Authentication**
   - `test_redirects_to_login_when_unauthenticated` — 401 → redirect to login
   - `test_returns_after_login` — sessionStorage stores return URL
   - (Mock auth state and routing for testing)

### 8b. API Hook Tests (if not already covered)

**File**: `crates/bodhi/src/hooks/useQuery.test.ts` (extend if needed)

Test React Query hooks in isolation:
- `test_useAccessRequestReview_fetches_data`
- `test_useApproveAccessRequest_mutation`
- `test_useDenyAccessRequest_mutation`

(May already be covered by component tests — avoid duplication)

## Files to Create/Modify

| File | Action | Description |
|------|--------|-------------|
| `crates/bodhi/src/app/ui/apps/review-access/[id]/page.test.tsx` | Create | Component tests |
| `crates/bodhi/src/hooks/useQuery.test.ts` | Modify | Hook tests (if needed) |

## Research Questions

1. **MSW setup**: How is MSW v2 configured in existing tests? (Check test setup files)
2. **Router mocking**: How do we mock Next.js app router params/queries? (Check existing page tests)
3. **Window mocking**: How do we mock window.close() and window.location? (Check existing tests)
4. **Auth mocking**: How do we mock authenticated state? (Check existing auth-protected page tests)
5. **sessionStorage mocking**: How do we test sessionStorage usage? (Check existing storage tests)
6. **Loading states**: How are loading skeletons tested? (Check existing component tests)
7. **Error boundaries**: Do we need to test error boundaries? (Check existing error handling)

## Test Patterns

### Component Rendering
```typescript
import { render, screen, waitFor } from '@testing-library/react';
import { server } from '@/test/mocks/server';
import ReviewAccessPage from './page';

test('renders requested tools with display names', async () => {
  server.use(mockAccessRequestReview());

  render(<ReviewAccessPage params={{ id: 'test-uuid' }} searchParams={{ flow_type: 'popup' }} />);

  await waitFor(() => {
    expect(screen.getByTestId('app-client-id')).toHaveTextContent('app-test-123');
    expect(screen.getByTestId('tool-display-name-0')).toHaveTextContent('Exa Web Search');
  });
});
```

### User Interaction
```typescript
import { fireEvent } from '@testing-library/react';

test('approve button calls approve API with selected instances', async () => {
  server.use(mockAccessRequestReview(), mockApproveAccessRequest());

  render(<ReviewAccessPage params={{ id: 'test-uuid' }} searchParams={{ flow_type: 'popup' }} />);

  await waitFor(() => screen.getByTestId('tool-instance-select-0'));

  const select = screen.getByTestId('tool-instance-select-0');
  fireEvent.change(select, { target: { value: 'instance-1' } });

  const approveButton = screen.getByTestId('approve-button');
  fireEvent.click(approveButton);

  await waitFor(() => {
    // Verify API was called with correct data
    // Verify window.close() was called (mock)
  });
});
```

### Error Handling
```typescript
test('shows expired message when request is expired', async () => {
  server.use(mockAccessRequestReview({
    expires_at: Date.now() - 1000 // Expired
  }));

  render(<ReviewAccessPage params={{ id: 'test-uuid' }} searchParams={{ flow_type: 'popup' }} />);

  await waitFor(() => {
    expect(screen.getByTestId('error-message')).toHaveTextContent('expired');
  });
});
```

## Acceptance Criteria

### Test Coverage
- [ ] All user interactions tested (select, approve, deny)
- [ ] All UI states tested (loading, ready, submitting, success, error)
- [ ] Flow-specific behavior tested (popup vs redirect)
- [ ] Error states tested (expired, not found, validation, network)
- [ ] Auth flow tested (redirect to login, return after login)

### Code Quality
- [ ] Tests use data-testid for element selection
- [ ] No CSS selectors or implementation details in tests
- [ ] MSW v2 mocks follow existing patterns
- [ ] Clear test names describing scenario
- [ ] No console.log except for error scenarios (see CLAUDE.md)
- [ ] No inline timeouts (see CLAUDE.md)

### Test Execution
- [ ] `npm test` in crates/bodhi passes
- [ ] No flaky tests (deterministic)
- [ ] Tests run quickly (no unnecessary waits)

## Notes for Sub-Agent

- **Use playwright skill**: Review for data-testid patterns and testability guidelines
- **Follow existing patterns**: Study existing component tests in crates/bodhi/src
- **MSW v2**: Follow existing MSW v2 setup and handler patterns
- **React Testing Library**: Prefer `getByTestId` over other queries
- **User-centric tests**: Test user interactions, not implementation details
- **Mock window APIs**: window.close() and window.location need mocking for tests
- **No real timers**: Use fake timers if testing countdown/expiry display
- **No timeouts**: Don't add inline timeouts (see CLAUDE.md)

## Verification

```bash
cd crates/bodhi
npm test -- review-access
```

All tests should pass without warnings or errors.

## After Implementation

**Remember**: Run `make build.ui-clean && make build.ui` after frontend changes before running Playwright tests (see CLAUDE.md).

## Next Phase

Phase 9 will implement end-to-end tests with real Keycloak integration.
