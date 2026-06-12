# Docker Release Integration - Testing Guide

**Version:** 1.0
**Date:** 2025-10-10
**Status:** Completed

## Overview

This document provides comprehensive testing procedures for the Docker release integration feature, including validation steps for each phase, end-to-end testing scenarios, and regression testing checklists.

## Testing Philosophy

### Principles

1. **Verify Each Phase Independently:** Ensure backend, frontend, and documentation work in isolation
2. **Test End-to-End Flow:** Validate complete workflow from release to website display
3. **Check Edge Cases:** Test with missing data, unknown variants, network failures
4. **Regression Prevention:** Ensure existing functionality (desktop platforms) remains intact
5. **User Experience Validation:** Test actual user workflows and interactions

### Testing Levels

- **Unit Testing:** Individual functions and components
- **Integration Testing:** Backend script output → Frontend rendering
- **End-to-End Testing:** GitHub release → Website display
- **User Acceptance Testing:** Real-world usage scenarios
- **Regression Testing:** Existing features still work

## Phase 1: Backend Infrastructure Testing

### Prerequisites

```bash
cd /Users/amir36/Documents/workspace/src/github.com/BodhiSearch/BodhiApp/getbodhi.app
```

### Test 1.1: Script Execution (Dry-Run)

**Objective:** Verify script runs without errors

**Steps:**
```bash
npm run update_releases:check
```

**Expected Output:**
```
=== Checking latest releases (dry-run) ===

Fetching releases from GitHub...
Processing page 1 (100 releases)...
✓ Found Docker release: docker/v0.0.2
  Variants discovered: cpu, cuda, rocm, vulkan
✓ Found macos: app/v0.0.31 -> Bodhi.App_0.1.0_aarch64.dmg
✓ Found windows: app/v0.0.31 -> Bodhi.App_0.1.0_x64_en-US.msi
✓ Found linux: app/v0.0.31 -> Bodhi.App-0.1.0-1.x86_64.rpm

=== Dry-run mode - would write to .env.release_urls: ===
[Environment variables content]

=== Dry-run mode - would write to public/releases.json: ===
[JSON content]
```

**Validation:**
- ✅ Script completes without errors
- ✅ Docker release found
- ✅ 4 variants discovered (cpu, cuda, rocm, vulkan)
- ✅ Desktop platforms found
- ✅ JSON structure valid

### Test 1.2: File Generation

**Objective:** Verify files are generated correctly

**Steps:**
```bash
npm run update_releases
```

**Expected Output:**
```
=== Updating release URLs ===

Fetching releases from GitHub...
✓ Found Docker release: docker/v0.0.2
  Variants discovered: cpu, cuda, rocm, vulkan

✓ Updated .env.release_urls
  File: .env.release_urls
  Variables updated: 8

✓ Updated public/releases.json
  File: public/releases.json
  Desktop platforms: macos, windows, linux
  Docker variants: cpu, cuda, rocm, vulkan

Next steps:
  1. Review changes: git diff .env.release_urls public/releases.json
  2. Test build: npm run build
  3. Commit changes: git add .env.release_urls public/releases.json && git commit
```

**Validation:**
- ✅ `.env.release_urls` created/updated
- ✅ `public/releases.json` created/updated
- ✅ No script errors
- ✅ Correct number of variables
- ✅ Correct number of variants

### Test 1.3: Environment Variables

**Objective:** Verify Docker environment variables

**Steps:**
```bash
cat .env.release_urls | grep DOCKER
```

**Expected Output:**
```
NEXT_PUBLIC_DOCKER_VERSION=0.0.2
NEXT_PUBLIC_DOCKER_TAG=docker/v0.0.2
NEXT_PUBLIC_DOCKER_REGISTRY=ghcr.io/bodhisearch/bodhiapp
```

**Validation:**
- ✅ All 3 Docker variables present
- ✅ Version format correct (X.Y.Z)
- ✅ Tag format correct (docker/vX.Y.Z)
- ✅ Registry URL correct

### Test 1.4: JSON Structure

**Objective:** Verify releases.json structure

**Steps:**
```bash
cat public/releases.json | jq '.'
cat public/releases.json | jq '.docker'
cat public/releases.json | jq '.docker.variants | keys'
```

