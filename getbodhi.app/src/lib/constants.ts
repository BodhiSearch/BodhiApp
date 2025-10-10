// Download URL loaded from .env.release_urls
export const DOWNLOAD_URL = process.env.NEXT_PUBLIC_DOWNLOAD_URL_MACOS;

// Build-time validation
if (!DOWNLOAD_URL) {
  throw new Error(
    'Missing required environment variable: NEXT_PUBLIC_DOWNLOAD_URL_MACOS. ' +
      'Check .env.release_urls file or set environment variable.'
  );
}

export interface PlatformData {
  name: string;
  arch: string;
  downloadUrl: string | undefined;
  icon: 'apple' | 'monitor';
  fileType: string;
}

export const PLATFORMS: Record<'macos' | 'windows' | 'linux', PlatformData> = {
  macos: {
    name: 'macOS',
    arch: 'Apple Silicon',
    downloadUrl: process.env.NEXT_PUBLIC_DOWNLOAD_URL_MACOS,
    icon: 'apple',
    fileType: 'DMG',
  },
  windows: {
    name: 'Windows',
    arch: 'x64',
    downloadUrl: process.env.NEXT_PUBLIC_DOWNLOAD_URL_WINDOWS,
    icon: 'monitor',
    fileType: 'MSI',
  },
  linux: {
    name: 'Linux',
    arch: 'x64',
    downloadUrl: process.env.NEXT_PUBLIC_DOWNLOAD_URL_LINUX,
    icon: 'monitor',
    fileType: 'RPM',
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
