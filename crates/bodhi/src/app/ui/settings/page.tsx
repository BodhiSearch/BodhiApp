'use client';

import { useSettings } from '@/hooks/useQuery';
import { Card, CardContent, CardHeader, CardTitle } from '@/components/ui/card';
import { Skeleton } from '@/components/ui/skeleton';
import { Alert, AlertDescription } from '@/components/ui/alert';
import { AlertCircle } from 'lucide-react';

// Group settings by category
const SETTING_GROUPS = {
  app: ['BODHI_HOME'],
  model: ['HF_HOME'],
  server: ['BODHI_SCHEME', 'BODHI_HOST', 'BODHI_PORT'],
  logging: ['BODHI_LOGS', 'BODHI_LOG_LEVEL', 'BODHI_LOG_STDOUT'],
  execution: ['BODHI_EXEC_PATH', 'BODHI_EXEC_LOOKUP_PATH'],
} as const;

const GROUP_TITLES = {
  app: 'App Configuration',
  model: 'Model Files Configuration',
  server: 'Server Configuration',
  logging: 'Logging Configuration',
  execution: 'Llama.cpp Executable Configuration',
} as const;

export default function SettingsPage() {
  const { data: settings, isLoading, error } = useSettings();

  if (isLoading) {
    return <SettingsSkeleton />;
  }

  if (error) {
    return (
      <Alert variant="destructive">
        <AlertCircle className="h-4 w-4" />
        <AlertDescription>
          Failed to load settings: {error.message}
        </AlertDescription>
      </Alert>
    );
  }

  return (
    <div className="container mx-auto py-6 space-y-6">
      <h1 className="text-2xl font-bold">Application Settings</h1>

      {Object.entries(SETTING_GROUPS).map(([group, keys]) => (
        <Card key={group}>
          <CardHeader>
            <CardTitle>
              {GROUP_TITLES[group as keyof typeof GROUP_TITLES]}
            </CardTitle>
          </CardHeader>
          <CardContent className="space-y-4">
            {keys.map((key) => {
              const setting = settings?.find((s) => s.key === key);
              if (!setting) return null;

              return (
                <div key={key} className="space-y-1">
                  <div className="font-medium">{setting.key}</div>
                  <div className="text-sm text-muted-foreground">
                    Current Value: {String(setting.current_value)}
                  </div>
                  <div className="text-sm text-muted-foreground">
                    Default Value: {String(setting.default_value)}
                  </div>
                  <div className="text-sm text-muted-foreground">
                    Source: {setting.source}
                  </div>
                </div>
              );
            })}
          </CardContent>
        </Card>
      ))}
    </div>
  );
}

function SettingsSkeleton() {
  return (
    <div
      className="container mx-auto py-6 space-y-6"
      data-testid="settings-skeleton-container"
    >
      <Skeleton className="h-8 w-48" />

      {Object.keys(SETTING_GROUPS).map((group) => (
        <Card key={group} data-testid="settings-skeleton">
          <CardHeader>
            <Skeleton className="h-6 w-36" />
          </CardHeader>
          <CardContent className="space-y-4">
            {Array.from({ length: 4 }).map((_, i) => (
              <div key={i} className="space-y-2">
                <Skeleton className="h-5 w-32" />
                <Skeleton className="h-4 w-48" />
                <Skeleton className="h-4 w-48" />
                <Skeleton className="h-4 w-24" />
              </div>
            ))}
          </CardContent>
        </Card>
      ))}
    </div>
  );
}
