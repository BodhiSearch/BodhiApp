import { Label } from '@/components/ui/label';
import { Switch } from '@/components/ui/switch';
import { Textarea } from '@/components/ui/textarea';
import { useChatSettingsStore } from '@/stores/chatSettingsStore';

import { HelpTooltip } from './HelpTooltip';

interface SystemPromptProps {
  isLoading?: boolean;
  tooltip?: string;
}

export function SystemPrompt({ isLoading = false, tooltip }: SystemPromptProps) {
  const systemPrompt = useChatSettingsStore((s) => s.systemPrompt);
  const systemPrompt_enabled = useChatSettingsStore((s) => s.systemPrompt_enabled);
  const setSystemPrompt = useChatSettingsStore((s) => s.setSystemPrompt);
  const setSystemPromptEnabled = useChatSettingsStore((s) => s.setSystemPromptEnabled);

  // Control is shown only when the setting is switched on (design: off → control hidden).
  const showControl = systemPrompt_enabled;

  return (
    <div className="space-y-4">
      <div className="flex items-center justify-between">
        <div className="flex items-center gap-2">
          <Label htmlFor="system-prompt">System Prompt</Label>
          {tooltip && <HelpTooltip text={tooltip} sideOffset={8} />}
        </div>
        <Switch
          id="system-prompt-toggle"
          checked={systemPrompt_enabled}
          onCheckedChange={setSystemPromptEnabled}
          disabled={isLoading}
          size="sm"
        />
      </div>
      {showControl && (
        <Textarea
          id="system-prompt"
          placeholder="Enter system prompt..."
          value={systemPrompt || ''}
          onChange={(e) => setSystemPrompt(e.target.value || undefined)}
          disabled={isLoading}
          className="min-h-[100px]"
        />
      )}
    </div>
  );
}
