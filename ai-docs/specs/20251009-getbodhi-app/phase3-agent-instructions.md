# Phase 3 Agent Instructions: Test Deployment to Subpath

## Agent Responsibilities

You are executing **Phase 3** of the website migration from `bodhisearch.github.io` to the BodhiApp monorepo.

**IMPORTANT: Before starting, you MUST:**
1. Read `ai-docs/specs/20251009-getbodhi-app/agent-log.md` to see what previous agents have done
2. Read `ai-docs/specs/20251009-getbodhi-app/agent-ctx.md` for important insights and context

**After completing your work, you MUST:**
1. Update `agent-log.md` with your activities, verification results, and issues
2. Update `agent-ctx.md` with any insights or discoveries for future agents

---

## Phase 3 Overview

**Goal:** Deploy website to `bodhisearch.github.io/BodhiApp` and verify it works

**Why:** Test the deployment pipeline and basePath configuration before switching to production custom domain.

**Risk Level:** Low - Testing only, no impact on production site at `getbodhi.app`

**Rollback:** Simply disable GitHub Pages in BodhiApp repo settings

---

## Prerequisites Check

Before starting, verify Phase 2 is complete:

```bash
cd /Users/amir36/Documents/workspace/src/github.com/BodhiSearch/BodhiApp

# Check basePath is configured
grep "basePath" getbodhi.app/next.config.mjs
# Should show: basePath: '/BodhiApp',

# Check CNAME is backed up (not in public/)
ls getbodhi.app/public/CNAME 2>/dev/null && echo "ERROR: CNAME still in public/" || echo "OK: CNAME moved to backup"

# Check workflow exists
ls .github/workflows/deploy-website.yml
```

If any of these checks fail, **DO NOT PROCEED**. Report the issue and wait for Phase 2 to be completed properly.

---

## Task 3.1: Enable GitHub Pages for BodhiApp Repo

### Objective
Configure GitHub Pages to deploy from GitHub Actions.

### Steps

**IMPORTANT: These are manual steps in GitHub Web UI. You cannot automate these with gh CLI.**

1. **Navigate to Pages settings:**
   - Open in browser: `https://github.com/BodhiSearch/BodhiApp/settings/pages`
   - If you don't have permissions, report this and wait for user to do it

2. **Configure Pages source:**
   - Look for "Build and deployment" section
   - Under "Source" dropdown: Select **"GitHub Actions"**
   - **DO NOT select "Deploy from a branch"**
   - Click **Save** if button appears

