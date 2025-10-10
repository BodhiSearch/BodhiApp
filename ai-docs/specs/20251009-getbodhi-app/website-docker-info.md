# Docker Release Information Integration - Implementation Specification

**Created:** 2025-10-10
**Status:** Planning
**Agents:** Sequential execution (Phase 1 → Phase 2 → Phase 3)

## Overview

Integrate Docker release information into the getbodhi.app website, following the existing pattern used for desktop platform releases. This implementation must automatically discover and display all Docker variants (current: CPU, CUDA, ROCm; future: Vulkan, Intel, AMD) without requiring code changes when new variants are released.

## Current State Analysis

### Platform Release Automation (Working Model)
- **Script:** `getbodhi.app/scripts/update-release-urls.js`
- **Env file:** `getbodhi.app/.env.release_urls`
- **JSON output:** `getbodhi.app/public/releases.json`
- **Constants:** `getbodhi.app/src/lib/constants.ts`
- **Component:** `getbodhi.app/src/app/DownloadSection.tsx`
- **Automation:** `getbodhi.app/Makefile` targets (update_releases, update_releases.check)

### Docker Release Infrastructure (Available)
- **Production releases:** `docker/v*` tags (e.g., docker/v0.0.2)
- **Dev releases:** `docker-dev/v*` tags (not shown on website)
- **Current variants:** cpu (multi-arch), cuda, rocm
- **Future variants:** vulkan, intel, amd (planned)
- **Registry:** ghcr.io/bodhisearch/bodhiapp
- **Version fetcher:** `scripts/get_ghcr_version.py` (Python)
- **Workflow:** `.github/workflows/publish-docker.yml`
- **Documentation:** `crates/bodhi/src/docs/deployment/docker.md`

### Key Constraints
1. **No download button** - Docker section shows `docker pull` commands (not downloads)
2. **Link to docs** - Must reference `/docs/deployment/docker` for volume mounting and env vars
3. **Auto-discovery** - New variants must appear automatically when released
4. **Production only** - Only `docker/v*` releases shown (not `docker-dev/v*`)

## Implementation Plan - 3 Sequential Phases

### Phase 1: Backend Infrastructure with Variant Auto-Discovery

**Agent:** general-purpose
**Objective:** Extend release automation to auto-discover all Docker variants dynamically

**Tasks:**

1. **Extend `update-release-urls.js`** with Docker variant discovery:
   - Add tag pattern for `docker/v*` (production only)
   - Query GitHub releases API for latest `docker/v*` release
   - Parse release body to extract ALL available variants automatically
   - The `publish-docker.yml` workflow already documents variants in release body
   - Extract registry URLs and metadata for each discovered variant
   - Structure data dynamically without hardcoding variant list

2. **Design extensible data structure** in `public/releases.json`:
   ```json
   {
     "desktop": {
       "version": "0.0.31",
       "tag": "app/v0.0.31",
       "platforms": { ... }
     },
     "docker": {
       "version": "0.0.2",
       "tag": "docker/v0.0.2",
       "released_at": "2025-10-07T12:58:40Z",
       "registry": "ghcr.io/bodhisearch/bodhiapp",
       "variants": {
         "cpu": {
           "image_tag": "0.0.2-cpu",
           "latest_tag": "latest-cpu",
           "platforms": ["linux/amd64", "linux/arm64"],
           "pull_command": "docker pull ghcr.io/bodhisearch/bodhiapp:0.0.2-cpu",
           "description": "Multi-platform CPU variant"
         },
         "cuda": {
           "image_tag": "0.0.2-cuda",
           "latest_tag": "latest-cuda",
           "platforms": ["linux/amd64"],
           "pull_command": "docker pull ghcr.io/bodhisearch/bodhiapp:0.0.2-cuda",
           "gpu_type": "NVIDIA",
           "description": "NVIDIA GPU acceleration"
         },
         "rocm": {
           "image_tag": "0.0.2-rocm",
           "latest_tag": "latest-rocm",
           "platforms": ["linux/amd64"],
           "pull_command": "docker pull ghcr.io/bodhisearch/bodhiapp:0.0.2-rocm",
           "gpu_type": "AMD",
           "description": "AMD GPU acceleration"
         }
       }
     }
   }
   ```

