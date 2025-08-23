---
inclusion: manual
---

# Frontend React/TypeScript Guidelines

## Technology Stack Specifics

**Framework**: Next.js v14.2.6 App Router (Complete client-side SPA)
**State Management**: React Query v3.39.3 (NOT TanStack Query)
**Backend Integration**: `useQuery.ts` and `apiClient.ts`
**Routing**: Query parameter-based (e.g., `/chat/?id=<id>`)

## Required Documentation References

**MUST READ before any changes:**
- `ai-docs/01-architecture/frontend-react.md` - Next.js project structure and conventions
- `ai-docs/01-architecture/api-integration.md` - API integration patterns and query hooks
- `ai-docs/01-architecture/development-conventions.md` - Naming conventions and component structure

**FOR STYLING:**
- `ai-docs/01-architecture/ui-design-system.md` - UI/UX patterns and component usage

**FOR TESTING:**
- `ai-docs/01-architecture/frontend-testing.md` - Testing patterns and utilities

## Critical Frontend Rules

### Next.js App Router Structure
- **Pages**: `src/app/ui/<page>/page.tsx` (single file with both wrapper and content)
- **Components**: `src/app/ui/<page>/<Component>.tsx`
- **Layout**: Requires `'use client'` directive with AppHeader and theme handling
- **PWA**: Configuration with `@ducanh2912/next-pwa`

### Import & Component Standards
- **ALWAYS use `@/` absolute imports** instead of relative paths
- **Remove custom Image/Link components** - use Next.js built-ins
- **Merge page wrappers** with content into single `page.tsx` files
- **Clean code removal** without explanatory comments

### Authentication Integration
- **No anonymous access** - authentication always required
- **"Dumb" frontend** - send all query params to backend without validation
- **Pages handle redirects**, not hooks
- **Reuse `useMutation` patterns** for consistency

## Follow Documentation Patterns

All specific implementation details, file organization patterns, API integration standards, component structures, and testing requirements are documented in the referenced ai-docs files above. Refer to those documents for the authoritative guidance rather than duplicating conventions here.
