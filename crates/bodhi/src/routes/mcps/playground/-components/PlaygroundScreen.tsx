import { useCallback, useEffect, useMemo } from 'react';

import { getRouteApi } from '@tanstack/react-router';

import { EmptyState } from '@/components/EmptyState';
import { useShellChrome } from '@/components/shell';
import { ErrorPage } from '@/components/ui/ErrorPage';
import { Skeleton } from '@/components/ui/skeleton';
import { useGetMcp, useListMcps } from '@/hooks/mcps';
import { useMcpClient } from '@/hooks/mcps/useMcpClient';
import { useViewTransition } from '@/hooks/useViewTransition';
import { ROUTE_MCPS } from '@/lib/constants';
import { extractErrorMessage } from '@/lib/errorUtils';

import { CapabilityNav } from './CapabilityNav';
import { ConnectionStatus } from './ConnectionStatus';
import { InstancePicker } from './InstancePicker';
import { OverviewView, type Feature } from './OverviewView';
import { PlaygroundRail } from './PlaygroundRail';
import { PromptDetail } from './PromptDetail';
import { ToolDetail } from './ToolDetail';

import './playground.css';

const routeApi = getRouteApi('/mcps/playground/');

export interface PlaygroundSearch {
  id?: string;
  feature?: Feature;
  item?: string;
}

export function PlaygroundScreen() {
  const search = routeApi.useSearch();
  const navigate = routeApi.useNavigate();

  const id = search.id ?? '';
  const feature: Feature = search.feature ?? 'overview';
  const item = search.item ?? null;

  const { data: instancesData } = useListMcps();
  const instances = instancesData?.mcps ?? [];

  const { data: mcp, isLoading: mcpLoading, error: mcpError } = useGetMcp(id, { enabled: !!id });
  const mcpClient = useMcpClient(mcp?.path ?? null);

  // Connect/disconnect keyed on path only; the mcpClient identity changes each render and
  // would reconnect in a loop if included in deps.
  useEffect(() => {
    if (mcp?.path) {
      mcpClient.connect();
    }
    return () => {
      mcpClient.disconnect();
    };
  }, [mcp?.path]);

  const withViewTransition = useViewTransition();

  const selectInstance = useCallback(
    (nextId: string) => {
      if (nextId === id) return;
      navigate({
        search: () => ({ id: nextId }),
        replace: false,
      });
    },
    [navigate, id]
  );

  const selectFeature = useCallback(
    (nextFeature: Feature) => {
      if (nextFeature === feature && !item) return;
      withViewTransition(() => {
        navigate({
          search: (prev: PlaygroundSearch) => {
            const out: PlaygroundSearch = { ...prev };
            if (nextFeature === 'overview') delete out.feature;
            else out.feature = nextFeature;
            delete out.item;
            return out;
          },
          replace: true,
        });
      });
    },
    [navigate, withViewTransition, feature, item]
  );

  const selectItem = useCallback(
    (nextItem: string | null) => {
      if ((nextItem ?? undefined) === (item ?? undefined)) return;
      withViewTransition(() => {
        navigate({
          search: (prev: PlaygroundSearch) => {
            const out: PlaygroundSearch = { ...prev };
            if (nextItem) out.item = nextItem;
            else delete out.item;
            return out;
          },
          replace: true,
        });
      });
    },
    [navigate, withViewTransition, item]
  );

  const openResource = useCallback(
    (uri: string) => {
      withViewTransition(() => {
        navigate({
          search: (prev: PlaygroundSearch) => ({ ...prev, feature: 'resources', item: uri }),
          replace: true,
        });
      });
    },
    [navigate, withViewTransition]
  );

  const sidebar = useMemo(
    () => (
      <div className="pg-sidebar" data-testid="mcp-playground-sidebar">
        <InstancePicker instances={instances} selectedId={id} onSelect={selectInstance} />
        <CapabilityNav active={feature} counts={mcpClient.counts} onSelect={selectFeature} />
      </div>
    ),
    [instances, id, feature, mcpClient.counts, selectFeature, selectInstance]
  );

  const showRail = feature !== 'overview' && !!mcp;
  const rail = useMemo(() => {
    if (!showRail) return null;
    return (
      <PlaygroundRail
        feature={feature as Exclude<Feature, 'overview'>}
        selectedItem={item}
        onSelectItem={(next) => selectItem(next)}
        tools={mcpClient.tools}
        prompts={mcpClient.prompts}
        resources={mcpClient.resources}
        templates={mcpClient.resourceTemplates}
      />
    );
  }, [
    showRail,
    feature,
    item,
    selectItem,
    mcpClient.tools,
    mcpClient.prompts,
    mcpClient.resources,
    mcpClient.resourceTemplates,
  ]);

  const headerActions = useMemo(
    () => (
      <ConnectionStatus
        status={mcpClient.status}
        onRefresh={mcpClient.refresh}
        refreshing={mcpClient.status === 'refreshing' || mcpClient.status === 'connecting'}
      />
    ),
    [mcpClient.status, mcpClient.refresh]
  );

  const breadcrumb = useMemo(
    () => [
      { label: 'Bodhi' },
      { label: 'MCP', href: ROUTE_MCPS },
      { label: mcp ? `${mcp.name} · Playground` : 'Playground', current: true as const },
    ],
    [mcp]
  );

  useShellChrome({
    breadcrumb,
    sidebar,
    rail,
    headerActions,
    railDefaultOpen: showRail,
  });

  if (!id) {
    return <ErrorPage message="No MCP ID provided" />;
  }

  if (mcpError) {
    return <ErrorPage message={extractErrorMessage(mcpError, 'Failed to load MCP')} />;
  }

  if (mcpLoading || !mcp) {
    return (
      <div className="pg-loading" data-testid="mcp-playground-loading">
        <Skeleton className="h-12 w-1/2 mb-3" />
        <Skeleton className="h-32 w-full mb-3" />
        <Skeleton className="h-32 w-full" />
      </div>
    );
  }

  return (
    <div className="pg-center" data-testid="mcp-playground-page" data-test-feature={feature}>
      {mcpClient.status === 'error' && mcpClient.error && (
        <div className="pg-conn-error" data-testid="mcp-playground-conn-error">
          <strong>Connection error:</strong> {mcpClient.error}
        </div>
      )}

      {feature === 'overview' && (
        <OverviewView mcp={mcp} status={mcpClient.status} counts={mcpClient.counts} onSelectFeature={selectFeature} />
      )}

      {feature === 'tools' && (
        <ToolsCenter tools={mcpClient.tools} item={item} callTool={mcpClient.callTool} onOpenResource={openResource} />
      )}

      {feature === 'prompts' && (
        <PromptsCenter prompts={mcpClient.prompts} item={item} getPrompt={mcpClient.getPrompt} />
      )}
      {feature === 'resources' && <ComingSoon feature="resources" />}
      {feature === 'templates' && <ComingSoon feature="templates" />}
    </div>
  );
}