3. **Verify custom domain field:**
   - In "Custom domain" field: **Leave it BLANK**
   - **DO NOT** enter `getbodhi.app` yet (that's Phase 6)
   - If there's any domain there, **remove it**

4. **Check Actions permissions:**
   - Navigate to: `https://github.com/BodhiSearch/BodhiApp/settings/actions`
   - Under "Workflow permissions": Should be **"Read and write permissions"**
   - Under "Allow GitHub Actions to create pull requests": Should be **checked**
   - If not set correctly, this needs to be fixed before deployment

### Verification Checklist for Task 3.1

- [ ] Pages settings opened successfully
- [ ] Source set to "GitHub Actions" (not "Deploy from a branch")
- [ ] Custom domain field is BLANK (no domain entered)
- [ ] Actions have "Read and write permissions"
- [ ] Actions can create pull requests (checkbox checked)

### Important Notes

**You cannot fully automate this task** - GitHub requires Web UI interaction for initial Pages setup. However, you can:
1. Provide the URLs to the user
2. Explain what needs to be done
3. Verify the settings afterwards using `gh api`

**Verification command:**
```bash
gh api repos/BodhiSearch/BodhiApp/pages
```

Expected output should include:
```json
{
  "build_type": "workflow",
  "source": {...},
  "status": "built" (or "queued")
}
```

If `build_type` is not "workflow", Pages is not configured correctly.

---

## Task 3.2: Push Code and Trigger Workflow

### Objective
Push Phase 2 changes and manually trigger the deployment workflow.

### Steps

1. **Verify you're on main branch and up to date:**
   ```bash
   cd /Users/amir36/Documents/workspace/src/github.com/BodhiSearch/BodhiApp
   git branch --show-current
   # Should show: main

   git status
   # Should show: nothing to commit, working tree clean
   ```

2. **Check remote status:**
   ```bash
   git fetch origin
   git status
   ```

   Expected: Should show branch is behind if Phase 2 commit hasn't been pushed

3. **Push the changes:**
   ```bash
   git push origin main
   ```

4. **Wait a moment for GitHub to register the push:**
   ```bash
   sleep 3
   ```

5. **Trigger the workflow manually:**
   ```bash
   gh workflow run deploy-website.yml
   ```

   Expected output:
   ```
   ✓ Created workflow_dispatch event for deploy-website.yml at main
   ```

6. **Get the run ID and monitor:**
   ```bash
   # List recent runs
   gh run list --workflow=deploy-website.yml --limit 3

   # Watch the latest run
   gh run watch
   ```

   **Or view in browser:**
   ```bash
   gh run view --web
   ```

### Verification Checklist for Task 3.2

- [ ] On main branch
- [ ] Working tree clean (all committed)
- [ ] Changes pushed to origin/main
- [ ] Workflow triggered successfully
- [ ] Workflow run shows in `gh run list`
- [ ] Can monitor workflow with `gh run watch` or browser

### Troubleshooting

**If `gh workflow run` fails:**
- Error "workflow not found": Verify file exists at `.github/workflows/deploy-website.yml`
- Error "unauthorized": Check your gh CLI authentication (`gh auth status`)
- Error "branch not found": Ensure you pushed to main first

**If push fails:**
- Pull latest changes first: `git pull origin main`
- Check for conflicts and resolve
- Push again

---

## Task 3.3: Monitor Deployment

### Objective
Watch the deployment process and ensure it completes successfully.

### Steps

1. **Monitor the workflow run:**
   ```bash
   gh run watch
   ```

   This will show real-time status updates. Watch for:
   - ✓ Build job completes
   - ✓ Deploy job completes

2. **If monitoring with browser:**
   ```bash
   gh run view --web
   ```

   Look for:
   - Green checkmarks for each step
   - "Workflow run completed" message
   - No red X marks (failures)

3. **Check for errors in build logs:**
   ```bash
   # View logs after run completes
   gh run view --log
   ```

   Look for:
   - npm ci completed successfully
   - npm run build completed successfully
   - Pages deployment successful

4. **Get the deployment URL:**

   The workflow should output the Pages URL. You can find it:

   ```bash
   gh run view --log | grep -i "deployed"
   # Or look for: github-pages.url
   ```

   Expected URL format: `https://bodhisearch.github.io/BodhiApp`

### Verification Checklist for Task 3.3

- [ ] Build job completed successfully
- [ ] Deploy job completed successfully
- [ ] No error messages in logs
- [ ] Deployment URL obtained

### Troubleshooting

**If build fails:**
- Check error messages in logs: `gh run view --log`
- Common issues:
  - npm ci fails: package-lock.json out of sync
  - Build fails: basePath not configured correctly
  - Deploy fails: Pages not enabled or permissions issue

**If deployment fails:**
- Verify Pages is enabled (Task 3.1)
- Check Actions permissions
- Retry: `gh workflow run deploy-website.yml`

---

## Task 3.4: Test Deployed Site

### Objective
Verify the deployed website works correctly at the subpath URL.

### Steps

1. **Wait for deployment to fully propagate:**
   ```bash
   sleep 30  # Give Pages time to serve the new content
   ```

2. **Test homepage with curl:**
   ```bash
   curl -I https://bodhisearch.github.io/BodhiApp/
   ```

   Expected response:
   ```
   HTTP/2 200
   content-type: text/html
   ```

   **Not expected:**
   - `HTTP/2 404` - site not deployed or wrong path
   - `HTTP/2 301` - redirect (might indicate basePath issue)

3. **Test an asset path:**
   ```bash
   curl -I https://bodhisearch.github.io/BodhiApp/_next/static/css/
   ```

   Should return 200 or list directory

4. **Test a docs page:**
   ```bash
   curl -I https://bodhisearch.github.io/BodhiApp/docs/
   ```

   Expected: 200 OK

5. **Use Playwright to visually verify (if available):**

   If you have browser automation capabilities, navigate to:
   - Homepage: `https://bodhisearch.github.io/BodhiApp/`
   - Test that it renders without errors
   - Check browser console for 404s or errors
   - Verify navigation links work

6. **Check for common issues:**
   ```bash
   # Download homepage and check for /BodhiApp/ paths
   curl -s https://bodhisearch.github.io/BodhiApp/ | grep -o 'href="[^"]*"' | head -10
   ```

   Expected: All URLs should have /BodhiApp/ prefix

7. **Report URL to user for manual testing:**

   The user should manually test:
   - Homepage loads and looks correct
   - Navigation works
   - Docs pages load
   - Images load
   - No console errors in browser

### Verification Checklist for Task 3.4

- [ ] Homepage returns 200 OK
- [ ] Assets return 200 OK or proper status
- [ ] Docs pages return 200 OK
- [ ] URLs in HTML have /BodhiApp/ prefix
- [ ] No 404 errors from curl tests
- [ ] Site visually renders correctly (if browser tested)
- [ ] User has URL for manual testing

### Troubleshooting

**If homepage returns 404:**
- Verify Pages is serving from Actions (not branch)
- Check deployment logs for success
- Wait longer (Pages can take 1-2 minutes to update)
- Verify basePath is exactly '/BodhiApp' (case-sensitive)

**If assets 404:**
- Check if URLs in HTML have correct prefix
- Verify build completed successfully
- Check browser console for specific 404 paths

**If page loads but looks broken:**
- Likely CSS or JS not loading
- Check browser console for 404s
- Verify basePath configuration

---

## Task 3.5: Document Results

### Objective
Document the deployment test results for Phase 4 planning.

### Steps

1. **Gather deployment information:**
   ```bash
   # Get run ID
   RUN_ID=$(gh run list --workflow=deploy-website.yml --limit 1 --json databaseId --jq '.[0].databaseId')

   # Get run URL
   gh run view $RUN_ID --web --json url --jq '.url'

   # Get deployment status
   gh run view $RUN_ID --json conclusion --jq '.conclusion'
   ```

2. **Test multiple pages and document results:**

   Create a simple test report in your final message documenting:
   - Homepage: ✓ / ✗
   - Docs index: ✓ / ✗
   - Sample doc page: ✓ / ✗
   - Assets loading: ✓ / ✗
   - Navigation: ✓ / ✗
   - Console errors: Yes / No

3. **Take note of any issues for agent-ctx.md:**
   - Slow deployment time?
   - Specific pages not working?
   - Asset loading issues?
   - Browser compatibility issues?

---

## Final Steps: Update Context Files

### Update agent-log.md

Add entry in this format:

```markdown
## Agent: Phase 3 Execution - Test Deployment
**Date:** 2025-10-09
**Status:** Completed

### Tasks Performed
1. Enabled GitHub Pages for BodhiApp repo (Source: GitHub Actions)
2. Pushed Phase 2 changes to origin/main
3. Triggered deploy-website.yml workflow manually
4. Monitored deployment (Build + Deploy jobs)
5. Tested deployed site at bodhisearch.github.io/BodhiApp
6. Verified pages, assets, and navigation work correctly

### Verification Results
- [x] GitHub Pages enabled with Actions source
- [x] Custom domain field left blank (as planned)
- [x] Actions permissions set correctly
- [x] Workflow triggered successfully
- [x] Build job completed without errors
- [x] Deploy job completed successfully
- [x] Site accessible at bodhisearch.github.io/BodhiApp
- [x] Homepage returns 200 OK
- [x] Assets load correctly with /BodhiApp/ prefix
- [x] Docs pages load correctly
- [x] Navigation works (all links include basePath)

### Issues Encountered
[Document any issues here, or write "None"]

### Deployment Details
- Workflow run ID: [run-id]
- Build time: [X minutes]
- Deploy time: [X minutes]
- Total time: [X minutes]
- Deployment URL: https://bodhisearch.github.io/BodhiApp/
- Test results: [detailed test results]

### Files Created/Modified
- None (Phase 3 is deployment only, no code changes)

### Notes for Next Agent (Phase 4)
- Phase 3 complete - deployment verified
- Ready for Phase 4: Automated Docs Sync Integration
- Deployment pipeline works correctly
- basePath configuration confirmed working
- Consider any performance issues observed
- [Any specific recommendations based on deployment experience]
```

### Update agent-ctx.md

If you discovered anything important, add to agent-ctx.md:

```markdown
## Phase 3 Execution - Deployment Testing Insights
**Date:** 2025-10-09
**Agent:** Phase 3 Execution

### Discovery
[What you learned about GitHub Pages deployment, timing, performance, etc.]

### Impact
[How this affects Phase 4 and Phase 6]

### Recommendation
[What Phase 4 agent should know about the deployment pipeline]
```

---

## Success Criteria

Phase 3 is complete when:

1. ✅ GitHub Pages enabled for BodhiApp repo
2. ✅ Workflow triggered and completed successfully
3. ✅ Site deployed to bodhisearch.github.io/BodhiApp
4. ✅ Homepage loads and returns 200 OK
5. ✅ Assets load with correct /BodhiApp/ prefix
6. ✅ Docs pages load correctly
7. ✅ Navigation works between pages
8. ✅ agent-log.md updated with deployment details
9. ✅ agent-ctx.md updated with insights (if any)

---

## Important Reminders

- **Read agent-log.md and agent-ctx.md FIRST** before starting
- **Update both files** when you complete your work
- **Task 3.1 requires Web UI** - provide guidance to user if needed
- **Document deployment URL** for user testing
- **Note any performance or timing issues** in context files
- **This phase is safe** - no impact on production site at getbodhi.app
- **Rollback is simple** - just disable Pages in repo settings

---

## User Manual Testing Required

After automated tests pass, the user should manually verify:

**Manual Testing Checklist:**
- [ ] Visit https://bodhisearch.github.io/BodhiApp/
- [ ] Homepage renders correctly
- [ ] Logo and images load
- [ ] Navigation menu works
- [ ] Click through to Docs section
- [ ] Navigate between doc pages
- [ ] Verify no 404 errors in browser console
- [ ] Test on mobile device or responsive view
- [ ] Verify external links (GitHub, Discord) work

**Provide this URL to user:** `https://bodhisearch.github.io/BodhiApp/`

---

Good luck with Phase 3! Remember to update the log and context files when done.
