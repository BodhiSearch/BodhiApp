# AI IDE Memories - Augment Agent

This document contains refined and consistent memories for AI IDE interactions with the BodhiApp project.

## Project Architecture & Technology Stack

### Core Framework
- **Current Stack**: Next.js v14.2.6 (not 15.x) with App Router
- **Frontend Location**: `src/app/ui/` directory structure
- **Build Commands**: `npm run build` from `crates/bodhi` folder
- **Test Commands**: `npm run test` (CI mode) or `npm run test` (watch mode)
- **Testing Framework**: Vitest
- **State Management**: React Query v3.39.3 (not TanStack Query)

### Migration History & Lessons
- Project completed Next.js → React Router + Vite → Next.js migration cycle
- Reverted to Next.js v14 due to SSG limitations
- User prefers preserving functional features added during migrations while focusing structural changes on framework requirements only
- For Tauri apps specifically: User prefers Vite+React over Next.js

### Application Architecture
- **Client Architecture**: Complete client-side SPA with no server components
- **Backend Integration**: Uses `useQuery.ts` and `apiClient.ts` for API communication
- **Import Style**: Absolute imports using `@/` prefix instead of relative paths
- **Component Organization**: Page/component architecture with navigable components in `pages/` and implementation in `components/`
- **Routing Preference**: Query parameter-based routing (e.g., `/chat/?id=<id>`) over path parameters

## Code Organization & Structure

### Directory Structure
- **Pages**: `src/app/ui/<page>/page.tsx` (Next.js App Router)
- **Components**: `src/app/ui/<page>/<Component>.tsx`
- **Tests**: Co-located as `<file>.test.tsx` next to components
- **Migration Pattern**: Move from `src/components/<page>/` to `src/app/ui/<page>/`

### File Organization Principles
- Remove custom `Image.tsx` and `Link.tsx` components during Next.js migrations (use built-in Next.js components)
- Merge separate page wrapper and component files into single `page.tsx` files
- Clean removal of unused code without explanatory comments

## Authentication & Authorization

### OAuth Implementation
- **Flow**: SPA-managed OAuth 2.0 Authorization Code flow with PKCE
- **Frontend Role**: "Dumb" frontend - send all query params to backend without validation
- **Backend Responsibility**: Handle all OAuth logic, errors, and redirects
- **Anonymous Access**: Eliminated - authentication always required
- **UI Text**: "Log In" preferred over "Sign In"

### OAuth Security
- **State Parameter**: Cryptographic validation using SHA-256 digest
- **State Composition**: `scope + session_id_initials + 8char_random_id`
- **Scope Handling**: Extract from JWT claims, sort array, join with `%20`
- **Token Scopes**: Include `scope_user`, `scope_power_user`, `scope_manager`, `scope_admin` (exclude `offline_access`)
- **Integration Test Scopes**: `openid email profile roles`

### Authentication Flow
- `/auth/initiate` returns:
  - 401 with `auth_url` when login required
  - 303 with UI home location when already authenticated
- Pages handle redirects, not hooks
- Reuse existing `useMutation` patterns

## Testing Preferences

### Test Architecture
- **Framework**: Vitest with MSW for API mocking
- **Naming Convention**: `test_init_service_<method_name>_<test-specific>`
- **API Mocking**: Use MSW patterns (see `models/page.test.tsx`)
- **Base URL**: Keep `apiClient.baseURL` as empty string (`''`)

### Test Quality Standards
- Fewer, substantial test scenarios over many fine-grained tests
- Separate test cases for success and error scenarios
- Check last items of arrays when validating streaming responses
- Avoid `unmount()` in tests
- Fix root causes rather than using workarounds

### Integration Testing
- Create encrypted secrets files
- Set AppStatus to Ready
- Obtain auth tokens and insert session data
- Set up cookies properly
- Use test-utils feature flag pattern for Rust components

## Development Practices

### Package Management
- **Rule**: Always use package managers instead of manual file editing
- **JavaScript**: Use `npm install/uninstall`, `yarn add/remove`, or `pnpm add/remove`
- **Exception**: Only edit package files for complex configurations not achievable via package manager commands

### Error Handling & Code Quality
- Fix multiple compilation failures in same file in single iteration
- Apply React/Vitest/mocking best practices
- Create valuable, dependable, deterministic, and maintainable tests

### Documentation Requirements
- AI-first documentation with hierarchical organization
- All documentation must be factually verified against source code with `filename:linenumber` references
- Update `ai-docs/README.md` when adding/updating files in `ai-docs/` folder
- Generate knowledge transfer docs in `ai-docs/06-knowledge-transfer/` after major tasks

## Service & Library Design

### Architecture Patterns
- Unified app initialization with single service call pattern
- Service builder pattern with dependency injection
- Multi-phase initialization: setup → load/create configs → apply overrides → initialize components
- Error enums with `thiserror` over struct-based errors

### Library Design
- Separate publishable library crates
- Platform-agnostic design with C-style interfaces
- Builder patterns using `derive-builder` crate
- Functional approaches over object wrappers for test utilities

## PWA & Build Configuration

### Next.js Specific
- PWA configuration with `@ducanh2912/next-pwa`
- Webpack settings for ignored folders
- Use `postcss.config.mjs` instead of `.js`
- Include `public/` folder with static assets
- Layout requires `'use client'` directive with AppHeader and theme handling

### Build & Deployment
- Build-time markdown generation preferred over runtime processing
- Preserve PWA functionality during framework migrations
