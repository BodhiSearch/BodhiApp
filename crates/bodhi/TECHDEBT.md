# bodhi (frontend) Technical Debt

## Unify create/update forms to match single `*Form` API types

- **Currently**: Frontend has separate create and update form conversion functions (e.g., `convertFormToCreateApi`, `convertFormToUpdateApi` in `schemas/alias.ts`)
- **Should be**: Unified to use a single conversion since the backend now uses a single `*Form` type for both create and update (e.g., `UserAliasForm`, `ApiModelForm`)
- **Reason**: Reduces duplication and aligns frontend with backend's unified form pattern

## Skipped test: useListUsers error response timeout

- **File**: `src/hooks/users/useUsers.test.ts`
- **Test**: `User Hooks > useListUsers > handles error response`
- **Issue**: `useListUsers` hook explicitly sets `retry: 1`, overriding the test wrapper's `retry: false`. TanStack Query's exponential backoff between retries (~1s) exceeds `waitFor`'s default 1s timeout, causing the test to time out.
- **Fix options**:
  1. Accept `retry` as an option in `useListUsers` so tests can override it
  2. Create a custom `createWrapper({ retryDelay: 1 })` variant for tests with retry-enabled hooks
  3. Extract the retry config as a constant that tests can mock

## App Crash now no longer prints on stdout

Probably after boostrap-service refactoring, app silently crashing
