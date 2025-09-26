const e = 'application/json',
  r = 'Content-Type',
  t = 'http://localhost:1135',
  s = 'backendUrl',
  n = 'data: ',
  o = 'BODHI_API_RESPONSE',
  a = 'BODHI_STREAM_CHUNK',
  d = 'BODHI_ERROR',
  i = 'BODHI_TEST_CONNECTION_RESPONSE';
const c = (e) => ({
    info: (e, r) => {},
    error: (r, t) => {
      const s = (function (e) {
        if ('function' == typeof e) {
          const r = e();
          return Array.isArray(r) ? r : 'object' == typeof r ? [JSON.stringify(r)] : [r];
        }
        return Array.isArray(e)
          ? e
          : e && 'object' == typeof e
            ? [JSON.stringify(e)]
            : void 0 !== e
              ? [e]
              : [];
      })(t);
      console.error(`[Bodhi/${e}] ${r}`, ...s);
    },
  }),
  u = c('shared'),
  p = (e, r = 'unknown', t = {}) => {
    const s = 'string' == typeof e ? e : e.message,
      n = 'string' == typeof e ? new Error(e) : e;
    return (
      Object.assign(n, { context: t, location: r }),
      u.error('Error', { tag: r, errorMessage: s, context: t, stack: n.stack, error: n }),
      n
    );
  },
  y = (t, s = {}, n = null) => {
    const o = { method: t, headers: { [r]: e, ...s } };
    return 'GET' !== t && n && (o.body = JSON.stringify(n)), o;
  },
  h = (e, r, t = !1) => ({
    type: t ? 'BODHI_STREAM_ERROR' : o,
    requestId: r,
    response: { body: { error: { message: e } }, status: 0, headers: {} },
  }),
  g = async (t) => {
    const s = (t.headers.get(r) || '').includes(e) ? await t.json() : await t.text(),
      n = {};
    return (
      t.headers.forEach((e, r) => {
        n[r] = e;
      }),
      { body: s, status: t.status, headers: n }
    );
  },
  l = (e) => {
    if (!e || '' === e.trim()) return null;
    if (e.includes('[DONE]')) return { done: !0 };
    try {
      if (!e.startsWith(n))
        return e.startsWith(':')
          ? (u.info('Received SSE keep-alive comment'), null)
          : (u.info(`Received malformed SSE chunk: ${e}`), null);
      const r = e.replace(new RegExp(`^${n}`), ''),
        t = JSON.parse(r);
      return t && 'object' == typeof t
        ? t
        : (u.error('Received unexpected SSE data structure', { jsonStr: r }), { data: r });
    } catch (r) {
      const t = r;
      return (
        u.error('Error processing SSE chunk', { chunk: e, error: t.message }),
        { data: e, error: t.message }
      );
    }
  },
  m = async (e, r, t = 15e4) => {
    try {
      const s = new AbortController(),
        n = setTimeout(() => s.abort(), t),
        o = { ...r, signal: s.signal },
        a = await fetch(e, o);
      return clearTimeout(n), a;
    } catch (r) {
      if ('AbortError' === r?.name) throw new Error(`Request timeout after ${t}ms: ${e}`);
      throw r;
    }
  };
let f = t;
const I = c('background.js'),
  q = (e, r, t, s, n = !1) => {
    m(e, r)
      .then(async (e) => {
        if (!e.ok) {
          const r = await g(e);
          return void t({ type: n ? a : o, requestId: s, response: r });
        }
        if (n)
          ((e, r, t) => {
            const s = e.body.getReader(),
              n = new TextDecoder();
            let o = '';
            const d = async () => {
              try {
                const { done: i, value: c } = await s.read();
                if (i) {
                  if (o && '' !== o.trim()) {
                    const s = l(o);
                    s &&
                      r({
                        type: a,
                        requestId: t,
                        response: { body: s, status: e.status, headers: {} },
                      }),
                      (o = '');
                  }
                  return void r({
                    type: a,
                    requestId: t,
                    response: { body: { done: !0 }, status: e.status, headers: {} },
                  });
                }
                o += n.decode(c, { stream: !0 });
                const u = o.split('\n\n');
                o = u.pop() || '';
                for (const s of u) {
                  if (!s || '' === s.trim()) continue;
                  const n = l(s);
                  if (
                    n &&
                    (r({
                      type: a,
                      requestId: t,
                      response: { body: n, status: e.status, headers: {} },
                    }),
                    n.done)
                  )
                    return;
                }
                d();
              } catch (e) {
                const n = e;
                I.error('Error processing stream chunk', { error: n }), r(h(n.message, t, !0));
                try {
                  s.cancel();
                } catch (e) {
                  I.error('Error cancelling reader', { error: e });
                }
              }
            };
            d();
          })(e, t, s);
        else {
          const r = await g(e);
          t({ type: o, requestId: s, response: r });
        }
      })
      .catch((e) =>
        ((e, r, t, s = !1) => {
          u.error('Error fetching from API', { error: e }), r(h(e.message, t, s));
        })(e, t, s, n)
      );
  };
