# Phase 2: Multi-Platform Download Support - Context

## Project Overview

Extend the BodhiApp website homepage to support downloads for all 3 supported platforms (macOS, Windows, Linux) with automatic OS detection and professional UI design.

## Platform Support Analysis

### GitHub Release Assets (app/v0.0.31)

From latest release inspection:

```bash
$ gh api 'repos/BodhiSearch/BodhiApp/releases/tags/app/v0.0.31' --jq '.assets[].name'
```

**Assets Available:**
1. **macOS ARM64**: `Bodhi.App_0.1.0_aarch64.dmg`
2. **Windows x64**: `Bodhi.App_0.1.0_x64_en-US.msi`
3. **Linux x64**: `Bodhi.App-0.1.0-1.x86_64.rpm`

### Asset Naming Patterns

```javascript
const ASSET_PATTERNS = {
  'macos-arm64': /_aarch64\.dmg$/,
  'windows-x64': /_x64_en-US\.msi$/,
  'linux-x64': /\.x86_64\.rpm$/
};
```

## OS Detection Library Research

### Comparison Matrix

| Library | Size (gzipped) | Maintenance | Features | Recommendation |
|---------|---------------|-------------|----------|----------------|
| **platform** | ~1kB | Active | Basic OS/browser detection | ⭐ Best for minimal bundle |
| **bowser** | ~4.8kB | Active | Comprehensive detection | ⭐ Best balance |
| **ua-parser-js** | ~24kB | Active | Very comprehensive | User preference |

### Recommendation: **platform**

**Rationale:**
- Minimal bundle size impact (~1kB)
- Sufficient for our needs (OS + architecture detection)
- Well-maintained, simple API
- Perfect for Next.js SSR/CSR compatibility

**Installation:**
```bash
npm install platform
```

**Usage:**
```typescript
import platform from 'platform';

const os = platform.os?.family; // 'OS X', 'Windows', 'Linux'
const arch = platform.description; // Includes architecture info
```

## UI/UX Design Research

### Best Practices (2025)

**Key Findings:**
1. **Auto-detection is expected** - Popular sites (VS Code, Zoom, Slack) auto-detect OS
2. **Primary + Secondary pattern** - Show detected OS prominently, other platforms secondary
3. **Avoid spam appearance** - Clean, flat design for download buttons
4. **Clear labeling** - State explicitly which platform/architecture
5. **Accessibility** - Minimum 7mm hit area, keyboard navigation

### UI Pattern Decision

**Hero Section:**
- **Primary CTA**: Large button showing detected platform
  - Example: "Download for macOS (Apple Silicon)"
  - Icon + platform name + architecture
- **Secondary**: Small link below
  - "Download for other platforms" → scroll to Download Section

**Download Section:**
- **Platform Cards**: 3 cards side-by-side (responsive grid)
- Each card shows:
  - Platform icon (macOS/Windows/Linux)
  - Platform name + architecture
  - Download button
  - File size (if available)
- **Highlight detected platform**: Gradient border or shadow effect

### Design Mockup (Conceptual)

```
┌─────────────────────────────────────────┐
│          HERO SECTION                    │
│                                          │
│  ┌────────────────────────────────────┐ │
│  │ ⬇ Download for macOS (Apple Silicon)│ │ ← Detected platform
│  └────────────────────────────────────┘ │
│         Download for other platforms    │ ← Secondary link
└─────────────────────────────────────────┘

┌─────────────────────────────────────────┐
│       DOWNLOAD SECTION                   │
│  Choose your platform                    │
│                                          │
│  ┌───────┐  ┌───────┐  ┌───────┐       │
│  │  🍎   │  │  🪟   │  │  🐧   │       │
│  │macOS  │  │Windows│  │ Linux │       │
│  │ARM64  │  │  x64  │  │  x64  │       │
│  │       │  │       │  │       │       │
│  │[Down] │  │[Down] │  │[Down] │       │
│  └───────┘  └───────┘  └───────┘       │
│     ^                                    │
│     └─ Highlighted (detected)            │
└─────────────────────────────────────────┘
```

