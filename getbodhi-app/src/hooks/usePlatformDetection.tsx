'use client';

import { useState, useEffect } from 'react';
import { detectOS, getPlatformInfo, type OSType, type PlatformInfo } from '@/lib/platform-detection';

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

export function useDetectedOS(): OSType {
  const [os, setOs] = useState<OSType>('unknown');

  useEffect(() => {
    setOs(detectOS());
  }, []);

  return os;
}
