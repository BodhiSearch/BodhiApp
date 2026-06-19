import { useMemo, useState } from 'react';

import { Search, X } from 'lucide-react';

import { Tooltip, TooltipContent, TooltipProvider, TooltipTrigger } from '@/components/ui/tooltip';

import './model-selector.css';

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
  selectedModels: string[];
  onToggle: (model: string) => void;
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
    <div className="cam-selected-area">
      <div className="cam-selected-header">
        <span className="cam-selected-label">Selected Models ({selectedModels.length})</span>
        {selectedModels.length > 0 && onClearAll && (
          <button type="button" className="cam-clear-all" onClick={onClearAll} data-testid="clear-all-models">
            Clear All
          </button>
        )}
      </div>
      <div className="cam-chips-row" data-testid="selected-models-list">
        {selectedModels.length === 0 ? (
          <span className="cam-chips-empty" data-testid="no-models-selected">
            No models selected
          </span>
        ) : (
          selectedModels.map((model) => (
            <span key={model} className="cam-chip" data-testid={`selected-model-${model}`}>
              <span>{model}</span>
              <button
                type="button"
                className="cam-chip-x"
                onClick={() => onModelRemove(model)}
                title={`Remove ${model}`}
                aria-label={`Remove ${model}`}
                data-testid={`remove-model-${model}`}
              >
                ×
              </button>
            </span>
          ))
        )}
      </div>
    </div>
  );
}

function FetchModelButton({ onFetchModels, isFetchingModels, canFetch, fetchDisabledReason }: FetchModelButtonProps) {
  return (
    <TooltipProvider>
      <Tooltip>
        <TooltipTrigger asChild>
          <button
            type="button"
            className={`cam-link-btn${isFetchingModels ? ' loading' : ''}`}
            onClick={onFetchModels}
            disabled={!canFetch || isFetchingModels}
            data-testid="fetch-models-button"
            data-fetch-state={isFetchingModels ? 'loading' : 'idle'}
          >
            {isFetchingModels ? (
              <>
                <span className="cam-fetch-spin" />
                Fetching...
              </>
            ) : (
              'Fetch Models'
            )}
          </button>
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
    <div className="cam-search-wrap">
      <span className="cam-search-icon">
        <Search className="h-3.5 w-3.5" />
      </span>
      <input
        className="cam-search-input"
        value={searchTerm}
        onChange={(e) => onSearchChange(e.target.value)}
        placeholder="Search models..."
        disabled={disabled}
        data-testid="model-search-input"
      />
      {searchTerm && (
        <button
          type="button"
          className="cam-search-clear"
          onClick={onClearSearch}
          aria-label="Clear search"
          data-testid="clear-search-button"
        >
          <X className="h-3 w-3" />
        </button>
      )}
    </div>
  );
}

function ModelList({
  models,
  selectedModels,
  onToggle,
  searchTerm,
  isFetchingModels,
  availableModelsCount,
}: ModelListProps) {
  const getEmptyStateMessage = () => {
    if (availableModelsCount === 0) {
      return isFetchingModels ? 'Loading models...' : 'No models available. Fetch models to see options.';
    }
    return searchTerm ? `No models match "${searchTerm}"` : 'No models available.';
  };

  return (
    <div
      className="cam-model-list"
      data-testid="model-list-container"
      data-models-count={models.length}
      data-fetch-state={isFetchingModels ? 'loading' : 'complete'}
    >
      {models.length === 0 ? (
        <div className="cam-no-models" data-testid="empty-model-list">
          {getEmptyStateMessage()}
        </div>
      ) : (
        models.map((model) => {
          const checked = selectedModels.includes(model);
          return (
            <button
              key={model}
              type="button"
              className={`cam-model-item${checked ? ' checked' : ''}`}
              onClick={() => onToggle(model)}
              aria-pressed={checked}
              data-testid={`available-model-${model}`}
            >
              <input type="checkbox" className="cam-model-cb" checked={checked} readOnly tabIndex={-1} />
              <span className="cam-model-name">{model}</span>
            </button>
          );
        })
      )}
    </div>
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
    <div className="cam-available-header">
      <span className="cam-available-label">Available Models</span>
      <div className="cam-available-actions">
        <FetchModelButton
          onFetchModels={onFetchModels}
          isFetchingModels={isFetchingModels}
          canFetch={canFetch}
          fetchDisabledReason={fetchDisabledReason}
        />
        {availableModelsCount > 0 && (
          <button
            type="button"
            className="cam-link-btn"
            onClick={onSelectAll}
            disabled={filteredModelsCount === 0}
            data-testid="select-all-models"
          >
            Select All ({filteredModelsCount})
          </button>
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

  // Filter by search only — selected models STAY in the list (rendered checked + tinted),
  // so clicking a checked row toggles it off. Matches the design.
  const filteredModels = useMemo(() => {
    const searchLower = searchTerm.toLowerCase();
    return availableModels.filter((model) => model.toLowerCase().includes(searchLower));
  }, [availableModels, searchTerm]);

  const handleToggle = (model: string) => {
    if (selectedModels.includes(model)) {
      onModelRemove(model);
    } else {
      onModelSelect(model);
    }
  };

  const handleSelectAll = () => {
    const modelsToAdd = filteredModels.filter((model) => !selectedModels.includes(model));
    if (modelsToAdd.length > 0) {
      if (onModelsSelectAll) {
        onModelsSelectAll([...selectedModels, ...modelsToAdd]);
      } else {
        modelsToAdd.forEach((model) => onModelSelect(model));
      }
    }
  };

  const handleClearAll = () => {
    onModelsSelectAll?.([]);
  };

  const clearSearch = () => setSearchTerm('');

  return (
    <div className="model-selector-cam" data-testid="model-selector">
      <div className="cam-model-box">
        <SelectedModels
          selectedModels={selectedModels}
          onModelRemove={onModelRemove}
          onClearAll={onModelsSelectAll ? handleClearAll : undefined}
        />

        <div className="cam-available-area">
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
            selectedModels={selectedModels}
            onToggle={handleToggle}
            searchTerm={searchTerm}
            isFetchingModels={isFetchingModels}
            availableModelsCount={availableModels.length}
          />
        </div>
      </div>
    </div>
  );
}
