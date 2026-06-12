# Docker Release Integration - Implementation Details

**Version:** 1.0
**Date:** 2025-10-10
**Status:** Completed

## Overview

This document provides detailed technical implementation information for the Docker release integration feature, including code structure, API contracts, data flows, and integration points.

## File Structure

### Backend (Release Automation)

```
getbodhi.app/
├── scripts/
│   └── update-release-urls.js          # Main release automation script
├── .env.release_urls                   # Generated environment variables
├── public/
│   └── releases.json                   # Generated release data (served statically)
├── Makefile                            # Automation targets
└── package.json                        # NPM scripts
```

### Frontend (Website)

```
getbodhi.app/src/
├── app/
│   ├── page.tsx                        # Homepage (integrates DockerSection)
│   └── DockerSection.tsx               # Main Docker display component
├── lib/
│   ├── docker-variants.ts              # Variant metadata definitions
│   └── constants.ts                    # Environment variable imports
└── components/
    └── ui/                             # Shadcn/ui components (Card, Button, etc.)
```

### Documentation

```
crates/bodhi/src/docs/deployment/
└── docker.md                           # Docker deployment documentation

ai-docs/specs/20251009-getbodhi-app/
├── website-docker-info.md              # Original specification
├── agent-context.json                  # Phase execution context
├── SPEC.md                             # Architecture specification
├── IMPLEMENTATION.md                   # This file
├── TESTING.md                          # Testing procedures
├── VARIANT_EXTENSION.md                # Variant addition guide
├── phase1-backend.log                  # Phase 1 execution log
├── phase2-frontend.log                 # Phase 2 execution log
└── phase3-docs.log                     # Phase 3 execution log
```

## Backend Implementation

### update-release-urls.js

**Purpose:** Fetch latest GitHub releases and generate data files

**Key Functions:**

#### parseDockerVariants(releaseBody, registry, version)

Parses Docker variant information from GitHub release body.

```javascript
function parseDockerVariants(releaseBody, registry, version) {
  const variants = {};

  // Regex to match variant sections: ### <Variant> Variant
  const variantSectionRegex = /###\s+(\w+)\s+Variant[^#]*/gi;
  let match;

  while ((match = variantSectionRegex.exec(releaseBody)) !== null) {
    const variantName = match[1].toLowerCase();
    const sectionText = match[0];

    // Extract platforms (multi-platform or single)
    let platforms = ['linux/amd64']; // default

    const multiPlatformMatch = /Multi-platform:\s*([^)]+)\)/i.exec(sectionText);
    if (multiPlatformMatch && multiPlatformMatch[1].includes('AMD64') &&
        multiPlatformMatch[1].includes('ARM64')) {
      platforms = ['linux/amd64', 'linux/arm64'];
    } else {
      const platformMatch = /Platforms?[:\s]+(linux\/[^\n]+)/i.exec(sectionText);
      if (platformMatch) {
        platforms = platformMatch[1].split(',').map(p => p.trim());
      }
    }

    // Extract GPU type from section text
    let gpuType = null;
    if (/NVIDIA/i.test(sectionText)) {
      gpuType = 'NVIDIA';
    } else if (/AMD GPU/i.test(sectionText)) {
      gpuType = 'AMD';
    } else if (/Intel/i.test(sectionText)) {
      gpuType = 'Intel';
    } else if (/Cross-vendor/i.test(sectionText)) {
      gpuType = 'Cross-vendor';
    }

    // Extract description from header
    let description = '';
    const headerMatch = /###\s+\w+\s+Variant\s*\(([^)]+)\)/i.exec(sectionText);
    if (headerMatch) {
      description = headerMatch[1];
    } else if (gpuType) {
      description = `${gpuType} GPU acceleration`;
    }

    // Build variant object
    variants[variantName] = {
      image_tag: `${version}-${variantName}`,
      latest_tag: `latest-${variantName}`,
      platforms: platforms,
      pull_command: `docker pull ${registry}:${version}-${variantName}`,
      ...(gpuType && { gpu_type: gpuType }),
      ...(description && { description: description }),
    };
  }

  return variants;
}
```

