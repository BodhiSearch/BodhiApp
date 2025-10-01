# Setup UI Refactor - Phase-wise Implementation Plan

## Overview
This document provides a detailed, phase-wise implementation plan for refactoring the Bodhi App setup flow UI to achieve consistency, maintainability, and improved user experience.

## Project Context
- **Codebase Location**: `/crates/bodhi/src/app/ui/setup/`
- **Total Pages**: 6 setup flow pages + supporting components
- **Test Coverage**: Comprehensive tests exist for all pages
- **Testing Command**: `cd crates/bodhi && npm run test`
- **Build & Test Command**: `npm run build && npm run test && npm run test:typecheck`

## Key Issues to Address
1. **Inconsistent container widths**: max-w-4xl vs max-w-7xl
2. **Missing shared layout**: Each page implements its own structure
3. **Logo placement inconsistencies**: Missing on some pages
4. **Progress bar variations**: Different implementations across pages
5. **Button placement**: Inconsistent CTA positioning
6. **Card structures**: Varying patterns for similar content
7. **Typography inconsistencies**: Different heading sizes and styles

## Agent Execution Instructions
Before starting each phase:
1. Read `setup-ui-log.md` to understand previous phase completions
2. Read `setup-ui-ctx.md` for shared knowledge and insights
3. After completing each phase:
   - Run tests: `cd crates/bodhi && npm run test`
   - If tests fail, fix them before proceeding
   - Append completion details to `setup-ui-log.md`
   - Update `setup-ui-ctx.md` with any new insights

---

# PHASE 1: Create Shared Layout Infrastructure

## Objective
Create the foundational shared layout component that will be used across all setup pages.

## Tasks

### 1.1 Create layout.tsx for setup route
**File**: `/crates/bodhi/src/app/ui/setup/layout.tsx`

```tsx
'use client';

import { ReactNode } from 'react';
import { motion } from 'framer-motion';
import { SetupProvider } from './components/SetupProvider';

export default function SetupLayout({ children }: { children: ReactNode }) {
  return (
    <SetupProvider>
      <main className="min-h-screen bg-background">
        {children}
      </main>
    </SetupProvider>
  );
}
```

### 1.2 Create components directory structure
Create directory: `/crates/bodhi/src/app/ui/setup/components/`

### 1.3 Create SetupProvider component
**File**: `/crates/bodhi/src/app/ui/setup/components/SetupProvider.tsx`

```tsx
'use client';

import { createContext, useContext, ReactNode } from 'react';
import { usePathname } from 'next/navigation';
import { SETUP_STEPS } from '../constants';

interface SetupContextType {
  currentStep: number;
  isFirstStep: boolean;
  isLastStep: boolean;
  getStepFromPath: (path: string) => number;
}

const SetupContext = createContext<SetupContextType | undefined>(undefined);

export function SetupProvider({ children }: { children: ReactNode }) {
  const pathname = usePathname();

  const getStepFromPath = (path: string): number => {
    if (path.includes('/setup/resource-admin')) return SETUP_STEPS.RESOURCE_ADMIN;
    if (path.includes('/setup/download-models')) return SETUP_STEPS.DOWNLOAD_MODELS;
    if (path.includes('/setup/api-models')) return SETUP_STEPS.API_MODELS;
    if (path.includes('/setup/browser-extension')) return SETUP_STEPS.BROWSER_EXTENSION;
    if (path.includes('/setup/complete')) return SETUP_STEPS.COMPLETE;
    return SETUP_STEPS.WELCOME;
  };

  const currentStep = getStepFromPath(pathname);
  const isFirstStep = currentStep === SETUP_STEPS.WELCOME;
  const isLastStep = currentStep === SETUP_STEPS.COMPLETE;

  return (
    <SetupContext.Provider value={{ currentStep, isFirstStep, isLastStep, getStepFromPath }}>
      {children}
    </SetupContext.Provider>
  );
}

export function useSetupContext() {
  const context = useContext(SetupContext);
  if (!context) {
    throw new Error('useSetupContext must be used within SetupProvider');
  }
  return context;
}
```

## Verification
- Run: `cd crates/bodhi && npm run test`
- All existing tests should pass
- No visual changes yet

---

# PHASE 2: Create Shared Components

## Objective
Create reusable components for consistent layout across all setup pages.

## Tasks

### 2.1 Create SetupContainer component
**File**: `/crates/bodhi/src/app/ui/setup/components/SetupContainer.tsx`

