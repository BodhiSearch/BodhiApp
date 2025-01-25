'use client';

import { EditSettingDialog } from '@/app/ui/settings/EditSettingDialog';
import AppInitializer from '@/components/AppInitializer';
import { Alert, AlertDescription } from '@/components/ui/alert';
import { Badge } from '@/components/ui/badge';
import { Button } from '@/components/ui/button';
import {
  Card,
  CardContent,
  CardDescription,
  CardHeader,
  CardTitle,
} from '@/components/ui/card';
import { Skeleton } from '@/components/ui/skeleton';
import { useSettings } from '@/hooks/useQuery';
import { cn } from '@/lib/utils';
import { Setting } from '@/types/models';
import {
  AlertCircle,
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
        description:
          'Keep alive timeout for llama-server (in seconds). range 300 (5 mins)..=86400 (1 day)',
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
              {group.settings.map(({ key, editable, description }) => {
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
                      {description && (
                        <div className="text-sm text-primary font-semibold">
                          {description}
                        </div>
                      )}
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
  return (
    <AppInitializer authenticated={true} allowedStatus="ready">
      <SettingsPageContent config={SETTINGS_CONFIG} />
    </AppInitializer>
  );
}
