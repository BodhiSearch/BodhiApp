'use client';

import { Label } from '@/components/ui/label';
import { useChatSettings } from '@/hooks/use-chat-settings';
import { ComboBoxResponsive } from '@/components/Combobox';
import { CopyButton } from '@/components/CopyButton';

interface AliasSelectorProps {
  models: Array<{ alias: string }>;
  isLoading?: boolean;
}

export function AliasSelector({
  models,
  isLoading = false,
}: AliasSelectorProps) {
  const { model, setModel } = useChatSettings();

  // Transform models array to match ComboBoxResponsive's Status type
  const modelStatuses = models.map((m) => ({
    value: m.alias,
    label: m.alias,
  }));

  // Find the currently selected model
  const selectedStatus = model ? { value: model, label: model } : null;

  return (
    <div className="space-y-4" data-testid="model-selector-loaded">
      <div className="space-y-2">
        <div className="flex items-center justify-between group">
          <Label>Alias/Model</Label>
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
          loading={isLoading}
        />
      </div>
    </div>
  );
}
