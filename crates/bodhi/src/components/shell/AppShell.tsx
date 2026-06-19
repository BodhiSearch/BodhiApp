import {
  type CSSProperties,
  type PointerEvent as ReactPointerEvent,
  type ReactNode,
  useEffect,
  useRef,
  useState,
} from 'react';

import {
  GlobalTooltip,
  ShellBrand,
  ShellBreadcrumb,
  ShellFooter,
  type ShellBreadcrumbProps,
  type ShellFooterUser,
} from './ShellChrome';
import { ShellContext, type ShellContextValue } from './ShellContext';
import { ShellIcon } from './ShellIcon';
import { ShellNav } from './ShellNav';

import './shell.css';

const clamp = (v: number, a: number, b: number) => Math.max(a, Math.min(b, v));

export interface AppShellProps {
  /** primary-nav highlight + sub-pages */
  section?: string;
  subPage?: string | null;
  user?: ShellFooterUser;
  /** localStorage namespace for column widths */
  resizeKey?: string;

  sidebarWidth?: number;
  railWidth?: number;
  headerHeight?: number;
  bandHeight?: number;
  sbMin?: number;
  sbMax?: number;
  railMin?: number;
  railMax?: number;

  breadcrumb?: ShellBreadcrumbProps['items'];
  /** right of the header band (main) */
  headerActions?: ReactNode;

  brand?: ReactNode;
  /** page body below the nav (filters, etc.) */
  sidebar?: ReactNode;
  /** override user chip (optional) */
  footer?: ReactNode;
  /** main-column alert sub-band (optional) */
  banner?: ReactNode;

  /** shared toolbar band cells (optional) */
  toolbar?: ReactNode;
  sidebarToolbar?: ReactNode;
  railToolbar?: ReactNode;

  /** right panel (optional → 3rd column) */
  rail?: ReactNode;
  railHeader?: ReactNode;
  /** start with the rail showing (desktop) */
  railDefaultOpen?: boolean;

  contentClass?: string;
  /** set false to manage your own scroll region */
  mainScroll?: boolean;
  railScroll?: boolean;

  children?: ReactNode;
}

