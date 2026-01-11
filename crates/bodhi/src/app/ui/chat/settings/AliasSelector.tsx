'use client';

import { ComboBoxResponsive } from '@/components/Combobox';
import { CopyButton } from '@/components/CopyButton';
import { Button } from '@/components/ui/button';
import { Label } from '@/components/ui/label';
import { Tooltip, TooltipContent, TooltipProvider, TooltipTrigger } from '@/components/ui/tooltip';
import { useChatSettings } from '@/hooks/use-chat-settings';
import { useQueryClient } from '@/hooks/useQuery';
import { isApiAlias } from '@/lib/utils';
import { formatPrefixedModel } from '@/schemas/apiModel';
import { AliasResponse } from '@bodhiapp/ts-client';
import { HelpCircle, RefreshCw } from 'lucide-react';

interface AliasSelectorProps {
  models: AliasResponse[];
  isLoading?: boolean;
  tooltip: string;
}

export function AliasSelector({ models, isLoading = false, tooltip }: AliasSelectorProps) {
  const { model, setModel } = useChatSettings();
  const queryClient = useQueryClient();

  // Handle refresh button click
  const handleRefresh = async () => {
    // Invalidate and refetch models cache
    await queryClient.invalidateQueries({ queryKey: ['models'] });
  };

  // Transform models array to match ComboBoxResponsive's Status type
  const modelStatuses = models.flatMap((m) => {
    if (isApiAlias(m)) {
      // For API models, create entries for each individual model with prefix if exists
      return (m.models || []).map((modelName: string) => {
        const prefixedModelName = formatPrefixedModel(modelName, m.prefix);
        return {
          value: prefixedModelName,
          label: prefixedModelName,
        };
      });
    } else {
      // For local models, use the alias
      return [
        {
          value: m.alias,
          label: m.alias,
        },
      ];
    }
  });

  // Find the currently selected model
  const selectedStatus = model ? modelStatuses.find((s) => s.value === model) || { value: model, label: model } : null;

  return (
    <div className="space-y-4" data-testid="model-selector-loaded">
      <div className="space-y-2">
        <div className="flex items-center justify-between group">
          <div className="flex items-center gap-2">
            <Label>Alias/Model</Label>
            <TooltipProvider>
              <Tooltip delayDuration={300}>
                <TooltipTrigger asChild>
                  <HelpCircle className="h-4 w-4 text-muted-foreground hover:text-foreground transition-colors cursor-help" />
                </TooltipTrigger>
                <TooltipContent sideOffset={8} className="data-[side=bottom]:slide-in-from-top-2">
                  <p className="max-w-xs text-sm">{tooltip}</p>
                </TooltipContent>
              </Tooltip>
            </TooltipProvider>
          </div>
          <div className="opacity-0 group-hover:opacity-100 transition-opacity duration-200 flex items-center gap-1">
            <TooltipProvider>
              <Tooltip delayDuration={300}>
                <TooltipTrigger asChild>
                  <Button
                    variant="ghost"
                    size="icon"
                    className="h-6 w-6"
                    onClick={handleRefresh}
                    disabled={isLoading}
                    data-testid="refresh-models-button"
                  >
                    <RefreshCw className={`h-3.5 w-3.5 ${isLoading ? 'animate-spin' : ''}`} />
                  </Button>
                </TooltipTrigger>
                <TooltipContent sideOffset={8}>
                  <p className="text-sm">Refresh models</p>
                </TooltipContent>
              </Tooltip>
            </TooltipProvider>
            {model && <CopyButton text={model} size="icon" variant="ghost" />}
          </div>
        </div>
        <ComboBoxResponsive
          selectedStatus={selectedStatus}
          setSelectedStatus={(status) => setModel(status?.value ?? '')}
          statuses={modelStatuses}
          placeholder="Select alias"
          id="model-selector"
          data-testid="model-selector-trigger"
          loading={isLoading}
        />
      </div>
    </div>
  );
}