```tsx
'use client';

import { ReactNode } from 'react';
import { motion } from 'framer-motion';
import { SetupProgress } from '../SetupProgress';
import { BodhiLogo } from '../BodhiLogo';
import { useSetupContext } from './SetupProvider';
import { SETUP_STEP_LABELS, SETUP_TOTAL_STEPS } from '../constants';

interface SetupContainerProps {
  children: ReactNode;
  showLogo?: boolean;
  showProgress?: boolean;
}

const containerVariants = {
  hidden: { opacity: 0 },
  visible: {
    opacity: 1,
    transition: {
      staggerChildren: 0.1,
    },
  },
};

export function SetupContainer({
  children,
  showLogo = true,
  showProgress = true
}: SetupContainerProps) {
  const { currentStep } = useSetupContext();

  return (
    <motion.div
      className="mx-auto max-w-4xl space-y-8 p-4 md:p-8"
      variants={containerVariants}
      initial="hidden"
      animate="visible"
    >
      {showProgress && (
        <SetupProgress
          currentStep={currentStep}
          totalSteps={SETUP_TOTAL_STEPS}
          stepLabels={SETUP_STEP_LABELS}
        />
      )}
      {showLogo && <BodhiLogo />}
      {children}
    </motion.div>
  );
}
```

### 2.2 Create SetupCard component
**File**: `/crates/bodhi/src/app/ui/setup/components/SetupCard.tsx`

```tsx
'use client';

import { ReactNode } from 'react';
import { motion } from 'framer-motion';
import { Card, CardContent, CardFooter, CardHeader, CardTitle, CardDescription } from '@/components/ui/card';

interface SetupCardProps {
  title?: string | ReactNode;
  description?: string;
  children: ReactNode;
  footer?: ReactNode;
  className?: string;
}

const itemVariants = {
  hidden: { y: 20, opacity: 0 },
  visible: {
    y: 0,
    opacity: 1,
  },
};

export function SetupCard({ title, description, children, footer, className }: SetupCardProps) {
  return (
    <motion.div variants={itemVariants}>
      <Card className={className}>
        {title && (
          <CardHeader className="text-center">
            {typeof title === 'string' ? (
              <CardTitle>{title}</CardTitle>
            ) : (
              title
            )}
            {description && <CardDescription>{description}</CardDescription>}
          </CardHeader>
        )}
        <CardContent>{children}</CardContent>
        {footer && <CardFooter>{footer}</CardFooter>}
      </Card>
    </motion.div>
  );
}
```

### 2.3 Create SetupNavigation component
**File**: `/crates/bodhi/src/app/ui/setup/components/SetupNavigation.tsx`

```tsx
'use client';

import { Button } from '@/components/ui/button';
import { useSetupContext } from './SetupProvider';
import { ChevronLeft, ChevronRight } from 'lucide-react';

interface SetupNavigationProps {
  onBack?: () => void;
  onNext?: () => void;
  onSkip?: () => void;
  backLabel?: string;
  nextLabel?: string;
  skipLabel?: string;
  showBack?: boolean;
  showNext?: boolean;
  showSkip?: boolean;
  nextDisabled?: boolean;
  backDisabled?: boolean;
  className?: string;
}

export function SetupNavigation({
  onBack,
  onNext,
  onSkip,
  backLabel = 'Back',
  nextLabel = 'Continue',
  skipLabel = 'Skip for Now',
  showBack = true,
  showNext = true,
  showSkip = false,
  nextDisabled = false,
  backDisabled = false,
  className = '',
}: SetupNavigationProps) {
  const { isFirstStep } = useSetupContext();

  return (
    <div className={`flex items-center justify-between ${className}`}>
      <div>
        {showBack && !isFirstStep && (
          <Button
            variant="outline"
            onClick={onBack}
            disabled={backDisabled}
          >
            <ChevronLeft className="mr-2 h-4 w-4" />
            {backLabel}
          </Button>
        )}
      </div>
      <div className="flex gap-4">
        {showSkip && (
          <Button variant="outline" onClick={onSkip}>
            {skipLabel}
          </Button>
        )}
        {showNext && (
          <Button onClick={onNext} disabled={nextDisabled}>
            {nextLabel}
            <ChevronRight className="ml-2 h-4 w-4" />
          </Button>
        )}
      </div>
    </div>
  );
}
```

### 2.4 Export shared components
**File**: `/crates/bodhi/src/app/ui/setup/components/index.ts`

```tsx
export { SetupContainer } from './SetupContainer';
export { SetupCard } from './SetupCard';
export { SetupNavigation } from './SetupNavigation';
export { SetupProvider, useSetupContext } from './SetupProvider';
```

