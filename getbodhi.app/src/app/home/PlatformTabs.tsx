import { memo } from 'react';
import { Apple, Monitor } from 'lucide-react';
import { Badge } from '@/components/ui/badge';
import type { OSType } from '@/lib/platform-detection';
import { cn } from '@/lib/utils';

interface PlatformTab {
  id: OSType;
  name: string;
  arch: string;
  icon: 'apple' | 'monitor';
}

interface PlatformTabsProps {
  platforms: PlatformTab[];
  selectedPlatform: OSType;
  detectedPlatform: OSType;
  onPlatformChange: (platform: OSType) => void;
}

// Move outside component to prevent recreation on every render
const getIcon = (iconType: 'apple' | 'monitor') => {
  if (iconType === 'apple') {
    return <Apple className="h-12 w-12" />;
  }
  return <Monitor className="h-12 w-12" />;
};

function PlatformTabsComponent({ platforms, selectedPlatform, detectedPlatform, onPlatformChange }: PlatformTabsProps) {
  return (
    <div className="flex flex-col sm:flex-row justify-center gap-4 mb-12">
      {platforms.map((platform) => {
        const isActive = selectedPlatform === platform.id;
        const isDetected = detectedPlatform === platform.id;

        return (
          <button
            key={platform.id}
            onClick={() => onPlatformChange(platform.id)}
            className={cn(
              'relative flex flex-col items-center gap-3 p-6 rounded-xl transition-all duration-200 min-w-[160px]',
              isActive
                ? 'bg-violet-600 text-white ring-2 ring-violet-500 shadow-lg scale-105'
                : 'bg-white text-gray-700 border-2 border-gray-200 hover:border-violet-200 hover:shadow-md'
            )}
            aria-pressed={isActive}
            aria-label={`Select ${platform.name}`}
          >
            {/* Icon */}
            <div className={cn('transition-colors', isActive ? 'text-white' : 'text-violet-600')}>
              {getIcon(platform.icon)}
            </div>

            {/* Platform Info */}
            <div className="text-center">
              <div className="font-semibold text-lg">{platform.name}</div>
              <div className={cn('text-sm', isActive ? 'text-violet-100' : 'text-muted-foreground')}>
                {platform.arch}
              </div>
            </div>

            {/* Detected Badge */}
            {isDetected && (
              <Badge
                variant="secondary"
                className={cn(
                  'absolute -top-2 -right-2',
                  isActive ? 'bg-white text-violet-600' : 'bg-violet-100 text-violet-600'
                )}
              >
                Detected
              </Badge>
            )}
          </button>
        );
      })}
    </div>
  );
}

export const PlatformTabs = memo(PlatformTabsComponent);
