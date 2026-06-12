# Vite + React Router to Next.js Migration - V2 Specification

## Overview
This document outlines the revised approach for migrating the Bodhi frontend from Vite + React Router to Next.js 14.2.6, based on lessons learned from the initial migration attempt.

## Key Learnings from V1 Attempt

### Critical Issues Identified
1. **React Query v3.39.3 Compatibility**: React Query v3 has compatibility issues with Next.js 14 SSR/SSG during build time
2. **Component Structure Confusion**: Moving components between folders during migration created import path chaos
3. **Server vs Client Component Confusion**: Attempted to use App Router server components when the app is purely client-side
4. **Incremental Migration Complexity**: Trying to migrate piece-by-piece led to inconsistent states

### Root Cause Analysis
- The app is a **complete client-side SPA** with no server components
- Original Next.js setup used client-side rendering with 'use client' directive on layout
- React Query context creation fails during Next.js build process in SSR mode
- Import path changes during component moves broke the build process

## V2 Migration Strategy

### Phase 1: Preparation and Baseline
1. **Create baseline snapshot**
   ```bash
   git ls-tree --name-only -r 322187ff | grep "crates/bodhi/src" > original_files_list.txt
   ```

2. **Revert all component moves**
   - Move all components back from `src/components/<page>/` to `src/app/ui/<page>/`
   - Restore original file structure exactly as in commit 322187ff

3. **Document original structure**
   - Map every file from original location to target Next.js location
   - Identify which files need 'use client' directive
   - List all import paths that need updating

### Phase 2: Configuration Setup
1. **Install Next.js dependencies** (exact versions from original)
   ```json
   {
     "next": "14.2.6",
     "react": "^18",
     "react-dom": "^18",
     "react-query": "^3.39.3"
   }
   ```

2. **Configuration files** (copy exactly from commit 322187ff)
   - `next.config.mjs` - without PWA initially
   - `tsconfig.json` - with Next.js plugin
   - `postcss.config.mjs` - not .js
   - `vitest.config.ts` - with @vitejs/plugin-react

3. **Remove Vite-specific files**
   - `index.html`
   - `vite.config.ts`
   - `scripts/generate-static-routes.js`

### Phase 3: File-by-File Migration
1. **Core app structure**
   - `src/app/layout.tsx` - with 'use client' and original structure
   - `src/app/page.tsx` - redirect to /ui
   - `src/app/metadata.ts` - app metadata
   - `src/mdx-components.tsx` - MDX configuration

2. **Page migration order** (one at a time)
   - Start with simplest pages (login, home)
   - Add 'use client' directive to all page.tsx files
   - Verify each page builds before moving to next
   - Use absolute '@/' imports only

3. **Component restoration**
   - Keep components in their original `src/app/ui/<page>/` locations
   - Update import paths to use '@/' absolute imports
   - Maintain original component structure and dependencies

### Phase 4: React Query Integration
1. **Client-side only approach**
   - Ensure QueryClientProvider is only rendered client-side
   - Add hydration checks if needed
   - Test React Query hooks work in client components

2. **Fallback strategy if React Query v3 fails**
   - Upgrade to @tanstack/react-query v4
   - Update all import statements
   - Adapt to API changes (minimal in v4)

## Migration Process & Techniques

### File Comparison Workflow
```bash
# For each file in original structure
git show 322187ff:crates/bodhi/src/app/ui/chat/page.tsx > /tmp/original.tsx
diff /tmp/original.tsx src/app/ui/chat/page.tsx
# Apply necessary changes for Next.js compatibility
```

### Import Path Strategy
- **Always use '@/' absolute imports** - never relative paths
- **Update systematically**: `@/components/` → `@/app/ui/`
- **Verify imports exist** before updating references

### Build Validation Process
1. Run `npm run build` after each page migration
2. Fix compilation errors immediately
3. Don't proceed until build is clean
4. Test critical functionality after each phase

### Error Resolution Patterns
1. **Module not found**: Check import paths and file locations
2. **React Query context errors**: Ensure client-side only rendering
3. **TypeScript errors**: Update type imports and component props
4. **Build failures**: Add 'use client' directive to pages using hooks

## File Structure Mapping

### Original → Target Structure
```
src/app/ui/chat/page.tsx → src/app/ui/chat/page.tsx (add 'use client')
src/app/ui/chat/ChatHistory.tsx → src/app/ui/chat/ChatHistory.tsx
src/app/ui/chat/ChatUI.tsx → src/app/ui/chat/ChatUI.tsx
src/app/ui/models/page.tsx → src/app/ui/models/page.tsx (add 'use client')
src/app/ui/models/ModelsPage.tsx → src/app/ui/models/ModelsPage.tsx
```

### Required 'use client' Files
- All `src/app/ui/*/page.tsx` files
- `src/app/layout.tsx`
- Any component using React hooks or browser APIs

## Testing Strategy
1. **Build verification**: `npm run build` must pass
2. **Development server**: `npm run dev` should work
3. **Key functionality**: Test navigation, React Query, forms
4. **Production build**: Verify static export works

## Rollback Plan
- Keep original Vite configuration in separate branch
- Document all changes for easy reversal
- Test migration in isolated environment first

## Success Criteria
- [ ] All pages load correctly in Next.js
- [ ] React Query works without SSR issues
- [ ] Build process completes successfully
- [ ] Navigation between pages works
- [ ] All original functionality preserved
- [ ] Performance is maintained or improved

## Next Steps After Migration
1. Re-enable PWA functionality with @ducanh2912/next-pwa
2. Optimize bundle size and loading performance
3. Add any Next.js specific optimizations
4. Update documentation and deployment scripts
