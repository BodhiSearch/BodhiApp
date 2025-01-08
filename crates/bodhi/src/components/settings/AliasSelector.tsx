'use client';

import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from '@/components/ui/select';
import { Label } from '@/components/ui/label';
import { useChatSettings } from '@/hooks/use-chat-settings';

interface AliasSelectorProps {
  models: Array<{ alias: string }>;
  isLoading?: boolean;
}

export function AliasSelector({
  models,
  isLoading = false,
}: AliasSelectorProps) {
  const { model, setModel } = useChatSettings();

  return (
    <div className="space-y-4" data-testid="model-selector-loaded">
      <div className="space-y-2">
        <Label>Alias/Model</Label>
        <Select
          defaultValue={model}
          onValueChange={setModel}
          disabled={isLoading}
        >
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
