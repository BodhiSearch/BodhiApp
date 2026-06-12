# Phase 2 Agent Instructions: Configure for Subpath Testing

## Agent Responsibilities

You are executing **Phase 2** of the website migration from `bodhisearch.github.io` to the BodhiApp monorepo.

**IMPORTANT: Before starting, you MUST:**
1. Read `ai-docs/specs/20251009-getbodhi-app/agent-log.md` to see what previous agents have done
2. Read `ai-docs/specs/20251009-getbodhi-app/agent-ctx.md` for important insights and context

**After completing your work, you MUST:**
1. Update `agent-log.md` with your activities, verification results, and issues
2. Update `agent-ctx.md` with any insights or discoveries for future agents

---

## Phase 2 Overview

**Goal:** Configure website for subpath testing at `bodhisearch.github.io/BodhiApp`

**Why:** Before switching to custom domain, we test deployment at a subpath to ensure everything works without impacting production site.

**Risk Level:** Low - No deployment yet, just configuration changes

**Rollback:** Git revert if configuration breaks build

---

## Task 2.1: Modify Next.js Config for Subpath Deployment

### Objective
Add `basePath: '/BodhiApp'` to Next.js configuration for subpath testing.

### Steps

1. **Navigate to website directory:**
   ```bash
   cd /Users/amir36/Documents/workspace/src/github.com/BodhiSearch/BodhiApp/getbodhi.app
   ```

2. **Read current config:**
   ```bash
   cat next.config.mjs
   ```

3. **Edit next.config.mjs** - Add basePath line:

   **Current config structure:**
   ```javascript
   const nextConfig = {
     output: 'export',
     trailingSlash: true,
     // ... other settings
   };
   ```

   **Add this line after `output: 'export',`:**
   ```javascript
   basePath: '/BodhiApp',
   ```

   **Final structure should be:**
   ```javascript
   const nextConfig = {
     output: 'export',
     basePath: '/BodhiApp',  // ← ADD THIS LINE
     trailingSlash: true,
     // ... rest unchanged
   };
   ```

4. **Verify the change:**
   ```bash
   grep "basePath" next.config.mjs
   ```

   Expected output: Should show the basePath line

### Verification Checklist for Task 2.1

- [ ] `next.config.mjs` edited successfully
- [ ] `basePath: '/BodhiApp',` line added
- [ ] Line placed after `output: 'export',`
- [ ] No syntax errors in file

---

## Task 2.2: Temporarily Remove CNAME File

### Objective
Move CNAME file to backup so it doesn't interfere with subpath testing.

### Steps

1. **Verify CNAME exists:**
   ```bash
   ls -la public/CNAME
   cat public/CNAME
   ```

   Expected: File exists with content `getbodhi.app`

2. **Move to backup:**
   ```bash
   mv public/CNAME public/CNAME.backup
   ```

3. **Verify move succeeded:**
   ```bash
   ls -la public/CNAME      # Should NOT exist
   ls -la public/CNAME.backup  # Should exist
   cat public/CNAME.backup     # Should show: getbodhi.app
   ```

### Verification Checklist for Task 2.2

- [ ] `public/CNAME` file moved to `public/CNAME.backup`
- [ ] Original `public/CNAME` no longer exists
- [ ] Backup contains correct content: `getbodhi.app`

---

## Task 2.3: Test Build with basePath Configuration

### Objective
Verify the build works with basePath and URLs are prefixed correctly.

### Steps

1. **Build the site:**
   ```bash
   npm run build
   ```

   Expected: Build completes successfully

2. **Check build output exists:**
   ```bash
   ls -la out/
   ```

   Expected: `out/` directory with `index.html`, `_next/`, etc.

3. **Verify NO CNAME in output:**
   ```bash
   ls out/CNAME
   ```

   Expected: File should NOT exist (returns "No such file")

4. **Verify URLs have basePath prefix:**
   ```bash
   cat out/index.html | grep -o 'href="[^"]*"' | head -10
   ```

   Expected: URLs should start with `/BodhiApp/` not just `/`

   Example expected output:
   ```
   href="/BodhiApp/_next/static/..."
   href="/BodhiApp/docs/"
   ```

5. **Additional verification - check asset paths:**
   ```bash
   cat out/index.html | grep -o 'src="[^"]*"' | head -5
   ```

   Expected: Asset paths should also have `/BodhiApp/` prefix

### Verification Checklist for Task 2.3

- [ ] `npm run build` completed successfully
- [ ] `out/` directory created
- [ ] NO `out/CNAME` file (should not exist)
- [ ] URLs in HTML start with `/BodhiApp/`
- [ ] Asset paths (src="...") also have `/BodhiApp/` prefix
- [ ] No build errors or warnings (ignore deprecation warnings)

