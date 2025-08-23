'use client';

import { EditSettingDialog } from '@/app/ui/settings/EditSettingDialog';
import AppInitializer from '@/components/AppInitializer';
import { Badge } from '@/components/ui/badge';
import { Button } from '@/components/ui/button';
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/components/ui/card';
import { Skeleton } from '@/components/ui/skeleton';
import { useSettings } from '@/hooks/useQuery';
import { cn } from '@/lib/utils';
import { SettingInfo } from '@bodhiapp/ts-client';
import {
  Code,
  Database,
  FileText,
  Hash,
  List,
  LucideIcon,
  Pencil,
  Server,
  Settings,
  Terminal,
  ToggleLeft,
} from 'lucide-react';
import { useState } from 'react';
import { ErrorPage } from '@/components/ui/ErrorPage';
import { UserOnboarding } from '@/components/UserOnboarding';
import { CopyButton } from '@/components/CopyButton';

type SettingConfig = {
  key: string;
  editable: boolean;
  description?: string;
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
        description: 'The home directory for Bodhi application.',
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
        description: 'Home directory for Hugging Face model files.',
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
        description: 'Path to look for Llama.cpp executables.',
      },
      {
        key: 'BODHI_EXEC_VARIANT',
        editable: true,
        description: 'Optimized hardware specific variant of llama.cpp to use.',
      },
      {
        key: 'BODHI_KEEP_ALIVE_SECS',
        editable: true,
        description: 'Keep alive timeout for llama-server (in seconds). range 300 (5 mins)..=86400 (1 day)',
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
        description: 'Scheme used for server connection (http/https).',
      },
      {
        key: 'BODHI_HOST',
        editable: false,
        description: 'Host address for the server.',
      },
      {
        key: 'BODHI_PORT',
        editable: false,
        description: 'Port number for the server.',
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
        description: 'Directory for log files.',
      },
      {
        key: 'BODHI_LOG_LEVEL',
        editable: false,
        description: 'Level of logging (e.g., info, debug).',
      },
      {
        key: 'BODHI_LOG_STDOUT',
        editable: false,
        description: 'Whether to log to standard output.',
      },
    ],
  },
  dev: {
    title: 'Development Settings',
    description: 'Development settings of app',
    icon: Code,
    settings: [
      {
        key: 'BODHI_VERSION',
        editable: false,
        description: 'Version of app',
      },
      {
        key: 'BODHI_ENV_TYPE',
        editable: false,
        description: 'Environment type of app',
      },
      {
        key: 'BODHI_APP_TYPE',
        editable: false,
        description: 'App flavour',
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

// Add this helper function to determine badge variant
const getSourceBadgeVariant = (source: string) => {
  switch (source) {
    case 'system':
      return 'destructive'; // red
    case 'command line':
      return 'blue';
    case 'environment':
      return 'green';
    case 'settings_file':
      return 'orange';
    case 'default':
      return 'gray';
    default:
      return 'secondary';
  }
};

export function SettingsPageContent({ config }: SettingsPageContentProps) {
  const [editingSetting, setEditingSetting] = useState<SettingInfo | null>(null);
  const { data: settings, isLoading, error } = useSettings();

  if (isLoading) {
    return <SettingsSkeleton config={config} />;
  }

  if (error) {
    const errorMessage = error.response?.data?.error?.message || error.message;
    return <ErrorPage message={errorMessage} />;
  }

  return (
    <div className="flex flex-col gap-4 sm:container sm:mx-auto sm:py-4">
      <UserOnboarding storageKey="settings-banner-dismissed">
        Welcome to Settings! Here you can view and manage your application&apos;s configuration. It also shows the
        current value, and the source of that value. Some settings are editable while others are read-only.
      </UserOnboarding>

      {Object.entries(config).map(([groupKey, group]) => {
        const Icon = group.icon;

        return (
          <Card key={groupKey} className="border-x-0 sm:border-x rounded-none sm:rounded-lg">
            <CardHeader className="px-4 py-4">
              <div className="flex items-center gap-2">
                <Icon className="h-4 w-4 text-muted-foreground shrink-0" />
                <CardTitle className="text-base">{group.title}</CardTitle>
              </div>
              <CardDescription>{group.description}</CardDescription>
            </CardHeader>

            <CardContent className="space-y-4 px-4">
              {group.settings.map(({ key, editable, description }) => {
                const setting = settings?.find((s) => s.key === key);
                if (!setting) return null;

                return (
                  <div
                    key={key}
                    className={cn(
                      'flex flex-col sm:flex-row gap-3',
                      'p-4 rounded-lg border',
                      'bg-card hover:bg-accent/5',
                      setting.source !== 'default' && 'border-primary/20 bg-primary/5'
                    )}
                  >
                    {/* Main content section */}
                    <div className="flex-1 min-w-0 space-y-1.5">
                      {/* Setting header with name and description */}
                      <div className="space-y-1">
                        <div className="flex items-center flex-wrap gap-1.5">
                          {(() => {
                            const TypeIcon = SETTING_ICONS[setting.metadata.type as keyof typeof SETTING_ICONS];
                            return <TypeIcon className="h-3.5 w-3.5 sm:h-4 sm:w-4 text-muted-foreground shrink-0" />;
                          })()}
                          <div className="font-medium text-sm sm:text-base truncate">{setting.key}</div>
                          <Badge variant={getSourceBadgeVariant(setting.source)} className="text-xs h-5 px-1.5">
                            {setting.source}
                          </Badge>
                        </div>
                        {description && (
                          <div className="text-xs sm:text-sm text-primary font-medium">{description}</div>
                        )}
                      </div>

                      {/* Values section */}
                      <div className="space-y-1">
                        {/* Only show current value if source is not system */}
                        {setting.source !== 'system' && (
                          <div className="flex items-center gap-1.5">
                            <span className="text-xs sm:text-sm text-muted-foreground shrink-0">Current:</span>
                            <span className="text-xs sm:text-sm text-muted-foreground truncate">
                              {String(setting.current_value)}
                            </span>
                            {setting.metadata.type === 'string' && <CopyButton text={String(setting.current_value)} />}
                          </div>
                        )}

                        {/* Default value */}
                        <div className="flex items-center gap-1.5">
                          <span className="text-xs text-muted-foreground/60 shrink-0">Default:</span>
                          <span className="text-xs text-muted-foreground/60 truncate">
                            {String(setting.default_value)}
                          </span>
                        </div>
                      </div>
                    </div>

                    {/* Actions section */}
                    <div className="flex items-center gap-2 shrink-0">
                      <Badge variant="outline" className="text-xs h-5 px-1.5 shrink-0">
                        {setting.metadata.type}
                      </Badge>
                      {editable && (
                        <Button
                          variant="ghost"
                          size="icon"
                          onClick={() => setEditingSetting(setting)}
                          className="h-7 w-7 sm:h-8 sm:w-8 shrink-0"
                        >
                          <Pencil className="h-3.5 w-3.5 sm:h-4 sm:w-4" />
                          <span className="sr-only">Edit setting</span>
                        </Button>
                      )}
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
    <div className="flex flex-col gap-4 sm:container sm:mx-auto sm:py-4" data-testid="settings-skeleton-container">
      {Object.keys(config).map((group) => (
        <Card key={group} className="border-x-0 sm:border-x rounded-none sm:rounded-lg" data-testid="settings-skeleton">
          <CardHeader>
            <div className="flex items-center gap-2">
              <Skeleton className="h-5 w-5" />
              <Skeleton className="h-6 w-36" />
            </div>
            <Skeleton className="h-4 w-72" />
          </CardHeader>
          <CardContent className="space-y-4">
            {Array.from({ length: 3 }).map((_, i) => (
              <div key={i} className="flex items-start justify-between p-4 rounded-lg border">
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
  return (
    <AppInitializer authenticated={true} allowedStatus="ready">
      <SettingsPageContent config={SETTINGS_CONFIG} />
    </AppInitializer>
  );
}
