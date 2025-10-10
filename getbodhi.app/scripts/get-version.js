#!/usr/bin/env node

/**
 * Fetch current deployed version from live website
 * Source of truth: https://getbodhi.app/version.json
 *
 * Usage: node get-version.js
 * Returns: version string (e.g., "0.1.0") or "0.0.0" if not available
 */

async function fetchDeployedVersion() {
  try {
    const response = await fetch('https://getbodhi.app/version.json');

    if (!response.ok) {
      console.error(`Failed to fetch version.json: ${response.status} ${response.statusText}`);
      return '0.0.0';
    }

    const data = await response.json();

    if (!data.version) {
      console.error('version.json does not contain version field');
      return '0.0.0';
    }

    // Remove -dev suffix if present (we want the base version)
    const version = data.version.replace(/-dev$/, '');

    // Validate version format
    if (!/^\d+\.\d+\.\d+$/.test(version)) {
      console.error(`Invalid version format: ${version}`);
      return '0.0.0';
    }

    return version;
  } catch (error) {
    console.error(`Error fetching deployed version: ${error.message}`);
    // Return 0.0.0 for new sites that don't have version.json yet
    return '0.0.0';
  }
}

async function main() {
  try {
    const version = await fetchDeployedVersion();
    console.log(version);
    process.exit(0);
  } catch (error) {
    console.error('Error:', error.message);
    console.log('0.0.0');
    process.exit(1);
  }
}

// If run directly
if (import.meta.url === `file://${process.argv[1]}`) {
  main();
}

export { fetchDeployedVersion };
