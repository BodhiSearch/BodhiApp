# Docker Variant Extension Guide

**Version:** 1.0
**Date:** 2025-10-10
**Audience:** Developers adding new Docker variants

## Overview

This guide provides step-by-step instructions for adding new Docker variants to BodhiApp. The implementation uses automatic variant discovery, so most steps don't require website code changes.

## When to Add a New Variant

**Common Scenarios:**
- Supporting new GPU vendor (e.g., Intel GPUs)
- Adding experimental hardware acceleration (e.g., Metal, DirectML)
- Creating specialized variants (e.g., quantized models)
- Platform-specific optimizations

**Prerequisites:**
- Docker variant builds successfully
- llama.cpp supports the hardware
- Dockerfile exists with proper configuration
- Testing completed on target hardware

## Quick Reference

### Minimal Workflow (Auto-Discovery)

```bash
# 1. Create Dockerfile
devops/{variant}.Dockerfile

# 2. Update workflow
.github/workflows/publish-docker.yml

# 3. Release Docker images
git tag docker/vX.Y.Z
git push origin docker/vX.Y.Z

# 4. Update website
cd getbodhi.app
make update_releases

# 5. Deploy website
git add .env.release_urls public/releases.json
git commit -m "Update Docker releases with {variant} variant"
```

### Enhanced Workflow (With Custom Metadata)

Add step 4.5: Edit `getbodhi.app/src/lib/docker-variants.ts` for better UX

## Detailed Step-by-Step Guide

### Step 1: Create Dockerfile

**Location:** `devops/{variant}.Dockerfile`

**Example: Intel GPU Variant**

```dockerfile
# devops/intel.Dockerfile
FROM intel/oneapi-basekit:2024.0.0-devel-ubuntu22.04

# Install dependencies
RUN apt-get update && apt-get install -y \
    build-essential \
    cmake \
    git \
    wget \
    && rm -rf /var/lib/apt/lists/*

# Build llama.cpp with Intel GPU support
WORKDIR /app
RUN git clone https://github.com/ggerganov/llama.cpp.git
WORKDIR /app/llama.cpp
RUN cmake -B build -DLLAMA_SYCL=ON
RUN cmake --build build --config Release

# Copy BodhiApp binary
COPY --from=builder /app/target/release/bodhi /usr/local/bin/bodhi

# Environment and entrypoint
ENV BODHI_PORT=1135
ENV BODHI_HOST=0.0.0.0
EXPOSE 1135
ENTRYPOINT ["bodhi", "serve"]
```

**Key Requirements:**
- Base image with required runtime (Intel oneAPI, ROCm, etc.)
- llama.cpp built with hardware-specific flags
- BodhiApp binary included
- Proper environment variables
- Health check optional but recommended

**Testing:**
```bash
# Build locally
docker build -f devops/intel.Dockerfile -t bodhiapp:test-intel .

# Test run
docker run --device=/dev/dri -p 1135:1135 bodhiapp:test-intel

# Verify GPU access
docker exec -it <container> intel-gpu-top  # or equivalent
```

### Step 2: Update GitHub Workflow

**Location:** `.github/workflows/publish-docker.yml`

**Add to Matrix:**

```yaml
strategy:
  matrix:
    variant:
      - name: cpu
        dockerfile: devops/cpu.Dockerfile
        platforms: linux/amd64,linux/arm64
      - name: cuda
        dockerfile: devops/cuda.Dockerfile
        platforms: linux/amd64
      - name: rocm
        dockerfile: devops/rocm.Dockerfile
        platforms: linux/amd64
      - name: vulkan
        dockerfile: devops/vulkan.Dockerfile
        platforms: linux/amd64
      # NEW VARIANT
      - name: intel
        dockerfile: devops/intel.Dockerfile
        platforms: linux/amd64
```

**Update Release Body Template:**

```yaml
- name: Generate release body
  run: |
    cat > release-body.md << 'EOF'
    # Docker Release ${{ env.VERSION }}

    <!-- Existing variants -->

    ### Intel Variant

    Intel GPU acceleration for Intel Arc and Xe graphics.

    **Hardware:** Intel Arc A-series, Intel Xe graphics
    **Platforms:** linux/amd64

    \`\`\`bash
    docker pull ghcr.io/bodhisearch/bodhiapp:${{ env.VERSION }}-intel
    docker run --device=/dev/dri -p 1135:1135 \\
      -v bodhi-data:/data -v bodhi-models:/models \\
      ghcr.io/bodhisearch/bodhiapp:${{ env.VERSION }}-intel
    \`\`\`
    EOF
```

