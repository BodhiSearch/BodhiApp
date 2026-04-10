import { defineConfig } from '@hey-api/openapi-ts';

export default defineConfig({
  input: 'openapi-anthropic.json',
  output: 'src/types-anthropic',
  plugins: ['@hey-api/typescript'],
});
