const e = 'complete',
  t = '*',
  n = 'Invalid message format: missing requestId or request',
  a = 'BODHI_API_REQUEST',
  o = 'BODHI_STREAM_REQUEST',
  r = 'BODHI_STREAM_ERROR',
  s = 'BODHI_GET_EXTENSION_ID';
const i = (e) => ({
    info: (e, t) => {},
    error: (t, n) => {
      const a = (function (e) {
        if ('function' == typeof e) {
          const t = e();
          return Array.isArray(t) ? t : 'object' == typeof t ? [JSON.stringify(t)] : [t];
        }
        return Array.isArray(e)
          ? e
          : e && 'object' == typeof e
            ? [JSON.stringify(e)]
            : void 0 !== e
              ? [e]
              : [];
      })(n);
      console.error(`[Bodhi/${e}] ${t}`, ...a);
    },
  }),
  d = i('shared'),
  c = (e, t = 'unknown', n = {}) => {
    const a = 'string' == typeof e ? e : e.message,
      o = 'string' == typeof e ? new Error(e) : e;
    return (
      Object.assign(o, { context: n, location: t }),
      d.error('Error', { tag: t, errorMessage: a, context: n, stack: o.stack, error: o }),
      o
    );
  },
  p = (e, t, n = !1) => ({
    type: n ? r : 'BODHI_API_RESPONSE',
    requestId: t,
    response: { body: { error: { message: e } }, status: 0, headers: {} },
  }),
  g = i('content.js'),
  y = new Map(),
  u = () => {
    try {
      if (document.readyState !== e)
        return void document.addEventListener('readystatechange', () => {
          document.readyState === e && u();
        });
      const t = document.createElement('script');
      (t.src = chrome.runtime.getURL('inject.js')),
        (t.onload = () => {
          t.remove();
        }),
        (document.head || document.documentElement).appendChild(t),
        g.info('Interface script injected');
    } catch (e) {
      c(e, 'content.js:injectScript');
    }
  };
function m() {
  document.readyState === e
    ? (window.addEventListener('message', (e) => {
        var i;
        e.source === window &&
          ((i = e.data),
          i?.type === a
            ? ((e) => {
                if (e.data && e.data.type && e.data.type === a) {
                  g.info('Received api request from page', { type: e.data.type });
                  try {
                    if (!e.data.requestId || !e.data.request) throw new Error(n);
                    chrome.runtime.sendMessage(e.data, (n) => {
                      window.postMessage(n, e.origin || t);
                    });
                  } catch (n) {
                    const a = c(n, 'content.js:handleApiMessage');
                    e.data &&
                      e.data.requestId &&
                      window.postMessage(p(a.message, e.data.requestId), e.origin || t);
                  }
                }
              })(e)
            : (function (e) {
                  return e?.type === o;
                })(e.data)
              ? ((e) => {
                  if (e.data && e.data.type && e.data.type === o) {
                    g.info('Received streaming request from page', { type: e.data.type });
                    try {
                      const a = chrome.runtime.connect({ name: 'BODHI_STREAM_PORT' }),
                        o = e.data.requestId;
                      if (!o) throw new Error(n);
                      y.set(o, a),
                        a.onMessage.addListener((n) => {
                          window.postMessage(n, e.origin || t),
                            (('BODHI_STREAM_CHUNK' === n.type && n.response.body.done) ||
                              n.type === r) &&
                              (y.delete(o), a.disconnect());
                        }),
                        a.onDisconnect.addListener(() => {
                          y.has(o) &&
                            (window.postMessage(
                              p('Connection closed unexpectedly', o, !0),
                              e.origin || t
                            ),
                            y.delete(o));
                        }),
                        a.postMessage(e.data);
                    } catch (n) {
                      const a = c(n, 'content.js:handleStreamingMessage');
                      e.data &&
                        e.data.requestId &&
                        window.postMessage(p(a.message, e.data.requestId, !0), e.origin || t);
                    }
                  }
                })(e)
              : (function (e) {
                  return e?.type === s;
                })(e.data) &&
                ((e) => {
                  e.data &&
                    e.data.type &&
                    e.data.type === s &&
                    (g.info('Received get extension ID request from page', { type: e.data.type }),
                    (() => {
                      try {
                        const e = chrome.runtime.id;
                        g.info('Sending extension ID to page', { extensionId: e }),
                          window.postMessage(
                            { type: 'BODHI_SET_EXTENSION_ID', extension_id: e },
                            window.origin || t
                          );
                      } catch (e) {
                        c(e, 'content.js:sendExtensionId');
                      }
                    })());
                })(e));
      }),
      g.info('Message listeners registered'))
    : document.addEventListener('readystatechange', () => {
        document.readyState === e && m();
      });
}
(() => {
  try {
    g.info('Content script initializing'), u(), m(), g.info('Content script initialized');
  } catch (e) {
    c(e, 'content.js:initContentScript');
  }
})();
//# sourceMappingURL=content.js.map