3. **Update `.env.release_urls`** with Docker environment variables:
   ```
   # Docker version and tag
   NEXT_PUBLIC_DOCKER_VERSION=0.0.2
   NEXT_PUBLIC_DOCKER_TAG=docker/v0.0.2
   NEXT_PUBLIC_DOCKER_REGISTRY=ghcr.io/bodhisearch/bodhiapp
   ```

4. **Implementation strategy** for variant discovery:
   ```javascript
   async function fetchDockerVariants(release) {
     const variants = {};
     const registry = 'ghcr.io/bodhisearch/bodhiapp';
     const version = release.tag_name.replace('docker/v', '');

     // Parse release body for variant information
     // Pattern: "### CPU Variant", "### CUDA Variant", etc.
     const variantPattern = /### (\w+) Variant[^#]*/gi;
     let match;

     while ((match = variantPattern.exec(release.body)) !== null) {
       const variantName = match[1].toLowerCase();
       const sectionText = match[0];

       // Extract platforms from section
       const platformMatch = /Platforms[:\s]+(.*)/i.exec(sectionText);
       const platforms = platformMatch ?
         platformMatch[1].split(',').map(p => p.trim()) :
         ['linux/amd64'];

       // Extract GPU type if present
       const gpuMatch = /(NVIDIA|AMD|Intel)/i.exec(sectionText);
       const gpuType = gpuMatch ? gpuMatch[1] : null;

       variants[variantName] = {
         image_tag: `${version}-${variantName}`,
         latest_tag: `latest-${variantName}`,
         platforms: platforms,
         pull_command: `docker pull ${registry}:${version}-${variantName}`,
         ...(gpuType && { gpu_type: gpuType })
       };
     }

     return variants;
   }
   ```

**Deliverables:**
- Modified `getbodhi.app/scripts/update-release-urls.js` with Docker support
- Updated `getbodhi.app/public/releases.json` schema with docker section
- Updated `getbodhi.app/.env.release_urls` with Docker variables
- Working automation: `make website.update_releases` discovers all variants
- Verification: `make website.update_releases.check` validates Docker data

**Validation:**
```bash
cd getbodhi.app
npm run update_releases
cat .env.release_urls | grep DOCKER
cat public/releases.json | jq '.docker.variants | keys'
# Should show: ["cpu", "cuda", "rocm"]
```

**Context to Pass Forward:**
- Location of releases.json with Docker data
- Structure of docker.variants object
- List of discovered variants from current release
- Registry URL pattern
- Pull command format

**Agent Log:** `ai-docs/specs/20251009-getbodhi-app/phase1-backend.log`

---

### Phase 2: Frontend Docker Display Component

**Agent:** general-purpose
**Objective:** Create dynamic UI component that adapts to available Docker variants

**Prerequisites:**
- Phase 1 completed: releases.json contains docker section
- Read context from Phase 1 about data structure

**Tasks:**

