// Subpath entry point: @bodhiapp/ts-client/gemini
// Exposes the Gemini API types (generated from the filtered Gemini OpenAPI spec)
// as a dedicated module so consumers can write:
//
//   import { GenerateContentRequest, GenerateContentResponse } from '@bodhiapp/ts-client/gemini';
//
// instead of going through the flat root export.
export * from './types-gemini';
