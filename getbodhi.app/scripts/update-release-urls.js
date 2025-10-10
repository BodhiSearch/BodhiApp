#!/usr/bin/env node

/**
 * Update .env.release_urls with latest GitHub release download URLs
 *
 * Usage:
 *   npm run update_releases         # Update .env.release_urls
 *   npm run update_releases:check   # Dry-run (check only, no updates)
 */

import { Octokit } from '@octokit/rest';
import fs from 'fs';

const octokit = new Octokit();
const OWNER = 'BodhiSearch';
const REPO = 'BodhiApp';

// Tag patterns to search for (extensible for future artifacts)
const TAG_PATTERNS = [
  {
    regex: /^app\/v/,
    assetPattern: /_aarch64\.dmg$/,
    envVar: 'NEXT_PUBLIC_DOWNLOAD_URL_MACOS_ARM64',
  },
  // Future patterns:
  // { regex: /^docker\/v/, envVar: 'NEXT_PUBLIC_DOCKER_VERSION' },
  // { regex: /^docker-dev\/v/, envVar: 'NEXT_PUBLIC_DOCKER_DEV_VERSION' },
  // { regex: /^ts-client\/v/, envVar: 'NEXT_PUBLIC_TS_CLIENT_VERSION' },
  // { regex: /^napi\/v/, envVar: 'NEXT_PUBLIC_NAPI_VERSION' },
];

async function fetchLatestReleases() {
  const found = {};
  const sixMonthsAgo = new Date();
  sixMonthsAgo.setMonth(sixMonthsAgo.getMonth() - 6);

  let page = 1;
  let shouldContinue = true;

  console.log('Fetching releases from GitHub...');

  while (shouldContinue && Object.keys(found).length < TAG_PATTERNS.length) {
    const { data: releases } = await octokit.repos.listReleases({
      owner: OWNER,
      repo: REPO,
      per_page: 100,
      page,
    });

    if (releases.length === 0) {
      console.log('No more releases to fetch');
      break;
    }

    console.log(`Processing page ${page} (${releases.length} releases)...`);

    for (const release of releases) {
      const releaseDate = new Date(release.created_at);

      // Stop if release is older than 6 months
      if (releaseDate < sixMonthsAgo) {
        console.log(`Reached 6-month limit at ${release.tag_name}`);
        shouldContinue = false;
        break;
      }

      // Check each tag pattern
      for (const pattern of TAG_PATTERNS) {
        if (!found[pattern.envVar] && pattern.regex.test(release.tag_name)) {
          // If pattern has assetPattern, find matching asset
          if (pattern.assetPattern) {
            const asset = release.assets.find((a) => pattern.assetPattern.test(a.name));
            if (asset) {
              found[pattern.envVar] = asset.browser_download_url;
              console.log(`✓ Found ${pattern.envVar}: ${release.tag_name} -> ${asset.name}`);
            }
          } else {
            // No asset pattern - just store the tag
            found[pattern.envVar] = release.tag_name;
            console.log(`✓ Found ${pattern.envVar}: ${release.tag_name}`);
          }
        }
      }

      // If all patterns found, we can stop
      if (Object.keys(found).length === TAG_PATTERNS.length) {
        shouldContinue = false;
        break;
      }
    }

    // If we fetched less than 100, it's the last page
    if (releases.length < 100) {
      break;
    }

    page++;
  }

  return found;
}

function generateEnvFile(urls, dryRun) {
  const lines = [
    '# Auto-generated download URLs for website',
    `# Last updated: ${new Date().toISOString().split('T')[0]}`,
    '#',
    '# This file is checked into git and loaded by Next.js build process.',
    '# To update: run `npm run update_releases` or `make website.update_releases`',
    '',
  ];

  for (const [key, value] of Object.entries(urls)) {
    lines.push(`${key}=${value}`);
  }

  const content = lines.join('\n') + '\n';

  if (dryRun) {
    console.log('\n=== Dry-run mode - would write to .env.release_urls: ===\n');
    console.log(content);
    return;
  }

  fs.writeFileSync('.env.release_urls', content);
  console.log('\n✓ Updated .env.release_urls');
  console.log('  File:', '.env.release_urls');
  console.log('  Variables updated:', Object.keys(urls).length);
}

async function main() {
  const dryRun = process.argv.includes('--check');

  console.log(dryRun ? '=== Checking latest releases (dry-run) ===' : '=== Updating release URLs ===');
  console.log('');

  try {
    const urls = await fetchLatestReleases();

    if (Object.keys(urls).length === 0) {
      console.error('\n✗ No matching releases found!');
      console.error(
        '  Searched for patterns:',
        TAG_PATTERNS.map((p) => p.regex.source)
      );
      process.exit(1);
    }

    generateEnvFile(urls, dryRun);

    if (!dryRun) {
      console.log('\nNext steps:');
      console.log('  1. Review changes: git diff .env.release_urls');
      console.log('  2. Test build: npm run build');
      console.log('  3. Commit changes: git add .env.release_urls && git commit');
    }
  } catch (error) {
    console.error('\n✗ Error:', error.message);
    if (error.response) {
      console.error('  GitHub API response:', error.response.status, error.response.statusText);
    }
    process.exit(1);
  }
}

main();
