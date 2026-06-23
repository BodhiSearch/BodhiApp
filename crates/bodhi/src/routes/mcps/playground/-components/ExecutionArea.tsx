import { useCallback, useEffect, useState } from 'react';

import { Loader2 } from 'lucide-react';

import { Button } from '@/components/ui/button';
import { Textarea } from '@/components/ui/textarea';
import { type McpClientTool, type McpToolCallResult } from '@/hooks/mcps/useMcpClient';

import { FormInput } from './FormInput';
import { ResultSection } from './ResultSection';
import { buildDefaultParams, type ExecutionResult, type InputSchema, type ResultTab } from './types';

export function ExecutionArea({
  tool,
  callTool,
}: {
  tool: McpClientTool;
  callTool: (name: string, args: Record<string, unknown>) => Promise<McpToolCallResult>;
}) {
  const schema = (tool.inputSchema as InputSchema) || {};
  const [inputMode, setInputMode] = useState<'form' | 'json'>('form');
  const [params, setParams] = useState<Record<string, unknown>>(() => buildDefaultParams(schema));
  const [jsonText, setJsonText] = useState('');
  const [jsonError, setJsonError] = useState<string | null>(null);
  const [result, setResult] = useState<ExecutionResult | null>(null);
  const [resultTab, setResultTab] = useState<ResultTab>('response');
  const [isExecuting, setIsExecuting] = useState(false);

  useEffect(() => {
    setParams(buildDefaultParams(schema));
    setJsonText(JSON.stringify(buildDefaultParams(schema), null, 2));
    setJsonError(null);
    setResult(null);
    setResultTab('response');
    // Reset form only when switching tools; schema is derived from tool.name and would loop if listed.
  }, [tool.name]); // eslint-disable-line react-hooks/exhaustive-deps

  const currentParams = useCallback((): Record<string, unknown> => {
    if (inputMode === 'json') {
      try {
        return JSON.parse(jsonText);
      } catch {
        return params;
      }
    }
    return params;
  }, [inputMode, jsonText, params]);

  const handleFormChange = useCallback((newParams: Record<string, unknown>) => {
    setParams(newParams);
    setJsonText(JSON.stringify(newParams, null, 2));
    setJsonError(null);
  }, []);

  const handleJsonChange = useCallback((text: string) => {
    setJsonText(text);
    try {
      const parsed = JSON.parse(text);
      setParams(parsed);
      setJsonError(null);
    } catch {
      setJsonError('Invalid JSON');
    }
  }, []);

  const handleSwitchToJson = useCallback(() => {
    setJsonText(JSON.stringify(params, null, 2));
    setJsonError(null);
    setInputMode('json');
  }, [params]);

  const handleSwitchToForm = useCallback(() => {
    if (!jsonError) {
      try {
        setParams(JSON.parse(jsonText));
      } catch {
        // keep existing params
      }
    }
    setInputMode('form');
  }, [jsonText, jsonError]);

  const cleanParams = (p: Record<string, unknown>): Record<string, unknown> => {
    const cleaned: Record<string, unknown> = {};
    for (const [k, v] of Object.entries(p)) {
      if (v !== '' && v !== undefined) {
        cleaned[k] = v;
      }
    }
    return cleaned;
  };

  const handleExecute = async () => {
    const p = currentParams();
    const cleaned = cleanParams(p);
    setIsExecuting(true);
    try {
      const response = await callTool(tool.name, cleaned);
      setResult({ response, toolName: tool.name, params: cleaned });
      setResultTab('response');
    } catch (err) {
      setResult({
        response: { content: err instanceof Error ? err.message : 'Execution failed', isError: true },
        toolName: tool.name,
        params: cleaned,
      });
      setResultTab('response');
    } finally {
      setIsExecuting(false);
    }
  };

  return (
    <div className="flex-1 p-4 overflow-y-auto space-y-4">
      <div>
        <h2 className="text-lg font-semibold" data-testid="mcp-playground-tool-name">
          {tool.name}
        </h2>
        {tool.description && <p className="text-sm text-muted-foreground mt-1">{tool.description}</p>}
      </div>

      <div className="flex gap-1">
        <Button
          variant={inputMode === 'form' ? 'secondary' : 'ghost'}
          size="sm"
          onClick={handleSwitchToForm}
          data-testid="mcp-playground-input-mode-form"
        >
          Form
        </Button>
        <Button
          variant={inputMode === 'json' ? 'secondary' : 'ghost'}
          size="sm"
          onClick={handleSwitchToJson}
          data-testid="mcp-playground-input-mode-json"
        >
          JSON
        </Button>
      </div>

      {inputMode === 'form' ? (
        <FormInput schema={schema} values={params} onChange={handleFormChange} />
      ) : (
        <div className="space-y-1">
          <Textarea
            value={jsonText}
            onChange={(e) => handleJsonChange(e.target.value)}
            className="font-mono text-sm min-h-[120px]"
            data-testid="mcp-playground-json-editor"
          />
          {jsonError && <p className="text-xs text-destructive">{jsonError}</p>}
        </div>
      )}

      <Button onClick={handleExecute} disabled={isExecuting} data-testid="mcp-playground-execute-button">
        {isExecuting && <Loader2 className="h-4 w-4 mr-2 animate-spin" />}
        Execute
      </Button>

      {result && <ResultSection result={result} activeTab={resultTab} onTabChange={setResultTab} />}
    </div>
  );
}
