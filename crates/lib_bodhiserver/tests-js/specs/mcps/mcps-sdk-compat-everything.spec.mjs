// MCP SDK compatibility suite.
//
// Unlike mcps-mcp-proxy-everything.spec.mjs (which drives the MCP Inspector
// UI in a browser for black-box coverage), this file exercises BodhiApp's
// MCP proxy endpoint at /bodhi/v1/apps/mcps/{id}/mcp through the OFFICIAL
// @modelcontextprotocol/sdk TypeScript client. It guarantees that 3rd-party
// MCP clients (agents, CLIs, IDEs) continue to interoperate with BodhiApp's
// Streamable HTTP passthrough.
//
// Coverage (see the feature matrix archived in the plan file at
// .cursor/plans/mcp-sdk-compat-test_9bbd22d2.plan.md under
// "Appendix A — MCP feature catalogue"):
//   • Lifecycle: initialize, instructions, ping, clean close
//   • Session: mcp-session-id round-trip via StreamableHTTPClientTransport
//   • Tools: list, call (echo / get-sum / get-tiny-image / structuredContent
//     / annotations / resource_link blocks / trigger-long-running-operation
//     with progressToken streaming)
//   • Resources: list, read, listResourceTemplates, subscribe +
//     notifications/resources/updated stream
//   • Prompts: list, getPrompt(simple / args), completion/complete for
//     completable-prompt argument
//   • Logging: setLoggingLevel + notifications/message stream
//   • Bidirectional (server→client via SSE): sampling/createMessage,
//     elicitation/create, roots/list — each relayed to a client-side
//     handler registered before connect
//   • Auth: OAuth JWT happy path, `bodhiapp_` API token rejection
//
// Not covered here (by design — see Appendix A):
//   • Cancellation (notifications/cancelled) — tracked separately
//   • Experimental tasks API (tasks/list, tasks/cancel, async sampling/
//     elicitation tools)
//   • List pagination with a real multi-page cursor (upstream fits in one page)
//
// Matrix: 1 MCP server (everything) × 1 auth method (OAuth app token) ×
// N capabilities. The happy path runs end-to-end in one test() using
// test.step() segments so the Playwright report shows a per-phase trace.
// Negative API-token rejection is its own step at the tail.

import { expect, test } from '@/fixtures.mjs';
import { McpFixtures } from '@/fixtures/mcpFixtures.mjs';
import { AccessRequestReviewPage } from '@/pages/AccessRequestReviewPage.mjs';
import { LoginPage } from '@/pages/LoginPage.mjs';
import { McpsPage } from '@/pages/McpsPage.mjs';
import { OAuthTestApp } from '@/pages/OAuthTestApp.mjs';
import { TokensPage } from '@/pages/TokensPage.mjs';
import { SHARED_STATIC_SERVER_URL } from '@/test-helpers.mjs';
import { mintApiToken } from '@/utils/api-model-helpers.mjs';
import {
  getAuthServerConfig,
  getPreConfiguredAppClient,
  getTestCredentials,
} from '@/utils/auth-server-client.mjs';
import { buildMcpClient, safeCloseMcpClient, waitFor } from '@/utils/mcp-sdk-client.mjs';

const ECHO_MESSAGE = 'sdk-e2e-hello';
const ARGS_PROMPT_CITY = 'SdkE2ECity';

// Stub values used to verify bidirectional request/response traversal of the
// BodhiApp proxy. These strings are echoed by our client-side handlers and
// asserted in the corresponding tool results (so if the proxy ever drops the
// server→client leg, the tool body will be missing our marker).
const SAMPLING_STUB_TEXT = 'bodhiapp-sampling-ack';
const ELICITATION_STUB_NAME = 'BodhiSdkTester';
const CLIENT_ROOT_URI = 'file:///tmp/bodhiapp-sdk-compat-e2e-root';
const CLIENT_ROOT_NAME = 'bodhiapp-sdk-compat-root';

