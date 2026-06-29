import { Check } from 'lucide-react';

import { cn } from '@/lib/utils';

interface ListingToggleProps {
  checked: boolean;
  onToggle: () => void;
  label: string;
  /** Endpoint hint chip, e.g. `/v1/models`. */
  code?: string;
  description: string;
  /** When the access mode is `all`, listing is already implied — soft hint only. */
  redundant?: boolean;
  disabled?: boolean;
  testId?: string;
}

/** Standalone listing permission (S3 ListBucket-style) — independent of the
 *  inference/connect tier. With it off the discovery endpoint returns only the
 *  individually granted resources. */
export function ListingToggle({
  checked,
  onToggle,
  label,
  code,
  description,
  redundant = false,
  disabled = false,
  testId,
}: ListingToggleProps) {
  return (
    <div
      className={cn('ap-listing', checked && 'on', disabled && 'is-disabled')}
      role="checkbox"
      aria-checked={checked}
      aria-disabled={disabled}
      tabIndex={disabled ? -1 : 0}
      data-testid={testId}
      onClick={() => !disabled && onToggle()}
      onKeyDown={(e) => {
        if (disabled) return;
        if (e.key === ' ' || e.key === 'Enter') {
          e.preventDefault();
          onToggle();
        }
      }}
    >
      <div className={cn('ap-listing-check', checked && 'on')}>{checked && <Check strokeWidth={3} />}</div>
      <div className="ap-listing-main">
        <div className="ap-listing-title">
          {label}
          {code && <span className="ap-listing-code">{code}</span>}
        </div>
        <div className="ap-listing-desc">
          {description}
          {redundant && <span className="ap-listing-implied"> · “All” already lists everything</span>}
        </div>
      </div>
    </div>
  );
}
