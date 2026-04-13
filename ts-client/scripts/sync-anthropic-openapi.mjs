#!/usr/bin/env node

import { readFileSync, writeFileSync, copyFileSync, unlinkSync } from 'node:fs';
import { execSync } from 'node:child_process';
import { fileURLToPath } from 'node:url';
import { dirname, join } from 'node:path';

const __dirname = dirname(fileURLToPath(import.meta.url));
const TS_CLIENT_DIR = join(__dirname, '..');
const ROUTES_APP_RESOURCES = join(TS_CLIENT_DIR, '..', 'crates', 'routes_app', 'resources');

const STATS_YML_URL =
  'https://raw.githubusercontent.com/anthropics/anthropic-sdk-typescript/refs/heads/main/.stats.yml';
const META_FILE = join(TS_CLIENT_DIR, '.anthropic-openapi-meta.json');
const FILTER_FILE = join(TS_CLIENT_DIR, 'anthropic-openapi-filter.yaml');
const OUTPUT_FILE = join(TS_CLIENT_DIR, 'openapi-anthropic.json');
const ROUTES_APP_DEST = join(ROUTES_APP_RESOURCES, 'openapi-anthropic.json');

function parseSimpleYaml(text) {
  const result = {};
  for (const line of text.split('\n')) {
    const match = line.match(/^(\w+):\s*(.+)$/);
    if (match) {
      result[match[1]] = match[2].trim();
    }
  }
  return result;
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

  console.log('Fetching Anthropic SDK .stats.yml...');
  const statsResp = await fetch(STATS_YML_URL);
  if (!statsResp.ok) {
    throw new Error(`Failed to fetch .stats.yml: ${statsResp.status} ${statsResp.statusText}`);
  }
  const statsText = await statsResp.text();
  const stats = parseSimpleYaml(statsText);

  const remoteHash = stats.openapi_spec_hash;
  const remoteUrl = stats.openapi_spec_url;

  if (!remoteHash || !remoteUrl) {
    throw new Error(
      `Could not parse openapi_spec_hash / openapi_spec_url from .stats.yml:\n${statsText}`
    );
  }

  console.log(`  Remote hash: ${remoteHash}`);
  console.log(`  Remote URL:  ${remoteUrl}`);

  const meta = readMeta();
  if (!forceFlag && meta.openapi_spec_hash === remoteHash) {
    console.log('\nAnthropic OpenAPI spec is up to date. Nothing to do.');
    return;
  }

  if (forceFlag) {
    console.log('\n--force flag set, re-downloading regardless of hash.');
  } else {
    console.log(`\nHash changed: ${meta.openapi_spec_hash || '(none)'} -> ${remoteHash}`);
  }

  console.log('Downloading full Anthropic OpenAPI spec...');
  const specResp = await fetch(remoteUrl);
  if (!specResp.ok) {
    throw new Error(`Failed to download spec: ${specResp.status} ${specResp.statusText}`);
  }
  const specText = await specResp.text();

  const tmpInput = join(TS_CLIENT_DIR, '.anthropic-openapi-full.yml');
  writeFileSync(tmpInput, specText);
  console.log(`  Downloaded ${(specText.length / 1024).toFixed(1)} KB`);

  console.log('Filtering spec with openapi-format...');
  execSync(
    `npx openapi-format "${tmpInput}" -o "${OUTPUT_FILE}" --filterFile "${FILTER_FILE}" --json`,
    { cwd: TS_CLIENT_DIR, stdio: 'inherit' }
  );

  const filteredSize = readFileSync(OUTPUT_FILE, 'utf-8').length;
  console.log(`  Filtered spec: ${(filteredSize / 1024).toFixed(1)} KB`);

  console.log(`Copying to ${ROUTES_APP_DEST}...`);
  copyFileSync(OUTPUT_FILE, ROUTES_APP_DEST);

  writeMeta({ openapi_spec_hash: remoteHash, openapi_spec_url: remoteUrl });
  console.log('Updated .anthropic-openapi-meta.json');

  // Clean up temp file
  unlinkSync(tmpInput);

  console.log('\nGenerating TypeScript types...');
  execSync('npm run generate:types-anthropic', { cwd: TS_CLIENT_DIR, stdio: 'inherit' });
  execSync('npm run generate:msw-types-anthropic', { cwd: TS_CLIENT_DIR, stdio: 'inherit' });

  console.log('\nDone! Anthropic OpenAPI spec updated and types regenerated.');
}

main().catch((err) => {
  console.error('\nError:', err.message);
  process.exit(1);
});
