import { type KeyboardEvent, type ReactNode, useEffect, useMemo, useRef, useState } from 'react';

import { EmptyState } from '@/components/EmptyState';
import { ShellIcon, ShellSearch } from '@/components/shell';
import type {
  McpClientPrompt,
  McpClientResource,
  McpClientResourceTemplate,
  McpClientTool,
} from '@/hooks/mcps/useMcpClient';

import { hintsForTool } from './behaviour-hints';
import type { Feature } from './OverviewView';
import { toolFriendlyTitle } from './playgroundTypes';

interface RailRow {
  id: string;
  primary: ReactNode;
  secondary?: ReactNode;
}

export interface PlaygroundRailProps {
  feature: Exclude<Feature, 'overview'>;
  selectedItem: string | null;
  onSelectItem: (item: string) => void;
  tools: McpClientTool[];
  prompts: McpClientPrompt[];
  resources: McpClientResource[];
  templates: McpClientResourceTemplate[];
}

function buildToolRows(tools: McpClientTool[]): RailRow[] {
  return tools.map((t) => {
    const friendly = toolFriendlyTitle(t);
    const showCode = friendly !== t.name;
    const hint = hintsForTool(t)[0];
    return {
      id: t.name,
      primary: (
        <span className="pg-row-name">
          <span className="pg-row-text">{friendly}</span>
          {showCode && <span className="pg-row-code mono"> ({t.name})</span>}
          {hint && <span className={'pg-row-dot tone-' + hint.tone} title={`${hint.label} — ${hint.tip}`} />}
        </span>
      ),
      secondary: t.description ? <span className="pg-row-sub">{t.description}</span> : undefined,
    };
  });
}

function buildPromptRows(prompts: McpClientPrompt[]): RailRow[] {
  return prompts.map((p) => ({
    id: p.name,
    primary: (
      <span className="pg-row-name">
        <ShellIcon name="message-square-quote" size={13} />
        <span className="pg-row-text">{p.title || p.name}</span>
      </span>
    ),
    secondary: p.description ? <span className="pg-row-sub">{p.description}</span> : undefined,
  }));
}

function buildResourceRows(resources: McpClientResource[]): RailRow[] {
  return resources.map((r) => ({
    id: r.uri,
    primary: (
      <span className="pg-row-name">
        <ShellIcon name="file-text" size={13} />
        <span className="pg-row-text">{r.title || r.name}</span>
      </span>
    ),
    secondary: <span className="pg-row-sub mono">{r.uri}</span>,
  }));
}

function buildTemplateRows(templates: McpClientResourceTemplate[]): RailRow[] {
  return templates.map((t) => ({
    id: t.uriTemplate,
    primary: (
      <span className="pg-row-name">
        <ShellIcon name="layout-template" size={13} />
        <span className="pg-row-text">{t.title || t.name}</span>
      </span>
    ),
    secondary: <span className="pg-row-sub mono">{t.uriTemplate}</span>,
  }));
}

interface FeatureCopy {
  searchPlaceholder: string;
  emptyTitle: string;
  emptySub: string;
  emptyIcon: string;
}

const COPY: Record<Exclude<Feature, 'overview'>, FeatureCopy> = {
  tools: {
    searchPlaceholder: 'Search tools…',
    emptyTitle: 'No tools',
    emptySub: 'This MCP doesn’t expose any tools.',
    emptyIcon: 'wrench',
  },
  prompts: {
    searchPlaceholder: 'Search prompts…',
    emptyTitle: 'No prompts',
    emptySub: 'This MCP doesn’t publish any ready-made prompts.',
    emptyIcon: 'message-square-quote',
  },
  resources: {
    searchPlaceholder: 'Search resources…',
    emptyTitle: 'No resources',
    emptySub: 'This MCP doesn’t expose any resources to read.',
    emptyIcon: 'folder-open',
  },
  templates: {
    searchPlaceholder: 'Search templates…',
    emptyTitle: 'No templates',
    emptySub: 'This MCP doesn’t expose any resource templates.',
    emptyIcon: 'layout-template',
  },
};

