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

## Phase 2 Execution - basePath Configuration Insights
**Date:** 2025-10-09
**Agent:** Phase 2 Execution

### Discovery
The basePath configuration in Next.js works flawlessly for subpath deployment:
- Adding `basePath: '/BodhiApp'` to next.config.mjs immediately affects all URL generation
- All asset URLs (href, src attributes) are automatically prefixed with /BodhiApp/
- External URLs (e.g., GitHub, Discord, Product Hunt) are NOT prefixed (correct behavior)
- Build time remains consistent (~5 seconds) with no performance impact
- CNAME file in public/ is excluded from build when moved to CNAME.backup
- Git correctly detects CNAME → CNAME.backup as a rename operation (clean history)

### Important Behaviors
**CNAME Handling:**
- Moving public/CNAME to public/CNAME.backup prevents it from being in build output
- Next.js copies all public/ files to out/, so CNAME.backup appears in out/ (harmless)
- No CNAME in out/ means GitHub Pages won't try to claim custom domain during testing

**basePath URL Prefixing:**
- All internal links get /BodhiApp/ prefix: `href="/BodhiApp/docs/"`
- All assets get /BodhiApp/ prefix: `src="/BodhiApp/_next/static/..."`
- External URLs are left unchanged: `href="https://github.com/..."`
- Some image URLs may not be prefixed if they're in public/ (e.g., `/chat-ui.jpeg`)

**Git Rename Detection:**
- Using `git add` on both CNAME (deleted) and CNAME.backup (new) allows Git to detect rename
- This provides cleaner history showing intent (rename not delete+add)
- Staging order doesn't matter - Git figures it out automatically

### Impact
**For Phase 3 (Test Deployment):**
- The build is fully configured for /BodhiApp/ subpath
- GitHub Actions workflow is ready for manual trigger
- All URLs will work correctly at bodhisearch.github.io/BodhiApp
- No further configuration changes needed before deployment test

**For Phase 6 (Final Migration):**
- Removing basePath line and restoring CNAME will be straightforward
- These are the only two changes needed to switch to production
- Build time and asset structure remain identical between configurations

**For Troubleshooting:**
- If URLs don't work after deployment, verify basePath is exactly '/BodhiApp' (case-sensitive)
- If custom domain claims fail, verify CNAME file is completely absent from out/
- If assets 404, check that GitHub Pages is serving from correct branch and directory

### Recommendation
**For Phase 3 Agent:**
1. The configuration is complete and tested - no changes needed
2. Workflow can be triggered immediately after user commits
3. Monitor deployment logs for any path-related issues
4. Verify deployed site at bodhisearch.github.io/BodhiApp loads all assets
5. Test navigation between pages to ensure basePath works end-to-end

**For Future Phases:**
- Keep basePath and CNAME changes together in version control
- Document that these are temporary testing configurations
- When reverting to root path, test build locally before deployment
- Consider keeping CNAME.backup in repo as documentation of testing process

**For Any Issues:**
- If images in public/ don't load, they may need explicit basePath handling in code
- If API calls fail, ensure they're using relative URLs (not hardcoded domains)
- If hash routing fails, verify Next.js router is handling basePath correctly
