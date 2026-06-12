# Platform Detection - Quick Reference

## Import Statements

```typescript
// Platform detection utilities
import {
  detectOS,
  detectArchitecture,
  getPlatformInfo,
  getPlatformDisplayName,
  type OSType,
  type ArchType,
  type PlatformInfo,
} from '@/lib/platform-detection';

// React hooks
import { usePlatformDetection, useDetectedOS } from '@/hooks/usePlatformDetection';

// Platform metadata and constants
import { DOWNLOAD_URL, PLATFORMS, getPlatformData, APP_VERSION, APP_TAG, type PlatformData } from '@/lib/constants';
```

## Type Definitions

```typescript
// OS types
type OSType = 'macos' | 'windows' | 'linux' | 'unknown';

// Architecture types
type ArchType = 'arm64' | 'x64' | 'unknown';

// Platform information
interface PlatformInfo {
  os: OSType;
  arch: ArchType;
  description: string;
}

// Platform data
interface PlatformData {
  name: string;
  arch: string;
  downloadUrl: string | undefined;
  icon: 'apple' | 'monitor';
  fileType: string;
}
```

## Functions

### `detectOS(): OSType`

Detects the user's operating system.

**Returns**: `'macos'`, `'windows'`, `'linux'`, or `'unknown'`

**Example**:

```typescript
const os = detectOS();
console.log(os); // 'macos'
```

### `detectArchitecture(): ArchType`

Detects CPU architecture (best effort).

**Returns**: `'arm64'`, `'x64'`, or `'unknown'`

**Example**:

```typescript
const arch = detectArchitecture();
console.log(arch); // 'arm64'
```

### `getPlatformInfo(): PlatformInfo`

Returns comprehensive platform information.

**Returns**: Object with `os`, `arch`, and `description`

**Example**:

```typescript
const info = getPlatformInfo();
console.log(info);
// { os: 'macos', arch: 'arm64', description: 'Mac OS X (10.15.7) Safari 13.1.2' }
```

### `getPlatformDisplayName(os: OSType, arch: ArchType): string`

Returns user-friendly platform name.

**Example**:

```typescript
const name = getPlatformDisplayName('macos', 'arm64');
console.log(name); // "macOS (Apple Silicon)"
```

### `getPlatformData(os: 'macos' | 'windows' | 'linux'): PlatformData`

Returns platform-specific metadata.

**Example**:

```typescript
const platform = getPlatformData('macos');
console.log(platform);
// {
//   name: 'macOS',
//   arch: 'Apple Silicon',
//   downloadUrl: 'https://github.com/.../Bodhi.App_0.1.0_aarch64.dmg',
//   icon: 'apple',
//   fileType: 'DMG'
// }
```

## React Hooks

### `usePlatformDetection(): PlatformInfo`

React hook for detecting platform on client side.

**Important**: Returns `'unknown'` during SSR, updates after mount.

**Example**:

```typescript
'use client';

function MyComponent() {
  const { os, arch, description } = usePlatformDetection();

  return <div>Detected OS: {os}</div>;
}
```

### `useDetectedOS(): OSType`

Simplified hook that returns just the OS type.

**Example**:

```typescript
'use client';

function MyComponent() {
  const os = useDetectedOS();

  return <div>OS: {os}</div>;
}
```

## Constants

### `DOWNLOAD_URL`

Default download URL (macOS ARM64) from environment.

### `PLATFORMS`

Record containing all platform configurations.

**Example**:

```typescript
console.log(PLATFORMS.macos);
// {
//   name: 'macOS',
//   arch: 'Apple Silicon',
//   downloadUrl: 'https://...',
//   icon: 'apple',
//   fileType: 'DMG'
// }
```

### `APP_VERSION`

Application version from environment (`NEXT_PUBLIC_APP_VERSION`).

### `APP_TAG`

Git tag from environment (`NEXT_PUBLIC_APP_TAG`).

## Common Use Cases

### 1. Auto-Select Download Button

```typescript
'use client';

import { usePlatformDetection } from '@/hooks/usePlatformDetection';
import { getPlatformDisplayName } from '@/lib/platform-detection';
import { getPlatformData } from '@/lib/constants';

export function DownloadButton() {
  const { os, arch } = usePlatformDetection();

  if (os === 'unknown') {
    return <button>Download BodhiApp</button>;
  }

  const platform = getPlatformData(os as 'macos' | 'windows' | 'linux');
  const displayName = getPlatformDisplayName(os, arch);

  return (
    <a href={platform.downloadUrl} download>
      Download for {displayName}
    </a>
  );
}
```

### 2. Platform Cards with Detection

```typescript
'use client';

import { useDetectedOS } from '@/hooks/usePlatformDetection';
import { PLATFORMS } from '@/lib/constants';

export function PlatformCards() {
  const detectedOS = useDetectedOS();

  return (
    <div className="grid grid-cols-3 gap-4">
      {Object.entries(PLATFORMS).map(([osKey, platform]) => (
        <div
          key={osKey}
          className={osKey === detectedOS ? 'border-blue-500' : ''}
        >
          <h3>{platform.name}</h3>
          <p>{platform.arch}</p>
          <a href={platform.downloadUrl} download>
            Download {platform.fileType}
          </a>
        </div>
      ))}
    </div>
  );
}
```

### 3. Conditional Rendering Based on OS

```typescript
'use client';

import { useDetectedOS } from '@/hooks/usePlatformDetection';

export function PlatformSpecificInstructions() {
  const os = useDetectedOS();

  if (os === 'macos') {
    return <div>macOS installation instructions...</div>;
  }

  if (os === 'windows') {
    return <div>Windows installation instructions...</div>;
  }

  if (os === 'linux') {
    return <div>Linux installation instructions...</div>;
  }

  return <div>Choose your platform above for installation instructions.</div>;
}
```

## Testing

### Browser Console

```javascript
// Import and test
import { getPlatformInfo } from '@/lib/platform-detection';

const info = getPlatformInfo();
console.log('Platform:', info);
```

### Test Component

```typescript
import { TestPlatformDetection } from '@/components/test-platform-detection';

// Add to any page
export default function TestPage() {
  return <TestPlatformDetection />;
}
```

### Standalone HTML

Open `test-platform-detection.html` in any browser to see detection results.

## SSR Considerations

**Important**: All detection functions return `'unknown'` during server-side rendering to prevent hydration mismatches.

**Pattern**:

```typescript
export function MyComponent() {
  const { os } = usePlatformDetection();

  // During SSR, os will be 'unknown'
  // After client-side mount, os will update to actual value

  if (os === 'unknown') {
    return <div>Loading...</div>; // Or show all options
  }

  return <div>Detected: {os}</div>;
}
```

## Environment Variables

Required in `.env.release_urls`:

```env
NEXT_PUBLIC_DOWNLOAD_URL_MACOS_ARM64=https://...
NEXT_PUBLIC_DOWNLOAD_URL_WINDOWS_X64=https://...
NEXT_PUBLIC_DOWNLOAD_URL_LINUX_X64=https://...
NEXT_PUBLIC_APP_VERSION=0.0.31
NEXT_PUBLIC_APP_TAG=app/v0.0.31
```

## Icon Mapping (lucide-react)

```typescript
import { Apple, Monitor } from 'lucide-react';

const iconMap = {
  macos: Apple, // Apple logo
  windows: Monitor, // lucide-react doesn't have Windows icon
  linux: Monitor, // lucide-react doesn't have Linux icon
};
```

---

**Quick Start**: Import `usePlatformDetection` hook, use in client components, handle 'unknown' state for SSR.
