import { Apple, Monitor } from 'lucide-react';

interface PlatformIconProps {
  platform: 'macos' | 'windows' | 'linux';
  className?: string;
}

export function PlatformIcon({ platform, className = 'h-6 w-6' }: PlatformIconProps) {
  if (platform === 'macos') {
    return <Apple className={className} aria-label="macOS" />;
  }

  return <Monitor className={className} aria-label={platform === 'windows' ? 'Windows' : 'Linux'} />;
}
