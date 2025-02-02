'use client';

import { Label } from '@/components/ui/label';
import { Switch } from '@/components/ui/switch';
import { Textarea } from '@/components/ui/textarea';
import { useChatSettings } from '@/hooks/use-chat-settings';

interface SystemPromptProps {
  isLoading?: boolean;
}

export function SystemPrompt({ isLoading = false }: SystemPromptProps) {
  const {
    systemPrompt,
    systemPrompt_enabled,
    setSystemPrompt,
    setSystemPromptEnabled,
  } = useChatSettings();

  // Determine if interactions should be disabled
  const isDisabled = isLoading || !systemPrompt_enabled;

  return (
    <div className="space-y-4">
      <div className="flex items-center justify-between">
        <Label htmlFor="system-prompt" className="text-sm font-medium">
          System Prompt
        </Label>
        <Switch
          id="system-prompt-toggle"
          checked={systemPrompt_enabled}
          onCheckedChange={setSystemPromptEnabled}
          disabled={isLoading}
          size="sm"
        />
      </div>
      <Textarea
        id="system-prompt"
        placeholder="Enter system prompt..."
        value={systemPrompt || ''}
        onChange={(e) => setSystemPrompt(e.target.value || undefined)}
        disabled={isDisabled}
        className={`min-h-[100px] ${isDisabled ? 'opacity-50' : ''}`}
      />
    </div>
  );
}
