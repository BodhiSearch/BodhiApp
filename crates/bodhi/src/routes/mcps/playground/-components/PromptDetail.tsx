import { useCallback, useEffect, useMemo, useState } from 'react';

import { Eye, RotateCcw } from 'lucide-react';

import { ShellIcon } from '@/components/shell';
import { Button } from '@/components/ui/button';
import type { McpClientPrompt, McpPromptGetResult } from '@/hooks/mcps/useMcpClient';

import { ArgForm } from './ArgForm';
import { type RunState, buildDefaultFieldParams, idleRun, promptArgsToFields } from './playgroundTypes';
import { ResultPanel } from './ResultPanel';

export interface PromptDetailProps {
  prompt: McpClientPrompt;
  getPrompt: (name: string, args?: Record<string, string>) => Promise<McpPromptGetResult>;
}

export function PromptDetail({ prompt, getPrompt }: PromptDetailProps) {
  const fields = useMemo(() => promptArgsToFields(prompt.arguments), [prompt.arguments]);
  const hasArgs = fields.length > 0;

  const [values, setValues] = useState<Record<string, string>>(() => buildDefaultFieldParams(fields));
  const [errors, setErrors] = useState<Record<string, true>>({});
  const [run, setRun] = useState<RunState>(idleRun());

  // Reset state when the selected prompt changes. Keyed on prompt.name to avoid loops.
  useEffect(() => {
    setValues(buildDefaultFieldParams(fields));
    setErrors({});
    setRun(idleRun());
  }, [prompt.name]);

  const handleReset = useCallback(() => {
    setValues(buildDefaultFieldParams(fields));
    setErrors({});
    setRun(idleRun());
  }, [fields]);

  const handlePreview = useCallback(async () => {
    const missing: Record<string, true> = {};
    for (const f of fields) {
      if (f.required && !(values[f.name] && values[f.name].trim())) {
        missing[f.name] = true;
      }
    }
    if (Object.keys(missing).length) {
      setErrors(missing);
      setTimeout(() => setErrors({}), 2200);
      return;
    }
    setErrors({});

    const request = { method: 'prompts/get', params: { name: prompt.name, arguments: values } };
    setRun({ phase: 'running', request });

    const result = await getPrompt(prompt.name, values);
    if (result.isError) {
      setRun({
        phase: 'done',
        kind: 'messages',
        ok: false,
        data: { description: result.description, messages: result.messages || [] },
        raw: result,
        request,
        error: result.errorMessage || 'Prompt get failed',
        token: Date.now(),
      });
      return;
    }
    setRun({
      phase: 'done',
      kind: 'messages',
      ok: true,
      data: { description: result.description, messages: result.messages },
      raw: result,
      request,
      token: Date.now(),
    });
  }, [fields, getPrompt, prompt.name, values]);

  const friendly = prompt.title || prompt.name;
  const showCodeName = friendly !== prompt.name;

  return (
    <div className="pg-detail" data-testid="mcp-playground-prompt-detail" data-test-prompt={prompt.name}>
      <div className="pg-dh">
        <div className="pg-dh-ico">
          <ShellIcon name="sparkles" size={18} />
        </div>
        <div className="pg-dh-text">
          <div className="pg-dh-name-row">
            <span className="pg-dh-name" data-testid="mcp-playground-prompt-name">
              {friendly}
            </span>
            {showCodeName && <span className="pg-dh-codename mono">({prompt.name})</span>}
            <span className="pg-dh-tag">Prompt</span>
          </div>
          {prompt.description && <div className="pg-dh-desc">{prompt.description}</div>}
        </div>
      </div>

      <div className="pg-detail-scroll">
        <div className="pg-run-card">
          {hasArgs && (
            <div className="pg-run-head">
              <span className="pg-run-title">Arguments</span>
            </div>
          )}
          <ArgForm kind="fields" fields={fields} values={values} onChange={setValues} errors={errors} />
          <div className="pg-run-row">
            <Button
              type="button"
              onClick={handlePreview}
              disabled={run.phase === 'running'}
              className="pg-run-btn"
              data-testid="mcp-playground-prompt-preview-button"
            >
              <Eye className="h-4 w-4 mr-1" />
              Preview messages
            </Button>
            {hasArgs && (
              <Button
                type="button"
                variant="ghost"
                onClick={handleReset}
                className="pg-clear-btn"
                data-testid="mcp-playground-prompt-reset-button"
              >
                <RotateCcw className="h-4 w-4 mr-1" />
                Reset
              </Button>
            )}
          </div>
        </div>
        <ResultPanel run={run} title="Messages" emptyHint="Preview to see the rendered prompt." />
      </div>
    </div>
  );
}