function ToolsCenter({
  tools,
  item,
  callTool,
  onOpenResource,
}: {
  tools: ReturnType<typeof useMcpClient>['tools'];
  item: string | null;
  callTool: ReturnType<typeof useMcpClient>['callTool'];
  onOpenResource: (uri: string, name?: string) => void;
}) {
  const selected = useMemo(() => (item ? (tools.find((t) => t.name === item) ?? null) : null), [tools, item]);

  if (tools.length === 0) {
    return (
      <EmptyState
        icon="wrench"
        title="No tools"
        sub="This MCP doesn’t expose any tools."
        testId="mcp-playground-tools-empty"
      />
    );
  }

  if (!selected) {
    return (
      <div className="pg-pick" data-testid="mcp-playground-pick">
        <div className="pg-pick-ico">
          <svg width="24" height="24" viewBox="0 0 24 24" fill="none">
            <path
              d="m9 11-6 6v3h3l6-6"
              stroke="currentColor"
              strokeWidth="1.5"
              strokeLinecap="round"
              strokeLinejoin="round"
            />
            <path
              d="m22 12-3-3-6 6 3 3 6-6Z"
              stroke="currentColor"
              strokeWidth="1.5"
              strokeLinecap="round"
              strokeLinejoin="round"
            />
          </svg>
        </div>
        <div className="pg-pick-text">Pick a tool on the right to begin.</div>
      </div>
    );
  }

  return <ToolDetail key={selected.name} tool={selected} callTool={callTool} onOpenResource={onOpenResource} />;
}

function PromptsCenter({
  prompts,
  item,
  getPrompt,
}: {
  prompts: ReturnType<typeof useMcpClient>['prompts'];
  item: string | null;
  getPrompt: ReturnType<typeof useMcpClient>['getPrompt'];
}) {
  const selected = useMemo(() => (item ? (prompts.find((p) => p.name === item) ?? null) : null), [prompts, item]);

  if (prompts.length === 0) {
    return (
      <EmptyState
        icon="sparkles"
        title="No prompts"
        sub="This MCP doesn’t expose any prompts."
        testId="mcp-playground-prompts-empty"
      />
    );
  }

  if (!selected) {
    return (
      <div className="pg-pick" data-testid="mcp-playground-pick">
        <div className="pg-pick-text">Pick a prompt on the right to begin.</div>
      </div>
    );
  }

  return <PromptDetail key={selected.name} prompt={selected} getPrompt={getPrompt} />;
}

function ComingSoon({ feature }: { feature: Feature }) {
  return (
    <EmptyState
      icon="hourglass"
      title={`${feature.charAt(0).toUpperCase() + feature.slice(1)} are coming soon`}
      sub="The Screen-V2 playground enables this capability in a follow-up phase."
      testId={`mcp-playground-coming-${feature}`}
    />
  );
}