test.describe(
  'MCP SDK compatibility — Everything Server via @modelcontextprotocol/sdk',
  { tag: ['@mcps', '@mcp-proxy', '@everything', '@sdk'] },
  () => {
    let authServerConfig;
    let testCredentials;

    test.beforeAll(async () => {
      authServerConfig = getAuthServerConfig();
      testCredentials = getTestCredentials();
    });

    test('MCP proxy full protocol journey via official SDK client', async ({
      page,
      sharedServerUrl,
    }) => {
      const loginPage = new LoginPage(page, sharedServerUrl, authServerConfig, testCredentials);
      const mcpsPage = new McpsPage(page, sharedServerUrl);
      const tokensPage = new TokensPage(page, sharedServerUrl);
      const serverData = McpFixtures.createEverythingServerData();
      const instanceData = McpFixtures.createEverythingInstanceData();

      let mcpId;
      let appToken;
      let apiToken;
      let sdkClient;
      let sdkTransport;

      // ── Phase 1: Create MCP server + instance via Bodhi UI ──

      await test.step('Login and create everything MCP server + instance', async () => {
        await loginPage.performOAuthLogin('/ui/chat/');
        await mcpsPage.createMcpServer(serverData.url, serverData.name, serverData.description);
        await mcpsPage.createMcpInstance(
          serverData.name,
          instanceData.name,
          instanceData.slug,
          instanceData.description
        );
        await mcpsPage.expectMcpsListPage();
        mcpId = await mcpsPage.getMcpUuidByName(instanceData.name);
        expect(mcpId).toBeTruthy();
      });

      // ── Phase 2: Mint bodhiapp_ API token (for the negative case) ──

      await test.step('Mint bodhiapp_ API token for negative-case coverage', async () => {
        apiToken = await mintApiToken(
          tokensPage,
          page,
          `mcp-sdk-compat-${Date.now()}`,
          'scope_token_user'
        );
        expect(apiToken).toMatch(/^bodhiapp_/);
      });

      // ── Phase 3: OAuth access request + app token via test-oauth-app ──

      const appClient = getPreConfiguredAppClient();
      const redirectUri = `${SHARED_STATIC_SERVER_URL}/callback`;
      const app = new OAuthTestApp(page, SHARED_STATIC_SERVER_URL);

      await test.step('OAuth flow — configure + submit access request', async () => {
        await app.navigate();
        await app.config.configureOAuthForm({
          bodhiServerUrl: sharedServerUrl,
          authServerUrl: authServerConfig.authUrl,
          realm: authServerConfig.authRealm,
          clientId: appClient.clientId,
          redirectUri,
          scope: 'openid profile email',
          requested: JSON.stringify({
            version: '1',
            mcp_servers: [{ url: McpFixtures.EVERYTHING_SERVER_MCP_URL }],
          }),
        });
        await app.config.submitAccessRequest();
        await app.oauth.waitForAccessRequestRedirect(sharedServerUrl);
      });

      await test.step('Approve access request with MCP instance and exchange token', async () => {
        const reviewPage = new AccessRequestReviewPage(page, sharedServerUrl);
        await reviewPage.approveWithMcps([
          { url: McpFixtures.EVERYTHING_SERVER_MCP_URL, instanceId: mcpId },
        ]);

        await app.oauth.waitForAccessRequestCallback(SHARED_STATIC_SERVER_URL);
        await app.accessCallback.waitForLoaded();
        await app.accessCallback.clickLogin();
        await app.oauth.waitForTokenExchange(SHARED_STATIC_SERVER_URL);

        await app.dashboard.navigateTo();
        appToken = await app.dashboard.getAccessToken();
        expect(appToken).toBeTruthy();
        expect(appToken.startsWith('eyJ')).toBe(true);
      });

      // ── Phase 4: Connect via official SDK + exercise MCP surface ──

      // Shared capture buffers for server→client notifications. Handlers below
      // push into these; test.step blocks later assert on them with waitFor.
      const loggingMessages = [];
      const resourceUpdatedEvents = [];

      await test.step('SDK connect — initialize handshake with full capability advertisement', async () => {
        // Import the SDK schemas lazily so this file remains safe to grep in
        // workers that don't actually load the SDK. These schemas identify
        // the server→client requests/notifications we want to handle.
        const {
          CreateMessageRequestSchema,
          ElicitRequestSchema,
          ListRootsRequestSchema,
          LoggingMessageNotificationSchema,
          ResourceUpdatedNotificationSchema,
        } = await import('@modelcontextprotocol/sdk/types.js');

        const built = await buildMcpClient({
          serverUrl: sharedServerUrl,
          mcpId,
          token: appToken,
          clientInfo: { name: 'bodhiapp-sdk-compat-e2e', version: '0.0.0' },
          // Advertise sampling / elicitation / roots so the server registers
          // its conditional tools (`trigger-sampling-request`,
          // `trigger-elicitation-request`, `get-roots-list`) and actually
          // issues server→client requests through the BodhiApp proxy's
          // SSE stream. `logging` + `resources.subscribe` are server caps,
          // not client caps, so they don't need to be mirrored here.
          capabilities: {
            sampling: {},
            elicitation: {},
            roots: { listChanged: false },
          },
          registerHandlers: (client) => {
            // sampling/createMessage: stub an assistant text back. The tool
            // `trigger-sampling-request` stringifies the result into its
            // content block, so asserting our stub text appears proves the
            // proxy forwarded both legs (request → client handler → response).
            client.setRequestHandler(CreateMessageRequestSchema, async (req) => {
              return {
                model: 'bodhiapp-sdk-stub',
                role: 'assistant',
                content: {
                  type: 'text',
                  text: `${SAMPLING_STUB_TEXT} | echoed: ${
                    req.params?.messages?.[0]?.content?.text ?? ''
                  }`,
                },
              };
            });
            // elicitation/create: accept the dialog with the minimum required
            // field (`name`). Additional keys are allowed but ignored by the
            // server-side formatter.
            client.setRequestHandler(ElicitRequestSchema, async () => ({
              action: 'accept',
              content: { name: ELICITATION_STUB_NAME },
            }));
            // roots/list: return one synthetic root. Server caches this for
            // the session and exposes it through `get-roots-list`.
            client.setRequestHandler(ListRootsRequestSchema, async () => ({
              roots: [{ uri: CLIENT_ROOT_URI, name: CLIENT_ROOT_NAME }],
            }));
            // notifications/message: logging stream. We count + store the
            // most recent so `get-logging-messages` step can assert arrival.
            client.setNotificationHandler(LoggingMessageNotificationSchema, (notification) => {
              loggingMessages.push(notification.params);
            });
            // notifications/resources/updated: captured for the subscribe step.
            client.setNotificationHandler(ResourceUpdatedNotificationSchema, (notification) => {
              resourceUpdatedEvents.push(notification.params);
            });
          },
        });
        sdkClient = built.client;
        sdkTransport = built.transport;

        const serverVersion = sdkClient.getServerVersion();
        expect(serverVersion).toBeDefined();
        expect(serverVersion.name).toBeTruthy();
        // Upstream reports "mcp-servers/everything" (or historically
        // "example-servers/everything") as the implementation name. Assert
        // loosely so the test survives minor upstream renames.
        expect(serverVersion.name.toLowerCase()).toContain('everything');

        const caps = sdkClient.getServerCapabilities();
        expect(caps).toBeDefined();
        expect(caps.tools).toBeDefined();
        expect(caps.resources).toBeDefined();
        expect(caps.prompts).toBeDefined();
        expect(caps.logging).toBeDefined();
        expect(caps.resources.subscribe).toBe(true);
      });

      await test.step('Session-id round-trip — proxy forwards mcp-session-id', async () => {
        // StreamableHTTPClientTransport exposes the sessionId the server
        // assigned in the initialize response. A populated, non-empty
        // string proves the proxy forwarded `mcp-session-id` both ways.
        expect(typeof sdkTransport.sessionId).toBe('string');
        expect(sdkTransport.sessionId.length).toBeGreaterThan(0);
      });

      await test.step('Tools — list returns the full everything catalogue (incl. conditional)', async () => {
        const { tools } = await sdkClient.listTools();
        expect(Array.isArray(tools)).toBe(true);
        const names = tools.map((t) => t.name);
        for (const expected of McpFixtures.EVERYTHING_EXPECTED_TOOLS) {
          expect(names).toContain(expected);
        }
        // Conditional tools registered because we advertised sampling /
        // elicitation / roots at connect time. If the proxy dropped
        // capability advertisement during initialize, these would be absent.
        for (const expected of [
          'trigger-sampling-request',
          'trigger-elicitation-request',
          'get-roots-list',
        ]) {
          expect(names).toContain(expected);
        }
      });

      await test.step('Tools — callTool(echo) round-trips user payload', async () => {
        const result = await sdkClient.callTool({
          name: 'echo',
          arguments: { message: ECHO_MESSAGE },
        });
        expect(result.isError).not.toBe(true);
        const text = (result.content ?? [])
          .filter((b) => b.type === 'text')
          .map((b) => b.text)
          .join('');
        expect(text).toContain(ECHO_MESSAGE);
      });

      await test.step('Tools — callTool(get-sum) returns arithmetic result', async () => {
        const result = await sdkClient.callTool({
          name: 'get-sum',
          arguments: { a: 7, b: 13 },
        });
        expect(result.isError).not.toBe(true);
        const text = (result.content ?? [])
          .filter((b) => b.type === 'text')
          .map((b) => b.text)
          .join('');
        expect(text).toContain('20');
      });

      await test.step('Tools — callTool(get-tiny-image) returns an image block', async () => {
        const result = await sdkClient.callTool({ name: 'get-tiny-image', arguments: {} });
        if (result.isError) {
          const text = (result.content ?? [])
            .filter((b) => b.type === 'text')
            .map((b) => b.text)
            .join(' | ');
          throw new Error(`get-tiny-image returned isError=true. content: ${text}`);
        }
        const image = (result.content ?? []).find((b) => b.type === 'image');
        expect(image).toBeDefined();
        expect(image.mimeType).toMatch(/^image\//);
        expect(typeof image.data).toBe('string');
        expect(image.data.length).toBeGreaterThan(0);
      });

      await test.step('Tools — callTool(get-structured-content) preserves structuredContent', async () => {
        const result = await sdkClient.callTool({
          name: 'get-structured-content',
          arguments: { location: 'New York' },
        });
        expect(result.isError).not.toBe(true);
        expect(result.structuredContent).toBeDefined();
        expect(typeof result.structuredContent.temperature).toBe('number');
        expect(typeof result.structuredContent.conditions).toBe('string');
        expect(typeof result.structuredContent.humidity).toBe('number');
      });

      await test.step('Tools — progress notifications flow through the proxy SSE stream', async () => {
        // `trigger-long-running-operation` emits one notifications/progress
        // frame per step. We pass a progressToken via `_meta` and rely on
        // the SDK's onprogress callback (wired automatically by the Client
        // when a progressToken is supplied) to collect them. If the proxy
        // buffered SSE chunks, onprogress would never fire before the
        // final result — so observing ≥1 frame proves streaming works.
        const progressEvents = [];
        const result = await sdkClient.callTool(
          {
            name: 'trigger-long-running-operation',
            arguments: { duration: 1, steps: 3 },
          },
          undefined,
          {
            onprogress: (p) => progressEvents.push(p),
          }
        );
        expect(result.isError).not.toBe(true);
        expect(progressEvents.length).toBeGreaterThan(0);
        const last = progressEvents[progressEvents.length - 1];
        expect(typeof last.progress).toBe('number');
        expect(last.total).toBe(3);
      });

      await test.step('Resources — list returns non-empty catalogue', async () => {
        const { resources } = await sdkClient.listResources();
        expect(Array.isArray(resources)).toBe(true);
        expect(resources.length).toBeGreaterThan(0);
        expect(resources[0].uri).toBeTruthy();
      });

      await test.step('Resources — readResource returns contents', async () => {
        const { resources } = await sdkClient.listResources();
        const first = resources[0];
        const read = await sdkClient.readResource({ uri: first.uri });
        expect(Array.isArray(read.contents)).toBe(true);
        expect(read.contents.length).toBeGreaterThan(0);
        const entry = read.contents[0];
        expect(entry.uri).toBe(first.uri);
        const hasPayload = typeof entry.text === 'string' || typeof entry.blob === 'string';
        expect(hasPayload).toBe(true);
      });

      await test.step('Resources — listResourceTemplates returns both dynamic templates', async () => {
        const { resourceTemplates } = await sdkClient.listResourceTemplates();
        expect(Array.isArray(resourceTemplates)).toBe(true);
        const uris = resourceTemplates.map((t) => t.uriTemplate);
        for (const expected of McpFixtures.EVERYTHING_EXPECTED_RESOURCE_TEMPLATES) {
          expect(uris).toContain(expected);
        }
      });

      await test.step('Prompts — list returns expected prompts', async () => {
        const { prompts } = await sdkClient.listPrompts();
        expect(Array.isArray(prompts)).toBe(true);
        const names = prompts.map((p) => p.name);
        for (const expected of ['simple-prompt', 'args-prompt']) {
          expect(names).toContain(expected);
        }
      });

      await test.step('Prompts — getPrompt(simple-prompt) returns a user message', async () => {
        const got = await sdkClient.getPrompt({ name: 'simple-prompt' });
        expect(Array.isArray(got.messages)).toBe(true);
        expect(got.messages.length).toBeGreaterThan(0);
        const roles = got.messages.map((m) => m.role);
        expect(roles).toContain('user');
      });

      await test.step('Prompts — getPrompt(args-prompt) renders argument into text', async () => {
        const got = await sdkClient.getPrompt({
          name: 'args-prompt',
          arguments: { city: ARGS_PROMPT_CITY },
        });
        expect(Array.isArray(got.messages)).toBe(true);
        const flat = got.messages.map((m) => (m.content?.text ? m.content.text : '')).join(' ');
        expect(flat).toContain(ARGS_PROMPT_CITY);
      });

      await test.step('Ping — round-trip succeeds', async () => {
        const pong = await sdkClient.ping();
        // SDK returns the server's empty-object response; assert shape.
        expect(pong).toBeDefined();
        expect(typeof pong).toBe('object');
      });

      // ── Phase 4b: Content-block variants and metadata surfaces ──

      await test.step('Tools — callTool(get-annotated-message) preserves content annotations', async () => {
        const result = await sdkClient.callTool({
          name: 'get-annotated-message',
          arguments: { messageType: 'error', includeImage: false },
        });
        expect(result.isError).not.toBe(true);
        const first = (result.content ?? []).find((b) => b.type === 'text');
        expect(first).toBeDefined();
        // Annotations are metadata attached to content blocks. The proxy must
        // passthrough arbitrary JSON for these to survive the round trip.
        expect(first.annotations).toBeDefined();
        expect(typeof first.annotations.priority).toBe('number');
        expect(Array.isArray(first.annotations.audience)).toBe(true);
      });

      await test.step('Tools — callTool(get-resource-links) returns resource_link blocks', async () => {
        const result = await sdkClient.callTool({
          name: 'get-resource-links',
          arguments: { count: 3 },
        });
        expect(result.isError).not.toBe(true);
        const links = (result.content ?? []).filter((b) => b.type === 'resource_link');
        expect(links.length).toBe(3);
        for (const link of links) {
          expect(link.uri).toMatch(/^demo:\/\/resource\/dynamic\//);
          expect(typeof link.name).toBe('string');
        }
      });

      await test.step('Instructions — getInstructions() returns the upstream string', async () => {
        const instructions = sdkClient.getInstructions();
        // Upstream ships instructions in its `docs/instructions.md`. Any
        // non-empty string is fine — we just need proof the proxy didn't
        // strip the `instructions` field from the initialize response.
        expect(typeof instructions).toBe('string');
        expect(instructions.length).toBeGreaterThan(0);
      });

      // ── Phase 4c: Completions (server-side prompt argument completer) ──

      await test.step('Completions — complete(completable-prompt/department) returns filtered values', async () => {
        const completion = await sdkClient.complete({
          ref: { type: 'ref/prompt', name: 'completable-prompt' },
          argument: { name: 'department', value: 'E' },
        });
        expect(completion).toBeDefined();
        expect(Array.isArray(completion.completion?.values)).toBe(true);
        expect(completion.completion.values).toContain('Engineering');
      });

      // ── Phase 4d: Bidirectional — server requests the client (sampling / elicitation / roots) ──
      //
      // These steps prove the BodhiApp proxy relays server→client requests
      // over the same SSE session that carries tool calls. Each tool below
      // causes the upstream server to send a request back to us; our
      // registered handlers respond; the server then continues tool
      // execution and returns a result containing our stub markers.

      await test.step('Roots — callTool(get-roots-list) reflects client-provided roots', async () => {
        // syncRoots runs ~350ms after notifications/initialized. By the time
        // we reach here, dozens of other calls have elapsed, so the server
        // has the list cached. If the proxy broke bidirectional GET/SSE,
        // syncRoots would hang and this tool would report "no roots".
        const result = await sdkClient.callTool({
          name: 'get-roots-list',
          arguments: {},
        });
        expect(result.isError).not.toBe(true);
        const text = (result.content ?? [])
          .filter((b) => b.type === 'text')
          .map((b) => b.text)
          .join('\n');
        expect(text).toContain(CLIENT_ROOT_URI);
        expect(text).toContain(CLIENT_ROOT_NAME);
      });

      await test.step('Sampling — callTool(trigger-sampling-request) relays to client handler', async () => {
        const result = await sdkClient.callTool({
          name: 'trigger-sampling-request',
          arguments: { prompt: 'bodhiapp-sampling-probe', maxTokens: 32 },
        });
        expect(result.isError).not.toBe(true);
        const text = (result.content ?? [])
          .filter((b) => b.type === 'text')
          .map((b) => b.text)
          .join('\n');
        // Our client handler stamped SAMPLING_STUB_TEXT into the assistant
        // response; the upstream tool then embedded it in the tool result.
        expect(text).toContain(SAMPLING_STUB_TEXT);
      });

      await test.step('Elicitation — callTool(trigger-elicitation-request) relays to client handler', async () => {
        const result = await sdkClient.callTool({
          name: 'trigger-elicitation-request',
          arguments: {},
        });
        expect(result.isError).not.toBe(true);
        const text = (result.content ?? [])
          .filter((b) => b.type === 'text')
          .map((b) => b.text)
          .join('\n');
        // Server formats our accepted payload back into the tool result.
        expect(text).toContain(ELICITATION_STUB_NAME);
      });

      // ── Phase 4e: Streaming notifications (logging + resource updates) ──

      await test.step('Logging — setLoggingLevel + toggle-simulated-logging streams notifications/message', async () => {
        // Lower level so we receive debug/info/notice messages too.
        await sdkClient.setLoggingLevel('debug');
        // Start server's simulated logging loop (emits immediately + every 5s).
        await sdkClient.callTool({ name: 'toggle-simulated-logging', arguments: {} });
        try {
          await waitFor(() => loggingMessages.length >= 1, {
            timeoutMs: 8000,
            intervalMs: 100,
            label: 'logging notifications',
            snapshot: () => ({ received: loggingMessages.length }),
          });
          const first = loggingMessages[0];
          expect(first).toBeDefined();
          expect(typeof first.level).toBe('string');
          // `data` may be string or structured; just require presence.
          expect(first.data).toBeDefined();
        } finally {
          // Toggle off so the interval stops and does not leak between runs.
          await sdkClient.callTool({ name: 'toggle-simulated-logging', arguments: {} });
        }
      });

      await test.step('Subscriptions — subscribeResource + toggle-subscriber-updates streams notifications/resources/updated', async () => {
        const { resources } = await sdkClient.listResources();
        expect(resources.length).toBeGreaterThan(0);
        const targetUri = resources[0].uri;
        await sdkClient.subscribeResource({ uri: targetUri });
        try {
          // Server sends one notification immediately on enable, then every 5s.
          await sdkClient.callTool({ name: 'toggle-subscriber-updates', arguments: {} });
          await waitFor(() => resourceUpdatedEvents.some((e) => e.uri === targetUri), {
            timeoutMs: 8000,
            intervalMs: 100,
            label: 'resources/updated for subscribed uri',
            snapshot: () => resourceUpdatedEvents,
          });
        } finally {
          // Toggle off + unsubscribe so the interval stops cleanly.
          await sdkClient.callTool({ name: 'toggle-subscriber-updates', arguments: {} });
          await sdkClient.unsubscribeResource({ uri: targetUri });
        }
      });

      await test.step('Clean disconnect — client.close() tears down the session', async () => {
        await safeCloseMcpClient(sdkClient);
      });

      // ── Phase 5: Negative case — API token must be rejected ──

      await test.step('Negative — bodhiapp_ API token is rejected (OAuth-only route)', async () => {
        let caught;
        try {
          const attempt = await buildMcpClient({
            serverUrl: sharedServerUrl,
            mcpId,
            token: apiToken,
          });
          // Shouldn't reach here; ensure we tear down if it somehow connected.
          await safeCloseMcpClient(attempt.client);
        } catch (err) {
          caught = err;
        }
        expect(caught).toBeDefined();
        // The transport surfaces HTTP 401/403 either via `StreamableHTTPError`
        // (with a .code property) or via a thrown Error whose message quotes
        // the status. Accept either shape.
        const message = String(caught?.message ?? caught);
        const statusCode = Number(caught?.code ?? Number.NaN);
        const matchesStatus = [401, 403].includes(statusCode) || /\b40[13]\b/.test(message);
        expect(matchesStatus).toBe(true);
      });
    });
  }
);
