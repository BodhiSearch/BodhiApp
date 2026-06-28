import { useEffect, useMemo, useRef, useState } from 'react';

import { AliasResponse } from '@bodhiapp/ts-client';
import type { ApiFormat } from '@bodhiapp/ts-client';
import { Check, ChevronDown, RefreshCw } from 'lucide-react';

import { CopyButton } from '@/components/CopyButton';
import { Button } from '@/components/ui/button';
import { Label } from '@/components/ui/label';
import { Tooltip, TooltipContent, TooltipProvider, TooltipTrigger } from '@/components/ui/tooltip';
import { modelKeys } from '@/hooks/models/constants';
import { useQueryClient } from '@/hooks/useQuery';
import { cn, isApiAlias } from '@/lib/utils';
import { formatPrefixedModel, getApiModelId } from '@/schemas/apiModel';
import { useChatSettingsStore } from '@/stores/chatSettingsStore';

import { HelpTooltip } from './HelpTooltip';

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

  // Every selectable model name (aliases + prefixed API models), used as autocomplete suggestions.
  const allModels = useMemo(
    () =>
      models.flatMap((m) =>
        isApiAlias(m)
          ? (m.models || []).map((apiModel) => formatPrefixedModel(getApiModelId(apiModel, m.prefix), m.prefix))
          : [m.alias]
      ),
    [models]
  );

  // Free-text autocomplete: the input accepts ANY value; the list is suggestions, not a constraint.
  const [open, setOpen] = useState(false);
  const comboRef = useRef<HTMLDivElement>(null);

  useEffect(() => {
    if (!open) return;
    const onDocClick = (e: MouseEvent) => {
      if (comboRef.current && !comboRef.current.contains(e.target as Node)) setOpen(false);
    };
    document.addEventListener('mousedown', onDocClick);
    return () => document.removeEventListener('mousedown', onDocClick);
  }, [open]);

  const commitModel = (value: string) => {
    setModel(value);
    const alias = modelToAliasMap.get(value);
    if (alias && isApiAlias(alias)) {
      setApiFormat(alias.api_format as ApiFormat);
      setLlmLibertyProvider(alias.llm_liberty?.provider ?? null);
    } else {
      // Unknown / free-typed model: keep openai as the default format.
      setApiFormat('openai');
      setLlmLibertyProvider(null);
    }
  };

  // Matching suggestions float to the top (current model first), then the rest A→Z.
  const q = (model || '').toLowerCase().trim();
  const byName = (a: string, b: string) => a.localeCompare(b);
  const matching = allModels
    .filter((m) => m.toLowerCase().includes(q))
    .sort((a, b) => (a === model ? -1 : b === model ? 1 : byName(a, b)));
  const nonMatching = allModels.filter((m) => !m.toLowerCase().includes(q)).sort(byName);
  const renderOption = (m: string) => (
    <button
      key={m}
      type="button"
      data-testid={`combobox-option-${m}`}
      className={cn('chat-model-opt', m === model && 'sel')}
      onMouseDown={(e) => e.preventDefault()}
      onClick={() => {
        commitModel(m);
        setOpen(false);
      }}
    >
      <span className="name">{m}</span>
      {m === model && <Check className="h-3.5 w-3.5" />}
    </button>
  );

  return (
    <div className="space-y-4" data-testid="model-selector-loaded">
      <div className="space-y-2">
        <div className="flex items-center justify-between group">
          <div className="flex items-center gap-2">
            <Label>Alias/Model</Label>
            <HelpTooltip text={tooltip} sideOffset={8} className="data-[side=bottom]:slide-in-from-top-2" />
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
        <div className={cn('chat-model-combo', open && 'open')} ref={comboRef}>
          <input
            id="model-selector"
            data-testid="model-selector-trigger"
            className="chat-model-input"
            type="text"
            value={model ?? ''}
            placeholder="Search or type a model name…"
            spellCheck={false}
            autoComplete="off"
            disabled={isLoading}
            onChange={(e) => {
              commitModel(e.target.value);
              setOpen(true);
            }}
            onFocus={() => setOpen(true)}
            onClick={() => setOpen(true)}
            onKeyDown={(e) => {
              if (e.key === 'Escape') {
                setOpen(false);
                e.currentTarget.blur();
              }
            }}
          />
          <span className="chat-model-caret">
            <ChevronDown className="h-3.5 w-3.5" />
          </span>
          {open && (
            <div className="chat-model-pop">
              {matching.map(renderOption)}
              {matching.length > 0 && nonMatching.length > 0 && <div className="chat-model-div" />}
              {nonMatching.map(renderOption)}
              {allModels.length === 0 && <div className="chat-model-empty">No models available</div>}
            </div>
          )}
        </div>
      </div>
    </div>
  );
}
