(self.webpackChunk_N_E = self.webpackChunk_N_E || []).push([
  [332],
  {
    6760: (e, t, s) => {
      (window.__NEXT_P = window.__NEXT_P || []).push([
        '/',
        function () {
          return s(8921);
        },
      ]);
    },
    8921: (e, t, s) => {
      'use strict';
      s.r(t), s.d(t, { default: () => g });
      var r = s(7876),
        n = s(4232);
      let o = { TEST_CONNECTION: 'BODHI_TEST_CONNECTION' },
        c = () => {
          try {
            if (window.crypto && window.crypto.randomUUID) return window.crypto.randomUUID();
            return (
              Math.random().toString(36).substring(2, 15) +
              Math.random().toString(36).substring(2, 15)
            );
          } catch (e) {
            return (
              console.error('[Bodhi/ExtensionUI] Error generating request ID', e),
              'req_'.concat(Date.now(), '_').concat(Math.floor(1e3 * Math.random()))
            );
          }
        },
        a = (e) =>
          new Promise((t) => {
            let s = c(),
              r = { type: o.TEST_CONNECTION, requestId: s, testUrl: e };
            chrome.runtime.sendMessage(r, (s) => {
              if (chrome.runtime.lastError) {
                t({
                  status: 'unreachable',
                  url: e,
                  error: {
                    message: chrome.runtime.lastError.message || 'Extension communication error',
                    type: 'extension_error',
                  },
                });
                return;
              }
              if (!s || !s.response) {
                t({
                  status: 'unreachable',
                  url: e,
                  error: { message: 'No response from background script', type: 'extension_error' },
                });
                return;
              }
              t(s.response.body);
            });
          }),
        l = (e) => {
          switch (e.status) {
            case 'ready':
            case 'setup':
            case 'resource-admin':
              return 'connected';
            case 'unreachable':
              return 'disconnected';
            default:
              return 'error';
          }
        },
        i = (e) => {
          var t, s;
          switch (e.status) {
            case 'ready':
              return 'Connected - Ready for AI requests';
            case 'setup':
              return 'Connected - Server in setup mode';
            case 'resource-admin':
              return 'Connected - Resource administration mode';
            case 'unreachable':
              return (
                (null === (t = e.error) || void 0 === t ? void 0 : t.message) ||
                'Cannot connect to server'
              );
            case 'error':
              return (
                (null === (s = e.error) || void 0 === s ? void 0 : s.message) || 'Server error'
              );
            default:
              return 'Unknown status';
          }
        };
      function d(e) {
        let { onGetStarted: t } = e,
          [s, o] = (0, n.useState)(!1);
        return (
          (0, n.useEffect)(() => {
            let e = () => {
              o(window.outerWidth > 450);
            };
            return (
              e(),
              window.addEventListener('resize', e),
              () => window.removeEventListener('resize', e)
            );
          }, []),
          (0, r.jsx)('div', {
            className: s
              ? 'min-h-screen bg-gradient-to-br from-purple-600 to-teal-500 p-8 flex items-center justify-center'
              : 'w-[350px] h-[500px] overflow-auto bg-gradient-to-br from-purple-600 to-teal-500 p-3',
            children: (0, r.jsxs)('main', {
              className: 'flex flex-col bg-white rounded-lg shadow-lg w-full '.concat(
                s ? 'gap-5 p-6 max-w-2xl' : 'gap-3 p-4 h-full'
              ),
              children: [
                (0, r.jsxs)('div', {
                  className: 'text-center',
                  children: [
                    (0, r.jsx)('h1', {
                      className: 'font-bold text-gray-800 '.concat(
                        s ? 'text-2xl mb-3' : 'text-lg mb-1'
                      ),
                      children: 'Welcome to Bodhi Browser',
                    }),
                    (0, r.jsx)('p', {
                      className: 'text-gray-600 '.concat(s ? 'text-lg' : 'text-xs'),
                      children: 'AI-powered web browsing with your own local LLM',
                    }),
                  ],
                }),
                (0, r.jsx)('div', {
                  className: s ? 'grid md:grid-cols-2 gap-5' : 'flex-1 space-y-2',
                  children: [
                    {
                      icon: '⚡',
                      title: 'AI powered by Local LLMs',
                      description: 'Have AI features of websites powered by your own LLM',
                    },
                    {
                      icon: '\uD83D\uDEE1️',
                      title: 'Security',
                      description:
                        'All access by user consent, secured using industry standard OAuth2',
                    },
                    {
                      icon: '\uD83C\uDF10',
                      title: 'Web Integration',
                      description: 'Enables websites to use your local AI',
                    },
                    {
                      icon: '\uD83D\uDD12',
                      title: 'Privacy',
                      description: 'Local AI keeps your data private too',
                    },
                  ].map((e, t) =>
                    (0, r.jsxs)(
                      'div',
                      {
                        className: 'flex gap-3 '.concat(s ? 'items-start' : 'items-center'),
                        children: [
                          (0, r.jsx)('div', {
                            className:
                              'bg-purple-100 rounded-full flex items-center justify-center flex-shrink-0 '.concat(
                                s ? 'w-10 h-10' : 'w-6 h-6'
                              ),
                            children: (0, r.jsx)('span', {
                              className: 'text-purple-600 font-bold '.concat(
                                s ? 'text-lg' : 'text-xs'
                              ),
                              children: e.icon,
                            }),
                          }),
                          (0, r.jsxs)('div', {
                            children: [
                              (0, r.jsx)('h3', {
                                className: 'font-medium text-gray-800 '.concat(
                                  s ? 'text-lg mb-2' : 'text-xs'
                                ),
                                children: e.title,
                              }),
                              (0, r.jsx)('p', {
                                className: 'text-gray-600 '.concat(s ? 'text-sm' : 'text-xs'),
                                children: e.description,
                              }),
                            ],
                          }),
                        ],
                      },
                      t
                    )
                  ),
                }),
                (0, r.jsxs)('div', {
                  className: 'bg-gray-50 rounded-lg border '.concat(s ? 'p-5' : 'p-2'),
                  children: [
                    (0, r.jsx)('h3', {
                      className: 'font-medium text-gray-800 '.concat(
                        s ? 'text-lg mb-3' : 'text-xs mb-1'
                      ),
                      children: 'How it works:',
                    }),
                    (0, r.jsxs)('ol', {
                      className: 'text-gray-600 list-decimal list-inside '.concat(
                        s ? 'text-sm space-y-2' : 'text-xs space-y-0.5'
                      ),
                      children: [
                        (0, r.jsxs)(
                          'li',
                          {
                            children: [
                              'Download and install Bodhi App from',
                              ' ',
                              (0, r.jsx)('a', {
                                href: 'https://getbodhi.app/',
                                target: '_blank',
                                className: 'text-blue-500',
                                children: 'https://getbodhi.app/',
                              }),
                            ],
                          },
                          '0'
                        ),
                        [
                          'Configure extension for your Bodhi App url',
                          'Access Bodhi Platform powered websites to have them use your local AI with your permission',
                        ].map((e, t) => (0, r.jsx)('li', { children: e }, t)),
                      ],
                    }),
                  ],
                }),
                (0, r.jsx)('button', {
                  onClick: t,
                  className:
                    'w-full text-white bg-purple-600 rounded-md hover:bg-purple-700 focus:outline-none focus:ring-2 focus:ring-purple-500 focus:ring-offset-2 font-medium '.concat(
                      s ? 'px-6 py-3 text-lg' : 'px-3 py-2 text-sm'
                    ),
                  children: 'Get Started',
                }),
              ],
            }),
          })
        );
      }
      function u(e) {
        let { onBack: t } = e,
          [s, o] = (0, n.useState)(!1);
        return (
          (0, n.useEffect)(() => {
            let e = () => {
              o(window.outerWidth > 450);
            };
            return (
              e(),
              window.addEventListener('resize', e),
              () => window.removeEventListener('resize', e)
            );
          }, []),
          (0, r.jsx)('div', {
            className: s
              ? 'min-h-screen bg-gradient-to-br from-purple-600 to-teal-500 p-8 flex items-center justify-center'
              : 'w-[350px] h-[500px] overflow-auto bg-gradient-to-br from-purple-600 to-teal-500 p-4',
            children: (0, r.jsxs)('main', {
              className: 'flex flex-col bg-white rounded-lg shadow-lg w-full '.concat(
                s ? 'gap-5 p-6 max-w-4xl' : 'gap-4 p-4 h-full'
              ),
              children: [
                (0, r.jsxs)('div', {
                  className: 'flex items-center '.concat(s ? 'gap-4' : 'gap-3'),
                  children: [
                    (0, r.jsx)('button', {
                      onClick: t,
                      className: 'text-gray-500 hover:text-gray-700 focus:outline-none '.concat(
                        s ? 'text-lg' : ''
                      ),
                      'aria-label': 'Back to settings',
                      children: '← Back',
                    }),
                    (0, r.jsx)('h1', {
                      className: 'font-bold text-gray-800 '.concat(s ? 'text-2xl' : 'text-lg'),
                      children: 'Help & Information',
                    }),
                  ],
                }),
                (0, r.jsxs)('div', {
                  className: s ? 'grid md:grid-cols-2 gap-6' : 'flex-1 space-y-4 overflow-y-auto',
                  children: [
                    (0, r.jsxs)('div', {
                      className: 'space-y-4',
                      children: [
                        (0, r.jsxs)('div', {
                          className: 'space-y-4',
                          children: [
                            (0, r.jsx)('h2', {
                              className: 'font-semibold text-gray-800 '.concat(
                                s ? 'text-xl' : 'text-lg'
                              ),
                              children: 'Permissions Explained',
                            }),
                            [
                              {
                                icon: '\uD83D\uDCCD',
                                title: '"Access all websites"',
                                description:
                                  'This permission allows the extension to inject the Bodhi API object into web pages you visit. This is required for web applications to communicate with your local AI server. The extension only activates when websites request AI functionality.',
                                color: 'blue',
                              },
                              {
                                icon: '\uD83D\uDCBE',
                                title: '"Storage"',
                                description:
                                  "Used to remember your server URL settings so you don't have to re-enter them every time. No personal data or conversation history is stored.",
                                color: 'green',
                              },
                            ].map((e, t) =>
                              (0, r.jsxs)(
                                'div',
                                {
                                  className: 'bg-'
                                    .concat(e.color, '-50 rounded-lg border ')
                                    .concat(s ? 'p-4' : 'p-3'),
                                  children: [
                                    (0, r.jsxs)('h3', {
                                      className: 'font-medium text-'
                                        .concat(e.color, '-900 ')
                                        .concat(s ? 'text-lg mb-3' : 'text-sm mb-2'),
                                      children: [e.icon, ' ', e.title],
                                    }),
                                    (0, r.jsx)('p', {
                                      className: 'text-'
                                        .concat(e.color, '-800 leading-relaxed ')
                                        .concat(s ? 'text-sm' : 'text-xs'),
                                      children: e.description,
                                    }),
                                  ],
                                },
                                t
                              )
                            ),
                          ],
                        }),
                        (0, r.jsxs)('div', {
                          className: 'space-y-4',
                          children: [
                            (0, r.jsx)('h2', {
                              className: 'font-semibold text-gray-800 '.concat(
                                s ? 'text-xl' : 'text-lg'
                              ),
                              children: 'Privacy & Security',
                            }),
                            (0, r.jsx)('div', {
                              className: 'bg-purple-50 rounded-lg border '.concat(
                                s ? 'p-4' : 'p-3'
                              ),
                              children: (0, r.jsx)('div', {
                                className: 'space-y-3',
                                children: [
                                  'All AI processing happens on your local Bodhi App servers',
                                  'No data is sent to cloud services or third parties',
                                  'Extension only connects to your specified Bodhi App server',
                                  'Open source code',
                                ].map((e, t) =>
                                  (0, r.jsxs)(
                                    'div',
                                    {
                                      className: 'flex items-center '.concat(s ? 'gap-3' : 'gap-2'),
                                      children: [
                                        (0, r.jsx)('span', {
                                          className: 'text-purple-600 '.concat(s ? 'text-lg' : ''),
                                          children: '✓',
                                        }),
                                        (0, r.jsx)('span', {
                                          className: 'text-purple-800 '.concat(
                                            s ? 'text-sm' : 'text-xs'
                                          ),
                                          children: e,
                                        }),
                                      ],
                                    },
                                    t
                                  )
                                ),
                              }),
                            }),
                          ],
                        }),
                      ],
                    }),
                    (0, r.jsxs)('div', {
                      className: 'space-y-4',
                      children: [
                        (0, r.jsxs)('div', {
                          className: 'space-y-4',
                          children: [
                            (0, r.jsx)('h2', {
                              className: 'font-semibold text-gray-800 '.concat(
                                s ? 'text-xl' : 'text-lg'
                              ),
                              children: 'Troubleshooting',
                            }),
                            (0, r.jsx)('div', {
                              className: 'space-y-3',
                              children: [
                                {
                                  title: 'Connection Failed?',
                                  tips: [
                                    'Make sure your Bodhi App server is running',
                                    'Check the server URL is correct (e.g. http://localhost:1135)',
                                    'Check if firewall is blocking the connection',
                                  ],
                                },
                                {
                                  title: 'Extension Not Working?',
                                  tips: [
                                    'Refresh the web page after changing settings',
                                    'Check browser console for error messages',
                                    'Try disabling and re-enabling the extension',
                                    'Restart your browser if problems persist',
                                  ],
                                },
                              ].map((e, t) =>
                                (0, r.jsxs)(
                                  'div',
                                  {
                                    className: 'bg-gray-50 rounded-lg border '.concat(
                                      s ? 'p-4' : 'p-3'
                                    ),
                                    children: [
                                      (0, r.jsx)('h4', {
                                        className: 'font-medium text-gray-800 '.concat(
                                          s ? 'text-lg mb-2' : 'text-sm'
                                        ),
                                        children: e.title,
                                      }),
                                      (0, r.jsx)('ul', {
                                        className:
                                          'text-gray-600 space-y-1 list-disc list-inside '.concat(
                                            s ? 'text-sm' : 'text-xs mt-1'
                                          ),
                                        children: e.tips.map((e, t) =>
                                          (0, r.jsx)('li', { children: e }, t)
                                        ),
                                      }),
                                    ],
                                  },
                                  t
                                )
                              ),
                            }),
                          ],
                        }),
                        (0, r.jsxs)('div', {
                          className: 'space-y-4',
                          children: [
                            (0, r.jsx)('h2', {
                              className: 'font-semibold text-gray-800 '.concat(
                                s ? 'text-xl' : 'text-lg'
                              ),
                              children: 'Usage Tips',
                            }),
                            (0, r.jsx)('div', {
                              className: 'bg-yellow-50 rounded-lg border '.concat(
                                s ? 'p-4' : 'p-3'
                              ),
                              children: (0, r.jsx)('ul', {
                                className: 'text-yellow-800 list-disc list-inside '.concat(
                                  s ? 'text-sm space-y-2' : 'text-xs space-y-1'
                                ),
                                children: [
                                  'Use the "Test Connection" button to verify your setup',
                                  'Server status is shown in the connection badge',
                                  'You can open settings in a new tab for easier configuration',
                                ].map((e, t) => (0, r.jsx)('li', { children: e }, t)),
                              }),
                            }),
                          ],
                        }),
                      ],
                    }),
                  ],
                }),
              ],
            }),
          })
        );
      }
      let x = 'http://localhost:1135',
        p = 'backendUrl',
        m = 'bodhiWelcomeShown';
      function h() {
        let [e, t] = (0, n.useState)(''),
          [s, o] = (0, n.useState)(!1),
          [c, h] = (0, n.useState)({ type: '', text: '' }),
          [g, f] = (0, n.useState)(null),
          [y, b] = (0, n.useState)(!1),
          [w, v] = (0, n.useState)('settings'),
          [j, N] = (0, n.useState)(!1),
          S = async (e) => {
            chrome.storage.local.get([p], async (s) => {
              let r = s[p] || x;
              t(r);
              try {
                let t = await a(r);
                f(t),
                  s[p] ||
                    'connected' !== l(t) ||
                    (await chrome.storage.local.set({ [p]: x }),
                    console.log('[Bodhi/Settings] Auto-saved default URL after '.concat(e)));
              } catch (t) {
                console.error('Failed to test connection on '.concat(e, ':'), t);
              }
            });
          };
        (0, n.useEffect)(() => {
          let e = () => {
            N(window.outerWidth > 450);
          };
          return (
            e(),
            window.addEventListener('resize', e),
            localStorage.getItem(m) ? S('component mount') : v('welcome'),
            () => window.removeEventListener('resize', e)
          );
        }, []);
        let k = async () => {
            if (!e) {
              h({ type: 'error', text: 'Please enter a backend URL first' });
              return;
            }
            b(!0), h({ type: '', text: '' });
            try {
              let t = await a(e);
              f(t);
              let s = l(t);
              'connected' === s
                ? h({ type: 'success', text: 'Connection successful!' })
                : h({ type: 'error', text: i(t) });
            } catch (e) {
              h({ type: 'error', text: 'Failed to test connection' }),
                console.error('Connection test error:', e);
            } finally {
              b(!1);
            }
          },
          E = async (s) => {
            s.preventDefault(), o(!0), h({ type: '', text: '' });
            try {
              let s = e.replace(/\/+$/, '');
              new URL(s),
                await chrome.storage.local.set({ [p]: s }),
                t(s),
                h({ type: 'success', text: 'Settings saved successfully!' });
            } catch (t) {
              h({
                type: 'error',
                text: "Input url '".concat(e, "' is not a valid URL. Please enter a valid URL"),
              });
            } finally {
              o(!1);
            }
          };
        if ('welcome' === w)
          return (0, r.jsx)(d, {
            onGetStarted: () => {
              localStorage.setItem(m, 'true'), v('settings'), S('welcome completion');
            },
          });
        if ('help' === w)
          return (0, r.jsx)(u, {
            onBack: () => {
              v('settings');
            },
          });
        let C = g ? l(g) : null,
          I = j ? 'text-base' : 'text-sm';
        return (0, r.jsx)('div', {
          className: j
            ? 'min-h-screen bg-gradient-to-br from-purple-600 to-teal-500 p-8 flex items-center justify-center'
            : 'w-[350px] h-[500px] overflow-auto bg-gradient-to-br from-purple-600 to-teal-500 p-4',
          children: (0, r.jsxs)('main', {
            className: 'flex flex-col bg-white rounded-lg shadow-lg w-full '.concat(
              j ? 'gap-5 p-6 max-w-2xl' : 'gap-4 p-4'
            ),
            children: [
              (0, r.jsxs)('div', {
                className: 'flex items-center justify-between',
                children: [
                  (0, r.jsx)('h1', {
                    className: 'font-bold text-gray-800 '.concat(j ? 'text-2xl' : 'text-lg'),
                    children: j ? 'Bodhi Browser Settings' : 'Bodhi Browser',
                  }),
                  (0, r.jsxs)('div', {
                    className: 'flex items-center gap-2',
                    children: [
                      C &&
                        (0, r.jsx)('span', {
                          className: 'rounded-full font-medium '
                            .concat(j ? 'px-3 py-2 text-sm' : 'px-2 py-1 text-xs', ' ')
                            .concat(
                              {
                                connected: 'bg-green-100 text-green-800',
                                disconnected: 'bg-red-100 text-red-800',
                                error: 'bg-yellow-100 text-yellow-800',
                              }[C]
                            ),
                          children:
                            'connected' === C
                              ? '● Connected'
                              : 'disconnected' === C
                                ? '● Disconnected'
                                : '● Error',
                        }),
                      (0, r.jsx)('button', {
                        onClick: () => {
                          v('help');
                        },
                        className:
                          'text-gray-500 hover:text-gray-700 focus:outline-none focus:ring-2 focus:ring-purple-500 rounded-full flex items-center justify-center '.concat(
                            j ? 'w-8 h-8' : 'w-6 h-6'
                          ),
                        title: 'Help and information',
                        'aria-label': 'Help',
                        children: '?',
                      }),
                    ],
                  }),
                ],
              }),
              (0, r.jsxs)('div', {
                className: j ? 'grid md:grid-cols-2 gap-6' : 'space-y-4',
                children: [
                  (0, r.jsx)('div', {
                    className: 'space-y-4',
                    children: (0, r.jsxs)('form', {
                      onSubmit: E,
                      className: 'space-y-4',
                      children: [
                        (0, r.jsxs)('div', {
                          className: 'space-y-3',
                          children: [
                            (0, r.jsx)('label', {
                              htmlFor: 'backendUrl',
                              className: 'block font-medium text-gray-700 '.concat(I),
                              children: 'Bodhi App Server URL',
                            }),
                            (0, r.jsx)('input', {
                              type: 'text',
                              id: 'backendUrl',
                              value: e,
                              onChange: (e) => t(e.target.value),
                              placeholder: x,
                              className:
                                'w-full border border-gray-300 rounded-md shadow-sm bg-white text-gray-900 placeholder-gray-500 focus:outline-none focus:ring-2 focus:ring-purple-500 focus:border-purple-500 '.concat(
                                  j ? 'px-4 py-3 text-base' : 'px-3 py-2 text-sm'
                                ),
                            }),
                            (0, r.jsx)('button', {
                              type: 'button',
                              onClick: k,
                              disabled: y || !e,
                              className:
                                'w-full border border-purple-300 text-purple-700 rounded-md hover:bg-purple-50 focus:outline-none focus:ring-2 focus:ring-purple-500 focus:ring-offset-2 '
                                  .concat(j ? 'px-4 py-3 text-base' : 'px-3 py-2 text-sm', ' ')
                                  .concat(y || !e ? 'opacity-50 cursor-not-allowed' : ''),
                              children: y ? 'Testing Connection...' : 'Test Connection',
                            }),
                          ],
                        }),
                        c.text &&
                          (0, r.jsx)('div', {
                            id: 'message-container',
                            className: 'message-container rounded-md '
                              .concat(j ? 'p-4 text-base' : 'p-3 text-sm', ' ')
                              .concat(c.type, ' ')
                              .concat(
                                'success' === c.type
                                  ? 'bg-green-100 text-green-700'
                                  : 'bg-red-100 text-red-700'
                              ),
                            children: c.text,
                          }),
                        (0, r.jsx)('button', {
                          id: 'submit',
                          type: 'submit',
                          disabled: s,
                          className:
                            'w-full text-white bg-purple-600 rounded-md hover:bg-purple-700 focus:outline-none focus:ring-2 focus:ring-purple-500 focus:ring-offset-2 font-medium '
                              .concat(j ? 'px-6 py-3 text-base' : 'px-4 py-2 text-sm', ' ')
                              .concat(s ? 'opacity-50 cursor-not-allowed' : ''),
                          children: s ? 'Saving...' : 'Save Settings',
                        }),
                      ],
                    }),
                  }),
                  (g && 'connected' === C) || j
                    ? (0, r.jsxs)('div', {
                        className: 'space-y-4',
                        children: [
                          g &&
                            'connected' === C &&
                            (0, r.jsxs)('div', {
                              className: 'bg-gray-50 rounded-lg border '.concat(j ? 'p-5' : 'p-3'),
                              children: [
                                (0, r.jsx)('h3', {
                                  className: 'font-medium text-gray-900 '.concat(
                                    j ? 'text-lg mb-4' : 'text-sm mb-2'
                                  ),
                                  children: 'Server Information',
                                }),
                                (0, r.jsxs)('div', {
                                  className: 'text-gray-600 '.concat(
                                    j ? 'space-y-2 text-sm' : 'space-y-1 text-xs'
                                  ),
                                  children: [
                                    (0, r.jsxs)('div', {
                                      className: 'flex justify-between',
                                      children: [
                                        (0, r.jsx)('span', {
                                          className: j ? 'font-medium' : '',
                                          children: 'Status:',
                                        }),
                                        (0, r.jsx)('span', {
                                          className: 'capitalize '.concat(
                                            j ? 'bg-green-100 text-green-800 px-2 py-1 rounded' : ''
                                          ),
                                          children: g.status,
                                        }),
                                      ],
                                    }),
                                    g.version &&
                                      (0, r.jsxs)('div', {
                                        className: 'flex justify-between',
                                        children: [
                                          (0, r.jsx)('span', {
                                            className: j ? 'font-medium' : '',
                                            children: 'Version:',
                                          }),
                                          (0, r.jsx)('span', { children: g.version }),
                                        ],
                                      }),
                                    (0, r.jsxs)('div', {
                                      className: 'flex justify-between '.concat(
                                        j ? 'items-start' : ''
                                      ),
                                      children: [
                                        (0, r.jsx)('span', {
                                          className: j ? 'font-medium' : '',
                                          children: 'URL:',
                                        }),
                                        (0, r.jsx)('span', {
                                          className: j ? 'text-right break-all' : 'truncate ml-2',
                                          children: g.url,
                                        }),
                                      ],
                                    }),
                                  ],
                                }),
                              ],
                            }),
                          j &&
                            (0, r.jsxs)('div', {
                              className: 'bg-purple-50 p-5 rounded-lg border',
                              children: [
                                (0, r.jsx)('h3', {
                                  className: 'text-lg font-medium text-purple-900 mb-3',
                                  children: 'Quick Setup Guide',
                                }),
                                (0, r.jsx)('ol', {
                                  className:
                                    'text-sm text-purple-800 space-y-1 list-decimal list-inside',
                                  children: [
                                    'Start Bodhi App on your local machine',
                                    'Enter the server URL',
                                    'Click "Test Connection" to verify',
                                    'Save your settings',
                                  ].map((e, t) => (0, r.jsx)('li', { children: e }, t)),
                                }),
                              ],
                            }),
                        ],
                      })
                    : null,
                ],
              }),
            ],
          }),
        });
      }
      function g() {
        return (0, r.jsx)(h, {});
      }
    },
  },
  (e) => {
    var t = (t) => e((e.s = t));
    e.O(0, [636, 593, 792], () => t(6760)), (_N_E = e.O());
  },
]);