**Release Body Format Requirements:**

The variant auto-discovery relies on this format:

```markdown
### {Variant} Variant

{Description of variant}

**Hardware:** {Hardware requirements}
**Platforms:** {Platform list}

```bash
docker pull ghcr.io/bodhisearch/bodhiapp:{version}-{variant}
```
```

**Critical Elements:**
1. Section header: `### {Variant} Variant` (must match pattern)
2. Hardware description (optional, for docs)
3. Platforms line (parsed for platform list)
4. Docker command example (optional, for users)

**Parsing Behavior:**
- Variant name: Captured from `### Intel Variant` → `intel` (lowercase)
- Platforms: Extracted from **Platforms:** line or inferred
- GPU type: Detected from keywords (NVIDIA, AMD GPU, Intel, Cross-vendor)
- Description: Everything between header and next section

### Step 3: Create and Push Docker Release

**Create Release Tag:**

```bash
# Ensure clean working directory
git status

# Create tag for Docker release
git tag docker/v0.0.3 -m "Docker release v0.0.3 - adds Intel variant"

# Push tag (triggers workflow)
git push origin docker/v0.0.3
```

**Monitor Workflow:**

```bash
# Watch workflow progress
gh run watch

# Or view in browser
open https://github.com/BodhiSearch/BodhiApp/actions/workflows/publish-docker.yml
```

**Verify Release:**

```bash
# Check release created
gh release view docker/v0.0.3

# Verify release body contains Intel section
gh release view docker/v0.0.3 --json body -q .body | grep "### Intel Variant"

# Check Docker images published
gh api /orgs/BodhiSearch/packages/container/bodhiapp/versions
```

**Manual Test (Optional):**

```bash
# Pull released image
docker pull ghcr.io/bodhisearch/bodhiapp:0.0.3-intel

# Run container
docker run --device=/dev/dri -p 1135:1135 \
  -v bodhi-data:/data -v bodhi-models:/models \
  ghcr.io/bodhisearch/bodhiapp:0.0.3-intel

# Test API
curl http://localhost:1135/bodhi/v1/info

# Stop container
docker stop <container-id>
```

### Step 4: Update Website Release Data

**Run Update Script:**

```bash
cd getbodhi.app

# Dry-run first (verify discovery)
make update_releases.check
```

**Expected Output:**

```
=== Checking latest releases (dry-run) ===

Fetching releases from GitHub...
✓ Found Docker release: docker/v0.0.3
  Variants discovered: cpu, cuda, rocm, vulkan, intel

=== Dry-run mode - would write to public/releases.json: ===
{
  "docker": {
    "version": "0.0.3",
    "variants": {
      "cpu": { ... },
      "cuda": { ... },
      "rocm": { ... },
      "vulkan": { ... },
      "intel": {
        "image_tag": "0.0.3-intel",
        "latest_tag": "latest-intel",
        "platforms": ["linux/amd64"],
        "pull_command": "docker pull ghcr.io/bodhisearch/bodhiapp:0.0.3-intel",
        "gpu_type": "Intel",
        "description": "Intel GPU acceleration"
      }
    }
  }
}
```

**Validation:**
- ✅ Intel variant discovered automatically
- ✅ 5 variants total
- ✅ GPU type: "Intel"
- ✅ Platform: linux/amd64
- ✅ Pull command correct

**Execute Update:**

```bash
# Update files
make update_releases

# Verify changes
git diff .env.release_urls public/releases.json
```

**Review Changes:**

```bash
# Check Docker version updated
cat .env.release_urls | grep DOCKER_VERSION
# Should show: NEXT_PUBLIC_DOCKER_VERSION=0.0.3

# Check Intel variant present
cat public/releases.json | jq '.docker.variants.intel'
# Should show complete Intel variant object

# Count variants
cat public/releases.json | jq '.docker.variants | keys | length'
# Should show: 5
```

### Step 4.5: Add Custom Metadata (Optional but Recommended)

