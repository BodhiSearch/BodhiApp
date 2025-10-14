'use client';

import { useState, useEffect, useMemo, memo, useRef } from 'react';
import { Container } from '@/components/ui/container';
import { motion, AnimatePresence } from 'framer-motion';
import { useDetectedOS } from '@/hooks/usePlatformDetection';
import { PLATFORMS, SECTION_GRADIENTS } from '@/lib/constants';
import { PlatformTabs } from '@/app/home/PlatformTabs';
import { InstallationOptions } from '@/app/home/InstallationOptions';
import { fadeInUp, slideTransition } from '@/app/home/animations';
import { cn } from '@/lib/utils';
import type { OSType } from '@/lib/platform-detection';

function DownloadSectionComponent() {
  const detectedOS = useDetectedOS();
  const [selectedPlatform, setSelectedPlatform] = useState<OSType>(detectedOS !== 'unknown' ? detectedOS : 'macos');
  const userHasSelected = useRef(false);

  // Update selected platform when detection completes (only if user hasn't manually selected)
  useEffect(() => {
    if (
      !userHasSelected.current &&
      detectedOS !== 'unknown' &&
      selectedPlatform === 'macos' &&
      detectedOS !== 'macos'
    ) {
      setSelectedPlatform(detectedOS);
    }
  }, [detectedOS, selectedPlatform]);

  const handlePlatformChange = (platform: OSType) => {
    userHasSelected.current = true;
    setSelectedPlatform(platform);
  };

  const platformTabs = useMemo(
    () =>
      Object.entries(PLATFORMS)
        .filter(([key]) => key !== 'unknown')
        .map(([key, data]) => ({
          id: key as OSType,
          name: data.name,
          arch: data.arch,
          icon: data.icon,
        })),
    []
  );

  const currentPlatform = PLATFORMS[selectedPlatform as keyof typeof PLATFORMS];

  return (
    <section id="download-section" className={cn('pt-20 pb-8', SECTION_GRADIENTS.violetToWhite)}>
      <Container>
        <motion.div {...fadeInUp} className="text-center">
          <h2 className="text-3xl font-bold mb-4">Download for your platform</h2>
          <p className="text-muted-foreground mb-12 max-w-2xl mx-auto">
            Choose your operating system to download BodhiApp. All platforms support running LLMs locally with full
            privacy.
          </p>

          {/* Platform Tabs */}
          <PlatformTabs
            platforms={platformTabs}
            selectedPlatform={selectedPlatform}
            detectedPlatform={detectedOS}
            onPlatformChange={handlePlatformChange}
          />

          {/* Installation Options with Animation */}
          <AnimatePresence mode="wait">
            <motion.div key={selectedPlatform} {...slideTransition}>
              {currentPlatform && <InstallationOptions platform={currentPlatform} />}
            </motion.div>
          </AnimatePresence>
        </motion.div>
      </Container>
    </section>
  );
}

export const DownloadSection = memo(DownloadSectionComponent);
