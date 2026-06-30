import { useCallback, useEffect, useState } from 'react';

import { useNavigate } from '@tanstack/react-router';

import { ShellIcon } from '@/components/shell';
import { useViewTransition } from '@/hooks/useViewTransition';

export const fmtDate = (iso: string) =>
  new Date(iso).toLocaleDateString(undefined, { month: 'short', day: 'numeric', year: 'numeric' });

const readSelectFromUrl = () => new URLSearchParams(window.location.search).get('select');

// Render source of truth is local; the URL is mirrored (replace, no history entries) so links are
// shareable and browser Back/Forward works — the popstate listener pulls the id back out of the URL.
export function useUrlMirroredSelection(routeTo: string) {
  const navigate = useNavigate();
  const withViewTransition = useViewTransition();
  const [selectedId, setSelectedId] = useState<string | null>(() => readSelectFromUrl());

  useEffect(() => {
    const onPop = () => setSelectedId(readSelectFromUrl());
    window.addEventListener('popstate', onPop);
    return () => window.removeEventListener('popstate', onPop);
  }, []);

  const select = useCallback(
    (id: string | null) => {
      withViewTransition(() => setSelectedId(id));
      navigate({
        to: routeTo,
        search: (prev: Record<string, unknown>) => ({ ...prev, select: id ?? undefined }),
        replace: true,
      } as never);
    },
    [withViewTransition, navigate, routeTo]
  );

  return { selectedId, select };
}

export function DetailRow({ icon, label, value }: { icon: string; label: string; value: string }) {
  return (
    <div className="dp-row">
      <span className="dp-row-k">
        <ShellIcon name={icon} size={13} /> {label}
      </span>
      <span className="dp-row-v">{value}</span>
    </div>
  );
}

// Normalized grant shape shared by `ModelGrant`/`McpGrant` (token grants) and `ResourceAccess`
// (app access), which carries an extra `list` field but is structurally compatible here.
type GrantAccess = { type: 'all' } | { type: 'specific'; ids: string[] };

export function grantSummary(access: GrantAccess | undefined, noun: string): string {
  if (!access || access.type === 'all') return `All ${noun}s`;
  const n = access.ids.length;
  return n ? `${n} ${noun}${n === 1 ? '' : 's'}` : `No ${noun}s`;
}

export function GrantChips({ ids, testIdPrefix }: { ids: string[]; testIdPrefix: string }) {
  if (ids.length === 0) return null;
  return (
    <div className="dp-chips">
      {ids.map((m) => (
        <span key={m} className="dp-chip" data-testid={`${testIdPrefix}-${m}`}>
          {m}
        </span>
      ))}
    </div>
  );
}