## releases.json Structure

### Purpose
- Provide machine-readable release information
- Enable automation and programmatic downloads
- Accessible via HTTP at `/releases.json`

### Schema

```json
{
  "version": "0.0.31",
  "tag": "app/v0.0.31",
  "released_at": "2025-10-07T10:45:42Z",
  "platforms": {
    "macos": {
      "arm64": {
        "download_url": "https://github.com/BodhiSearch/BodhiApp/releases/download/app/v0.0.31/Bodhi.App_0.1.0_aarch64.dmg",
        "filename": "Bodhi.App_0.1.0_aarch64.dmg",
        "size_bytes": null
      }
    },
    "windows": {
      "x64": {
        "download_url": "https://github.com/BodhiSearch/BodhiApp/releases/download/app/v0.0.31/Bodhi.App_0.1.0_x64_en-US.msi",
        "filename": "Bodhi.App_0.1.0_x64_en-US.msi",
        "size_bytes": null
      }
    },
    "linux": {
      "x64": {
        "download_url": "https://github.com/BodhiSearch/BodhiApp/releases/download/app/v0.0.31/Bodhi.App-0.1.0-1.x86_64.rpm",
        "filename": "Bodhi.App-0.1.0-1.x86_64.rpm",
        "size_bytes": null
      }
    }
  }
}
```

**Note:** `size_bytes` is optional for initial implementation (GitHub API doesn't provide in releases list).

## Technical Architecture

### Data Flow

```
GitHub Releases API
    ↓
update-release-urls.js
    ├──→ .env.release_urls (3 URLs + tag)
    └──→ public/releases.json (structured JSON)
         ↓
Next.js Build
    ├──→ Environment Variables (constants.ts)
    └──→ Static JSON (public asset)
         ↓
Homepage Components
    ├──→ Platform Detection (usePlatformDetection)
    ├──→ DownloadButton (auto-selected platform)
    └──→ DownloadSection (all platforms)
```

### Environment Variables

```env
# .env.release_urls
NEXT_PUBLIC_APP_VERSION=0.0.31
NEXT_PUBLIC_APP_TAG=app/v0.0.31
NEXT_PUBLIC_DOWNLOAD_URL_MACOS_ARM64=https://...
NEXT_PUBLIC_DOWNLOAD_URL_WINDOWS_X64=https://...
NEXT_PUBLIC_DOWNLOAD_URL_LINUX_X64=https://...
```

## Implementation Checklist

Phase 2 implementation tasks:

- [ ] Create phase2-context.md (this file)
- [ ] Create phase2-agent-log.md
- [ ] Extend update-release-urls.js for 3 platforms
- [ ] Generate .env.release_urls with all platforms
- [ ] Generate public/releases.json
- [ ] Update next.config.mjs validation
- [ ] Install platform detection library
- [ ] Create platform-detection.ts utilities
- [ ] Create usePlatformDetection hook
- [ ] Create PlatformIcon component
- [ ] Create DownloadButton component
- [ ] Extend constants.ts with platform data
- [ ] Update HeroSection
- [ ] Update DownloadSection
- [ ] Test on multiple browsers/OS
- [ ] Document results

## Dependencies

**New npm packages:**
- `platform` (~1kB) - OS detection

**Existing UI components to leverage:**
- `@/components/ui/button` - Base button styling
- `@/components/ui/card` - Platform cards
- `lucide-react` - Icons (Download, ChevronDown, etc.)

## Success Metrics

1. ✅ All 3 platforms downloadable from homepage
2. ✅ Auto-detection works on macOS, Windows, Linux browsers
3. ✅ Mobile-responsive design
4. ✅ `public/releases.json` auto-generated
5. ✅ Build succeeds with all platform URLs
6. ✅ Professional, trustworthy UI design
7. ✅ Accessible (keyboard navigation, screen reader compatible)

## Next Steps

Proceed to implementation using agents:
1. **automation-agent**: Extend release automation script
2. **platform-agent**: Implement OS detection
3. **ui-agent**: Build components and update homepage

All work will be logged in `phase2-agent-log.md`.
