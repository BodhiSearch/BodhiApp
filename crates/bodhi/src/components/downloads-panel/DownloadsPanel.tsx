import { useCallback } from 'react';

import type { DownloadRequest } from '@bodhiapp/ts-client';

import { ShellIcon } from '@/components/shell';
import { Skeleton } from '@/components/ui/skeleton';

import './downloads-panel.css';

/** Derived UI section. The backend has only pending/completed/error, so DOWNLOADING vs
 *  QUEUED is split on `started_at` (set on the first progress sync). */
export type DownloadSection = 'downloading' | 'queued' | 'failed' | 'completed';

export function sectionOf(d: DownloadRequest): DownloadSection {
  if (d.status === 'completed') return 'completed';
  if (d.status === 'error') return 'failed';
  return d.started_at ? 'downloading' : 'queued';
}

/** Active = in-flight or waiting; drives the Downloads badge count and polling. */
export function isActive(d: DownloadRequest): boolean {
  const s = sectionOf(d);
  return s === 'downloading' || s === 'queued';
}

function fmtBytes(bytes?: number | null): string {
  if (bytes == null) return '—';
  const gb = bytes / 1_000_000_000;
  return gb >= 1 ? `${gb.toFixed(2)} GB` : `${(bytes / 1_000_000).toFixed(1)} MB`;
}

function repoName(d: DownloadRequest): { org: string; name: string } {
  const slash = d.repo.indexOf('/');
  return slash === -1 ? { org: '', name: d.repo } : { org: d.repo.slice(0, slash), name: d.repo.slice(slash + 1) };
}

const MONTHS = ['Jan', 'Feb', 'Mar', 'Apr', 'May', 'Jun', 'Jul', 'Aug', 'Sep', 'Oct', 'Nov', 'Dec'];

/** "Today, 16:49" / "Yesterday" / "12 Mar" — matches the design's completed-section style. */
function fmtCompleted(iso: string): string {
  const d = new Date(iso);
  const now = new Date();
  const sameDay = d.toDateString() === now.toDateString();
  const yesterday = new Date(now);
  yesterday.setDate(now.getDate() - 1);
  if (sameDay) {
    const hh = String(d.getHours()).padStart(2, '0');
    const mm = String(d.getMinutes()).padStart(2, '0');
    return `Today, ${hh}:${mm}`;
  }
  if (d.toDateString() === yesterday.toDateString()) return 'Yesterday';
  return `${d.getDate()} ${MONTHS[d.getMonth()]}`;
}

interface ItemProps {
  d: DownloadRequest;
  onArchive: (id: string) => void;
  onRetry: (id: string) => void;
  busy: boolean;
}

function DownloadItem({ d, onArchive, onRetry, busy }: ItemProps) {
  const section = sectionOf(d);
  const { org, name } = repoName(d);
  const pct =
    d.total_bytes && d.total_bytes > 0 ? Math.min(100, Math.round((d.downloaded_bytes / d.total_bytes) * 100)) : 0;

  return (
    <div className={`ld-dl-item ld-dl-${section}`} data-testid={`ld-dl-item-${d.id}`} data-test-state={section}>
      <div className="ld-dl-item-head">
        <div className="ld-dl-repo">
          {org && <span className="ld-dl-org">{org}/</span>}
          <span className="ld-dl-name">{name}</span>
        </div>
        {section === 'failed' && (
          <button
            className="ld-dl-act"
            title="Retry download"
            disabled={busy}
            onClick={() => onRetry(d.id)}
            data-testid={`ld-dl-retry-${d.id}`}
          >
            <ShellIcon name="rotate-cw" size={13} />
          </button>
        )}
        {section !== 'downloading' && (
          <button
            className="ld-dl-act"
            title="Dismiss"
            disabled={busy}
            onClick={() => onArchive(d.id)}
            data-testid={`ld-dl-archive-${d.id}`}
          >
            <ShellIcon name="x" size={13} />
          </button>
        )}
      </div>
      <div className="ld-dl-file mono">{d.filename}</div>

      {section === 'downloading' && (
        <>
          <div className="ld-dl-progress">
            <div className="ld-dl-progress-bar" style={{ width: `${pct}%` }} />
          </div>
          <div className="ld-dl-meta">
            <span>{pct}%</span>
            <span>
              {fmtBytes(d.downloaded_bytes)} / {fmtBytes(d.total_bytes)}
            </span>
          </div>
        </>
      )}
      {section === 'queued' && (
        <div className="ld-dl-meta">
          <span>Waiting…</span>
          <span>{fmtBytes(d.total_bytes)}</span>
        </div>
      )}
      {section === 'failed' && (
        <div className="ld-dl-meta ld-dl-error">
          <span>{d.error ?? 'Download failed'}</span>
          <span>{fmtBytes(d.total_bytes)}</span>
        </div>
      )}
      {section === 'completed' && (
        <div className="ld-dl-meta">
          <span className="ld-dl-done">
            <ShellIcon name="check" size={11} /> Completed · {fmtCompleted(d.updated_at)}
          </span>
          <span>{fmtBytes(d.total_bytes)}</span>
        </div>
      )}
    </div>
  );
}

