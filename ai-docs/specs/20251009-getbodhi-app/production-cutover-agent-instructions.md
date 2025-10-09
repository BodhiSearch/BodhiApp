# Production Cutover Agent Instructions: Switch to getbodhi.app

## Agent Responsibilities

You are executing the **Production Cutover** phase of the website migration.

**IMPORTANT: Before starting, you MUST:**
1. Read `ai-docs/specs/20251009-getbodhi-app/agent-log.md` to see what previous agents have done
2. Read `ai-docs/specs/20251009-getbodhi-app/agent-ctx.md` for important insights and context

**After completing your work, you MUST:**
1. Update `agent-log.md` with your activities, verification results, and issues
2. Update `agent-ctx.md` with any insights or discoveries

---

## Production Cutover Overview

**Goal:** Remove testing configuration and prepare for production deployment at `getbodhi.app`

**What Changes:**
- Remove `basePath: '/BodhiApp'` (testing config)
- Restore `public/CNAME` file (production domain)
- Build and verify root path URLs

**Risk Level:** Medium - Will affect production after user performs manual cutover steps

**Important:** This agent only prepares the code. User will manually switch DNS in separate steps.

---

## Task 1: Remove basePath from Next.js Config

### Objective
Remove the testing `basePath` configuration to serve from root path.

### Steps

1. **Navigate to website directory:**
   ```bash
   cd /Users/amir36/Documents/workspace/src/github.com/BodhiSearch/BodhiApp/getbodhi.app
   ```

2. **Read current config:**
   ```bash
   cat next.config.mjs
   ```

3. **Verify basePath exists:**
   ```bash
   grep "basePath" next.config.mjs
   ```

   Expected: Should show `basePath: '/BodhiApp',`

4. **Remove the basePath line:**

   Edit `next.config.mjs` to remove this line:
   ```javascript
   basePath: '/BodhiApp',
   ```

   The config should now look like:
   ```javascript
   const nextConfig = {
     output: 'export',
     // basePath line removed
     trailingSlash: true,
     // ... rest of config
   };
   ```

5. **Verify removal:**
   ```bash
   grep "basePath" next.config.mjs
   ```

   Expected: No output (line removed) or commented line only

6. **Check syntax:**
   ```bash
   node -c next.config.mjs 2>&1 | grep -i error
   ```

   Expected: No errors

### Verification Checklist for Task 1

- [ ] `next.config.mjs` edited successfully
- [ ] `basePath: '/BodhiApp',` line removed
- [ ] No syntax errors in file
- [ ] File still valid JavaScript

---

## Task 2: Restore CNAME File

### Objective
Move CNAME back from backup to enable custom domain deployment.

### Steps

1. **Verify backup exists:**
   ```bash
   ls -la public/CNAME.backup
   cat public/CNAME.backup
   ```

   Expected: File exists with content `getbodhi.app`

2. **Verify original CNAME doesn't exist:**
   ```bash
   ls public/CNAME 2>&1
   ```

   Expected: "No such file or directory"

3. **Restore CNAME:**
   ```bash
   mv public/CNAME.backup public/CNAME
   ```

4. **Verify restoration:**
   ```bash
   ls -la public/CNAME
   cat public/CNAME
   ```

   Expected: File exists with content `getbodhi.app`

5. **Verify backup is gone:**
   ```bash
   ls public/CNAME.backup 2>&1
   ```

   Expected: "No such file or directory" (moved, not copied)

### Verification Checklist for Task 2

- [ ] `public/CNAME` file restored
- [ ] Contains correct content: `getbodhi.app`
- [ ] `public/CNAME.backup` no longer exists (moved successfully)

---

## Task 3: Test Build with Production Configuration

### Objective
Verify the website builds correctly for production (root path, no basePath).

### Steps

1. **Clean previous build:**
   ```bash
   rm -rf out .next
   ```

2. **Run production build:**
   ```bash
   npm run build
   ```

   Expected: Build completes successfully

3. **Verify build output exists:**
   ```bash
   ls -la out/
   ```

   Expected: `out/` directory with `index.html`, `_next/`, etc.

4. **Verify CNAME in build output:**
   ```bash
   cat out/CNAME
   ```

   Expected: File exists with content `getbodhi.app`

5. **Verify URLs have NO basePath (root relative):**
   ```bash
   cat out/index.html | grep -o 'href="[^"]*"' | head -10
   ```

   Expected: URLs should start with `/` (not `/BodhiApp/`)

   Examples:
   ```
   href="/_next/static/..."
   href="/docs/"
   href="/favicon.ico"
   ```

   **NOT:**
   ```
   href="/BodhiApp/_next/static/..."
   href="/BodhiApp/docs/"
   ```

6. **Verify asset paths:**
   ```bash
   cat out/index.html | grep -o 'src="[^"]*"' | head -5
   ```

   Expected: Asset paths should also start with `/` (not `/BodhiApp/`)

### Verification Checklist for Task 3

- [ ] `npm run build` completed successfully
- [ ] `out/` directory created
- [ ] `out/CNAME` exists with `getbodhi.app`
- [ ] URLs in HTML start with `/` (not `/BodhiApp/`)
- [ ] Asset paths start with `/` (not `/BodhiApp/`)
- [ ] No build errors or warnings (ignore deprecation warnings)

### Troubleshooting

**If build fails:**
- Check next.config.mjs syntax
- Ensure basePath line was cleanly removed
- Check for missing commas or brackets
- Try: `rm -rf .next node_modules && npm install && npm run build`