**Expected Output:**
```json
{
  "desktop": { ... },
  "docker": {
    "version": "0.0.2",
    "tag": "docker/v0.0.2",
    "released_at": "2025-10-07T12:58:40Z",
    "registry": "ghcr.io/bodhisearch/bodhiapp",
    "variants": {
      "cpu": { ... },
      "cuda": { ... },
      "rocm": { ... },
      "vulkan": { ... }
    }
  }
}

["cpu", "cuda", "rocm", "vulkan"]
```

**Validation:**
- ✅ Top-level keys: `desktop` and `docker`
- ✅ Docker version field present
- ✅ Docker tag field present
- ✅ Released_at in ISO-8601 format
- ✅ Registry URL correct
- ✅ 4 variants present
- ✅ All variant keys lowercase

### Test 1.5: Variant Data Completeness

**Objective:** Verify each variant has required fields

**Steps:**
```bash
cat public/releases.json | jq '.docker.variants.cpu'
cat public/releases.json | jq '.docker.variants.cuda'
cat public/releases.json | jq '.docker.variants.rocm'
cat public/releases.json | jq '.docker.variants.vulkan'
```

**Expected Fields (each variant):**
```json
{
  "image_tag": "0.0.2-cpu",
  "latest_tag": "latest-cpu",
  "platforms": ["linux/amd64", "linux/arm64"],
  "pull_command": "docker pull ghcr.io/bodhisearch/bodhiapp:0.0.2-cpu",
  "description": "Multi-platform: AMD64 + ARM64"
}
```

**Validation per Variant:**
- ✅ `image_tag` present and correct format
- ✅ `latest_tag` present and correct format
- ✅ `platforms` array with at least one entry
- ✅ `pull_command` complete and correct
- ✅ `gpu_type` present for GPU variants (cuda, rocm, vulkan)
- ✅ `description` present and meaningful

### Test 1.6: CPU Variant Multi-Platform

**Objective:** Verify CPU variant supports multiple platforms

**Steps:**
```bash
cat public/releases.json | jq '.docker.variants.cpu.platforms'
```

**Expected Output:**
```json
["linux/amd64", "linux/arm64"]
```

**Validation:**
- ✅ Both AMD64 and ARM64 present
- ✅ Platform format correct (linux/arch)

### Test 1.7: GPU Variants Metadata

**Objective:** Verify GPU variants have gpu_type field

**Steps:**
```bash
cat public/releases.json | jq '.docker.variants.cuda.gpu_type'
cat public/releases.json | jq '.docker.variants.rocm.gpu_type'
cat public/releases.json | jq '.docker.variants.vulkan.gpu_type'
```

**Expected Output:**
```
"NVIDIA"
"AMD"
"Cross-vendor"
```

**Validation:**
- ✅ CUDA has gpu_type: "NVIDIA"
- ✅ ROCm has gpu_type: "AMD"
- ✅ Vulkan has gpu_type: "Cross-vendor"

### Test 1.8: Backward Compatibility

**Objective:** Verify desktop platforms still work

**Steps:**
```bash
cat .env.release_urls | grep DOWNLOAD
cat public/releases.json | jq '.desktop'
```

**Expected Output:**
```
NEXT_PUBLIC_DOWNLOAD_URL_MACOS=https://github.com/...
NEXT_PUBLIC_DOWNLOAD_URL_WINDOWS=https://github.com/...
NEXT_PUBLIC_DOWNLOAD_URL_LINUX=https://github.com/...

{
  "version": "0.0.31",
  "tag": "app/v0.0.31",
  "released_at": "2025-10-07T11:16:55Z",
  "platforms": { ... }
}
```

**Validation:**
- ✅ Desktop URLs present
- ✅ Desktop section in releases.json intact
- ✅ All 3 platforms present
- ✅ Structure unchanged

## Phase 2: Frontend Display Testing

### Prerequisites

```bash
cd /Users/amir36/Documents/workspace/src/github.com/BodhiSearch/BodhiApp/getbodhi.app
npm run build  # Ensure clean build
npm run dev    # Start dev server
```

### Test 2.1: Component Renders

**Objective:** Verify DockerSection appears on homepage

**Steps:**
1. Navigate to `http://localhost:3000`
2. Scroll to Docker section
3. Inspect component structure

**Expected Behavior:**
- ✅ "Deploy with Docker" heading visible
- ✅ Section appears after DownloadSection
- ✅ No JavaScript errors in console
- ✅ No missing images or broken layouts

### Test 2.2: Variant Cards Display

**Objective:** Verify all variant cards render

**Steps:**
1. Count variant cards
2. Check each card displays
3. Verify grid layout

