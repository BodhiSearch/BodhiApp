import { Fragment, type ReactNode, type RefObject, useEffect, useLayoutEffect, useRef, useState } from 'react';

import { useTheme } from '@/components/ThemeProvider';
import { BASE_PATH, ROUTE_CHAT } from '@/lib/constants';

import { ShellIcon } from './ShellIcon';

/* ── Global tooltip (fixed-position; escapes sidebar overflow) ──
   One listener delegates over `.shell-tip` elements only — i.e. the
   COLLAPSED icon-rail buttons (and the avatar). Expanded sidebar items
   already show their label inline, and popover/popup options carry no
   tooltip, so neither fires a hint. Flips left near the viewport edge. */
interface TipState {
  text: string | null;
  top: number;
  aRight: number;
  aLeft: number;
}

export function GlobalTooltip() {
  const [tip, setTip] = useState<TipState | null>(null);
  const ref = useRef<HTMLDivElement>(null);
  useEffect(() => {
    let cur: Element | null = null;
    let timer: ReturnType<typeof setTimeout> | undefined;
    const onOver = (e: MouseEvent) => {
      const target = e.target as Element | null;
      const el = target?.closest?.('.shell-tip[data-tip]') ?? null;
      if (el === cur) return;
      cur = el;
      clearTimeout(timer);
      if (!el) {
        setTip(null);
        return;
      }
      timer = setTimeout(() => {
        if (cur !== el) return;
        const r = el.getBoundingClientRect();
        setTip({ text: el.getAttribute('data-tip'), top: r.top + r.height / 2, aRight: r.right, aLeft: r.left });
      }, 150);
    };
    const onOut = (e: MouseEvent) => {
      const target = e.target as Element | null;
      const el = target?.closest?.('.shell-tip[data-tip]') ?? null;
      if (!el || el !== cur) return;
      const related = e.relatedTarget as Node | null;
      if (related && el.contains(related)) return;
      cur = null;
      clearTimeout(timer);
      setTip(null);
    };
    document.addEventListener('mouseover', onOver, true);
    document.addEventListener('mouseout', onOut, true);
    return () => {
      document.removeEventListener('mouseover', onOver, true);
      document.removeEventListener('mouseout', onOut, true);
      clearTimeout(timer);
    };
  }, []);
  useLayoutEffect(() => {
    if (!tip || !ref.current) return;
    const w = ref.current.offsetWidth;
    let left = tip.aRight + 10;
    if (left + w > window.innerWidth - 8) left = Math.max(8, tip.aLeft - 10 - w);
    ref.current.style.left = left + 'px';
  }, [tip]);
  if (!tip || !tip.text) return null;
  return (
    <div ref={ref} className="shell-tooltip" style={{ top: tip.top, left: tip.aRight + 10 }}>
      {tip.text}
    </div>
  );
}

/* ── Anchored popover (fixed-position, escapes overflow) ────── */
export interface AnchoredPopoverProps {
  open: boolean;
  anchorRef: RefObject<HTMLElement | null>;
  onClose: () => void;
  children: ReactNode;
}

export function AnchoredPopover({ open, anchorRef, onClose, children }: AnchoredPopoverProps) {
  const popRef = useRef<HTMLDivElement>(null);
  const [pos, setPos] = useState<{ top: number; left: number } | null>(null);

  useLayoutEffect(() => {
    if (!open || !anchorRef.current) {
      setPos(null);
      return;
    }
    const a = anchorRef.current.getBoundingClientRect();
    const ph = popRef.current ? popRef.current.offsetHeight : 260;
    let top = a.top;
    if (top + ph > window.innerHeight - 8) top = Math.max(8, window.innerHeight - 8 - ph);
    setPos({ top, left: a.right + 8 });
  }, [open, anchorRef]);

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
      className="shell-pop"
      style={{ top: pos ? pos.top : -9999, left: pos ? pos.left : -9999 }}
      onClick={(e) => e.stopPropagation()}
    >
      {children}
    </div>
  );
}

/* ── Default brand ────────────────────── */
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

/* ── Breadcrumb ────────────────────── */
export interface ShellBreadcrumbItem {
  label: string;
  href?: string;
  current?: boolean;
}

export interface ShellBreadcrumbProps {
  items?: ShellBreadcrumbItem[] | ReactNode;
}

/** Resolve a breadcrumb href so it works under the SPA's `/ui` basepath. Callers pass clean
 *  route paths (e.g. `/models/`); we prepend BASE_PATH for absolute paths and leave hashes,
 *  protocol-relative URLs, and externals untouched. */
function resolveBreadcrumbHref(href?: string): string {
  if (!href) return '#';
  if (href.startsWith('#')) return href;
  if (href.startsWith(BASE_PATH + '/') || href === BASE_PATH) return href;
  if (/^([a-z]+:)?\/\//i.test(href)) return href;
  if (href.startsWith('/')) return BASE_PATH + href;
  return href;
}

export function ShellBreadcrumb({ items }: ShellBreadcrumbProps) {
  if (!items) return null;
  if (!Array.isArray(items)) return <div className="shell-bc">{items}</div>;
  return (
    <div className="shell-bc">
      {items.map((it, i) => (
        <Fragment key={i}>
          {i > 0 && <ShellIcon name="chevron-right" size={11} />}
          {it.current ? (
            <span className="shell-bc-current">{it.label}</span>
          ) : (
            <a className="shell-bc-seg" href={resolveBreadcrumbHref(it.href)}>
              {it.label}
            </a>
          )}
        </Fragment>
      ))}
    </div>
  );
}
