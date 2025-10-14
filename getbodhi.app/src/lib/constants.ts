// Download URL loaded from .env.release_urls
export const DOWNLOAD_URL = process.env.NEXT_PUBLIC_DOWNLOAD_URL_MACOS;

// Build-time validation
if (!DOWNLOAD_URL) {
  throw new Error(
    'Missing required environment variable: NEXT_PUBLIC_DOWNLOAD_URL_MACOS. ' +
      'Check .env.release_urls file or set environment variable.'
  );
}

export interface PackageManager {
  name: string;
  command: string;
}

export interface PlatformData {
  name: string;
  arch: string;
  downloadUrl: string | undefined;
  icon: 'apple' | 'monitor';
  fileType: string;
  fileSize?: string;
  packageManagers: PackageManager[];
}

export const PLATFORMS: Record<'macos' | 'windows' | 'linux', PlatformData> = {
  macos: {
    name: 'macOS',
    arch: 'Apple Silicon',
    downloadUrl: process.env.NEXT_PUBLIC_DOWNLOAD_URL_MACOS,
    icon: 'apple',
    fileType: 'DMG',
    packageManagers: [
      {
        name: 'Homebrew',
        command: 'brew install BodhiSearch/apps/bodhi',
      },
    ],
  },
  windows: {
    name: 'Windows',
    arch: 'x64',
    downloadUrl: process.env.NEXT_PUBLIC_DOWNLOAD_URL_WINDOWS,
    icon: 'monitor',
    fileType: 'MSI',
    packageManagers: [
      // Future: WinGet, Chocolatey
    ],
  },
  linux: {
    name: 'Linux',
    arch: 'x64',
    downloadUrl: process.env.NEXT_PUBLIC_DOWNLOAD_URL_LINUX,
    icon: 'monitor',
    fileType: 'RPM',
    packageManagers: [
      // Future: APT, DNF, etc.
    ],
  },
};

export function getPlatformData(os: 'macos' | 'windows' | 'linux'): PlatformData {
  return PLATFORMS[os];
}

export const APP_VERSION = process.env.NEXT_PUBLIC_APP_VERSION;
export const APP_TAG = process.env.NEXT_PUBLIC_APP_TAG;

// Docker release info from environment
export const DOCKER_VERSION = process.env.NEXT_PUBLIC_DOCKER_VERSION;
export const DOCKER_TAG = process.env.NEXT_PUBLIC_DOCKER_TAG;
export const DOCKER_REGISTRY = process.env.NEXT_PUBLIC_DOCKER_REGISTRY;

// Social links
export const SOCIAL_LINKS = {
  github: 'https://github.com/BodhiSearch/BodhiApp',
  discord: 'https://discord.gg/3vur28nz82',
  productHunt:
    'https://www.producthunt.com/posts/bodhi-app-run-llms-locally?embed=true&utm_source=badge-featured&utm_medium=badge&utm_souce=badge-bodhi-app-run-llms-locally',
} as const;

// Section gradient patterns
export const SECTION_GRADIENTS = {
  violetToWhite: 'bg-gradient-to-b from-violet-50 to-white',
  whiteToViolet: 'bg-gradient-to-b from-white to-violet-50',
} as const;

// Common style constants
export const STYLES = {
  sectionHeading: 'text-3xl font-semibold tracking-tight',
  sectionDescription: 'text-xl text-muted-foreground mx-auto max-w-2xl',
  iconBackground: 'bg-violet-100',
  iconColor: 'text-violet-600',
  iconSize: 'h-6 w-6',
  iconSizeSmall: 'h-4 w-4',
  linkHover: 'hover:text-violet-600',
  featureGrid: 'grid grid-cols-1 gap-6 sm:grid-cols-2 lg:grid-cols-3',
} as const;