**Expected Behavior:**
- ✅ 4 variant cards displayed
- ✅ Cards arranged in grid (3 columns desktop, 2 tablet, 1 mobile)
- ✅ Each card shows variant name
- ✅ Each card shows description
- ✅ Each card shows platforms
- ✅ Each card shows pull command
- ✅ Each card has copy button

### Test 2.3: Card Content Validation

**Objective:** Verify each card shows correct information

**Per Variant:**

**CPU Card:**
- ✅ Display Name: "CPU"
- ✅ Icon: Blue Cpu icon
- ✅ Badge: "Recommended" (green with check icon)
- ✅ Description: "Multi-platform (AMD64 + ARM64)"
- ✅ Platforms: "linux/amd64, linux/arm64"
- ✅ Pull command: `docker pull ghcr.io/bodhisearch/bodhiapp:0.0.2-cpu`
- ✅ No GPU badge

**CUDA Card:**
- ✅ Display Name: "CUDA"
- ✅ Icon: Green Zap icon
- ✅ Badge: "NVIDIA GPU" (violet)
- ✅ Description: "NVIDIA GPU acceleration (8-12x faster)"
- ✅ Platforms: "linux/amd64"
- ✅ Pull command: `docker pull ghcr.io/bodhisearch/bodhiapp:0.0.2-cuda`
- ✅ No "Recommended" badge

**ROCm Card:**
- ✅ Display Name: "ROCm"
- ✅ Icon: Red Zap icon
- ✅ Badge: "AMD GPU" (violet)
- ✅ Description: "AMD GPU acceleration"
- ✅ Platforms: "linux/amd64"
- ✅ Pull command: `docker pull ghcr.io/bodhisearch/bodhiapp:0.0.2-rocm`

**Vulkan Card:**
- ✅ Display Name: "Vulkan"
- ✅ Icon: Purple Zap icon
- ✅ Badge: "Cross-vendor GPU" (violet)
- ✅ Description: "Cross-vendor GPU acceleration"
- ✅ Platforms: "linux/amd64"
- ✅ Pull command: `docker pull ghcr.io/bodhisearch/bodhiapp:0.0.2-vulkan`

### Test 2.4: Copy-to-Clipboard Functionality

**Objective:** Verify copy button works

**Steps:**
1. Click "Copy Command" on CPU card
2. Observe button state change
3. Paste into text editor
4. Wait 2 seconds
5. Observe button revert

**Expected Behavior:**
- ✅ Button changes to "Copied!" with check icon
- ✅ Clipboard contains correct docker pull command
- ✅ Button reverts to "Copy Command" after 2s
- ✅ Works for all variants

**Test on Multiple Variants:**
```
CPU: docker pull ghcr.io/bodhisearch/bodhiapp:0.0.2-cpu
CUDA: docker pull ghcr.io/bodhisearch/bodhiapp:0.0.2-cuda
ROCm: docker pull ghcr.io/bodhisearch/bodhiapp:0.0.2-rocm
Vulkan: docker pull ghcr.io/bodhisearch/bodhiapp:0.0.2-vulkan
```

### Test 2.5: Documentation Link

**Objective:** Verify link to Docker docs works

**Steps:**
1. Scroll to bottom of Docker section
2. Click "Docker deployment documentation" link
3. Verify navigation

**Expected Behavior:**
- ✅ Blue information box visible
- ✅ Link text: "Docker deployment documentation"
- ✅ Clicking navigates to `/docs/deployment/docker`
- ✅ No 404 error
- ✅ Docker docs page loads

### Test 2.6: Responsive Layout

**Objective:** Verify layout adapts to screen sizes

**Desktop (1920x1080):**
- ✅ 3 columns for first 3 cards
- ✅ 1 card wraps to second row
- ✅ Consistent card heights
- ✅ No horizontal scroll

**Tablet (768x1024):**
- ✅ 2 columns
- ✅ Cards wrap naturally
- ✅ Touch-friendly button sizes
- ✅ No layout breaks

**Mobile (375x667):**
- ✅ 1 column (stacked)
- ✅ Full-width cards
- ✅ Code block scrolls horizontally if needed
- ✅ Copy button remains accessible

### Test 2.7: Animation Behavior

**Objective:** Verify animations work smoothly

**Steps:**
1. Refresh page
2. Scroll to Docker section
3. Observe animation

