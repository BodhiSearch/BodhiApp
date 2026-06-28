import { Loader2, RefreshCw } from 'lucide-react';

import { ShellIcon } from '@/components/shell';
import { Button } from '@/components/ui/button';
import type { McpConnectionStatus } from '@/hooks/mcps/useMcpClient';

export interface ConnectionStatusProps {
  status: McpConnectionStatus;
  onRefresh: () => void;
  /** Disable refresh while we're already busy. */
  refreshing: boolean;
}

const LABEL: Record<McpConnectionStatus, string> = {
  disconnected: 'Disconnected',
  connecting: 'Connecting',
  connected: 'Connected',
  refreshing: 'Refreshing',
  error: 'Error',
};

export function ConnectionStatus({ status, onRefresh, refreshing }: ConnectionStatusProps) {
  const tone =
    status === 'connected' ? 'ok' : status === 'error' ? 'err' : status === 'disconnected' ? 'muted' : 'warn';
  const showSpinner = status === 'connecting' || status === 'refreshing';

  return (
    <div className="pg-connstatus">
      <span className={`pg-pill ${tone}`} data-testid="mcp-playground-connection-status" data-test-state={status}>
        {showSpinner ? (
          <Loader2 className="h-3 w-3 animate-spin" />
        ) : (
          <ShellIcon
            name={status === 'connected' ? 'circle-check' : status === 'error' ? 'circle-alert' : 'circle-dashed'}
            size={11}
          />
        )}
        {LABEL[status]}
      </span>
      <Button
        variant="ghost"
        size="sm"
        onClick={onRefresh}
        disabled={refreshing || status !== 'connected'}
        data-testid="mcp-playground-refresh-button"
        title="Refresh capabilities"
      >
        <RefreshCw className={'h-4 w-4' + (refreshing ? ' animate-spin' : '')} />
      </Button>
    </div>
  );
}
