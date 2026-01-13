'use client';

import React, { useState, useMemo } from 'react';

import { X, Search, Loader2 } from 'lucide-react';

import { Badge } from '@/components/ui/badge';
import { Button } from '@/components/ui/button';
import { Input } from '@/components/ui/input';
import { Label } from '@/components/ui/label';
import { ScrollArea } from '@/components/ui/scroll-area';
import { Tooltip, TooltipContent, TooltipProvider, TooltipTrigger } from '@/components/ui/tooltip';

interface ModelSelectorProps {
  selectedModels: string[];
  availableModels: string[];
  onModelSelect: (model: string) => void;
  onModelRemove: (model: string) => void;
  onModelsSelectAll?: (models: string[]) => void;
  onFetchModels: () => void;
  isFetchingModels: boolean;
  canFetch: boolean;
  fetchDisabledReason?: string;
}

interface SelectedModelsProps {
  selectedModels: string[];
  onModelRemove: (model: string) => void;
  onClearAll?: () => void;
}

interface FetchModelButtonProps {
  onFetchModels: () => void;
  isFetchingModels: boolean;
  canFetch: boolean;
  fetchDisabledReason?: string;
}

interface ModelSearchProps {
  searchTerm: string;
  onSearchChange: (term: string) => void;
  onClearSearch: () => void;
  disabled?: boolean;
}

interface ModelListProps {
  models: string[];
  onModelSelect: (model: string) => void;
  searchTerm: string;
  isFetchingModels: boolean;
  availableModelsCount: number;
}

interface AvailableModelsHeaderProps {
  filteredModelsCount: number;
  availableModelsCount: number;
  onSelectAll: () => void;
  onFetchModels: () => void;
  isFetchingModels: boolean;
  canFetch: boolean;
  fetchDisabledReason?: string;
}

function SelectedModels({ selectedModels, onModelRemove, onClearAll }: SelectedModelsProps) {
  return (
    <div className="space-y-2">
      <div className="flex items-center justify-between">
        <Label>Selected Models ({selectedModels.length})</Label>
        {selectedModels.length > 0 && onClearAll && (
          <Button
            variant="link"
            size="sm"
            onClick={onClearAll}
            className="h-auto p-0 text-destructive hover:text-destructive"
            data-testid="clear-all-models"
          >
            Clear All
          </Button>
        )}
      </div>
      {selectedModels.length > 0 ? (
        <div
          className="flex flex-wrap gap-2 p-3 border rounded-md bg-muted/50 min-h-[48px]"
          data-testid="selected-models-list"
        >
          {selectedModels.map((model) => (
            <Badge
              key={model}
              variant="secondary"
              className="flex items-center gap-1"
              data-testid={`selected-model-${model}`}
            >
              <span>{model}</span>
              <Button
                variant="ghost"
                size="sm"
                className="h-4 w-4 p-0 hover:bg-destructive hover:text-destructive-foreground"
                onClick={() => onModelRemove(model)}
                data-testid={`remove-model-${model}`}
              >
                <X className="h-3 w-3" />
              </Button>
            </Badge>
          ))}
        </div>
      ) : (
        <div
          className="p-3 border rounded-md bg-muted/50 min-h-[48px] flex items-center justify-center text-muted-foreground"
          data-testid="no-models-selected"
        >
          No models selected
        </div>
      )}
    </div>
  );
}

function FetchModelButton({ onFetchModels, isFetchingModels, canFetch, fetchDisabledReason }: FetchModelButtonProps) {
  return (
    <TooltipProvider>
      <Tooltip>
        <TooltipTrigger asChild>
          <span>
            <Button
              variant="link"
              size="sm"
              onClick={onFetchModels}
              disabled={!canFetch || isFetchingModels}
              className="h-auto p-0"
              data-testid="fetch-models-button"
              data-fetch-state={isFetchingModels ? 'loading' : 'idle'}
            >
              {isFetchingModels ? (
                <>
                  <Loader2 className="h-3 w-3 mr-1 animate-spin" />
                  Fetching...
                </>
              ) : (
                'Fetch Models'
              )}
            </Button>
          </span>
        </TooltipTrigger>
        {!canFetch && !isFetchingModels && (
          <TooltipContent>
            <p>{fetchDisabledReason}</p>
          </TooltipContent>
        )}
      </Tooltip>
    </TooltipProvider>
  );
}

function ModelSearch({ searchTerm, onSearchChange, onClearSearch, disabled = false }: ModelSearchProps) {
  return (
    <div className="relative">
      <Search className="absolute left-2 top-2.5 h-4 w-4 text-muted-foreground" />
      <Input
        value={searchTerm}
        onChange={(e) => onSearchChange(e.target.value)}
        className="pl-8 pr-8"
        placeholder="Search models..."
        disabled={disabled}
        data-testid="model-search-input"
      />
      {searchTerm && (
        <Button
          variant="ghost"
          size="sm"
          className="absolute right-1 top-1 h-6 w-6 p-0"
          onClick={onClearSearch}
          data-testid="clear-search-button"
        >
          <X className="h-3 w-3" />
        </Button>
      )}
    </div>
  );
}