1. **Create variant metadata system** in `src/lib/docker-variants.ts`:
   ```typescript
   export interface DockerVariantMetadata {
     name: string;
     displayName: string;
     description: string;
     icon: 'cpu' | 'gpu-nvidia' | 'gpu-amd' | 'gpu-generic';
     gpuVendor?: 'NVIDIA' | 'AMD' | 'Intel' | 'Cross-vendor';
     recommended?: boolean;
     color?: string; // Tailwind color class
   }

   export const VARIANT_METADATA: Record<string, DockerVariantMetadata> = {
     cpu: {
       name: 'cpu',
       displayName: 'CPU',
       description: 'Multi-platform (AMD64 + ARM64)',
       icon: 'cpu',
       recommended: true,
       color: 'blue'
     },
     cuda: {
       name: 'cuda',
       displayName: 'CUDA',
       description: 'NVIDIA GPU acceleration (8-12x faster)',
       icon: 'gpu-nvidia',
       gpuVendor: 'NVIDIA',
       color: 'green'
     },
     rocm: {
       name: 'rocm',
       displayName: 'ROCm',
       description: 'AMD GPU acceleration',
       icon: 'gpu-amd',
       gpuVendor: 'AMD',
       color: 'red'
     },
     vulkan: {
       name: 'vulkan',
       displayName: 'Vulkan',
       description: 'Cross-vendor GPU acceleration',
       icon: 'gpu-generic',
       gpuVendor: 'Cross-vendor',
       color: 'purple'
     },
     intel: {
       name: 'intel',
       displayName: 'Intel',
       description: 'Intel GPU acceleration',
       icon: 'gpu-generic',
       gpuVendor: 'Intel',
       color: 'indigo'
     }
   };

   export function getVariantMetadata(variantKey: string): DockerVariantMetadata {
     return VARIANT_METADATA[variantKey] || {
       name: variantKey,
       displayName: variantKey.toUpperCase(),
       description: `${variantKey} variant`,
       icon: 'cpu',
       color: 'gray'
     };
   }
   ```

2. **Update `src/lib/constants.ts`** with Docker constants:
   ```typescript
   // Docker release info from environment
   export const DOCKER_VERSION = process.env.NEXT_PUBLIC_DOCKER_VERSION;
   export const DOCKER_TAG = process.env.NEXT_PUBLIC_DOCKER_TAG;
   export const DOCKER_REGISTRY = process.env.NEXT_PUBLIC_DOCKER_REGISTRY;
   ```

3. **Create `src/app/DockerSection.tsx`** component:
   - Load docker data from `public/releases.json`
   - Dynamically render cards for ALL discovered variants
   - Display docker pull command with copy-to-clipboard button
   - Show platforms (linux/amd64, linux/arm64)
   - GPU badge for accelerated variants
   - "Recommended" badge for CPU variant
   - Link to `/docs/deployment/docker` at bottom with clear message