**Pattern Matching Logic:**

1. **Variant Name:** Captured from `### {Name} Variant`
2. **Platforms:**
   - Look for `Multi-platform: AMD64 + ARM64`
   - Fallback to `Platforms: linux/amd64, linux/arm64`
   - Default to `['linux/amd64']`
3. **GPU Type:** Search for keywords: NVIDIA, AMD GPU, Intel, Cross-vendor
4. **Description:** Extract from header parentheses or generate from GPU type

**Data Extracted:**
- `variantName` (key): lowercase variant identifier
- `image_tag`: Version-specific tag (e.g., "0.0.2-cpu")
- `latest_tag`: Latest tag (e.g., "latest-cpu")
- `platforms`: Array of supported platforms
- `pull_command`: Complete docker pull command
- `gpu_type` (optional): GPU vendor
- `description` (optional): Human-readable description

#### fetchLatestReleases()

Fetches releases from GitHub API and processes them.

```javascript
async function fetchLatestReleases() {
  const found = {};
  let desktopMetadata = null;
  let dockerMetadata = null;
  const sixMonthsAgo = new Date();
  sixMonthsAgo.setMonth(sixMonthsAgo.getMonth() - 6);

  let page = 1;
  let shouldContinue = true;

  // Paginate through releases
  while (shouldContinue) {
    const { data: releases } = await octokit.repos.listReleases({
      owner: OWNER,
      repo: REPO,
      per_page: 100,
      page,
    });

    if (releases.length === 0) break;

    for (const release of releases) {
      const releaseDate = new Date(release.created_at);

      // Stop if older than 6 months
      if (releaseDate < sixMonthsAgo) {
        shouldContinue = false;
        break;
      }

      // Check tag patterns
      for (const pattern of TAG_PATTERNS) {
        if (pattern.regex.test(release.tag_name)) {
          // Handle Docker patterns
          if (pattern.type === 'docker') {
            if (!dockerMetadata) {
              const versionMatch = release.tag_name.match(/v([\d.]+)$/);
              const version = versionMatch ? versionMatch[1] : release.tag_name;
              const variants = parseDockerVariants(
                release.body || '',
                pattern.registry,
                version
              );

              dockerMetadata = {
                version: version,
                tag: release.tag_name,
                released_at: release.published_at || release.created_at,
                registry: pattern.registry,
                variants: variants,
              };

              found.docker = dockerMetadata;
              console.log(`✓ Found Docker release: ${release.tag_name}`);
              console.log(`  Variants discovered: ${Object.keys(variants).join(', ')}`);
            }
          }
          // Handle desktop patterns...
        }
      }

      // Check if we found everything and exit early
      const desktopFound = /* ... */;
      const dockerFound = dockerMetadata !== null;

      if (desktopFound && dockerFound) {
        shouldContinue = false;
        break;
      }
    }

    if (releases.length < 100) break;
    page++;
  }

  return { found, desktopMetadata, dockerMetadata };
}
```

**Search Strategy:**
1. Paginate through releases (100 per page)
2. Stop at 6-month limit to avoid excessive API calls
3. Match tag patterns: `docker/v*` for Docker releases
4. Parse first matching release only
5. Exit early when all patterns found

#### generateReleasesJson(data, desktopMetadata, dockerMetadata, dryRun)

Generates the public/releases.json file.

```javascript
function generateReleasesJson(data, desktopMetadata, dockerMetadata, dryRun) {
  const releasesData = {};

  // Build desktop platform structure
  if (desktopMetadata) {
    const platforms = {};
    for (const value of Object.values(data)) {
      if (value.platformKey && value.archKey) {
        if (!platforms[value.platformKey]) {
          platforms[value.platformKey] = {};
        }
        platforms[value.platformKey][value.archKey] = {
          download_url: value.url,
          filename: value.filename,
        };
      }
    }

    releasesData.desktop = {
      version: desktopMetadata.version,
      tag: desktopMetadata.tag,
      released_at: desktopMetadata.released_at,
      platforms,
    };
  }

  // Add Docker data
  if (dockerMetadata) {
    releasesData.docker = {
      version: dockerMetadata.version,
      tag: dockerMetadata.tag,
      released_at: dockerMetadata.released_at,
      registry: dockerMetadata.registry,
      variants: dockerMetadata.variants,
    };
  }

  const content = JSON.stringify(releasesData, null, 2) + '\n';

  if (dryRun) {
    console.log('\n=== Dry-run mode - would write to public/releases.json: ===\n');
    console.log(content);
    return;
  }

  if (!fs.existsSync('public')) {
    fs.mkdirSync('public', { recursive: true });
  }

  fs.writeFileSync('public/releases.json', content);
  console.log('\n✓ Updated public/releases.json');
}
```

