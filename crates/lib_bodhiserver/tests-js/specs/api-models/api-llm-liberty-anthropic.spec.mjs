// Local-only end-to-end smoke test for the LLM Liberty OAuth + Anthropic flow.
//
// Gated on BODHI_E2E_LOCAL because the test consumes a real `npx
// @bodhiapp/llm-liberty@latest login anthropic` envelope from
// `tests-js/data/local/anthropic.json`. That envelope holds short-lived
// (~8h) OAuth tokens that cannot be checked into CI; the file is per-developer
// and gitignored.
//
// What this exercises end-to-end:
//   1. Create an `llm_liberty_oauth` API model from the envelope JSON
//   2. Fetch models from Anthropic via the stored credentials
//   3. Test connection with claude-haiku-4-5-20251001
//   4. Save the alias
//   5. Open chat, route to /anthropic/v1/messages via the api_format-aware
//      adapter, ask a smoke question, assert the answer.
//
// To run:
//   BODHI_E2E_LOCAL=1 npm run test:playwright -- specs/api-models/api-llm-liberty-anthropic.spec.mjs

import { existsSync, readFileSync } from 'node:fs';
import { dirname, join } from 'node:path';
import { fileURLToPath } from 'node:url';

import { expect, test } from '@/fixtures.mjs';
import { ApiModelFormPage } from '@/pages/ApiModelFormPage.mjs';
import { ChatPage } from '@/pages/ChatPage.mjs';
import { ChatSettingsPage } from '@/pages/ChatSettingsPage.mjs';
import { LoginPage } from '@/pages/LoginPage.mjs';
import { ModelsListPage } from '@/pages/ModelsListPage.mjs';
import { getAuthServerConfig, getTestCredentials } from '@/utils/auth-server-client.mjs';

const __filename = fileURLToPath(import.meta.url);
const __dirname = dirname(__filename);
const ENVELOPE_PATH = join(__dirname, '..', '..', 'data', 'local', 'anthropic.json');
const HAIKU_MODEL = 'claude-haiku-4-5-20251001';
const SMOKE_QUESTION = 'what day comes after monday, answer in one word';

function loadEnvelope() {
  if (!existsSync(ENVELOPE_PATH)) {
    throw new Error(
      `[llm-liberty/anthropic] tests-js/data/local/anthropic.json missing. ` +
        `Run \`npx @bodhiapp/llm-liberty@latest login anthropic\` and save its JSON output to ` +
        `${ENVELOPE_PATH}.`
    );
  }
  let parsed;
  try {
    parsed = JSON.parse(readFileSync(ENVELOPE_PATH, 'utf8'));
  } catch (err) {
    throw new Error(`[llm-liberty/anthropic] anthropic.json is not valid JSON: ${err.message}`);
  }

  if (parsed.version !== '1.0.0') {
    throw new Error(
      `[llm-liberty/anthropic] anthropic.json version="${parsed.version}", expected "1.0.0". ` +
        `Re-run \`npx @bodhiapp/llm-liberty@latest login anthropic\`.`
    );
  }
  if (parsed.provider !== 'anthropic') {
    throw new Error(
      `[llm-liberty/anthropic] anthropic.json provider="${parsed.provider}", expected "anthropic".`
    );
  }
  if (!parsed.access_token || !parsed.refresh_token) {
    throw new Error(
      `[llm-liberty/anthropic] anthropic.json missing access_token/refresh_token. Re-run llm-liberty login.`
    );
  }

  const expiresAt = Number(parsed.expires_at);
  if (!Number.isFinite(expiresAt)) {
    throw new Error(
      `[llm-liberty/anthropic] anthropic.json expires_at="${parsed.expires_at}" is not a number.`
    );
  }
  const nowSec = Math.floor(Date.now() / 1000);
  if (expiresAt <= nowSec) {
    const expiredHrs = ((nowSec - expiresAt) / 3600).toFixed(1);
    throw new Error(
      `[llm-liberty/anthropic] access_token in anthropic.json expired ${expiredHrs}h ago. ` +
        `Re-run \`npx @bodhiapp/llm-liberty@latest login anthropic\` to refresh.`
    );
  }

  return parsed;
}

// OFFLINE-ONLY EXCEPTION TO feedback_no_skip_for_missing_env.md.
// This spec drives a live OAuth flow against Anthropic. Tokens are short-lived
// (~8h) and the refresh token may be revoked when flagged as third-party usage,
// so we cannot store a long-lived envelope as a CI secret. Local-only by design.
// Run with: BODHI_E2E_LOCAL=1 npm run test:playwright -- <filter>
test.describe('LLM Liberty OAuth - Anthropic end-to-end (local only)', () => {
  test.skip(
    !process.env.BODHI_E2E_LOCAL,
    'Set BODHI_E2E_LOCAL=1 to run; needs tests-js/data/local/anthropic.json from `npx @bodhiapp/llm-liberty@latest login anthropic`. See file-top comment for the no-skip-for-missing-env exception rationale.'
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
    modelsPage = new ModelsListPage(page, sharedServerUrl);
    apiModelFormPage = new ApiModelFormPage(page, sharedServerUrl);
    chatPage = new ChatPage(page, sharedServerUrl);
    chatSettingsPage = new ChatSettingsPage(page, sharedServerUrl);
  });

  test('paste envelope, fetch models, test connection, save, chat with haiku', async () => {
    await loginPage.performOAuthLogin();

    // 1. Open the New API Model form.
    await modelsPage.navigateToModels();
    await modelsPage.clickNewApiModel();
    await apiModelFormPage.form.waitForFormReady();

    // 2. Switch to LLM Liberty OAuth and paste the envelope.
    await apiModelFormPage.form.selectApiFormat('llm_liberty_oauth');
    await apiModelFormPage.form.fillName('LLM Liberty Anthropic');
    await apiModelFormPage.form.fillLlmLibertyEnvelope(envelope);

    // 3. Fetch models from Anthropic via the stored OAuth token, then pick Haiku.
    await apiModelFormPage.form.fetchAndSelectModels([HAIKU_MODEL]);

    // 4. Test connection — exercises POST /bodhi/v1/api-models/test with the
    //    LlmLiberty variant carrying the full envelope (create-mode path).
    await apiModelFormPage.form.testConnection();

    // 5. Persist the alias.
    await apiModelFormPage.createModel();

    // 6. Drive the chat UI. Routes llm_liberty_oauth + provider=anthropic to
    //    /anthropic/v1/messages via pi-agent-core.
    await chatPage.navigateToChat();
    await chatSettingsPage.selectModel(HAIKU_MODEL);
    await chatSettingsPage.setMaxTokensEnabled(true);

    await chatPage.sendMessage(SMOKE_QUESTION);
    await chatPage.waitForResponseComplete();

    const reply = await chatPage.getLastAssistantMessage();
    expect(reply, 'assistant reply must mention Tuesday (case-insensitive)').toMatch(/tuesday/i);
  });
});
