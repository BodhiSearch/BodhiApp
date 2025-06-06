# Migration Guides

This document provides comprehensive guides for migrating between different frameworks and architectural patterns in the Bodhi App project.

## NextJS to React+Vite Migration

### Overview

Successfully migrated the Bodhi App codebase from Next.js component structure conventions to React+Vite conventions, moving all page-specific components and updating import paths accordingly.

### Migration Phases

#### Phase 1: Component Structure Migration

**Moved 10 component folders** from Next.js convention to React+Vite convention:

| From (Next.js) | To (React+Vite) | Status |
|----------------|-----------------|---------|
| `src/app/ui/chat/*` | `src/components/chat/*` | ✅ Moved |
| `src/app/ui/home/*` | `src/components/home/*` | ✅ Moved |
| `src/app/ui/login/*` | `src/components/login/*` | ✅ Moved |
| `src/app/ui/models/*` | `src/components/models/*` | ✅ Moved |
| `src/app/ui/modelfiles/*` | `src/components/modelfiles/*` | ✅ Moved |
| `src/app/ui/pull/*` | `src/components/pull/*` | ✅ Moved |
| `src/app/ui/settings/*` | `src/components/settings/*` | ✅ Moved |
| `src/app/ui/setup/*` | `src/components/setup/*` | ✅ Merged with existing |
| `src/app/ui/tokens/*` | `src/components/tokens/*` | ✅ Moved |
| `src/app/ui/users/*` | `src/components/users/*` | ✅ Moved |

#### Phase 2: Complete NextJS Removal

1. **Removed "use client" directives**: Eliminated all Next.js specific "use client" directives from ~70+ files
2. **Renamed page.tsx files**: Renamed 16 page.tsx files to descriptive component names
3. **Migrated docs functionality**: Moved docs components and updated React Router integration
4. **Removed app folder**: Completely removed the Next.js app folder structure
5. **Migrated layout functionality**: Moved layout.tsx functionality to App.tsx
6. **Updated all imports**: Fixed all import statements to reference new file locations

### File Renaming Pattern

```
# Before (Next.js)
src/components/home/page.tsx
src/components/chat/page.tsx
src/components/models/page.tsx

# After (React+Vite)
src/components/home/HomePage.tsx
src/components/chat/ChatPage.tsx
src/components/models/ModelsPage.tsx
```

### New Structure Achieved

```
src/components/
├── chat/           # Chat-related components
├── home/           # Home page components  
├── login/          # Login page components
├── models/         # Model management components
├── modelfiles/     # Model files components
├── pull/           # Model download components
├── settings/       # Settings page components
├── setup/          # Setup wizard components
├── tokens/         # API tokens components
├── users/          # User management components
├── ui/             # Common UI components (shadcn/ui)
└── docs/           # Documentation components
```

### Import Path Updates

**Before (Next.js):**
```typescript
import { ChatPage } from '@/app/ui/chat/page';
import { TokenForm } from '@/app/ui/tokens/TokenForm';
```

**After (React+Vite):**
```typescript
import { ChatPage } from '@/components/chat/ChatPage';
import { TokenForm } from '@/components/tokens/TokenForm';
```

### Migration Commands

```bash
# Component moves
mv src/app/ui/chat src/components/
mv src/app/ui/home src/components/
mv src/app/ui/login src/components/
mv src/app/ui/models src/components/
mv src/app/ui/modelfiles src/components/
mv src/app/ui/pull src/components/
mv src/app/ui/settings src/components/
mv src/app/ui/tokens src/components/
mv src/app/ui/users src/components/

# Setup merge (existing folder)
cp -r src/app/ui/setup/* src/components/setup/
rm -rf src/app/ui/setup

# Remove empty directories
rm -rf src/app/ui
rm -rf src/app

# Verification
grep -r "@/app/ui/" src/ --include="*.tsx" --include="*.ts"
npm run test -- run
```

### Validation Results

- ✅ **Tests**: All 335 tests passing, 0 failures
- ✅ **Imports**: Zero broken import statements
- ✅ **Structure**: Complete React+Vite structure achieved
- ✅ **Next.js Dependencies**: All Next.js specific code removed
- ✅ **App Folder Migration**: All app folder files properly migrated

## Component Migration Best Practices

### 1. Pre-Migration Checklist

- [ ] Run full test suite to establish baseline
- [ ] Document current import patterns
- [ ] Identify all files that need updates
- [ ] Create backup of current state

### 2. Migration Process

1. **Move component folders** using `mv` commands
2. **Update import statements** in all affected files
3. **Rename files** to follow new conventions
4. **Update test files** and their imports
5. **Verify functionality** with test suite

### 3. Post-Migration Verification

- [ ] All tests passing
- [ ] No broken import statements
- [ ] Application builds successfully
- [ ] All pages render correctly
- [ ] No console errors

### 4. Common Issues and Solutions

#### Issue: Broken Import Paths
**Solution**: Use global search and replace to update import patterns:
```bash
# Find all old imports
grep -r "@/app/ui/" src/ --include="*.tsx" --include="*.ts"

# Replace with new imports (use IDE find/replace)
@/app/ui/ → @/components/
```

#### Issue: Test File Imports
**Solution**: Update test file imports and mocks:
```typescript
// Before
import { HomePage } from '@/app/ui/home/page';

// After
import { HomePage } from '@/components/home/HomePage';
```

#### Issue: Relative Import Conflicts
**Solution**: Convert relative imports to absolute imports:
```typescript
// Before
import { Component } from '../../../ui/component';

// After
import { Component } from '@/components/ui/component';
```

## Framework Migration Guidelines

### General Principles

1. **Incremental Migration**: Move components in logical groups
2. **Test-Driven**: Maintain test coverage throughout migration
3. **Documentation**: Update documentation as you migrate
4. **Validation**: Verify each step before proceeding

### Migration Checklist Template

```markdown
## Migration Checklist

### Pre-Migration
- [ ] Current state documented
- [ ] Test suite passing
- [ ] Backup created

### Migration Steps
- [ ] Component structure updated
- [ ] Import paths updated
- [ ] File naming conventions applied
- [ ] Test files updated

### Post-Migration
- [ ] All tests passing
- [ ] Application builds
- [ ] Manual testing completed
- [ ] Documentation updated
```

### Rollback Strategy

If migration issues occur:

1. **Revert file moves**: Use git to restore original structure
2. **Restore imports**: Revert import path changes
3. **Verify tests**: Ensure all tests pass after rollback
4. **Analyze issues**: Identify what went wrong before retry

## Future Migration Considerations

### Potential Future Migrations

1. **State Management**: Migration to different state management solutions
2. **Styling**: Migration to different CSS frameworks
3. **Testing**: Migration to different testing frameworks
4. **Build Tools**: Migration to different build systems

### Migration Planning

For future migrations:

1. **Impact Assessment**: Analyze scope and complexity
2. **Timeline Planning**: Estimate effort and schedule
3. **Risk Mitigation**: Identify potential issues and solutions
4. **Team Coordination**: Ensure team alignment and communication

---

*This guide ensures smooth and reliable migrations while maintaining code quality and functionality throughout the process.*
