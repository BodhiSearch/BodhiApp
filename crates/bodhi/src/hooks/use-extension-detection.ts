/**
 * Extension detection hook for checking if bodhi-browser extension is installed
 */

import { useState, useEffect } from 'react';

type ExtensionStatus = 'detecting' | 'installed' | 'not-installed';

interface ExtensionDetection {
  status: ExtensionStatus;
  extensionId: string | null;
  refresh: () => void;
  redetect: () => void;
}

export function useExtensionDetection(): ExtensionDetection {
  const [status, setStatus] = useState<ExtensionStatus>('detecting');
  const [extensionId, setExtensionId] = useState<string | null>(null);

  const checkExtension = async () => {
    try {
      if (typeof window !== 'undefined' && (window as any).bodhiext) {
        const id = await (window as any).bodhiext.getExtensionId();
        setExtensionId(id);
        setStatus('installed');
      } else {
        setStatus('not-installed');
      }
    } catch (error) {
      console.log('Extension detection error:', error);
      setStatus('not-installed');
    }
  };

  useEffect(() => {
    // Initial check with delay for extension loading
    const timer = setTimeout(checkExtension, 500);

    // Listen for extension initialization event
    const handleInitialized = (event: CustomEvent) => {
      if (event.detail?.extensionId) {
        setExtensionId(event.detail.extensionId);
        setStatus('installed');
      }
    };

    window.addEventListener('bodhiext:initialized', handleInitialized as EventListener);

    return () => {
      clearTimeout(timer);
      window.removeEventListener('bodhiext:initialized', handleInitialized as EventListener);
    };
  }, []);

  const refresh = () => {
    window.location.reload();
  };

  const redetect = () => {
    setStatus('detecting');
    setTimeout(checkExtension, 100);
  };

  return { status, extensionId, refresh, redetect };
}
