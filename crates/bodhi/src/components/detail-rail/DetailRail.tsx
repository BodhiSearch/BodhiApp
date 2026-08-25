import type { ReactNode } from 'react';

/** Composition primitives for the V2 detail rail (`dp-*` panel); de-duplicates a `Row` helper nine rail panels each re-declared. */

export function DetailRail({
  children,
  className,
  testId,
}: {
  children: ReactNode;
  className?: string;
  testId?: string;
}) {
  return (
    <div className={className ? `dp-panel ${className}` : 'dp-panel'} data-testid={testId}>
      {children}
    </div>
  );
}

export function DetailRailBody({ children }: { children: ReactNode }) {
  return <div className="dp-body">{children}</div>;
}

export function DetailRailSection({ label, children }: { label?: ReactNode; children: ReactNode }) {
  return (
    <div className="dp-section">
      {label != null && <div className="dp-sec-lbl">{label}</div>}
      {children}
    </div>
  );
}

export function DetailRailRows({ children, testId }: { children: ReactNode; testId?: string }) {
  return (
    <div className="dp-rows" data-testid={testId}>
      {children}
    </div>
  );
}

/** Renders nothing when the value is null/undefined/empty (behavior the per-rail `Row` helpers all shared). */
export function DetailRailRow({ k, v, mono = true }: { k: string; v: ReactNode; mono?: boolean }) {
  if (v == null || v === '') return null;
  return (
    <div className="dp-row">
      <span className="dp-row-k">{k}</span>
      <span className={mono ? 'dp-row-v mono' : 'dp-row-v'}>{v}</span>
    </div>
  );
}
