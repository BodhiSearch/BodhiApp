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

/**
 * Static Docker variants configuration
 * We always use -latest tags, so variants are statically defined
 * This prevents intermittent build failures from removing variants from the website
 */
const DOCKER_VARIANTS = {
  cpu: {
    description: 'Multi-platform: AMD64 + ARM64',
    platforms: ['linux/amd64', 'linux/arm64'],
    docker_flags: [],
  },
  cuda: {
    description: 'NVIDIA GPU acceleration',
    platforms: ['linux/amd64'],
    gpu_type: 'NVIDIA',
    docker_flags: ['--gpus all'],
  },
  rocm: {
    description: 'AMD GPU acceleration',
    platforms: ['linux/amd64'],
    gpu_type: 'AMD',
    docker_flags: ['--device=/dev/kfd', '--device=/dev/dri', '--group-add video'],
  },
  intel: {
    description: 'Intel GPU acceleration (SYCL)',
    platforms: ['linux/amd64'],
    gpu_type: 'Intel',
    docker_flags: ['--device=/dev/dri'],
  },
  cann: {
    description: 'Huawei Ascend NPU acceleration',
    platforms: ['linux/amd64', 'linux/arm64'],
    gpu_type: 'Huawei Ascend',
    docker_flags: [
      '--device=/dev/davinci0',
      '--device=/dev/davinci_manager',
      '--device=/dev/devmm_svm',
      '--device=/dev/hisi_hdc',
    ],
  },
  musa: {
    description: 'Moore Threads GPU acceleration',
    platforms: ['linux/amd64'],
    gpu_type: 'Moore Threads',
    docker_flags: ['--device=/dev/musa'],
  },
  vulkan: {
    description: 'Cross-vendor GPU acceleration',
    platforms: ['linux/amd64'],
    gpu_type: 'Vulkan',
    docker_flags: ['--device=/dev/dri'],
  },
};

// Tag patterns to search for (extensible for future artifacts)
const TAG_PATTERNS = [
  {
    regex: /^app\/v/,
    type: 'desktop',
    platforms: [
      {
        id: 'macos',
        assetPattern: /^Bodhi_App\.dmg$/,
        envVar: 'NEXT_PUBLIC_DOWNLOAD_URL_MACOS',
        platformKey: 'macos',
        archKey: 'silicon',
      },
      {
        id: 'windows',
        assetPattern: /^Bodhi_App\.msi$/,
        envVar: 'NEXT_PUBLIC_DOWNLOAD_URL_WINDOWS',
        platformKey: 'windows',
        archKey: 'x64',
      },
      {
        id: 'linux',
        assetPattern: /^Bodhi_App\.rpm$/,
        envVar: 'NEXT_PUBLIC_DOWNLOAD_URL_LINUX',
        platformKey: 'linux',
        archKey: 'x64',
      },
    ],
  },
  {
    regex: /^docker\/v/,
    type: 'docker',
    registry: 'ghcr.io/bodhisearch/bodhiapp',
  },
  // { regex: /^ts-client\/v/, envVar: 'NEXT_PUBLIC_TS_CLIENT_VERSION' },
  // { regex: /^napi\/v/, envVar: 'NEXT_PUBLIC_NAPI_VERSION' },
];