4. **Component structure:**
   ```tsx
   'use client';

   import { Container } from '@/components/ui/container';
   import { Card } from '@/components/ui/card';
   import { Button } from '@/components/ui/button';
   import { Copy, Cpu, Zap } from 'lucide-react';
   import Link from 'next/link';
   import { motion } from 'framer-motion';
   import { getVariantMetadata } from '@/lib/docker-variants';
   import { useState } from 'react';

   interface DockerVariant {
     image_tag: string;
     latest_tag: string;
     platforms: string[];
     pull_command: string;
     gpu_type?: string;
     description?: string;
   }

   export function DockerSection() {
     // Load from releases.json or API endpoint
     const dockerData = loadDockerData();

     return (
       <section id="docker-section" className="py-20 bg-gradient-to-b from-white to-slate-50">
         <Container>
           <motion.div
             initial={{ opacity: 0, y: 20 }}
             whileInView={{ opacity: 1, y: 0 }}
             viewport={{ once: true }}
             className="text-center"
           >
             <h2 className="text-3xl font-bold mb-4">Deploy with Docker</h2>
             <p className="text-muted-foreground mb-12 max-w-2xl mx-auto">
               Pull official Docker images with hardware-specific optimizations.
               Available for CPU and GPU-accelerated inference.
             </p>

             <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-6 max-w-6xl mx-auto">
               {Object.entries(dockerData.variants).map(([key, variant]) => (
                 <DockerVariantCard
                   key={key}
                   variantKey={key}
                   variant={variant}
                   version={dockerData.version}
                 />
               ))}
             </div>

             <div className="mt-12 p-6 bg-blue-50 rounded-lg max-w-3xl mx-auto">
               <p className="text-sm text-blue-900">
                 <strong>Important:</strong> Running Docker containers requires volume mounting
                 and specific environment variables. See our{' '}
                 <Link
                   href="/docs/deployment/docker"
                   className="text-blue-600 hover:underline font-semibold"
                 >
                   Docker deployment documentation
                 </Link>
                 {' '}for complete setup instructions.
               </p>
             </div>
           </motion.div>
         </Container>
       </section>
     );
   }

   function DockerVariantCard({ variantKey, variant, version }) {
     const [copied, setCopied] = useState(false);
     const metadata = getVariantMetadata(variantKey);

     const handleCopy = () => {
       navigator.clipboard.writeText(variant.pull_command);
       setCopied(true);
       setTimeout(() => setCopied(false), 2000);
     };

     return (
       <Card className="p-6 flex flex-col hover:shadow-lg transition-all">
         <div className="flex items-start justify-between mb-4">
           <div className="flex items-center gap-3">
             {/* Icon based on metadata.icon */}
             <div className={`p-2 rounded-lg bg-${metadata.color}-100`}>
               {metadata.gpuVendor ?
                 <Zap className={`h-6 w-6 text-${metadata.color}-600`} /> :
                 <Cpu className={`h-6 w-6 text-${metadata.color}-600`} />
               }
             </div>
             <div>
               <h3 className="font-semibold text-lg">{metadata.displayName}</h3>
               {variant.gpu_type && (
                 <span className="text-xs bg-violet-100 text-violet-700 px-2 py-1 rounded">
                   {variant.gpu_type} GPU
                 </span>
               )}
             </div>
           </div>
           {metadata.recommended && (
             <span className="text-xs bg-green-100 text-green-700 px-2 py-1 rounded">
               Recommended
             </span>
           )}
         </div>

         <p className="text-sm text-muted-foreground mb-4">
           {metadata.description}
         </p>

         <div className="text-xs text-muted-foreground mb-4">
           Platforms: {variant.platforms.join(', ')}
         </div>

         <div className="mt-auto">
           <div className="relative">
             <code className="block text-xs bg-slate-100 p-3 rounded mb-2 overflow-x-auto">
               {variant.pull_command}
             </code>
             <Button
               size="sm"
               variant="outline"
               className="w-full gap-2"
               onClick={handleCopy}
             >
               <Copy className="h-4 w-4" />
               {copied ? 'Copied!' : 'Copy Command'}
             </Button>
           </div>
         </div>
       </Card>
     );
   }
   ```

5. **Integrate into homepage:**
   - Add DockerSection after DownloadSection
   - Ensure responsive layout
   - Match visual design with existing sections

**Deliverables:**
- `src/lib/docker-variants.ts` with metadata system
- Updated `src/lib/constants.ts` with Docker constants
- `src/app/DockerSection.tsx` component
- Homepage integration
- Responsive design matching existing style

**Validation:**
```bash
cd getbodhi.app
npm run dev
# Navigate to http://localhost:3000
# Verify:
# - Docker section appears after platform downloads
# - All variants render with cards
# - Copy button copies docker pull command
# - Link to /docs/deployment/docker works
# - Badges show correctly (GPU type, Recommended)
```

**Context to Pass Forward:**
- Component locations created
- Data loading pattern used
- UI patterns established
- Copy-to-clipboard implementation

**Agent Log:** `ai-docs/specs/20251009-getbodhi-app/phase2-frontend.log`

---

### Phase 3: Documentation & Testing

**Agent:** general-purpose
**Objective:** Update documentation and validate end-to-end flow

**Prerequisites:**
- Phase 1 completed: Backend automation working
- Phase 2 completed: Frontend components rendering
- Read context from both previous phases

**Tasks:**

1. **Update Docker documentation** (`crates/bodhi/src/docs/deployment/docker.md`):
   - Add section at the top: "Latest Docker Releases"
   - Reference website: "Visit [getbodhi.app](https://getbodhi.app) for the latest Docker image versions"
   - Update "Quick Start" examples to reference website for version numbers
   - Add note: "The website automatically updates when new variants are released"
   - Update variant comparison table if it exists
   - Maintain existing content about volume mounting and environment variables

