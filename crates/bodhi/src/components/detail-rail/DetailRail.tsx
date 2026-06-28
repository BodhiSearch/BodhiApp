import type { ReactNode } from 'react';

/**
 * Composition primitives for the V2 detail rail (the `dp-*` panel shown when a list row is
 * selected). Nine rail panels shared this markup and each re-declared a local `Row` helper;
 * these slots de-duplicate that while keeping the existing CSS classes (zero visual change).
 * Screens keep their own bespoke sections (capability chips, served-by lists, footers) and
 * just compose them inside these wrappers.
 */

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

/**
 * A single key/value row. Renders nothing when the value is null/undefined/empty (the behavior the
 * per-rail `Row` helpers all shared). Value text is monospaced by default.
 */
export function DetailRailRow({ k, v, mono = true }: { k: string; v: ReactNode; mono?: boolean }) {
  if (v == null || v === '') return null;
  return (
    <div className="dp-row">
      <span className="dp-row-k">{k}</span>
      <span className={mono ? 'dp-row-v mono' : 'dp-row-v'}>{v}</span>
    </div>
  );
}
