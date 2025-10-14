'use client';

import { useState, useCallback, memo } from 'react';
import { Copy, Check } from 'lucide-react';
import { cn } from '@/lib/utils';

interface CopyableCodeBlockProps {
  command: string;
  language?: 'bash' | 'yaml';
  className?: string;
}

function CopyableCodeBlockComponent({ command, language = 'bash', className }: CopyableCodeBlockProps) {
  const [copied, setCopied] = useState(false);

  const handleCopy = useCallback(async () => {
    try {
      await navigator.clipboard.writeText(command);
      setCopied(true);
      setTimeout(() => setCopied(false), 2000);
    } catch (err) {
      console.error('Failed to copy:', err);
    }
  }, [command]);

  return (
    <div className="relative group">
      <pre
        onClick={handleCopy}
        className={cn(
          'relative overflow-x-auto rounded-lg border p-4 transition-all cursor-pointer max-h-[400px]',
          'bg-slate-50 border-slate-200 hover:border-slate-300',
          'dark:bg-slate-900 dark:border-slate-800 dark:hover:border-slate-700',
          copied && 'border-green-500 dark:border-green-500',
          className
        )}
      >
        <code className="text-sm font-mono text-slate-800 dark:text-slate-200 whitespace-pre-wrap break-all">
          {command}
        </code>

        {/* Copy icon - fades in on hover */}
        <div className="absolute top-3 right-3 opacity-0 group-hover:opacity-100 transition-opacity">
          {copied ? (
            <Check className="h-5 w-5 text-green-600 dark:text-green-400" />
          ) : (
            <Copy className="h-5 w-5 text-slate-500 dark:text-slate-400" />
          )}
        </div>
      </pre>
    </div>
  );
}

export const CopyableCodeBlock = memo(CopyableCodeBlockComponent);
