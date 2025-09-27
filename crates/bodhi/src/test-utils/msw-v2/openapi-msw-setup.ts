/**
 * OpenAPI MSW setup and infrastructure for type-safe mocking
 *
 * This file provides typed HTTP handlers using openapi-msw library
 * for full type safety based on the generated OpenAPI schema.
 */
import { createOpenApiHttp } from 'openapi-msw';
import type { paths } from '../generated/openapi-schema';

// Create typed HTTP handler using OpenAPI schema
export const typedHttp = createOpenApiHttp<paths>();

// Re-export MSW HttpResponse for convenience
export { HttpResponse } from 'msw';
