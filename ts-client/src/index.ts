// Re-export BodhiApp management API types.
// For OpenAI/Ollama-compatible types, use the subpath import:
//   import { CreateChatCompletionRequest } from '@bodhiapp/ts-client/openai';
export * from './types';

// MSW-compatible schema types for both specs
export type { paths, components } from './openapi-typescript/openapi-schema';
export type {
  paths as pathsOai,
  components as componentsOai,
} from './openapi-typescript/openapi-schema-oai';
export type {
  paths as pathsAnthropic,
  components as componentsAnthropic,
} from './openapi-typescript/openapi-schema-anthropic';
export type {
  paths as pathsGemini,
  components as componentsGemini,
} from './openapi-typescript/openapi-schema-gemini';