const SECTION_LABELS: Record<DownloadSection, string> = {
  downloading: 'Downloading',
  queued: 'Queued',
  failed: 'Failed',
  completed: 'Completed',
};
const SECTION_ORDER: DownloadSection[] = ['downloading', 'queued', 'failed', 'completed'];

interface PanelProps {
  items: DownloadRequest[];
  loading: boolean;
  onArchive: (id: string) => void;
  onRetry: (id: string) => void;
  busy: boolean;
  /** Move focus to the main list (down-arrow handoff). Up-arrow intentionally does nothing. */
  onJumpToList: () => void;
}

export function DownloadsPanel({ items, loading, onArchive, onRetry, busy, onJumpToList }: PanelProps) {
  const onKeyDown = useCallback(
    (e: React.KeyboardEvent<HTMLDivElement>) => {
      if (e.key === 'ArrowDown') {
        e.preventDefault();
        onJumpToList();
      }
    },
    [onJumpToList]
  );

  if (loading && items.length === 0) {
    return (
      <div className="ld-dl-panel" data-testid="ld-downloads-panel" data-pagestatus="loading">
        {Array.from({ length: 4 }).map((_, i) => (
          <Skeleton key={i} className="h-16 w-full mb-3" data-testid="ld-dl-skeleton" />
        ))}
      </div>
    );
  }

  if (items.length === 0) {
    return (
      <div className="ld-dl-panel ld-dl-empty" data-testid="ld-downloads-panel" data-pagestatus="ready">
        <div className="empty-icon">
          <ShellIcon name="download" size={26} />
        </div>
        <div className="empty-title">No downloads yet</div>
        <div className="empty-sub">Pull a model from the catalog to see it here.</div>
      </div>
    );
  }

  const grouped = SECTION_ORDER.map((section) => ({
    section,
    rows: items.filter((d) => sectionOf(d) === section),
  })).filter((g) => g.rows.length > 0);

  return (
    <div
      className="ld-dl-panel"
      data-testid="ld-downloads-panel"
      data-pagestatus="ready"
      tabIndex={0}
      onKeyDown={onKeyDown}
    >
      {grouped.map(({ section, rows }) => (
        <div key={section} className="ld-dl-group" data-testid={`ld-dl-group-${section}`}>
          <div className="ld-dl-group-head">
            {SECTION_LABELS[section]} <span className="ld-dl-count">{rows.length}</span>
          </div>
          {rows.map((d) => (
            <DownloadItem key={d.id} d={d} onArchive={onArchive} onRetry={onRetry} busy={busy} />
          ))}
        </div>
      ))}
    </div>
  );
}

export function DownloadsPanelHeader({ onClose }: { onClose: () => void }) {
  return (
    <div className="dp-head">
      <div className="dp-head-icon" style={{ background: 'var(--c-saffron-bg)', color: 'var(--c-saffron-text)' }}>
        <ShellIcon name="download" size={15} />
      </div>
      <div className="dp-head-body">
        <div className="dp-head-title">Downloads</div>
        <div className="dp-head-sub">Active and recent pulls</div>
      </div>
      <button className="dp-close" onClick={onClose} title="Close" data-testid="ld-downloads-close">
        <ShellIcon name="x" size={15} />
      </button>
    </div>
  );
}
