# Docker Release Integration - Architecture Specification

**Version:** 1.0
**Date:** 2025-10-10
**Status:** Completed

## Executive Summary

This specification documents the architecture and design decisions for integrating Docker release information into the getbodhi.app website. The implementation enables automatic discovery and display of all Docker variants without requiring code changes when new variants are released.

## Design Philosophy

### Core Principles

1. **Auto-Discovery**: New Docker variants should appear automatically when released, without manual configuration
2. **Single Source of Truth**: GitHub releases contain all variant metadata; website reflects this data
3. **Extensibility**: Architecture supports unlimited variants with graceful fallbacks
4. **Consistency**: Follows existing patterns used for desktop platform releases
5. **User-Centric**: Prioritizes ease of use with copy-to-clipboard commands and clear documentation links

### Key Architectural Decisions

#### 1. Variant Discovery Strategy

**Decision:** Parse GitHub release body for variant sections using regex patterns

**Rationale:**
- GitHub workflow already generates detailed release body with variant sections
- Each variant documented as `### {Variant} Variant` with metadata
- No additional API calls or infrastructure needed
- Scales automatically with new variants
- Human-readable release notes serve dual purpose

**Implementation:**
```javascript
// Pattern matches: ### CPU Variant, ### CUDA Variant, etc.
const variantSectionRegex = /###\s+(\w+)\s+Variant[^#]*/gi;
```

**Trade-offs:**
- ✅ Zero additional infrastructure
- ✅ Self-documenting releases
- ✅ Works with existing workflow
- ⚠️ Depends on consistent release body format
- ⚠️ Requires regex maintenance for format changes

#### 2. Data Structure Design

**Decision:** Extend `releases.json` with parallel docker section alongside desktop

**Rationale:**
- Maintains consistency with existing desktop platform structure
- Single JSON file simplifies frontend data loading
- Supports independent versioning of desktop and Docker releases
- Easy to cache and distribute via CDN

**Structure:**
```json
{
  "desktop": { /* platform releases */ },
  "docker": {
    "version": "0.0.2",
    "tag": "docker/v0.0.2",
    "released_at": "ISO-8601 timestamp",
    "registry": "ghcr.io/bodhisearch/bodhiapp",
    "variants": {
      "{variant-name}": {
        "image_tag": "version-variant",
        "latest_tag": "latest-variant",
        "platforms": ["linux/amd64", "linux/arm64"],
        "pull_command": "docker pull ...",
        "gpu_type": "optional GPU vendor",
        "description": "optional description"
      }
    }
  }
}
```

**Benefits:**
- Static JSON file enables CDN caching
- Type-safe TypeScript interfaces
- Supports any number of variants
- Optional fields enable graceful evolution

#### 3. Metadata System with Fallbacks

**Decision:** Separate metadata file with automatic fallback for unknown variants

**Rationale:**
- Known variants get rich display metadata (icons, colors, descriptions)
- Unknown variants automatically render with sensible defaults
- Enables future variants without frontend changes
- Separation of data vs presentation concerns

**Architecture:**
```typescript
// Known variant metadata (curated)
VARIANT_METADATA[variantKey] = {
  displayName, description, icon, color, gpuVendor
}

// Automatic fallback for unknown variants
getVariantMetadata(key) {
  return VARIANT_METADATA[key] || generateFallback(key);
}
```

**Extensibility:**
- New variants discovered automatically from release
- Render with default metadata initially
- Add custom metadata later for better UX
- No breaking changes to existing variants

#### 4. Frontend Rendering Strategy

**Decision:** Dynamic React component with client-side data loading

**Rationale:**
- Loads releases.json at runtime via fetch
- Maps over variants dynamically with no hardcoded lists
- Grid layout scales automatically (1-2-3 columns responsive)
- Framer Motion animations stagger based on array length

**Component Architecture:**
```
DockerSection (container)
├── Fetches releases.json
├── Handles loading/error states
└── Maps variants → DockerVariantCard[]
    ├── Variant metadata lookup
    ├── Copy-to-clipboard interaction
    ├── Platform badges
    └── GPU type badges
```

**Responsiveness:**
- Mobile: 1 column
- Tablet: 2 columns
- Desktop: 3 columns
- 4+ variants: Grid wraps naturally

#### 5. User Experience Patterns

**Decision:** Docker pull commands with copy-to-clipboard (not download buttons)

**Rationale:**
- Docker images aren't "downloaded" like desktop apps
- Users need to pull from registry
- Copy-to-clipboard is more useful than fake download
- Matches Docker ecosystem conventions
- Reduces confusion about Docker deployment workflow