**Output Structure:**
- Nested structure: `desktop` and `docker` top-level keys
- Docker section includes registry and variants
- Pretty-printed JSON with 2-space indent
- Newline at end of file for git cleanliness

### TAG_PATTERNS Configuration

Defines which releases to search for.

```javascript
const TAG_PATTERNS = [
  {
    regex: /^app\/v/,
    type: 'desktop',
    platforms: [
      {
        id: 'macos',
        assetPattern: /Bodhi[\s.]App.*\.dmg$/,
        envVar: 'NEXT_PUBLIC_DOWNLOAD_URL_MACOS',
        platformKey: 'macos',
        archKey: 'silicon',
      },
      // ... windows, linux
    ],
  },
  {
    regex: /^docker\/v/,
    type: 'docker',
    registry: 'ghcr.io/bodhisearch/bodhiapp',
  },
  // Future patterns can be added here
];
```

**Extensibility:**
- Add new patterns for other release types
- Each pattern specifies regex, type, and processing logic
- Supports both asset-based releases (desktop) and metadata-based (Docker)

## Frontend Implementation

### DockerSection.tsx

Main component for displaying Docker variants.

```tsx
'use client';

import { Container } from '@/components/ui/container';
import { Card } from '@/components/ui/card';
import { Button } from '@/components/ui/button';
import { Copy, Cpu, Zap, CheckCircle2 } from 'lucide-react';
import Link from 'next/link';
import { motion } from 'framer-motion';
import { getVariantMetadata } from '@/lib/docker-variants';
import { useState, useEffect } from 'react';

interface DockerVariant {
  image_tag: string;
  latest_tag: string;
  platforms: string[];
  pull_command: string;
  gpu_type?: string;
  description?: string;
}

interface DockerData {
  version: string;
  tag: string;
  released_at: string;
  registry: string;
  variants: Record<string, DockerVariant>;
}

interface ReleasesData {
  docker: DockerData;
}

export function DockerSection() {
  const [dockerData, setDockerData] = useState<DockerData | null>(null);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    fetch('/releases.json')
      .then((res) => {
        if (!res.ok) throw new Error('Failed to load releases data');
        return res.json();
      })
      .then((data: ReleasesData) => {
        setDockerData(data.docker);
        setLoading(false);
      })
      .catch((err) => {
        console.error('Error loading Docker data:', err);
        setError('Failed to load Docker release information');
        setLoading(false);
      });
  }, []);

  if (loading) {
    return (
      <section id="docker-section" className="py-20 bg-gradient-to-b from-white to-slate-50">
        <Container>
          <div className="text-center text-muted-foreground">
            Loading Docker releases...
          </div>
        </Container>
      </section>
    );
  }

  if (error || !dockerData) {
    return (
      <section id="docker-section" className="py-20 bg-gradient-to-b from-white to-slate-50">
        <Container>
          <div className="text-center text-muted-foreground">
            {error || 'No Docker data available'}
          </div>
        </Container>
      </section>
    );
  }

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
            {Object.entries(dockerData.variants).map(([key, variant], index) => (
              <motion.div
                key={key}
                initial={{ opacity: 0, y: 20 }}
                whileInView={{ opacity: 1, y: 0 }}
                viewport={{ once: true }}
                transition={{ delay: 0.1 * index }}
              >
                <DockerVariantCard variantKey={key} variant={variant} />
              </motion.div>
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
              </Link>{' '}
              for complete setup instructions.
            </p>
          </div>
        </motion.div>
      </Container>
    </section>
  );
}
```

