const e = 'POST',
  t = '/v1/chat/completions',
  r = 'bodhiext:initialized';
const s = (e) => ({
    info: (e, t) => {},
    error: (t, r) => {
      const s = (function (e) {
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
      })(r);
      console.error(`[Bodhi/${e}] ${t}`, ...s);
    },
  }),
  n =
    (s('shared'),
    (e, t, r) => {
      const s = new Error(e);
      return (s.status = t), (s.body = r), s;
    }),
  o = (e, t) => (e?.body?.error?.message ? e.body.error.message : t);
!(function () {
  const i = 'Request timed out',
    a = 'Stream error',
    d = s('inject.js');
  d.info('Interface initializing'),
    (() => {
      if (void 0 !== window.bodhiext)
        return void d.info('Interface already exists, skipping initialization');
      const s = {
          _private: {
            requests: new Map(),
            streams: new Map(),
            baseUrl: 'http://localhost:1135',
            generateId: () => {
              try {
                return window.crypto && window.crypto.randomUUID
                  ? window.crypto.randomUUID()
                  : Math.random().toString(36).substring(2, 15) +
                      Math.random().toString(36).substring(2, 15);
              } catch (e) {
                return (
                  d.error('Error generating request ID', { error: e }),
                  `req_${Date.now()}_${Math.floor(1e3 * Math.random())}`
                );
              }
            },
            handleApiResponse: (e) => {
              if (e.data && e.data.type)
                try {
                  const { type: t, requestId: r, response: s } = e.data;
                  switch (t) {
                    case 'BODHI_API_RESPONSE':
                      u(r, s);
                      break;
                    case 'BODHI_STREAM_CHUNK':
                      c(r, s);
                      break;
                    case 'BODHI_STREAM_ERROR':
                      p(r, s);
                      break;
                    case 'BODHI_ERROR':
                      _(r, s);
                      break;
                    case 'BODHI_SET_EXTENSION_ID':
                      l(e.data.extension_id);
                  }
                } catch (e) {
                  d.error('Error handling API response', { error: e });
                }
              else d.info('Ignoring API response with no type', { data: e.data });
            },
            sendApiRequest: function (e, t, r = null, s = {}) {
              return new Promise((n, o) => {
                try {
                  const a = this.generateId();
                  this.requests.set(a, { resolve: n, reject: o }),
                    window.postMessage(
                      {
                        type: 'BODHI_API_REQUEST',
                        requestId: a,
                        request: { method: e, endpoint: t, body: r, headers: s },
                      },
                      window.origin || '*'
                    ),
                    setTimeout(() => {
                      this.requests.has(a) && (this.requests.delete(a), o(new Error(`${i}: ${t}`)));
                    }, 15e4);
                } catch (e) {
                  o(e);
                }
              });
            },
            sendStreamRequest: function (e, t, r = null, s = {}) {
              const n = this.generateId();
              return new ReadableStream({
                start: (o) => {
                  this.streams.set(n, {
                    enqueue: (e) => o.enqueue(e),
                    error: (e) => o.error(e),
                    complete: () => o.close(),
                  }),
                    window.postMessage(
                      {
                        type: 'BODHI_STREAM_REQUEST',
                        requestId: n,
                        request: { method: e, endpoint: t, body: r, headers: s },
                      },
                      window.origin || '*'
                    ),
                    setTimeout(() => {
                      this.streams.has(n) &&
                        (this.streams.get(n).error(new Error(`${i}: ${t}`)),
                        this.streams.delete(n));
                    }, 12e4);
                },
                cancel: (e) => {
                  this.streams.has(n) &&
                    (this.streams.delete(n),
                    d.info('Stream was cancelled', { requestId: n, reason: e }));
                },
              });
            },
            requestExtensionId: function () {
              window.postMessage({ type: 'BODHI_GET_EXTENSION_ID' }, window.origin || '*');
            },
            _getExtensionId: function () {
              return s.extension_id
                ? Promise.resolve(s.extension_id)
                : new Promise((e) => {
                    const t = (s) => {
                      window.removeEventListener(r, t), e(s.detail.extensionId);
                    };
                    window.addEventListener(r, t), this.requestExtensionId();
                  });
            },
          },
          extension_id: null,
          ping: function () {
            return this._private.sendApiRequest('GET', '/ping').then((e) => e.body);
          },
          serverState: function () {
            return this._private
              .sendApiRequest('GET', '/bodhi/v1/info')
              .then((e) =>
                0 === e.status
                  ? {
                      status: 'unreachable',
                      url: this._private.baseUrl,
                      error: {
                        message: e.body?.error?.message || 'Failed to connect to server',
                        type: 'network_error',
                      },
                    }
                  : 200 === e.status && e.body
                    ? { status: e.body.status, version: e.body.version, url: this._private.baseUrl }
                    : e.status >= 400
                      ? {
                          status: 'error',
                          url: this._private.baseUrl,
                          error: e.body?.error || {
                            message: `Server returned status ${e.status}`,
                            type: 'server_error',
                          },
                        }
                      : {
                          status: 'error',
                          url: this._private.baseUrl,
                          error: {
                            message: 'Unexpected response format from server',
                            type: 'response_error',
                          },
                        }
              )
              .catch((e) => ({
                status: 'unreachable',
                url: this._private.baseUrl,
                error: {
                  message: e.message || 'Failed to connect to server',
                  type: 'network_error',
                },
              }));
          },
          chat: {
            completions: {
              create: function (r) {
                if (r && !0 === r.stream) {
                  const n = s._private.sendStreamRequest.bind(s._private)(e, t, r);
                  return {
                    [Symbol.asyncIterator]: async function* () {
                      const e = n.getReader();
                      try {
                        for (;;) {
                          const { done: t, value: r } = await e.read();
                          if (t) break;
                          yield r.body;
                        }
                      } finally {
                        e.releaseLock();
                      }
                    },
                  };
                }
                return s._private.sendApiRequest(e, t, r).then((e) => e.body);
              },
            },
          },
        },
        u = (e, t) => {
          if (!e || !s._private.requests.has(e))
            return void d.info('No matching request found for API response', { requestId: e });
          const { resolve: r } = s._private.requests.get(e);
          r(t), s._private.requests.delete(e);
        },
        c = (e, t) => {
          if (!e || !s._private.streams.has(e)) return;
          const r = s._private.streams.get(e);
          if (!((i = t.status) >= 200 && i < 299)) {
            const i = o(t, a);
            return r.error(n(i, t.status, t.body)), void s._private.streams.delete(e);
          }
          var i;
          if (t.body && t.body.done) return r.complete(), void s._private.streams.delete(e);
          r.enqueue(t);
        },
        p = (e, t) => {
          if (!e || !s._private.streams.has(e))
            return void d.info('No matching stream found for STREAM_ERROR', { requestId: e });
          const r = s._private.streams.get(e),
            i = o(t, a);
          r.error(n(i, t.status, t.body)), s._private.streams.delete(e);
        },
        _ = (e, t) => {
          if (e)
            if (s._private.streams.has(e)) {
              const r = s._private.streams.get(e),
                i = o(t, a);
              r.error(n(i, t.status, t.body)), s._private.streams.delete(e);
            } else if (s._private.requests.has(e)) {
              const { reject: r } = s._private.requests.get(e),
                i = o(t, 'Unknown error');
              r(n(i, t.status, t.body)), s._private.requests.delete(e);
            }
        },
        l = (e) => {
          s.extension_id ||
            ((s.extension_id = e),
            d.info('Extension ID set', { extension_id: s.extension_id }),
            window.dispatchEvent(new CustomEvent(r, { detail: { extensionId: s.extension_id } })),
            d.info('Dispatched event', { event: r }));
        },
        v = {
          sendApiRequest: s._private.sendApiRequest.bind(s._private),
          sendStreamRequest: s._private.sendStreamRequest.bind(s._private),
          ping: s.ping.bind(s),
          serverState: s.serverState.bind(s),
          chat: s.chat,
          getExtensionId: () => s._private._getExtensionId(),
        };
      Object.freeze(v),
        Object.defineProperty(window, 'bodhiext', { value: v, writable: !1, configurable: !1 }),
        window.addEventListener('message', s._private.handleApiResponse),
        s._private.requestExtensionId(),
        d.info('Interface successfully created');
    })();
})();
//# sourceMappingURL=inject.js.map
