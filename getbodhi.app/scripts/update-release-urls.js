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
    platforms: [
      {
        id: 'macos',
        assetPattern: /Bodhi[\s.]App.*\.dmg$/,
        envVar: 'NEXT_PUBLIC_DOWNLOAD_URL_MACOS',
        platformKey: 'macos',
        archKey: 'silicon',
      },
      {
        id: 'windows',
        assetPattern: /Bodhi[\s.]App.*\.msi$/,
        envVar: 'NEXT_PUBLIC_DOWNLOAD_URL_WINDOWS',
        platformKey: 'windows',
        archKey: 'x64',
      },
      {
        id: 'linux',
        assetPattern: /Bodhi[\s.]App.*\.rpm$/,
        envVar: 'NEXT_PUBLIC_DOWNLOAD_URL_LINUX',
        platformKey: 'linux',
        archKey: 'x64',
      },
    ],
  },
  // Future patterns:
  // { regex: /^docker\/v/, envVar: 'NEXT_PUBLIC_DOCKER_VERSION' },
  // { regex: /^docker-dev\/v/, envVar: 'NEXT_PUBLIC_DOCKER_DEV_VERSION' },
  // { regex: /^ts-client\/v/, envVar: 'NEXT_PUBLIC_TS_CLIENT_VERSION' },
  // { regex: /^napi\/v/, envVar: 'NEXT_PUBLIC_NAPI_VERSION' },
];

async function fetchLatestReleases() {
  const found = {};
  let releaseMetadata = null;
  const sixMonthsAgo = new Date();
  sixMonthsAgo.setMonth(sixMonthsAgo.getMonth() - 6);

  let page = 1;
  let shouldContinue = true;

  console.log('Fetching releases from GitHub...');

  // Count total platforms to find
  const totalPlatforms = TAG_PATTERNS.reduce(
    (sum, pattern) => sum + (pattern.platforms ? pattern.platforms.length : 1),
    0
  );

  while (shouldContinue && Object.keys(found).length < totalPlatforms) {
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
        if (pattern.regex.test(release.tag_name)) {
          // Handle platform-based patterns
          if (pattern.platforms) {
            for (const platform of pattern.platforms) {
              if (!found[platform.envVar]) {
                const asset = release.assets.find((a) => platform.assetPattern.test(a.name));
                if (asset) {
                  found[platform.envVar] = {
                    url: asset.browser_download_url,
                    filename: asset.name,
                    platformKey: platform.platformKey,
                    archKey: platform.archKey,
                  };
                  console.log(`✓ Found ${platform.id}: ${release.tag_name} -> ${asset.name}`);
                }
              }
            }

            // Store release metadata (version, tag, released_at) from first matching release
            if (!releaseMetadata) {
              const versionMatch = release.tag_name.match(/v([\d.]+)$/);
              releaseMetadata = {
                version: versionMatch ? versionMatch[1] : release.tag_name,
                tag: release.tag_name,
                released_at: release.published_at || release.created_at,
              };
            }
          } else if (pattern.assetPattern) {
            // Legacy single-asset pattern
            if (!found[pattern.envVar]) {
              const asset = release.assets.find((a) => pattern.assetPattern.test(a.name));
              if (asset) {
                found[pattern.envVar] = {
                  url: asset.browser_download_url,
                  filename: asset.name,
                };
                console.log(`✓ Found ${pattern.envVar}: ${release.tag_name} -> ${asset.name}`);
              }
            }
          } else {
            // No asset pattern - just store the tag
            if (!found[pattern.envVar]) {
              found[pattern.envVar] = {
                url: release.tag_name,
              };
              console.log(`✓ Found ${pattern.envVar}: ${release.tag_name}`);
            }
          }
        }
      }

      // If all platforms found, we can stop
      if (Object.keys(found).length === totalPlatforms) {
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

  return { found, releaseMetadata };
}

function generateEnvFile(data, releaseMetadata, dryRun) {
  const lines = [
    '# Auto-generated download URLs for website',
    `# Last updated: ${new Date().toISOString().split('T')[0]}`,
    '#',
    '# This file is checked into git and loaded by Next.js build process.',
    '# To update: run `npm run update_releases` or `make website.update_releases`',
    '',
  ];

  // Add version and tag if available
  if (releaseMetadata) {
    lines.push('# App version and tag');
    lines.push(`NEXT_PUBLIC_APP_VERSION=${releaseMetadata.version}`);
    lines.push(`NEXT_PUBLIC_APP_TAG=${releaseMetadata.tag}`);
    lines.push('');
    lines.push('# Platform download URLs');
  }

  // Add download URLs
  for (const [key, value] of Object.entries(data)) {
    const url = typeof value === 'string' ? value : value.url;
    lines.push(`${key}=${url}`);
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
  console.log('  Variables updated:', Object.keys(data).length + (releaseMetadata ? 2 : 0));
}

function generateReleasesJson(data, releaseMetadata, dryRun) {
  if (!releaseMetadata) {
    console.log('\n⚠ Skipping releases.json generation - no release metadata available');
    return;
  }

  // Build nested platform structure
  const platforms = {};

  for (const value of Object.values(data)) {
    if (value.platformKey && value.archKey) {
      if (!platforms[value.platformKey]) {
        platforms[value.platformKey] = {};
      }
      platforms[value.platformKey][value.archKey] = {
        download_url: value.url,
        filename: value.filename,
      };
    }
  }

  const releasesData = {
    version: releaseMetadata.version,
    tag: releaseMetadata.tag,
    released_at: releaseMetadata.released_at,
    platforms,
  };

  const content = JSON.stringify(releasesData, null, 2) + '\n';

  if (dryRun) {
    console.log('\n=== Dry-run mode - would write to public/releases.json: ===\n');
    console.log(content);
    return;
  }

  // Ensure public directory exists
  if (!fs.existsSync('public')) {
    fs.mkdirSync('public', { recursive: true });
  }

  fs.writeFileSync('public/releases.json', content);
  console.log('\n✓ Updated public/releases.json');
  console.log('  File:', 'public/releases.json');
  console.log('  Platforms:', Object.keys(platforms).length);
}

async function main() {
  const dryRun = process.argv.includes('--check');

  console.log(dryRun ? '=== Checking latest releases (dry-run) ===' : '=== Updating release URLs ===');
  console.log('');

  try {
    const { found, releaseMetadata } = await fetchLatestReleases();

    if (Object.keys(found).length === 0) {
      console.error('\n✗ No matching releases found!');
      console.error(
        '  Searched for patterns:',
        TAG_PATTERNS.map((p) => p.regex.source)
      );
      process.exit(1);
    }

    generateEnvFile(found, releaseMetadata, dryRun);
    generateReleasesJson(found, releaseMetadata, dryRun);

    if (!dryRun) {
      console.log('\nNext steps:');
      console.log('  1. Review changes: git diff .env.release_urls public/releases.json');
      console.log('  2. Test build: npm run build');
      console.log('  3. Commit changes: git add .env.release_urls public/releases.json && git commit');
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
