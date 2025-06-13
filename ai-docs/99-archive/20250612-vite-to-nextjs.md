# Migration Specification: React Router + Vite to Next.js Pages Router

**Document Reference**: @ai-docs/README.md and @ai-docs/01-architecture/README.md

## Executive Summary

This specification outlines a comprehensive, phase-based migration plan to revert the Bodhi App frontend from React Router + Vite back to Next.js Pages Router. The migration is necessary to restore Static Site Generation (SSG) capabilities that are not adequately supported in the current React + Vite setup.

### Migration Context

- **Original Migration**: Next.js Pages Router → React Router + Vite (commits 33c2c418..b9a331df)
- **Files Affected**: 117 files with 7,506 additions and 46 deletions
- **Current State**: React Router + Vite with custom SSG implementation
- **Target State**: Next.js Pages Router with native SSG support
- **Validation Strategy**: Each phase must pass `npm run format`, `npm run build`, and `npm run test`

## Strategic Approach

### Migration Philosophy

1. **Incremental Transformation**: Each phase maintains working application state
2. **Risk Mitigation**: Start with least impactful changes, progressively address complexity
3. **Validation-Driven**: Continuous testing ensures no functionality regression
4. **Dependency-First**: Address package dependencies before structural changes
5. **SSG-Focused**: Prioritize restoring native Next.js SSG capabilities

### Phase-Based Execution

The migration follows a 6-phase approach designed for systematic, low-risk execution:

1. **Phase 1**: Dependency Management & Build System
2. **Phase 2**: File Structure Preparation
3. **Phase 3**: Component Migration & Routing
4. **Phase 4**: Next.js App Directory Structure
5. **Phase 5**: Build Configuration & SSG
6. **Phase 6**: Testing & Cleanup

## Phase 1: Dependency Management & Build System

### Objective
Replace Vite-specific dependencies with Next.js equivalents while maintaining application functionality.

### Package.json Changes

#### Dependencies to Remove
```json
{
  "vite": "^6.0.3",
  "vite-ssg": "^27.0.1", 
  "vite-tsconfig-paths": "^5.1.4",
  "@vitejs/plugin-react": "^4.3.4",
  "react-router-dom": "^7.6.2"
}
```

#### Dependencies to Add
```json
{
  "next": "^15.1.0",
  "@next/mdx": "^15.1.0",
  "@types/mdx": "^2.0.13"
}
```

#### Dependencies to Update
```json
{
  "react-query": "^3.39.3" → "@tanstack/react-query": "^5.0.0"
}
```

### Script Changes

#### Remove Vite Scripts
```json
{
  "dev": "vite",
  "build": "vite build && npm run build:static",
  "build:static": "node scripts/generate-static-routes.js",
  "build:vite": "vite build",
  "preview": "vite preview"
}
```

#### Add Next.js Scripts
```json
{
  "dev": "next dev",
  "build": "next build",
  "start": "next start",
  "export": "next export"
}
```

### Configuration Files

#### Files to Remove
- `vite.config.ts`
- `vite-plugins/docs-generator.ts`
- `scripts/generate-static-routes.js`
- `index.html`
- `vite-env.d.ts`

#### Files to Create
- `next.config.mjs`
- `middleware.ts` (for route handling)

### Validation Commands
```bash
npm install
npm run format
npm run build
npm run test
```

## Phase 2: File Structure Preparation

### Objective
Prepare the file structure for Next.js while maintaining React Router functionality temporarily.

### Directory Structure Changes

#### Create Next.js Directories
```
src/
├── app/                    # Next.js App Router (new)
│   ├── globals.css        # Move from src/styles/
│   ├── layout.tsx         # Root layout
│   └── page.tsx           # Root page
├── pages/                 # Keep temporarily for migration
└── components/            # Keep existing structure
```