### Troubleshooting

**If build fails:**
- Check next.config.mjs syntax (commas, brackets)
- Verify basePath line is correct
- Try rebuilding: `rm -rf .next out && npm run build`
- Document error in agent-ctx.md

**If URLs don't have basePath:**
- Verify basePath line is exactly: `basePath: '/BodhiApp',`
- Check spelling and case sensitivity
- Rebuild after fixing

---

## Task 2.4: Create GitHub Actions Workflow

### Objective
Create workflow file for deploying website to GitHub Pages.

### Steps

1. **Navigate to project root:**
   ```bash
   cd /Users/amir36/Documents/workspace/src/github.com/BodhiSearch/BodhiApp
   ```

2. **Create workflow directory if needed:**
   ```bash
   mkdir -p .github/workflows
   ```

3. **Create workflow file:**

   File: `.github/workflows/deploy-website.yml`

   **IMPORTANT: Use ONLY workflow_dispatch trigger (manual only for now)**

   ```yaml
   name: Deploy Website to GitHub Pages

   on:
     # Manual trigger only
     workflow_dispatch:

   permissions:
     contents: read
     pages: write
     id-token: write

   concurrency:
     group: "pages"
     cancel-in-progress: false

   jobs:
     build:
       runs-on: ubuntu-latest
       defaults:
         run:
           working-directory: getbodhi.app
       steps:
         - name: Checkout
           uses: actions/checkout@v4

         - name: Setup Node
           uses: actions/setup-node@v4
           with:
             node-version: "20"
             cache: 'npm'
             cache-dependency-path: getbodhi.app/package-lock.json

         - name: Setup Pages
           uses: actions/configure-pages@v5
           with:
             static_site_generator: next

         - name: Restore cache
           uses: actions/cache@v4
           with:
             path: |
               getbodhi.app/.next/cache
             key: ${{ runner.os }}-nextjs-${{ hashFiles('getbodhi.app/package-lock.json') }}-${{ hashFiles('getbodhi.app/**.[jt]s', 'getbodhi.app/**.[jt]sx') }}
             restore-keys: |
               ${{ runner.os }}-nextjs-${{ hashFiles('getbodhi.app/package-lock.json') }}-

         - name: Install dependencies
           run: npm ci

         - name: Build with Next.js
           run: npm run build

         - name: Upload artifact
           uses: actions/upload-pages-artifact@v3
           with:
             path: ./getbodhi.app/out

     deploy:
       environment:
         name: github-pages
         url: ${{ steps.deployment.outputs.page_url }}
       runs-on: ubuntu-latest
       needs: build
       steps:
         - name: Deploy to GitHub Pages
           id: deployment
           uses: actions/deploy-pages@v4
   ```

4. **Verify workflow file created:**
   ```bash
   ls -la .github/workflows/deploy-website.yml
   cat .github/workflows/deploy-website.yml | head -20
   ```

5. **Validate YAML syntax (basic check):**
   ```bash
   cat .github/workflows/deploy-website.yml | grep -E "^(name|on|jobs|steps):" | head -10
   ```

   Expected: Should show proper YAML structure

### Verification Checklist for Task 2.4

- [ ] `.github/workflows/deploy-website.yml` created
- [ ] Uses `workflow_dispatch` trigger only (no automatic triggers)
- [ ] Workflow has two jobs: `build` and `deploy`
- [ ] Working directory set to `getbodhi.app`
- [ ] Node.js version 20 specified
- [ ] Proper caching configured
- [ ] YAML syntax looks valid

---

## Task 2.5: Stage Changes to Git

### Objective
Stage all Phase 2 changes for manual commit by user.

### Steps

1. **Navigate to project root:**
   ```bash
   cd /Users/amir36/Documents/workspace/src/github.com/BodhiSearch/BodhiApp
   ```

2. **Check what changed:**
   ```bash
   git status
   ```

   Expected changes:
   - Modified: `getbodhi.app/next.config.mjs`
   - Renamed: `getbodhi.app/public/CNAME` → `getbodhi.app/public/CNAME.backup`
   - New file: `.github/workflows/deploy-website.yml`

3. **Stage the changes:**
   ```bash
   git add getbodhi.app/next.config.mjs
   git add getbodhi.app/public/CNAME.backup
   git add .github/workflows/deploy-website.yml
   ```

4. **Verify staging:**
   ```bash
   git status
   git diff --cached --stat
   ```

   Expected: Shows 3 files staged (modified, renamed, new)

5. **Check diff for next.config.mjs:**
   ```bash
   git diff --cached getbodhi.app/next.config.mjs
   ```

   Expected: Should show `+  basePath: '/BodhiApp',` line added

### Verification Checklist for Task 2.5