**UX Flow:**
1. User sees variant cards with descriptions
2. Clicks "Copy Command" button
3. Visual feedback confirms copy (2s)
4. Pastes into terminal
5. Prominent link to docs for volume mounting and env vars

## System Architecture

### Data Flow Diagram

```
┌─────────────────────────────────────────┐
│  Developer: Create Docker Release       │
│  - Tag: docker/v0.0.2                   │
│  - Workflow: publish-docker.yml         │
│  - Generates release body with variants │
└────────────────┬────────────────────────┘
                 │
                 ↓
┌─────────────────────────────────────────┐
│  GitHub Release (docker/v0.0.2)         │
│  Release Body:                          │
│    ### CPU Variant                      │
│    Multi-platform: AMD64 + ARM64        │
│                                         │
│    ### CUDA Variant                     │
│    NVIDIA GPU acceleration              │
│                                         │
│    ### ROCm Variant                     │
│    AMD GPU acceleration                 │
│                                         │
│    ### Vulkan Variant                   │
│    Cross-vendor GPU acceleration        │
└────────────────┬────────────────────────┘
                 │
                 ↓
┌─────────────────────────────────────────┐
│  Backend: update-release-urls.js        │
│  - Fetch latest docker/v* release       │
│  - Regex parse variant sections         │
│  - Extract: name, platforms, GPU type   │
│  - Build variant objects dynamically    │
└────────────────┬────────────────────────┘
                 │
                 ↓
┌─────────────────────────────────────────┐
│  Generated Files:                       │
│  - .env.release_urls                    │
│    NEXT_PUBLIC_DOCKER_VERSION=0.0.2     │
│    NEXT_PUBLIC_DOCKER_TAG=...           │
│                                         │
│  - public/releases.json                 │
│    docker.variants: { cpu, cuda, ... }  │
└────────────────┬────────────────────────┘
                 │
                 ↓
┌─────────────────────────────────────────┐
│  Frontend: DockerSection.tsx            │
│  - Fetch /releases.json at runtime      │
│  - Map variants → cards dynamically     │
│  - Apply metadata with fallbacks        │
│  - Render grid (responsive, animated)   │
└────────────────┬────────────────────────┘
                 │
                 ↓
┌─────────────────────────────────────────┐
│  User Experience:                       │
│  ┌─────────┐ ┌─────────┐ ┌─────────┐  │
│  │   CPU   │ │  CUDA   │ │  ROCm   │  │
│  │ (blue)  │ │ (green) │ │  (red)  │  │
│  └─────────┘ └─────────┘ └─────────┘  │
│  ┌─────────┐                           │
│  │ Vulkan  │                           │
│  │(purple) │                           │
│  └─────────┘                           │
│                                         │
│  Each card: pull command + copy button │
│  Documentation link at bottom          │
└─────────────────────────────────────────┘
```

### Component Relationships

```
Backend Layer:
┌──────────────────────────────────────┐
│ update-release-urls.js               │
│ ├── GitHub API (Octokit)             │
│ ├── Regex Parser (variant sections)  │
│ └── File Writers                     │
│     ├── .env.release_urls            │
│     └── public/releases.json         │
└──────────────────────────────────────┘

Data Layer:
┌──────────────────────────────────────┐
│ releases.json                        │
│ ├── desktop: {...}                   │
│ └── docker:                          │
│     ├── version                      │
│     ├── registry                     │
│     └── variants: { ... }            │
└──────────────────────────────────────┘

Frontend Layer:
┌──────────────────────────────────────┐
│ DockerSection.tsx                    │
│ ├── Data Fetching (useEffect)        │
│ ├── Loading State                    │
│ ├── Error Handling                   │
│ └── Variant Rendering                │
│     ├── DockerVariantCard[]          │
│     │   ├── Metadata Lookup          │
│     │   ├── Icon Selection           │
│     │   ├── Badge Display            │
│     │   └── Copy Button              │
│     └── Documentation Link           │
└──────────────────────────────────────┘

Metadata Layer:
┌──────────────────────────────────────┐
│ docker-variants.ts                   │
│ ├── VARIANT_METADATA (curated)       │
│ │   ├── cpu: { blue, recommended }  │
│ │   ├── cuda: { green, NVIDIA }     │
│ │   ├── rocm: { red, AMD }          │
│ │   ├── vulkan: { purple, Cross }   │
│ │   └── intel: { indigo, Intel }    │
│ └── getVariantMetadata() (fallback)  │
└──────────────────────────────────────┘
```

## Design Patterns

### 1. Variant Auto-Discovery Pattern

**Problem:** How to automatically detect and display new Docker variants without code changes?

**Solution:** Parse structured release body with regex, extract metadata dynamically

