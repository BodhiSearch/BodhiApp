/* ═══════════════════════════════════════════════════
   bodhi-select.js — app-wide themed <select> dropdown

   Native <select> popups are drawn by the OS and can't be themed.
   This progressively enhances EVERY <select> on the page: the closed
   control stays the real native element (so value / focus / forms /
   React state all keep working untouched) — we only suppress the OS
   popup and render a themed listbox overlay in its place.

   Opt out per element with  data-native  (or <select multiple> / size>1).
   Works for static HTML and React-mounted selects (MutationObserver).
═══════════════════════════════════════════════════ */
(function () {
  if (window.__bodhiSelectInit) return;
  window.__bodhiSelectInit = true;

  /* ── styles ─────────────────────────────────────── */
  const css = `
  .bsel-pop {
    position: fixed; z-index: 9999;
    box-sizing: border-box;
    background: hsl(var(--popover));
    color: hsl(var(--popover-foreground));
    border: 1px solid hsl(var(--border-strong));
    border-radius: var(--radius-md);
    box-shadow: var(--shadow-lg);
    padding: 5px;
    max-height: 300px; overflow-y: auto;
    font-family: var(--font-sans, inherit);
    animation: bselIn .12s ease;
    scrollbar-width: thin;
  }
  @keyframes bselIn {
    from { transform: translateY(-4px) scale(.99); }
    to   { transform: none; }
  }
  .bsel-opt {
    display: flex; align-items: center; gap: 8px;
    padding: 7px 9px 7px 10px;
    border-radius: var(--radius-sm, 6px);
    font-size: 13.5px; line-height: 1.3;
    color: hsl(var(--popover-foreground));
    cursor: pointer; user-select: none;
    white-space: nowrap;
  }
  .bsel-opt-label { flex: 1; min-width: 0; overflow: hidden; text-overflow: ellipsis; }
  .bsel-opt-check {
    width: 15px; height: 15px; flex-shrink: 0;
    display: inline-flex; align-items: center; justify-content: center;
    color: hsl(var(--primary-hover));
    opacity: 0;
  }
  .bsel-opt[aria-selected="true"] .bsel-opt-check { opacity: 1; }
  .bsel-opt[aria-selected="true"] { color: hsl(var(--foreground)); font-weight: 600; }
  .bsel-opt.active { background: hsl(var(--muted)); }
  .bsel-opt[aria-selected="true"].active { background: hsl(var(--primary) / .14); }
  .bsel-opt[aria-disabled="true"] { color: hsl(var(--faint-foreground)); cursor: default; opacity: .6; }
  .bsel-opt[aria-disabled="true"].active { background: transparent; }
  .bsel-group-label {
    padding: 8px 9px 4px; font-size: 11px; font-weight: 600;
    letter-spacing: .04em; text-transform: uppercase;
    color: hsl(var(--muted-foreground));
  }
  .bsel-host-open { box-shadow: var(--shadow-glow); border-color: hsl(var(--ring)) !important; }
  .bsel-pop::-webkit-scrollbar { width: 9px; }
  .bsel-pop::-webkit-scrollbar-thumb { background: hsl(var(--border-strong)); border-radius: 9px; border: 2px solid hsl(var(--popover)); }
  `;
  const style = document.createElement('style');
  style.id = 'bodhi-select-styles';
  style.textContent = css;
  document.head.appendChild(style);

  const CHECK = '<svg viewBox="0 0 24 24" width="15" height="15" fill="none" stroke="currentColor" stroke-width="3" stroke-linecap="round" stroke-linejoin="round"><polyline points="20 6 9 17 4 12"></polyline></svg>';

  /* native value setter so React's value-tracker notices the change */
  const nativeValueSet = Object.getOwnPropertyDescriptor(window.HTMLSelectElement.prototype, 'value').set;

  const enhanced = new WeakSet();
  let pop = null;          // the floating listbox element
  let host = null;         // the <select> currently open
  let optEls = [];         // rendered .bsel-opt nodes (option entries only)
  let activeIdx = -1;
  let typeBuffer = '';
  let typeTimer = null;

  function shouldEnhance(sel) {
    return sel && sel.tagName === 'SELECT' && !sel.multiple && sel.size <= 1 && !sel.hasAttribute('data-native');
  }

  /* ── build / position the overlay ───────────────── */
  function buildPop(sel) {
    pop = document.createElement('div');
    pop.className = 'bsel-pop';
    pop.setAttribute('role', 'listbox');
    optEls = [];
    const children = Array.from(sel.children);
    const selectedIdx = sel.selectedIndex;

    children.forEach((node) => {
      if (node.tagName === 'OPTGROUP') {
        const gl = document.createElement('div');
        gl.className = 'bsel-group-label';
        gl.textContent = node.label || '';
        pop.appendChild(gl);
        Array.from(node.children).forEach((o) => addOpt(o, sel, selectedIdx));
      } else if (node.tagName === 'OPTION') {
        addOpt(node, sel, selectedIdx);
      }
    });
  }

  function addOpt(option, sel, selectedIdx) {
    const el = document.createElement('div');
    el.className = 'bsel-opt';
    el.setAttribute('role', 'option');
    const isSel = option.index === selectedIdx;
    el.setAttribute('aria-selected', isSel ? 'true' : 'false');
    if (option.disabled) el.setAttribute('aria-disabled', 'true');
    el.dataset.value = option.value;
    el.dataset.idx = String(option.index);
    el.innerHTML = '<span class="bsel-opt-check">' + CHECK + '</span><span class="bsel-opt-label"></span>';
    el.querySelector('.bsel-opt-label').textContent = option.textContent;
    if (!option.disabled) {
      el.addEventListener('click', () => commit(option.index));
      el.addEventListener('mousemove', () => setActive(optEls.indexOf(el)));
    }
    pop.appendChild(el);
    optEls.push(el);
    return el;
  }

  function position() {
    if (!pop || !host) return;
    const r = host.getBoundingClientRect();
    pop.style.minWidth = r.width + 'px';
    pop.style.left = r.left + 'px';
    // measure
    pop.style.top = '-9999px';
    pop.style.maxHeight = '300px';
    const ph = pop.offsetHeight;
    const below = window.innerHeight - r.bottom - 8;
    const above = r.top - 8;
    if (below >= ph || below >= above) {
      pop.style.top = (r.bottom + 4) + 'px';
      pop.style.maxHeight = Math.min(300, below) + 'px';
    } else {
      pop.style.maxHeight = Math.min(300, above) + 'px';
      pop.style.top = (r.top - Math.min(ph, above) - 4) + 'px';
    }
    // keep within right edge
    const rightOverflow = r.left + pop.offsetWidth - (window.innerWidth - 8);
    if (rightOverflow > 0) pop.style.left = Math.max(8, r.left - rightOverflow) + 'px';
  }

  /* ── open / close ───────────────────────────────── */
  function open(sel) {
    if (host === sel) return;
    close();
    if (sel.disabled) return;
    host = sel;
    buildPop(sel);
    document.body.appendChild(pop);
    host.classList.add('bsel-host-open');
    position();
    activeIdx = optEls.findIndex((e) => e.getAttribute('aria-selected') === 'true');
    if (activeIdx < 0) activeIdx = optEls.findIndex((e) => e.getAttribute('aria-disabled') !== 'true');
    setActive(activeIdx, true);
    window.addEventListener('scroll', onScroll, true);
    window.addEventListener('resize', close, true);
  }

  function close() {
    if (!pop) return;
    window.removeEventListener('scroll', onScroll, true);
    window.removeEventListener('resize', close, true);
    if (host) host.classList.remove('bsel-host-open');
    pop.remove();
    pop = null; host = null; optEls = []; activeIdx = -1;
  }

  function onScroll(e) {
    // reposition while scrolling the page; close if the host scrolls out
    if (pop && pop.contains(e.target)) return;
    position();
  }

  function setActive(idx, scroll) {
    if (idx < 0 || idx >= optEls.length) return;
    optEls.forEach((e) => e.classList.remove('active'));
    const el = optEls[idx];
    if (!el) return;
    el.classList.add('active');
    activeIdx = idx;
    if (scroll) el.scrollIntoView({ block: 'nearest' });
  }

  function moveActive(dir) {
    let i = activeIdx;
    for (let n = 0; n < optEls.length; n++) {
      i = (i + dir + optEls.length) % optEls.length;
      if (optEls[i].getAttribute('aria-disabled') !== 'true') { setActive(i, true); return; }
    }
  }

  function commit(optionIdx) {
    const sel = host;
    if (!sel) return;
    if (sel.selectedIndex !== optionIdx) {
      nativeValueSet.call(sel, sel.options[optionIdx].value);
      sel.dispatchEvent(new Event('input', { bubbles: true }));
      sel.dispatchEvent(new Event('change', { bubbles: true }));
    }
    close();
    sel.focus();
  }

  function typeAhead(ch) {
    clearTimeout(typeTimer);
    typeBuffer += ch.toLowerCase();
    typeTimer = setTimeout(() => { typeBuffer = ''; }, 600);
    const match = optEls.findIndex((e) =>
      e.getAttribute('aria-disabled') !== 'true' &&
      e.textContent.trim().toLowerCase().startsWith(typeBuffer));
    if (match >= 0) setActive(match, true);
  }

  /* ── input interception on the native select ────── */
  function onHostMouseDown(e) {
    const sel = e.currentTarget;
    if (!shouldEnhance(sel) || sel.disabled) return;
    e.preventDefault();              // suppress OS popup
    if (host === sel) { close(); } else { sel.focus(); open(sel); }
  }

  function onHostKeyDown(e) {
    const sel = e.currentTarget;
    if (!shouldEnhance(sel) || sel.disabled) return;
    if (host === sel) {
      // overlay is open — drive it
      switch (e.key) {
        case 'ArrowDown': e.preventDefault(); moveActive(1); break;
        case 'ArrowUp':   e.preventDefault(); moveActive(-1); break;
        case 'Home':      e.preventDefault(); setActive(0, true); if (optEls[0] && optEls[0].getAttribute('aria-disabled') === 'true') moveActive(1); break;
        case 'End':       e.preventDefault(); setActive(optEls.length - 1, true); if (optEls[activeIdx] && optEls[activeIdx].getAttribute('aria-disabled') === 'true') moveActive(-1); break;
        case 'Enter':
        case ' ':         e.preventDefault(); if (activeIdx >= 0) commit(Number(optEls[activeIdx].dataset.idx)); break;
        case 'Escape':    e.preventDefault(); close(); sel.focus(); break;
        case 'Tab':       close(); break;
        default:          if (e.key.length === 1) { e.preventDefault(); typeAhead(e.key); }
      }
    } else {
      // closed — open on the usual keys
      if (['ArrowDown', 'ArrowUp', 'Enter', ' '].includes(e.key)) {
        e.preventDefault(); open(sel);
      }
    }
  }

  function enhance(sel) {
    if (!shouldEnhance(sel) || enhanced.has(sel)) return;
    enhanced.add(sel);
    sel.addEventListener('mousedown', onHostMouseDown);
    sel.addEventListener('keydown', onHostKeyDown);
    sel.addEventListener('blur', () => { /* close handled by outside pointer */ });
  }

  function scan(root) {
    (root.querySelectorAll ? root.querySelectorAll('select') : []).forEach(enhance);
  }

  /* close on outside interaction */
  document.addEventListener('pointerdown', (e) => {
    if (!pop) return;
    if (pop.contains(e.target) || (host && host.contains(e.target))) return;
    close();
  }, true);

  /* enhance existing + observe for React-mounted selects */
  function start() {
    scan(document);
    const mo = new MutationObserver((muts) => {
      for (const m of muts) {
        m.addedNodes.forEach((n) => {
          if (n.nodeType !== 1) return;
          if (n.tagName === 'SELECT') enhance(n);
          else scan(n);
        });
      }
    });
    mo.observe(document.body, { childList: true, subtree: true });
  }

  if (document.readyState === 'loading') {
    document.addEventListener('DOMContentLoaded', start);
  } else {
    start();
  }
})();