#### File Movements
1. **Move Global Styles**: `src/styles/globals.css` → `src/app/globals.css`
2. **Create Root Layout**: Extract layout logic from `App.tsx` → `src/app/layout.tsx`
3. **Create Root Page**: Create `src/app/page.tsx` for root route

### Layout Migration

#### Extract from App.tsx
```typescript
// Current App.tsx structure to extract:
// - Theme provider setup
// - Query client provider
// - Global error boundaries
// - Navigation components
```

#### Create app/layout.tsx
```typescript
// Root layout with:
// - HTML structure
// - Theme providers
// - Global styles
// - Client providers
```

### Validation Commands
```bash
npm run format
npm run build
npm run test
```

## Phase 3: Component Migration & Routing

### Objective
Migrate page components from React Router structure to Next.js Pages Router structure.

### Page Component Migration

#### Current Structure (React Router)
```
src/pages/
├── ChatPage.tsx           # Lazy-loaded wrapper
├── HomePage.tsx           # Lazy-loaded wrapper
├── ModelsPage.tsx         # Lazy-loaded wrapper
└── ...

src/components/
├── chat/ChatPage.tsx      # Actual implementation
├── home/HomePage.tsx      # Actual implementation
├── models/ModelsPage.tsx  # Actual implementation
└── ...
```

#### Target Structure (Next.js)
```
src/app/
├── ui/
│   ├── page.tsx           # Home page
│   ├── chat/
│   │   └── page.tsx       # Chat page
│   ├── models/
│   │   ├── page.tsx       # Models list
│   │   ├── new/
│   │   │   └── page.tsx   # New model
│   │   └── edit/
│   │       └── page.tsx   # Edit model
│   └── ...
└── docs/
    ├── page.tsx           # Docs index
    └── [...slug]/
        └── page.tsx       # Dynamic docs pages
```

### Migration Steps

#### Step 3.1: Create Next.js Page Structure
For each current page component, create corresponding Next.js page:

1. **Home Page**: `src/pages/HomePage.tsx` → `src/app/ui/page.tsx`
2. **Chat Page**: `src/pages/ChatPage.tsx` → `src/app/ui/chat/page.tsx`
3. **Models Page**: `src/pages/ModelsPage.tsx` → `src/app/ui/models/page.tsx`
4. **Settings Page**: `src/pages/SettingsPage.tsx` → `src/app/ui/settings/page.tsx`
5. **Login Page**: `src/pages/LoginPage.tsx` → `src/app/ui/login/page.tsx`
6. **Setup Pages**: `src/pages/SetupPage.tsx` → `src/app/ui/setup/page.tsx`
7. **Docs Pages**: `src/pages/docs/` → `src/app/docs/`

#### Step 3.2: Update Component Imports
Each Next.js page should import the actual component implementation:

```typescript
// src/app/ui/chat/page.tsx
import ChatPageContent from '@/components/chat/ChatPage';

export default function ChatPage() {
  return <ChatPageContent />;
}
```

#### Step 3.3: Handle Dynamic Routes
- **Chat with ID**: Implement using Next.js searchParams
- **Docs Slug**: Use `[...slug]` dynamic route
- **Model Edit**: Use `[id]` dynamic route

### Navigation Updates

#### Remove React Router Dependencies
1. Update `src/hooks/use-navigation.tsx` to use Next.js navigation
2. Replace `useNavigate` with `useRouter` from Next.js
3. Replace `useLocation` with `usePathname` from Next.js
4. Update `src/components/Link.tsx` to use Next.js Link

#### Update Navigation Components
1. **AppNavigation**: Update route checking logic
2. **AppBreadcrumb**: Update path parsing for Next.js routes
3. **LoginMenu**: Update logout redirect logic

### Validation Commands
```bash
npm run format
npm run build
npm run test
```

## Phase 4: Next.js App Directory Structure

### Objective
Complete the migration to Next.js App Router structure and remove React Router dependencies.

### App Directory Completion