**Expected Behavior:**
- ✅ Section fades in on scroll
- ✅ Cards appear with staggered delay (0.1s each)
- ✅ Smooth motion (no jank)
- ✅ Animation runs once (viewport: { once: true })

### Test 2.8: Loading State

**Objective:** Verify loading state displays

**Steps:**
1. Slow down network in DevTools (Slow 3G)
2. Refresh page
3. Observe Docker section

**Expected Behavior:**
- ✅ "Loading Docker releases..." text appears
- ✅ No error during load
- ✅ Content replaces loading text when ready
- ✅ No layout shift

### Test 2.9: Error State

**Objective:** Verify error handling

**Steps:**
1. Rename `public/releases.json` temporarily
2. Refresh page
3. Observe error state
4. Restore file

**Expected Behavior:**
- ✅ "Failed to load Docker release information" message
- ✅ No JavaScript console errors
- ✅ Page doesn't crash
- ✅ Other sections still work

### Test 2.10: TypeScript Type Safety

**Objective:** Verify no TypeScript errors

**Steps:**
```bash
npm run build
```

**Expected Behavior:**
- ✅ Build completes successfully
- ✅ No TypeScript errors
- ✅ No type warnings
- ✅ Generated output in `.next/`

## Phase 3: Documentation Testing

### Test 3.1: Docker.md Updates

**Objective:** Verify documentation has new section

**Steps:**
```bash
cat crates/bodhi/src/docs/deployment/docker.md | grep -A 10 "Latest Docker Releases"
```

**Expected Content:**
- ✅ Section "Latest Docker Releases" present
- ✅ Link to getbodhi.app
- ✅ Explanation of automation
- ✅ Benefits listed (bullet points)
- ✅ Note about version tags

### Test 3.2: Variant Comparison Table

**Objective:** Verify table includes all variants

**Steps:**
```bash
cat crates/bodhi/src/docs/deployment/docker.md | grep -A 6 "Variant Comparison"
```

**Expected Content:**
- ✅ CPU variant row
- ✅ CUDA variant row
- ✅ ROCm variant row
- ✅ Vulkan variant row (NEW)
- ✅ Note about new variants at bottom

### Test 3.3: Variant Selection Guide

**Objective:** Verify choosing guidance includes Vulkan

**Steps:**
```bash
cat crates/bodhi/src/docs/deployment/docker.md | grep -A 5 "Choosing a Variant"
```

**Expected Content:**
- ✅ NVIDIA GPU → CUDA
- ✅ AMD GPU → ROCm
- ✅ Cross-vendor GPU → Vulkan (NEW)
- ✅ CPU only → CPU

### Test 3.4: Makefile Comments

**Objective:** Verify Makefile has Docker mentions

**Steps:**
```bash
cat getbodhi.app/Makefile | grep -A 2 "update_releases"
```

**Expected Content:**
- ✅ Comment mentions Docker variants
- ✅ Check target mentions Docker validation
- ✅ Clear help text

## End-to-End Testing Scenarios

### Scenario 1: New Docker Release Flow

**Workflow:**
1. New Docker release created: `docker/v0.0.3`
2. Release body includes 5 variants (adds Intel)
3. Developer runs: `make update_releases`
4. Website rebuilt
5. Users see new version

**Validation Steps:**

**Step 1: Verify Release Exists**
```bash
gh release view docker/v0.0.3
```
- ✅ Release exists
- ✅ Body contains variant sections

**Step 2: Update Releases**
```bash
cd getbodhi.app
make update_releases
```
- ✅ Script discovers new release
- ✅ 5 variants discovered (including intel)
- ✅ Files updated

**Step 3: Verify Data**
```bash
cat .env.release_urls | grep DOCKER_VERSION
cat public/releases.json | jq '.docker.version'
cat public/releases.json | jq '.docker.variants | keys'
```
- ✅ Version updated to 0.0.3
- ✅ Intel variant present
- ✅ 5 variants total

**Step 4: Build Website**
```bash
npm run build
npm run dev
```
- ✅ Build succeeds
- ✅ Website displays 5 cards
- ✅ Intel variant renders with fallback metadata

**Step 5: Add Intel Metadata** (Optional)
```bash
# Edit src/lib/docker-variants.ts
# Add intel metadata
```
- ✅ Intel card shows custom icon/color
- ✅ No breaking changes to other variants

### Scenario 2: Missing releases.json

**Setup:**
```bash
mv getbodhi.app/public/releases.json getbodhi.app/public/releases.json.bak
```

