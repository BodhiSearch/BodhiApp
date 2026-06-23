import { useCallback, useState } from 'react';

import { Check, Copy, X } from 'lucide-react';
import { Prism as SyntaxHighlighter } from 'react-syntax-highlighter';
import { oneDark, oneLight } from 'react-syntax-highlighter/dist/esm/styles/prism';

import { useTheme } from '@/components/ThemeProvider';
import { Badge } from '@/components/ui/badge';
import { Button } from '@/components/ui/button';
import { toast } from '@/hooks/use-toast';
import { copyToClipboard, getClipboardUnavailableMessage } from '@/lib/clipboard';

import { type ExecutionResult, type ResultTab } from './types';

export function ResultSection({
  result,
  activeTab,
  onTabChange,
}: {
  result: ExecutionResult;
  activeTab: ResultTab;
  onTabChange: (tab: ResultTab) => void;
}) {
  const { theme } = useTheme();
  const isSuccess = !result.response.isError;
  const [copied, setCopied] = useState(false);

  const getTabContent = useCallback((): string => {
    switch (activeTab) {
      case 'response':
        if (result.response.isError) return JSON.stringify(result.response.content, null, 2);
        return JSON.stringify(result.response.content, null, 2);
      case 'raw':
        return JSON.stringify(result.response, null, 2);
      case 'request':
        return JSON.stringify({ tool: result.toolName, params: result.params }, null, 2);
    }
  }, [activeTab, result]);

  const handleCopy = async () => {
    try {
      await copyToClipboard(getTabContent());
      setCopied(true);
      setTimeout(() => setCopied(false), 2000);
    } catch {
      toast({
        title: 'Failed to copy',
        description: getClipboardUnavailableMessage() ?? undefined,
        variant: 'destructive',
      });
    }
  };

  return (
    <div className="border rounded-lg" data-testid="mcp-playground-result-section">
      <div className="flex items-center justify-between border-b px-3 py-2">
        <div className="flex items-center gap-2">
          <div data-testid="mcp-playground-result-status" data-test-state={isSuccess ? 'success' : 'error'}>
            {isSuccess ? (
              <Badge variant="default" className="gap-1">
                <Check className="h-3 w-3" /> Success
              </Badge>
            ) : (
              <Badge variant="destructive" className="gap-1">
                <X className="h-3 w-3" /> Error
              </Badge>
            )}
          </div>
          <div className="flex gap-1 ml-4">
            {(['response', 'raw', 'request'] as ResultTab[]).map((tab) => (
              <Button
                key={tab}
                variant={activeTab === tab ? 'secondary' : 'ghost'}
                size="sm"
                onClick={() => onTabChange(tab)}
                data-testid={`mcp-playground-result-tab-${tab}`}
              >
                {tab === 'response' ? 'Response' : tab === 'raw' ? 'Raw JSON' : 'Request'}
              </Button>
            ))}
          </div>
        </div>
        <Button variant="ghost" size="sm" onClick={handleCopy} data-testid="mcp-playground-copy-button">
          {copied ? <Check className="h-4 w-4" /> : <Copy className="h-4 w-4" />}
        </Button>
      </div>
      <div className="max-h-[400px] overflow-auto" data-testid="mcp-playground-result-content">
        {activeTab === 'response' && result.response.isError ? (
          <div className="p-4 text-sm text-destructive font-mono whitespace-pre-wrap">
            {JSON.stringify(result.response.content, null, 2)}
          </div>
        ) : (
          <SyntaxHighlighter
            language="json"
            style={theme === 'dark' ? oneDark : oneLight}
            customStyle={{ margin: 0, borderRadius: 0, fontSize: '0.85rem' }}
          >
            {getTabContent()}
          </SyntaxHighlighter>
        )}
      </div>
    </div>
  );
}
