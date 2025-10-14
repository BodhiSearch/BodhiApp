import { memo } from 'react';
import { Download, Package } from 'lucide-react';
import { Button } from '@/components/ui/button';
import { Card } from '@/components/ui/card';
import { CopyableCodeBlock } from '@/app/home/CopyableCodeBlock';
import Link from 'next/link';
import type { OSType } from '@/lib/platform-detection';
import type { PlatformData } from '@/lib/constants';

interface HeroCTAProps {
  platform: OSType;
  platformData?: PlatformData;
}

function HeroCTAComponent({ platform, platformData }: HeroCTAProps) {
  // Fallback for unknown platform
  if (!platformData || platform === 'unknown') {
    return (
      <div className="max-w-xl mx-auto space-y-4">
        <Button size="lg" className="w-full gap-2" asChild>
          <Link href="#download-section">
            <Download className="h-5 w-5" />
            Download BodhiApp
          </Link>
        </Button>
        <div className="text-center">
          <Link
            href="#download-section"
            className="text-sm text-violet-600 hover:text-violet-700 hover:underline font-medium inline-flex items-center gap-1"
          >
            View all platforms
            <span>↓</span>
          </Link>
        </div>
      </div>
    );
  }

  const hasPackageManagers = platformData.packageManagers.length > 0;
  const primaryPackageManager = hasPackageManagers ? platformData.packageManagers[0] : null;

  return (
    <div className="w-full space-y-4">
      {/* Primary Download and Package Manager - Side by side on desktop, stacked on mobile */}
      <div className="flex flex-col md:flex-row justify-center items-stretch gap-4 px-4 max-w-5xl mx-auto">
        {/* Primary Download Card */}
        {platformData.downloadUrl ? (
          <Card className="p-6 bg-gray-50 border border-gray-200 w-full md:w-auto md:min-w-[400px]">
            <div className="flex items-center gap-2 mb-3">
              <Download className="h-4 w-4 text-violet-600" />
              <span className="text-sm font-medium text-gray-700">{platformData.fileType} file</span>
            </div>
            <div className="flex items-center justify-center min-h-[60px]">
              <Button size="lg" className="gap-2 shadow-md hover:shadow-lg" asChild>
                <Link href={platformData.downloadUrl}>
                  <Download className="h-5 w-5" />
                  Download for {platformData.name} ({platformData.arch})
                </Link>
              </Button>
            </div>
          </Card>
        ) : (
          <Card className="p-6 bg-gray-50 border border-gray-200 w-full md:w-auto md:min-w-[400px]">
            <div className="flex items-center gap-2 mb-3">
              <Download className="h-4 w-4 text-violet-600" />
              <span className="text-sm font-medium text-gray-700">{platformData.fileType} file</span>
            </div>
            <div className="flex items-center justify-center min-h-[60px]">
              <Button size="lg" className="gap-2" disabled>
                <Download className="h-5 w-5" />
                Download Not Available
              </Button>
            </div>
          </Card>
        )}

        {/* Secondary Package Manager Card */}
        {hasPackageManagers && primaryPackageManager && (
          <Card className="p-6 bg-gray-50 border border-gray-200 w-full md:w-auto md:min-w-[400px]">
            <div className="flex items-center gap-2 mb-3">
              <Package className="h-4 w-4 text-violet-600" />
              <span className="text-sm font-medium text-gray-700">Install via {primaryPackageManager.name}</span>
            </div>
            <div className="flex items-center min-h-[60px]">
              <CopyableCodeBlock command={primaryPackageManager.command} />
            </div>
          </Card>
        )}
      </div>

      {/* Tertiary Platform Selector Link */}
      <div className="text-center">
        <Link
          href="#download-section"
          className="text-sm text-violet-600 hover:text-violet-700 hover:underline font-medium inline-flex items-center gap-1"
        >
          Download for other platforms
          <span>↓</span>
        </Link>
      </div>
    </div>
  );
}

export const HeroCTA = memo(HeroCTAComponent);