## Verification
- Run: `cd crates/bodhi && npm run test`
- All tests should still pass
- Components created but not yet used

---

# PHASE 3: Refactor Welcome Page

## Objective
Refactor the welcome/setup page to use shared components.

## Tasks

### 3.1 Backup existing page
- Copy `/crates/bodhi/src/app/ui/setup/page.tsx` to `page.tsx.backup`

### 3.2 Refactor page.tsx
**File**: `/crates/bodhi/src/app/ui/setup/page.tsx`

Key changes:
- Use `SetupContainer` wrapper
- Use `SetupCard` for form card
- Remove local container/progress implementation
- Maintain all existing functionality
- Keep benefit cards as-is (they're specific to this page)

```tsx
'use client';

import { BenefitCard } from '@/app/ui/setup/BenefitCard';
import { WelcomeCard } from '@/app/ui/setup/WelcomeCard';
import { SetupContainer, SetupCard } from '@/app/ui/setup/components';
import AppInitializer from '@/components/AppInitializer';
// ... rest of imports remain the same

function SetupContent() {
  // ... all existing state and hooks remain the same

  return (
    <SetupContainer>
      <WelcomeCard />

      <motion.div className="grid grid-cols-1 md:grid-cols-2 gap-4" variants={itemVariants}>
        {benefits.map((benefit) => (
          <BenefitCard key={benefit.title} {...benefit} />
        ))}
      </motion.div>

      <SetupCard
        title="Setup Your Bodhi Server"
      >
        <Form {...form}>
          {/* Form content remains exactly the same */}
        </Form>
      </SetupCard>
    </SetupContainer>
  );
}

export default function Setup() {
  return (
    <AppInitializer allowedStatus="setup" authenticated={false}>
      <SetupContent />
    </AppInitializer>
  );
}
```

## Verification
- Run: `cd crates/bodhi && npm run test`
- Specifically check: `npm run test -- page.test.tsx`
- Visual check: Page should look the same but use shared components
- If tests fail, fix them before proceeding

---

# PHASE 4: Refactor Admin Setup Page

## Objective
Refactor the resource-admin page to use shared components and ensure logo is displayed.

## Tasks

### 4.1 Backup existing page
- Copy `/crates/bodhi/src/app/ui/setup/resource-admin/page.tsx` to `page.tsx.backup`

### 4.2 Refactor resource-admin page
**File**: `/crates/bodhi/src/app/ui/setup/resource-admin/page.tsx`

Key changes:
- Use `SetupContainer` (fixes missing logo issue)
- Use `SetupCard` for content
- Use `SetupNavigation` for button (though customized for OAuth flow)

## Verification
- Run: `cd crates/bodhi && npm run test -- resource-admin`
- Logo should now be visible on this page
- OAuth flow should work as before

---

# PHASE 5: Refactor Download Models Page

## Objective
Change container width from max-w-7xl to max-w-4xl for consistency. Adjust layout accordingly.

## Tasks

### 5.1 Backup existing page
- Copy `/crates/bodhi/src/app/ui/setup/download-models/page.tsx` to `page.tsx.backup`

### 5.2 Refactor download-models page
**File**: `/crates/bodhi/src/app/ui/setup/download-models/page.tsx`

Key changes:
- Use `SetupContainer` (enforces max-w-4xl)
- Keep existing model grid but adjust responsive breakpoints
- Consider changing grid from 3 columns to 2 for better fit
- Use `SetupNavigation` for continue button

### 5.3 Update ModelCard component if needed
Adjust ModelCard sizing if necessary to fit better in narrower container.

## Verification
- Run: `cd crates/bodhi && npm run test -- download-models`
- Check visual appearance - models should fit well in narrower container
- All download functionality should work

---

# PHASE 6: Refactor API Models Page

## Objective
Refactor API models page for consistency while maintaining form functionality.

## Tasks

### 6.1 Backup existing page
- Copy `/crates/bodhi/src/app/ui/setup/api-models/page.tsx` to `page.tsx.backup`

### 6.2 Refactor api-models page
**File**: `/crates/bodhi/src/app/ui/setup/api-models/page.tsx`

Key changes:
- Use `SetupContainer`
- Use `SetupCard` for intro and form sections
- Use `SetupNavigation` for skip button

## Verification
- Run: `cd crates/bodhi && npm run test -- api-models`
- Form functionality should remain intact
- Navigation should work

---

# PHASE 7: Refactor Browser Extension Page

## Objective
Refactor browser extension page for consistency.

## Tasks

### 7.1 Backup existing page
- Copy `/crates/bodhi/src/app/ui/setup/browser-extension/page.tsx` to `page.tsx.backup`

### 7.2 Refactor browser-extension page
**File**: `/crates/bodhi/src/app/ui/setup/browser-extension/page.tsx`

Key changes:
- Use `SetupContainer`
- Use `SetupCard` for different states
- Use `SetupNavigation` where appropriate

## Verification
- Run: `cd crates/bodhi && npm run test -- browser-extension`
- Extension detection should work
- All states should display correctly

---

# PHASE 8: Refactor Complete Page

## Objective
Refactor completion page while maintaining celebration elements.

## Tasks

### 8.1 Backup existing page
- Copy `/crates/bodhi/src/app/ui/setup/complete/page.tsx` to `page.tsx.backup`

### 8.2 Refactor complete page
**File**: `/crates/bodhi/src/app/ui/setup/complete/page.tsx`

Key changes:
- Use `SetupContainer` but without progress bar (showProgress={false})
- Keep confetti effect
- Use `SetupCard` for community and resource sections
- Maintain all links and functionality

## Verification
- Run: `cd crates/bodhi && npm run test`
- Confetti should still appear
- All links should work

---

# PHASE 9: Update Shared Animations

## Objective
Ensure consistent animations across all pages.

## Tasks

### 9.1 Update animation variants
**File**: `/crates/bodhi/src/app/ui/setup/types.ts`

Ensure all pages use the same animation variants.

### 9.2 Remove duplicate animation definitions
Remove local animation definitions from individual pages.

## Verification
- Navigate through entire setup flow
- Animations should be smooth and consistent

---

# PHASE 10: Navigation Enhancement

## Objective
Add proper back navigation and improve flow between pages.

## Tasks

### 10.1 Update each page with back navigation
Add proper back button handling to each page (except welcome).

### 10.2 Create navigation utilities
**File**: `/crates/bodhi/src/app/ui/setup/utils/navigation.ts`

```tsx
export function getPreviousStep(currentPath: string): string {
  // Implementation to return previous step URL
}

export function getNextStep(currentPath: string): string {
  // Implementation to return next step URL
}
```

### 10.3 Wire up navigation in each page
Update each page to use the navigation utilities.

## Verification
- Test back button on each page
- Ensure proper flow forward and backward

---

# PHASE 11: Final Testing & Cleanup

## Objective
Comprehensive testing and cleanup of old code.

## Tasks

### 11.1 Run full test suite
```bash
cd crates/bodhi
npm run build
npm run test
npm run test:typecheck
```

### 11.2 Fix any failing tests
Update test files if component changes require it.

### 11.3 Remove backup files
Delete all `.backup` files created during refactoring.

### 11.4 Remove unused imports
Clean up any unused imports from refactored pages.

## Verification
- All tests pass
- Build succeeds
- TypeScript has no errors

---

# PHASE 12: Visual QA & Documentation

## Objective
Final visual check and documentation update.

## Tasks

### 12.1 Visual walkthrough
- Start from welcome page
- Navigate through entire flow
- Check responsive design
- Verify all interactions work

### 12.2 Update component documentation
Add JSDoc comments to new shared components.

### 12.3 Create migration guide
Document what changed for other developers.

### 12.4 Performance check
- Check bundle size hasn't increased significantly
- Verify no performance regressions

## Final Verification Checklist
- [ ] All pages use consistent max-w-4xl width
- [ ] Logo appears on all pages
- [ ] Progress bar is consistent
- [ ] Navigation works forward and backward
- [ ] All tests pass
- [ ] Build succeeds
- [ ] TypeScript check passes
- [ ] Visual appearance is improved
- [ ] No functionality is broken

---

# Success Criteria

The refactoring is complete when:
1. All setup pages use shared components
2. Container width is consistent (max-w-4xl)
3. Logo appears on all pages
4. Progress bar implementation is unified
5. All existing tests pass
6. Build and type checking succeed
7. Navigation is intuitive with back/forward options
8. Visual consistency is achieved across all pages

# Rollback Plan

If critical issues arise:
1. All original files are backed up with `.backup` extension
2. Can revert using git: `git checkout -- crates/bodhi/src/app/ui/setup/`
3. Delete new components folder if needed
4. Restore from backups if git not available

# Notes for Agent

- **IMPORTANT**: Run tests after EVERY phase
- If tests fail, fix them before moving to next phase
- Update log file after each phase completion
- Document any unexpected findings in context file
- If stuck, document the issue clearly and try alternative approach
- Maintain backward compatibility for AppInitializer wrapper
- Don't modify test files unless absolutely necessary