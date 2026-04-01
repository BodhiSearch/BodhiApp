import { useCallback, useEffect, useMemo, useState } from 'react';

import { ArrowLeft, Check, Copy, Loader2, RefreshCw, X } from 'lucide-react';
import { Link, useSearch } from '@tanstack/react-router';
import { Prism as SyntaxHighlighter } from 'react-syntax-highlighter';
import { oneDark, oneLight } from 'react-syntax-highlighter/dist/esm/styles/prism';

import AppInitializer from '@/components/AppInitializer';
import { useTheme } from '@/components/ThemeProvider';
import { Badge } from '@/components/ui/badge';
import { Button } from '@/components/ui/button';
import { Checkbox } from '@/components/ui/checkbox';
import { ErrorPage } from '@/components/ui/ErrorPage';
import { Input } from '@/components/ui/input';
import { Label } from '@/components/ui/label';
import { Skeleton } from '@/components/ui/skeleton';
import { Textarea } from '@/components/ui/textarea';
import { toast } from '@/hooks/use-toast';
import { useGetMcp } from '@/hooks/mcps';
import { useMcpClient, type McpClientTool, type McpToolCallResult } from '@/hooks/mcps/useMcpClient';
import { cn } from '@/lib/utils';

// eslint-disable-next-line @typescript-eslint/no-explicit-any
type InputSchema = { type?: string; properties?: Record<string, any>; required?: string[] };

type ResultTab = 'response' | 'raw' | 'request';

interface ExecutionResult {
  response: McpToolCallResult;
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

// ============================================================================
// ToolSidebar
// ============================================================================

function ToolSidebar({
  tools,
  selectedTool,
  onSelectTool,
  onRefresh,
  isRefreshing,
  connectionStatus,
}: {
  tools: McpClientTool[];
  selectedTool: string | null;
  onSelectTool: (name: string) => void;
  onRefresh: () => void;
  isRefreshing: boolean;
  connectionStatus: string;
}) {
  return (
    <div className="w-64 shrink-0 border-r flex flex-col h-full" data-testid="mcp-playground-tool-sidebar">
      <div className="p-3 border-b flex items-center justify-between">
        <div className="flex items-center gap-2">
          <Button
            variant="ghost"
            size="sm"
            onClick={onRefresh}
            disabled={isRefreshing || connectionStatus !== 'connected'}
            data-testid="mcp-playground-refresh-button"
          >
            <RefreshCw className={cn('h-4 w-4', isRefreshing && 'animate-spin')} />
          </Button>
          <span className="text-xs text-muted-foreground" data-testid="mcp-playground-connection-status">
            {connectionStatus}
          </span>
        </div>
      </div>
      <div className="flex-1 overflow-y-auto" data-testid="mcp-playground-tool-list">
        {tools.map((tool) => {
          const isSelected = selectedTool === tool.name;
          return (
            <button
              key={tool.name}
              onClick={() => onSelectTool(tool.name)}
              className={cn('w-full text-left px-3 py-2 text-sm border-b transition-colors', isSelected && 'bg-accent')}
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

// ============================================================================
// ExecutionArea
// ============================================================================

function ExecutionArea({
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

// ============================================================================
// McpPlaygroundContent
// ============================================================================

function McpPlaygroundContent() {
  const search = useSearch({ strict: false });
  const id = search.id || '';
  const { data: mcp, isLoading, error } = useGetMcp(id, { enabled: !!id });
  const [selectedToolName, setSelectedToolName] = useState<string | null>(null);

  const mcpClient = useMcpClient(mcp?.mcp_endpoint ?? null);

  // Connect when mcp data is available
  useEffect(() => {
    if (mcp?.mcp_endpoint) {
      mcpClient.connect();
    }
    return () => {
      mcpClient.disconnect();
    };
  }, [mcp?.mcp_endpoint]); // eslint-disable-line react-hooks/exhaustive-deps

  const tools = mcpClient.tools;

  const selectedTool = useMemo(() => tools.find((t) => t.name === selectedToolName) || null, [tools, selectedToolName]);

  const handleRefresh = () => {
    mcpClient.refreshTools();
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
          <Link to="/mcps/">
            <ArrowLeft className="h-4 w-4 mr-1" />
            MCP Servers
          </Link>
        </Button>
        <span className="text-muted-foreground">/</span>
        <h1 className="font-semibold">{mcp.name} -- Playground</h1>
        {mcpClient.status === 'error' && mcpClient.error && (
          <span className="text-xs text-destructive ml-2">{mcpClient.error}</span>
        )}
      </div>

      <div className="flex flex-1 min-h-0">
        <ToolSidebar
          tools={tools}
          selectedTool={selectedToolName}
          onSelectTool={setSelectedToolName}
          onRefresh={handleRefresh}
          isRefreshing={mcpClient.status === 'connecting'}
          connectionStatus={mcpClient.status}
        />

        {selectedTool ? (
          <ExecutionArea key={selectedTool.name} tool={selectedTool} callTool={mcpClient.callTool} />
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
