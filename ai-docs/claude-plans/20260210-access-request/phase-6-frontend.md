# Phase 6: Frontend — Review/Approve Page

## Purpose

Implement the user-facing review and approval page where users can see what tools an app is requesting, select which instances to grant access to, and approve or deny the request.

## Dependencies

- **Phase 3**: Backend API endpoints for review, approve, deny implemented

## Key Components

### 6a. Review Page Component

**File**: `crates/bodhi/src/app/ui/apps/review-access/[id]/page.tsx` (new)

**Behavior**:
1. Extract `id` from URL params, `flow_type` from query params
2. Check if user is logged in
   - If not → store current URL in sessionStorage → redirect to login
   - After login → redirect back to review page (use existing `AppInitializer` pattern)
3. Fetch `GET /bodhi/v1/apps/access-request/{id}/review`
4. Display UI:
   - App name/client ID
   - List of requested tools with display names
   - For each tool type: dropdown/radio to select which user instance to grant
   - Approve button
   - Deny button
   - Expiry countdown (optional nice-to-have)
5. On Approve:
   - Call `POST /bodhi/v1/apps/access-request/{id}/approve` with selected instance UUIDs
   - On success: handle post-approval action based on flow_type
6. On Deny:
   - Call `POST /bodhi/v1/apps/access-request/{id}/deny`
   - On success: handle post-denial action based on flow_type
7. Post-action behavior:
   - If `flow_type == "redirect"` → `window.location.href = redirect_uri`
   - If `flow_type == "popup"` → `window.close()`

**States to handle**:
- Loading (fetching review data)
- Error (request not found, expired, network error)
- Ready (showing review form)
- Submitting (approve/deny in progress)
- Success (approved/denied, handling redirect/close)

### 6b. API Hooks

**File**: `crates/bodhi/src/hooks/useQuery.ts` (extend)

Add React Query hooks:
```typescript
// Fetch review data
export function useAccessRequestReview(id: string) {
  return useQuery({
    queryKey: ['access-request-review', id],
    queryFn: () => fetch(`/bodhi/v1/apps/access-request/${id}/review`).then(r => r.json()),
  });
}

// Approve mutation
export function useApproveAccessRequest(id: string, options?: UseMutationOptions) {
  return useMutation({
    mutationFn: (tools_approved: string[]) =>
      fetch(`/bodhi/v1/apps/access-request/${id}/approve`, {
        method: 'POST',
        body: JSON.stringify({ tools_approved }),
      }).then(r => r.json()),
    ...options,
  });
}

// Deny mutation
export function useDenyAccessRequest(id: string, options?: UseMutationOptions) {
  return useMutation({
    mutationFn: () =>
      fetch(`/bodhi/v1/apps/access-request/${id}/deny`, {
        method: 'POST',
      }).then(r => r.json()),
    ...options,
  });
}
```

### 6c. Route Constant

**File**: `crates/bodhi/src/lib/constants.ts`

Add:
```typescript
export const ROUTE_APP_REVIEW_ACCESS = '/ui/apps/review-access';
```

### 6d. UI Components (if needed)

Consider creating reusable components:
- `ToolInstanceSelector` — dropdown for selecting tool instance
- `AccessRequestExpiry` — countdown timer display
- `AccessRequestError` — error state display

Or implement inline if simple enough.

## Files to Create/Modify

| File | Action | Description |
|------|--------|-------------|
| `crates/bodhi/src/app/ui/apps/review-access/[id]/page.tsx` | Create | Main review page |
| `crates/bodhi/src/hooks/useQuery.ts` | Modify | Add API hooks |
| `crates/bodhi/src/lib/constants.ts` | Modify | Add route constant |

## Research Questions

1. **Login redirect**: How does existing code handle "login required" redirects? (Check `AppInitializer` or similar)
2. **Session storage**: What's the pattern for storing return URLs? (Check existing login flow)
3. **Query params**: How do we extract query params in Next.js 14 app router? (Check existing pages)
4. **URL params**: How do we extract `[id]` param? (Check existing dynamic routes)
5. **API client**: Do we have a centralized API client, or direct fetch? (Check existing hooks)
6. **Error handling**: What's the UI pattern for displaying API errors? (Check existing error components)
7. **Button states**: What's the pattern for disabled/loading button states? (Check existing forms)
8. **Shadcn components**: Which Shadcn UI components should we use? (Select, Button, Card, etc.)

## UI/UX Considerations

### Layout
- Clean, focused interface — user should clearly understand what they're approving
- App name/ID prominently displayed
- Each tool type as a card or section
- Clear call-to-action buttons

### Tool Instance Selection
- Dropdown or radio buttons for selecting instance
- Show instance name (user-friendly label)
- Disable/grey out instances without API key or that are disabled
- Help text explaining what access the app will have

### Error States
- Request not found → clear message with link back to home
- Request expired → show expiry time, explain what happened
- Network error → retry button
- Validation error (e.g., must select instance) → inline error messages

### Loading States
- Skeleton loader while fetching review data
- Disabled buttons with spinner during approve/deny submission
- Prevent double-submission

### Accessibility
- Proper ARIA labels
- Keyboard navigation
- Focus management (auto-focus first input, trap focus in popup mode)
- Screen reader announcements for state changes

## Acceptance Criteria

### Functionality
- [ ] Page loads and fetches review data
- [ ] Displays app client ID and requested tools
- [ ] Shows user's available instances per tool type
- [ ] User can select which instances to grant
- [ ] Approve button submits selection and handles response
- [ ] Deny button denies request
- [ ] Redirect flow: navigates to redirect_uri after action
- [ ] Popup flow: closes window after action
- [ ] Login redirect works for unauthenticated users
- [ ] Returns to review page after login

### Error Handling
- [ ] Shows error for expired request
- [ ] Shows error for not found request
- [ ] Shows validation error if no instance selected
- [ ] Handles network errors gracefully
- [ ] Clear error messages for all failure cases

### UI/UX
- [ ] Clean, intuitive interface
- [ ] Loading states during API calls
- [ ] Disabled states for invalid selections
- [ ] Responsive design (works on mobile/desktop)
- [ ] Consistent with existing BodhiApp UI patterns
- [ ] Uses Shadcn UI components

### Testing (Phase 8)
- [ ] Component tests with MSW mocking (defer to Phase 8)
- [ ] Test data-testid attributes for Playwright (defer to Phase 9)

## Notes for Sub-Agent

- **Follow existing patterns**: Look at existing pages for routing, auth, API calls
- **Use Shadcn UI**: Follow existing component usage (Button, Card, Select, etc.)
- **TypeScript strict mode**: Ensure proper typing throughout
- **React Query**: Use existing patterns for queries and mutations
- **Error handling**: Match existing error UI patterns
- **Login flow**: Study existing login redirect logic — don't reinvent
- **Window.close()**: Test popup flow carefully — may not work in all browsers (security restriction)
- **data-testid attributes**: Add for Playwright tests (Phase 9)

## After Implementation

1. **Rebuild UI**: `make build.ui-clean && make build.ui` (see CLAUDE.md critical note)
2. **Manual test**: Open review URL in browser, test both flows
3. **Prepare for Phase 8**: Ensure data-testid attributes are in place for component tests

## Next Phase

Phase 7 will implement comprehensive Rust unit and API tests for the backend changes.
