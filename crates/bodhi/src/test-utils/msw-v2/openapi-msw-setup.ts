/**
 * OpenAPI MSW setup and infrastructure for type-safe mocking
 *
 * This file provides typed HTTP handlers using openapi-msw library
 * for full type safety based on the generated OpenAPI schema.
 */
import { createOpenApiHttp } from 'openapi-msw';
import type { paths } from '../generated/openapi-schema';
export type { components } from '../generated/openapi-schema';

// Default internal server error values for error handlers
export const INTERNAL_SERVER_ERROR = {
  code: 'internal_error',
  message: 'Internal server error',
  type: 'internal_server_error',
  status: 500,
} as const;

// Create typed HTTP handler using OpenAPI schema
export const typedHttp = createOpenApiHttp<paths>();

// Re-export MSW HttpResponse for convenience
export { HttpResponse } from 'msw';
