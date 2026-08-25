#!/usr/bin/env node

import { existsSync, mkdirSync, rmSync, readFileSync, writeFileSync } from 'fs';
import { join, dirname } from 'path';
import { fileURLToPath } from 'url';
import { execSync } from 'child_process';

const __dirname = dirname(fileURLToPath(import.meta.url));
const REPO = 'BodhiSearch/bodhi-browser';
const EXTENSION_DIR = join(__dirname, '..', 'tests-js', 'extension');
const EXTENSION_PATH = join(EXTENSION_DIR, 'bodhi-browser-ext');
const VERSION_FILE = join(EXTENSION_DIR, 'version.txt');

async function getLatestExtensionRelease() {
  const releases = execSync(`gh release list --repo ${REPO} --limit 20`, { encoding: 'utf8' });
  const extensionReleases = releases.split('\n').filter(line => line.startsWith('bodhi-browser-ext'));

  if (extensionReleases.length === 0) {
    throw new Error('No bodhi-browser-ext releases found');
  }

  const latestRelease = extensionReleases[0];
  const versionMatch = latestRelease.match(/v([\d.]+)/);
  const version = versionMatch ? versionMatch[1] : null;
  const tag = `bodhi-browser-ext/v${version}`;

  return { version, tag };
}

function getCurrentVersion() {
  if (existsSync(VERSION_FILE)) {
    const versionText = readFileSync(VERSION_FILE, 'utf8').trim();
    const match = versionText.match(/v([\d.]+)/);
    return match ? match[1] : null;
  }
  return null;
}

async function downloadExtension(force = false) {
  if (!existsSync(EXTENSION_DIR)) {
    mkdirSync(EXTENSION_DIR, { recursive: true });
  }

  const extensionExists = existsSync(EXTENSION_PATH);
  const currentVersion = getCurrentVersion();

  if (extensionExists && currentVersion && !force) {
    console.log(`✓ Extension already downloaded (v${currentVersion})`);
    console.log('  Use --force to check for updates');
    return;
  }

  console.log('Checking for latest extension version on GitHub...');
  const { version, tag } = await getLatestExtensionRelease();

  if (currentVersion === version && extensionExists && !force) {
    console.log(`✓ Already have latest version (v${version})`);
    return;
  }

  console.log(`Downloading extension v${version} from GitHub...`);

  if (existsSync(EXTENSION_PATH)) {
    console.log('Removing old extension...');
    rmSync(EXTENSION_PATH, { recursive: true, force: true });
  }

  const tempZip = join(EXTENSION_DIR, 'bodhi-browser-ext.zip');
  execSync(`gh release download ${tag} --repo ${REPO} --pattern bodhi-browser-ext.zip --dir ${EXTENSION_DIR}`);

  mkdirSync(EXTENSION_PATH, { recursive: true });
  execSync(`cd "${EXTENSION_PATH}" && unzip -qo "${tempZip}"`);

  rmSync(tempZip);

  writeFileSync(VERSION_FILE, `v${version}\n`);

  if (existsSync(join(EXTENSION_PATH, 'manifest.json'))) {
    console.log(`✓ Extension v${version} downloaded and extracted successfully`);
  } else {
    console.error('✗ Failed to extract extension properly');
    process.exit(1);
  }
}

const force = process.argv.includes('--force');

try {
  await downloadExtension(force);
} catch (error) {
  console.error('Error downloading extension:', error.message);
  process.exit(1);
}