#### Create Missing Pages
1. **Not Found**: `src/app/not-found.tsx`
2. **Loading**: `src/app/loading.tsx`
3. **Error**: `src/app/error.tsx`

#### Nested Layouts
1. **UI Layout**: `src/app/ui/layout.tsx` for common UI structure
2. **Docs Layout**: `src/app/docs/layout.tsx` for documentation

### Component Updates

#### Remove React Router Code
1. **Remove Router Setup**: Delete router configuration from `main.tsx`
2. **Update App.tsx**: Remove React Router providers
3. **Remove Pages Directory**: Delete `src/pages/` after migration complete

#### Update Hook Dependencies
1. **use-navigation.tsx**: Complete Next.js migration
2. **useLogoutHandler.ts**: Update for Next.js navigation
3. **Remove router-utils.tsx**: No longer needed

### API Integration Updates

#### Update Query Hooks
Ensure React Query hooks work with Next.js:
1. **Client Components**: Add "use client" directive where needed
2. **Server Components**: Identify components that can be server-side
3. **Hydration**: Ensure proper client-server hydration

### Validation Commands
```bash
npm run format
npm run build
npm run test
```

## Phase 5: Build Configuration & SSG

### Objective
Configure Next.js for Static Site Generation and remove Vite-specific configurations.

### Next.js Configuration

#### next.config.mjs
```javascript
/** @type {import('next').NextConfig} */
const nextConfig = {
  output: 'export',
  trailingSlash: true,
  images: {
    unoptimized: true
  },
  experimental: {
    mdxRs: true
  }
};

export default nextConfig;
```

#### Static Generation Setup
1. **generateStaticParams**: For dynamic routes
2. **Static Exports**: Configure for static hosting
3. **Asset Optimization**: Configure for production builds

### MDX Integration

#### Update MDX Configuration
1. **Remove Vite MDX**: Replace vite MDX plugin
2. **Add Next.js MDX**: Configure @next/mdx
3. **Update MDX Components**: Ensure compatibility

#### Docs System Migration
1. **Docs Client**: Update `src/lib/docs-client.ts` for Next.js
2. **Docs Generation**: Replace Vite plugin with Next.js approach
3. **Static Data**: Update docs data generation

### Build System Updates

