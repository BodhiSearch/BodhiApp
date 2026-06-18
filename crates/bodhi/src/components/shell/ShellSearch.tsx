import { type FocusEvent, type KeyboardEvent } from 'react';

import { ShellIcon } from './ShellIcon';

export interface ShellSearchProps {
  value?: string;
  onChange?: (value: string) => void;
  placeholder?: string;
  size?: 'md' | 'sm';
  kbd?: string;
  autoFocus?: boolean;
  onKeyDown?: (e: KeyboardEvent<HTMLInputElement>) => void;
  onBlur?: (e: FocusEvent<HTMLInputElement>) => void;
}

export function ShellSearch({
  value = '',
  onChange,
  placeholder,
  size = 'md',
  kbd,
  autoFocus,
  onKeyDown,
  onBlur,
}: ShellSearchProps) {
  const cls = 'shell-search' + (size === 'sm' ? ' sm' : '') + (kbd ? ' has-kbd' : '');
  return (
    <div className={cls}>
      <span className="ss-ico">
        <ShellIcon name="search" size={size === 'sm' ? 12 : 14} />
      </span>
      <input
        type="text"
        placeholder={placeholder}
        value={value}
        autoFocus={autoFocus}
        onKeyDown={onKeyDown}
        onBlur={onBlur}
        onChange={(e) => onChange?.(e.target.value)}
      />
      {kbd && <span className="ss-kbd">{kbd}</span>}
    </div>
  );
}