- [ ] `getbodhi.app/next.config.mjs` staged (modified)
- [ ] `getbodhi.app/public/CNAME.backup` staged (added/renamed)
- [ ] `.github/workflows/deploy-website.yml` staged (new file)
- [ ] Git diff shows basePath line added
- [ ] No unexpected files staged

### Important Note

**DO NOT RUN `git commit`**

The user prefers to do commits manually. Your job is to stage the files only.

**Suggested commit message for user (document in agent-log.md):**
```
Configure website for subpath testing (Phase 2)

- Add basePath: '/BodhiApp' to Next.js config for testing
- Temporarily remove CNAME (moved to CNAME.backup)
- Create GitHub Actions workflow for Pages deployment
- Manual trigger only (workflow_dispatch)
- Ready for Phase 3: Test deployment to bodhisearch.github.io/BodhiApp
```

---

## Final Steps: Update Context Files

### Update agent-log.md

Add entry in this format:

```markdown
## Agent: Phase 2 Execution - Subpath Configuration
**Date:** 2025-10-09
**Status:** Completed

### Tasks Performed
1. Modified next.config.mjs to add basePath: '/BodhiApp'
2. Moved public/CNAME to public/CNAME.backup (temporary)
3. Tested build with basePath - verified URLs have correct prefix
4. Created GitHub Actions workflow (.github/workflows/deploy-website.yml)
5. Staged all changes to git (not committed per user preference)

### Verification Results
- [x] basePath added to next.config.mjs
- [x] CNAME file backed up (not in build output)
- [x] Build succeeds with basePath configuration
- [x] URLs in HTML have /BodhiApp/ prefix
- [x] Asset paths also prefixed correctly
- [x] GitHub Actions workflow created (manual trigger only)
- [x] All changes staged in git

### Issues Encountered
[Document any issues here, or write "None"]

### Build Results
- Build time: [X seconds]
- Pages generated: [X pages]
- Asset URL examples: [paste a few URLs from out/index.html]

### Files Created/Modified
- Modified: getbodhi.app/next.config.mjs (added basePath)
- Added: getbodhi.app/public/CNAME.backup
- Removed: getbodhi.app/public/CNAME (moved to backup)
- Created: .github/workflows/deploy-website.yml

### Notes for Next Agent (Phase 3)
- Phase 2 complete and verified
- Ready for Phase 3: Test Deployment to Subpath
- User should manually commit before Phase 3
- Suggested commit message provided above
- Build is configured for /BodhiApp/ subpath
- CNAME is backed up and will be restored in Phase 6
- Workflow uses workflow_dispatch only (manual trigger)
```

### Update agent-ctx.md

If you discovered anything important, add to agent-ctx.md:

```markdown
## Phase 2 Execution - basePath Configuration Insights
**Date:** 2025-10-09
**Agent:** Phase 2 Execution

### Discovery
[What you learned about basePath configuration, build behavior, etc.]

### Impact
[How this affects Phase 3 deployment testing]

### Recommendation
[What Phase 3 agent should know or watch out for]
```

---

## Success Criteria

Phase 2 is complete when:

1. ✅ basePath added to next.config.mjs
2. ✅ CNAME file backed up (not in build)
3. ✅ Build tested and URLs have /BodhiApp/ prefix
4. ✅ GitHub Actions workflow created (manual trigger only)
5. ✅ Changes staged in git (not committed)
6. ✅ agent-log.md updated with your activities
7. ✅ agent-ctx.md updated with insights (if any)

---

## Important Reminders

- **Read agent-log.md and agent-ctx.md FIRST** before starting
- **Update both files** when you complete your work
- **DO NOT commit to git** - user does this manually
- **Document any issues** in agent-ctx.md to help next agent
- **Verify each step** before proceeding to the next
- **This phase is safe** - no deployment, no production impact
- **Workflow uses workflow_dispatch only** - no automatic triggers

---

## Commands Summary

```bash
# Task 2.1: Modify config
cd /Users/amir36/Documents/workspace/src/github.com/BodhiSearch/BodhiApp/getbodhi.app
# Edit next.config.mjs to add: basePath: '/BodhiApp',

# Task 2.2: Remove CNAME
mv public/CNAME public/CNAME.backup

# Task 2.3: Test build
npm run build
cat out/index.html | grep -o 'href="[^"]*"' | head -10

# Task 2.4: Create workflow
cd /Users/amir36/Documents/workspace/src/github.com/BodhiSearch/BodhiApp
# Create .github/workflows/deploy-website.yml with content above

# Task 2.5: Stage changes
git add getbodhi.app/next.config.mjs
git add getbodhi.app/public/CNAME.backup
git add .github/workflows/deploy-website.yml
git status

# Update context files
# (Use your preferred text editor or tools)
```

Good luck with Phase 2! Remember to update the log and context files when done.