**If URLs still have /BodhiApp/:**
- Verify basePath line is completely removed (not just commented)
- Check there's no cached build: `rm -rf .next out`
- Rebuild: `npm run build`

---

## Task 4: Stage Changes to Git

### Objective
Stage all production cutover changes for user's manual commit.

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
   - Renamed: `getbodhi.app/public/CNAME.backup` → `getbodhi.app/public/CNAME`

3. **Stage the changes:**
   ```bash
   git add getbodhi.app/next.config.mjs
   git add getbodhi.app/public/CNAME
   ```

4. **Verify staging:**
   ```bash
   git status
   git diff --cached --stat
   ```

   Expected: Shows 2 files staged (modified + renamed)

5. **Check diff for next.config.mjs:**
   ```bash
   git diff --cached getbodhi.app/next.config.mjs
   ```

   Expected: Should show `-  basePath: '/BodhiApp',` line removed

### Verification Checklist for Task 4

- [ ] `getbodhi.app/next.config.mjs` staged (modified - basePath removed)
- [ ] `getbodhi.app/public/CNAME` staged (renamed from CNAME.backup)
- [ ] Git diff shows basePath line removed
- [ ] No unexpected files staged

### Important Note

**DO NOT RUN `git commit`**

The user will commit manually. Your job is to stage the files only.

**Suggested commit message for user (document in agent-log.md):**
```
Configure website for production deployment

- Remove basePath: '/BodhiApp' (testing config no longer needed)
- Restore CNAME for getbodhi.app domain
- URLs now serve from root path
- Ready for DNS cutover to BodhiApp repo
```

---

## Final Steps: Update Context Files

### Update agent-log.md

Add entry in this format:

```markdown
## Agent: Production Cutover Preparation
**Date:** 2025-10-09
**Status:** Completed

### Tasks Performed
1. Removed basePath: '/BodhiApp' from next.config.mjs
2. Restored public/CNAME from public/CNAME.backup
3. Tested production build - verified root path URLs
4. Staged changes to git (not committed per user preference)

### Verification Results
- [x] basePath removed from next.config.mjs
- [x] CNAME file restored to public/CNAME
- [x] CNAME.backup no longer exists (properly moved)
- [x] Build succeeds with production configuration
- [x] URLs in HTML start with / (root relative, no /BodhiApp/)
- [x] Asset paths also root relative
- [x] out/CNAME exists with correct domain
- [x] Changes staged in git

### Issues Encountered
[Document any issues here, or write "None"]

### Build Results
- Build time: [X seconds]
- Pages generated: [X pages]
- URL examples from out/index.html:
  - href="/_next/static/..."
  - href="/docs/"
  - href="/favicon.ico"
- CNAME in output: getbodhi.app

### Files Modified
- Modified: getbodhi.app/next.config.mjs (removed basePath)
- Renamed: getbodhi.app/public/CNAME.backup → getbodhi.app/public/CNAME

### Notes for User - Manual DNS Cutover Steps Required
**The code is ready. Now user must perform manual cutover:**

1. **Commit these changes**
2. **Disable old site:** bodhisearch.github.io settings/pages → Unpublish
3. **Push changes:** git push origin main
4. **Deploy:** gh workflow run deploy-website.yml
5. **Configure domain:** BodhiApp settings/pages → Custom domain: getbodhi.app
6. **Verify:** https://getbodhi.app

**Expected downtime:** 1-3 minutes between step 2 and step 6

**Critical:** Steps 2-6 should be done in quick succession to minimize downtime
```

### Update agent-ctx.md

If you discovered anything important, add to agent-ctx.md:

```markdown
## Production Cutover Preparation Insights
**Date:** 2025-10-09
**Agent:** Production Cutover Preparation

### Discovery
[What you learned about removing basePath, CNAME restoration, etc.]

### Impact
[How this affects the production deployment]

### Recommendation
[What user should know about the manual cutover process]
```

---

## Success Criteria

Production cutover preparation is complete when:

1. ✅ basePath removed from next.config.mjs
2. ✅ CNAME restored to public/CNAME
3. ✅ Build tested with production config
4. ✅ URLs confirmed as root relative (no /BodhiApp/)
5. ✅ CNAME in build output
6. ✅ Changes staged in git (not committed)
7. ✅ agent-log.md updated with preparation details
8. ✅ agent-ctx.md updated with insights (if any)

---

## Important Reminders

- **Read agent-log.md and agent-ctx.md FIRST** before starting
- **Update both files** when you complete your work
- **DO NOT commit to git** - user does this manually
- **This prepares for cutover** - actual DNS switch is manual
- **Test build thoroughly** - production deployment depends on it
- **Verify URLs have no /BodhiApp/** - critical for production

---

## Commands Summary

```bash
# Task 1: Remove basePath
cd /Users/amir36/Documents/workspace/src/github.com/BodhiSearch/BodhiApp/getbodhi.app
# Edit next.config.mjs to remove: basePath: '/BodhiApp',

# Task 2: Restore CNAME
mv public/CNAME.backup public/CNAME

# Task 3: Test build
rm -rf out .next
npm run build
cat out/CNAME
cat out/index.html | grep -o 'href="[^"]*"' | head -10

# Task 4: Stage changes
cd /Users/amir36/Documents/workspace/src/github.com/BodhiSearch/BodhiApp
git add getbodhi.app/next.config.mjs
git add getbodhi.app/public/CNAME
git status

# Update context files
# (Use your preferred text editor or tools)
```

Good luck with the production cutover preparation! Remember to update the log and context files when done.