**Why Add Metadata:**
- Better visual design (custom colors, icons)
- Clearer descriptions for users
- Professional appearance
- Consistent with other variants

**Edit File:** `getbodhi.app/src/lib/docker-variants.ts`

**Add Metadata Entry:**

```typescript
export const VARIANT_METADATA: Record<string, DockerVariantMetadata> = {
  // ... existing variants (cpu, cuda, rocm, vulkan)

  // NEW: Intel variant metadata
  intel: {
    name: 'intel',
    displayName: 'Intel',
    description: 'Intel GPU acceleration (Arc, Xe graphics)',
    icon: 'gpu-generic',
    gpuVendor: 'Intel',
    color: 'indigo',
  },
};
```

**Metadata Fields:**

| Field | Type | Required | Description | Example |
|-------|------|----------|-------------|---------|
| name | string | Yes | Internal key (match variant name) | "intel" |
| displayName | string | Yes | User-facing name | "Intel" |
| description | string | Yes | Brief explanation | "Intel GPU acceleration" |
| icon | string | Yes | Icon type | "cpu" \| "gpu-generic" |
| gpuVendor | string | No | GPU vendor for badge | "Intel" |
| recommended | boolean | No | Show recommended badge | false |
| color | string | Yes | Tailwind color class | "indigo" |

**Available Colors:**
- `blue` - Cool, professional (CPU variant)
- `green` - Performance, speed (CUDA variant)
- `red` - Power, AMD (ROCm variant)
- `purple` - Versatile (Vulkan variant)
- `indigo` - Modern, Intel (Intel variant)
- `gray` - Neutral fallback

**Available Icons:**
- `cpu` - For CPU variants
- `gpu-nvidia` - Not used (same as gpu-generic with color)
- `gpu-amd` - Not used (same as gpu-generic with color)
- `gpu-generic` - For any GPU variant

**Test Metadata:**

```bash
# Build website
npm run build

# Start dev server
npm run dev

# Navigate to http://localhost:3000
# Verify Intel card shows custom color/description
```

**Without Custom Metadata:**
- Display Name: "INTEL" (auto-capitalized)
- Description: "intel variant" (generic)
- Color: Gray (default)
- Icon: CPU (default)

**With Custom Metadata:**
- Display Name: "Intel"
- Description: "Intel GPU acceleration (Arc, Xe graphics)"
- Color: Indigo
- Icon: GPU (Zap icon)
- GPU Badge: "Intel GPU"

### Step 5: Build and Test Website

**Build Website:**

```bash
cd getbodhi.app

# Install dependencies (if needed)
npm install

# Build production bundle
npm run build
```

**Expected Output:**

```
✓ Compiled successfully
✓ Static page generation completed
✓ Finalizing page optimization
```

**Validation:**
- ✅ Build completes without errors
- ✅ No TypeScript errors
- ✅ No missing dependencies

**Test Locally:**

```bash
# Start dev server
npm run dev

# Open browser
open http://localhost:3000
```

**Manual Verification:**

1. **Scroll to Docker Section**
   - ✅ "Deploy with Docker" heading visible
   - ✅ 5 variant cards displayed
   - ✅ Grid layout: 3 columns, wraps to 2nd row

2. **Check Intel Card**
   - ✅ Display Name: "Intel"
   - ✅ Icon: Indigo Zap icon (GPU)
   - ✅ Badge: "Intel GPU" (violet)
   - ✅ Description: Custom description (if metadata added)
   - ✅ Platforms: "linux/amd64"
   - ✅ Pull command: `docker pull ghcr.io/bodhisearch/bodhiapp:0.0.3-intel`

3. **Test Copy Button**
   - ✅ Click "Copy Command" on Intel card
   - ✅ Button changes to "Copied!" with check icon
   - ✅ Clipboard contains: `docker pull ghcr.io/bodhisearch/bodhiapp:0.0.3-intel`
   - ✅ Button reverts after 2 seconds

4. **Test Responsive Design**
   - Desktop (1920px): 3 columns, 2nd row has 2 cards
   - Tablet (768px): 2 columns, 3 rows
   - Mobile (375px): 1 column, 5 rows
   - ✅ No horizontal overflow
   - ✅ Cards maintain consistent height

