'use client';

import { useCallback, useEffect, useMemo, useState } from 'react';

import { AlertTriangle, ArrowLeft, Check, Copy, Loader2, RefreshCw, X } from 'lucide-react';
import Link from 'next/link';
import { useSearchParams } from 'next/navigation';
import { Prism as SyntaxHighlighter } from 'react-syntax-highlighter';
import { oneDark, oneLight } from 'react-syntax-highlighter/dist/esm/styles/prism';

import AppInitializer from '@/components/AppInitializer';
import { useTheme } from '@/components/ThemeProvider';
import { Alert, AlertDescription } from '@/components/ui/alert';
import { Badge } from '@/components/ui/badge';
import { Button } from '@/components/ui/button';
import { Checkbox } from '@/components/ui/checkbox';
import { ErrorPage } from '@/components/ui/ErrorPage';
import { Input } from '@/components/ui/input';
import { Label } from '@/components/ui/label';
import { Skeleton } from '@/components/ui/skeleton';
import { Textarea } from '@/components/ui/textarea';
import { toast } from '@/hooks/use-toast';
import { useExecuteMcpTool, useMcp, useRefreshMcpTools, type McpExecuteResponse, type McpTool } from '@/hooks/useMcps';
import { useQueryClient } from '@/hooks/useQuery';
import { cn } from '@/lib/utils';

// eslint-disable-next-line @typescript-eslint/no-explicit-any
type InputSchema = { type?: string; properties?: Record<string, any>; required?: string[] };

type ResultTab = 'response' | 'raw' | 'request';

interface ExecutionResult {
  response: McpExecuteResponse;
  toolName: string;
  params: Record<string, unknown>;
}

const getDefaultValue = (propSchema: { type?: string }): unknown => {
  switch (propSchema.type) {
    case 'boolean':
      return false;
    case 'number':
    case 'integer':
      return '';
    case 'array':
      return [];
    case 'object':
      return {};
    default:
      return '';
  }
};

const buildDefaultParams = (schema: InputSchema | undefined): Record<string, unknown> => {
  if (!schema?.properties) return {};
  const params: Record<string, unknown> = {};
  for (const [key, prop] of Object.entries(schema.properties)) {
    params[key] = getDefaultValue(prop);
  }
  return params;
};

function formatRelativeTime(dateStr: string): string {
  const date = new Date(dateStr);
  const now = new Date();
  const diffMs = now.getTime() - date.getTime();
  const diffSec = Math.floor(diffMs / 1000);
  if (diffSec < 60) return `${diffSec}s ago`;
  const diffMin = Math.floor(diffSec / 60);
  if (diffMin < 60) return `${diffMin}m ago`;
  const diffHr = Math.floor(diffMin / 60);
  if (diffHr < 24) return `${diffHr}h ago`;
  const diffDay = Math.floor(diffHr / 24);
  return `${diffDay}d ago`;
}

// ============================================================================
// ToolSidebar
// ============================================================================

function ToolSidebar({
  tools,
  toolsFilter,
  selectedTool,
  updatedAt,
  onSelectTool,
  onRefresh,
  isRefreshing,
}: {
  tools: McpTool[];
  toolsFilter: string[];
  selectedTool: string | null;
  updatedAt: string;
  onSelectTool: (name: string) => void;
  onRefresh: () => void;
  isRefreshing: boolean;
}) {
  return (
    <div className="w-64 shrink-0 border-r flex flex-col h-full" data-testid="mcp-playground-tool-sidebar">
      <div className="p-3 border-b flex items-center justify-between">
        <div className="flex items-center gap-2">
          <Button
            variant="ghost"
            size="sm"
            onClick={onRefresh}
            disabled={isRefreshing}
            data-testid="mcp-playground-refresh-button"
          >
            <RefreshCw className={cn('h-4 w-4', isRefreshing && 'animate-spin')} />
          </Button>
          <span className="text-xs text-muted-foreground" data-testid="mcp-playground-last-refreshed">
            {formatRelativeTime(updatedAt)}
          </span>
        </div>
      </div>
      <div className="flex-1 overflow-y-auto" data-testid="mcp-playground-tool-list">
        {tools.map((tool) => {
          const isWhitelisted = toolsFilter.includes(tool.name);
          const isSelected = selectedTool === tool.name;
          return (
            <button
              key={tool.name}
              onClick={() => onSelectTool(tool.name)}
              className={cn(
                'w-full text-left px-3 py-2 text-sm border-b transition-colors',
                isSelected && 'bg-accent',
                !isWhitelisted && 'opacity-50'
              )}
              data-testid={`mcp-playground-tool-${tool.name}`}
            >
              <div className="font-medium truncate">{tool.name}</div>
              {tool.description && <div className="text-xs text-muted-foreground truncate">{tool.description}</div>}
            </button>
          );
        })}
        {tools.length === 0 && <div className="p-4 text-sm text-muted-foreground text-center">No tools available</div>}
      </div>
    </div>
  );
}

// ============================================================================
// FormInput - schema-driven form field generator
// ============================================================================