chrome.runtime.onInstalled.addListener((e) => {
  try {
    I.info('Extension event', { reason: e.reason }),
      'install' === e.reason
        ? (chrome.tabs.create({ url: chrome.runtime.getURL('index.html') }),
          I.info('Opened extension page on first install'))
        : 'update' === e.reason &&
          I.info('Extension updated', { previousVersion: e.previousVersion });
  } catch (r) {
    p(r, 'background.js:onInstalled', { details: e });
  }
}),
  chrome.runtime.onConnect.addListener((e) => {
    'BODHI_STREAM_PORT' === e.name
      ? e.onMessage.addListener(async (r) => {
          I.info('Received onConnect request', { type: r.type });
          try {
            const t = r?.requestId || '';
            if ('BODHI_STREAM_REQUEST' !== r.type)
              return (
                I.error('Invalid message type for onConnect request', { type: r.type }),
                void e.postMessage(h('Invalid message type', t, !0))
              );
            const s = E(r);
            if (s.type === d)
              return (
                I.error('Malformed streaming request', { requestId: t }), void e.postMessage(s)
              );
            I.info('Processing onConnect request', { requestId: t });
            const { method: n, endpoint: o, body: a, headers: i = {} } = s,
              c = y(n, i, a);
            q(`${f}${o}`, c, (r) => e.postMessage(r), t, !0);
          } catch (t) {
            const s = r.data?.requestId || '';
            p(t, 'background.js:onConnect', { requestId: s, messageType: r.type }),
              e.postMessage(h(t.message, s, !0));
          }
        })
      : I.error('Invalid port name', { portName: e.name });
  }),
  chrome.runtime.onMessage.addListener((e, r, t) => {
    I.info('Received onMessage request', { senderId: r.id });
    try {
      const r = e?.requestId || '';
      if ('BODHI_TEST_CONNECTION' === e.type)
        return (
          I.info('Processing test connection request', { requestId: r }),
          ((e, r) => {
            const { testUrl: t, requestId: s } = e;
            try {
              new URL(t);
            } catch {
              return void r({
                type: i,
                requestId: s,
                response: {
                  body: {
                    status: 'error',
                    url: t,
                    error: {
                      message: `'${t}' is not a valid URL. Please enter a valid URL like http://localhost:1135`,
                      type: 'validation_error',
                    },
                  },
                  status: 0,
                  headers: {},
                },
              });
            }
            const n = `${t}/bodhi/v1/info`,
              o = y('GET', {}, null);
            m(n, o)
              .then(async (e) => {
                const n = await g(e);
                if (0 !== n.status)
                  if (200 === n.status && n.body)
                    r({
                      type: i,
                      requestId: s,
                      response: {
                        body: { status: n.body.status, version: n.body.version, url: t },
                        status: 200,
                        headers: n.headers,
                      },
                    });
                  else {
                    if (n.status >= 400) {
                      const e =
                        n.body?.error?.message || `Server returned error status ${n.status}`;
                      return void r({
                        type: i,
                        requestId: s,
                        response: {
                          body: {
                            status: 'error',
                            url: t,
                            error: { message: `Server error: ${e}`, type: 'server_error' },
                          },
                          status: n.status,
                          headers: n.headers,
                        },
                      });
                    }
                    r({
                      type: i,
                      requestId: s,
                      response: {
                        body: {
                          status: 'error',
                          url: t,
                          error: {
                            message: 'Unexpected response format from server',
                            type: 'response_error',
                          },
                        },
                        status: n.status,
                        headers: n.headers,
                      },
                    });
                  }
                else
                  r({
                    type: i,
                    requestId: s,
                    response: {
                      body: {
                        status: 'unreachable',
                        url: t,
                        error: {
                          message: `Cannot reach server at ${t}. Please check if the Bodhi app is running and the URL is correct.`,
                          type: 'network_error',
                        },
                      },
                      status: 0,
                      headers: {},
                    },
                  });
              })
              .catch((e) => {
                const n = e;
                I.error('Error testing connection', { error: n.message, testUrl: t, requestId: s });
                let o = `Cannot connect to ${t}. `;
                n.message.includes('fetch')
                  ? (o += 'Please check if the Bodhi app is running and the URL is correct.')
                  : n.message.includes('timeout')
                    ? (o += 'Connection timed out. The server may be overloaded or unreachable.')
                    : (o += `Error: ${n.message}`),
                  r({
                    type: i,
                    requestId: s,
                    response: {
                      body: {
                        status: 'unreachable',
                        url: t,
                        error: { message: o, type: 'network_error' },
                      },
                      status: 0,
                      headers: {},
                    },
                  });
              });
          })(e, t),
          !0
        );
      if ('BODHI_API_REQUEST' !== e.type)
        return (
          I.error('Invalid message type for onMessage request', { type: e.type, requestId: r }),
          void t(h('Invalid message type', r))
        );
      const s = E(e);
      if (s.type === d)
        return I.error('Malformed request in onMessage', { requestId: r }), void t(s);
      const { method: n, endpoint: a, body: c, headers: u = {} } = s;
      I.info('Processing onMessage request', { method: n, endpoint: a, requestId: r });
      const p = y(n, u, c);
      return (
        m(`${f}${a}`, p)
          .then(async (e) => {
            const s = await g(e);
            t({ type: o, requestId: r, response: s });
          })
          .catch((e) => {
            const s = e;
            I.error('Error fetching from API', { error: s.message, requestId: r }),
              t(h(s.message, r));
          }),
        !0
      );
    } catch (r) {
      const s = e?.requestId || '',
        n = r;
      I.error('Error handling onMessage request', { error: n.message, requestId: s }),
        t(h(n.message, s));
    }
  });