**Component Structure:**
- Client component (requires hooks and interactivity)
- Data fetching in useEffect on mount
- Loading and error states with graceful fallbacks
- Framer Motion animations for smooth appearance
- Responsive grid layout
- Documentation link at bottom

**State Management:**
- `dockerData`: Loaded Docker release data
- `loading`: Loading state for skeleton/spinner
- `error`: Error message if fetch fails

**Data Loading:**
- Fetches `/releases.json` from public directory
- Runs once on component mount
- Parses JSON response
- Handles fetch errors gracefully

### DockerVariantCard Component

Individual variant card with copy functionality.

```tsx
function DockerVariantCard({
  variantKey,
  variant,
}: {
  variantKey: string;
  variant: DockerVariant;
}) {
  const [copied, setCopied] = useState(false);
  const metadata = getVariantMetadata(variantKey);

  const handleCopy = async () => {
    try {
      await navigator.clipboard.writeText(variant.pull_command);
      setCopied(true);
      setTimeout(() => setCopied(false), 2000);
    } catch (err) {
      console.error('Failed to copy:', err);
    }
  };

  const colorClasses = {
    blue: { bg: 'bg-blue-100', text: 'text-blue-600' },
    green: { bg: 'bg-green-100', text: 'text-green-600' },
    red: { bg: 'bg-red-100', text: 'text-red-600' },
    purple: { bg: 'bg-purple-100', text: 'text-purple-600' },
    indigo: { bg: 'bg-indigo-100', text: 'text-indigo-600' },
    gray: { bg: 'bg-gray-100', text: 'text-gray-600' },
  };

  const colors = colorClasses[metadata.color as keyof typeof colorClasses] || colorClasses.gray;

  return (
    <Card className="p-6 flex flex-col hover:shadow-lg transition-all h-full">
      {/* Header with icon and badges */}
      <div className="flex items-start justify-between mb-4">
        <div className="flex items-center gap-3">
          <div className={`p-2 rounded-lg ${colors.bg}`}>
            {metadata.gpuVendor ? (
              <Zap className={`h-6 w-6 ${colors.text}`} />
            ) : (
              <Cpu className={`h-6 w-6 ${colors.text}`} />
            )}
          </div>
          <div>
            <h3 className="font-semibold text-lg">{metadata.displayName}</h3>
            {variant.gpu_type && (
              <span className="text-xs bg-violet-100 text-violet-700 px-2 py-1 rounded mt-1 inline-block">
                {variant.gpu_type} GPU
              </span>
            )}
          </div>
        </div>
        {metadata.recommended && (
          <span className="text-xs bg-green-100 text-green-700 px-2 py-1 rounded flex items-center gap-1">
            <CheckCircle2 className="h-3 w-3" />
            Recommended
          </span>
        )}
      </div>

      {/* Description and platforms */}
      <p className="text-sm text-muted-foreground mb-4">
        {metadata.description}
      </p>

      <div className="text-xs text-muted-foreground mb-4">
        <span className="font-medium">Platforms:</span> {variant.platforms.join(', ')}
      </div>

      {/* Pull command and copy button */}
      <div className="mt-auto">
        <div className="relative">
          <code className="block text-xs bg-slate-100 p-3 rounded mb-2 overflow-x-auto whitespace-nowrap">
            {variant.pull_command}
          </code>
          <Button size="sm" variant="outline" className="w-full gap-2" onClick={handleCopy}>
            {copied ? (
              <>
                <CheckCircle2 className="h-4 w-4 text-green-600" />
                Copied!
              </>
            ) : (
              <>
                <Copy className="h-4 w-4" />
                Copy Command
              </>
            )}
          </Button>
        </div>
      </div>
    </Card>
  );
}
```

**Card Layout:**
1. **Header:** Icon (CPU/GPU), variant name, GPU badge
2. **Badges:** Recommended badge (if applicable)
3. **Description:** From metadata or release body
4. **Platforms:** List of supported platforms
5. **Pull Command:** Code block with command
6. **Copy Button:** With visual feedback

