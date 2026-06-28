import { BASE_PATH, ROUTE_CHAT } from '@/lib/constants';

export interface ShellBrandProps {
  collapsed?: boolean;
}

export function ShellBrand({ collapsed }: ShellBrandProps) {
  return (
    <a href={ROUTE_CHAT} style={{ display: 'flex', alignItems: 'center', gap: 10 }}>
      <img
        src={`${BASE_PATH}/bodhi-logo/bodhi-logo-60.svg`}
        alt="Bodhi"
        onError={(e) => {
          (e.currentTarget as HTMLImageElement).style.display = 'none';
        }}
      />
      {!collapsed && (
        <span>
          <span className="shell-brand-t" style={{ display: 'block' }}>
            Bodhi
          </span>
          <span className="shell-brand-s" style={{ display: 'block' }}>
            AI Gateway
          </span>
        </span>
      )}
    </a>
  );
}
