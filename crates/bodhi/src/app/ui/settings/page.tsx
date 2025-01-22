'use client';

import { Alert, AlertDescription } from '@/components/ui/alert';
import { Badge } from '@/components/ui/badge';
import {
  Card,
  CardContent,
  CardDescription,
  CardHeader,
  CardTitle,
} from '@/components/ui/card';
import { Skeleton } from '@/components/ui/skeleton';
import {
  Tooltip,
  TooltipContent,
  TooltipProvider,
  TooltipTrigger,
} from '@/components/ui/tooltip';
import { useSettings } from '@/hooks/useQuery';
import { cn } from '@/lib/utils';
import {
  AlertCircle,
  Database,
  FileText,
  Hash,
  Info,
  List,
  Server,
  Settings,
  Terminal,
  ToggleLeft,
  Pencil,
  LucideIcon,
} from 'lucide-react';
import { EditSettingDialog } from './components/EditSettingDialog';
import { Button } from '@/components/ui/button';
import { useState } from 'react';
import { Setting } from '@/types/models';

type SettingConfig = {
  key: string;
  editable: boolean;
};

type GroupConfig = {
  title: string;
  description: string;
  icon: LucideIcon;
  settings: SettingConfig[];
};

type SettingsConfig = {
  [key: string]: GroupConfig;
};

const SETTINGS_CONFIG: SettingsConfig = {
  app: {
    title: 'App Configuration',
    description: 'Core application settings and paths',
    icon: Settings,
    settings: [
      {
        key: 'BODHI_HOME',
        editable: false,
      },
    ],
  },
  model: {
    title: 'Model Files Configuration',
    description: 'Model file storage and configuration',
    icon: Database,
    settings: [
      {
        key: 'HF_HOME',
        editable: false,
      },
    ],
  },
  execution: {
    title: 'Llama.cpp Executable Configuration',
    description: 'LLama.cpp execution settings',
    icon: Terminal,
    settings: [
      {
        key: 'BODHI_EXEC_LOOKUP_PATH',
        editable: false,
      },
      {
        key: 'BODHI_EXEC_PATH',
        editable: true,
      },
    ],
  },
  server: {
    title: 'Server Configuration',
    description: 'Server connection and networking settings',
    icon: Server,
    settings: [
      {
        key: 'BODHI_SCHEME',
        editable: false,
      },
      {
        key: 'BODHI_HOST',
        editable: false,
      },
      {
        key: 'BODHI_PORT',
        editable: false,
      },
    ],
  },
  logging: {
    title: 'Logging Configuration',
    description: 'Logging and debugging configuration',
    icon: FileText,
    settings: [
      {
        key: 'BODHI_LOGS',
        editable: false,
      },
      {
        key: 'BODHI_LOG_LEVEL',
        editable: false,
      },
      {
        key: 'BODHI_LOG_STDOUT',
        editable: false,
      },
    ],
  },
} as const;

const SETTING_ICONS = {
  string: FileText,
  number: Hash,
  boolean: ToggleLeft,
  option: List,
} as const;

type SettingsPageContentProps = {
  config: SettingsConfig;
};

