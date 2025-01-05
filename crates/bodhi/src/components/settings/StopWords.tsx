'use client';

import { useState, KeyboardEvent } from 'react';
import { Label } from '@/components/ui/label';
import { Input } from '@/components/ui/input';
import { Badge } from '@/components/ui/badge';
import { Switch } from '@/components/ui/switch';
import { X } from 'lucide-react';

interface StopWordsProps {
  initialStopWords?: string[];
  initialEnabled?: boolean;
  isLoading?: boolean;
}

export function StopWords({
  initialStopWords = [],
  initialEnabled = true,
  isLoading = false
}: StopWordsProps) {
  const [isEnabled, setIsEnabled] = useState(initialEnabled);
  const [stopWords, setStopWords] = useState<string[]>(initialStopWords);
  const [inputValue, setInputValue] = useState('');

  const handleKeyDown = (e: KeyboardEvent<HTMLInputElement>) => {
    if (e.key === 'Enter' && inputValue.trim()) {
      e.preventDefault();
      if (!stopWords.includes(inputValue.trim())) {
        setStopWords([...stopWords, inputValue.trim()]);
        setInputValue('');
      }
    }
  };

  const removeStopWord = (wordToRemove: string) => {
    setStopWords(stopWords.filter((word) => word !== wordToRemove));
  };

  // Determine if interactions should be disabled
  const isDisabled = isLoading || !isEnabled;

  return (
    <div className="space-y-2">
      <div className="flex items-center justify-between">
        <Label htmlFor="stop-words" className="text-sm font-medium">
          Stop Words
        </Label>
        <Switch
          id="stop-words-toggle"
          checked={isEnabled}
          onCheckedChange={setIsEnabled}
          disabled={isLoading}
        />
      </div>
      <div className="space-y-2">
        <Input
          id="stop-words"
          placeholder="Type and press Enter to add stop words..."
          value={inputValue}
          onChange={(e) => setInputValue(e.target.value)}
          onKeyDown={handleKeyDown}
          disabled={isDisabled}
        />
        {stopWords.length > 0 && (
          <div className="flex flex-wrap gap-2">
            {stopWords.map((word) => (
              <Badge
                key={word}
                variant="secondary"
                className={`group flex items-center gap-1 pr-1 ${isDisabled ? 'opacity-50' : ''}`}
              >
                {word}
                <button
                  onClick={() => removeStopWord(word)}
                  className="ml-1 rounded-full p-1 hover:bg-secondary"
                  aria-label={`Remove ${word}`}
                  disabled={isDisabled}
                >
                  <X className="h-3 w-3" />
                </button>
              </Badge>
            ))}
          </div>
        )}
      </div>
    </div>
  );
}