export function AppShell({
  section = 'chat',
  subPage = null,
  user = {},
  resizeKey,
  sidebarWidth = 240,
  railWidth = 340,
  headerHeight = 56,
  bandHeight = 52,
  sbMin = 190,
  sbMax = 380,
  railMin = 300,
  railMax = 560,
  breadcrumb,
  headerActions,
  brand,
  sidebar,
  footer,
  banner,
  toolbar,
  sidebarToolbar,
  railToolbar,
  rail,
  railHeader,
  railDefaultOpen = true,
  contentClass = '',
  mainScroll = true,
  railScroll = true,
  children,
}: AppShellProps) {
  const key = resizeKey ?? section;
  const shellRef = useRef<HTMLDivElement>(null);
  const [collapsed, setCollapsed] = useState(false); // icon rail (desktop/tablet)
  const [railCollapsed, setRailCollapsed] = useState(!railDefaultOpen);
  const [sbOpen, setSbOpen] = useState(false); // mobile drawer
  const [railOpen, setRailOpen] = useState(false); // mobile drawer
  const [dragging, setDragging] = useState(false);
  const [openPop, setOpenPop] = useState<string | null>(null); // which collapsed popover is open (only one)
  const [isMobile, setIsMobile] = useState<boolean>(
    () => typeof window !== 'undefined' && window.matchMedia('(max-width:767px)').matches
  );

  useEffect(() => {
    const m = window.matchMedia('(max-width:767px)');
    const h = (e: MediaQueryListEvent) => setIsMobile(e.matches);
    m.addEventListener('change', h);
    return () => m.removeEventListener('change', h);
  }, []);

  const hasRail = Boolean(rail);
  const hasBand = Boolean(toolbar || sidebarToolbar || railToolbar);
  const effCollapsed = collapsed && !isMobile;

  // When a screen publishes rail content (e.g. selecting a row), open the rail so
  // it's actually visible; clear it when the content goes away. Without this the
  // column stays at the initial collapsed width and the panel never shows.
  // On mobile the rail is a fixed drawer: open it whenever content is present (not just
  // on the false→true edge), so re-selecting a row after a manual close still opens it.
  useEffect(() => {
    if (hasRail) {
      if (isMobile) setRailOpen(true);
      else setRailCollapsed(false);
    } else {
      setRailOpen(false);
    }
  }, [hasRail, isMobile]);

  /* ── column resize (widths persist; collapse does not) ── */
  useEffect(() => {
    const shell = shellRef.current;
    if (!shell) return;
    const sw = parseFloat(localStorage.getItem(`bodhi.${key}.sideW`) ?? '');
    if (!isNaN(sw)) shell.style.setProperty('--shell-sb-w-user', clamp(sw, sbMin, sbMax) + 'px');
    const rw = parseFloat(localStorage.getItem(`bodhi.${key}.railW`) ?? '');
    if (!isNaN(rw)) shell.style.setProperty('--shell-rail-w-user', clamp(rw, railMin, railMax) + 'px');
  }, []);

  const startDrag = (side: 'left' | 'right', e: ReactPointerEvent) => {
    e.preventDefault();
    const shell = shellRef.current;
    if (!shell) return;
    const isLeft = side === 'left';
    const colEl = shell.querySelector(isLeft ? '.shell-sidebar' : '.shell-rail');
    if (!colEl) return;
    const varName = isLeft ? '--shell-sb-w-user' : '--shell-rail-w-user';
    const min = isLeft ? sbMin : railMin;
    const max = isLeft ? sbMax : railMax;
    const startX = e.clientX;
    const startW = colEl.getBoundingClientRect().width;
    setDragging(true);
    const move = (mv: PointerEvent) => {
      const dx = mv.clientX - startX;
      shell.style.setProperty(varName, clamp(isLeft ? startW + dx : startW - dx, min, max) + 'px');
    };
    const up = () => {
      window.removeEventListener('pointermove', move);
      window.removeEventListener('pointerup', up);
      setDragging(false);
      const v = parseFloat(shell.style.getPropertyValue(varName));
      if (!isNaN(v)) localStorage.setItem(`bodhi.${key}.${isLeft ? 'sideW' : 'railW'}`, String(Math.round(v)));
    };
    window.addEventListener('pointermove', move);
    window.addEventListener('pointerup', up);
  };
  const resetWidth = (side: 'left' | 'right') => {
    const shell = shellRef.current;
    if (!shell) return;
    const isLeft = side === 'left';
    shell.style.removeProperty(isLeft ? '--shell-sb-w-user' : '--shell-rail-w-user');
    localStorage.removeItem(`bodhi.${key}.${isLeft ? 'sideW' : 'railW'}`);
  };

  const toggleSidebar = () => {
    setOpenPop(null);
    if (isMobile) setSbOpen((o) => !o);
    else setCollapsed((c) => !c);
  };
  const toggleRail = () => {
    if (isMobile) setRailOpen((o) => !o);
    else setRailCollapsed((c) => !c);
  };
  const ctx: ShellContextValue = {
    collapsed: effCollapsed,
    isMobile,
    openPop,
    setOpenPop,
    openRail: () => {
      setRailCollapsed(false);
      setRailOpen(true);
    },
    closeRail: () => setRailOpen(false),
    collapseRail: () => {
      setRailCollapsed(true);
      setRailOpen(false);
    },
  };

  const shellClass = [
    'shell',
    effCollapsed ? 'sb-collapsed' : '',
    railCollapsed && !isMobile ? 'rail-collapsed' : '',
    !hasRail ? 'no-rail' : '',
    sbOpen ? 'sb-open' : '',
    railOpen ? 'rail-open' : '',
    dragging ? 'is-dragging' : '',
  ]
    .filter(Boolean)
    .join(' ');

  const shellStyle = {
    '--sb-w-cfg': sidebarWidth + 'px',
    '--rail-w-cfg': railWidth + 'px',
    '--header-h-cfg': headerHeight + 'px',
    '--band-h-cfg': bandHeight + 'px',
  } as CSSProperties;

  return (
    <ShellContext.Provider value={ctx}>
      <div className={shellClass} style={shellStyle} ref={shellRef}>
        {/* ══ SIDEBAR ══ */}
        <aside className={'shell-col shell-sidebar' + (effCollapsed ? ' is-collapsed' : '')}>
          <div className="shell-headrow shell-brand">{brand || <ShellBrand collapsed={effCollapsed} />}</div>
          {hasBand && !effCollapsed && <div className="shell-bandrow shell-sb-band">{sidebarToolbar}</div>}
          {effCollapsed ? (
            <div className="shell-iconrail">
              <ShellNav section={section} subPage={subPage} />
              {sidebar && (
                <>
                  <div className="shell-iconrail-div" />
                  {sidebar}
                </>
              )}
            </div>
          ) : (
            <div className="shell-body shell-nav-body">
              <ShellNav section={section} subPage={subPage} />
              {sidebar && (
                <>
                  <div className="shell-nav-div" />
                  {sidebar}
                </>
              )}
            </div>
          )}
          <div className="shell-foot">{footer || <ShellFooter user={user} collapsed={effCollapsed} />}</div>
        </aside>

        {/* ══ MAIN ══ */}
        <main className="shell-col shell-main">
          <div className="shell-headrow shell-header">
            <button
              className="shell-icon-btn shell-sb-toggle"
              onClick={toggleSidebar}
              title={isMobile ? 'Open menu' : 'Collapse sidebar'}
            >
              <ShellIcon name="panel-left" size={16} />
            </button>
            <ShellBreadcrumb items={breadcrumb} />
            <div className="shell-head-actions">
              {headerActions}
              {hasRail && (
                <button className="shell-icon-btn shell-rail-toggle" onClick={toggleRail} title="Toggle detail panel">
                  <ShellIcon name="panel-right" size={16} />
                </button>
              )}
            </div>
          </div>

          {banner && <div className="shell-mainband">{banner}</div>}
          {hasBand && <div className="shell-bandrow shell-toolbar">{toolbar}</div>}

          <div className={'shell-body' + (mainScroll ? '' : ' is-fill')}>
            <div className={'shell-content ' + contentClass}>{children}</div>
          </div>
        </main>

        {/* ══ RAIL ══ */}
        {hasRail && (
          <aside className="shell-col shell-rail">
            {railHeader !== undefined && (
              <div className="shell-headrow" style={{ padding: '0 8px 0 14px' }}>
                {railHeader}
              </div>
            )}
            {hasBand && (
              <div className="shell-bandrow" style={{ padding: '0 14px' }}>
                {railToolbar}
              </div>
            )}
            <div className={'shell-body' + (railScroll ? '' : ' is-fill')}>{rail}</div>
          </aside>
        )}

        {/* ══ RESIZE HANDLES (hover-reveal) ══ */}
        {!isMobile && (
          <div
            className="shell-resize left"
            style={{ left: 'var(--shell-sb-track)' }}
            onPointerDown={(e) => startDrag('left', e)}
            onDoubleClick={() => resetWidth('left')}
          >
            <div className="shell-resize-grip" />
          </div>
        )}
        {!isMobile && hasRail && !railCollapsed && (
          <div
            className="shell-resize right"
            style={{ left: 'calc(100% - var(--shell-rail-track))', transform: 'translateX(-50%)' }}
            onPointerDown={(e) => startDrag('right', e)}
            onDoubleClick={() => resetWidth('right')}
          >
            <div className="shell-resize-grip" />
          </div>
        )}

        {/* ══ TOOLTIP + DRAWER SCRIM ══ */}
        <GlobalTooltip />
        <div
          className="shell-scrim"
          onClick={() => {
            setSbOpen(false);
            setRailOpen(false);
          }}
        />
      </div>
    </ShellContext.Provider>
  );
}
