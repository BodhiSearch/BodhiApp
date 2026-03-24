# Plan: Migrate Documentation to getbodhi.app

## Context

During the Next.js → Vite + TanStack Router migration (commit 5b6b2a6b), docs were moved to `crates/bodhi/pending/` as a staging area. The decision has been made to make `getbodhi.app/` the permanent source of truth for documentation, removing the old sync-from-embedded-app pattern.

**Key finding**: The files in `crates/bodhi/pending/` are byte-identical with their counterparts in `getbodhi.app/` — the content "move" is a no-op. The work is removing sync infrastructure, restoring app navigation links, and deleting the staging area.

---

## Phase 1: Website Cleanup (getbodhi.app)

### 1.1 Delete sync script
- **Delete** `getbodhi.app/scripts/sync-docs.js`

### 1.2 Edit `getbodhi.app/package.json`
Remove 3 lines from `scripts`:
- `"sync:docs": "node scripts/sync-docs.js"`
- `"sync:docs:check": "node scripts/sync-docs.js --check"`
- `"prebuild": "npm run sync:docs"`

### 1.3 Edit `getbodhi.app/Makefile`
- Delete `sync.docs` target (lines 8-9)
- Delete `sync.docs.check` target (lines 11-12)
- Change `release.precheck: sync.docs.check` → `release.precheck:` (remove dependency)
- Remove the "Docs sync check passed" echo (line 37)
- Remove `sync.docs sync.docs.check` from `.PHONY` (line 77)
- Update `build` comment to remove "(auto-syncs docs first via prebuild)" (line 20)

---

## Phase 2: Root Makefile Cleanup

### 2.1 Edit `Makefile.website.mk`
- Delete `docs.sync` target (lines 18-19)
- Delete `docs.sync-check` target (lines 21-22)
- Remove `docs.sync docs.sync-check` from `.PHONY` (line 5)
- Remove comment "# Website documentation sync" (line 17)

---

## Phase 3: App Navigation Restoration

### 3.1 Edit `crates/bodhi/src/hooks/navigation/useNavigation.tsx`
Add `BookOpen`, `BookText`, `FileJson` to lucide-react imports.

Restore "Documentation" group after the "Settings" group (after line 179):
```tsx
{
  title: 'Documentation',
  icon: BookText,
  items: [
    {
      title: 'App Guide',
      href: 'https://getbodhi.app/docs/',
      description: 'User guides and documentation',
      icon: BookOpen,
      target: '_blank',
    },
    {
      title: 'OpenAPI Docs',
      href: '/swagger-ui',
      description: 'API Documentation',
      icon: FileJson,
      target: '_blank',
    },
  ],
},
```

No type changes needed — `NavigationItem` already has `target?: string`.
No renderer changes needed — `AppNavigation.tsx` already handles `target` with `<a>` tags (lines 98-106, 122-128).

### 3.2 Edit `crates/bodhi/src/app/setup/complete/page.tsx`
Update `resourceLinks` (line 60-67):
- Change URL: `'https://docs.getbodhi.app/getting-started'` → `'https://getbodhi.app/docs/'`
- Add `target: '_blank'` and `rel: 'noopener noreferrer'` to the `<motion.a>` at line 162 (match the pattern used by `socialLinks` at lines 138-139)

---

## Phase 4: Test Updates

### 4.1 Edit `crates/bodhi/src/hooks/navigation/useNavigation.test.tsx`
- Restore the "Documentation" group in `testNavigationItems` (was removed in the Vite migration)
- Restore the docs path test case in the parameterized path-matching tests: `['/docs/', 'App Guide', '/docs/', 'User guides and documentation', 'Documentation']`
- Add assertions for `target: '_blank'` on external links

### 4.2 Verify setup complete page tests
- Test at `crates/bodhi/src/app/setup/complete/page.test.tsx` checks for 'Getting Started Guide' text — title isn't changing so existing test passes
- Add assertion for new URL and `target="_blank"` attribute

---

## Phase 5: Delete Pending Directory

### 5.1 Delete `crates/bodhi/pending/` entirely
- `docs-md/` — 22 markdown + 10 `_meta.json` (identical in `getbodhi.app/src/docs/`)
- `doc-images/` — 28 images (identical in `getbodhi.app/public/doc-images/`)
- `docs-react/` — rendering components (identical in `getbodhi.app/src/app/docs/`)

---

## Verification

1. **Frontend tests**: `cd crates/bodhi && npm test` — navigation and setup-complete tests pass
2. **Website build**: `cd getbodhi.app && npm run build` — static export succeeds without prebuild hook
3. **Makefile targets**: `make -C getbodhi.app release.precheck` — runs without sync.docs.check
4. **Visual check**: `cd crates/bodhi && npm run dev` — Documentation group appears in sidebar with "App Guide" and "OpenAPI Docs", both open in new tabs

---

## Files Modified

| File | Action |
|------|--------|
| `getbodhi.app/scripts/sync-docs.js` | Delete |
| `getbodhi.app/package.json` | Remove sync scripts + prebuild |
| `getbodhi.app/Makefile` | Remove sync targets, simplify release.precheck |
| `Makefile.website.mk` | Remove docs.sync targets |
| `crates/bodhi/src/hooks/navigation/useNavigation.tsx` | Restore Documentation nav group |
| `crates/bodhi/src/hooks/navigation/useNavigation.test.tsx` | Restore docs test cases |
| `crates/bodhi/src/app/setup/complete/page.tsx` | Update docs URL + target blank |
| `crates/bodhi/src/app/setup/complete/page.test.tsx` | Verify new URL + target |
| `crates/bodhi/pending/` | Delete entirely |
