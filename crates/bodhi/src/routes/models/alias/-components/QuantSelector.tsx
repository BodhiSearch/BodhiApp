import { useMemo } from 'react';

import { CheckCircle, Info, Loader2 } from 'lucide-react';

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
      <div className="lf-quant-note" data-testid="quant-loading">
        <Loader2 className="h-3.5 w-3.5 animate-spin" />
        Fetching quantisations for <span className="lf-mono">{repo.trim()}</span>…
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
        <p className="lf-hint mt-1.5">
          {parsed
            ? 'No catalog quantisations found — enter the GGUF filename to download.'
            : 'Enter an <org>/<repo> above to list quantisations, or type the GGUF filename here.'}
        </p>
      </div>
    );
  }

  const selectedStatus = value ? statusOf(value) : null;

  return (
    <div data-testid="quant-table">
      <div className="lf-table-scroll">
        <div className="lf-table-wrap">
          <table className="lf-table">
            <thead>
              <tr>
                <th className="lf-th" style={{ width: 28 }} />
                <th className="lf-th">Quant</th>
                <th className="lf-th">Size</th>
                <th className="lf-th">Status</th>
              </tr>
            </thead>
            <tbody>
              {quants.map((q) => {
                const status = statusOf(q.filename);
                const isSelected = value === q.filename;
                return (
                  <tr
                    key={q.filename}
                    className={`lf-tr${isSelected ? ' lf-tr-sel' : ''}`}
                    onClick={() => onSelect(q.filename)}
                    data-testid={`quant-row-${q.name}`}
                    data-test-state={isSelected ? 'selected' : 'idle'}
                  >
                    <td className="lf-td">
                      <span className={`lf-qradio${isSelected ? ' lf-qradio-on' : ''}`}>
                        {isSelected && <span className="lf-qradio-dot" />}
                      </span>
                    </td>
                    <td className="lf-td lf-mono" style={{ fontWeight: isSelected ? 600 : 400 }}>
                      {q.name}
                    </td>
                    <td className="lf-td lf-mono">{fmtSize(q.size)}</td>
                    <td className="lf-td">
                      <span className={`lf-status lf-status-${status}`} data-testid={`quant-status-${q.name}`}>
                        {STATUS_LABEL[status]}
                      </span>
                    </td>
                  </tr>
                );
              })}
            </tbody>
          </table>
        </div>
      </div>

      {value && selectedStatus !== 'downloaded' && (
        <div className="lf-quant-note mt-2.5" data-testid="quant-download-note">
          <Info className="h-3.5 w-3.5" />
          <span>
            {value} is {selectedStatus === 'downloading' ? 'downloading' : 'not downloaded yet'} — it will download
            automatically after save.
          </span>
        </div>
      )}
      {value && selectedStatus === 'downloaded' && (
        <div className="lf-quant-note lf-quant-note-ok mt-2.5" data-testid="quant-downloaded-note">
          <CheckCircle className="h-3.5 w-3.5" />
          <span>{value} is already downloaded locally.</span>
        </div>
      )}
    </div>
  );
}