#### Remove Vite Files
1. **Delete vite.config.ts**
2. **Delete vite-plugins/**
3. **Delete scripts/generate-static-routes.js**
4. **Update .gitignore**: Remove Vite-specific entries

#### Update TypeScript Configuration
1. **tsconfig.json**: Update for Next.js
2. **Remove vite-env.d.ts**
3. **Add next-env.d.ts**

### Validation Commands
```bash
npm run format
npm run build
npm run test
```

## Phase 6: Testing & Cleanup

### Objective
Update test configurations, fix any remaining issues, and clean up migration artifacts.

### Test Configuration Updates

#### Update Test Setup
1. **Vitest Configuration**: Ensure compatibility with Next.js
2. **Test Utilities**: Update `src/tests/wrapper.tsx` for Next.js
3. **Router Testing**: Update `src/tests/router-utils.tsx`

#### Component Test Updates
1. **Navigation Tests**: Update for Next.js navigation
2. **Page Tests**: Update for Next.js page structure
3. **Hook Tests**: Update for Next.js environment

### Final Cleanup

#### Remove Migration Artifacts
1. **Delete src/pages/**: Remove old React Router pages
2. **Clean Dependencies**: Remove unused packages
3. **Update Documentation**: Update relevant docs

#### Performance Optimization
1. **Bundle Analysis**: Verify Next.js bundle optimization
2. **Static Generation**: Verify all routes generate correctly
3. **Asset Loading**: Verify asset optimization

### Final Validation
```bash
npm run format
npm run build
npm run test
npm run start  # Test production build
```

## Risk Assessment & Mitigation

### High-Risk Areas

#### 1. State Management
- **Risk**: React Query state loss during navigation
- **Mitigation**: Ensure proper hydration and client-side state persistence

#### 2. Dynamic Routing
- **Risk**: Chat URLs with query parameters may break
- **Mitigation**: Implement Next.js searchParams handling early

#### 3. MDX Integration
- **Risk**: Docs system may break during MDX migration
- **Mitigation**: Test docs functionality after each phase

#### 4. Build Performance
- **Risk**: Next.js build may be slower than Vite
- **Mitigation**: Optimize Next.js configuration for build speed

### Rollback Strategy

#### Phase-Level Rollback
Each phase includes validation commands. If validation fails:
1. **Identify Issue**: Use test output to identify specific failures
2. **Targeted Fix**: Address specific issues without reverting entire phase
3. **Re-validate**: Run validation commands again
4. **Document Issues**: Update this specification with lessons learned

#### Complete Rollback
If migration must be abandoned:
1. **Git Reset**: Reset to pre-migration commit
2. **Dependency Restore**: Restore original package.json
3. **Documentation Update**: Document reasons for rollback

## Success Criteria

### Functional Requirements
1. **All Routes Work**: Every current route accessible via direct URL
2. **SSG Functions**: Static site generation produces correct output
3. **Navigation Works**: All navigation patterns function correctly
4. **Tests Pass**: All existing tests continue to pass
5. **Build Succeeds**: Production build completes without errors

### Performance Requirements
1. **Build Time**: Next.js build time comparable to Vite
2. **Bundle Size**: JavaScript bundle size within 10% of current
3. **Load Time**: Page load times maintained or improved

### Quality Requirements
1. **Code Quality**: ESLint and Prettier checks pass
2. **Type Safety**: TypeScript compilation without errors
3. **Test Coverage**: Test coverage maintained at current levels

## Implementation Timeline

### Estimated Duration: 3-5 Days

#### Day 1: Phases 1-2
- Dependency management
- File structure preparation
- Initial Next.js setup

#### Day 2: Phase 3
- Component migration
- Routing updates
- Navigation system updates

#### Day 3: Phase 4
- App directory completion
- React Router removal
- Hook updates

#### Day 4: Phase 5
- Build configuration
- SSG setup
- MDX integration

#### Day 5: Phase 6
- Testing updates
- Final cleanup
- Performance optimization

## Detailed Implementation Guide

### Phase 1 Detailed Steps

#### Step 1.1: Update package.json Dependencies
```bash
# Remove Vite dependencies
npm uninstall vite vite-ssg vite-tsconfig-paths @vitejs/plugin-react react-router-dom

# Add Next.js dependencies
npm install next@^15.1.0 @next/mdx@^15.1.0

# Update React Query
npm uninstall react-query
npm install @tanstack/react-query@^5.0.0
```

#### Step 1.2: Update Scripts in package.json
Replace the scripts section with Next.js equivalents:
```json
{
  "scripts": {
    "dev": "next dev",
    "build": "next build",
    "start": "next start",
    "export": "next export",
    "lint": "eslint . --ext .js,.jsx,.ts,.tsx",
    "test": "vitest run",
    "eslint": "eslint . --ext .js,.jsx,.ts,.tsx",
    "format": "eslint --ext .js,.jsx,.ts,.tsx --fix .",
    "format:check": "eslint --ext .js,.jsx,.ts,.tsx --fix-dry-run .",
    "prepare": "husky"
  }
}
```

#### Step 1.3: Create next.config.mjs
```javascript
/** @type {import('next').NextConfig} */
const nextConfig = {
  output: 'export',
  trailingSlash: true,
  distDir: 'dist',
  images: {
    unoptimized: true
  },
  experimental: {
    mdxRs: true
  },
  webpack: (config) => {
    config.resolve.alias = {
      ...config.resolve.alias,
      '@': path.resolve(__dirname, './src'),
    };
    return config;
  }
};

