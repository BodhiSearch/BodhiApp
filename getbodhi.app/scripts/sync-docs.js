#!/usr/bin/env node

/**
 * Documentation Sync System
 * Syncs docs, images, and components from embedded app to website
 *
 * Usage:
 *   npm run sync:docs         # Sync
 *   npm run sync:docs:check   # Check only (dry-run)
 */

import fs from 'fs-extra';
import { glob } from 'glob';
import path from 'path';
import { fileURLToPath } from 'url';

const __dirname = path.dirname(fileURLToPath(import.meta.url));

// Configuration
const BODHI_SRC = path.resolve(__dirname, '../../crates/bodhi');
const WEBSITE_ROOT = path.resolve(__dirname, '..');

const SYNC_TARGETS = [
  {
    name: 'Documentation content',
    source: path.join(BODHI_SRC, 'src/docs'),
    dest: path.join(WEBSITE_ROOT, 'src/docs'),
    patterns: ['**/*.md', '**/_meta.json'],
  },
  {
    name: 'Documentation images',
    source: path.join(BODHI_SRC, 'public/doc-images'),
    dest: path.join(WEBSITE_ROOT, 'public/doc-images'),
    patterns: ['**/*.{jpg,jpeg,png,gif,svg}'],
  },
  {
    name: 'Rendering components',
    source: path.join(BODHI_SRC, 'src/app/docs'),
    dest: path.join(WEBSITE_ROOT, 'src/app/docs'),
    patterns: ['**/*.{tsx,ts,css,html}'],
    ignore: ['**/*.test.{tsx,ts}', '**/__tests__/**', '**/test-utils.ts'],
  },
];

async function syncTarget(target, checkMode = false) {
  const { name, source, dest, patterns, ignore = [] } = target;

  console.log(`${checkMode ? 'Checking' : 'Syncing'} ${name}...`);

  if (!fs.existsSync(source)) {
    console.log(`  Source not found: ${source}`);
    return false;
  }

  // Get source files using glob
  const sourceFiles = await glob(patterns, {
    cwd: source,
    ignore: ignore,
    nodir: true,
  });

  // Get existing dest files
  const destFiles = fs.existsSync(dest)
    ? await glob(patterns, {
        cwd: dest,
        ignore: ignore,
        nodir: true,
      })
    : [];

  const changes = {
    added: [],
    modified: [],
    deleted: [],
  };

  // Check added/modified files
  for (const file of sourceFiles) {
    const srcPath = path.join(source, file);
    const dstPath = path.join(dest, file);

    if (!fs.existsSync(dstPath)) {
      changes.added.push(file);
      if (!checkMode) {
        await fs.copy(srcPath, dstPath);
      }
    } else {
      // Compare file contents
      const srcContent = await fs.readFile(srcPath);
      const dstContent = await fs.readFile(dstPath);
      if (!srcContent.equals(dstContent)) {
        changes.modified.push(file);
        if (!checkMode) {
          await fs.copy(srcPath, dstPath);
        }
      }
    }
  }

  // Check deleted files
  for (const file of destFiles) {
    const srcPath = path.join(source, file);
    const dstPath = path.join(dest, file);

    if (!fs.existsSync(srcPath)) {
      changes.deleted.push(file);
      if (!checkMode) {
        await fs.remove(dstPath);
      }
    }
  }

  const totalChanges = changes.added.length + changes.modified.length + changes.deleted.length;

  if (checkMode) {
    if (totalChanges > 0) {
      console.log(`  ✗ OUT OF SYNC`);
      if (changes.added.length > 0) console.log(`    ${changes.added.length} files to add`);
      if (changes.modified.length > 0) console.log(`    ${changes.modified.length} files to update`);
      if (changes.deleted.length > 0) console.log(`    ${changes.deleted.length} files to remove`);
      return false;
    } else {
      console.log(`  ✓ In sync`);
      return true;
    }
  } else {
    console.log(
      `  ✓ Done (${changes.added.length} added, ${changes.modified.length} updated, ${changes.deleted.length} removed)`
    );
    return true;
  }
}

async function main() {
  const checkMode = process.argv.includes('--check');

  console.log(`\n=== ${checkMode ? 'Checking' : 'Syncing'} Documentation ===\n`);

  let allInSync = true;

  for (const target of SYNC_TARGETS) {
    const result = await syncTarget(target, checkMode);
    if (!result) allInSync = false;
  }

  console.log('');

  if (checkMode) {
    if (!allInSync) {
      console.log('=== Out of sync - run `npm run sync:docs` to sync ===\n');
      process.exit(1);
    } else {
      console.log('=== All documentation in sync ===\n');
      process.exit(0);
    }
  } else {
    console.log('=== Sync complete ===\n');
    process.exit(0);
  }
}

main().catch((err) => {
  console.error('Error:', err.message);
  process.exit(1);
});
