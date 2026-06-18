import { useShell } from './ShellContext';
import { ShellIcon } from './ShellIcon';

export interface ShellModeOption {
  id: string;
  label: string;
  sub?: string;
  icon?: string;
}

export interface ShellModeSwitchProps {
  value: string;
  onChange: (id: string) => void;
  options?: ShellModeOption[];
  label?: string;
}

export function ShellModeSwitch({ value, onChange, options = [], label }: ShellModeSwitchProps) {
  const { collapsed } = useShell();
  if (collapsed) {
    return (
      <>
        {options.map((o) => (
          <button
            key={o.id}
            className={'shell-railbtn shell-tip' + (o.id === value ? ' on' : '')}
            data-tip={o.label}
            onClick={() => onChange(o.id)}
          >
            <ShellIcon name={o.icon || 'circle'} size={18} />
          </button>
        ))}
      </>
    );
  }
  return (
    <div>
      {label && (
        <div className="shell-fg-label" style={{ paddingTop: 10 }}>
          <span className="fg-name">{label}</span>
        </div>
      )}
      <div className="shell-modeswitch">
        {options.map((o) => (
          <button
            key={o.id}
            data-tip={o.label + (o.sub ? ' — ' + o.sub : '')}
            className={'shell-mode-card' + (o.id === value ? ' on' : '')}
            onClick={() => onChange(o.id)}
          >
            <span className="shell-mode-ico">
              <ShellIcon name={o.icon || 'circle'} size={16} />
            </span>
            <span>
              <span className="shell-mode-label" style={{ display: 'block' }}>
                {o.label}
              </span>
              {o.sub && (
                <span className="shell-mode-sub" style={{ display: 'block' }}>
                  {o.sub}
                </span>
              )}
            </span>
          </button>
        ))}
      </div>
    </div>
  );
}
