# Plan: Remove old Next.js `src/app/` — Inline into TanStack Router `src/routes/`

## Context

The Next.js-to-TanStack-Router migration (commit `5b6b2a6b`, 2026-03-21) moved routing to `src/routes/` but left all page implementations in the old `src/app/` directory. Route files are thin wrappers like:

```tsx
import LoginPage from '@/app/login/page';
export const Route = createFileRoute('/login/')({ component: LoginPage });
```

This creates confusion (looks like dead Next.js code) and splits each page across two locations unnecessarily. Goal: inline page components into route files, co-locate sub-components using TanStack Router's `-` prefix convention, and delete `src/app/` entirely.

## Scope

- **127 files** in `src/app/`: 33 page.tsx, ~40 sub-components, 6 data/type files, 44 tests, globals.css, favicon.ico
- **39 route files** in `src/routes/` to be expanded with inlined content

## Key Decisions

| Decision | Choice |
|----------|--------|
| Sub-components | Co-locate in `-components/` dirs inside `src/routes/<domain>/` (TanStack ignores `-` prefix) |
| Tests | Co-locate alongside their components (e.g., `src/routes/chat/index.test.tsx`, `src/routes/chat/-components/ChatMessage.test.tsx`) |
| `globals.css` | Move to `src/styles/globals.css` |
| Model catalog data/types | Move to `src/hooks/models/` (shared between setup pages and `useModelCatalog` hook) |
| `favicon.ico` | Delete from `src/app/` (already exists in `public/`) |

## Import Rewriting Rules

| Old pattern | New pattern |
|-------------|-------------|
| `from '@/app/<domain>/page'` | eliminated (inlined) |
| `from '@/app/<domain>/<Component>'` | `from './-components/<Component>'` |
| `from '@/app/<domain>/components/<X>'` | `from './-components/<X>'` or `from '../-components/<X>'` |
| `from '@/app/setup/types'` | `from '../-shared/types'` |
| `from '@/app/setup/constants'` | `from '../-shared/constants'` |
| `from '@/app/setup/download-models/data'` | `from '@/hooks/models/model-catalog-data'` |
| `from '@/app/setup/download-models/types'` | `from '@/hooks/models/model-catalog-types'` |
| `import '@/app/globals.css'` | `import '@/styles/globals.css'` |
| Test: `from '@/app/login/page'` | `from '@/routes/login/index'` (or relative) |

Named exports from page files (e.g., `LoginContent`, `SettingsPageContent`) are preserved as exports in the route file — TanStack Router only uses the `Route` export.

## Phases

### Phase 1: Infrastructure (no route changes)

1. **Move `globals.css`** to `src/styles/globals.css`
   - Update `src/routes/__root.tsx` line 3
   - Update `components.json` line 8
2. **Delete `src/app/favicon.ico`** (duplicate of `public/favicon.ico`)
3. **Move model catalog data** to hooks layer:
   - `src/app/setup/download-models/types.ts` -> `src/hooks/models/model-catalog-types.ts`
   - `src/app/setup/download-models/data.ts` -> `src/hooks/models/model-catalog-data.ts`
   - Update `src/hooks/models/useModelCatalog.ts` imports

**Gate**: `cd crates/bodhi && npm test`

### Phase 2: Simple pages (12 pages with no sub-components)

Pages where `page.tsx` content is directly inlined into `index.tsx`:

`/` (root), `/home/`, `/login/`, `/request-access/`, `/auth/callback/`, `/auth/dashboard/callback/`, `/users/`, `/users/pending/`, `/users/access-requests/`, `/models/files/`, `/models/files/pull/`, `/mcps/oauth/callback/`

For each:
1. Merge `page.tsx` content into route `index.tsx` (imports + component body)
2. Move `page.test.tsx` to co-located `index.test.tsx`, update test import path
3. Delete the `src/app/<path>/` directory

**Gate**: `cd crates/bodhi && npm test`

### Phase 3: Medium pages with sub-components

#### 3a: Settings (2 source + 2 test files)
```
src/routes/settings/
  index.tsx                    (inlined page)
  index.test.tsx
  -components/EditSettingDialog.tsx + .test.tsx
```

