import { useCallback, useEffect, useMemo, useState } from 'react';

import { Layers, RotateCcw } from 'lucide-react';

import { Button } from '@/components/ui/button';
import type { McpClientResourceTemplate, McpResourceReadResult } from '@/hooks/mcps/useMcpClient';

import { ArgForm } from './ArgForm';
import {
  type ArgField,
  type ResourceReadable,
  type RunState,
  buildDefaultFieldParams,
  fillTemplate,
  idleRun,
} from './playgroundTypes';
import { ResultPanel } from './ResultPanel';

export interface TemplateDetailProps {
  template: McpClientResourceTemplate;
  readResource: (uri: string) => Promise<McpResourceReadResult>;
}

/** Extract `{var}` placeholders from a URI template (RFC 6570 level-1). */
function extractTemplateVars(uriTemplate: string): string[] {
  const out: string[] = [];
  const seen = new Set<string>();
  const regex = /\{([^}]+)\}/g;
  let m: RegExpExecArray | null;
  while ((m = regex.exec(uriTemplate)) !== null) {
    const name = m[1];
    if (!seen.has(name)) {
      seen.add(name);
      out.push(name);
    }
  }
  return out;
}

export function TemplateDetail({ template, readResource }: TemplateDetailProps) {
  const fields: ArgField[] = useMemo(
    () =>
      extractTemplateVars(template.uriTemplate).map((name) => ({
        name,
        required: true,
        placeholder: '',
      })),
    [template.uriTemplate]
  );

  const [values, setValues] = useState<Record<string, string>>(() => buildDefaultFieldParams(fields));
  const [errors, setErrors] = useState<Record<string, true>>({});
  const [run, setRun] = useState<RunState>(idleRun());

  // Reset state when the selected template changes. Keyed on uriTemplate.
  useEffect(() => {
    setValues(buildDefaultFieldParams(fields));
    setErrors({});
    setRun(idleRun());
  }, [template.uriTemplate]);

  const resolvedUri = useMemo(() => fillTemplate(template.uriTemplate, values), [template.uriTemplate, values]);
  const allFilled = useMemo(
    () => fields.every((f) => values[f.name] && values[f.name].trim().length > 0),
    [fields, values]
  );

  const handleReset = useCallback(() => {
    setValues(buildDefaultFieldParams(fields));
    setErrors({});
    setRun(idleRun());
  }, [fields]);

  const handleResolveAndRead = useCallback(async () => {
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

    const uri = fillTemplate(template.uriTemplate, values);
    const request = { method: 'resources/read', params: { uri } };
    setRun({ phase: 'running', request });

    const result = await readResource(uri);
    if (result.isError) {
      const data: ResourceReadable = {
        uri,
        mimeType: template.mimeType,
        contents: result.contents || [],
      };
      setRun({
        phase: 'done',
        kind: 'resource',
        ok: false,
        data,
        raw: result,
        request,
        error: result.errorMessage || 'Template read failed',
        token: Date.now(),
      });
      return;
    }

    const data: ResourceReadable = {
      uri,
      mimeType: template.mimeType,
      contents: result.contents,
    };
    setRun({
      phase: 'done',
      kind: 'resource',
      ok: true,
      data,
      raw: result,
      request,
      token: Date.now(),
    });
  }, [fields, readResource, template.mimeType, template.uriTemplate, values]);

  const friendly = template.title || template.name;
  const showCodeName = friendly !== template.name;

  return (
    <div className="pg-detail" data-testid="mcp-playground-template-detail" data-test-template={template.uriTemplate}>
      <div className="pg-dh">
        <div className="pg-dh-ico">
          <Layers className="h-5 w-5" />
        </div>
        <div className="pg-dh-text">
          <div className="pg-dh-name-row">
            <span className="pg-dh-name" data-testid="mcp-playground-template-name">
              {friendly}
            </span>
            {showCodeName && <span className="pg-dh-codename mono">({template.name})</span>}
            <span className="pg-dh-tag">Template</span>
          </div>
          {template.description && <div className="pg-dh-desc">{template.description}</div>}
          <div className="pg-dh-meta mono">uriTemplate: {template.uriTemplate}</div>
        </div>
      </div>

      <div className="pg-detail-scroll">
        <div className="pg-run-card">
          {fields.length > 0 && (
            <div className="pg-run-head">
              <span className="pg-run-title">Fill in</span>
            </div>
          )}
          <ArgForm kind="fields" fields={fields} values={values} onChange={setValues} errors={errors} />

          <div className="pg-template-preview" data-testid="mcp-playground-template-preview">
            <span className="pg-tp-label">Resolves to</span>
            <code className="pg-tp-uri mono" data-testid="mcp-playground-template-resolved" data-filled={allFilled}>
              {resolvedUri}
            </code>
          </div>

          <div className="pg-run-row">
            <Button
              type="button"
              onClick={handleResolveAndRead}
              disabled={run.phase === 'running'}
              className="pg-run-btn"
              data-testid="mcp-playground-template-read-button"
            >
              Resolve & read
            </Button>
            {fields.length > 0 && (
              <Button
                type="button"
                variant="ghost"
                onClick={handleReset}
                className="pg-clear-btn"
                data-testid="mcp-playground-template-reset-button"
              >
                <RotateCcw className="h-4 w-4 mr-1" />
                Reset
              </Button>
            )}
          </div>
        </div>
        <ResultPanel run={run} title="Contents" emptyHint="Fill in the fields then resolve and read." />
      </div>
    </div>
  );
}
