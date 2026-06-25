import type { AliasResponse, ApiAliasResponse } from '@bodhiapp/ts-client';

import { isApiAlias } from '@/lib/utils';

type ApiModel = ApiAliasResponse['models'][number];

/** Display id for an API model: OpenAI/Anthropic expose `id`, Gemini exposes `name`. */
export function modelId(m: ApiModel): string {
  if ('id' in m && m.id) return m.id;
  if ('name' in m && m.name) return m.name;
  return JSON.stringify(m);
}

/**
 * The `model` string the chat endpoint needs to route to a specific model of an
 * API alias. Mirrors the backend `ApiAlias::matchable_models()`: prefix + model id.
 */
export function apiModelChatString(alias: ApiAliasResponse, model: ApiModel): string {
  return `${alias.prefix ?? ''}${modelId(model)}`;
}

/**
 * The `model` string that selects this alias in the chat endpoint, mirroring
 * backend `DataService::find_alias`. user/model/model_router resolve by their
 * `alias` field verbatim (local GGUF aliases are pre-derived as `{repo}:{quant}`).
 * Returns null for API aliases, which resolve per-model — use `apiModelChatString`.
 */
export function chatModelForAlias(alias: AliasResponse): string | null {
  if (isApiAlias(alias)) return null;
  return alias.alias;
}
