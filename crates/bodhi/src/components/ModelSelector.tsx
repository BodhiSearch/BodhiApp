'use client';

import React, { useState, useMemo } from 'react';
import { Button } from '@/components/ui/button';
import { Input } from '@/components/ui/input';
import { Label } from '@/components/ui/label';
import { Badge } from '@/components/ui/badge';
import { ScrollArea } from '@/components/ui/scroll-area';
import { X, Search, Loader2 } from 'lucide-react';
import { Tooltip, TooltipContent, TooltipProvider, TooltipTrigger } from '@/components/ui/tooltip';

interface ModelSelectorProps {
  selectedModels: string[];
  availableModels: string[];
  onModelSelect: (model: string) => void;
  onModelRemove: (model: string) => void;
  onModelsSelectAll?: (models: string[]) => void; // Optional batch select function
  onFetchModels: () => void;
  isFetchingModels: boolean;
  canFetch: boolean; // disabled if no api key/base url
  fetchDisabledReason?: string; // tooltip text when fetch is disabled
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

  // Filter available models based on search term, excluding already selected models
  const filteredModels = useMemo(() => {
    const searchLower = searchTerm.toLowerCase();
    return availableModels.filter(
      (model) => model.toLowerCase().includes(searchLower) && !selectedModels.includes(model)
    );
  }, [availableModels, searchTerm, selectedModels]);

  const handleSelectAll = () => {
    // Add all filtered models to selected at once
    const modelsToAdd = filteredModels.filter((model) => !selectedModels.includes(model));
    if (modelsToAdd.length > 0) {
      if (onModelsSelectAll) {
        // Use batch select if available
        const allSelected = [...selectedModels, ...modelsToAdd];
        onModelsSelectAll(allSelected);
      } else {
        // Fallback to individual selections (but don't trigger form submission)
        modelsToAdd.forEach((model) => {
          onModelSelect(model);
        });
      }
    }
  };

  const clearSearch = () => {
    setSearchTerm('');
  };

  return (
    <div className="space-y-4">
      {/* Selected Models Section */}
      <div className="space-y-2">
        <div className="flex items-center justify-between">
          <Label>Selected Models ({selectedModels.length})</Label>
          {selectedModels.length > 0 && (
            <Button
              variant="link"
              size="sm"
              onClick={() => onModelsSelectAll && onModelsSelectAll([])}
              className="h-auto p-0 text-destructive hover:text-destructive"
            >
              Clear All
            </Button>
          )}
        </div>
        {selectedModels.length > 0 ? (
          <div className="flex flex-wrap gap-2 p-3 border rounded-md bg-muted/50 min-h-[48px]">
            {selectedModels.map((model) => (
              <Badge key={model} variant="secondary" className="flex items-center gap-1">
                <span>{model}</span>
                <Button
                  variant="ghost"
                  size="sm"
                  className="h-4 w-4 p-0 hover:bg-destructive hover:text-destructive-foreground"
                  onClick={() => onModelRemove(model)}
                >
                  <X className="h-3 w-3" />
                </Button>
              </Badge>
            ))}
          </div>
        ) : (
          <div className="p-3 border rounded-md bg-muted/50 min-h-[48px] flex items-center justify-center text-muted-foreground">
            No models selected
          </div>
        )}
      </div>

      {/* Available Models Section */}
      <div className="space-y-2">
        <div className="flex items-center justify-between">
          <Label>Available Models</Label>
          {availableModels.length > 0 ? (
            <Button
              variant="link"
              size="sm"
              onClick={handleSelectAll}
              disabled={filteredModels.length === 0}
              className="h-auto p-0"
            >
              Select All ({filteredModels.length})
            </Button>
          ) : (
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
          )}
        </div>

        {/* Search Input */}
        <div className="relative">
          <Search className="absolute left-2 top-2.5 h-4 w-4 text-muted-foreground" />
          <Input
            value={searchTerm}
            onChange={(e) => setSearchTerm(e.target.value)}
            className="pl-8 pr-8"
            placeholder="Search models..."
            disabled={availableModels.length === 0}
          />
          {searchTerm && (
            <Button variant="ghost" size="sm" className="absolute right-1 top-1 h-6 w-6 p-0" onClick={clearSearch}>
              <X className="h-3 w-3" />
            </Button>
          )}
        </div>

        {/* Scrollable Model List */}
        <ScrollArea className="h-[120px] border rounded-md">
          {availableModels.length === 0 ? (
            <div className="p-4 text-center text-muted-foreground">
              {isFetchingModels ? 'Loading models...' : 'No models available. Fetch models to see options.'}
            </div>
          ) : filteredModels.length === 0 ? (
            <div className="p-4 text-center text-muted-foreground">
              {searchTerm ? 'No models match your search.' : 'All models are already selected.'}
            </div>
          ) : (
            <div className="p-1">
              {filteredModels.map((model) => (
                <div
                  key={model}
                  className="flex items-center p-2 hover:bg-accent cursor-pointer rounded-sm"
                  onClick={() => onModelSelect(model)}
                >
                  <span className="text-sm">{model}</span>
                </div>
              ))}
            </div>
          )}
        </ScrollArea>
      </div>
    </div>
  );
}
