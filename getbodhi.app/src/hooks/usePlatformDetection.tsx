'use client';

import { useState, useEffect } from 'react';
import { detectOS, getPlatformInfo, type OSType, type PlatformInfo } from '@/lib/platform-detection';

/**
 * React hook for detecting user's platform on the client side
 *
 * Returns 'unknown' during SSR and updates to actual platform on mount
 * This prevents hydration mismatches
 *
 * @returns Platform information including OS and architecture
 */
export function usePlatformDetection(): PlatformInfo {
  const [platformInfo, setPlatformInfo] = useState<PlatformInfo>({
    os: 'unknown',
    arch: 'unknown',
    description: '',
  });

  useEffect(() => {
    const info = getPlatformInfo();
    setPlatformInfo(info);
  }, []);

  return platformInfo;
}

/**
 * Simpler hook that just returns the detected OS
 *
 * @returns OSType - 'macos' | 'windows' | 'linux' | 'unknown'
 */
export function useDetectedOS(): OSType {
  const [os, setOs] = useState<OSType>('unknown');

  useEffect(() => {
    setOs(detectOS());
  }, []);

  return os;
}
