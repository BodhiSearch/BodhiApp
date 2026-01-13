'use client';

import { useState } from 'react';

import { ModelSelector } from '@/components/ModelSelector';
import { Label } from '@/components/ui/label';

import { ApiProvider } from '../providers/constants';

interface ModelSelectionSectionProps {
  selectedModels: string[];
  availableModels: string[];
  onModelSelect: (model: string) => void;
  onModelRemove: (model: string) => void;
  onModelsSelectAll?: (models: string[]) => void;
  onFetchModels: () => void;
  isFetchingModels: boolean;
  canFetch: boolean;
  fetchDisabledReason?: string;
  error?: string;
  provider?: ApiProvider | null;
  autoSelectCommon?: boolean;
  fetchStatus?: 'idle' | 'loading' | 'success' | 'error';
  disabled?: boolean;
  'data-testid'?: string;
}

export function ModelSelectionSection({
  selectedModels,
  availableModels,
  onModelSelect,
  onModelRemove,
  onModelsSelectAll,
  onFetchModels,
  isFetchingModels,
  canFetch,
  fetchDisabledReason,
  error,
  provider,
  autoSelectCommon = false,
  fetchStatus = 'idle',
  disabled = false,
  'data-testid': testId = 'model-selection-section',
}: ModelSelectionSectionProps) {
  // Auto-select common models when they become available
  useState(() => {
    if (autoSelectCommon && provider?.commonModels.length && availableModels.length) {
      const commonModelsAvailable = provider.commonModels.filter(
        (model) => availableModels.includes(model) && !selectedModels.includes(model)
      );
      if (commonModelsAvailable.length > 0 && onModelsSelectAll) {
        const newSelection = [...selectedModels, ...commonModelsAvailable.slice(0, 3)];
        onModelsSelectAll(newSelection);
      }
    }
  });

  return (
    <div
      className={`space-y-2 ${disabled ? 'opacity-50 pointer-events-none' : ''}`}
      data-testid={testId}
      data-teststate={disabled ? 'disabled' : 'enabled'}
    >
      <Label>Model Selection</Label>

      {disabled && (
        <p className="text-sm text-muted-foreground">
          Model selection is disabled when using &quot;Forward all requests with prefix&quot; mode.
        </p>
      )}

      {!disabled && provider && (
        <p className="text-sm text-muted-foreground">
          Select which {provider.name} models you&apos;d like to use.
          {autoSelectCommon && provider.commonModels.length > 0 && <span> Popular models will be auto-selected.</span>}
        </p>
      )}

      <div
        data-testid="fetch-models-container"
        data-models-fetched={availableModels.length > 0}
        data-can-fetch={canFetch}
        data-fetch-state={fetchStatus}
      >
        <ModelSelector
          selectedModels={selectedModels}
          availableModels={availableModels}
          onModelSelect={onModelSelect}
          onModelRemove={onModelRemove}
          onModelsSelectAll={onModelsSelectAll}
          onFetchModels={onFetchModels}
          isFetchingModels={isFetchingModels}
          canFetch={canFetch}
          fetchDisabledReason={fetchDisabledReason}
        />
      </div>

      {error && (
        <p className="text-sm text-destructive" data-testid={`${testId}-error`}>
          {error}
        </p>
      )}

      {!disabled && !availableModels.length && canFetch && !isFetchingModels && (
        <p className="text-xs text-muted-foreground">
          Click &quot;Fetch Models&quot; to discover available models from your provider.
        </p>
      )}
    </div>
  );
}
