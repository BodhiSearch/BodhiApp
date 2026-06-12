# Extend MCP Proxy E2E Tests — Full Spec Coverage

## Context

The current E2E test (`mcps-mcp-proxy-everything.spec.mjs`) has a single test that proves the Inspector connects via Direct mode and can list tools/resources/prompts/ping. It's a thin happy-path. We need to expand coverage to:

1. **Tools filter enforcement (SECURITY)** — verify restricted tools don't leak through the proxy
2. **Deep tool execution** — call tools with different return types (numbers, images, structured content)
3. **Resource reading** — actually read a resource's content, not just list
4. **Prompt execution** — get a prompt with arguments, verify content
5. **Completions** — verify argument autocomplete works through the proxy
6. **Session lifecycle** — verify DELETE terminates session

**Sampling**: The everything server supports sampling via `trigger-sampling-request` tool. However, the Bodhi proxy (`McpProxyHandler`) implements `ServerHandler` only, not `ClientHandler`. Sampling is a server→client request (upstream MCP server asks the client to do LLM inference). The proxy doesn't forward these back to the downstream client. **Sampling is not testable through the proxy** until we add bidirectional request forwarding. Out of scope for this PR.

## Test Structure

Expand to **3 tests** in the same describe block. All share the same setup pattern (Phase 1-3 from current test).

### Helper: `setupInspectorConnection(page, sharedServerUrl, mcpId, accessToken)`

Extract the Inspector setup steps (Phase 3) into a reusable async function to avoid duplication across tests. This includes:
- route interception for Accept header (Playwright bug workaround)
- navigate to inspector
- switch to Streamable HTTP + Direct + set URL + set Bearer token
- click Connect + wait for Connected

### Test 1: Full protocol journey (EXISTING, expand)

Keep the current test but add deeper assertions:

**Tools — call get-sum with numbers:**
- Select "get-sum" tool (scroll tool list or search)
- Fill `a` = 7, `b` = 13 (number inputs)
- Run Tool → verify result contains "20"

**Tools — call get-tiny-image:**
- Select get-tiny-image → Run Tool → verify "image" appears in result

**Resources — read a resource:**
- Click on `architecture.md` in the resource list → verify content panel shows text (the Inspector auto-reads on click)

**Prompts — get with arguments:**
- Click `args-prompt` in prompt list
- Fill `city` argument = "TestCity"
- Click "Get Prompt" → verify result contains "TestCity"

**Completions:**
- Click `completable-prompt` in prompt list
- Type "Eng" in the `department` argument field → verify autocomplete suggestion "Engineering" appears

### Test 2: Session lifecycle — DELETE terminates session (NEW)

**Phase 1-3:** Same setup pattern

**Verify session works:**
- Connect via Inspector → verify Connected
- Tools tab → List Tools → verify tools appear

**Delete session:**
- Use `page.evaluate()` fetch with DELETE method + Mcp-Session-Id header
- Verify 2xx response

**Verify session invalidated:**
- Use `page.evaluate()` fetch with POST + same Mcp-Session-Id
- Verify 4xx response

Again `page.evaluate` is justified — there's no Inspector UI action for deleting a session.

## Inspector UI Interaction Patterns

From the exploration, here's how to interact with each Inspector feature:

**Select a tool from list:**
```js
await page.getByText('Echo Tool').first().click();  // or tool display name
```

**Fill tool parameters:**
- String params: `page.locator('textarea').first().fill('value')`
- Number params: `page.locator('input[type="number"]').nth(N).fill('7')`
- For specific params, look for the label text then the adjacent input

**Run tool and check result:**
```js
await page.getByRole('button', { name: 'Run Tool' }).click();
await expect(page.getByText('Success')).toBeVisible({ timeout: 10000 });
```

**Read a resource (click in list):**
- Click on resource name → Inspector auto-reads and shows content in right panel

**Get prompt with arguments:**
- Click prompt name → fill argument inputs → click "Get Prompt"
- Prompt arguments use Combobox components (text inputs with autocomplete)

**Completions:**
- Type in a prompt argument Combobox → suggestions appear in dropdown

**Key selectors:**
- Tool result status: text "Success" or "Error"
- Resource content: right panel after clicking resource
- Prompt result: JSON view after "Get Prompt"
- History entries: `text=tools/list`, `text=tools/call`, etc.

## Files to Modify

| File | Change |
|------|--------|
| `tests-js/specs/mcps/mcps-mcp-proxy-everything.spec.mjs` | Expand test 1, add tests 2 and 3 |
| `tests-js/fixtures/mcpFixtures.mjs` | No change needed — constants already exist |

## Verification

```bash
cd crates/lib_bodhiserver_napi
# Must rebuild NAPI if proxy code changed:
# make build.ui-rebuild

# Run everything tests only:
HEADLESS=true npx playwright test --config=playwright.config.mjs --grep "Everything" --project standalone

# Run all tests (regression check):
npm run test:playwright
```