**Implementation:**
1. GitHub workflow writes variant sections to release body
2. Script uses regex to find all `### {Variant} Variant` sections
3. Extract platforms, GPU type, description from section text
4. Build variant object with computed fields (pull_command, image_tag)
5. Store in releases.json for frontend consumption

**Benefits:**
- Zero-configuration variant addition
- Self-documenting releases
- Scales to unlimited variants

### 2. Graceful Fallback Pattern

**Problem:** How to display unknown variants without breaking the UI?

**Solution:** Metadata system with automatic fallback generation

**Implementation:**
```typescript
function getVariantMetadata(variantKey: string) {
  // Try known metadata first
  if (VARIANT_METADATA[variantKey]) {
    return VARIANT_METADATA[variantKey];
  }

  // Generate fallback for unknown variants
  return {
    name: variantKey,
    displayName: variantKey.toUpperCase(),
    description: `${variantKey} variant`,
    icon: 'cpu',
    color: 'gray'
  };
}
```

**Benefits:**
- Unknown variants render correctly
- No null/undefined errors
- Graceful degradation of UX
- Time to add better metadata later

### 3. Dynamic Grid Layout Pattern

**Problem:** How to layout variable number of variants responsively?

**Solution:** CSS Grid with responsive columns and natural wrapping

**Implementation:**
```tsx
<div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-6">
  {Object.entries(variants).map(([key, variant]) => (
    <VariantCard key={key} {...variant} />
  ))}
</div>
```

**Behavior:**
- 3 variants: 3-column row
- 4 variants: 3 + 1 with natural wrap
- 5+ variants: Multi-row grid
- Mobile: All stack to 1 column
- Tablet: 2 columns

### 4. Copy-to-Clipboard with Feedback Pattern

**Problem:** How to provide clear feedback for copy action?

**Solution:** Async clipboard API with visual state management

**Implementation:**
```typescript
const [copied, setCopied] = useState(false);

const handleCopy = async () => {
  await navigator.clipboard.writeText(command);
  setCopied(true);
  setTimeout(() => setCopied(false), 2000);
};
```

**UX:**
- Default: "Copy Command" with Copy icon
- Active: "Copied!" with CheckCircle icon (green)
- Auto-reset after 2 seconds
- Error handling for clipboard API failures

## Technology Choices

### Backend (Node.js Script)

**Choice:** Node.js with Octokit GitHub API client

**Why:**
- Consistent with Next.js website ecosystem
- Octokit provides robust GitHub API access
- JavaScript regex for release body parsing
- Easy file I/O for .env and JSON generation

**Alternatives Considered:**
- Python script: Inconsistent with Next.js stack
- Bash script: Complex regex, poor error handling
- GitHub Actions only: Would require custom action

### Data Format (JSON)

**Choice:** Static JSON file served from public/ directory

**Why:**
- Type-safe with TypeScript interfaces
- CDN-cacheable for fast loading
- Version-controllable for history
- Standard web format, universal support

**Alternatives Considered:**
- API endpoint: Adds server complexity, unnecessary
- GraphQL: Over-engineered for simple data
- YAML: Not natively supported in browsers

### Frontend (React + TypeScript)

**Choice:** React functional components with hooks

**Why:**
- Consistent with Next.js 14 App Router
- Type safety with TypeScript interfaces
- useState for client-side state management
- useEffect for data fetching at mount

**Alternatives Considered:**
- Server components: Need client interactivity (copy button)
- Static generation: Want runtime data loading
- Class components: Deprecated pattern

### UI Library (Shadcn/ui + Tailwind)

**Choice:** Shadcn/ui components with Tailwind CSS

**Why:**
- Consistent with existing website design system
- Accessible components out of box
- Responsive utilities for grid layout
- Easy color customization for variant badges

**Alternatives Considered:**
- Material-UI: Heavier bundle, different aesthetic
- Styled-components: Inconsistent with Tailwind approach
- Plain CSS: More code, less consistency

## Extensibility Strategy

### Adding New Variants (Future)

**Scenario:** Adding Intel GPU variant

**Required Steps:**
1. Create `devops/intel.Dockerfile` with Intel GPU support
2. Add `intel` to Docker workflow matrix
3. Release Docker image with `docker/vX.Y.Z` tag
4. Workflow generates release body with `### Intel Variant` section
5. **Website updates automatically** when `make update_releases` runs
6. (Optional) Add Intel metadata to `docker-variants.ts` for better UX

**No Code Changes Required:**
- ✅ Backend script discovers Intel automatically
- ✅ Frontend renders Intel card dynamically
- ✅ Falls back to default metadata initially
- ✅ Grid layout adjusts automatically

