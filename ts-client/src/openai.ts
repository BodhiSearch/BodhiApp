// Subpath entry point: @bodhiapp/ts-client/openai
// Exposes the OpenAI/Ollama-compatible types (generated from openapi-oai.json)
// as a dedicated module so consumers can write:
//
//   import { CreateChatCompletionRequest, Role } from '@bodhiapp/ts-client/openai';
//
// instead of going through the flat root export (which would collide with the
// BodhiApp management types).
export * from './types-oai';
