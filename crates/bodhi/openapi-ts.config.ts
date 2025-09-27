import { defineConfig } from 'openapi-typescript';

export default defineConfig({
  input: '../../openapi.json',
  output: 'src/test-utils/generated/openapi-schema.ts',
  exportType: true, // Export as type-only
});
