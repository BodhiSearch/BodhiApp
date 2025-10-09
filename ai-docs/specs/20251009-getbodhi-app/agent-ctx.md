# Agent Shared Context

This file contains shared knowledge, insights, and troubleshooting discoveries that agents should be aware of.

## Purpose
- Share insights and troubleshooting solutions between agents
- Document unexpected behaviors and workarounds
- Provide context that's not in the main plan
- Help agents avoid repeating the same troubleshooting

## Format
Agents should add insights in this format:

```markdown
## [Topic/Area] - [Agent Name]
**Date:** YYYY-MM-DD

### Discovery
What was learned or discovered

### Impact
How this affects the migration or future work

### Recommendation
What subsequent agents should do with this information
```

---

## Project Context

### Source Repository
- **Location:** `/Users/amir36/Documents/workspace/src/github.com/BodhiSearch/bodhisearch.github.io`
- **Current deployment:** https://getbodhi.app
- **GitHub Pages:** Enabled with custom CNAME
- **Tech stack:** Next.js 14, TailwindCSS, Shadcn UI

### Target Repository
- **Location:** `/Users/amir36/Documents/workspace/src/github.com/BodhiSearch/BodhiApp`
- **Target directory:** `getbodhi.app/`
- **No deployment yet** - Phase 1 is local only

### Key Technical Facts
1. **CNAME Conflict:** Only one repo can claim `getbodhi.app` at a time
2. **basePath Strategy:** Use `/BodhiApp` for testing, remove for production
3. **Docs Sync:** Website docs come from `crates/bodhi/src/docs/`
4. **No Commits:** User prefers manual git commits (per CLAUDE.md)

---

## Shared Insights

## Phase 1 Execution - Build Verification Insights
**Date:** 2025-10-09
**Agent:** Phase 1 Execution

### Discovery
The website build process is clean and well-configured:
- Next.js 14.2.22 with static export working correctly
- 18 static pages generated successfully (homepage + docs pages)
- Build completes in ~5 seconds (very fast)
- Asset URLs are correctly root-relative (not using basePath yet)
- CNAME file is properly preserved from public/ to out/ during build
- `.gitignore` in the source repo correctly excludes build artifacts

### Dependencies Status
- 541 npm packages installed successfully
- Some deprecation warnings present but non-blocking:
  - eslint@8.57.1 (deprecated)
  - inflight@1.0.6 (deprecated, memory leak)
  - glob@7.2.3 (deprecated)
  - rimraf@3.0.2 (deprecated)
  - @humanwhocodes packages (deprecated)
- 3 security vulnerabilities inherited from source repo (1 low, 1 moderate, 1 critical)
- Browserslist is outdated

### Impact
**For Phase 2 (Subpath Configuration):**
- The build is clean and ready for basePath configuration
- No structural issues that would complicate adding `/BodhiApp` basePath
- Asset loading works correctly, so basePath should integrate smoothly

**For Future Maintenance:**
- Dependencies should be updated when convenient (especially eslint)
- Security vulnerabilities should be audited separately from migration
- The fast build time makes iteration quick for testing

### Recommendation
**For Phase 2 Agent:**
1. The current build is at root path (`/`) - you'll need to add `basePath: '/BodhiApp'` to `next.config.mjs`
2. After adding basePath, verify that asset URLs change from `/` to `/BodhiApp/`
3. CNAME should be temporarily removed or commented out during testing phase
4. The build is fast, so you can iterate quickly on configuration changes

**For Future Agents:**
- Consider updating dependencies in a separate task after migration is complete
- The security vulnerabilities are not urgent but should be addressed in a maintenance cycle
- The Next.js config already has `ignoreBuildErrors: true` and `ignoreDuringBuilds: true` for TypeScript/ESLint, which helps maintain velocity

## Build Artifact Exclusion Pattern
**Date:** 2025-10-09
**Agent:** Phase 1 Execution

### Discovery
The rsync copy command successfully excluded all build artifacts:
- `.git/` directory (source control)
- `node_modules/` (dependencies, 451 directories in source)
- `.next/` (Next.js build cache)
- `out/` (static export output)

The `.gitignore` file in getbodhi.app/ correctly lists these exclusions, so git staging also excluded them automatically.

### Impact
- Clean git history with only source files
- No accidental commit of large binary or generated files
- Future builds won't conflict with git state

### Recommendation
- This exclusion pattern should be maintained for any future file operations
- When copying or syncing files, always use the same exclusion list
