import { KeyboardEvent, useState } from 'react';

import { X } from 'lucide-react';

import { Badge } from '@/components/ui/badge';
import { Button } from '@/components/ui/button';
import { Input } from '@/components/ui/input';
import { Label } from '@/components/ui/label';
import { Switch } from '@/components/ui/switch';
import { useChatSettingsStore } from '@/stores/chatSettingsStore';

import { HelpTooltip } from './HelpTooltip';

interface StopWordsProps {
  isLoading?: boolean;
  tooltip?: string;
}

export function StopWords({ isLoading = false, tooltip }: StopWordsProps) {
  const stop = useChatSettingsStore((s) => s.stop);
  const stop_enabled = useChatSettingsStore((s) => s.stop_enabled);
  const setStop = useChatSettingsStore((s) => s.setStop);
  const setStopEnabled = useChatSettingsStore((s) => s.setStopEnabled);
  const [inputValue, setInputValue] = useState('');

  const stopWords = stop || [];

  const handleKeyDown = (e: KeyboardEvent<HTMLInputElement>) => {
    if (e.key === 'Enter' && inputValue.trim()) {
      e.preventDefault();
      if (!stopWords.includes(inputValue.trim())) {
        const newStopWords = [...stopWords, inputValue.trim()];
        setStop(newStopWords);
        setInputValue('');
      }
    }
  };

  const removeStopWord = (wordToRemove: string) => {
    const newStopWords = stopWords.filter((word) => word !== wordToRemove);
    setStop(newStopWords);
  };

  // Control is shown only when the setting is switched on (design: off → control hidden).
  const showControl = stop_enabled;

  return (
    <div className="space-y-2">
      <div className="flex items-center justify-between">
        <div className="flex items-center gap-2">
          <Label htmlFor="stop-words">Stop Words</Label>
          {tooltip && <HelpTooltip text={tooltip} sideOffset={8} />}
        </div>
        <Switch
          id="stop-words-toggle"
          checked={stop_enabled}
          onCheckedChange={setStopEnabled}
          disabled={isLoading}
          size="sm"
        />
      </div>
      {showControl && (
        <div className="space-y-2">
          <Input
            id="stop-words"
            placeholder="Type and press Enter to add stop words..."
            value={inputValue}
            onChange={(e) => setInputValue(e.target.value)}
            onKeyDown={handleKeyDown}
            disabled={isLoading}
          />
          {stopWords.length > 0 && (
            <div className="flex flex-wrap gap-2">
              {stopWords.map((word) => (
                <Badge key={word} variant="secondary" className="group flex items-center gap-1 pr-1">
                  {word}
                  <Button
                    onClick={() => removeStopWord(word)}
                    className="ml-1 rounded-full p-1 hover:bg-secondary"
                    aria-label={`Remove ${word}`}
                    disabled={isLoading}
                  >
                    <X className="h-3 w-3" />
                  </Button>
                </Badge>
              ))}
            </div>
          )}
        </div>
      )}
    </div>
  );
}
