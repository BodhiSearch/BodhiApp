import { useRef, useState } from 'react';

import { AnchoredPopover } from './ShellChrome';
import { useShell } from './ShellContext';
import { ShellIcon } from './ShellIcon';

export interface ShellFilterChip {
  label: string;
  color?: 'indigo' | 'lotus' | 'saffron' | 'leaf' | 'teal' | 'neutral';
  defaultOn?: boolean;
}

export interface ShellFilterGroupProps {
  icon?: string;
  label: string;
  chips?: ShellFilterChip[];
  note?: string;
  clearable?: boolean;
}

export function ShellFilterGroup({ icon = 'filter', label, chips = [], note, clearable }: ShellFilterGroupProps) {
  const { collapsed, openPop, setOpenPop } = useShell();
  const popId = 'fg:' + label;
  const open = openPop === popId;
  const [sel, setSel] = useState<Set<string>>(() => new Set(chips.filter((c) => c.defaultOn).map((c) => c.label)));
  const anchorRef = useRef<HTMLButtonElement>(null);

  const toggle = (lbl: string) =>
    setSel((prev) => {
      const next = new Set(prev);
      if (next.has(lbl)) next.delete(lbl);
      else next.add(lbl);
      return next;
    });
  const clear = () => setSel(new Set());

  const chipEls = chips.map((c) => (
    <button
      key={c.label}
      data-tip={c.label}
      className={'shell-fc fc-' + (c.color || 'neutral') + (sel.has(c.label) ? ' on' : '')}
      onClick={() => toggle(c.label)}
    >
      {c.label}
    </button>
  ));

  if (collapsed) {
    return (
      <>
        <button
          ref={anchorRef}
          className={'shell-railbtn shell-tip' + (sel.size ? ' on' : '')}
          data-tip={label}
          onClick={(e) => {
            e.stopPropagation();
            setOpenPop(open ? null : popId);
          }}
        >
          <ShellIcon name={icon} size={17} />
          {sel.size > 0 && <span className="rb-badge">{sel.size}</span>}
        </button>
        <AnchoredPopover open={open} anchorRef={anchorRef} onClose={() => setOpenPop(null)}>
          <div className="shell-pop-title">
            <span>{label}</span>
            {clearable && sel.size > 0 && (
              <button className="fg-clear" onClick={clear}>
                Clear
              </button>
            )}
          </div>
          <div className="shell-pop-chips">{chipEls}</div>
        </AnchoredPopover>
      </>
    );
  }

  return (
    <div className="shell-filtergroup">
      <div className="shell-fg-label">
        <span className="fg-ico">
          <ShellIcon name={icon} size={13} />
        </span>
        <span className="fg-name">
          {label}
          {note && <span className="fg-note"> {note}</span>}
        </span>
        {clearable && sel.size > 0 && (
          <button className="fg-clear" onClick={clear}>
            Clear
          </button>
        )}
      </div>
      <div className="shell-fc-row">{chipEls}</div>
    </div>
  );
}