export function SettingsPageContent({ config }: SettingsPageContentProps) {
  const [editingSetting, setEditingSetting] = useState<Setting | null>(null);
  const { data: settings, isLoading, error } = useSettings();

  if (isLoading) {
    return <SettingsSkeleton config={config} />;
  }

  if (error) {
    const errorMessage = error.response?.data?.error?.message || error.message;
    return (
      <Alert variant="destructive">
        <AlertCircle className="h-4 w-4" />
        <AlertDescription>
          Failed to load settings: {errorMessage}
        </AlertDescription>
      </Alert>
    );
  }

  return (
    <div className="container mx-auto py-6 space-y-6">
      <div className="space-y-2 mb-6">
        <h1 className="text-2xl font-bold">Application Settings</h1>
        <p className="text-muted-foreground">
          View and manage application configuration settings
        </p>
      </div>

      {Object.entries(config).map(([groupKey, group]) => {
        const Icon = group.icon;

        return (
          <Card key={groupKey}>
            <CardHeader>
              <div className="flex items-center gap-2">
                <Icon className="h-5 w-5 text-muted-foreground" />
                <CardTitle>{group.title}</CardTitle>
              </div>
              <CardDescription>{group.description}</CardDescription>
            </CardHeader>
            <CardContent className="space-y-4">
              {group.settings.map(({ key, editable }) => {
                const setting = settings?.find((s) => s.key === key);
                if (!setting) return null;

                return (
                  <div
                    key={key}
                    className={cn(
                      'flex items-start justify-between p-4 rounded-lg border bg-card hover:bg-accent/5',
                      setting.source !== 'default' &&
                        'border-primary/20 bg-primary/5'
                    )}
                  >
                    <div className="space-y-1">
                      <div className="flex items-center gap-2">
                        {(() => {
                          const TypeIcon =
                            SETTING_ICONS[
                              setting.metadata
                                .type as keyof typeof SETTING_ICONS
                            ];
                          return (
                            <TypeIcon className="h-4 w-4 text-muted-foreground" />
                          );
                        })()}
                        <div className="font-medium">{setting.key}</div>
                        <Badge
                          variant={
                            setting.source === 'default'
                              ? 'secondary'
                              : 'default'
                          }
                        >
                          {setting.source}
                        </Badge>
                      </div>
                      <div className="text-sm text-muted-foreground max-w-[500px] break-all">
                        Current: {String(setting.current_value)}
                      </div>
                      <div className="text-xs text-muted-foreground/60">
                        Default: {String(setting.default_value)}
                      </div>
                    </div>
                    <div className="flex items-center gap-2">
                      <Badge variant="outline">{setting.metadata.type}</Badge>
                      {editable && (
                        <Button
                          variant="ghost"
                          size="icon"
                          onClick={() => setEditingSetting(setting)}
                          className="h-8 w-8"
                        >
                          <Pencil className="h-4 w-4" />
                          <span className="sr-only">Edit setting</span>
                        </Button>
                      )}
                      <TooltipProvider>
                        <Tooltip>
                          <TooltipTrigger asChild>
                            <Info className="h-4 w-4 text-muted-foreground cursor-help" />
                          </TooltipTrigger>
                          <TooltipContent>
                            <div className="space-y-1">
                              <p>Type: {setting.metadata.type}</p>
                              {setting.metadata.range && (
                                <p>
                                  Range: {setting.metadata.range.min} -{' '}
                                  {setting.metadata.range.max}
                                </p>
                              )}
                              {setting.metadata.options && (
                                <p>
                                  Options: {setting.metadata.options.join(', ')}
                                </p>
                              )}
                            </div>
                          </TooltipContent>
                        </Tooltip>
                      </TooltipProvider>
                    </div>
                  </div>
                );
              })}
            </CardContent>
          </Card>
        );
      })}

      {editingSetting && (
        <EditSettingDialog
          setting={editingSetting}
          open={!!editingSetting}
          onOpenChange={(open) => !open && setEditingSetting(null)}
        />
      )}
    </div>
  );
}

function SettingsSkeleton({ config }: { config: SettingsConfig }) {
  return (
    <div
      className="container mx-auto py-6 space-y-6"
      data-testid="settings-skeleton-container"
    >
      <div className="space-y-2 mb-6">
        <Skeleton className="h-8 w-48" />
        <Skeleton className="h-5 w-96" />
      </div>

      {Object.keys(config).map((group) => (
        <Card key={group} data-testid="settings-skeleton">
          <CardHeader>
            <div className="flex items-center gap-2">
              <Skeleton className="h-5 w-5" />
              <Skeleton className="h-6 w-36" />
            </div>
            <Skeleton className="h-4 w-72 mt-1.5" />
          </CardHeader>
          <CardContent className="space-y-4">
            {Array.from({ length: 3 }).map((_, i) => (
              <div
                key={i}
                className="flex items-start justify-between p-4 rounded-lg border"
              >
                <div className="space-y-2">
                  <div className="flex items-center gap-2">
                    <Skeleton className="h-4 w-4" />
                    <Skeleton className="h-5 w-32" />
                    <Skeleton className="h-5 w-24" />
                  </div>
                  <Skeleton className="h-4 w-48" />
                  <Skeleton className="h-4 w-40" />
                </div>
                <div className="flex items-center gap-2">
                  <Skeleton className="h-5 w-16" />
                  <Skeleton className="h-4 w-4" />
                </div>
              </div>
            ))}
          </CardContent>
        </Card>
      ))}
    </div>
  );
}

export default function SettingsPage() {
  return <SettingsPageContent config={SETTINGS_CONFIG} />;
}