const E = (e) => {
  const { type: r, requestId: t, request: s } = e,
    n = s?.method,
    o = s?.endpoint;
  let a = '';
  return (
    r
      ? t
        ? s
          ? n
            ? o || (a = 'endpoint')
            : (a = 'method')
          : (a = 'request')
        : (a = 'requestId')
      : (a = 'type'),
    a
      ? {
          type: d,
          requestId: t || '',
          response: {
            status: 0,
            headers: {},
            body: { error: { message: `Malformed request: missing ${a}` } },
          },
        }
      : {
          endpoint: o.startsWith('/') ? o : `/${o}`,
          method: n,
          body: s.body,
          headers: s.headers || {},
        }
  );
};
chrome.runtime.onMessageExternal.addListener((e, r, t) => {
  I.info('Received onMessageExternal request', { senderId: r.id });
  try {
    const s = E(e),
      n = e.requestId;
    if (s.type === d)
      return I.info('Error response onMessageExternal', { requestId: n }), void t(s);
    const { method: a, endpoint: i, body: c, headers: u = {} } = s,
      l = y(a, u, c);
    return (
      m(`${f}${i}`, l)
        .then(async (e) => {
          const r = await g(e);
          t({ type: o, requestId: n, response: r });
        })
        .catch((e) => {
          const s = e;
          p(s, 'background.js:onMessageExternal', { requestId: n, senderId: r.id }),
            t(h(s.message, n || ''));
        }),
      !0
    );
  } catch (s) {
    const n = s;
    p(n, 'background.js:onMessageExternal', {
      messageType: e?.type,
      requestId: e?.requestId,
      senderId: r.id,
    }),
      t(h(n.message, e?.requestId || ''));
  }
}),
  chrome.runtime.onConnectExternal.addListener((e) => {
    I.info('Received onConnectExternal connection request', {
      senderId: e.sender?.id || 'unknown',
    });
    try {
      e.onMessage.addListener(async (r) => {
        const t = E(r),
          s = r.requestId;
        try {
          if (t.type === d) return void e.postMessage(t);
          const { endpoint: r, method: n, body: o, headers: a = {} } = t,
            i = y(n, a, o);
          q(`${f}${r}`, i, (r) => e.postMessage(r), s || '', !0);
        } catch (r) {
          const t = r;
          p(t, 'background.js:onConnectExternal.onMessage', {
            requestId: s,
            senderId: e.sender?.id,
          }),
            e.postMessage(h(t.message, s || '', !0));
        }
      }),
        e.onDisconnect.addListener(() => {
          I.info('Extension disconnected', { senderId: e.sender?.id || 'unknown' });
        });
    } catch (r) {
      p(r, 'background.js:onConnectExternal', { senderId: e.sender?.id });
    }
  }),
  (() => {
    try {
      I.info('Background service worker initialized'),
        chrome.storage.local.get([s], (e) => {
          e[s]
            ? ((f = e[s]), I.info('Using configured backend URL', { API_BASE_URL: f }))
            : I.info('Using default backend URL', { API_BASE_URL: f });
        }),
        chrome.storage.onChanged.addListener((e, r) => {
          'local' === r &&
            e[s] &&
            ((f = e[s].newValue || t), I.info('Backend URL updated', { API_BASE_URL: f }));
        });
    } catch (e) {
      p(e, 'background.js:initExtension');
    }
  })();
//# sourceMappingURL=background.js.map