function ModelList({ models, onModelSelect, searchTerm, isFetchingModels, availableModelsCount }: ModelListProps) {
  const getEmptyStateMessage = () => {
    if (availableModelsCount === 0) {
      return isFetchingModels ? 'Loading models...' : 'No models available. Fetch models to see options.';
    }
    return searchTerm ? 'No models match your search.' : 'All models are already selected.';
  };

  return (
    <ScrollArea
      className="h-[120px] border rounded-md"
      data-testid="model-list-container"
      data-models-count={models.length}
      data-fetch-state={isFetchingModels ? 'loading' : 'complete'}
    >
      {models.length === 0 ? (
        <div className="p-4 text-center text-muted-foreground" data-testid="empty-model-list">
          {getEmptyStateMessage()}
        </div>
      ) : (
        <div className="p-1">
          {models.map((model) => (
            <div
              key={model}
              className="flex items-center p-2 hover:bg-accent cursor-pointer rounded-sm"
              onClick={() => onModelSelect(model)}
              data-testid={`available-model-${model}`}
            >
              <span className="text-sm">{model}</span>
            </div>
          ))}
        </div>
      )}
    </ScrollArea>
  );
}

function AvailableModelsHeader({
  filteredModelsCount,
  availableModelsCount,
  onSelectAll,
  onFetchModels,
  isFetchingModels,
  canFetch,
  fetchDisabledReason,
}: AvailableModelsHeaderProps) {
  return (
    <div className="flex items-center justify-between">
      <Label>Available Models</Label>
      <div className="flex items-center gap-2">
        <FetchModelButton
          onFetchModels={onFetchModels}
          isFetchingModels={isFetchingModels}
          canFetch={canFetch}
          fetchDisabledReason={fetchDisabledReason}
        />
        {availableModelsCount > 0 && (
          <Button
            variant="link"
            size="sm"
            onClick={onSelectAll}
            disabled={filteredModelsCount === 0}
            className="h-auto p-0"
            data-testid="select-all-models"
          >
            Select All ({filteredModelsCount})
          </Button>
        )}
      </div>
    </div>
  );
}

export function ModelSelector({
  selectedModels,
  availableModels,
  onModelSelect,
  onModelRemove,
  onModelsSelectAll,
  onFetchModels,
  isFetchingModels,
  canFetch,
  fetchDisabledReason = 'Please provide API key and base URL to fetch models',
}: ModelSelectorProps) {
  const [searchTerm, setSearchTerm] = useState('');

  const filteredModels = useMemo(() => {
    const searchLower = searchTerm.toLowerCase();
    return availableModels.filter(
      (model) => model.toLowerCase().includes(searchLower) && !selectedModels.includes(model)
    );
  }, [availableModels, searchTerm, selectedModels]);

  const handleSelectAll = () => {
    const modelsToAdd = filteredModels.filter((model) => !selectedModels.includes(model));
    if (modelsToAdd.length > 0) {
      if (onModelsSelectAll) {
        const allSelected = [...selectedModels, ...modelsToAdd];
        onModelsSelectAll(allSelected);
      } else {
        modelsToAdd.forEach((model) => {
          onModelSelect(model);
        });
      }
    }
  };

  const handleClearAll = () => {
    onModelsSelectAll?.([]);
  };

  const clearSearch = () => {
    setSearchTerm('');
  };

  return (
    <div className="space-y-4" data-testid="model-selector">
      <SelectedModels
        selectedModels={selectedModels}
        onModelRemove={onModelRemove}
        onClearAll={onModelsSelectAll ? handleClearAll : undefined}
      />

      <div className="space-y-2">
        <AvailableModelsHeader
          filteredModelsCount={filteredModels.length}
          availableModelsCount={availableModels.length}
          onSelectAll={handleSelectAll}
          onFetchModels={onFetchModels}
          isFetchingModels={isFetchingModels}
          canFetch={canFetch}
          fetchDisabledReason={fetchDisabledReason}
        />

        <ModelSearch
          searchTerm={searchTerm}
          onSearchChange={setSearchTerm}
          onClearSearch={clearSearch}
          disabled={availableModels.length === 0}
        />

        <ModelList
          models={filteredModels}
          onModelSelect={onModelSelect}
          searchTerm={searchTerm}
          isFetchingModels={isFetchingModels}
          availableModelsCount={availableModels.length}
        />
      </div>
    </div>
  );
}
