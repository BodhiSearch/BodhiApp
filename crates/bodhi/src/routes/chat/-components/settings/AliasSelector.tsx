import { useEffect, useMemo } from 'react';

import { AliasResponse } from '@bodhiapp/ts-client';
import { HelpCircle, RefreshCw } from 'lucide-react';

import { ComboBoxResponsive } from '@/components/Combobox';
import { CopyButton } from '@/components/CopyButton';
import { Button } from '@/components/ui/button';
import { Label } from '@/components/ui/label';
import { Tooltip, TooltipContent, TooltipProvider, TooltipTrigger } from '@/components/ui/tooltip';
import { modelKeys } from '@/hooks/models/constants';
import { useQueryClient } from '@/hooks/useQuery';
import type { ApiFormat } from '@bodhiapp/ts-client';
import { isApiAlias } from '@/lib/utils';
import { formatPrefixedModel, getApiModelId } from '@/schemas/apiModel';
import { useChatSettingsStore } from '@/stores/chatSettingsStore';

interface AliasSelectorProps {
  models: AliasResponse[];
  isLoading?: boolean;
  tooltip: string;
}

export function AliasSelector({ models, isLoading = false, tooltip }: AliasSelectorProps) {
  const model = useChatSettingsStore((s) => s.model);
  const setModel = useChatSettingsStore((s) => s.setModel);
  const setApiFormat = useChatSettingsStore((s) => s.setApiFormat);
  const setLlmLibertyProvider = useChatSettingsStore((s) => s.setLlmLibertyProvider);
  const queryClient = useQueryClient();

  const handleRefresh = async () => {
    await queryClient.invalidateQueries({ queryKey: modelKeys.all });
  };

  // Map prefixed model names back to their parent AliasResponse for api_format detection
  const modelToAliasMap = useMemo(() => {
    const map = new Map<string, AliasResponse>();
    models.forEach((m) => {
      if (isApiAlias(m)) {
        (m.models || []).forEach((apiModel) => {
          map.set(formatPrefixedModel(getApiModelId(apiModel, m.prefix), m.prefix), m);
        });
      } else {
        map.set(m.alias, m);
      }
    });
    return map;
  }, [models]);

  const selectedAlias = useMemo(() => modelToAliasMap.get(model ?? ''), [modelToAliasMap, model]);
  const selectedApiFormat: ApiFormat =
    selectedAlias && isApiAlias(selectedAlias) ? (selectedAlias.api_format as ApiFormat) : 'openai';
  const selectedLlmLibertyProvider: string | null =
    selectedAlias && isApiAlias(selectedAlias) ? (selectedAlias.llm_liberty?.provider ?? null) : null;

  // Sync apiFormat when the selected model changes (e.g., from URL params or chat history restore)
  useEffect(() => {
    if (!model || models.length === 0) return;
    setApiFormat(selectedApiFormat);
    setLlmLibertyProvider(selectedLlmLibertyProvider);
  }, [model, models.length, selectedApiFormat, selectedLlmLibertyProvider, setApiFormat, setLlmLibertyProvider]);

  const modelStatuses = models.flatMap((m) => {
    if (isApiAlias(m)) {
      return (m.models || []).map((apiModel) => {
        const prefixedModelName = formatPrefixedModel(getApiModelId(apiModel, m.prefix), m.prefix);
        return {
          value: prefixedModelName,
          label: prefixedModelName,
        };
      });
    } else {
      return [
        {
          value: m.alias,
          label: m.alias,
        },
      ];
    }
  });

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
          setSelectedStatus={(status) => {
            const value = status?.value ?? '';
            setModel(value);
            const alias = modelToAliasMap.get(value);
            if (alias && isApiAlias(alias)) {
              setApiFormat(alias.api_format as ApiFormat);
              setLlmLibertyProvider(alias.llm_liberty?.provider ?? null);
            } else {
              setApiFormat('openai');
              setLlmLibertyProvider(null);
            }
          }}
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
