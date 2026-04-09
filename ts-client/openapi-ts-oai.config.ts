import { defineConfig } from '@hey-api/openapi-ts';

export default defineConfig({
  input: '../openapi-oai.json',
  output: 'src/types-oai',
  plugins: ['@hey-api/typescript'],
});