5. **Test Other Variants Still Work**
   - ✅ CPU card displays correctly
   - ✅ CUDA card displays correctly
   - ✅ ROCm card displays correctly
   - ✅ Vulkan card displays correctly

### Step 6: Commit and Deploy

**Review Changes:**

```bash
cd getbodhi.app

# Check which files changed
git status

# Review changes
git diff .env.release_urls
git diff public/releases.json
git diff src/lib/docker-variants.ts  # if metadata added
```

**Expected Changes:**

1. **.env.release_urls:**
   - `NEXT_PUBLIC_DOCKER_VERSION` updated to 0.0.3
   - `NEXT_PUBLIC_DOCKER_TAG` updated to docker/v0.0.3

2. **public/releases.json:**
   - `docker.version` updated to "0.0.3"
   - `docker.variants.intel` added

3. **src/lib/docker-variants.ts:** (if metadata added)
   - Intel metadata entry added

**Commit Changes:**

```bash
# Add files
git add .env.release_urls public/releases.json src/lib/docker-variants.ts

# Commit with clear message
git commit -m "Update Docker releases to v0.0.3 with Intel GPU variant

- Auto-discovered Intel variant from docker/v0.0.3 release
- Added custom metadata for Intel variant (indigo theme)
- Updated releases.json with 5 variants
- Updated environment variables to v0.0.3"

# Push to main branch
git push origin main
```

**Deploy Website:**

```bash
# Create website release tag
cd getbodhi.app
make release

# Or manually:
# git tag getbodhi.app/vX.Y.Z
# git push origin getbodhi.app/vX.Y.Z
```

**Monitor Deployment:**

```bash
# Watch GitHub Actions workflow
gh run watch

# Or view in browser
open https://github.com/BodhiSearch/BodhiApp/actions/workflows/deploy-website.yml
```

**Verify Live Website:**

```bash
# Wait for deployment (usually 3-5 minutes)

# Check deployed version
curl https://getbodhi.app/version.json

# Check releases data
curl https://getbodhi.app/releases.json | jq '.docker.variants | keys'
# Should show: ["cpu", "cuda", "intel", "rocm", "vulkan"]

# Test copy-to-clipboard on live site
open https://getbodhi.app
```

## Troubleshooting

### Variant Not Discovered

**Symptom:** Script doesn't find new variant

**Possible Causes:**
1. Release body missing `### {Variant} Variant` section
2. Variant name has special characters (not `\w+`)
3. Section formatting incorrect

**Solutions:**

```bash
# Check release body format
gh release view docker/v0.0.3 --json body -q .body

# Verify section header exists
gh release view docker/v0.0.3 --json body -q .body | grep "### Intel Variant"

# Test regex pattern locally
echo "### Intel Variant\n\nDescription" | grep -E "###\s+(\w+)\s+Variant"
```

**Fix:**
```bash
# Edit release body (GitHub web UI)
# Ensure format: ### Intel Variant

# Or recreate release
gh release delete docker/v0.0.3 --yes
gh release create docker/v0.0.3 -F release-body.md
```

### GPU Type Not Detected

**Symptom:** `gpu_type` field missing in releases.json

**Possible Causes:**
1. Release body doesn't contain GPU vendor keyword
2. Keyword not in detection list

**Solutions:**

```bash
# Check release body for keywords
gh release view docker/v0.0.3 --json body -q .body | grep -i intel

# Add keyword to release body
# Edit release body to include: "Intel GPU" or "Intel Arc"
```

**Keywords Detected:**
- "NVIDIA" → gpu_type: "NVIDIA"
- "AMD GPU" → gpu_type: "AMD"
- "Intel" → gpu_type: "Intel"
- "Cross-vendor" → gpu_type: "Cross-vendor"

### Platforms Not Parsed Correctly

**Symptom:** Only shows `["linux/amd64"]` despite multi-platform support

**Possible Causes:**
1. Release body missing platform specification
2. Format doesn't match regex patterns

**Solutions:**

**Format 1 (Multi-platform keyword):**
```markdown
### CPU Variant (Multi-platform: AMD64 + ARM64)
```

**Format 2 (Explicit platforms):**
```markdown
### CPU Variant

**Platforms:** linux/amd64, linux/arm64
```

**Update script if needed:**
```javascript
// In parseDockerVariants function
// Add new format detection
```

### Website Build Fails

**Symptom:** `npm run build` errors

