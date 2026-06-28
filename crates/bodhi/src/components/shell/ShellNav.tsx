import { useEffect, useMemo, useRef, useState } from 'react';

import { Link } from '@tanstack/react-router';

import { useGetAppInfo } from '@/hooks/info';
import { useGetUser } from '@/hooks/users';
import { isAdminRole } from '@/lib/roles';

import { SHELL_NAV } from './shell-nav-config';
import { AnchoredPopover } from './ShellChrome';
import { useShell } from './ShellContext';
import { ShellIcon } from './ShellIcon';

const LOTUS = '#DB456C';

export interface ShellNavProps {
  section?: string;
  subPage?: string | null;
}

export function ShellNav({ section = 'chat', subPage = null }: ShellNavProps) {
  const { collapsed, openPop, setOpenPop } = useShell();
  const open = openPop === 'nav';
  const anchorRef = useRef<HTMLButtonElement>(null);
  const itemRefs = useRef<(HTMLAnchorElement | null)[]>([]);
  const [focusIndex, setFocusIndex] = useState(0);
  const wasOpen = useRef(false);
  const { data: appInfo } = useGetAppInfo();
  const isMultiTenant = appInfo?.deployment === 'multi_tenant';
  const { data: userInfo } = useGetUser();
  const isAdmin = userInfo?.auth_status === 'logged_in' && userInfo.role ? isAdminRole(userInfo.role) : false;

  // Drop sub-pages the current context can't use: hideInMultiTenant (e.g. local-model catalog — no
  // downloads there) and adminOnly (e.g. MCP server registration) for non-admins.
  const cur = useMemo(() => {
    const base = SHELL_NAV.find((n) => n.id === section) || SHELL_NAV[0];
    const subPages = base.subPages.filter(
      (sp) => !(isMultiTenant && sp.hideInMultiTenant) && !(sp.adminOnly && !isAdmin)
    );
    return { ...base, subPages };
  }, [section, isMultiTenant, isAdmin]);

  useEffect(() => {
    if (!open) return;
    const h = () => setOpenPop(null);
    document.addEventListener('click', h);
    return () => document.removeEventListener('click', h);
  }, [open, setOpenPop]);

  const activeIndex = Math.max(
    0,
    SHELL_NAV.findIndex((n) => n.id === section)
  );

  // Move focus into the menu (onto the active item) when it opens, and return focus to the
  // trigger when it closes — so arrow keys work immediately and the keyboard never gets stranded.
  useEffect(() => {
    if (open && !wasOpen.current) {
      setFocusIndex(activeIndex);
      requestAnimationFrame(() => itemRefs.current[activeIndex]?.focus());
    } else if (!open && wasOpen.current) {
      anchorRef.current?.focus();
    }
    wasOpen.current = open;
  }, [open, activeIndex]);

  // Roving focus among the items: ↑/↓ move (no wrap), Home/End jump, Escape closes (the only
  // close path on the expanded sidebar, which has no AnchoredPopover wrapper). Enter/Space fall
  // through so the focused <Link> activates natively (its onClick also closes the menu).
  function onItemKeyDown(e: React.KeyboardEvent, idx: number) {
    if (e.ctrlKey || e.metaKey || e.altKey) return;
    let next = idx;
    if (e.key === 'ArrowDown') next = Math.min(SHELL_NAV.length - 1, idx + 1);
    else if (e.key === 'ArrowUp') next = Math.max(0, idx - 1);
    else if (e.key === 'Home') next = 0;
    else if (e.key === 'End') next = SHELL_NAV.length - 1;
    else if (e.key === 'Escape') {
      e.preventDefault();
      setOpenPop(null);
      return;
    } else return;
    e.preventDefault();
    setFocusIndex(next);
    itemRefs.current[next]?.focus();
  }

  const menuItems = SHELL_NAV.map((item, idx) => (
    <Link
      key={item.id}
      to={item.href}
      ref={(el) => {
        itemRefs.current[idx] = el;
      }}
      role="menuitem"
      tabIndex={idx === focusIndex ? 0 : -1}
      data-testid={`shell-nav-${item.id}`}
      className={'shell-nav-item' + (item.id === section ? ' on' : '')}
      onClick={() => setOpenPop(null)}
      onKeyDown={(e) => onItemKeyDown(e, idx)}
    >
      <ShellIcon name={item.icon} color={item.id === section ? LOTUS : 'currentColor'} />
      {item.label}
      {item.badge && <span className="shell-nav-badge">{item.badge}</span>}
    </Link>
  ));

  if (collapsed) {
    return (
      <>
        <button
          ref={anchorRef}
          className="shell-railbtn shell-tip on"
          data-testid={`shell-nav-${cur.id}`}
          data-tip={cur.label + ' · switch section'}
          aria-haspopup="menu"
          aria-expanded={open}
          onClick={(e) => {
            e.stopPropagation();
            setOpenPop(open ? null : 'nav');
          }}
        >
          <ShellIcon name={cur.icon} size={18} />
        </button>
        <AnchoredPopover open={open} anchorRef={anchorRef} onClose={() => setOpenPop(null)}>
          <div className="shell-pop-title">Go to section</div>
          <div role="menu">{menuItems}</div>
        </AnchoredPopover>
        {cur.subPages && cur.subPages.length > 0 && <div className="shell-iconrail-div" />}
        {cur.subPages &&
          cur.subPages.map((sp) => (
            <Link
              key={sp.id}
              to={sp.href}
              data-testid={`shell-sub-${sp.id}`}
              className={'shell-railbtn shell-tip' + (sp.id === subPage ? ' on' : '')}
              data-tip={sp.label}
            >
              <ShellIcon name={sp.icon || 'circle'} size={17} />
              {sp.badge && <span className="rb-badge">{sp.badge}</span>}
            </Link>
          ))}
      </>
    );
  }

  return (
    <div className="shell-nav-block">
      <div className={'shell-nav' + (open ? ' open' : '')} onClick={(e) => e.stopPropagation()}>
        <button
          ref={anchorRef}
          className="shell-nav-trigger"
          data-testid={`shell-nav-trigger`}
          data-tip="Switch section"
          aria-haspopup="menu"
          aria-expanded={open}
          onClick={(e) => {
            e.stopPropagation();
            setOpenPop(open ? null : 'nav');
          }}
        >
          <span className="lead">
            <ShellIcon name={cur.icon} size={15} color={LOTUS} />
          </span>
          <span className="lbl">{cur.label}</span>
          <span className="chev">
            <ShellIcon name="chevron-down" />
          </span>
        </button>
        {open && (
          <div className="shell-nav-menu" role="menu">
            {menuItems}
          </div>
        )}
      </div>
      {cur.subPages && cur.subPages.length > 0 && (
        <div className="shell-sub">
          {cur.subPages.map((sp) => (
            <Link
              key={sp.id}
              to={sp.href}
              data-tip={sp.label}
              data-testid={`shell-sub-${sp.id}`}
              className={'shell-sub-item' + (sp.id === subPage ? ' on' : '')}
            >
              <ShellIcon name={sp.icon || 'circle'} size={13} color={sp.id === subPage ? LOTUS : 'currentColor'} />
              {sp.label}
              {sp.badge && <span className="shell-sub-badge">{sp.badge}</span>}
            </Link>
          ))}
        </div>
      )}
    </div>
  );
}
