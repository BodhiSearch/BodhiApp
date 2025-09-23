#!/usr/bin/env node

/**
 * Increment the patch version of a semver version string
 * Usage: node increment_version.js <version>
 * Example: node increment_version.js 1.2.3 -> 1.2.4
 */

function incrementPatchVersion(version) {
  // Validate version format (x.y.z)
  const versionRegex = /^(\d+)\.(\d+)\.(\d+)$/;
  const match = version.match(versionRegex);

  if (!match) {
    throw new Error(`Invalid version format: ${version}. Expected format: x.y.z`);
  }

  const [, major, minor, patch] = match;
  const newPatch = parseInt(patch, 10) + 1;

  return `${major}.${minor}.${newPatch}`;
}

function main() {
  const currentVersion = process.argv[2];

  if (!currentVersion) {
    console.error('Usage: node increment_version.js <version>');
    console.error('Example: node increment_version.js 1.2.3');
    process.exit(1);
  }

  try {
    const nextVersion = incrementPatchVersion(currentVersion);
    console.log(nextVersion);
    process.exit(0);
  } catch (error) {
    console.error('Error:', error.message);
    process.exit(1);
  }
}

// If run directly
if (import.meta.url === `file://${process.argv[1]}`) {
  main();
}

export { incrementPatchVersion };