**Possible Causes:**
1. TypeScript type errors
2. Invalid releases.json format
3. Missing dependencies

**Solutions:**

```bash
# Validate JSON
cat public/releases.json | jq '.'

# Check TypeScript errors
npm run build 2>&1 | grep -i error

# Reinstall dependencies
rm -rf node_modules package-lock.json
npm install

# Check for type mismatches
cat public/releases.json | jq '.docker.variants.intel | keys'
```

### Copy Button Doesn't Work

**Symptom:** Clicking copy button does nothing

**Possible Causes:**
1. Clipboard API not available (HTTP)
2. Browser permissions
3. JavaScript error

**Solutions:**

```bash
# Ensure using HTTPS or localhost
npm run dev  # Runs on localhost (clipboard API works)

# Check browser console for errors
# Open DevTools → Console

# Test in different browser
# Chrome, Firefox, Safari

# Verify pull_command field exists
cat public/releases.json | jq '.docker.variants.intel.pull_command'
```

### Metadata Not Applied

**Symptom:** Variant shows gray color with "INTEL" name

**Possible Causes:**
1. Metadata key doesn't match variant name
2. TypeScript cache issue
3. Build didn't pick up changes

**Solutions:**

```bash
# Check metadata key matches variant name
cat src/lib/docker-variants.ts | grep -A 6 "intel:"

# Clear Next.js cache
rm -rf .next

# Rebuild
npm run build
npm run dev

# Verify metadata export
cat src/lib/docker-variants.ts | grep "export const VARIANT_METADATA"
```

## Checklist

Use this checklist when adding a new variant:

### Pre-Release Checklist

- [ ] Dockerfile created and tested locally
- [ ] llama.cpp hardware support verified
- [ ] Docker image builds successfully
- [ ] Container runs with proper device access
- [ ] Inference works on target hardware
- [ ] Performance benchmarked (optional)

### Release Checklist

