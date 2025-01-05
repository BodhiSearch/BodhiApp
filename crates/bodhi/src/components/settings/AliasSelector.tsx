'use client';

import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from '@/components/ui/select';
import { Label } from '@/components/ui/label';
import { useModels } from '@/hooks/useQuery';
import { useEffect } from 'react';

interface AliasSelectorProps {
  initialAlias?: string;
  onAliasChange?: (alias: string) => void;
  isLoadingCallback?: (isLoading: boolean) => void;
}

export function AliasSelector({ 
  initialAlias, 
  onAliasChange,
  isLoadingCallback 
}: AliasSelectorProps) {
  const { data: modelsResponse, isLoading } = useModels(1, 100, 'alias', 'asc');
  const models = modelsResponse?.data || [];

  // Notify parent component about loading state changes
  useEffect(() => {
    isLoadingCallback?.(isLoading);
  }, [isLoading, isLoadingCallback]);

  return (
    <div className="space-y-4" data-testid="model-selector-loaded">
      <div className="space-y-2">
        <Label>Alias</Label>
        <Select defaultValue={initialAlias} onValueChange={onAliasChange} disabled={isLoading}>
          <SelectTrigger>
            <SelectValue placeholder="Select alias" />
          </SelectTrigger>
          <SelectContent>
            {models.map((model) => (
              <SelectItem key={model.alias} value={model.alias}>
                {model.alias}
              </SelectItem>
            ))}
          </SelectContent>
        </Select>
      </div>
    </div>
  );
}
