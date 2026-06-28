import { type ReactNode, type RefObject, useEffect, useLayoutEffect, useRef, useState } from 'react';

import { useTheme } from '@/components/ThemeProvider';

import { ShellIcon } from './ShellIcon';

/* ── User footer chip ──────────────────────
   The parent (RootShell in `__root.tsx`) wires the real `user` via `useGetUser` and the
   `onLogout` handler via `useLogoutHandler`. The fallback placeholder values are kept only as
   a safety net so the chip still renders if a screen mounts ShellFooter without props. */
export interface ShellFooterUser {
  initials?: string;
  name?: string;
  email?: string;
  role?: string;
}

export interface ShellFooterProps {
  user: ShellFooterUser;
  collapsed?: boolean;
  onLogout?: () => void;
  logoutPending?: boolean;
}

export function ShellFooter({ user, collapsed, onLogout, logoutPending }: ShellFooterProps) {
  const u = {
    initials: user.initials || '?',
    name: user.name || 'Guest',
    email: user.email || '',
    role: user.role || '',
  };

  const [open, setOpen] = useState(false);
  const anchorRef = useRef<HTMLButtonElement>(null);

  // Disable column resize handles while the menu is open — they sit at the
  // sidebar edge and otherwise conflict with the popup.
  useEffect(() => {
    document.body.classList.toggle('shell-menu-open', open);
    return () => document.body.classList.remove('shell-menu-open');
  }, [open]);

  function logout() {
    setOpen(false);
    onLogout?.();
  }

  const toggle = (e: React.MouseEvent) => {
    e.stopPropagation();
    setOpen((o) => !o);
  };

  const chip = collapsed ? (
    <button
      ref={anchorRef}
      className={'shell-avatar shell-tip shell-userbtn-collapsed' + (open ? ' on' : '')}
      data-tip={u.name + ' · ' + u.role}
      onClick={toggle}
    >
      {u.initials}
    </button>
  ) : (
    <button ref={anchorRef} className={'shell-userbtn' + (open ? ' on' : '')} onClick={toggle}>
      <span className="shell-avatar">{u.initials}</span>
      <span className="shell-userbtn-meta">
        <span className="shell-foot-name">{u.name}</span>
        <span className="shell-foot-role">{u.role}</span>
      </span>
      <span className="shell-userbtn-chev">
        <ShellIcon name="chevron-up" size={14} />
      </span>
    </button>
  );

  return (
    <>
      {/* Always-visible theme switch, above the user chip. Collapsed: icon-only stack. */}
      <ThemeSwitch collapsed={collapsed} />
      {chip}
      <UserMenuPop open={open} anchorRef={anchorRef} collapsed={collapsed} onClose={() => setOpen(false)}>
        <div className="shell-um-head">
          <span className="shell-avatar shell-um-avatar">{u.initials}</span>
          <span className="shell-um-id">
            <span className="shell-um-name">{u.name}</span>
            <span className="shell-um-email">{u.email}</span>
          </span>
        </div>
        <div className="shell-um-items">
          <button
            className="shell-um-item shell-um-logout"
            onClick={logout}
            disabled={logoutPending}
            data-testid="shell-footer-logout"
          >
            <ShellIcon name="log-out" size={14} />
            <span className="shell-um-label">{logoutPending ? 'Logging out…' : 'Log out'}</span>
          </button>
        </div>
      </UserMenuPop>
    </>
  );
}

/* ── Theme switch — always visible in the sidebar footer, above the user chip ──
   A 3-segment Light/Dark/System control (icon-only when the sidebar is collapsed),
   so migrated screens are easy to eyeball in both modes. Uses ThemeProvider. */
const THEME_OPTIONS: { id: 'light' | 'dark' | 'system'; label: string; icon: string }[] = [
  { id: 'light', label: 'Light', icon: 'sun' },
  { id: 'dark', label: 'Dark', icon: 'moon' },
  { id: 'system', label: 'System', icon: 'monitor' },
];

function ThemeSwitch({ collapsed }: { collapsed?: boolean }) {
  const { theme, setTheme } = useTheme();

  // Collapsed: a single button showing the current theme's icon, cycling
  // Light → Dark → System on click (all three reachable without expanding).
  if (collapsed) {
    const idx = THEME_OPTIONS.findIndex((o) => o.id === theme);
    const cur = THEME_OPTIONS[idx === -1 ? 2 : idx];
    const next = THEME_OPTIONS[(idx + 1) % THEME_OPTIONS.length];
    return (
      <div className="shell-theme is-collapsed" role="group" aria-label="Theme" data-testid="shell-theme-switch">
        <button
          type="button"
          className="shell-theme-btn on"
          aria-label={`Theme: ${cur.label}. Switch to ${next.label}`}
          onClick={() => setTheme(next.id)}
          data-testid="shell-theme-cycle"
        >
          <ShellIcon name={cur.icon} size={15} />
        </button>
      </div>
    );
  }

  return (
    <div className="shell-theme" role="group" aria-label="Theme" data-testid="shell-theme-switch">
      {THEME_OPTIONS.map((opt) => (
        <button
          key={opt.id}
          type="button"
          className={'shell-theme-btn' + (theme === opt.id ? ' on' : '')}
          aria-pressed={theme === opt.id}
          aria-label={opt.label}
          onClick={() => setTheme(opt.id)}
          data-testid={`shell-theme-${opt.id}`}
        >
          <ShellIcon name={opt.icon} size={15} />
        </button>
      ))}
    </div>
  );
}

/* ── User menu popover (fixed; opens UP from the chip, RIGHT when collapsed) ── */
interface UserMenuPopProps {
  open: boolean;
  anchorRef: RefObject<HTMLElement | null>;
  collapsed?: boolean;
  onClose: () => void;
  children: ReactNode;
}

function UserMenuPop({ open, anchorRef, collapsed, onClose, children }: UserMenuPopProps) {
  const popRef = useRef<HTMLDivElement>(null);
  const [pos, setPos] = useState<{ top: number; left: number } | null>(null);

  useLayoutEffect(() => {
    if (!open || !anchorRef.current) {
      setPos(null);
      return;
    }
    const a = anchorRef.current.getBoundingClientRect();
    const ph = popRef.current ? popRef.current.offsetHeight : 280;
    const pw = popRef.current ? popRef.current.offsetWidth : 256;
    let top: number;
    let left: number;
    if (collapsed) {
      left = a.right + 10;
      top = a.bottom - ph;
    } else {
      left = a.left;
      top = a.top - ph - 8;
    }
    if (top < 8) top = 8;
    if (left + pw > window.innerWidth - 8) left = Math.max(8, window.innerWidth - 8 - pw);
    setPos({ top, left });
  }, [open, collapsed, anchorRef]);

  useEffect(() => {
    if (!open) return;
    const h = () => onClose();
    const k = (e: KeyboardEvent) => {
      if (e.key === 'Escape') onClose();
    };
    document.addEventListener('click', h);
    document.addEventListener('keydown', k);
    return () => {
      document.removeEventListener('click', h);
      document.removeEventListener('keydown', k);
    };
  }, [open, onClose]);

  if (!open) return null;
  return (
    <div
      ref={popRef}
      className="shell-usermenu"
      style={{ top: pos ? pos.top : -9999, left: pos ? pos.left : -9999 }}
      onClick={(e) => e.stopPropagation()}
    >
      {children}
    </div>
  );
}