**Test:**
1. Start dev server
2. Navigate to homepage
3. Observe error handling

**Expected Behavior:**
- ✅ Error message displayed
- ✅ No page crash
- ✅ Other sections still work

**Cleanup:**
```bash
mv getbodhi.app/public/releases.json.bak getbodhi.app/public/releases.json
```

### Scenario 3: Unknown Variant

**Setup:**
```bash
# Manually edit releases.json to add fictional variant
cat public/releases.json | jq '.docker.variants.fictional = {
  "image_tag": "0.0.2-fictional",
  "latest_tag": "latest-fictional",
  "platforms": ["linux/amd64"],
  "pull_command": "docker pull ghcr.io/bodhisearch/bodhiapp:0.0.2-fictional"
}' > public/releases.json.tmp
mv public/releases.json.tmp public/releases.json
```

**Test:**
1. Refresh page
2. Observe fictional variant card

**Expected Behavior:**
- ✅ Card renders with fallback metadata
- ✅ Display name: "FICTIONAL"
- ✅ Description: "fictional variant"
- ✅ Gray color scheme
- ✅ CPU icon (default)
- ✅ Copy button works

**Cleanup:**
```bash
# Restore original releases.json
git restore public/releases.json
```

### Scenario 4: Clipboard API Not Available

**Setup:**
```javascript
// In browser DevTools console
const originalClipboard = navigator.clipboard;
delete navigator.clipboard;
```

**Test:**
1. Try to copy command
2. Observe behavior

**Expected Behavior:**
- ✅ Error logged to console
- ✅ Button doesn't change to "Copied!"
- ✅ UI doesn't crash
- ✅ Other functionality works

**Cleanup:**
```javascript
navigator.clipboard = originalClipboard;
```

## Regression Testing Checklist

### Desktop Platform Functionality

- ✅ DownloadSection still renders
- ✅ Platform download buttons work
- ✅ macOS download link correct
- ✅ Windows download link correct
- ✅ Linux download link correct
- ✅ Version numbers display correctly

### Website Navigation

- ✅ Homepage loads
- ✅ Hero section displays
- ✅ Features section displays
- ✅ Download section displays
- ✅ Docker section displays (NEW)
- ✅ CTA section displays
- ✅ Footer displays

### Documentation Links

- ✅ /docs/deployment/docker loads
- ✅ Docker docs content complete
- ✅ Internal links work
- ✅ External links work

### Build Process

- ✅ `npm run build` succeeds
- ✅ No TypeScript errors
- ✅ No linting errors
- ✅ Output bundle size reasonable
- ✅ Static export works

### Mobile Experience

- ✅ Homepage responsive
- ✅ Docker section responsive
- ✅ Touch targets sized correctly
- ✅ No horizontal overflow
- ✅ Text readable without zoom

## Performance Testing

### Page Load Time

**Test:**
```bash
# Build for production
npm run build

# Measure page load
# Use Lighthouse in Chrome DevTools
```

**Benchmarks:**
- ✅ First Contentful Paint < 1.5s
- ✅ Largest Contentful Paint < 2.5s
- ✅ Time to Interactive < 3.5s
- ✅ Total Blocking Time < 300ms

### Bundle Size

**Test:**
```bash
npm run build
ls -lh .next/static/chunks/
```

**Expected:**
- ✅ Main bundle < 200KB (gzipped)
- ✅ Page bundle < 50KB (gzipped)
- ✅ No huge third-party dependencies

### Data Loading

**Test:**
```bash
# Check releases.json size
ls -lh public/releases.json

# Check load time
curl -w "@curl-format.txt" -o /dev/null -s http://localhost:3000/releases.json
```

**Expected:**
- ✅ releases.json < 10KB
- ✅ Loads in < 100ms (local)
- ✅ Cacheable (proper headers)

## Accessibility Testing

### Keyboard Navigation

**Test:**
1. Tab through Docker section
2. Use Enter to activate copy button
3. Verify focus indicators

**Expected:**
- ✅ All interactive elements focusable
- ✅ Focus order logical
- ✅ Copy button activates with Enter/Space
- ✅ Focus indicators visible

### Screen Reader Testing

**Test with VoiceOver (macOS):**
1. Enable VoiceOver
2. Navigate to Docker section
3. Listen to announcements

**Expected:**
- ✅ Section heading announced
- ✅ Variant names announced
- ✅ Descriptions announced
- ✅ Button labels clear
- ✅ Link to docs announced