**Icon Selection:**
- CPU variants: Cpu icon (chip symbol)
- GPU variants: Zap icon (lightning bolt)
- Color from metadata (blue, green, red, purple, indigo)

**Copy Interaction:**
1. User clicks "Copy Command"
2. Async clipboard write
3. Button text changes to "Copied!" with check icon
4. Auto-revert after 2 seconds
5. Error handling if clipboard API fails

### docker-variants.ts

Metadata definitions for known variants.

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
    color: 'blue',
  },
  cuda: {
    name: 'cuda',
    displayName: 'CUDA',
    description: 'NVIDIA GPU acceleration (8-12x faster)',
    icon: 'gpu-nvidia',
    gpuVendor: 'NVIDIA',
    color: 'green',
  },
  rocm: {
    name: 'rocm',
    displayName: 'ROCm',
    description: 'AMD GPU acceleration',
    icon: 'gpu-amd',
    gpuVendor: 'AMD',
    color: 'red',
  },
  vulkan: {
    name: 'vulkan',
    displayName: 'Vulkan',
    description: 'Cross-vendor GPU acceleration',
    icon: 'gpu-generic',
    gpuVendor: 'Cross-vendor',
    color: 'purple',
  },
  intel: {
    name: 'intel',
    displayName: 'Intel',
    description: 'Intel GPU acceleration',
    icon: 'gpu-generic',
    gpuVendor: 'Intel',
    color: 'indigo',
  },
};

export function getVariantMetadata(variantKey: string): DockerVariantMetadata {
  return (
    VARIANT_METADATA[variantKey] || {
      name: variantKey,
      displayName: variantKey.toUpperCase(),
      description: `${variantKey} variant`,
      icon: 'cpu',
      color: 'gray',
    }
  );
}
```

**Metadata Fields:**
- `name`: Internal key (lowercase)
- `displayName`: User-facing name
- `description`: Explanation of variant purpose
- `icon`: Icon type for visual identification
- `gpuVendor`: GPU vendor for badge display
- `recommended`: Show recommendation badge
- `color`: Tailwind color for theming

**Fallback Behavior:**
- Unknown variants get auto-generated metadata
- Name capitalized for display
- Generic CPU icon and gray color
- Description auto-generated
- No recommendation badge

### Integration in Homepage

```tsx
// src/app/page.tsx
import { DockerSection } from './DockerSection';

export default function HomePage() {
  return (
    <div className="flex flex-col">
      <HeroSection />
      <FeaturesSection />
      <DownloadSection />
      <DockerSection />  {/* Added after DownloadSection */}
      <CTASection />
      <Footer />
    </div>
  );
}
```

**Placement:**
- After DownloadSection (desktop platforms)
- Before CTASection (call-to-action)
- Part of main page flow
- Consistent spacing and styling

## Data Contracts

### releases.json Schema

```typescript
interface ReleasesData {
  desktop: {
    version: string;              // e.g., "0.0.31"
    tag: string;                  // e.g., "app/v0.0.31"
    released_at: string;          // ISO-8601 timestamp
    platforms: {
      [platformKey: string]: {    // "macos", "windows", "linux"
        [archKey: string]: {      // "silicon", "x64"
          download_url: string;
          filename: string;
        };
      };
    };
  };
  docker: {
    version: string;              // e.g., "0.0.2"
    tag: string;                  // e.g., "docker/v0.0.2"
    released_at: string;          // ISO-8601 timestamp
    registry: string;             // e.g., "ghcr.io/bodhisearch/bodhiapp"
    variants: {
      [variantKey: string]: {     // "cpu", "cuda", "rocm", "vulkan", ...
        image_tag: string;        // e.g., "0.0.2-cpu"
        latest_tag: string;       // e.g., "latest-cpu"
        platforms: string[];      // e.g., ["linux/amd64", "linux/arm64"]
        pull_command: string;     // e.g., "docker pull ghcr.io/..."
        gpu_type?: string;        // Optional: "NVIDIA", "AMD", "Intel", "Cross-vendor"
        description?: string;     // Optional: Human-readable description
      };
    };
  };
}
```

**Key Characteristics:**
- Top-level keys: `desktop` and `docker`
- Parallel structure for consistency
- Independent versioning
- Docker variants keyed by name (lowercase)
- Optional fields for graceful evolution

### Environment Variables

Generated in `.env.release_urls`:

```bash
# Desktop app version and tag
NEXT_PUBLIC_APP_VERSION=0.0.31
NEXT_PUBLIC_APP_TAG=app/v0.0.31

