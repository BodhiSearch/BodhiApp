import { type FocusEvent, type KeyboardEvent, useEffect, useRef } from 'react';

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
  const inputRef = useRef<HTMLInputElement>(null);

  // Wire ⌘K / Ctrl+K to focus this search input. Ignored while another input/textarea/select is
  // focused so existing inline editors keep their typing. The kbd label opts the instance in.
  useEffect(() => {
    if (!kbd) return;
    const onKey = (e: globalThis.KeyboardEvent) => {
      const isCmdOrCtrl = e.metaKey || e.ctrlKey;
      if (!isCmdOrCtrl || e.altKey || e.shiftKey) return;
      if (e.key !== 'k' && e.key !== 'K') return;
      const ae = document.activeElement;
      if (
        ae instanceof HTMLElement &&
        ae !== inputRef.current &&
        ae.closest('input, textarea, select, [contenteditable=""], [contenteditable="true"]')
      ) {
        return;
      }
      const node = inputRef.current;
      if (!node) return;
      e.preventDefault();
      node.focus();
      node.select();
    };
    document.addEventListener('keydown', onKey);
    return () => document.removeEventListener('keydown', onKey);
  }, [kbd]);

  return (
    <div className={cls}>
      <span className="ss-ico">
        <ShellIcon name="search" size={size === 'sm' ? 12 : 14} />
      </span>
      <input
        ref={inputRef}
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
