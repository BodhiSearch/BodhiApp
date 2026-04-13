import { ApiModelFixtures } from '@/fixtures/apiModelFixtures.mjs';
import { TokenFixtures } from '@/fixtures/tokenFixtures.mjs';

/**
 * Register an API model via the UI form.
 * After calling, the browser is on the models list page.
 *
 * @param {ModelsListPage} modelsPage
 * @param {ApiModelFormPage} formPage
 * @param {string} apiKey
 * @param {object} [formatConfig] - Format config from ApiModelFixtures.API_FORMATS (defaults to openai)
 * @returns {{ modelId: string, modelName: string }}
 */
export async function registerApiModelViaUI(modelsPage, formPage, apiKey, formatConfig = null) {
  const config = formatConfig || ApiModelFixtures.API_FORMATS.openai;
  const modelData = ApiModelFixtures.createModelDataForFormat(
    Object.keys(ApiModelFixtures.API_FORMATS).find(
      (k) => ApiModelFixtures.API_FORMATS[k] === config
    ) || 'openai'
  );
  await modelsPage.navigateToModels();
  await modelsPage.clickNewApiModel();
  await formPage.form.waitForFormReady();
  await formPage.form.selectApiFormat(config.format);
  // If the format defines extra headers/body defaults, verify the form pre-populates them.
  if (config.extraHeaders || config.extraBody) {
    await formPage.form.expectExtrasPrefilledFor(config);
  }
  await formPage.form.fillBasicInfo(apiKey, modelData.baseUrl);
  await formPage.form.fetchAndSelectModels([config.model]);
  await formPage.form.testConnection();
  const modelId = await formPage.createModelAndCaptureId();
  return { modelId, modelName: config.model };
}

/**
 * Create a BodhiApp API token via the tokens UI.
 *
 * @param {TokensPage} tokensPage
 * @param {import('@playwright/test').Page} page
 * @param {string} name - Token name
 * @param {string} scope - Token scope (e.g. 'scope_token_user')
 * @returns {Promise<string>} The minted token (bodhiapp_...)
 */
export async function mintApiToken(tokensPage, page, name, scope) {
  await tokensPage.navigateToTokens();
  await tokensPage.createToken(name, scope);
  await TokenFixtures.mockClipboard(page);
  const token = await tokensPage.copyTokenFromDialog();
  await tokensPage.closeTokenDialog();
  return token;
}

/**
 * Make a POST request to a BodhiApp API endpoint using a Bearer token.
 *
 * @param {string} serverUrl - BodhiApp server base URL
 * @param {string} token - Bearer token (bodhiapp_... or OAuth access token)
 * @param {string} endpoint - API endpoint path (e.g. '/v1/chat/completions')
 * @param {object} body - Request body
 * @returns {{ resp: Response, data: any }}
 */
export async function fetchWithBearer(serverUrl, token, endpoint, body) {
  const resp = await fetch(`${serverUrl}${endpoint}`, {
    method: 'POST',
    headers: {
      'Content-Type': 'application/json',
      Authorization: `Bearer ${token}`,
    },
    body: JSON.stringify(body),
  });
  const data = await resp.json();
  return { resp, data };
}

/**
 * Make a POST request and read the SSE response body, returning the first parsed data chunk.
 * Used for Gemini :streamGenerateContent which returns text/event-stream.
 *
 * @param {string} serverUrl - BodhiApp server base URL
 * @param {string} token - Bearer token
 * @param {string} endpoint - API endpoint path
 * @param {object} body - Request body
 * @returns {{ resp: Response, data: any }} data.chunks holds all parsed SSE
 *   chunks (one per `data:` line). For convenience, `data` also mirrors the
 *   first chunk's top-level fields so non-streaming extractors still work;
 *   streaming-aware extractors should read `data.chunks`.
 */
export async function fetchWithBearerSSE(serverUrl, token, endpoint, body) {
  const resp = await fetch(`${serverUrl}${endpoint}`, {
    method: 'POST',
    headers: {
      'Content-Type': 'application/json',
      Authorization: `Bearer ${token}`,
    },
    body: JSON.stringify(body),
  });
  const text = await resp.text();
  // Upstream stream shape depends on whether `?alt=sse` was passed upstream:
  //  - without (default): JSON array `[{...},{...}]` of stream chunks
  //  - with:             SSE lines `data: {...}\r\n\r\n`
  // Parse both transparently.
  let chunks = [];
  const trimmed = text.trim();
  if (trimmed.startsWith('[')) {
    try {
      const arr = JSON.parse(trimmed);
      if (Array.isArray(arr)) chunks = arr;
    } catch {
      /* fall through to SSE parse */
    }
  }
  if (chunks.length === 0) {
    for (const line of text.split('\n')) {
      if (!line.startsWith('data:')) continue;
      const payload = line.slice('data:'.length).trim();
      if (!payload || payload === '[DONE]') continue;
      try {
        chunks.push(JSON.parse(payload));
      } catch {
        // skip malformed chunk
      }
    }
  }
  const data = chunks[0] ? { ...chunks[0], chunks } : { chunks };
  return { resp, data };
}

/**
 * Make a POST request using x-api-key header (Anthropic SDK auth style).
 * anthropic_auth_middleware on the anthropic_apis route group rewrites
 * x-api-key to Authorization: Bearer before passing to api_auth_middleware.
 *
 * @param {string} serverUrl - BodhiApp server base URL
 * @param {string} token - BodhiApp API token (bodhiapp_...)
 * @param {string} endpoint - API endpoint path (e.g. '/anthropic/v1/messages')
 * @param {object} body - Request body
 * @returns {{ resp: Response, data: any }}
 */
export async function fetchWithApiKey(serverUrl, token, endpoint, body) {
  const resp = await fetch(`${serverUrl}${endpoint}`, {
    method: 'POST',
    headers: {
      'Content-Type': 'application/json',
      'x-api-key': token,
    },
    body: JSON.stringify(body),
  });
  const data = await resp.json();
  return { resp, data };
}
