import { type ReactNode, useEffect, useState } from 'react';

import { Check, Copy, Loader2 } from 'lucide-react';

import { ShellIcon } from '@/components/shell';
import { Button } from '@/components/ui/button';
import { toast } from '@/hooks/use-toast';
import { copyToClipboard, getClipboardUnavailableMessage } from '@/lib/clipboard';

import type { RunState } from './playgroundTypes';
import { MessagesView, ResourceContentsView, ToolResultView } from './ReadableResult';

export type ResultTab = 'readable' | 'raw' | 'request';

export interface ResultPanelProps {
  run: RunState;
  /** Friendly label for the readable tab (e.g. "Result", "Messages", "Contents"). */
  title?: string;
  /** Hint shown when the run is idle. */
  emptyHint?: string;
  /** Resource-link click handler so `<ToolResultView>` can deep-link to Resources. */
  onOpenResource?: (uri: string, name?: string) => void;
}

function stringify(value: unknown): string {
  if (value == null) return '';
  if (typeof value === 'string') return value;
  try {
    return JSON.stringify(value, null, 2);
  } catch {
    return String(value);
  }
}

export function ResultPanel({ run, title = 'Result', emptyHint, onOpenResource }: ResultPanelProps) {
  const [tab, setTab] = useState<ResultTab>('readable');

  // Reset to readable on each fresh run.
  const token = run.phase === 'done' ? run.token : -1;
  useEffect(() => {
    setTab('readable');
  }, [token]);

  if (run.phase === 'idle') {
    return (
      <div className="pg-result" data-testid="mcp-playground-result" data-test-state="idle">
        <div className="pg-result-idle">
          <ShellIcon name="sparkles" size={26} />
          <div className="pg-ri-t">Nothing run yet</div>
          <div className="pg-ri-s">{emptyHint || 'Fill in anything needed, then run it to see the result here.'}</div>
        </div>
      </div>
    );
  }

  if (run.phase === 'running') {
    return (
      <div className="pg-result" data-testid="mcp-playground-result" data-test-state="running">
        <div className="pg-result-running">
          <Loader2 className="h-6 w-6 animate-spin" />
          <div>Working…</div>
        </div>
      </div>
    );
  }

  const ok = run.ok;
  const status: 'success' | 'error' = ok ? 'success' : 'error';

  const requestText = stringify(run.request);
  const rawText = stringify(run.raw);
  // Readable copy text — flatten to text where possible
  let readableCopy: string;
  if (run.kind === 'tool') {
    readableCopy = stringify({ content: run.data.content, structuredContent: run.data.structuredContent });
  } else if (run.kind === 'messages') {
    readableCopy = stringify(run.data.messages);
  } else {
    readableCopy = stringify(run.data);
  }

  const copyTarget = tab === 'request' ? requestText : tab === 'raw' ? rawText : readableCopy;

  let body: ReactNode = null;
  if (!ok) {
    body = (
      <div className="pg-error" data-testid="mcp-playground-result-error">
        <ShellIcon name="circle-alert" size={15} />
        <span>{run.error || 'Something went wrong.'}</span>
      </div>
    );
  } else if (tab === 'readable') {
    if (run.kind === 'tool') body = <ToolResultView model={run.data} onOpenResource={onOpenResource} />;
    else if (run.kind === 'messages') body = <MessagesView messages={run.data.messages} />;
    else body = <ResourceContentsView data={run.data} />;
  } else if (tab === 'raw') {
    body = (
      <pre className="pg-code" data-testid="mcp-playground-result-raw">
        {rawText}
      </pre>
    );
  } else {
    body = (
      <pre className="pg-code" data-testid="mcp-playground-result-request">
        {requestText}
      </pre>
    );
  }

  return (
    <div className="pg-result" data-testid="mcp-playground-result" data-test-state={status}>
      <div className="pg-result-head">
        <span
          className={'pg-status ' + (ok ? 'ok' : 'err')}
          data-testid="mcp-playground-result-status"
          data-test-state={status}
        >
          <ShellIcon name={ok ? 'circle-check' : 'circle-alert'} size={12} />
          {ok ? 'Success' : 'Error'}
        </span>
        <div className="pg-result-tabs">
          <TabButton current={tab} value="readable" label={title} onClick={() => setTab('readable')} />
          <TabButton current={tab} value="raw" label="Raw" onClick={() => setTab('raw')} />
          <TabButton current={tab} value="request" label="Request" onClick={() => setTab('request')} />
        </div>
        <CopyBtn text={copyTarget} />
      </div>
      <div className="pg-result-body">{body}</div>
    </div>
  );
}

function TabButton({
  current,
  value,
  label,
  onClick,
}: {
  current: ResultTab;
  value: ResultTab;
  label: string;
  onClick: () => void;
}) {
  const on = current === value;
  return (
    <button
      type="button"
      className={'pg-rtab' + (on ? ' on' : '')}
      onClick={onClick}
      data-testid={`mcp-playground-result-tab-${value}`}
      data-test-state={on ? 'active' : 'inactive'}
    >
      {label}
    </button>
  );
}

function CopyBtn({ text }: { text: string }) {
  const [done, setDone] = useState(false);
  const handleCopy = async () => {
    try {
      await copyToClipboard(text);
      setDone(true);
      setTimeout(() => setDone(false), 1400);
    } catch {
      toast({
        title: 'Failed to copy',
        description: getClipboardUnavailableMessage() ?? undefined,
        variant: 'destructive',
      });
    }
  };
  return (
    <Button variant="ghost" size="sm" onClick={handleCopy} data-testid="mcp-playground-copy-button">
      {done ? <Check className="h-4 w-4" /> : <Copy className="h-4 w-4" />}
    </Button>
  );
}