export default nextConfig;
```

#### Step 1.4: Remove Vite Configuration Files
```bash
rm vite.config.ts
rm -rf vite-plugins/
rm -rf scripts/generate-static-routes.js
rm index.html
rm src/vite-env.d.ts
```

### Phase 2 Detailed Steps

#### Step 2.1: Create App Directory Structure
```bash
mkdir -p src/app
mkdir -p src/app/ui
mkdir -p src/app/docs
```

#### Step 2.2: Move Global Styles
```bash
mv src/styles/globals.css src/app/globals.css
rmdir src/styles
```

#### Step 2.3: Create Root Layout (src/app/layout.tsx)
Extract layout logic from current App.tsx and create:
```typescript
import type { Metadata } from 'next';
import './globals.css';
import { ClientProviders } from '@/components/ClientProviders';

export const metadata: Metadata = {
  title: 'Bodhi App',
  description: 'AI-powered chat application',
};

export default function RootLayout({
  children,
}: {
  children: React.ReactNode;
}) {
  return (
    <html lang="en">
      <body>
        <ClientProviders>
          {children}
        </ClientProviders>
      </body>
    </html>
  );
}
```

#### Step 2.4: Create Root Page (src/app/page.tsx)
```typescript
import { redirect } from 'next/navigation';

export default function RootPage() {
  redirect('/ui');
}
```

### Phase 3 Detailed Steps

#### Step 3.1: Create Next.js Pages for Each Route

**Home Page (src/app/ui/page.tsx)**:
```typescript
import HomePageContent from '@/components/home/HomePage';

export default function HomePage() {
  return <HomePageContent />;
}
```

**Chat Page (src/app/ui/chat/page.tsx)**:
```typescript
import ChatPageContent from '@/components/chat/ChatPage';

export default function ChatPage() {
  return <ChatPageContent />;
}
```

**Models Page (src/app/ui/models/page.tsx)**:
```typescript
import ModelsPageContent from '@/components/models/ModelsPage';

export default function ModelsPage() {
  return <ModelsPageContent />;
}
```

Continue this pattern for all pages...

#### Step 3.2: Update Navigation Hooks

**Update src/hooks/use-navigation.tsx**:
```typescript
'use client';

import { useRouter, usePathname } from 'next/navigation';

export function useNavigation() {
  const router = useRouter();
  const pathname = usePathname();

  const navigate = (path: string) => {
    router.push(path);
  };

  const isActive = (path: string) => {
    return pathname === path;
  };

  return {
    navigate,
    isActive,
    pathname,
  };
}
```

#### Step 3.3: Update Link Component

**Update src/components/Link.tsx**:
```typescript
import NextLink from 'next/link';
import { ReactNode } from 'react';

interface LinkProps {
  href: string;
  children: ReactNode;
  className?: string;
}

export function Link({ href, children, className }: LinkProps) {
  return (
    <NextLink href={href} className={className}>
      {children}
    </NextLink>
  );
}
```

### Phase 4 Detailed Steps

#### Step 4.1: Create UI Layout (src/app/ui/layout.tsx)
```typescript
import { AppHeader } from '@/components/navigation/AppHeader';
import { AppNavigation } from '@/components/navigation/AppNavigation';

export default function UILayout({
  children,
}: {
  children: React.ReactNode;
}) {
  return (
    <div className="min-h-screen bg-background">
      <AppHeader />
      <div className="flex">
        <AppNavigation />
        <main className="flex-1">
          {children}
        </main>
      </div>
    </div>
  );
}
```

#### Step 4.2: Create Not Found Page (src/app/not-found.tsx)
```typescript
import NotFoundPageContent from '@/components/not-found/NotFoundPageContent';

export default function NotFound() {
  return <NotFoundPageContent />;
}
```

#### Step 4.3: Update Client Providers

**Update src/components/ClientProviders.tsx**:
```typescript
'use client';

import { QueryClient, QueryClientProvider } from '@tanstack/react-query';
import { ThemeProvider } from '@/components/ThemeProvider';
import { useState } from 'react';