function matches(row: RailRow, q: string): boolean {
  if (!q) return true;
  const needle = q.toLowerCase();
  const haystack =
    (typeof row.id === 'string' ? row.id : '').toLowerCase() +
    ' ' +
    flattenChildren(row.primary).toLowerCase() +
    ' ' +
    flattenChildren(row.secondary).toLowerCase();
  return haystack.includes(needle);
}

function flattenChildren(node: ReactNode): string {
  if (node == null || node === false) return '';
  if (typeof node === 'string' || typeof node === 'number') return String(node);
  if (Array.isArray(node)) return node.map(flattenChildren).join(' ');
  if (typeof node === 'object' && 'props' in (node as object)) {
    // eslint-disable-next-line @typescript-eslint/no-explicit-any
    return flattenChildren((node as any).props.children);
  }
  return '';
}

export function PlaygroundRail({
  feature,
  selectedItem,
  onSelectItem,
  tools,
  prompts,
  resources,
  templates,
}: PlaygroundRailProps) {
  const rows = useMemo(() => {
    if (feature === 'tools') return buildToolRows(tools);
    if (feature === 'prompts') return buildPromptRows(prompts);
    if (feature === 'resources') return buildResourceRows(resources);
    return buildTemplateRows(templates);
  }, [feature, tools, prompts, resources, templates]);

  const [query, setQuery] = useState('');
  useEffect(() => {
    setQuery('');
  }, [feature]);

  const filtered = useMemo(() => rows.filter((r) => matches(r, query)), [rows, query]);

  const listRef = useRef<HTMLDivElement>(null);
  const copy = COPY[feature];

  const focusRow = (index: number) => {
    const node = listRef.current?.querySelector<HTMLButtonElement>(`button[data-rail-index="${index}"]`);
    node?.focus();
  };

  const handleSearchKey = (e: KeyboardEvent<HTMLInputElement>) => {
    if (e.key === 'ArrowDown') {
      e.preventDefault();
      focusRow(0);
    } else if (e.key === 'Enter' && filtered.length > 0) {
      e.preventDefault();
      onSelectItem(filtered[0].id);
    }
  };

  const handleRowKey = (e: KeyboardEvent<HTMLButtonElement>, index: number) => {
    if (e.key === 'ArrowDown') {
      e.preventDefault();
      focusRow(Math.min(index + 1, filtered.length - 1));
    } else if (e.key === 'ArrowUp') {
      e.preventDefault();
      if (index === 0) focusRow(0);
      else focusRow(index - 1);
    } else if (e.key === 'Home') {
      e.preventDefault();
      focusRow(0);
    } else if (e.key === 'End') {
      e.preventDefault();
      focusRow(filtered.length - 1);
    }
  };

  return (
    <div className="pg-rail" data-testid="mcp-playground-rail" data-test-feature={feature}>
      <div className="pg-rail-search" data-testid="mcp-playground-rail-search">
        <ShellSearch
          value={query}
          onChange={setQuery}
          placeholder={copy.searchPlaceholder}
          onKeyDown={handleSearchKey}
        />
      </div>
      <div className="pg-rail-list" ref={listRef} data-testid="mcp-playground-rail-list">
        {filtered.length === 0 ? (
          rows.length === 0 ? (
            <EmptyState
              icon={copy.emptyIcon}
              title={copy.emptyTitle}
              sub={copy.emptySub}
              testId="mcp-playground-rail-empty"
            />
          ) : (
            <div className="pg-rail-empty">No matches for “{query}”.</div>
          )
        ) : (
          filtered.map((row, i) => {
            const on = row.id === selectedItem;
            return (
              <button
                key={row.id}
                type="button"
                className={'pg-rail-row' + (on ? ' on' : '')}
                onClick={() => onSelectItem(row.id)}
                onKeyDown={(e) => handleRowKey(e, i)}
                data-rail-index={i}
                data-testid={`mcp-playground-rail-item-${row.id}`}
                data-test-active={on}
                aria-current={on ? 'true' : undefined}
              >
                {row.primary}
                {row.secondary}
              </button>
            );
          })
        )}
      </div>
    </div>
  );
}