function FormInput({
  schema,
  values,
  onChange,
}: {
  schema: InputSchema;
  values: Record<string, unknown>;
  onChange: (values: Record<string, unknown>) => void;
}) {
  const properties = schema.properties || {};
  const required = schema.required || [];

  const handleChange = (key: string, value: unknown) => {
    onChange({ ...values, [key]: value });
  };

  return (
    <div className="space-y-4">
      {Object.entries(properties).map(([key, prop]) => {
        const isRequired = required.includes(key);
        const propType = prop.type || 'string';

        if (propType === 'boolean') {
          return (
            <div key={key} className="flex items-center gap-2" data-testid={`mcp-playground-param-${key}`}>
              <Checkbox
                id={`param-${key}`}
                checked={!!values[key]}
                onCheckedChange={(checked) => handleChange(key, !!checked)}
              />
              <Label htmlFor={`param-${key}`}>
                {key}
                {isRequired && <span className="text-destructive ml-1">*</span>}
              </Label>
              {prop.description && <span className="text-xs text-muted-foreground">({prop.description})</span>}
            </div>
          );
        }

        if (propType === 'array' || propType === 'object') {
          const strValue =
            typeof values[key] === 'string'
              ? (values[key] as string)
              : JSON.stringify(values[key] ?? (propType === 'array' ? [] : {}), null, 2);
          return (
            <div key={key} className="space-y-1" data-testid={`mcp-playground-param-${key}`}>
              <Label htmlFor={`param-${key}`}>
                {key}
                {isRequired && <span className="text-destructive ml-1">*</span>}
                {prop.description && <span className="text-xs text-muted-foreground ml-2">({prop.description})</span>}
              </Label>
              <Textarea
                id={`param-${key}`}
                value={strValue}
                onChange={(e) => {
                  try {
                    handleChange(key, JSON.parse(e.target.value));
                  } catch {
                    handleChange(key, e.target.value);
                  }
                }}
                className="font-mono text-sm"
                rows={3}
              />
            </div>
          );
        }

        return (
          <div key={key} className="space-y-1" data-testid={`mcp-playground-param-${key}`}>
            <Label htmlFor={`param-${key}`}>
              {key}
              {isRequired && <span className="text-destructive ml-1">*</span>}
              {prop.description && <span className="text-xs text-muted-foreground ml-2">({prop.description})</span>}
            </Label>
            <Input
              id={`param-${key}`}
              type={propType === 'number' || propType === 'integer' ? 'number' : 'text'}
              value={String(values[key] ?? '')}
              onChange={(e) => {
                if (propType === 'number' || propType === 'integer') {
                  handleChange(key, e.target.value === '' ? '' : Number(e.target.value));
                } else {
                  handleChange(key, e.target.value);
                }
              }}
            />
          </div>
        );
      })}
      {Object.keys(properties).length === 0 && (
        <div className="text-sm text-muted-foreground">This tool has no parameters.</div>
      )}
    </div>
  );
}

// ============================================================================
// ResultSection - 3-tab result display
// ============================================================================