2. **Update getbodhi.app Makefile** documentation:
   - Add comments to `website.update_releases` target about Docker inclusion
   - Update `release.precheck` comments to mention Docker validation
   - Ensure help text mentions Docker updates

3. **Create comprehensive specs documentation:**

   **a) `ai-docs/specs/20251009-getbodhi-app/SPEC.md`:**
   - Overall design with variant auto-discovery strategy
   - Architecture decisions
   - Data flow diagrams
   - Future extensibility plan

   **b) `ai-docs/specs/20251009-getbodhi-app/IMPLEMENTATION.md`:**
   - Technical details of discovery mechanism
   - Code structure and organization
   - Component hierarchy
   - API integration points

   **c) `ai-docs/specs/20251009-getbodhi-app/TESTING.md`:**
   - Verification steps for each phase
   - End-to-end testing scenarios
   - Regression testing checklist
   - Performance considerations

   **d) `ai-docs/specs/20251009-getbodhi-app/VARIANT_EXTENSION.md`:**
   - Guide for adding new Docker variants
   - Workflow for releases
   - Checklist for variant addition
   - Troubleshooting guide

4. **Create end-to-end testing checklist:**
   - [ ] Run `make website.update_releases`
   - [ ] Verify releases.json contains docker section
   - [ ] Verify all current variants discovered (cpu, cuda, rocm)
   - [ ] Check .env.release_urls has Docker variables
   - [ ] Build website: `npm run build`
   - [ ] Test website locally: `npm run dev`
   - [ ] Verify Docker section renders on homepage
   - [ ] Test copy-to-clipboard functionality
   - [ ] Verify link to Docker docs works
   - [ ] Check responsive design on mobile
   - [ ] Validate docker pull commands are correct
   - [ ] Test with missing variant metadata (fallback)
   - [ ] Verify variant badges display correctly

5. **Create variant extension workflow documentation:**
   ```markdown
   # Adding a New Docker Variant

   When releasing a new Docker variant (e.g., Vulkan, Intel):

   1. **Add to Docker workflow:**
      - Edit `.github/workflows/publish-docker.yml`
      - Add variant to matrix (e.g., `vulkan`)
      - Add platform specification

   2. **Create Dockerfile:**
      - Create `devops/vulkan.Dockerfile`
      - Follow existing variant patterns

   3. **Release Docker image:**
      - Tag release: `docker/vX.Y.Z`
      - Workflow builds all variants
      - Release body includes variant sections

   4. **Update website (automatic):**
      - Run: `make website.update_releases`
      - New variant discovered automatically
      - Appears in releases.json

   5. **Add metadata (optional):**
      - Edit `getbodhi.app/src/lib/docker-variants.ts`
      - Add entry for better display name/description
      - If skipped, uses fallback metadata

   6. **Deploy website:**
      - Commit changes
      - Create website release tag
      - New variant appears automatically
   ```

**Deliverables:**
- Updated `crates/bodhi/src/docs/deployment/docker.md`
- Documented Makefile targets
- Complete specs in `ai-docs/specs/20251009-getbodhi-app/`
- End-to-end testing checklist
- Variant extension guide

**Validation:**
```bash
# End-to-end test
cd getbodhi.app
make update_releases
npm run build
npm run dev

# Check documentation
cat ../crates/bodhi/src/docs/deployment/docker.md | grep -A5 "Latest"

# Verify specs
ls -la ai-docs/specs/20251009-getbodhi-app/
```

**Agent Log:** `ai-docs/specs/20251009-getbodhi-app/phase3-docs.log`

---

## Technical Design Decisions

### 1. Variant Auto-Discovery Strategy
**Decision:** Parse GitHub release body for variant information
**Rationale:**
- `.github/workflows/publish-docker.yml` already generates detailed release body
- Each variant has a markdown section: `### {Variant} Variant`
- Contains platforms, GPU type, docker commands
- No additional API calls needed
- Scales automatically with new variants