# Docker version and tag
NEXT_PUBLIC_DOCKER_VERSION=0.0.2
NEXT_PUBLIC_DOCKER_TAG=docker/v0.0.2
NEXT_PUBLIC_DOCKER_REGISTRY=ghcr.io/bodhisearch/bodhiapp

# Platform download URLs
NEXT_PUBLIC_DOWNLOAD_URL_MACOS=https://github.com/...
NEXT_PUBLIC_DOWNLOAD_URL_WINDOWS=https://github.com/...
NEXT_PUBLIC_DOWNLOAD_URL_LINUX=https://github.com/...
```

**Usage:**
- Loaded by Next.js at build time
- Available as `process.env.NEXT_PUBLIC_*`
- Used in components and build process
- Checked into version control

## Build and Deployment

### NPM Scripts

```json
{
  "scripts": {
    "update_releases": "node scripts/update-release-urls.js",
    "update_releases:check": "node scripts/update-release-urls.js --check",
    "build": "next build",
    "dev": "next dev"
  }
}
```

### Makefile Targets

```makefile
update_releases: ## Update .env.release_urls and releases.json from latest GitHub releases
	@npm run update_releases

update_releases.check: ## Check latest releases (dry-run)
	@npm run update_releases:check

build: ## Build website (auto-syncs docs first via prebuild)
	@npm run build
```

### Workflow Integration

**Release Process:**
1. Docker images released with `docker/v*` tag
2. GitHub workflow generates release body with variant sections
3. Developer runs `make update_releases` locally
4. Script fetches latest release and parses variants
5. Updates `.env.release_urls` and `public/releases.json`
6. Developer commits changes
7. Website deployed with updated data

**CI/CD Integration:**
```bash
# Pre-release checks
make update_releases.check  # Validates latest releases

# Website build
make build  # Builds with latest releases.json
```

## Error Handling

### Backend Errors

**GitHub API Errors:**
```javascript
try {
  const { data: releases } = await octokit.repos.listReleases({ ... });
} catch (error) {
  console.error('\n✗ Error:', error.message);
  if (error.response) {
    console.error('  GitHub API response:', error.response.status);
  }
  process.exit(1);
}
```

**Parsing Errors:**
```javascript
// No variants found
if (Object.keys(variants).length === 0) {
  console.warn(`⚠ No variants found in release ${tag}`);
}

// Missing platform info
if (!platformMatch) {
  platforms = ['linux/amd64']; // Default fallback
}
```

### Frontend Errors

**Fetch Errors:**
```tsx
fetch('/releases.json')
  .then((res) => {
    if (!res.ok) throw new Error('Failed to load releases data');
    return res.json();
  })
  .catch((err) => {
    console.error('Error loading Docker data:', err);
    setError('Failed to load Docker release information');
    setLoading(false);
  });
```

**Missing Data:**
```tsx
if (error || !dockerData) {
  return (
    <div className="text-center text-muted-foreground">
      {error || 'No Docker data available'}
    </div>
  );
}
```

**Clipboard Errors:**
```tsx
try {
  await navigator.clipboard.writeText(variant.pull_command);
  setCopied(true);
} catch (err) {
  console.error('Failed to copy:', err);
  // User sees no feedback, but doesn't break UI
}
```

## Performance Considerations

### Backend Performance

**GitHub API:**
- Pagination: 100 releases per page
- Early exit: Stop when all patterns found
- Time limit: 6-month cutoff to avoid excessive pagination
- Rate limits: Use authenticated requests for higher limits

**File I/O:**
- Synchronous writes (acceptable for build scripts)
- Pretty-printed JSON for human readability
- Minimal file writes (only .env and JSON)

### Frontend Performance

**Data Loading:**
- Static JSON file (CDN-cacheable)
- Single fetch on component mount
- No polling or repeated fetches
- Optimistic rendering after load

**Rendering:**
- Client component for interactivity
- Lazy animations with Framer Motion
- Staggered card animations (0.1s delay)
- No layout shift (skeleton in loading state)

**Bundle Size:**
- Lucide icons (tree-shakeable)
- Tailwind CSS (purged unused classes)
- Minimal JavaScript for copy functionality

## Testing Integration Points

### Backend Testing

```bash
# Dry-run mode (no file writes)
npm run update_releases:check

