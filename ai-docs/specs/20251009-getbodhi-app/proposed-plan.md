# Website Migration Plan: bodhisearch.github.io → BodhiApp Monorepo

**Date:** 2025-10-09
**Objective:** Migrate website repository into BodhiApp monorepo while maintaining `getbodhi.app` custom domain with zero downtime.
**Strategy:** Safe, incremental migration with verification at each step and clear rollback paths.

---

## Table of Contents

- [Overview](#overview)
- [PHASE 0: Pre-Migration Preparation](#phase-0-pre-migration-preparation-manual)
- [PHASE 1: Setup Website in Monorepo](#phase-1-setup-website-in-monorepo-no-deployment)
- [PHASE 2: Configure for Subpath Testing](#phase-2-configure-for-subpath-testing)
- [PHASE 3: Test Deployment to Subpath](#phase-3-test-deployment-to-subpath)
- [PHASE 4: Automated Docs Sync Integration](#phase-4-automated-docs-sync-integration)
- [PHASE 5: Release Automation Integration](#phase-5-release-automation-integration)
- [PHASE 6: DNS Cutover to Custom Domain](#phase-6-dns-cutover-to-custom-domain)
- [PHASE 7: Cleanup & Documentation](#phase-7-cleanup--documentation)
- [Rollback Procedures](#rollback-procedures)
- [Post-Migration Monitoring](#post-migration-monitoring)
- [Timeline Estimate](#timeline-estimate)
- [Final Checklist](#final-checklist)

---

## Overview

### Current State
- Website at `bodhisearch.github.io` repository
- Custom domain: `getbodhi.app`
- Docs manually copied from `crates/bodhi/src/docs/`
- Separate repository maintenance

### Target State
- Website in `BodhiApp/getbodhi.app/` directory
- Same custom domain: `getbodhi.app`
- Automated docs sync from source
- Integrated release workflows
- Single repository for code + docs

### Key Technical Considerations
- GitHub Pages custom domains work for project pages (not just user pages)
- Need temporary `basePath` for testing at `bodhisearch.github.io/BodhiApp`
- Remove `basePath` when switching to custom domain
- CNAME conflict: only one repo can claim `getbodhi.app` at a time

---

## PHASE 0: Pre-Migration Preparation (Manual)

### 0.1 Create Backup & Document Current State

**Manual Steps:**

1. **Clone website repo as backup:**
   ```bash
   cd ~/Documents/workspace/src/github.com/BodhiSearch
   cp -r bodhisearch.github.io bodhisearch.github.io.backup
   ```

2. **Document current DNS configuration:**
   - Navigate to: https://github.com/BodhiSearch/bodhisearch.github.io/settings/pages
   - Take screenshot of GitHub Pages settings
   - Verify CNAME shows: `getbodhi.app`
   - Note deployment source (branch + folder)

3. **Verify current website functionality:**
   - Visit: https://getbodhi.app
   - Test all navigation links
   - Check docs pages load correctly
   - Verify download links work
   - Document any broken links/issues to fix

**Verification Checklist:**
- [ ] Backup created and verified
- [ ] Screenshots saved
- [ ] Current site fully functional

**Risk Mitigation:**
- Backup repo allows complete restoration
- No changes made yet - zero risk

---

## PHASE 1: Setup Website in Monorepo (No Deployment)

### 1.1 Copy Website Files to Monorepo

**Manual Steps:**
```bash
cd /Users/amir36/Documents/workspace/src/github.com/BodhiSearch/BodhiApp

# Create website directory
mkdir getbodhi.app

# Copy all files EXCEPT .git, node_modules, .next, out
rsync -av \
  --exclude='.git' \
  --exclude='node_modules' \
  --exclude='.next' \
  --exclude='out' \
  /Users/amir36/Documents/workspace/src/github.com/BodhiSearch/bodhisearch.github.io/ \
  getbodhi.app/

# Verify copy
ls -la getbodhi.app/
```

**Verification Checklist:**
- [ ] All source files copied (src/, public/, package.json, etc.)
- [ ] .gitignore present
- [ ] CNAME file in public/CNAME contains `getbodhi.app`
- [ ] No node_modules or build artifacts copied

### 1.2 Test Local Build

**Manual Steps:**
```bash
cd getbodhi.app

# Install dependencies
npm install

# Test dev server
npm run dev
# Visit http://localhost:3000 - verify site works

# Stop dev server (Ctrl+C)

# Test production build
npm run build

# Verify build output
ls -la out/
cat out/CNAME  # Should show: getbodhi.app
```

**Verification Checklist:**
- [ ] Dev server runs without errors
- [ ] All pages render correctly at localhost:3000
- [ ] Production build succeeds
- [ ] `out/` directory contains all expected files
- [ ] `out/CNAME` exists with correct domain

**Rollback:** Delete `getbodhi.app/` directory if issues found.

### 1.3 Add to Git (But Don't Commit Yet)

**Manual Steps:**
```bash
cd /Users/amir36/Documents/workspace/src/github.com/BodhiSearch/BodhiApp

# Check what will be added
git status getbodhi.app/

# Add to staging (but don't commit)
git add getbodhi.app/
```

**Verification Checklist:**
- [ ] Git shows website files as staged changes
- [ ] No unexpected files in staging

---

## PHASE 2: Configure for Subpath Testing

### 2.1 Modify Next.js Config for Subpath Deployment

**Manual Step - Edit File:**

File: `getbodhi.app/next.config.mjs`

**Add basePath (temporary for testing):**
```javascript
/** @type {import('next').NextConfig} */
const nextConfig = {
  // Enable static exports
  output: 'export',
  basePath: '/BodhiApp',  // ← ADD THIS LINE FOR TESTING
  trailingSlash: true,

  // Optimize image handling
  images: {
    unoptimized: true, // Required for static export
  },

  // Optimize production builds
  compress: true,
  poweredByHeader: false,

  // Cache optimizations
  generateEtags: true,

  // Optimize static rendering
  reactStrictMode: true,
  swcMinify: true,

  // Experimental features for better performance
  experimental: {
    // Optimize CSS
    optimizePackageImports: ['lucide-react'],

    // Optimize bundle
    turbo: {
      rules: {
        // Add rules for static analysis
        '*.md': ['raw-loader'],
      },
    },
  },

  // Disable checks during production builds
  typescript: {
    ignoreBuildErrors: true,
  },
  eslint: {
    ignoreDuringBuilds: true,
  },
};

export default nextConfig;
```

**Also temporarily remove CNAME:**
```bash
cd getbodhi.app
mv public/CNAME public/CNAME.backup  # Temporarily disable custom domain
```

**Verification:**
```bash
# Rebuild with new config
npm run build

# Check paths in output
cat out/index.html | grep -o 'href="[^"]*"' | head -5
# Should see /BodhiApp/ prefix in URLs

ls out/CNAME  # Should NOT exist (we moved it)
```

**Expected:** All asset URLs prefixed with `/BodhiApp/`

### 2.2 Create GitHub Actions Workflow

**Manual Step - Create File:**

File: `.github/workflows/deploy-website.yml`

```yaml
name: Deploy Website to GitHub Pages

on:
  # Manual trigger only for now
  workflow_dispatch:

  # Later we'll add:
  # push:
  #   branches: ["main"]
  #   paths:
  #     - 'getbodhi.app/**'
  #     - 'crates/bodhi/src/docs/**'

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

**Verification:**
```bash
# Validate YAML syntax
cat .github/workflows/deploy-website.yml | grep -v '^#' | head -20

# Check workflow added to git
git add .github/workflows/deploy-website.yml
git status
```

### 2.3 Commit Test Configuration

**Manual Steps:**
```bash
cd /Users/amir36/Documents/workspace/src/github.com/BodhiSearch/BodhiApp

# Review changes
git status
git diff --cached getbodhi.app/next.config.mjs

# Stage all changes
git add getbodhi.app/
git add .github/workflows/deploy-website.yml

# Commit manually (following user preference)
# Suggested message: "Add website to monorepo (subpath test config)"
```

**What to Commit:**
- All `getbodhi.app/` files
- `.github/workflows/deploy-website.yml`

---

## PHASE 3: Test Deployment to Subpath

### 3.1 Enable GitHub Pages for BodhiApp Repo

**Manual Steps (GitHub Web UI):**

1. Navigate to: `https://github.com/BodhiSearch/BodhiApp/settings/pages`

2. Configure Pages:
   - **Source:** GitHub Actions (not branch)
   - **Custom domain:** Leave BLANK for now
   - Click Save

3. Verify permissions:
   - Settings → Actions → General
   - Workflow permissions: Read and write permissions
   - Allow GitHub Actions to create pull requests: ✓

**Verification Checklist:**
- [ ] Pages enabled with Actions source
- [ ] No custom domain configured yet
- [ ] Workflow permissions set correctly

### 3.2 Push Code and Trigger Workflow

**Manual Steps:**
```bash
# Push the commit you made
git push origin main

# Wait a moment, then trigger workflow
gh workflow run deploy-website.yml

# Monitor workflow
gh run watch

# Or view in browser:
# https://github.com/BodhiSearch/BodhiApp/actions/workflows/deploy-website.yml
```

**Verification Checklist:**
- [ ] Workflow runs successfully
- [ ] Build completes without errors
- [ ] Deploy step succeeds

### 3.3 Test Subpath Deployment

**Manual Steps:**

1. **Find deployment URL:**
   ```bash
   gh run view --web
   # Or visit: https://github.com/BodhiSearch/BodhiApp/actions
   # Click latest run → deploy job → see URL
   ```

2. **Test the site:**
   - Visit: `https://bodhisearch.github.io/BodhiApp/`
   - Test navigation - all links should work
   - Check docs pages load
   - Verify images/assets load
   - Test responsive design

3. **Verify paths:**
   ```bash
   curl -I https://bodhisearch.github.io/BodhiApp/
   # Should return 200 OK

   curl -I https://bodhisearch.github.io/BodhiApp/_next/static/...
   # Assets should load
   ```

**Verification Checklist:**
- [ ] Site accessible at bodhisearch.github.io/BodhiApp
- [ ] All pages load correctly
- [ ] Navigation works
- [ ] Assets (images, CSS, JS) load
- [ ] No console errors in browser

**If Issues Found:**
- Check browser console for 404s
- Verify basePath in next.config.mjs
- Re-run workflow if needed
- DO NOT PROCEED until working

**Rollback (if needed):**
- Disable Pages in BodhiApp repo settings
- Old site at getbodhi.app remains unaffected

---

## PHASE 4: Automated Docs Sync Integration

### 4.1 Create Docs Sync Script

**Manual Step - Create File:**

File: `scripts/sync-website-docs.sh`

```bash
#!/usr/bin/env bash
set -euo pipefail

# Sync docs from BodhiApp source to website
SOURCE_DOCS="crates/bodhi/src/docs"
DEST_DOCS="getbodhi.app/src/docs"

echo "Syncing docs from $SOURCE_DOCS to $DEST_DOCS..."

# Remove old docs (except .gitkeep if exists)
rm -rf "$DEST_DOCS"
mkdir -p "$DEST_DOCS"

# Copy all markdown files
rsync -av \
  --include='*/' \
  --include='*.md' \
  --include='_meta.json' \
  --exclude='*' \
  "$SOURCE_DOCS/" \
  "$DEST_DOCS/"

echo "✓ Docs synced successfully"
echo "Files synced:"
find "$DEST_DOCS" -type f -name "*.md" | wc -l
```

**Make executable:**
```bash
chmod +x scripts/sync-website-docs.sh
```

**Test:**
```bash
./scripts/sync-website-docs.sh

# Verify
diff -r crates/bodhi/src/docs/ getbodhi.app/src/docs/
# Should show no differences
```

**Verification Checklist:**
- [ ] Script runs without errors
- [ ] All .md files copied
- [ ] Directory structure preserved
- [ ] No extra files copied

### 4.2 Add Makefile Targets

**Manual Step - Edit File:**

File: `Makefile` (add at end, before `.PHONY`)

```makefile
# Website targets
website.sync-docs: ## Sync docs from crates/bodhi/src/docs to website
	@./scripts/sync-website-docs.sh

website.dev: ## Run website development server
	@cd getbodhi.app && npm run dev

website.build: website.sync-docs ## Build website (syncs docs first)
	@cd getbodhi.app && npm run build

website.test-local: website.build ## Test website build locally
	@echo "Website built to: getbodhi.app/out/"
	@echo "To test locally, run: cd getbodhi.app && npx serve out"
```

**Update .PHONY at end of Makefile:**
```makefile
.PHONY: test format coverage ... website.sync-docs website.dev website.build website.test-local help
```

**Test:**
```bash
make website.sync-docs
make website.build
```

**Verification Checklist:**
- [ ] `make website.sync-docs` works
- [ ] `make website.build` syncs docs then builds
- [ ] No errors

### 4.3 Integrate Docs Sync into Workflow

**Manual Step - Edit File:**

File: `.github/workflows/deploy-website.yml`

**Add step before "Build with Next.js":**
```yaml
      - name: Sync docs from source
        run: |
          chmod +x ../scripts/sync-website-docs.sh
          ../scripts/sync-website-docs.sh
        working-directory: getbodhi.app
```

**Commit these changes:**
```bash
git add scripts/sync-website-docs.sh
git add Makefile
git add .github/workflows/deploy-website.yml

# Manual commit by user
```

---

## PHASE 5: Release Automation Integration

### 5.1 Create Version Update Script

**Manual Step - Create File:**

File: `scripts/update-website-release-links.sh`

```bash
#!/usr/bin/env bash
set -euo pipefail

# Update download links in website to latest GitHub release
# Usage: ./scripts/update-website-release-links.sh [version]
#   If version not provided, fetches latest release

REPO="BodhiSearch/BodhiApp"

if [ -n "${1:-}" ]; then
  VERSION="$1"
  echo "Using specified version: $VERSION"
else
  echo "Fetching latest release version..."
  VERSION=$(gh api repos/$REPO/releases/latest --jq '.tag_name')
  echo "Latest version: $VERSION"
fi

# Update landing page or config with new version
# This depends on how your website stores download links
# Example for a config file:

CONFIG_FILE="getbodhi.app/src/lib/release-info.ts"

if [ ! -f "$CONFIG_FILE" ]; then
  echo "Creating release info file..."
  mkdir -p "$(dirname "$CONFIG_FILE")"
  cat > "$CONFIG_FILE" <<EOF
// Auto-generated release information
// Updated by scripts/update-website-release-links.sh

export const LATEST_RELEASE = {
  version: '$VERSION',
  downloadUrls: {
    macOS_intel: 'https://github.com/$REPO/releases/download/$VERSION/BodhiApp_x64.dmg',
    macOS_apple: 'https://github.com/$REPO/releases/download/$VERSION/BodhiApp_aarch64.dmg',
    windows: 'https://github.com/$REPO/releases/download/$VERSION/BodhiApp_x64_en-US.msi',
    linux_deb: 'https://github.com/$REPO/releases/download/$VERSION/bodhi_amd64.deb',
    linux_appimage: 'https://github.com/$REPO/releases/download/$VERSION/bodhi_amd64.AppImage',
  },
  releaseNotesUrl: 'https://github.com/$REPO/releases/tag/$VERSION',
};
EOF
else
  # Update existing file (simple sed replacement)
  sed -i.bak "s|version: '[^']*'|version: '$VERSION'|" "$CONFIG_FILE"
  sed -i.bak "s|releases/download/[^/]*/|releases/download/$VERSION/|g" "$CONFIG_FILE"
  rm "$CONFIG_FILE.bak"
fi

echo "✓ Updated release links to version: $VERSION"
```

**Make executable:**
```bash
chmod +x scripts/update-website-release-links.sh
```

**Manual Testing:**
```bash
# Test with current latest release
./scripts/update-website-release-links.sh

# Verify file created
cat getbodhi.app/src/lib/release-info.ts
```

**Verification Checklist:**
- [ ] Script creates/updates release info file
- [ ] Version number correct
- [ ] Download URLs valid

### 5.2 Add Makefile Target for Release

**Manual Step - Edit File:**

File: `Makefile` (add with website targets)

```makefile
website.update-release: ## Update website with latest release info
	@./scripts/update-website-release-links.sh

website.update-release-version: ## Update website with specific release version (use VERSION=vX.Y.Z)
	@./scripts/update-website-release-links.sh $(VERSION)
```

**Update .PHONY:**
```makefile
.PHONY: ... website.update-release website.update-release-version ...
```

### 5.3 Document Release Process

**Manual Step - Create File:**

File: `getbodhi.app/RELEASE_PROCESS.md`

```markdown
# Website Release Update Process

## After Creating New App Release

1. Update website release links:
   ```bash
   make website.update-release
   ```

2. Commit and push changes:
   ```bash
   git add getbodhi.app/src/lib/release-info.ts
   git commit -m "Update website to latest release"
   git push
   ```

3. Website will auto-deploy via GitHub Actions

## Manual Version Update

To update to a specific version:
```bash
make website.update-release-version VERSION=v1.2.3
```

## Verification

After deployment, verify:
- https://getbodhi.app shows correct version
- Download links work
- Release notes link correct
```

---

## PHASE 6: DNS Cutover to Custom Domain

**⚠️ CRITICAL PHASE - Follow Steps Precisely**

### 6.1 Prepare Final Configuration

**Manual Steps:**

1. **Remove basePath from Next.js config:**

File: `getbodhi.app/next.config.mjs`

```javascript
/** @type {import('next').NextConfig} */
const nextConfig = {
  output: 'export',
  // basePath: '/BodhiApp',  ← REMOVE THIS LINE
  trailingSlash: true,

  // ... rest unchanged
};

export default nextConfig;
```

2. **Restore CNAME file:**
```bash
cd getbodhi.app
mv public/CNAME.backup public/CNAME
cat public/CNAME  # Verify shows: getbodhi.app
```

3. **Test build with new config:**
```bash
npm run build

# Verify no basePath in output
cat out/index.html | grep href | head -3
# Should show / not /BodhiApp/

# Verify CNAME copied to output
cat out/CNAME  # Should show: getbodhi.app
```

**Verification Checklist:**
- [ ] basePath removed from config
- [ ] CNAME file restored
- [ ] Build succeeds
- [ ] URLs in out/index.html start with `/` not `/BodhiApp/`
- [ ] out/CNAME exists with correct domain

### 6.2 Update Workflow for Production

**Manual Step - Edit File:**

File: `.github/workflows/deploy-website.yml`

**Update trigger to enable auto-deploy:**
```yaml
on:
  push:
    branches: ["main"]
    paths:
      - 'getbodhi.app/**'
      - 'crates/bodhi/src/docs/**'
      - 'scripts/sync-website-docs.sh'
      - '.github/workflows/deploy-website.yml'

  workflow_dispatch:  # Keep manual trigger too
```

**Commit these changes:**
```bash
git add getbodhi.app/next.config.mjs
git add getbodhi.app/public/CNAME
git add .github/workflows/deploy-website.yml

# Manual commit
# Suggested message: "Configure website for custom domain deployment"
```

### 6.3 DNS Cutover - Execute Carefully

**⚠️ TIMING CRITICAL - DO THESE STEPS IN QUICK SUCCESSION:**

**Step 1: Disable old website Pages (GitHub Web UI)**
- Navigate to: `https://github.com/BodhiSearch/bodhisearch.github.io/settings/pages`
- Click "Unpublish site" or set Source to "None"
- Confirm - this releases the CNAME immediately
- **Screenshot this for documentation**

**Step 2: Push new configuration**
```bash
git push origin main
```

**Step 3: Trigger new deployment**
```bash
# Either wait for auto-trigger (if push includes workflow changes)
# Or manually trigger:
gh workflow run deploy-website.yml

# Monitor:
gh run watch
```

**Step 4: Configure custom domain in BodhiApp repo**
- Navigate to: `https://github.com/BodhiSearch/BodhiApp/settings/pages`
- Custom domain: Enter `getbodhi.app`
- Wait for DNS check (should be instant since DNS already configured)
- ✓ Enforce HTTPS (checkbox)
- Save

**Step 5: Verify deployment**
```bash
# Wait ~1-2 minutes for deployment

# Test custom domain
curl -I https://getbodhi.app
# Should return 200 OK

# Check HTTPS
curl -I https://getbodhi.app | grep -i location
# Should redirect HTTP → HTTPS
```

**Verification Checklist:**
- [ ] Old site (bodhisearch.github.io) disabled
- [ ] New deployment completed successfully
- [ ] Custom domain configured in BodhiApp repo
- [ ] https://getbodhi.app loads correctly
- [ ] All pages work
- [ ] Assets load correctly
- [ ] HTTPS enforced

**Downtime Estimate:** 1-3 minutes between disabling old site and new site going live.

### 6.4 Comprehensive Testing

**Manual Testing Checklist:**

**1. Homepage:**
- [ ] https://getbodhi.app loads
- [ ] Logo and images load
- [ ] Navigation works
- [ ] Download links present

**2. Documentation:**
- [ ] Docs index page loads
- [ ] Individual doc pages load
- [ ] Images in docs load
- [ ] Code blocks render correctly

**3. SEO/Technical:**
- [ ] robots.txt accessible
- [ ] Sitemap present (if applicable)
- [ ] No console errors in browser
- [ ] Lighthouse score reasonable (run in incognito)

**4. Cross-browser testing:**
- [ ] Chrome
- [ ] Safari
- [ ] Firefox

**5. Mobile responsive:**
- [ ] Test on mobile device or dev tools

**If Issues Found:**

**Minor issues (styling, content):**
- Fix in code
- Push to main
- Auto-deploys

**Major issues (site down):**
- See ROLLBACK PHASE 6 below

---

## PHASE 7: Cleanup & Documentation

### 7.1 Archive Old Repository

**Manual Steps:**

1. **Add migration notice to old repo:**

File: `/Users/amir36/Documents/workspace/src/github.com/BodhiSearch/bodhisearch.github.io/README.md`

```markdown
# ⚠️ REPOSITORY ARCHIVED

This repository has been merged into the main BodhiApp monorepo.

**New Location:** https://github.com/BodhiSearch/BodhiApp/tree/main/getbodhi.app

**Website:** https://getbodhi.app (still active, now deployed from monorepo)

## Why?

The website is now part of the BodhiApp monorepo for:
- Automated docs sync from source code
- Integrated release workflows
- Simplified maintenance

All future website updates should be made in the BodhiApp repository.
```

2. **Commit and push to old repo:**
   ```bash
   cd /Users/amir36/Documents/workspace/src/github.com/BodhiSearch/bodhisearch.github.io

   # Add notice
   git add README.md
   git commit -m "Archive notice - moved to BodhiApp monorepo"
   git push origin main
   ```

3. **Archive the repository (GitHub Web UI):**
   - Navigate to: `https://github.com/BodhiSearch/bodhisearch.github.io/settings`
   - Scroll to bottom → "Archive this repository"
   - Confirm archival
   - **Repository becomes read-only**

**Verification Checklist:**
- [ ] README updated with migration notice
- [ ] Changes pushed
- [ ] Repository archived
- [ ] Old repo shows "Archived" badge

### 7.2 Update BodhiApp Documentation

**Manual Step - Edit File:**

File: `CLAUDE.md` (add new section)

```markdown
## Website Development

The BodhiApp website (https://getbodhi.app) is maintained in `getbodhi.app/` directory.

### Development Commands
- `make website.dev` - Run development server
- `make website.sync-docs` - Sync docs from crates/bodhi/src/docs
- `make website.build` - Build website (syncs docs first)
- `make website.update-release` - Update download links to latest release

### Deployment
- Automatic deployment via GitHub Actions on push to main
- Workflow: `.github/workflows/deploy-website.yml`
- Deploys to: https://getbodhi.app (GitHub Pages with custom domain)

### Content Updates
- **Docs:** Edit in `crates/bodhi/src/docs/` - auto-synced to website
- **Landing page:** Edit in `getbodhi.app/src/app/`
- **Components:** `getbodhi.app/src/components/`

### Release Process
After creating a new app release, update website:
```bash
make website.update-release
git add getbodhi.app/src/lib/release-info.ts
git commit -m "Update website to latest release"
git push
```

Website auto-deploys via GitHub Actions.
```

**Commit:**
```bash
git add CLAUDE.md
# Manual commit
```

### 7.3 Create Website Development Guide

**Manual Step - Create File:**

File: `getbodhi.app/README.md`

```markdown
# BodhiApp Website

Official website for BodhiApp, deployed at https://getbodhi.app

## Tech Stack

- **Framework:** Next.js 14 (App Router)
- **Styling:** TailwindCSS + Shadcn UI
- **Content:** Markdown with MDX support
- **Deployment:** GitHub Pages with custom domain
- **Build:** Static export (`output: 'export'`)

## Development

### Prerequisites
- Node.js 20+
- npm

### Quick Start

```bash
# From project root
make website.dev

# Or from this directory
npm install
npm run dev
```

Visit: http://localhost:3000

### Building

```bash
# From project root (recommended - syncs docs)
make website.build

# Or from this directory
npm run build
```

Output: `out/` directory

### Docs Sync

Documentation is sourced from `../crates/bodhi/src/docs/` and synced automatically.

**Manual sync:**
```bash
make website.sync-docs
```

**Automatic sync:**
- Happens during `make website.build`
- Happens in GitHub Actions deployment

## Deployment

### Automatic
- Push to `main` branch
- Changes to `getbodhi.app/**` or `crates/bodhi/src/docs/**`
- GitHub Actions deploys automatically

### Manual
```bash
gh workflow run deploy-website.yml
```

### Workflow
See: `.github/workflows/deploy-website.yml`

## Content Management

### Landing Page
- Edit: `src/app/page.tsx`
- Components: `src/components/`

### Documentation
- **Source:** `../crates/bodhi/src/docs/` (edit here!)
- **Rendered:** Synced to `src/docs/`
- **DO NOT** edit files in `src/docs/` directly - they are overwritten

### Release Links
Update after new app release:
```bash
make website.update-release
```

Or manually edit: `src/lib/release-info.ts`

## Troubleshooting

### Build Errors
1. Clear cache: `rm -rf .next out`
2. Reinstall: `rm -rf node_modules && npm install`
3. Rebuild: `npm run build`

### Deployment Issues
- Check GitHub Actions: https://github.com/BodhiSearch/BodhiApp/actions/workflows/deploy-website.yml
- Verify Pages settings: https://github.com/BodhiSearch/BodhiApp/settings/pages
- Check DNS: `dig getbodhi.app`

### Docs Not Updating
```bash
# Re-sync docs
make website.sync-docs

# Verify
diff -r ../crates/bodhi/src/docs/ src/docs/
```

## Architecture

- **Static Site:** Next.js static export, no server required
- **Routing:** App Router with file-based routing
- **Styling:** Utility-first with Tailwind, components from Shadcn
- **Content:** Markdown processed with unified/remark/rehype
- **Deployment:** GitHub Pages serves `out/` directory

## License

Part of BodhiApp - see ../LICENSE
```

---

## Rollback Procedures

### Rollback from Phase 6 (DNS Cutover)

**If new site has critical issues:**

1. **Re-enable old website immediately:**
   ```bash
   # In old repo
   cd /Users/amir36/Documents/workspace/src/github.com/BodhiSearch/bodhisearch.github.io

   # Ensure CNAME is present
   cat public/CNAME  # Should show: getbodhi.app

   # Re-enable Pages (GitHub Web UI)
   # - Settings → Pages
   # - Source: GitHub Actions
   # - Save

   # Trigger deployment
   gh workflow run nextjs.yml
   ```

2. **Disable new deployment:**
   ```bash
   # Disable Pages in BodhiApp repo
   # GitHub Web UI:
   # https://github.com/BodhiSearch/BodhiApp/settings/pages
   # Set source to "None"
   ```

3. **Verify old site restored:**
   ```bash
   # Wait 2-3 minutes
   curl -I https://getbodhi.app
   ```

**Downtime:** 2-5 minutes

### Rollback from Phase 3 (Subpath Testing)

**Simple - just disable Pages:**
- GitHub Settings → Pages → Disable
- No impact on production site

### Rollback from Phases 1-2

**Even simpler:**
```bash
git reset --hard HEAD~1  # Or relevant commit
# Optionally: git push -f origin main
```

---

## Post-Migration Monitoring

### Week 1 Monitoring

**Daily checks:**
- [ ] https://getbodhi.app loads
- [ ] Check GitHub Actions runs
- [ ] Monitor any error reports

**Weekly checks:**
- [ ] Verify docs sync working
- [ ] Test release update process
- [ ] Check analytics (if configured)

### 30-Day Checklist

After 30 days of successful operation:
- [ ] Delete local backup: `rm -rf ~/Documents/workspace/src/github.com/BodhiSearch/bodhisearch.github.io.backup`
- [ ] Confirm no issues reported
- [ ] Update any external links pointing to old repo
- [ ] Consider archiving old repo permanently

---

## Timeline Estimate

| Phase | Estimated Time | Can be Paused? |
|-------|----------------|----------------|
| 0. Preparation | 30 min | Yes |
| 1. Setup in Monorepo | 1 hour | Yes |
| 2. Configure Subpath | 1 hour | Yes |
| 3. Test Deployment | 30 min | Yes |
| 4. Docs Sync | 1 hour | Yes |
| 5. Release Integration | 1-2 hours | Yes |
| 6. DNS Cutover | 15 min | **NO - do in one session** |
| 7. Cleanup | 1 hour | Yes |

**Total:** ~6-8 hours spread over multiple sessions

**Critical Section:** Phase 6 (DNS Cutover) - must complete in one 15-30 minute session

---

## Final Checklist

Before declaring migration complete:

- [ ] Old repo archived with migration notice
- [ ] New site serving at https://getbodhi.app
- [ ] Docs sync working
- [ ] GitHub Actions workflow functioning
- [ ] CLAUDE.md updated
- [ ] Website README.md created
- [ ] Makefile targets working
- [ ] All pages tested and working
- [ ] HTTPS enforced
- [ ] No console errors
- [ ] Mobile responsive
- [ ] Download links correct
- [ ] Release process documented
- [ ] Team notified of new workflow

---

## Notes

- This plan prioritizes safety and verifiability over speed
- Each phase has clear verification checkpoints
- Rollback procedures are defined for critical phases
- Phase 6 (DNS cutover) is the only phase that cannot be paused
- Backup is created but old repo remains functional until Phase 6
- Custom domain works for project pages (not just user/org pages)
- CNAME can only be claimed by one repository at a time