**Optional Enhancement:**
```typescript
// docker-variants.ts
intel: {
  name: 'intel',
  displayName: 'Intel',
  description: 'Intel GPU acceleration',
  icon: 'gpu-generic',
  gpuVendor: 'Intel',
  color: 'indigo'
}
```

### Handling Variant Deprecation

**Scenario:** Removing old variant from releases

**Approach:**
1. Stop building variant in Docker workflow
2. Remove from release body template
3. Next release won't include deprecated variant
4. Website updates automatically to show only active variants
5. Old versions remain in releases.json history

**No Breaking Changes:**
- Metadata entry can stay (no harm)
- Old docs/references remain accurate
- Users can still pull old tags

### Supporting Beta/Dev Variants

**Future Extension:** Support `docker-dev/v*` releases

**Approach:**
```javascript
// Add to TAG_PATTERNS
{
  regex: /^docker-dev\/v/,
  type: 'docker-dev',
  registry: 'ghcr.io/bodhisearch/bodhiapp'
}
```

**Frontend:**
- Add `releasesData.dockerDev` section
- Render separate "Development Variants" section
- Clear badge distinction (Dev vs Production)

## Success Metrics

### Functional Success

✅ **Auto-Discovery:** All 4 variants discovered from release body
✅ **Data Structure:** releases.json validated with correct structure
✅ **Frontend Rendering:** All variants display with cards
✅ **Copy Functionality:** Copy-to-clipboard works reliably
✅ **Responsive Layout:** Grid adapts mobile → tablet → desktop
✅ **Documentation Link:** Clear path to setup instructions

### Non-Functional Success

✅ **Performance:** Page loads fast, releases.json cached
✅ **Type Safety:** TypeScript catches contract violations
✅ **Accessibility:** Keyboard navigation, ARIA labels
✅ **Error Handling:** Graceful degradation for missing data
✅ **Maintainability:** Clear separation of concerns
✅ **Extensibility:** New variants work without code changes

### User Experience Success

✅ **Discoverability:** Users find Docker section on homepage
✅ **Clarity:** Variant differences clearly explained
✅ **Efficiency:** Copy-to-clipboard reduces manual typing
✅ **Guidance:** Documentation link provides complete setup
✅ **Trust:** Version numbers show latest releases

## Risk Assessment

### Technical Risks

**Risk:** GitHub release body format changes

**Mitigation:**
- Regex patterns have fallback tolerance
- Script logs parsing failures for debugging
- Manual validation in release process
- Documentation of expected format

**Risk:** Variant metadata drift

**Mitigation:**
- Fallback system ensures UI doesn't break
- Metadata optional, not required
- Can update metadata independently
- User testing catches display issues

**Risk:** GitHub API rate limits

**Mitigation:**
- Script uses authenticated requests (higher limits)
- Caches releases.json for 1 hour
- Dry-run mode for testing
- Manual generation as backup

### Process Risks

**Risk:** Release workflow incomplete variant documentation

**Mitigation:**
- Template enforces variant sections
- CI validation of release body structure
- Manual review before publish
- Script logs missing data

**Risk:** Website update lag behind Docker releases

**Mitigation:**
- `make update_releases` in release checklist
- CI job validates releases.json freshness
- User prompt to update if stale
- Clear documentation of update process

## Future Enhancements

### Phase 4: Advanced Features (Potential)

**API Endpoint:**
```typescript
// GET /api/releases
// Returns releases.json dynamically
// Enables client-side filtering/search
```

**Version Comparison:**
```tsx
// Show changelog between versions
// Link to GitHub release notes
// Display release dates and age
```

**Variant Filtering:**
```tsx
// Filter by GPU vendor (NVIDIA, AMD, Intel)
// Filter by platform (ARM64 vs AMD64)
// Search by variant name
```

**Usage Analytics:**
```tsx
// Track popular variants (privacy-respecting)
// Show community recommendations
// Display pull counts if available
```

**Interactive Docs:**
```tsx
// Variant-specific setup instructions
// Hardware requirements checker
// Configuration wizard
```

## Conclusion

This architecture successfully implements automatic Docker variant discovery and display while maintaining flexibility for future growth. The design patterns established here can be applied to other release types (TypeScript client, NAPI bindings) following the same paradigm.

**Key Achievements:**
- Zero-configuration variant addition
- Graceful fallback for unknown variants
- Responsive, accessible UI
- Type-safe data contracts
- Comprehensive error handling
- Clear documentation paths

**Next Steps:**
- Monitor user feedback on variant selection
- Gather performance metrics on different variants
- Consider advanced features based on usage patterns
- Extend pattern to other artifact types as needed
