#!/usr/bin/env node

/**
 * Fetch the latest app version from GitHub releases API
 * Handles migration from v* to app/v* format
 * Usage: node get_github_release_version.js <repo>
 * Example: node get_github_release_version.js BodhiSearch/BodhiApp
 * Returns the latest version string or "0.0.25" if no app releases found
 */

import { execSync } from 'child_process';

function getAppVersionFromReleases(repo) {
  try {
    // Use gh CLI to get releases with app-related tags
    const command = `gh api repos/${repo}/releases --jq '[.[] | select(.tag_name | startswith("v") or startswith("app/v")) | .tag_name] | map(if startswith("app/v") then ltrimstr("app/v") elif startswith("v") then ltrimstr("v") else . end) | map(split(".") | map(tonumber)) | sort_by(.[0], .[1], .[2]) | last | join(".")'`;

    const result = execSync(command, { encoding: 'utf8' }).trim();

    // If we get "null" it means no releases found
    if (result === 'null' || result === '') {
      return '0.0.25'; // Start from last known version before migration
    }

    // Validate version format
    if (!/^\d+\.\d+\.\d+$/.test(result)) {
      console.error(`Invalid version format received: ${result}`);
      return '0.0.25';
    }

    return result;
  } catch (error) {
    console.error(`Error fetching GitHub releases: ${error.message}`);
    return '0.0.25'; // Fallback to last known version
  }
}

function main() {
  const repo = process.argv[2];

  if (!repo) {
    console.error('Usage: node get_github_release_version.js <repo>');
    console.error('Example: node get_github_release_version.js BodhiSearch/BodhiApp');
    process.exit(1);
  }

  try {
    const version = getAppVersionFromReleases(repo);
    console.log(version);
    process.exit(0);
  } catch (error) {
    console.error('Error:', error.message);
    console.log('0.0.25'); // Fallback version
    process.exit(1);
  }
}

// If run directly
if (import.meta.url === `file://${process.argv[1]}`) {
  main();
}