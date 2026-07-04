import type { ModelRouterResponse } from '@bodhiapp/ts-client';
import { describe, expect, it } from 'vitest';

import {
  createMockAnthropicModel,
  createMockApiAlias,
  createMockGeminiModel,
  createMockModelAlias,
  createMockOpenAIModel,
  createMockUserAlias,
} from '@/test-fixtures/models';

import { apiModelChatString, chatModelForAlias, modelId } from './modelAlias';

function makeRouterAlias(): ModelRouterResponse {
  return {
    source: 'model_router',
    id: 'router-1',
    alias: 'smart-fallback',
    targets: [{ alias: 'openai-main', model: 'gpt-4o', enabled: true }],
    strategy: { strategy: 'fallback', cooldown_secs: 30, max_attempts: 0, honor_retry_after: true },
    created_at: '2024-01-01T00:00:00Z',
    updated_at: '2024-01-01T00:00:00Z',
    access: true,
  };
}

describe('modelId', () => {
  it('reads id for OpenAI models', () => {
    expect(modelId(createMockOpenAIModel('gpt-4o'))).toBe('gpt-4o');
  });

  it('reads id for Anthropic models', () => {
    expect(modelId(createMockAnthropicModel('claude-sonnet-4-5'))).toBe('claude-sonnet-4-5');
  });

  it('reads name for Gemini models', () => {
    expect(modelId(createMockGeminiModel('models/gemini-2.5-flash'))).toBe('models/gemini-2.5-flash');
  });
});

describe('chatModelForAlias', () => {
  it('passes through a local GGUF alias verbatim ({repo}:{quant})', () => {
    expect(chatModelForAlias(createMockModelAlias({ alias: 'org/repo-GGUF:Q4_K_M' }))).toBe('org/repo-GGUF:Q4_K_M');
  });

  it('passes through a user alias verbatim', () => {
    expect(chatModelForAlias(createMockUserAlias({ alias: 'my-coder' }))).toBe('my-coder');
  });

  it('passes through a model-router alias verbatim', () => {
    expect(chatModelForAlias(makeRouterAlias())).toBe('smart-fallback');
  });

  it('returns null for an API alias (resolves per-model)', () => {
    expect(chatModelForAlias(createMockApiAlias())).toBeNull();
  });
});

describe('apiModelChatString', () => {
  it('prepends the prefix to the model id', () => {
    const alias = createMockApiAlias({ prefix: 'azure/', models: [createMockOpenAIModel('gpt-4')] });
    expect(apiModelChatString(alias, alias.models[0])).toBe('azure/gpt-4');
  });

  it('uses the bare model id when no prefix is set', () => {
    const alias = createMockApiAlias({ models: [createMockOpenAIModel('gpt-4')] });
    expect(apiModelChatString(alias, alias.models[0])).toBe('gpt-4');
  });

  it('treats a null prefix as empty', () => {
    const alias = createMockApiAlias({ prefix: null, models: [createMockOpenAIModel('gpt-4')] });
    expect(apiModelChatString(alias, alias.models[0])).toBe('gpt-4');
  });

  it('uses the Gemini model name', () => {
    const alias = createMockApiAlias({
      api_format: 'gemini',
      prefix: 'g/',
      models: [createMockGeminiModel('models/gemini-2.5-flash')],
    });
    expect(apiModelChatString(alias, alias.models[0])).toBe('g/models/gemini-2.5-flash');
  });
});