### 2. Data Structure
**Decision:** Extend `releases.json` with docker section
**Rationale:**
- Maintains consistency with desktop platform data
- Single source of truth for all releases
- Easier to load and parse in frontend
- Supports versioning and rollback

### 3. Variant Metadata System
**Decision:** Separate metadata file with fallback
**Rationale:**
- Known variants get rich metadata (icons, colors, descriptions)
- Unknown variants use sensible defaults
- Easy to extend without breaking existing functionality
- Separation of concerns (data vs. display)

### 4. UI Component Strategy
**Decision:** Dynamic rendering based on releases.json data
**Rationale:**
- No hardcoded variant lists in UI code
- Grid layout scales automatically (1-2-3 columns)
- Works with current variants (3) and future variants (5+)
- Consistent card design across all variants

### 5. Docker Commands vs Downloads
**Decision:** Show docker pull commands (not download buttons)
**Rationale:**
- Docker images aren't downloaded like desktop apps
- Users need to pull from registry
- Copy-to-clipboard is more useful than fake download
- Matches Docker ecosystem conventions

### 6. Documentation Reference
**Decision:** Link to `/docs/deployment/docker` prominently
**Rationale:**
- Docker setup requires volume mounting, env vars, etc.
- Too complex for homepage to explain fully
- Docs page has comprehensive setup instructions
- Reduces homepage clutter

## Data Flow Architecture

```
┌─────────────────────────────────────────────────┐
│  GitHub Release (docker/v0.0.2)                 │
│  - Tag: docker/v0.0.2                           │
│  - Body: Contains ### Variant sections          │
│  - Created by: publish-docker.yml workflow      │
└─────────────────┬───────────────────────────────┘
                  │
                  ↓
┌─────────────────────────────────────────────────┐
│  update-release-urls.js                         │
│  - Query GitHub Releases API                    │
│  - Find latest docker/v* release                │
│  - Parse body with variant pattern regex        │
│  - Extract: name, platforms, GPU type           │
└─────────────────┬───────────────────────────────┘
                  │
                  ↓
┌─────────────────────────────────────────────────┐
│  releases.json & .env.release_urls              │
│  - docker.variants: { cpu, cuda, rocm, ... }    │
│  - NEXT_PUBLIC_DOCKER_VERSION=0.0.2             │
│  - Each variant: image_tag, pull_command, etc.  │
└─────────────────┬───────────────────────────────┘
                  │
                  ↓
┌─────────────────────────────────────────────────┐
│  DockerSection.tsx Component                    │
│  - Load releases.json                           │
│  - Map variants to cards                        │
│  - Apply metadata from docker-variants.ts       │
│  - Render dynamically                           │
└─────────────────┬───────────────────────────────┘
                  │
                  ↓
┌─────────────────────────────────────────────────┐
│  User Sees: Docker Section                      │
│  - CPU card (Recommended)                       │
│  - CUDA card (NVIDIA GPU badge)                 │
│  - ROCm card (AMD GPU badge)                    │
│  - [Future: Vulkan, Intel cards appear auto]    │
│  - Copy docker pull commands                    │
│  - Link to Docker docs                          │
└─────────────────────────────────────────────────┘
```

## Success Criteria

### Functional Requirements
- ✅ Script auto-discovers ALL variants from Docker releases
- ✅ No hardcoded variant lists (except metadata)
- ✅ Website displays docker pull commands with copy button
- ✅ Link to Docker docs for setup instructions
- ✅ New variants (Vulkan, Intel) appear automatically when released
- ✅ UI scales gracefully with different variant counts
- ✅ Fallback handling for unknown variants

### Non-Functional Requirements
- ✅ Visual design matches existing sections
- ✅ Responsive layout (mobile, tablet, desktop)
- ✅ Fast page load (releases.json cached)
- ✅ Copy-to-clipboard works in all browsers
- ✅ Accessible (ARIA labels, keyboard navigation)

