import { defineConfig } from '@hey-api/openapi-ts';

export default defineConfig({
  input: 'openapi-gemini.json',
  output: 'src/types-gemini',
  plugins: ['@hey-api/typescript'],
});
