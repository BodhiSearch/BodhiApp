#!/usr/bin/env node

/**
 * Increment version of a semver version string
 * Usage: node increment_version.js <version> [patch|minor] [prerelease-id]
 * Examples:
 *   node increment_version.js 1.2.3 -> 1.2.4 (default: patch increment)
 *   node increment_version.js 1.2.3 minor -> 1.3.0
 *   node increment_version.js 1.2.3 minor dev -> 1.3.0-dev
 *   node increment_version.js 1.2.3-dev patch -> 1.2.3 (removes prerelease)
 */

function parseVersion(version) {
  const versionRegex = /^(\d+)\.(\d+)\.(\d+)(?:-(.+))?$/;
  const match = version.match(versionRegex);

  if (!match) {
    throw new Error(`Invalid version format: ${version}. Expected format: x.y.z or x.y.z-prerelease`);
  }

  const [, major, minor, patch, prerelease] = match;
  return {
    major: parseInt(major, 10),
    minor: parseInt(minor, 10),
    patch: parseInt(patch, 10),
    prerelease: prerelease || null
  };
}

function formatVersion(parts) {
  const { major, minor, patch, prerelease } = parts;
  let version = `${major}.${minor}.${patch}`;
  if (prerelease) {
    version += `-${prerelease}`;
  }
  return version;
}

function incrementPatchVersion(version) {
  const parts = parseVersion(version);
  parts.patch += 1;
  parts.prerelease = null;
  return formatVersion(parts);
}

function incrementMinorVersion(version) {
  const parts = parseVersion(version);
  parts.minor += 1;
  parts.patch = 0;
  parts.prerelease = null;
  return formatVersion(parts);
}

function addPrereleaseIdentifier(version, identifier) {
  const parts = parseVersion(version);
  parts.prerelease = identifier;
  return formatVersion(parts);
}

function removePrereleaseIdentifier(version) {
  const parts = parseVersion(version);
  parts.prerelease = null;
  return formatVersion(parts);
}

function main() {
  const currentVersion = process.argv[2];
  const incrementType = process.argv[3] || 'patch';
  const prereleaseId = process.argv[4];

  if (!currentVersion) {
    console.error('Usage: node increment_version.js <version> [patch|minor] [prerelease-id]');
    console.error('Examples:');
    console.error('  node increment_version.js 1.2.3           -> 1.2.4');
    console.error('  node increment_version.js 1.2.3 minor     -> 1.3.0');
    console.error('  node increment_version.js 1.2.3 minor dev -> 1.3.0-dev');
    console.error('  node increment_version.js 1.2.3-dev patch -> 1.2.3');
    process.exit(1);
  }

  try {
    let nextVersion;

    if (incrementType === 'patch') {
      nextVersion = incrementPatchVersion(currentVersion);
    } else if (incrementType === 'minor') {
      nextVersion = incrementMinorVersion(currentVersion);
    } else {
      throw new Error(`Invalid increment type: ${incrementType}. Expected 'patch' or 'minor'`);
    }

    if (prereleaseId) {
      nextVersion = addPrereleaseIdentifier(nextVersion, prereleaseId);
    }

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

export { incrementPatchVersion, incrementMinorVersion, addPrereleaseIdentifier, removePrereleaseIdentifier };