### Documentation Requirements
- ✅ Docker docs reference website for latest versions
- ✅ Variant extension guide complete
- ✅ Testing checklist validated
- ✅ Architecture documented in specs

### Automation Requirements
- ✅ `make website.update_releases` discovers Docker variants
- ✅ `make website.update_releases.check` validates data
- ✅ Release workflow creates properly formatted release body
- ✅ Website build uses latest releases.json

## Future Enhancements

1. **API Endpoint** (Optional):
   - Create `/api/releases` endpoint
   - Return releases.json data
   - Enable client-side dynamic loading
   - Reduce page build size

2. **Version Comparison**:
   - Show changelog between versions
   - Link to GitHub release notes
   - Display release dates

3. **Variant Search/Filter**:
   - Filter by GPU vendor
   - Filter by platform (ARM64 vs AMD64)
   - Search by name

4. **Usage Statistics**:
   - Track most popular variants
   - Show download counts (if available)
   - Display community recommendations

## Context Files for Agent Communication

**Common Context File:** `ai-docs/specs/20251009-getbodhi-app/agent-context.json`

This file will be updated by each agent to pass information forward:

```json
{
  "phase1": {
    "status": "completed|in-progress|pending",
    "releases_json_path": "getbodhi.app/public/releases.json",
    "env_file_path": "getbodhi.app/.env.release_urls",
    "discovered_variants": ["cpu", "cuda", "rocm"],
    "docker_version": "0.0.2",
    "registry_url": "ghcr.io/bodhisearch/bodhiapp",
    "data_structure": {
      "docker": {
        "version": "string",
        "tag": "string",
        "released_at": "ISO date",
        "registry": "string",
        "variants": {
          "{variant_name}": {
            "image_tag": "string",
            "latest_tag": "string",
            "platforms": ["string"],
            "pull_command": "string",
            "gpu_type": "optional string"
          }
        }
      }
    },
    "notes": []
  },
  "phase2": {
    "status": "completed|in-progress|pending",
    "components_created": [
      "src/lib/docker-variants.ts",
      "src/app/DockerSection.tsx"
    ],
    "integration_point": "src/app/page.tsx",
    "metadata_pattern": "VARIANT_METADATA[key] with fallback",
    "copy_implementation": "navigator.clipboard.writeText",
    "notes": []
  },
  "phase3": {
    "status": "completed|in-progress|pending",
    "docs_updated": [
      "crates/bodhi/src/docs/deployment/docker.md"
    ],
    "specs_created": [
      "SPEC.md",
      "IMPLEMENTATION.md",
      "TESTING.md",
      "VARIANT_EXTENSION.md"
    ],
    "testing_complete": true,
    "notes": []
  }
}
```

## Rollout Plan

**Week 1: Phase 1**
- Implement backend infrastructure
- Test variant discovery
- Validate data structure
- No user-facing changes

**Week 2: Phase 2**
- Implement frontend components
- Test UI rendering
- Validate responsive design
- Deploy to staging for review

**Week 3: Phase 3**
- Update documentation
- Complete testing
- Deploy to production
- Monitor for issues

## Risk Mitigation

### Risk 1: GitHub API Rate Limits
**Mitigation:**
- Cache releases.json for 1 hour
- Use authenticated requests (higher limits)
- Fallback to cached data if API fails

### Risk 2: Release Body Format Changes
**Mitigation:**
- Robust regex patterns with fallbacks
- Log parsing errors for debugging
- Manual validation step in release process

### Risk 3: Unknown Variant Display
**Mitigation:**
- Fallback metadata system
- Sensible defaults for all fields
- Graceful degradation of UI

### Risk 4: Docker Documentation Drift
**Mitigation:**
- Reference website as single source of truth
- Automate version updates where possible
- Regular documentation reviews

---

**Implementation Status:** Ready for Phase 1
**Next Step:** Launch Phase 1 agent with this specification
