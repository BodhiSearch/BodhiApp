import { memo } from 'react';
import { Card } from '@/components/ui/card';
import { Button } from '@/components/ui/button';
import { Download, Package, CheckCircle2 } from 'lucide-react';
import { CopyableCodeBlock } from '@/app/home/CopyableCodeBlock';
import Link from 'next/link';
import type { PlatformData } from '@/lib/constants';
import { cn } from '@/lib/utils';

interface InstallationOptionsProps {
  platform: PlatformData;
}

function InstallationOptionsComponent({ platform }: InstallationOptionsProps) {
  const hasPackageManagers = platform.packageManagers.length > 0;

  return (
    <div
      className={cn('grid grid-cols-1 gap-6 mx-auto', hasPackageManagers ? 'lg:grid-cols-2 max-w-5xl' : 'max-w-2xl')}
    >
      {/* Direct Download Card */}
      <Card className="p-6 flex flex-col bg-gradient-to-br from-violet-50 to-white">
        <div className="flex items-center gap-3 mb-4">
          <div className="p-3 rounded-lg bg-violet-100">
            <Download className="h-6 w-6 text-violet-600" />
          </div>
          <h3 className="text-xl font-semibold">Direct Download</h3>
        </div>

        <div className="flex-1 space-y-3 mb-6">
          <div className="text-sm text-muted-foreground">
            <span className="font-medium">File:</span> Bodhi_App.{platform.fileType.toLowerCase()}
          </div>
          <div className="text-sm text-muted-foreground">
            <span className="font-medium">Architecture:</span> {platform.arch}
          </div>
          {platform.fileSize && (
            <div className="text-sm text-muted-foreground">
              <span className="font-medium">Size:</span> {platform.fileSize}
            </div>
          )}
        </div>

        {platform.downloadUrl ? (
          <Button size="lg" className="w-full gap-2" asChild>
            <Link href={platform.downloadUrl}>
              <Download className="h-5 w-5" />
              Download for {platform.name}
            </Link>
          </Button>
        ) : (
          <Button size="lg" className="w-full gap-2" disabled>
            <Download className="h-5 w-5" />
            Not Available
          </Button>
        )}
      </Card>

      {/* Package Managers Card - Only render if available */}
      {hasPackageManagers && (
        <Card className="p-6 flex flex-col bg-gradient-to-br from-violet-50 to-white">
          <div className="flex items-center gap-3 mb-4">
            <div className="p-3 rounded-lg bg-violet-100">
              <Package className="h-6 w-6 text-violet-600" />
            </div>
            <h3 className="text-xl font-semibold">Package Managers</h3>
          </div>

          <div className="flex-1 space-y-6">
            {platform.packageManagers.map((pm, index) => (
              <div key={index} className="space-y-3">
                <div className="flex items-center gap-2">
                  <Package className="h-4 w-4 text-violet-600" />
                  <span className="font-medium text-sm">{pm.name}</span>
                </div>
                <CopyableCodeBlock command={pm.command} />
                <div className="space-y-1">
                  {pm.benefits.map((benefit, idx) => (
                    <div key={idx} className="flex items-center gap-2 text-xs text-muted-foreground">
                      <CheckCircle2 className="h-3 w-3 text-green-600" />
                      <span>{benefit}</span>
                    </div>
                  ))}
                </div>
              </div>
            ))}
          </div>
        </Card>
      )}
    </div>
  );
}

export const InstallationOptions = memo(InstallationOptionsComponent);
