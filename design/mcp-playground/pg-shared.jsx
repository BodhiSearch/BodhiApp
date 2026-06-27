/* ═══════════════════════════════════════════════════════════════
   BODHI MCP — PLAYGROUND PRIMITIVES + CONTEXTS
   mcp-playground/pg-shared.jsx   (load AFTER pg-data.jsx, BEFORE pg-render)

   The small primitives every playground page reuses: React hook aliases
   (declared ONCE for the page), JSON highlighter, the navigation context
   (resource-link jumps), the server glyph and status dot. The instance picker and capability
   nav live in pg-chrome.jsx (they drive cross-page navigation now).
   Published to window.
═══════════════════════════════════════════════════════════════ */
const { useState, useEffect, useRef, useMemo } = React;

/* ── tiny primitives ─────────────────────────────────────────── */
function PgSpinner({ size = 14 }) {
  return <span className="pg-spin" style={{ width: size, height: size }} />;
}
function prettyKey(k) {
  return String(k)
    .replace(/[_-]+/g, ' ')
    .replace(/([a-z0-9])([A-Z])/g, '$1 $2')
    .replace(/^./, c => c.toUpperCase());
}
function escapeHtml(str) {
  return String(str).replace(/&/g, '&amp;').replace(/</g, '&lt;').replace(/>/g, '&gt;');
}
function syntaxHighlight(json) {
  if (typeof json !== 'string') json = JSON.stringify(json, null, 2);
  return escapeHtml(json).replace(
    /("(\\u[a-zA-Z0-9]{4}|\\[^u]|[^\\"])*"(\s*:)?|\b(true|false|null)\b|-?\d+(?:\.\d*)?(?:[eE][+\-]?\d+)?)/g,
    m => {
      if (/^"/.test(m)) return /:$/.test(m) ? `<span class="json-key">${m}</span>` : `<span class="json-str">${m}</span>`;
      if (/true|false/.test(m)) return `<span class="json-bool">${m}</span>`;
      if (/null/.test(m)) return `<span class="json-null">${m}</span>`;
      return `<span class="json-num">${m}</span>`;
    }
  );
}

/* ── instance ↔ URL helpers (instance carried across pages) ──── */
function instQS(inst) {
  return new URLSearchParams({ instance: inst.instId, name: inst.instName, server: inst.serverId }).toString();
}
function urlParams() { return new URLSearchParams(window.location.search); }
function resolveURLInstance() {
  const p = urlParams();
  const instId = p.get('instance'), serverId = p.get('server');
  if (!instId && !serverId) return null;
  return (typeof findInstance === 'function') ? findInstance(instId, serverId) : null;
}

/* ── Navigation context — a readable result can jump to another page
   (resource links open the Resources page). ── */
const PgNavContext = React.createContext({ openResource: () => {} });
const usePgNav = () => React.useContext(PgNavContext);

/* ── server glyph (icon tile) ────────────────────────────────── */
function ServerGlyph({ s, size = 30, radius = 8 }) {
  if (!s) return <div className="pg-glyph" style={{ width: size, height: size, borderRadius: radius, background: 'hsl(var(--muted))' }} />;
  return (
    <div className="pg-glyph" style={{ width: size, height: size, borderRadius: radius, background: s.iconBg, color: s.iconColor, fontSize: size * 0.46 }}>
      {s.icon}
    </div>
  );
}

/* ── status dot for an instance ──────────────────────────────── */
function StatusDot({ status }) {
  const map = {
    connected: { c: 'var(--c-connected-text)', t: 'Connected' },
    pending:   { c: 'var(--c-pending-text)',   t: 'Authorizing' },
  };
  const m = map[status] || { c: 'hsl(var(--muted-foreground))', t: status || '' };
  return <span className="pg-dot" style={{ background: m.c }} title={m.t} />;
}

Object.assign(window, {
  useState, useEffect, useRef, useMemo,
  PgSpinner, prettyKey, escapeHtml, syntaxHighlight,
  instQS, urlParams, resolveURLInstance,
  PgNavContext, usePgNav, ServerGlyph, StatusDot,
});
