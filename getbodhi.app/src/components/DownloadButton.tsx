'use client';

import { Download } from 'lucide-react';
import { Button } from '@/components/ui/button';
import Link from 'next/link';
import { useDetectedOS } from '@/hooks/usePlatformDetection';
import { PLATFORMS, type PlatformData } from '@/lib/constants';
import { getPlatformDisplayName } from '@/lib/platform-detection';

interface DownloadButtonProps {
  /** Button variant - 'default' for primary CTA, 'outline' for secondary */
  variant?: 'default' | 'outline';
  /** Button size */
  size?: 'default' | 'lg';
  /** Additional CSS classes */
  className?: string;
  /** Force a specific platform (overrides auto-detection) */
  forcePlatform?: 'macos' | 'windows' | 'linux';
}

export function DownloadButton({
  variant = 'default',
  size = 'lg',
  className = '',
  forcePlatform,
}: DownloadButtonProps) {
  const detectedOS = useDetectedOS();

  const platformKey = forcePlatform || (detectedOS !== 'unknown' ? detectedOS : 'macos');
  const platform = PLATFORMS[platformKey as keyof typeof PLATFORMS] as PlatformData;

  if (!platform?.downloadUrl) {
    return (
      <Button variant={variant} size={size} className={className} disabled>
        <Download className="h-5 w-5" />
        Download Not Available
      </Button>
    );
  }

  const displayText = forcePlatform
    ? `Download for ${platform.name}`
    : detectedOS !== 'unknown'
      ? `Download for ${getPlatformDisplayName(detectedOS, 'x64')}`
      : 'Download BodhiApp';

  return (
    <Button variant={variant} size={size} className={`gap-2 ${className}`} asChild>
      <Link href={platform.downloadUrl}>
        <Download className="h-5 w-5" />
        {displayText}
      </Link>
    </Button>
  );
}
