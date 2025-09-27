'use client';

import { Label } from '@/components/ui/label';
import { Switch } from '@/components/ui/switch';
import { Textarea } from '@/components/ui/textarea';
import { Tooltip, TooltipContent, TooltipProvider, TooltipTrigger } from '@/components/ui/tooltip';
import { useChatSettings } from '@/hooks/use-chat-settings';
import { HelpCircle } from 'lucide-react';

interface SystemPromptProps {
  isLoading?: boolean;
  tooltip?: string;
}

export function SystemPrompt({ isLoading = false, tooltip }: SystemPromptProps) {
  const { systemPrompt, systemPrompt_enabled, setSystemPrompt, setSystemPromptEnabled } = useChatSettings();

  // Determine if interactions should be disabled
  const isDisabled = isLoading || !systemPrompt_enabled;

  return (
    <div className="space-y-4">
      <div className="flex items-center justify-between">
        <div className="flex items-center gap-2">
          <Label htmlFor="system-prompt">System Prompt</Label>
          {tooltip && (
            <TooltipProvider>
              <Tooltip delayDuration={300}>
                <TooltipTrigger asChild>
                  <HelpCircle className="h-4 w-4 text-muted-foreground hover:text-foreground transition-colors cursor-help" />
                </TooltipTrigger>
                <TooltipContent sideOffset={8}>
                  <p className="max-w-xs text-sm">{tooltip}</p>
                </TooltipContent>
              </Tooltip>
            </TooltipProvider>
          )}
        </div>
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
