'use client';

import { Label } from '@/components/ui/label';
import { Switch } from '@/components/ui/switch';
import { Textarea } from '@/components/ui/textarea';
import { useState } from 'react';

interface SystemPromptProps {
  isLoading?: boolean;
  initialEnabled?: boolean;
}

export function SystemPrompt({ 
  isLoading = false,
  initialEnabled = true
}: SystemPromptProps) {
  const [isEnabled, setIsEnabled] = useState(initialEnabled);
  const [prompt, setPrompt] = useState('');

  // Determine if interactions should be disabled
  const isDisabled = isLoading || !isEnabled;

  return (
    <div className="space-y-4">
      <div className="flex items-center justify-between">
        <Label htmlFor="system-prompt" className="text-sm font-medium">
          System Prompt
        </Label>
        <Switch
          id="system-prompt-toggle"
          checked={isEnabled}
          onCheckedChange={setIsEnabled}
          disabled={isLoading}
        />
      </div>
      <Textarea
        id="system-prompt"
        placeholder="Enter system prompt..."
        value={prompt}
        onChange={(e) => setPrompt(e.target.value)}
        disabled={isDisabled}
        className={`min-h-[100px] ${isDisabled ? 'opacity-50' : ''}`}
      />
    </div>
  );
}
