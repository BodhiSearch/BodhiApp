import type { ReactNode } from 'react';

import { ShellIcon } from '@/components/shell';

interface EmptyStateProps {
  /** kebab-case lucide icon name, e.g. 'search-x', 'key-round'. */
  icon: string;
  iconSize?: number;
  title: ReactNode;
  /** Optional supporting line; omit for a title-only empty state. */
  sub?: ReactNode;
  testId?: string;
}

/**
 * Shared list/catalog empty state — the `.empty-state` / `.empty-icon` / `.empty-title`
 * / `.empty-sub` block that the V2 screens render when a filtered list has no rows.
 * Styling lives in the existing CSS classes (unchanged); this only de-duplicates the markup.
 */
export function EmptyState({ icon, iconSize = 28, title, sub, testId }: EmptyStateProps) {
  return (
    <div className="empty-state" data-testid={testId}>
      <div className="empty-icon">
        <ShellIcon name={icon} size={iconSize} />
      </div>
      <div className="empty-title">{title}</div>
      {sub != null && <div className="empty-sub">{sub}</div>}
    </div>
  );
}
