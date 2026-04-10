// Subpath entry point: @bodhiapp/ts-client/anthropic
// Exposes the Anthropic API types (generated from the filtered Anthropic OpenAPI spec)
// as a dedicated module so consumers can write:
//
//   import { CreateMessageParams, Message } from '@bodhiapp/ts-client/anthropic';
//
// instead of going through the flat root export.
export * from './types-anthropic';
