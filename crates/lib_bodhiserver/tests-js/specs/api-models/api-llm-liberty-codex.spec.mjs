// Local-only end-to-end smoke test for the LLM Liberty OAuth + openai-codex flow.
//
// Gated on BODHI_E2E_LOCAL because the test consumes a real Codex OAuth envelope
// from `tests-js/data/local/codex.json`. That envelope holds short-lived OAuth
// tokens that cannot be checked into CI; the file is per-developer and gitignored.
//
// What this exercises end-to-end:
//   1. Create an `llm_liberty_oauth` API model from the codex.json envelope
//   2. Fetch models from Codex via the stored credentials (custom slug→ApiModel parser)
//   3. Test connection with gpt-5.2 (or whichever slug is returned first)
//   4. Save the alias
//   5. Open chat, route to /v1/responses via the api_format-aware adapter
//      (agentStore: llm_liberty_oauth + openai-codex → openai-responses piApi),
//      ask a smoke question, assert the answer.
//
// To run:
//   BODHI_E2E_LOCAL=1 npm run test:playwright -- specs/api-models/api-llm-liberty-codex.spec.mjs

import { existsSync, readFileSync } from 'node:fs';
import { dirname, join } from 'node:path';
import { fileURLToPath } from 'node:url';

import { expect, test } from '@/fixtures.mjs';
import { ApiModelFormPage } from '@/pages/ApiModelFormPage.mjs';
import { ChatPage } from '@/pages/ChatPage.mjs';
import { ChatSettingsPage } from '@/pages/ChatSettingsPage.mjs';
import { LoginPage } from '@/pages/LoginPage.mjs';
import { ModelsListPageV2 } from '@/pages/ModelsListPageV2.mjs';
import { getAuthServerConfig, getTestCredentials } from '@/utils/auth-server-client.mjs';

const __filename = fileURLToPath(import.meta.url);
const __dirname = dirname(__filename);
const ENVELOPE_PATH = join(__dirname, '..', '..', 'data', 'local', 'codex.json');
const CODEX_MODEL = 'gpt-5.2';
const SMOKE_QUESTION = 'Reply with exactly: hello bodhi';

function loadEnvelope() {
  expect(
    existsSync(ENVELOPE_PATH),
    `[llm-liberty/codex] tests-js/data/local/codex.json missing. Run ` +
      `\`npx @bodhiapp/llm-liberty@latest login openai-codex\` and save its JSON output to ${ENVELOPE_PATH}.`
  ).toBe(true);

  let parsed;
  try {
    parsed = JSON.parse(readFileSync(ENVELOPE_PATH, 'utf8'));
  } catch (err) {
    expect.fail(`[llm-liberty/codex] codex.json is not valid JSON: ${err.message}`);
  }

  expect(
    parsed.version,
    `[llm-liberty/codex] codex.json version mismatch — re-run \`npx @bodhiapp/llm-liberty@latest login openai-codex\`.`
  ).toBe('1.0.0');
  expect(parsed.provider, `[llm-liberty/codex] codex.json provider mismatch.`).toBe('openai-codex');
  expect(
    parsed.access_token,
    `[llm-liberty/codex] codex.json missing access_token. Re-run llm-liberty login.`
  ).toBeTruthy();
  expect(
    parsed.refresh_token,
    `[llm-liberty/codex] codex.json missing refresh_token. Re-run llm-liberty login.`
  ).toBeTruthy();

  const expiresAt = Number(parsed.expires_at);
  expect(
    Number.isFinite(expiresAt),
    `[llm-liberty/codex] codex.json expires_at="${parsed.expires_at}" is not a number.`
  ).toBe(true);

  const nowSec = Math.floor(Date.now() / 1000);
  const expiredHrs = ((nowSec - expiresAt) / 3600).toFixed(1);
  expect(
    expiresAt,
    `[llm-liberty/codex] access_token in codex.json expired ${expiredHrs}h ago. ` +
      `Re-run \`npx @bodhiapp/llm-liberty@latest login openai-codex\` to refresh.`
  ).toBeGreaterThan(nowSec);

  return parsed;
}

// OFFLINE-ONLY EXCEPTION TO feedback_no_skip_for_missing_env.md.
// This spec drives a live OAuth flow against the OpenAI Codex backend. Tokens
// are short-lived and the refresh token may be revoked by OpenAI at any time,
// so we cannot store a long-lived envelope as a CI secret. Local-only by design.
// Run with: BODHI_E2E_LOCAL=1 npm run test:playwright -- <filter>
test.describe('LLM Liberty OAuth - openai-codex end-to-end (local only)', () => {
  test.skip(
    !process.env.BODHI_E2E_LOCAL,
    'Set BODHI_E2E_LOCAL=1 to run; needs tests-js/data/local/codex.json from `npx @bodhiapp/llm-liberty@latest login openai-codex`. See file-top comment for the no-skip-for-missing-env exception rationale.'
  );

  let envelope;
  let authServerConfig;
  let testCredentials;
  let loginPage;
  let modelsPage;
  let apiModelFormPage;
  let chatPage;
  let chatSettingsPage;

  test.beforeAll(() => {
    envelope = loadEnvelope();
    authServerConfig = getAuthServerConfig();
    testCredentials = getTestCredentials();
  });

  test.beforeEach(async ({ page, sharedServerUrl }) => {
    loginPage = new LoginPage(page, sharedServerUrl, authServerConfig, testCredentials);
    modelsPage = new ModelsListPageV2(page, sharedServerUrl);
    apiModelFormPage = new ApiModelFormPage(page, sharedServerUrl);
    chatPage = new ChatPage(page, sharedServerUrl);
    chatSettingsPage = new ChatSettingsPage(page, sharedServerUrl);
  });

  test('paste codex envelope, fetch models, test connection, save, chat via /v1/responses', async () => {
    await loginPage.performOAuthLogin();

    // 1. Open the New API Model form.
    await modelsPage.navigateToModels();
    await modelsPage.clickNewApiModel();
    await apiModelFormPage.form.waitForFormReady();

    // 2. Switch to LLM Liberty OAuth and paste the codex envelope.
    await apiModelFormPage.form.selectApiFormat('llm_liberty_oauth');
    await apiModelFormPage.form.fillName('LLM Liberty Codex');
    await apiModelFormPage.form.fillLlmLibertyEnvelope(envelope);

    // 3. Fetch models from Codex (custom slug parser: reads models[].slug → ApiModel::OpenAI).
    //    Select gpt-5.2 — the first slug returned by the Codex /models endpoint.
    await apiModelFormPage.form.fetchAndSelectModels([CODEX_MODEL]);

    // 4. Test connection — exercises POST /bodhi/v1/api-models/test with the
    //    LlmLiberty variant carrying the full envelope (create-mode path).
    await apiModelFormPage.form.testConnection();

    // 5. Persist the alias.
    await apiModelFormPage.createModel();

    // 6. Drive the chat UI. The agent store routes llm_liberty_oauth +
    //    provider=openai-codex to /v1/responses via pi-agent-core (openai-responses piApi).
    await chatPage.navigateToChat();
    await chatSettingsPage.selectModel(CODEX_MODEL);

    // Codex rejects max_output_tokens; toggle stays off.
    await chatSettingsPage.setMaxTokensEnabled(false);

    await chatPage.sendMessage(SMOKE_QUESTION);
    await chatPage.waitForResponseComplete();

    const reply = await chatPage.getLastAssistantMessage();
    expect(reply, 'assistant reply must contain "hello bodhi" (case-insensitive)').toMatch(
      /hello bodhi/i
    );
  });
});
