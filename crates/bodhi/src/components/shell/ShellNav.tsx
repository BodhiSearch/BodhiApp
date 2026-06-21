import { useEffect, useMemo, useRef } from 'react';

import { Link } from '@tanstack/react-router';

import { useGetAppInfo } from '@/hooks/info';

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
  const { data: appInfo } = useGetAppInfo();
  const isMultiTenant = appInfo?.deployment === 'multi_tenant';

  // Drop sub-pages flagged hideInMultiTenant (e.g. local-model catalog — no downloads there).
  const cur = useMemo(() => {
    const base = SHELL_NAV.find((n) => n.id === section) || SHELL_NAV[0];
    if (!isMultiTenant) return base;
    return { ...base, subPages: base.subPages.filter((sp) => !sp.hideInMultiTenant) };
  }, [section, isMultiTenant]);

  useEffect(() => {
    if (!open) return;
    const h = () => setOpenPop(null);
    document.addEventListener('click', h);
    return () => document.removeEventListener('click', h);
  }, [open, setOpenPop]);

  const menuItems = SHELL_NAV.map((item) => (
    <Link
      key={item.id}
      to={item.href}
      data-testid={`shell-nav-${item.id}`}
      className={'shell-nav-item' + (item.id === section ? ' on' : '')}
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
          onClick={(e) => {
            e.stopPropagation();
            setOpenPop(open ? null : 'nav');
          }}
        >
          <ShellIcon name={cur.icon} size={18} />
        </button>
        <AnchoredPopover open={open} anchorRef={anchorRef} onClose={() => setOpenPop(null)}>
          <div className="shell-pop-title">Go to section</div>
          {menuItems}
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
          className="shell-nav-trigger"
          data-testid={`shell-nav-trigger`}
          data-tip="Switch section"
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
        {open && <div className="shell-nav-menu">{menuItems}</div>}
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
