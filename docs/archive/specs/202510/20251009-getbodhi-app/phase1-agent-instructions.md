# Phase 1 Agent Instructions: Setup Website in Monorepo

## Agent Responsibilities

You are executing **Phase 1** of the website migration from `bodhisearch.github.io` to the BodhiApp monorepo.

**IMPORTANT: Before starting, you MUST:**
1. Read `ai-docs/specs/20251009-getbodhi-app/agent-log.md` to see what previous agents have done
2. Read `ai-docs/specs/20251009-getbodhi-app/agent-ctx.md` for important insights and context

**After completing your work, you MUST:**
1. Update `agent-log.md` with your activities, verification results, and issues
2. Update `agent-ctx.md` with any insights or discoveries for future agents

---

## Phase 1 Overview

**Goal:** Copy website files to monorepo and verify local build works

**Risk Level:** Low - No deployment, no changes to production site

**Rollback:** Simply delete `getbodhi.app/` directory if issues occur

---

## Task 1.1: Copy Website Files to Monorepo

### Objective
Copy all source files from `bodhisearch.github.io` to `getbodhi.app/` directory in BodhiApp repo.

### Steps

1. **Navigate to BodhiApp repo:**
   ```bash
   cd /Users/amir36/Documents/workspace/src/github.com/BodhiSearch/BodhiApp
   ```

2. **Verify source repo exists:**
   ```bash
   ls -la /Users/amir36/Documents/workspace/src/github.com/BodhiSearch/bodhisearch.github.io
   ```

   Expected: Directory exists with Next.js project files

3. **Create target directory:**
   ```bash
   mkdir -p getbodhi.app
   ```

4. **Copy files using rsync (excludes build artifacts):**
   ```bash
   rsync -av \
     --exclude='.git' \
     --exclude='node_modules' \
     --exclude='.next' \
     --exclude='out' \
     /Users/amir36/Documents/workspace/src/github.com/BodhiSearch/bodhisearch.github.io/ \
     getbodhi.app/
   ```

5. **Verify copy completed:**
   ```bash
   ls -la getbodhi.app/
   ```

   Expected files/directories:
   - `src/` - Source code
   - `public/` - Static assets
   - `package.json` - Dependencies
   - `package-lock.json` - Lock file
   - `next.config.mjs` - Next.js config
   - `tailwind.config.ts` - Tailwind config
   - `tsconfig.json` - TypeScript config
   - `.gitignore` - Git ignore rules
   - `.eslintrc.json` - ESLint config
   - `components.json` - Shadcn config

6. **Verify CNAME file:**
   ```bash
   cat getbodhi.app/public/CNAME
   ```

   Expected output: `getbodhi.app`

### Verification Checklist for Task 1.1

- [ ] `getbodhi.app/` directory created
- [ ] All source files copied
- [ ] `src/` directory exists with code
- [ ] `public/` directory exists with assets
- [ ] `package.json` exists
- [ ] `.gitignore` present
- [ ] CNAME file in `public/CNAME` contains `getbodhi.app`
- [ ] NO `node_modules/` copied (should be excluded)
- [ ] NO `.next/` copied (should be excluded)
- [ ] NO `out/` copied (should be excluded)
- [ ] NO `.git/` copied (should be excluded)

---

## Task 1.2: Test Local Build

### Objective
Verify the website builds and runs correctly in the new location.

### Steps

1. **Navigate to website directory:**
   ```bash
   cd getbodhi.app
   ```

2. **Install dependencies:**
   ```bash
   npm install
   ```

   Expected: Dependencies install without errors

3. **Test development server:**
   ```bash
   npm run dev
   ```

   This will start dev server on http://localhost:3000

   **Verification (use browser or curl):**
   - Visit http://localhost:3000
   - Homepage should load
   - Navigation should work
   - Images/assets should load
   - No console errors in browser dev tools

   **Stop dev server:** Ctrl+C

4. **Test production build:**
   ```bash
   npm run build
   ```

   Expected: Build completes successfully with no errors

5. **Verify build output:**
   ```bash
   ls -la out/
   cat out/CNAME
   ```

   Expected:
   - `out/` directory exists
   - Contains `index.html`
   - Contains `_next/` directory
   - Contains `CNAME` file with `getbodhi.app`
   - Static assets present

6. **Check build paths (verify no basePath issues):**
   ```bash
   cat out/index.html | grep -o 'href="[^"]*"' | head -5
   ```

   Expected: URLs start with `/` (not `/BodhiApp/` - that comes in Phase 2)

### Verification Checklist for Task 1.2

- [ ] `npm install` completed successfully
- [ ] Dev server started without errors
- [ ] Homepage loads at http://localhost:3000
- [ ] Navigation works in dev mode
- [ ] Assets/images load correctly
- [ ] No console errors
- [ ] `npm run build` completed successfully
- [ ] `out/` directory created
- [ ] `out/index.html` exists
- [ ] `out/CNAME` exists with correct domain
- [ ] `out/_next/` directory exists
- [ ] Asset URLs in HTML start with `/` (root-relative)

### Troubleshooting

