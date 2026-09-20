import { cn } from '@/lib/utils';

import { ShellIcon } from './ShellIcon';

export interface ShellRailToggleProps {
  icon?: string;
  title?: string;
  open: boolean;
  onToggle: () => void;
}

export function ShellRailToggle({
  icon = 'panel-right',
  title = 'Toggle detail panel',
  open,
  onToggle,
}: ShellRailToggleProps) {
  return (
    <button
      type="button"
      className={cn('shell-icon-btn shell-rail-toggle', icon === 'panel-right' && 'is-panel', open && 'is-on')}
      onClick={onToggle}
      title={title}
      aria-label={title}
      aria-pressed={open}
      data-testid="shell-rail-toggle"
    >
      <ShellIcon name={icon} size={16} />
    </button>
  );
}
