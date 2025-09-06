'use client';

import { ComboBoxResponsive } from '@/components/Combobox';
import { CopyButton } from '@/components/CopyButton';
import { Label } from '@/components/ui/label';
import { Tooltip, TooltipContent, TooltipProvider, TooltipTrigger } from '@/components/ui/tooltip';
import { useChatSettings } from '@/hooks/use-chat-settings';
import { formatPrefixedModel } from '@/schemas/apiModel';
import { Alias } from '@bodhiapp/ts-client';
import { HelpCircle } from 'lucide-react';

interface AliasSelectorProps {
  models: Alias[];
  isLoading?: boolean;
  tooltip: string;
}

export function AliasSelector({ models, isLoading = false, tooltip }: AliasSelectorProps) {
  const { model, setModel } = useChatSettings();

  // Transform models array to match ComboBoxResponsive's Status type
  const modelStatuses = models.flatMap((m) => {
    if (m.source === 'api') {
      // For API models, create entries for each individual model with prefix if exists
      return (m.models || []).map((modelName) => {
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
          value: m.alias || '',
          label: m.alias || '',
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
          {model && (
            <div className="opacity-0 group-hover:opacity-100 transition-opacity duration-200">
              <CopyButton text={model} size="icon" variant="ghost" />
            </div>
          )}
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