# Validate JSON structure
cat public/releases.json | jq '.docker.variants | keys'

# Verify environment variables
cat .env.release_urls | grep DOCKER
```

### Frontend Testing

```bash
# Build website
npm run build

# Start dev server
npm run dev

# Navigate to http://localhost:3000
# Verify Docker section renders
# Test copy-to-clipboard functionality
# Check responsive layout
```

### Integration Testing

```bash
# Full workflow
cd getbodhi.app
make update_releases
make build

# Verify outputs
ls -la public/releases.json
cat .env.release_urls
```

## Monitoring and Debugging

### Backend Logging

```javascript
console.log('Fetching releases from GitHub...');
console.log(`✓ Found Docker release: ${release.tag_name}`);
console.log(`  Variants discovered: ${Object.keys(variants).join(', ')}`);
console.log('\n✓ Updated public/releases.json');
console.log('  Docker variants:', variants.length > 0 ? variants.join(', ') : 'none');
```

### Frontend Logging

```tsx
console.error('Error loading Docker data:', err);
console.error('Failed to copy:', err);
```

### Debug Commands

```bash
# Check releases.json structure
cat getbodhi.app/public/releases.json | jq '.'

# Check specific variant
cat getbodhi.app/public/releases.json | jq '.docker.variants.cpu'

# Verify environment variables
grep DOCKER getbodhi.app/.env.release_urls

# Test script without writing
npm run update_releases:check
```

## Code Quality

### TypeScript Type Safety

**Interfaces:**
```typescript
interface DockerVariant {
  image_tag: string;
  latest_tag: string;
  platforms: string[];
  pull_command: string;
  gpu_type?: string;
  description?: string;
}
```

**Type Guards:**
```typescript
if (!res.ok) throw new Error('Failed to load releases data');
```

**Strict Null Checks:**
```typescript
if (error || !dockerData) {
  return <ErrorState />;
}
```

### Code Organization

**Separation of Concerns:**
- Backend: Data fetching and file generation
- Frontend: Display and interaction
- Metadata: Variant presentation information

**Modularity:**
- `parseDockerVariants()`: Parsing logic
- `fetchLatestReleases()`: GitHub API interaction
- `generateReleasesJson()`: File generation
- `DockerVariantCard`: Individual variant display
- `getVariantMetadata()`: Metadata lookup with fallback

### Error Handling Patterns

**Try-Catch:**
```javascript
try {
  await navigator.clipboard.writeText(command);
  setCopied(true);
} catch (err) {
  console.error('Failed to copy:', err);
}
```

**Promise Rejection:**
```typescript
fetch('/releases.json')
  .then(/* ... */)
  .catch(err => setError(err.message));
```

**Early Returns:**
```tsx
if (loading) return <LoadingState />;
if (error) return <ErrorState />;
return <SuccessState />;
```

## Summary

This implementation provides a robust, extensible system for automatically discovering and displaying Docker variants. Key technical achievements:

- **Backend:** Regex-based parsing with fallback defaults
- **Frontend:** Dynamic rendering with graceful error handling
- **Data:** Type-safe contracts with optional fields
- **UX:** Copy-to-clipboard with visual feedback
- **Performance:** Static JSON with CDN caching
- **Extensibility:** New variants work without code changes

The architecture follows established Next.js patterns while introducing Docker-specific optimizations for the unique requirements of container deployments.
