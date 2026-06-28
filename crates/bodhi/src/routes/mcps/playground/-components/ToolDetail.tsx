import { useCallback, useEffect, useMemo, useState } from 'react';

import { Play, RotateCcw } from 'lucide-react';

import { ShellIcon } from '@/components/shell';
import { Button } from '@/components/ui/button';
import type { McpClientTool, McpToolCallResult } from '@/hooks/mcps/useMcpClient';

import { ArgForm } from './ArgForm';
import { BehaviourHints } from './BehaviourHints';
import {
  type InputSchema,
  type RunState,
  buildDefaultSchemaParams,
  cleanParams,
  idleRun,
  toToolReadable,
  toolFriendlyTitle,
} from './playgroundTypes';
import { ResultPanel } from './ResultPanel';

export interface ToolDetailProps {
  tool: McpClientTool;
  callTool: (name: string, args: Record<string, unknown>) => Promise<McpToolCallResult>;
  onOpenResource?: (uri: string, name?: string) => void;
}

export function ToolDetail({ tool, callTool, onOpenResource }: ToolDetailProps) {
  const schema = (tool.inputSchema as InputSchema) || {};
  const hasInputs = useMemo(() => Object.keys(schema.properties || {}).length > 0, [schema]);

  const [values, setValues] = useState<Record<string, unknown>>(() => buildDefaultSchemaParams(schema));
  const [errors, setErrors] = useState<Record<string, true>>({});
  const [run, setRun] = useState<RunState>(idleRun());

  // Reset state when the tool changes. Keyed on tool.name; schema is derived from the tool
  // and intentionally not listed as a dep to avoid loops.
  useEffect(() => {
    setValues(buildDefaultSchemaParams(schema));
    setErrors({});
    setRun(idleRun());
  }, [tool.name]);

  const handleReset = useCallback(() => {
    setValues(buildDefaultSchemaParams(schema));
    setErrors({});
    setRun(idleRun());
  }, [schema]);

  const handleRun = useCallback(async () => {
    const cleaned = cleanParams(values);
    const errs: Record<string, true> = {};
    for (const req of schema.required || []) {
      const v = cleaned[req];
      if (v === undefined || (typeof v === 'string' && v.trim() === '')) {
        errs[req] = true;
      }
    }
    if (Object.keys(errs).length) {
      setErrors(errs);
      setTimeout(() => setErrors({}), 2200);
      return;
    }
    setErrors({});

    const request = { method: 'tools/call', params: { name: tool.name, arguments: cleaned } };
    setRun({ phase: 'running', request });

    const response = await callTool(tool.name, cleaned);
    const readable = toToolReadable(response.content, response.structuredContent, response.isError);
    const raw = {
      content: response.content,
      ...(response.structuredContent !== undefined ? { structuredContent: response.structuredContent } : {}),
      ...(response.isError ? { isError: true } : {}),
    };
    setRun({
      phase: 'done',
      kind: 'tool',
      ok: !response.isError,
      data: readable,
      raw,
      request,
      error: response.isError ? extractErrorText(response.content) : undefined,
      token: Date.now(),
    });
  }, [tool.name, values, schema.required, callTool]);

  const friendly = toolFriendlyTitle(tool);
  const showCodeName = friendly !== tool.name;

  return (
    <div className="pg-detail" data-testid="mcp-playground-tool-detail" data-test-tool={tool.name}>
      <div className="pg-dh">
        <div className="pg-dh-ico">
          <ShellIcon name="wrench" size={18} />
        </div>
        <div className="pg-dh-text">
          <div className="pg-dh-name-row">
            <span className="pg-dh-name" data-testid="mcp-playground-tool-name">
              {friendly}
            </span>
            {showCodeName && <span className="pg-dh-codename mono">({tool.name})</span>}
            <span className="pg-dh-tag">Tool</span>
          </div>
          {tool.description && <div className="pg-dh-desc">{tool.description}</div>}
          <BehaviourHints tool={tool} />
        </div>
      </div>

      <div className="pg-detail-scroll">
        <div className="pg-run-card">
          {hasInputs && (
            <div className="pg-run-head">
              <span className="pg-run-title">Inputs</span>
            </div>
          )}
          <ArgForm kind="schema" schema={schema} values={values} onChange={setValues} errors={errors} />
          <div className="pg-run-row">
            <Button
              type="button"
              onClick={handleRun}
              disabled={run.phase === 'running'}
              className="pg-run-btn"
              data-testid="mcp-playground-run-button"
            >
              <Play className="h-4 w-4 mr-1" />
              Run tool
            </Button>
            {hasInputs && (
              <Button
                type="button"
                variant="ghost"
                onClick={handleReset}
                className="pg-clear-btn"
                data-testid="mcp-playground-reset-button"
              >
                <RotateCcw className="h-4 w-4 mr-1" />
                Reset
              </Button>
            )}
          </div>
        </div>
        <ResultPanel
          run={run}
          title="Result"
          emptyHint="Run this tool to see what comes back"
          onOpenResource={onOpenResource}
        />
      </div>
    </div>
  );
}

function extractErrorText(content: unknown): string {
  if (typeof content === 'string') return content;
  if (Array.isArray(content)) {
    for (const c of content) {
      if (c && typeof c === 'object' && 'text' in c && typeof (c as { text?: unknown }).text === 'string') {
        return (c as { text: string }).text;
      }
    }
  }
  if (content && typeof content === 'object' && 'text' in content) {
    const t = (content as { text?: unknown }).text;
    if (typeof t === 'string') return t;
  }
  return 'Tool returned an error.';
}
