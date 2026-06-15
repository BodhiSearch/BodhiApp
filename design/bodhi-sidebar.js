/* ═══════════════════════════════════════════
   Bodhi Sidebar — Shared Vanilla JS Component
   bodhi-sidebar.js

   Usage:
     initBodhiSidebar(containerEl, {
       section: 'models',        // 'chat'|'models'|'mcp'|'api-keys'|'settings'
       subPage: 'my-models',     // sub-page id (optional)
       user: { initials, name, role }
     });

   Inserts: logo · nav dropdown · sub-pages · divider
   into the beginning of containerEl, then appends footer.
   Existing children (e.g. .side-scroll) stay in place.
═══════════════════════════════════════════ */

(function (global) {
  'use strict';

  /* ─── Navigation config ─────────────────────────────────── */
  const BODHI_NAV = [
    {
      id: 'chat', label: 'Chat', icon: 'message-circle',
      href: 'Bodhi Chat.html', subPages: [],
    },
    {
      id: 'models', label: 'Models', icon: 'cpu',
      href: 'Bodhi Models.html', badge: '14',
      subPages: [
        { id: 'new-local-model',    label: 'New Local Model',    icon: 'plus-circle', href: 'Create New Local Model v4.html' },
        { id: 'new-api-model',      label: 'New API Model',      icon: 'plug-zap',    href: 'Create API Model.html' },
        { id: 'new-fallback-model', label: 'New Fallback Alias', icon: 'route',       href: 'Create Fallback Model.html' },
      ],
    },
    {
      id: 'mcp', label: 'MCP', icon: 'plug',
      href: 'Bodhi MCP Discover v2.html', badge: '3',
      subPages: [
        { id: 'discover',         label: 'Discover',         icon: 'compass',     href: 'Bodhi MCP Discover v2.html' },
        { id: 'my-instances',     label: 'My Instances',     icon: 'server',      href: '#', badge: '3' },
        { id: 'new-mcp-server',   label: 'New MCP Server',   icon: 'plus-circle', href: '#' },
        { id: 'new-mcp-instance', label: 'New MCP Instance', icon: 'plus',        href: '#' },
      ],
    },
    { id: 'api-keys', label: 'API Keys', icon: 'key-round', href: '#', subPages: [] },
    { id: 'settings', label: 'Settings', icon: 'settings',  href: '#', subPages: [] },
  ];

  /* ─── CSS (injected once) ────────────────────────────────── */
  let stylesInjected = false;
  function injectStyles() {
    if (stylesInjected) return;
    stylesInjected = true;
    const s = document.createElement('style');
    s.textContent = `
/* ── BSB Logo ── */
.bsb-logo {
  display: flex; align-items: center; gap: 10px;
  padding: 13px 12px 12px;
  border-bottom: 1px solid hsl(var(--border));
  flex-shrink: 0;
}
.bsb-logo img { width: 28px; height: 28px; flex: none; }
.bsb-logo-t { font-size: 15px; font-weight: 700; letter-spacing: -.01em; line-height: 1; }
.bsb-logo-s {
  font-family: var(--font-mono); font-size: 9px; letter-spacing: .14em;
  color: hsl(var(--muted-foreground)); text-transform: uppercase; margin-top: 2px;
}

/* ── BSB Nav dropdown ── */
.bsb-nav { position: relative; padding: 10px 8px 4px; flex-shrink: 0; }
.bsb-trigger {
  width: 100%; display: flex; align-items: center; gap: 9px;
  padding: 9px 11px; border-radius: 10px;
  background: hsl(var(--muted)); border: 1px solid hsl(var(--border));
  cursor: pointer; font-size: 13px; font-weight: 600;
  color: hsl(var(--foreground)); font-family: inherit;
}
.bsb-trigger-ico { display: flex; align-items: center; flex: none; color: #DB456C; }
.bsb-trigger-ico svg { width: 15px; height: 15px; }
.bsb-trigger-lbl { flex: 1; text-align: left; }
.bsb-trigger-chev {
  display: flex; align-items: center; flex: none;
  color: hsl(var(--muted-foreground)); transition: transform 150ms;
}
.bsb-trigger-chev svg { width: 14px; height: 14px; }
.bsb-nav.open .bsb-trigger-chev { transform: rotate(180deg); }
.bsb-menu {
  display: none; position: absolute;
  top: calc(100% - 6px); left: 8px; right: 8px;
  background: hsl(var(--card)); border: 1px solid hsl(var(--border));
  border-radius: 10px; padding: 4px;
  box-shadow: 0 8px 28px rgba(0,0,0,.12); z-index: 300;
}
.bsb-nav.open .bsb-menu { display: block; }
.bsb-menu-item {
  display: flex; align-items: center; gap: 9px; padding: 8px 10px;
  border-radius: 7px; font-size: 13px; color: hsl(var(--muted-foreground));
  text-decoration: none; background: none; border: none;
  width: 100%; text-align: left; cursor: pointer; font-family: inherit;
  box-sizing: border-box;
}
.bsb-menu-item svg { width: 14px; height: 14px; flex: none; }
.bsb-menu-item:hover { background: hsl(var(--muted)); color: hsl(var(--foreground)); }
.bsb-menu-item.on { background: rgba(255,164,184,.18); color: #B02A52; font-weight: 600; }
.bsb-menu-item.on svg { color: #DB456C; }
.bsb-menu-badge {
  margin-left: auto; font-size: 10px; font-weight: 600;
  padding: 1px 6px; border-radius: 99px;
  background: rgba(255,164,184,.2); color: #B02A52;
}

/* ── BSB Sub-pages ── */
.bsb-sub { padding: 4px 8px 0; flex-shrink: 0; }
.bsb-sub-item {
  display: flex; align-items: center; gap: 8px;
  padding: 6px 11px; border-radius: 8px;
  font-size: 13px; font-weight: 500;
  color: hsl(var(--muted-foreground));
  text-decoration: none; background: none; border: none;
  width: 100%; text-align: left; cursor: pointer; font-family: inherit;
  transition: background 100ms, color 100ms; box-sizing: border-box;
}
.bsb-sub-item svg { width: 13px; height: 13px; flex: none; }
.bsb-sub-item:hover { background: hsl(var(--muted)); color: hsl(var(--foreground)); }
.bsb-sub-item.on { background: rgba(255,164,184,.14); color: #B02A52; font-weight: 600; }
.bsb-sub-item.on svg { color: #DB456C; }
.bsb-sub-badge {
  margin-left: auto; font-size: 10px; font-weight: 600;
  padding: 1px 6px; border-radius: 99px;
  background: rgba(255,164,184,.2); color: #B02A52;
}

/* ── BSB Divider ── */
.bsb-divider { height: 1px; background: hsl(var(--border)); margin: 8px 10px 4px; flex-shrink: 0; }

/* ── BSB Footer ── */
.bsb-foot {
  border-top: 1px solid hsl(var(--border));
  padding: 10px 12px;
  display: flex; align-items: center; gap: 9px; flex-shrink: 0;
}
.bsb-avatar {
  width: 30px; height: 30px; border-radius: 50%;
  background: #3E4AA8; color: #fff;
  display: flex; align-items: center; justify-content: center;
  font-size: 11px; font-weight: 700; flex: none;
}
.bsb-logout {
  margin-left: auto; width: 26px; height: 26px; border-radius: 6px;
  display: flex; align-items: center; justify-content: center;
  background: none; border: none; color: hsl(var(--muted-foreground)); cursor: pointer;
}
.bsb-logout:hover { background: hsl(var(--muted)); }
.bsb-logout svg { width: 14px; height: 14px; }
    `;
    document.head.appendChild(s);
  }

  /* ─── Helpers ────────────────────────────────────────────── */
  function icon(name) {
    const i = document.createElement('i');
    i.setAttribute('data-lucide', name);
    return i;
  }

  function insertAfter(newEl, ref) {
    ref.parentNode.insertBefore(newEl, ref.nextSibling);
  }

  /* ─── Main function ───────────────────────────────────────── */
  function initBodhiSidebar(container, opts) {
    opts = opts || {};
    const section = opts.section || 'chat';
    const subPage = opts.subPage || null;
    const uo = opts.user || {};
    const u = {
      initials: uo.initials || 'YO',
      name:     uo.name     || 'Yogesh',
      role:     uo.role     || 'Admin',
    };

    injectStyles();

    const cur = BODHI_NAV.find(n => n.id === section) || BODHI_NAV[0];

    /* ── Logo ──────────────────────────── */
    const logo = document.createElement('div');
    logo.className = 'bsb-logo';
    const logoImg = document.createElement('img');
    logoImg.src = 'assets/bodhi-logo-60.svg';
    logoImg.alt = 'Bodhi';
    const logoTxt = document.createElement('div');
    logoTxt.innerHTML = '<div class="bsb-logo-t">Bodhi</div><div class="bsb-logo-s">AI Gateway</div>';
    logo.append(logoImg, logoTxt);
    container.insertBefore(logo, container.firstChild);

    /* ── Nav dropdown ──────────────────── */
    const nav = document.createElement('div');
    nav.className = 'bsb-nav';

    const trigger = document.createElement('button');
    trigger.className = 'bsb-trigger';
    const tIco = document.createElement('span');
    tIco.className = 'bsb-trigger-ico';
    tIco.appendChild(icon(cur.icon));
    const tLbl = document.createElement('span');
    tLbl.className = 'bsb-trigger-lbl';
    tLbl.textContent = cur.label;
    const tChev = document.createElement('span');
    tChev.className = 'bsb-trigger-chev';
    tChev.appendChild(icon('chevron-down'));
    trigger.append(tIco, tLbl, tChev);

    const menu = document.createElement('div');
    menu.className = 'bsb-menu';
    BODHI_NAV.forEach(item => {
      const a = document.createElement('a');
      a.className = 'bsb-menu-item' + (item.id === section ? ' on' : '');
      a.href = item.href || '#';
      const iw = document.createElement('span');
      iw.style.cssText = 'display:flex;align-items:center;flex:none;';
      iw.appendChild(icon(item.icon));
      a.append(iw, document.createTextNode(item.label));
      if (item.badge) {
        const b = document.createElement('span');
        b.className = 'bsb-menu-badge';
        b.textContent = item.badge;
        a.appendChild(b);
      }
      menu.appendChild(a);
    });

    nav.append(trigger, menu);
    insertAfter(nav, logo);

    trigger.addEventListener('click', e => { e.stopPropagation(); nav.classList.toggle('open'); });
    menu.addEventListener('click', e => e.stopPropagation());
    document.addEventListener('click', () => nav.classList.remove('open'));

    /* ── Sub-pages ─────────────────────── */
    if (cur.subPages && cur.subPages.length) {
      const sub = document.createElement('div');
      sub.className = 'bsb-sub';
      cur.subPages.forEach(sp => {
        const a = document.createElement('a');
        a.className = 'bsb-sub-item' + (sp.id === subPage ? ' on' : '');
        a.href = sp.href || '#';
        const iw = document.createElement('span');
        iw.style.cssText = 'display:flex;align-items:center;flex:none;';
        iw.appendChild(icon(sp.icon || 'circle'));
        a.append(iw, document.createTextNode(sp.label));
        if (sp.badge) {
          const b = document.createElement('span');
          b.className = 'bsb-sub-badge';
          b.textContent = sp.badge;
          a.appendChild(b);
        }
        sub.appendChild(a);
      });
      insertAfter(sub, nav);

      const div = document.createElement('div');
      div.className = 'bsb-divider';
      insertAfter(div, sub);
    }

    /* ── Footer ─────────────────────────── */
    const foot = document.createElement('div');
    foot.className = 'bsb-foot';
    const av = document.createElement('div');
    av.className = 'bsb-avatar';
    av.textContent = u.initials;
    const uInfo = document.createElement('div');
    uInfo.style.minWidth = '0';
    uInfo.innerHTML =
      `<div style="font-size:13px;font-weight:500">${u.name}</div>` +
      `<div style="font-size:11px;color:hsl(var(--muted-foreground))">${u.role}</div>`;
    const logout = document.createElement('button');
    logout.className = 'bsb-logout';
    logout.title = 'Log out';
    logout.appendChild(icon('log-out'));
    foot.append(av, uInfo, logout);
    container.appendChild(foot);

    /* ── Init lucide ───────────────────── */
    if (typeof lucide !== 'undefined') lucide.createIcons();
  }

  global.initBodhiSidebar = initBodhiSidebar;
  global.BODHI_NAV_CONFIG  = BODHI_NAV;

}(window));