### Color Contrast

**Test:**
```
Check contrast ratios in DevTools
```

**Expected:**
- ✅ Text on backgrounds: >= 4.5:1
- ✅ Icon colors distinguishable
- ✅ Badge text readable
- ✅ Link text distinguishable

## Automation Potential

### CI/CD Integration

**Possible GitHub Actions:**

```yaml
name: Validate Releases

on:
  push:
    paths:
      - 'getbodhi.app/public/releases.json'
      - 'getbodhi.app/.env.release_urls'

jobs:
  validate:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v3
      - name: Validate releases.json
        run: |
          cd getbodhi.app
          cat public/releases.json | jq '.'
          test $(cat public/releases.json | jq '.docker.variants | length') -ge 4
      - name: Check Docker variables
        run: |
          cd getbodhi.app
          grep -q NEXT_PUBLIC_DOCKER_VERSION .env.release_urls
      - name: Build website
        run: |
          cd getbodhi.app
          npm install
          npm run build
```

### Automated Testing Script

```bash
#!/bin/bash
# test-docker-integration.sh

set -e

cd getbodhi.app

echo "=== Testing Backend ==="
npm run update_releases:check
cat public/releases.json | jq '.docker.variants | keys'

echo "=== Testing Build ==="
npm run build

echo "=== Validation Complete ==="
echo "✓ Backend script works"
echo "✓ Data structure valid"
echo "✓ Website builds successfully"
```

## Test Failure Troubleshooting

### Backend Test Failures

**Symptom:** Script fails to find Docker release

**Possible Causes:**
- No docker/v* release exists
- Release is older than 6 months
- GitHub API rate limit hit

**Solutions:**
```bash
# Check if release exists
gh release list | grep docker

# Check API rate limit
gh api rate_limit

# Use authenticated requests (higher limits)
```

**Symptom:** Variants not discovered

**Possible Causes:**
- Release body doesn't contain variant sections
- Regex pattern mismatch
- Release body format changed

**Solutions:**
```bash
# Check release body
gh release view docker/v0.0.2 --json body -q .body

# Check for ### Variant sections
gh release view docker/v0.0.2 --json body -q .body | grep "###"

# Manually validate regex pattern
```

### Frontend Test Failures

**Symptom:** Component doesn't render

**Possible Causes:**
- releases.json missing or invalid
- TypeScript type errors
- React component errors

**Solutions:**
```bash
# Validate JSON
cat public/releases.json | jq '.'

# Check build errors
npm run build

# Check browser console
# Open DevTools → Console tab
```

**Symptom:** Copy button doesn't work

**Possible Causes:**
- Clipboard API not available (HTTP vs HTTPS)
- Browser permissions denied
- JavaScript error

**Solutions:**
```bash
# Use HTTPS or localhost (clipboard API requirements)
# Check browser console for errors
# Try in different browser
```

## Summary and Recommendations

### Testing Coverage

**Completed:**
- ✅ Phase 1: Backend variant discovery
- ✅ Phase 2: Frontend rendering and interaction
- ✅ Phase 3: Documentation updates
- ✅ End-to-end workflow validation
- ✅ Regression testing for existing features
- ✅ Responsive design validation
- ✅ Accessibility basics

**Future Enhancements:**
- Automated E2E tests with Playwright
- Visual regression testing
- Performance monitoring
- User analytics integration

### Key Success Metrics

**Functional:**
- All 4 variants discovered automatically
- All variants display correctly
- Copy-to-clipboard works reliably
- Documentation accurate and complete

**Non-Functional:**
- Page loads quickly (<3s)
- No TypeScript errors
- Responsive across devices
- Accessible with keyboard/screen reader

### Continuous Validation

**Before Each Release:**
1. Run `make update_releases.check`
2. Verify variant count matches expectation
3. Build website: `npm run build`
4. Spot-check on localhost
5. Commit changes

**Periodic Checks:**
- Monthly: Verify Docker docs accuracy
- Quarterly: Test with new browser versions
- After major Next.js updates: Full regression test

### Next Steps

1. Add automated CI tests for releases.json validation
2. Set up Playwright tests for E2E flow
3. Monitor user feedback on variant selection
4. Track copy-to-clipboard usage with privacy-respecting analytics
5. Document user issues and edge cases as discovered

This testing guide ensures the Docker release integration remains reliable and maintainable as the feature evolves and new variants are added.
