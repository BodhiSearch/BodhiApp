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

## Dark theme needs review/revamp

- **Issue**: Dark theme is currently unusable across the app
- **Plan**: Will be addressed as part of the planned UI revamp
- **Context**: Theme switching infrastructure works (light/dark/system), but dark theme styling needs comprehensive review

## Inconsistent new/edit URL patterns

- **Issue**: Some entities use `/new?id=X` for edit mode (e.g., `/mcps/new?id=X`), while others use dedicated `/edit?id=X` routes (e.g., `/mcps/servers/edit?id=X`, `/models/alias/edit?id=X`)
- **Plan**: Standardize to consistent pattern across all entities
- **Context**: The `/new?id=X` pattern originated from Next.js static export limitations. Now that TanStack Router supports dynamic routes, this can be cleaned up

## Per-route document titles

- **Issue**: All pages show the same browser tab title ("Bodhi App - Run LLMs Locally"). The old Next.js `metadata.ts` provided per-route titles via `APP_TITLE_TEMPLATE: '%s - Bodhi App'`
- **Plan**: Implement using TanStack Router's `head` property on routes or `react-helmet-async`
- **Context**: Helps users with many tabs distinguish between different Bodhi pages