- [ ] Updated `.github/workflows/publish-docker.yml` matrix
- [ ] Updated release body template with variant section
- [ ] Created Docker release tag (`docker/vX.Y.Z`)
- [ ] Workflow completed successfully
- [ ] Images published to ghcr.io
- [ ] Release body contains variant section
- [ ] Release body format correct (### {Variant} Variant)

### Website Update Checklist

- [ ] Ran `make update_releases.check` (dry-run)
- [ ] Verified variant discovered in output
- [ ] Ran `make update_releases` (actual update)
- [ ] `.env.release_urls` updated
- [ ] `public/releases.json` updated
- [ ] Variant has all required fields
- [ ] (Optional) Added custom metadata to `docker-variants.ts`
- [ ] Website builds without errors (`npm run build`)
- [ ] Tested locally (`npm run dev`)
- [ ] Variant card displays correctly
- [ ] Copy button works
- [ ] Responsive layout verified

### Deployment Checklist

- [ ] Committed changes to git
- [ ] Pushed to main branch
- [ ] Created website release tag
- [ ] Deployment workflow completed
- [ ] Live website shows new variant
- [ ] releases.json accessible via CDN
- [ ] Copy-to-clipboard works on live site
- [ ] All existing variants still work
- [ ] Documentation updated (if needed)

### Post-Deployment Checklist

- [ ] Notified team of new variant
- [ ] Updated internal docs
- [ ] Announced on social media / blog (if applicable)
- [ ] Monitored user feedback
- [ ] Added troubleshooting notes for common issues

## Common Patterns

### Pattern 1: GPU Vendor Variant

**Characteristics:**
- Requires GPU device access
- Vendor-specific base image
- llama.cpp with vendor flags
- Single platform (linux/amd64)

**Examples:** CUDA, ROCm, Intel

**Template:**

```dockerfile
FROM {vendor-base-image}
RUN cmake -B build -D{VENDOR_FLAG}=ON
...
```

```yaml
variant:
  - name: {vendor}
    dockerfile: devops/{vendor}.Dockerfile
    platforms: linux/amd64
```

```markdown
### {Vendor} Variant

{Vendor} GPU acceleration for {hardware}.

**Hardware:** {GPU models}
**Platforms:** linux/amd64
```

### Pattern 2: Cross-Vendor Variant

**Characteristics:**
- Works across multiple vendors
- Generic GPU API (Vulkan, DirectML)
- Wider compatibility, potentially lower performance

**Example:** Vulkan

**Template:**

```dockerfile
FROM ubuntu:22.04
RUN apt-get install -y vulkan-tools
RUN cmake -B build -DLLAMA_VULKAN=ON
...
```

```yaml
variant:
  - name: vulkan
    dockerfile: devops/vulkan.Dockerfile
    platforms: linux/amd64
```

```markdown
### Vulkan Variant

Cross-vendor GPU acceleration using Vulkan API.

**Hardware:** NVIDIA, AMD, Intel GPUs
**Platforms:** linux/amd64
```

### Pattern 3: Multi-Platform Variant

**Characteristics:**
- Supports multiple CPU architectures
- No GPU requirements
- Slower but wider compatibility

**Example:** CPU

**Template:**

```dockerfile
FROM ubuntu:22.04
RUN cmake -B build
# No special flags
...
```

```yaml
variant:
  - name: cpu
    dockerfile: devops/cpu.Dockerfile
    platforms: linux/amd64,linux/arm64
```

```markdown
### CPU Variant (Multi-platform: AMD64 + ARM64)

General-purpose CPU inference for all platforms.

**Hardware:** Any CPU
**Platforms:** linux/amd64, linux/arm64
```

## Best Practices

### Documentation

1. **Clear Hardware Requirements:** Specify exact GPU models/generations supported
2. **Device Mapping Examples:** Show docker run commands with correct device flags
3. **Performance Notes:** Include expected speedup vs CPU variant
4. **Limitations:** Document known issues or unsupported models

### Testing

1. **Local Testing:** Always test variant locally before release
2. **Performance Benchmarking:** Run inference tests on target hardware
3. **Multi-Model Testing:** Test with different model sizes/quantizations
4. **Resource Monitoring:** Check GPU utilization, VRAM usage

### Release Notes

1. **Version Changelog:** Document what's new in this variant
2. **Breaking Changes:** Note any compatibility changes
3. **Migration Guide:** If replacing old variant, provide upgrade path
4. **Known Issues:** Be transparent about limitations

### Maintenance

1. **Regular Updates:** Keep base images and dependencies updated
2. **Security Patches:** Monitor and apply security updates
3. **Performance Tuning:** Optimize based on user feedback
4. **Deprecation Plan:** If variant becomes obsolete, provide migration path

## Examples

### Example 1: Adding Intel Arc Support (Complete Walkthrough)

See main guide above.

### Example 2: Adding Experimental Variant (Vulkan Async)

**Scenario:** Add async compute variant of Vulkan for better parallelism

**Steps:**

1. **Create Dockerfile:**
```bash
cp devops/vulkan.Dockerfile devops/vulkan-async.Dockerfile
# Edit to enable async compute flags
```

2. **Update Workflow:**
```yaml
- name: vulkan-async
  dockerfile: devops/vulkan-async.Dockerfile
  platforms: linux/amd64
```

3. **Update Release Body:**
```markdown
### Vulkan-Async Variant (Experimental)

Vulkan variant with async compute for better GPU parallelization.

**Hardware:** NVIDIA, AMD, Intel GPUs with async compute support
**Platforms:** linux/amd64
**Status:** Experimental - may have stability issues
```

4. **Release and Update:**
```bash
git tag docker/v0.0.4
git push origin docker/v0.0.4
cd getbodhi.app
make update_releases
```

5. **Add Metadata:**
```typescript
'vulkan-async': {
  name: 'vulkan-async',
  displayName: 'Vulkan Async (Experimental)',
  description: 'Async compute for better GPU parallelization',
  icon: 'gpu-generic',
  gpuVendor: 'Cross-vendor',
  color: 'purple',
},
```

## Summary

Adding a new Docker variant involves:
1. Creating the Dockerfile with proper hardware support
2. Updating the GitHub workflow to build the variant
3. Releasing with properly formatted release body
4. Running website update script (automatic discovery)
5. Optionally adding custom metadata for better UX
6. Testing and deploying

The key benefit of this architecture is that steps 1-3 handle the Docker side, and step 4 automatically updates the website without code changes. Step 5 is optional but recommended for the best user experience.

For questions or issues, refer to the troubleshooting section or consult IMPLEMENTATION.md for technical details.