function ResultSection({
  result,
  activeTab,
  onTabChange,
}: {
  result: ExecutionResult;
  activeTab: ResultTab;
  onTabChange: (tab: ResultTab) => void;
}) {
  const { theme } = useTheme();
  const isSuccess = result.response.result !== undefined && !result.response.error;
  const [copied, setCopied] = useState(false);

  const getTabContent = useCallback((): string => {
    switch (activeTab) {
      case 'response':
        if (result.response.error) return result.response.error;
        return JSON.stringify(result.response.result, null, 2);
      case 'raw':
        return JSON.stringify(result.response, null, 2);
      case 'request':
        return JSON.stringify({ tool: result.toolName, params: result.params }, null, 2);
    }
  }, [activeTab, result]);

  const handleCopy = async () => {
    try {
      await navigator.clipboard.writeText(getTabContent());
      setCopied(true);
      setTimeout(() => setCopied(false), 2000);
    } catch {
      toast({ title: 'Failed to copy', variant: 'destructive' });
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
        {activeTab === 'response' && result.response.error ? (
          <div className="p-4 text-sm text-destructive font-mono whitespace-pre-wrap">{result.response.error}</div>
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

// ============================================================================
// ExecutionArea
// ============================================================================

function ExecutionArea({ mcpId, tool, isWhitelisted }: { mcpId: string; tool: McpTool; isWhitelisted: boolean }) {
  const schema = (tool.input_schema as InputSchema) || {};
  const [inputMode, setInputMode] = useState<'form' | 'json'>('form');
  const [params, setParams] = useState<Record<string, unknown>>(() => buildDefaultParams(schema));
  const [jsonText, setJsonText] = useState('');
  const [jsonError, setJsonError] = useState<string | null>(null);
  const [result, setResult] = useState<ExecutionResult | null>(null);
  const [resultTab, setResultTab] = useState<ResultTab>('response');

  useEffect(() => {
    setParams(buildDefaultParams(schema));
    setJsonText(JSON.stringify(buildDefaultParams(schema), null, 2));
    setJsonError(null);
    setResult(null);
    setResultTab('response');
  }, [tool.name]); // eslint-disable-line react-hooks/exhaustive-deps

  const executeMutation = useExecuteMcpTool({
    onSuccess: (response) => {
      setResult({ response, toolName: tool.name, params: currentParams() });
      setResultTab('response');
    },
    onError: (message) => {
      setResult({
        response: { error: message },
        toolName: tool.name,
        params: currentParams(),
      });
      setResultTab('response');
    },
  });

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

  const handleExecute = () => {
    const p = currentParams();
    executeMutation.mutate({
      id: mcpId,
      toolName: tool.name,
      params: cleanParams(p),
    });
  };

  return (
    <div className="flex-1 p-4 overflow-y-auto space-y-4">
      <div>
        <h2 className="text-lg font-semibold" data-testid="mcp-playground-tool-name">
          {tool.name}
        </h2>
        {tool.description && <p className="text-sm text-muted-foreground mt-1">{tool.description}</p>}
      </div>

      {!isWhitelisted && (
        <Alert data-testid="mcp-playground-not-whitelisted-warning">
          <AlertTriangle className="h-4 w-4" />
          <AlertDescription>This tool is not whitelisted. Execution may be rejected by the server.</AlertDescription>
        </Alert>
      )}

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

      <Button onClick={handleExecute} disabled={executeMutation.isLoading} data-testid="mcp-playground-execute-button">
        {executeMutation.isLoading && <Loader2 className="h-4 w-4 mr-2 animate-spin" />}
        Execute
      </Button>

      {result && <ResultSection result={result} activeTab={resultTab} onTabChange={setResultTab} />}
    </div>
  );
}

// ============================================================================
// McpPlaygroundContent
// ============================================================================

function McpPlaygroundContent() {
  const searchParams = useSearchParams();
  const id = searchParams.get('id') || '';
  const queryClient = useQueryClient();
  const { data: mcp, isLoading, error } = useMcp(id, { enabled: !!id });
  const [selectedToolName, setSelectedToolName] = useState<string | null>(null);

  const refreshMutation = useRefreshMcpTools({
    onSuccess: () => {
      queryClient.invalidateQueries(['mcps', id]);
      toast({ title: 'Tools refreshed' });
    },
    onError: (message) => {
      toast({ title: 'Failed to refresh tools', description: message, variant: 'destructive' });
    },
  });

  const tools = useMemo(() => mcp?.tools_cache || [], [mcp?.tools_cache]);
  const toolsFilter = useMemo(() => mcp?.tools_filter || [], [mcp?.tools_filter]);

  const selectedTool = useMemo(() => tools.find((t) => t.name === selectedToolName) || null, [tools, selectedToolName]);

  const isSelectedToolWhitelisted = useMemo(
    () => (selectedToolName ? toolsFilter.includes(selectedToolName) : true),
    [toolsFilter, selectedToolName]
  );

  const handleRefresh = () => {
    refreshMutation.mutate({ id });
  };

  if (!id) {
    return <ErrorPage message="No MCP ID provided" />;
  }

  if (error) {
    const errorMessage = error.response?.data?.error?.message || error.message || 'Failed to load MCP';
    return <ErrorPage message={errorMessage} />;
  }

  if (isLoading || !mcp) {
    return (
      <div className="container mx-auto p-4" data-testid="mcp-playground-loading">
        <div className="space-y-4">
          <Skeleton className="h-10 w-full" />
          <Skeleton className="h-64 w-full" />
        </div>
      </div>
    );
  }

  return (
    <div className="flex flex-col h-[calc(100vh-4rem)]" data-testid="mcp-playground-page">
      <div className="border-b px-4 py-3 flex items-center gap-3">
        <Button variant="ghost" size="sm" asChild data-testid="mcp-playground-back-button">
          <Link href="/ui/mcps/">
            <ArrowLeft className="h-4 w-4 mr-1" />
            MCP Servers
          </Link>
        </Button>
        <span className="text-muted-foreground">/</span>
        <h1 className="font-semibold">{mcp.name} — Playground</h1>
      </div>

      <div className="flex flex-1 min-h-0">
        <ToolSidebar
          tools={tools}
          toolsFilter={toolsFilter}
          selectedTool={selectedToolName}
          updatedAt={mcp.updated_at}
          onSelectTool={setSelectedToolName}
          onRefresh={handleRefresh}
          isRefreshing={refreshMutation.isLoading}
        />

        {selectedTool ? (
          <ExecutionArea
            key={selectedTool.name}
            mcpId={id}
            tool={selectedTool}
            isWhitelisted={isSelectedToolWhitelisted}
          />
        ) : (
          <div className="flex-1 flex items-center justify-center text-muted-foreground">
            Select a tool from the sidebar to get started
          </div>
        )}
      </div>
    </div>
  );
}

export default function McpPlaygroundPage() {
  return (
    <AppInitializer authenticated={true} allowedStatus="ready">
      <McpPlaygroundContent />
    </AppInitializer>
  );
}
