import { useState } from 'react';

import { ChevronDown, ChevronRight, Brain } from 'lucide-react';

import { Collapsible, CollapsibleContent, CollapsibleTrigger } from '@/components/ui/collapsible';
import { cn } from '@/lib/utils';

interface ThinkingBlockProps {
  thinking: string;
  isStreaming?: boolean;
}

export function ThinkingBlock({ thinking, isStreaming = false }: ThinkingBlockProps) {
  const [isOpen, setIsOpen] = useState(false);

  if (!thinking) return null;

  return (
    <div
      className="my-2 rounded-lg border border-purple-200 dark:border-purple-800 bg-purple-50/50 dark:bg-purple-950/20"
      data-testid="thinking-block"
    >
      <Collapsible open={isOpen} onOpenChange={setIsOpen}>
        <CollapsibleTrigger
          className="flex w-full items-center justify-between p-2 hover:bg-purple-100/50 dark:hover:bg-purple-900/30 transition-colors"
          data-testid="thinking-block-toggle"
        >
          <div className="flex items-center gap-2">
            <Brain className={cn('h-4 w-4 text-purple-600 dark:text-purple-400', isStreaming && 'animate-pulse')} />
            <span className="text-xs font-medium text-purple-700 dark:text-purple-300">
              {isStreaming ? 'Thinking...' : 'Thought process'}
            </span>
          </div>
          {isOpen ? (
            <ChevronDown className="h-4 w-4 text-purple-500" />
          ) : (
            <ChevronRight className="h-4 w-4 text-purple-500" />
          )}
        </CollapsibleTrigger>
        <CollapsibleContent data-testid="thinking-block-content">
          <div className="border-t border-purple-200 dark:border-purple-800 px-3 py-2">
            <pre className="text-xs text-muted-foreground whitespace-pre-wrap break-words max-h-60 overflow-y-auto">
              {thinking}
            </pre>
          </div>
        </CollapsibleContent>
      </Collapsible>
    </div>
  );
}