export function ClientProviders({ children }: { children: React.ReactNode }) {
  const [queryClient] = useState(() => new QueryClient({
    defaultOptions: {
      queries: {
        staleTime: 60 * 1000,
      },
    },
  }));

  return (
    <QueryClientProvider client={queryClient}>
      <ThemeProvider>
        {children}
      </ThemeProvider>
    </QueryClientProvider>
  );
}
```

### Phase 5 Detailed Steps

#### Step 5.1: Update TypeScript Configuration

**Update tsconfig.json**:
```json
{
  "compilerOptions": {
    "target": "es5",
    "lib": ["dom", "dom.iterable", "es6"],
    "allowJs": true,
    "skipLibCheck": true,
    "strict": true,
    "forceConsistentCasingInFileNames": true,
    "noEmit": true,
    "esModuleInterop": true,
    "module": "esnext",
    "moduleResolution": "bundler",
    "resolveJsonModule": true,
    "isolatedModules": true,
    "jsx": "preserve",
    "incremental": true,
    "plugins": [
      {
        "name": "next"
      }
    ],
    "baseUrl": ".",
    "paths": {
      "@/*": ["./src/*"]
    }
  },
  "include": ["next-env.d.ts", "**/*.ts", "**/*.tsx", ".next/types/**/*.ts"],
  "exclude": ["node_modules"]
}
```

#### Step 5.2: Update MDX Configuration

**Update next.config.mjs for MDX**:
```javascript
import createMDX from '@next/mdx';

/** @type {import('next').NextConfig} */
const nextConfig = {
  output: 'export',
  trailingSlash: true,
  distDir: 'dist',
  pageExtensions: ['js', 'jsx', 'mdx', 'ts', 'tsx'],
  images: {
    unoptimized: true
  }
};

const withMDX = createMDX({
  options: {
    remarkPlugins: [],
    rehypePlugins: [],
  },
});

export default withMDX(nextConfig);
```

### Phase 6 Detailed Steps

#### Step 6.1: Update Test Configuration

**Update vitest.config.ts** (if using Vitest with Next.js):
```typescript
import { defineConfig } from 'vitest/config';
import react from '@vitejs/plugin-react';
import path from 'path';

export default defineConfig({
  plugins: [react()],
  test: {
    environment: 'jsdom',
    setupFiles: ['./src/tests/setup.ts'],
  },
  resolve: {
    alias: {
      '@': path.resolve(__dirname, './src'),
    },
  },
});
```

#### Step 6.2: Update Test Wrapper

**Update src/tests/wrapper.tsx**:
```typescript
import { QueryClient, QueryClientProvider } from '@tanstack/react-query';
import { ReactNode } from 'react';

const createTestQueryClient = () => new QueryClient({
  defaultOptions: {
    queries: { retry: false },
    mutations: { retry: false },
  },
});

export function TestWrapper({ children }: { children: ReactNode }) {
  const testQueryClient = createTestQueryClient();

  return (
    <QueryClientProvider client={testQueryClient}>
      {children}
    </QueryClientProvider>
  );
}
```

#### Step 6.3: Final Cleanup
```bash
# Remove old React Router pages
rm -rf src/pages/

# Remove router utilities
rm src/tests/router-utils.tsx

# Remove main.tsx (no longer needed)
rm src/main.tsx

# Remove old App.tsx (replaced by layout.tsx)
rm src/App.tsx src/App.test.tsx
```

## Conclusion

This migration specification provides a systematic approach to reverting from React Router + Vite to Next.js Pages Router while maintaining application functionality and restoring native SSG capabilities. The phase-based approach minimizes risk and ensures continuous validation throughout the migration process.

The specification is designed for AI coding assistants to follow step-by-step, with clear validation points and detailed instructions for each phase. Each phase builds upon the previous one while maintaining a working application state, enabling incremental progress with minimal risk of breaking functionality.
