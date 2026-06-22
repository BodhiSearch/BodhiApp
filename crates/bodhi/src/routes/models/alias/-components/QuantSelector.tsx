import { useMemo } from 'react';

import { Check, Loader2 } from 'lucide-react';

import { Input } from '@/components/ui/input';
import { useListDownloads, useListModelFiles } from '@/hooks/models';
import { useModelDetail } from '@/hooks/reference';

/** Per-quant local availability, derived from downloaded files + in-flight downloads. */
type QuantStatus = 'downloaded' | 'downloading' | 'remote';

const STATUS_LABEL: Record<QuantStatus, string> = {
  downloaded: 'Downloaded',
  downloading: 'Downloading',
  remote: 'Not downloaded',
};

/** `<org>/<repo>` → `{ namespace, repo }`; null when the input isn't a single org/repo pair. */
function splitRepo(repo: string): { namespace: string; repo: string } | null {
  const trimmed = repo.trim();
  const parts = trimmed.split('/');
  if (parts.length !== 2 || !parts[0] || !parts[1]) return null;
  return { namespace: parts[0], repo: parts[1] };
}

function fmtSize(bytes?: number | null): string {
  if (bytes == null) return '—';
  const gb = bytes / 1_000_000_000;
  return gb >= 1 ? `${gb.toFixed(1)} GB` : `${(bytes / 1_000_000).toFixed(0)} MB`;
}

interface QuantSelectorProps {
  /** The `<org>/<repo>` the form currently targets. */
  repo: string;
  /** Selected filename (the chosen quant, or a manually-typed filename). */
  value: string;
  onSelect: (filename: string) => void;
}

/**
 * Picks the GGUF file for a local alias from the reference catalog's quant table, annotated with
 * local download status. When the repo isn't in the catalog (or has no quants) it falls back to a
 * plain filename input so any repo/file still works.
 */
export function QuantSelector({ repo, value, onSelect }: QuantSelectorProps) {
  const parsed = splitRepo(repo);
  const selected = parsed ? { source: 'huggingface', namespace: parsed.namespace, repo: parsed.repo } : null;
  const { data: detail, isLoading, isError } = useModelDetail(selected);

  const { data: filesData } = useListModelFiles(1, 100, 'repo', 'asc');
  const { data: downloadsData } = useListDownloads(1, 100, { enablePolling: true });

  // Filenames already on disk / actively downloading for this repo, for status correlation.
  const downloadedNames = useMemo(() => {
    const set = new Set<string>();
    (filesData?.data ?? []).forEach((f) => {
      if (f.repo === repo.trim()) set.add(f.filename);
    });
    return set;
  }, [filesData, repo]);

  const downloadingNames = useMemo(() => {
    const set = new Set<string>();
    (downloadsData?.data ?? []).forEach((d) => {
      if (d.repo === repo.trim() && d.status === 'pending') set.add(d.filename);
    });
    return set;
  }, [downloadsData, repo]);

  const statusOf = (filename: string): QuantStatus => {
    if (downloadedNames.has(filename)) return 'downloaded';
    if (downloadingNames.has(filename)) return 'downloading';
    return 'remote';
  };

  const quants = detail?.quants ?? [];

  if (parsed && isLoading) {
    return (
      <div className="flex items-center gap-2 text-sm text-muted-foreground py-3" data-testid="quant-loading">
        <Loader2 className="h-4 w-4 animate-spin" />
        Fetching quantisations for <span className="font-mono">{repo.trim()}</span>…
      </div>
    );
  }

  // Catalog miss (no repo, 404/error, or no quants) → manual filename entry keeps any repo usable.
  if (!parsed || isError || quants.length === 0) {
    return (
      <div data-testid="quant-manual">
        <Input
          value={value}
          onChange={(e) => onSelect(e.target.value)}
          placeholder="model-file.gguf"
          data-testid="filename-input"
          className="font-mono"
        />
        <p className="text-sm text-muted-foreground mt-1.5">
          {parsed
            ? 'No catalog quantisations found — enter the GGUF filename to download.'
            : 'Enter an <org>/<repo> above to list quantisations, or type the GGUF filename here.'}
        </p>
      </div>
    );
  }

  return (
    <div data-testid="quant-table">
      <div className="rounded-md border divide-y">
        {quants.map((q) => {
          const status = statusOf(q.filename);
          const isSelected = value === q.filename;
          return (
            <button
              type="button"
              key={q.filename}
              onClick={() => onSelect(q.filename)}
              data-testid={`quant-row-${q.name}`}
              data-test-state={isSelected ? 'selected' : 'idle'}
              className={`flex w-full items-center gap-3 px-3 py-2 text-left text-sm transition-colors hover:bg-muted/50 ${
                isSelected ? 'bg-muted' : ''
              }`}
            >
              <span className="flex h-4 w-4 items-center justify-center">
                {isSelected && <Check className="h-4 w-4 text-primary" />}
              </span>
              <span className={`font-mono ${isSelected ? 'font-semibold' : ''}`}>{q.name}</span>
              <span className="font-mono text-muted-foreground">{fmtSize(q.size)}</span>
              <span className="ml-auto text-xs text-muted-foreground" data-testid={`quant-status-${q.name}`}>
                {STATUS_LABEL[status]}
              </span>
            </button>
          );
        })}
      </div>
      {value && statusOf(value) === 'remote' && (
        <p className="text-sm text-muted-foreground mt-1.5" data-testid="quant-download-note">
          {value} is not downloaded yet — it will download automatically after save.
        </p>
      )}
    </div>
  );
}