async function fetchLatestReleases() {
  const found = {};
  let desktopMetadata = null;
  let dockerMetadata = null;
  const sixMonthsAgo = new Date();
  sixMonthsAgo.setMonth(sixMonthsAgo.getMonth() - 6);

  let page = 1;
  let shouldContinue = true;

  console.log('Fetching releases from GitHub...');

  // Count total items to find (desktop platforms + docker release)
  const desktopPlatforms = TAG_PATTERNS.find((p) => p.type === 'desktop')?.platforms?.length || 0;
  const dockerPatterns = TAG_PATTERNS.filter((p) => p.type === 'docker').length;

  while (shouldContinue) {
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
          // Handle Docker patterns - just extract version info
          if (pattern.type === 'docker') {
            if (!dockerMetadata) {
              console.log(`  Found Docker release: ${release.tag_name}`);

              const versionMatch = release.tag_name.match(/v([\d.]+)$/);
              dockerMetadata = {
                version: versionMatch ? versionMatch[1] : release.tag_name,
                tag: release.tag_name,
                released_at: release.published_at || release.created_at,
                registry: pattern.registry,
              };

              found.docker = dockerMetadata;
              console.log(`✓ Using Docker release: ${release.tag_name}`);
              console.log(`  All variants will use -latest tags from: ${pattern.registry}`);
            }
          }
          // Handle desktop platform patterns
          else if (pattern.platforms) {
            for (const platform of pattern.platforms) {
              if (!found[platform.envVar]) {
                const asset = release.assets.find((a) => platform.assetPattern.test(a.name));
                if (asset) {
                  found[platform.envVar] = {
                    url: asset.browser_download_url,
                    filename: asset.name,
                    size: asset.size,
                    platformKey: platform.platformKey,
                    archKey: platform.archKey,
                  };
                  console.log(`✓ Found ${platform.id}: ${release.tag_name} -> ${asset.name}`);
                }
              }
            }

            // Store desktop release metadata from first matching release
            if (!desktopMetadata) {
              const versionMatch = release.tag_name.match(/v([\d.]+)$/);
              desktopMetadata = {
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
          }
        }
      }

      // Check if we found everything
      const desktopFound =
        desktopPlatforms === 0 ||
        Object.keys(found).filter((k) => k.startsWith('NEXT_PUBLIC_DOWNLOAD')).length === desktopPlatforms;
      const dockerFound = dockerPatterns === 0 || dockerMetadata !== null;

      if (desktopFound && dockerFound) {
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

  return { found, desktopMetadata, dockerMetadata };
}

/**
 * Load existing releases.json as backup
 * @returns {object|null} - Existing releases data or null if not found
 */
function loadExistingReleases() {
  try {
    if (fs.existsSync('public/releases.json')) {
      const content = fs.readFileSync('public/releases.json', 'utf8');
      return JSON.parse(content);
    }
  } catch (error) {
    console.log(`  ⚠ Could not load existing releases.json: ${error.message}`);
  }
  return null;
}

/**
 * Check if platform data is complete
 * @param {object} platformData - Platform data object with archKey
 * @param {string} platformName - Name of platform for logging
 * @returns {boolean} - True if all required fields are present
 */
function isPlatformComplete(platformData, platformName) {
  if (!platformData || Object.keys(platformData).length === 0) {
    console.log(`  ⚠ ${platformName}: No data available`);
    return false;
  }

  // Check each arch variant
  for (const [archKey, archData] of Object.entries(platformData)) {
    if (archKey === 'version' || archKey === 'tag') continue; // Skip metadata fields

    if (!archData.download_url || !archData.filename || !archData.size) {
      console.log(`  ⚠ ${platformName}.${archKey}: Missing required fields`);
      return false;
    }
  }

  console.log(`  ✓ ${platformName}: Complete`);
  return true;
}

/**
 * Fetch checksums.json from a release
 * @param {string} tag - Release tag (e.g., "app/v0.0.36")
 * @returns {Promise<object|null>} - Checksums data or null if not found
 */
async function fetchChecksums(tag) {
  try {
    console.log(`  Fetching checksums for ${tag}...`);
    const { data: release } = await octokit.repos.getReleaseByTag({
      owner: OWNER,
      repo: REPO,
      tag,
    });

    const checksumsAsset = release.assets.find((a) => a.name === 'checksums.json');
    if (!checksumsAsset) {
      console.log(`  ⚠ checksums.json not found for ${tag}, checksums will be omitted`);
      return null;
    }

    const response = await fetch(checksumsAsset.browser_download_url);
    if (!response.ok) {
      console.log(`  ⚠ Failed to fetch checksums.json: ${response.statusText}`);
      return null;
    }

    const checksums = await response.json();
    console.log(`  ✓ Fetched checksums for ${Object.keys(checksums.checksums || {}).length} files`);
    return checksums;
  } catch (error) {
    console.log(`  ⚠ Error fetching checksums: ${error.message}`);
    return null;
  }
}

function generateEnvFile(data, desktopMetadata, dockerMetadata, dryRun) {
  const lines = [
    '# Auto-generated download URLs for website',
    `# Last updated: ${new Date().toISOString().split('T')[0]}`,
    '#',
    '# This file is checked into git and loaded by Next.js build process.',
    '# To update: run `npm run update_releases` or `make website.update_releases`',
    '',
  ];

  // Add desktop app version and tag if available
  if (desktopMetadata) {
    lines.push('# Desktop app version and tag');
    lines.push(`NEXT_PUBLIC_APP_VERSION=${desktopMetadata.version}`);
    lines.push(`NEXT_PUBLIC_APP_TAG=${desktopMetadata.tag}`);
    lines.push('');
  }

  // Add Docker version and tag if available
  if (dockerMetadata) {
    lines.push('# Docker version and tag');
    lines.push(`NEXT_PUBLIC_DOCKER_VERSION=${dockerMetadata.version}`);
    lines.push(`NEXT_PUBLIC_DOCKER_TAG=${dockerMetadata.tag}`);
    lines.push(`NEXT_PUBLIC_DOCKER_REGISTRY=${dockerMetadata.registry}`);
    lines.push('');
  }

  // Add platform download URLs
  if (desktopMetadata) {
    lines.push('# Platform download URLs');
  }

  for (const [key, value] of Object.entries(data)) {
    if (key === 'docker') continue; // Skip docker metadata (already handled above)
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
  const varCount =
    Object.keys(data).length - (data.docker ? 1 : 0) + (desktopMetadata ? 2 : 0) + (dockerMetadata ? 3 : 0);
  console.log('  Variables updated:', varCount);
}

async function generateReleasesJson(data, desktopMetadata, dockerMetadata, dryRun) {
  if (!desktopMetadata && !dockerMetadata) {
    console.log('\n⚠ Skipping releases.json generation - no release metadata available');
    return;
  }

  console.log('\n=== Building releases.json with per-platform validation ===');

  // Load existing releases as backup
  const backup = loadExistingReleases();
  const releasesData = {};

  // Build desktop platform structure with per-platform atomicity
  if (desktopMetadata) {
    console.log('\n--- Desktop Platforms ---');
    console.log(`Latest release: ${desktopMetadata.tag} (${desktopMetadata.version})`);

    // Fetch checksums from release
    const checksumsData = await fetchChecksums(desktopMetadata.tag);
    const checksums = checksumsData?.checksums || {};

    // Build new platforms data grouped by platform
    const newPlatforms = {};

    for (const value of Object.values(data)) {
      if (value.platformKey && value.archKey) {
        if (!newPlatforms[value.platformKey]) {
          newPlatforms[value.platformKey] = {
            version: desktopMetadata.version,
            tag: desktopMetadata.tag,
          };
        }

        // Look up checksum by filename
        const checksumInfo = checksums[value.filename] || {};

        newPlatforms[value.platformKey][value.archKey] = {
          download_url: value.url,
          filename: value.filename,
          size: value.size,
          ...(checksumInfo.sha256 && { sha256: checksumInfo.sha256 }),
        };
      }
    }

    // Validate and merge with backup per platform
    const finalPlatforms = {};

    for (const platformKey of ['macos', 'windows', 'linux']) {
      const newPlatformData = newPlatforms[platformKey];
      const backupPlatformData = backup?.desktop?.platforms?.[platformKey];

      console.log(`\nValidating ${platformKey}:`);

      if (isPlatformComplete(newPlatformData, platformKey)) {
        finalPlatforms[platformKey] = newPlatformData;
        console.log(`  → Using new data from ${desktopMetadata.tag}`);
      } else if (backupPlatformData) {
        finalPlatforms[platformKey] = backupPlatformData;
        const backupVersion = backupPlatformData.version || backup.desktop.version;
        const backupTag = backupPlatformData.tag || backup.desktop.tag;
        console.log(`  → Falling back to ${backupTag} (${backupVersion})`);
      } else {
        console.log(`  → No valid data available, skipping ${platformKey}`);
      }
    }

    releasesData.desktop = {
      version: desktopMetadata.version,
      tag: desktopMetadata.tag,
      released_at: desktopMetadata.released_at,
      platforms: finalPlatforms,
    };
  }

  // Add Docker data with static variants and per-variant version tracking
  if (dockerMetadata) {
    console.log('\n--- Docker Variants ---');
    console.log(`Latest release: ${dockerMetadata.tag} (${dockerMetadata.version})`);

    const variants = {};

    // Define variant order for consistent display
    const variantOrder = ['cpu', 'cuda', 'rocm', 'vulkan', 'intel', 'cann', 'musa'];

    // Generate variants in order with version tracking
    for (const variantName of variantOrder) {
      const variantConfig = DOCKER_VARIANTS[variantName];
      if (variantConfig) {
        variants[variantName] = {
          version: dockerMetadata.version,
          tag: dockerMetadata.tag,
          latest_tag: `latest-${variantName}`,
          platforms: variantConfig.platforms,
          pull_command: `docker pull ${dockerMetadata.registry}:latest-${variantName}`,
          docker_flags: variantConfig.docker_flags,
          description: variantConfig.description,
          ...(variantConfig.gpu_type && { gpu_type: variantConfig.gpu_type }),
        };
      }
    }

    releasesData.docker = {
      version: dockerMetadata.version,
      tag: dockerMetadata.tag,
      released_at: dockerMetadata.released_at,
      registry: dockerMetadata.registry,
      variants,
    };
  }

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

  if (desktopMetadata) {
    console.log('\n--- Desktop Platform Sync Status ---');
    for (const [platformKey, platformData] of Object.entries(releasesData.desktop.platforms)) {
      const platformVersion = platformData.version || releasesData.desktop.version;
      const platformTag = platformData.tag || releasesData.desktop.tag;
      const isLatest = platformVersion === desktopMetadata.version;
      const status = isLatest ? '✓ SYNCED' : '⚠ OUT OF SYNC';
      console.log(`  ${platformKey}: ${status} (${platformTag})`);
    }
  }

  if (dockerMetadata) {
    console.log('\n--- Docker Variant Sync Status ---');
    const variants = Object.keys(releasesData.docker.variants);
    console.log(`  All variants synced to ${dockerMetadata.tag}`);
    console.log(`  Variants: ${variants.join(', ')}`);
  }
}

async function main() {
  const dryRun = process.argv.includes('--check');

  console.log(dryRun ? '=== Checking latest releases (dry-run) ===' : '=== Updating release URLs ===');
  console.log('');

  try {
    const { found, desktopMetadata, dockerMetadata } = await fetchLatestReleases();

    if (Object.keys(found).length === 0) {
      console.error('\n✗ No matching releases found!');
      console.error(
        '  Searched for patterns:',
        TAG_PATTERNS.map((p) => p.regex.source)
      );
      process.exit(1);
    }

    generateEnvFile(found, desktopMetadata, dockerMetadata, dryRun);
    await generateReleasesJson(found, desktopMetadata, dockerMetadata, dryRun);

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
