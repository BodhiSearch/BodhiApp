/**
 * Platform detection utilities for auto-selecting download links
 * Uses 'platform' library for lightweight, reliable OS detection
 */

import platform from 'platform';

export type OSType = 'macos' | 'windows' | 'linux' | 'unknown';
export type ArchType = 'arm64' | 'x64' | 'unknown';

export interface PlatformInfo {
  os: OSType;
  arch: ArchType;
  description: string;
}

/**
 * Detect the user's operating system
 * Returns 'macos', 'windows', 'linux', or 'unknown'
 */
export function detectOS(): OSType {
  if (typeof window === 'undefined') {
    return 'unknown';
  }

  const osFamily = platform.os?.family?.toLowerCase() || '';

  if (osFamily.includes('os x') || osFamily.includes('mac')) {
    return 'macos';
  }
  if (osFamily.includes('windows')) {
    return 'windows';
  }
  if (osFamily.includes('linux') || osFamily.includes('ubuntu') || osFamily.includes('fedora')) {
    return 'linux';
  }

  return 'unknown';
}

/**
 * Detect the user's CPU architecture
 * Returns 'arm64', 'x64', or 'unknown'
 *
 * Note: Browser UA detection for architecture is unreliable
 * This is a best-effort attempt
 */
export function detectArchitecture(): ArchType {
  if (typeof window === 'undefined') {
    return 'unknown';
  }

  const description = platform.description?.toLowerCase() || '';

  if (description.includes('arm') || description.includes('aarch64')) {
    return 'arm64';
  }

  return 'x64';
}

/**
 * Get comprehensive platform information
 */
export function getPlatformInfo(): PlatformInfo {
  const os = detectOS();
  const arch = detectArchitecture();
  const description = platform.description || 'Unknown platform';

  return {
    os,
    arch,
    description,
  };
}

/**
 * Get user-friendly platform name
 * e.g., "macOS (Apple Silicon)", "Windows (x64)", "Linux (x64)"
 */
export function getPlatformDisplayName(os: OSType, arch: ArchType): string {
  const osNames = {
    macos: 'macOS',
    windows: 'Windows',
    linux: 'Linux',
    unknown: 'Your Platform',
  };

  const archNames = {
    arm64: 'Apple Silicon',
    x64: 'x64',
    unknown: '',
  };

  const osName = osNames[os];
  const archName = archNames[arch];

  if (os === 'macos' && arch === 'arm64') {
    return `${osName} (${archName})`;
  }

  if (arch !== 'unknown' && os !== 'unknown') {
    return `${osName} (${archName})`;
  }

  return osName;
}