**If `npm install` fails:**
- Check Node.js version: `node --version` (should be 20+)
- Check npm version: `npm --version`
- Try removing package-lock.json and retrying
- Document issue in agent-ctx.md

**If dev server fails:**
- Check for port conflicts (3000 already in use)
- Check error messages carefully
- Try different port: `npm run dev -- -p 3001`
- Document issue in agent-ctx.md

**If build fails:**
- Check TypeScript errors (config has `ignoreBuildErrors: true`)
- Check ESLint errors (config has `ignoreDuringBuilds: true`)
- Verify all dependencies installed
- Document issue in agent-ctx.md

---

## Task 1.3: Add to Git (But Don't Commit)

### Objective
Stage files for git commit (but don't actually commit - user prefers manual commits).

### Steps

1. **Navigate to BodhiApp repo root:**
   ```bash
   cd /Users/amir36/Documents/workspace/src/github.com/BodhiSearch/BodhiApp
   ```

2. **Check git status:**
   ```bash
   git status getbodhi.app/
   ```

   Expected: Shows untracked files in `getbodhi.app/`

3. **Add files to staging:**
   ```bash
   git add getbodhi.app/
   ```

4. **Verify staging:**
   ```bash
   git status
   ```

   Expected: Shows staged changes for `getbodhi.app/` directory

5. **Check what's staged (sanity check):**
   ```bash
   git diff --cached --stat
   ```

   Expected: Lists all website files as new files

### Verification Checklist for Task 1.3

- [ ] Git shows `getbodhi.app/` as staged changes
- [ ] No unexpected files staged (check output carefully)
- [ ] `node_modules/` NOT staged (should be in .gitignore)
- [ ] `.next/` NOT staged (should be in .gitignore)
- [ ] `out/` NOT staged (should be in .gitignore)
- [ ] Source files staged (src/, public/, config files)

### Important Notes

**DO NOT RUN `git commit`**

The user (per CLAUDE.md) prefers to do git commits manually. Your job is to stage the files only.

Suggested commit message for user (document in agent-log.md):
```
Add getbodhi.app website to monorepo (Phase 1)

- Copy all source files from bodhisearch.github.io
- Verified local build works
- No deployment yet - Phase 1 setup only
```

---

## Final Steps: Update Context Files

### Update agent-log.md

Add entry in this format:

```markdown
## Agent: Phase 1 Execution - Website Setup
**Date:** 2025-10-09
**Status:** Completed

### Tasks Performed
1. Copied website files from bodhisearch.github.io to getbodhi.app/
2. Installed dependencies and tested local build
3. Verified production build succeeds
4. Staged files to git (not committed per user preference)

### Verification Results
- [x] All source files copied successfully
- [x] npm install completed without errors
- [x] Dev server runs on localhost:3000
- [x] Production build succeeds
- [x] CNAME file present with correct domain
- [x] Files staged in git

### Issues Encountered
[Document any issues here, or write "None"]

### Files Created/Modified
- Created: getbodhi.app/ (entire directory)
- Staged in git: getbodhi.app/**

### Notes for Next Agent
- Phase 1 complete and verified
- Ready for Phase 2: Configure for Subpath Testing
- User should manually commit before Phase 2
- Suggested commit message provided above
```

### Update agent-ctx.md

If you discovered anything important, add to agent-ctx.md:

```markdown
## Phase 1 Execution Insights - [Your Agent Name]
**Date:** 2025-10-09

### Discovery
[What you learned - e.g., build time, any warnings, dependency versions]

### Impact
[How this might affect future phases]

### Recommendation
[What next agent should know or watch out for]
```

---

## Success Criteria

Phase 1 is complete when:

1. ✅ Website files copied to `getbodhi.app/`
2. ✅ Local build tested and working
3. ✅ Files staged in git (not committed)
4. ✅ agent-log.md updated with your activities
5. ✅ agent-ctx.md updated with insights (if any)

---

## Important Reminders

- **Read agent-log.md and agent-ctx.md FIRST** before starting
- **Update both files** when you complete your work
- **DO NOT commit to git** - user does this manually
- **Document any issues** in agent-ctx.md to help next agent
- **Verify each step** before proceeding to the next
- **This phase is safe** - no deployment, no production impact

---

## Commands Summary

```bash
# Task 1.1: Copy files
cd /Users/amir36/Documents/workspace/src/github.com/BodhiSearch/BodhiApp
mkdir -p getbodhi.app
rsync -av --exclude='.git' --exclude='node_modules' --exclude='.next' --exclude='out' \
  /Users/amir36/Documents/workspace/src/github.com/BodhiSearch/bodhisearch.github.io/ \
  getbodhi.app/

# Task 1.2: Test build
cd getbodhi.app
npm install
npm run dev  # Test, then Ctrl+C to stop
npm run build
ls -la out/
cat out/CNAME

# Task 1.3: Stage to git
cd /Users/amir36/Documents/workspace/src/github.com/BodhiSearch/BodhiApp
git add getbodhi.app/
git status

# Update context files
# (Use your preferred text editor or tools)
```

Good luck with Phase 1! Remember to update the log and context files when done.