#### 3b: Tokens (4 source + 3 test files)
```
src/routes/tokens/
  index.tsx, index.test.tsx
  -components/TokenDialog.tsx, TokenForm.tsx, CreateTokenDialog.tsx + tests
```

#### 3c: Models domain (15 source files + tests)
```
src/routes/models/
  index.tsx, index.test.tsx
  -components/ModelTableRow.tsx, ModelPreviewModal.tsx, ModelActions.tsx, SourceBadge.tsx, tooltips.ts
  alias/-components/AliasForm.tsx
  alias/new/index.tsx + test
  alias/edit/index.tsx + test
  api/new/index.tsx + test
  api/edit/index.tsx + test
  files/pull/-components/PullForm.tsx + test
```

#### 3d: MCPs domain (12 source files + tests)
```
src/routes/mcps/
  index.tsx, index.test.tsx
  new/index.tsx, new/-components/McpServerSelector.tsx + tests
  servers/index.tsx, servers/-components/AuthConfigForm.tsx
  servers/new/ servers/edit/ servers/view/ — index.tsx + tests
  playground/index.tsx + test
```

#### 3e: Apps domain (3 files)
```
src/routes/apps/access-requests/review/
  index.tsx, index.test.tsx, -components/McpServerCard.tsx
```

**Gate**: `cd crates/bodhi && npm test` after each sub-phase

### Phase 4: Chat page (12 source + 8 test files)

```
src/routes/chat/
  index.tsx                    (inlined page)
  index.test.tsx
  -components/
    ChatUI.tsx, ChatMessage.tsx + test, ChatHistory.tsx + test
    McpsPopover.tsx, NewChatButton.tsx + test, ToolCallMessage.tsx
    settings/
      SettingsSidebar.tsx + test, AliasSelector.tsx + test
      SettingSlider.tsx + test, StopWords.tsx + test
      SystemPrompt.tsx + test, tooltips.ts
```

**Gate**: `cd crates/bodhi && npm test`

### Phase 5: Setup domain (28 files — most complex)

```
src/routes/setup/
  route.tsx                    (inlined layout.tsx)
  index.tsx, index.test.tsx
  -shared/types.ts, constants.ts
  -components/
    index.ts (barrel), SetupProvider.tsx, SetupContainer.tsx
    SetupCard.tsx, SetupFooter.tsx, SetupNavigation.tsx
    BenefitCard.tsx, WelcomeCard.tsx, SetupModeCard.tsx
    SetupProgress.tsx + test, BodhiLogo.tsx
  download-models/
    index.tsx + test
    -components/ModelCard.tsx + test, ModelList.tsx, RatingStars.tsx
  api-models/ llm-engine/ browser-extension/ complete/ resource-admin/ tenants/
    — each: index.tsx + test
```

**Gate**: `cd crates/bodhi && npm test`

### Phase 6: Final cleanup

1. Verify no remaining `@/app/` imports: `grep -r "from '@/app/" src/`
2. Delete `src/app/` directory entirely
3. Verify TanStack Router route tree regenerates correctly (check `routeTree.gen.ts` has no `-components` entries)
4. Update `crates/bodhi/src/CLAUDE.md` to remove references to `src/app/` pattern
5. Run full verification

**Gate**: `cd crates/bodhi && npm test` + `cd crates/bodhi && npm run build`

## Critical Files

- `crates/bodhi/src/routes/__root.tsx` — globals.css import (Phase 1)
- `crates/bodhi/src/hooks/models/useModelCatalog.ts` — model catalog data import (Phase 1)
- `crates/bodhi/components.json` — shadcn css path (Phase 1)
- `crates/bodhi/vite.config.ts` — TanStack Router plugin config (verify `-` prefix behavior)
- `crates/bodhi/src/routeTree.gen.ts` — auto-generated; verify `-components` dirs are excluded

## Verification

After each phase gate:
```bash
cd crates/bodhi && npm test          # all component tests pass
```

After Phase 6 (final):
```bash
cd crates/bodhi && npm run build     # production build succeeds
grep -r "@/app/" crates/bodhi/src/   # zero results
cd crates/bodhi && npm run dev       # dev server starts, pages render
```
