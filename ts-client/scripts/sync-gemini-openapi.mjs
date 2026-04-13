#!/usr/bin/env node

import { readFileSync, writeFileSync, copyFileSync, unlinkSync } from 'node:fs';
import { execSync } from 'node:child_process';
import { fileURLToPath } from 'node:url';
import { dirname, join } from 'node:path';
import { createHash } from 'node:crypto';

const __dirname = dirname(fileURLToPath(import.meta.url));
const TS_CLIENT_DIR = join(__dirname, '..');
const ROUTES_APP_RESOURCES = join(TS_CLIENT_DIR, '..', 'crates', 'routes_app', 'resources');

const META_FILE = join(TS_CLIENT_DIR, '.gemini-openapi-meta.json');
const FILTER_FILE = join(TS_CLIENT_DIR, 'gemini-openapi-filter.yaml');
const OUTPUT_FILE = join(TS_CLIENT_DIR, 'openapi-gemini.json');
const ROUTES_APP_DEST = join(ROUTES_APP_RESOURCES, 'openapi-gemini.json');

function readEnv() {
  try {
    const envText = readFileSync(join(TS_CLIENT_DIR, '.env'), 'utf-8');
    for (const line of envText.split('\n')) {
      const match = line.match(/^GEMINI_API_KEY=(.+)$/);
      if (match) return match[1].trim();
    }
  } catch {
    // fall through
  }
  return process.env.GEMINI_API_KEY || '';
}

function readMeta() {
  try {
    return JSON.parse(readFileSync(META_FILE, 'utf-8'));
  } catch {
    return { openapi_spec_hash: '', openapi_spec_url: '' };
  }
}

function writeMeta(meta) {
  writeFileSync(META_FILE, JSON.stringify(meta, null, 2) + '\n');
}

async function main() {
  const forceFlag = process.argv.includes('--force');

  const apiKey = readEnv();
  if (!apiKey) {
    throw new Error(
      'GEMINI_API_KEY not found. Set it in ts-client/.env or as an environment variable.'
    );
  }

  const specUrl = `https://generativelanguage.googleapis.com/$discovery/OPENAPI3_0?version=v1beta&key=${apiKey}`;

  console.log('Fetching Gemini OpenAPI spec...');
  const specResp = await fetch(specUrl);
  if (!specResp.ok) {
    throw new Error(`Failed to fetch Gemini spec: ${specResp.status} ${specResp.statusText}`);
  }
  const specText = await specResp.text();
  console.log(`  Downloaded ${(specText.length / 1024).toFixed(1)} KB`);

  const remoteHash = createHash('sha256').update(specText).digest('hex');
  // Strip API key from the stored URL so it's safe to commit
  const safeUrl = `https://generativelanguage.googleapis.com/$discovery/OPENAPI3_0?version=v1beta`;

  console.log(`  Content hash: ${remoteHash}`);

  const meta = readMeta();
  if (!forceFlag && meta.openapi_spec_hash === remoteHash) {
    console.log('\nGemini OpenAPI spec is up to date. Nothing to do.');
    return;
  }

  if (forceFlag) {
    console.log('\n--force flag set, re-filtering regardless of hash.');
  } else {
    console.log(`\nHash changed: ${meta.openapi_spec_hash || '(none)'} -> ${remoteHash}`);
  }

  const tmpInput = join(TS_CLIENT_DIR, '.gemini-openapi-full.json');
  writeFileSync(tmpInput, specText);

  console.log('Filtering spec with openapi-format...');
  execSync(
    `npx openapi-format "${tmpInput}" -o "${OUTPUT_FILE}" --filterFile "${FILTER_FILE}" --json`,
    { cwd: TS_CLIENT_DIR, stdio: 'inherit' }
  );

  // Strip baseModelId from Model schema — Google's live endpoint never returns it (https://discuss.ai.google.dev/t/55268).
  const specAfterFilter = JSON.parse(readFileSync(OUTPUT_FILE, 'utf-8'));
  const modelSchema = specAfterFilter.components?.schemas?.Model;
  if (modelSchema) {
    if (modelSchema.properties?.baseModelId) {
      delete modelSchema.properties.baseModelId;
      console.log('Removed Model.baseModelId (Google live endpoint omits it; tracking https://discuss.ai.google.dev/t/55268)');
    }
    if (Array.isArray(modelSchema.required) && modelSchema.required.includes('baseModelId')) {
      modelSchema.required = modelSchema.required.filter((f) => f !== 'baseModelId');
      console.log('Removed baseModelId from Model.required (Google live endpoint omits it; tracking https://discuss.ai.google.dev/t/55268)');
    }
    writeFileSync(OUTPUT_FILE, JSON.stringify(specAfterFilter, null, 2) + '\n');
  }

  // The Gemini spec has dangling $ref pointers for some enum schemas that Google omitted.
  // Patch the filtered spec to inject stubs so tooling can resolve all refs.
  // Note: this runs after filtering, so schemas stripped as unused will be re-injected here if
  // they are still referenced — that is intentional.
  console.log('Patching missing schema stubs...');
  const filteredSpec = JSON.parse(readFileSync(OUTPUT_FILE, 'utf-8'));
  const schemas = filteredSpec.components?.schemas ?? {};
  const specText2 = JSON.stringify(filteredSpec);
  const allRefs = [...new Set([...specText2.matchAll(/"#\/components\/schemas\/(\w+)"/g)].map((m) => m[1]))];
  // Known stubs — warn if Google has started shipping the real schema so we can remove the stub.
  const KNOWN_STUBS = ['ToolType', 'MediaResolution'];
  for (const name of KNOWN_STUBS) {
    if (name in schemas) {
      console.warn(`[sync-gemini] Schema ${name} now ships upstream — remove stub injection`);
    }
  }
  const missing = allRefs.filter((name) => !(name in schemas));
  if (missing.length > 0) {
    console.log(`  Injecting stubs for: ${missing.join(', ')}`);
    for (const name of missing) {
      schemas[name] = { type: 'string', description: `Stub for ${name} (missing in upstream spec)` };
    }
    if (!filteredSpec.components) filteredSpec.components = {};
    filteredSpec.components.schemas = schemas;
    writeFileSync(OUTPUT_FILE, JSON.stringify(filteredSpec, null, 2) + '\n');
  }

  const filteredSize = readFileSync(OUTPUT_FILE, 'utf-8').length;
  console.log(`  Filtered spec: ${(filteredSize / 1024).toFixed(1)} KB`);

  console.log(`Copying to ${ROUTES_APP_DEST}...`);
  copyFileSync(OUTPUT_FILE, ROUTES_APP_DEST);

  writeMeta({ openapi_spec_hash: remoteHash, openapi_spec_url: safeUrl });
  console.log('Updated .gemini-openapi-meta.json');

  // Clean up temp file
  unlinkSync(tmpInput);

  console.log('\nGenerating TypeScript types...');
  execSync('npm run generate:types-gemini', { cwd: TS_CLIENT_DIR, stdio: 'inherit' });
  execSync('npm run generate:msw-types-gemini', { cwd: TS_CLIENT_DIR, stdio: 'inherit' });

  console.log('\nDone! Gemini OpenAPI spec updated and types regenerated.');
}

main().catch((err) => {
  console.error('\nError:', err.message);
  process.exit(1);
});
