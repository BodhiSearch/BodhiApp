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
  // Get latest bodhi-browser-ext release using gh CLI
  const releases = execSync(`gh release list --repo ${REPO} --limit 20`, { encoding: 'utf8' });
  const extensionReleases = releases.split('\n').filter(line => line.startsWith('bodhi-browser-ext'));

  if (extensionReleases.length === 0) {
    throw new Error('No bodhi-browser-ext releases found');
  }

  // Parse first (latest) release
  const latestRelease = extensionReleases[0];
  const versionMatch = latestRelease.match(/v([\d.]+)/);
  const version = versionMatch ? versionMatch[1] : null;
  const tag = `bodhi-browser-ext/v${version}`;

  return { version, tag };
}

function getCurrentVersion() {
  if (existsSync(VERSION_FILE)) {
    const versionText = readFileSync(VERSION_FILE, 'utf8').trim();
    // Extract version number from v1.2.3 format
    const match = versionText.match(/v([\d.]+)/);
    return match ? match[1] : null;
  }
  return null;
}

async function downloadExtension(force = false) {
  // Create extension directory if it doesn't exist
  if (!existsSync(EXTENSION_DIR)) {
    mkdirSync(EXTENSION_DIR, { recursive: true });
  }

  // Check if extension already exists
  const extensionExists = existsSync(EXTENSION_PATH);
  const currentVersion = getCurrentVersion();

  if (extensionExists && currentVersion && !force) {
    console.log(`✓ Extension already downloaded (v${currentVersion})`);
    console.log('  Use --force to check for updates');
    return;
  }

  console.log('Checking for latest extension version on GitHub...');
  const { version, tag } = await getLatestExtensionRelease();

  // Check if we already have this version
  if (currentVersion === version && extensionExists && !force) {
    console.log(`✓ Already have latest version (v${version})`);
    return;
  }

  // Download new version
  console.log(`Downloading extension v${version} from GitHub...`);

  // Clean existing extension directory if it exists
  if (existsSync(EXTENSION_PATH)) {
    console.log('Removing old extension...');
    rmSync(EXTENSION_PATH, { recursive: true, force: true });
  }

  // Download the extension zip to temp location
  const tempZip = join(EXTENSION_DIR, 'bodhi-browser-ext.zip');
  execSync(`gh release download ${tag} --repo ${REPO} --pattern bodhi-browser-ext.zip --dir ${EXTENSION_DIR}`);

  // Create extension directory and extract
  mkdirSync(EXTENSION_PATH, { recursive: true });
  execSync(`cd "${EXTENSION_PATH}" && unzip -qo "${tempZip}"`);

  // Clean up zip file
  rmSync(tempZip);

  // Update version file
  writeFileSync(VERSION_FILE, `v${version}\n`);

  // Verify extraction by checking for manifest.json
  if (existsSync(join(EXTENSION_PATH, 'manifest.json'))) {
    console.log(`✓ Extension v${version} downloaded and extracted successfully`);
  } else {
    console.error('✗ Failed to extract extension properly');
    process.exit(1);
  }
}

// Main execution
const force = process.argv.includes('--force');

try {
  await downloadExtension(force);
} catch (error) {
  console.error('Error downloading extension:', error.message);
  process.exit(1);
}