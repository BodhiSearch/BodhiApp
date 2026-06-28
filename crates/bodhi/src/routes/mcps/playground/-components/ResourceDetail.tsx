import { useCallback, useEffect, useState } from 'react';

import { FileText, RefreshCw } from 'lucide-react';

import { Button } from '@/components/ui/button';
import type { McpClientResource, McpResourceReadResult } from '@/hooks/mcps/useMcpClient';

import { type ResourceReadable, type RunState, idleRun } from './playgroundTypes';
import { ResultPanel } from './ResultPanel';

export interface ResourceDetailProps {
  resource: McpClientResource;
  readResource: (uri: string) => Promise<McpResourceReadResult>;
}

export function ResourceDetail({ resource, readResource }: ResourceDetailProps) {
  const [run, setRun] = useState<RunState>(idleRun());

  // Reset state when the selected resource changes. Keyed on resource.uri.
  useEffect(() => {
    setRun(idleRun());
  }, [resource.uri]);

  const handleRead = useCallback(async () => {
    const request = { method: 'resources/read', params: { uri: resource.uri } };
    setRun({ phase: 'running', request });

    const result = await readResource(resource.uri);
    if (result.isError) {
      const data: ResourceReadable = {
        uri: resource.uri,
        mimeType: resource.mimeType,
        contents: result.contents || [],
      };
      setRun({
        phase: 'done',
        kind: 'resource',
        ok: false,
        data,
        raw: result,
        request,
        error: result.errorMessage || 'Resource read failed',
        token: Date.now(),
      });
      return;
    }

    const data: ResourceReadable = {
      uri: resource.uri,
      mimeType: resource.mimeType,
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
  }, [readResource, resource.uri, resource.mimeType]);

  const friendly = resource.title || resource.name;
  const showCodeName = friendly !== resource.uri;

  return (
    <div className="pg-detail" data-testid="mcp-playground-resource-detail" data-test-resource={resource.uri}>
      <div className="pg-dh">
        <div className="pg-dh-ico">
          <FileText className="h-5 w-5" />
        </div>
        <div className="pg-dh-text">
          <div className="pg-dh-name-row">
            <span className="pg-dh-name" data-testid="mcp-playground-resource-name">
              {friendly}
            </span>
            {showCodeName && <span className="pg-dh-codename mono">{resource.uri}</span>}
            <span className="pg-dh-tag">Resource</span>
          </div>
          {resource.description && <div className="pg-dh-desc">{resource.description}</div>}
          {resource.mimeType && <div className="pg-dh-meta mono">mimeType: {resource.mimeType}</div>}
        </div>
      </div>

      <div className="pg-detail-scroll">
        <div className="pg-run-card">
          <div className="pg-run-row">
            <Button
              type="button"
              onClick={handleRead}
              disabled={run.phase === 'running'}
              className="pg-run-btn"
              data-testid="mcp-playground-resource-read-button"
            >
              <RefreshCw className="h-4 w-4 mr-1" />
              {run.phase === 'idle' ? 'Read resource' : 'Re-read'}
            </Button>
          </div>
        </div>
        <ResultPanel run={run} title="Contents" emptyHint="Read the resource to see its contents." />
      </div>
    </div>
  );
}
