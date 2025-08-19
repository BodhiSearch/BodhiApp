#!/usr/bin/env node
const fs = require('fs');
const path = require('path');
const { execSync } = require('child_process');

const MAX_ATTEMPTS = 5;
const RETRY_DELAY_MS = 10000; // 10 seconds

function sleep(ms) {
  return new Promise((resolve) => setTimeout(resolve, ms));
}

function checkPackageVersion(packageName, expectedVersion) {
  try {
    const result = execSync(`npm view "${packageName}" version`, {
      encoding: 'utf8',
      stdio: 'pipe',
    });
    return result.trim();
  } catch (error) {
    return null;
  }
}

async function verifyPackages() {
  const packageJsonPath = path.join(process.cwd(), 'package.json');

  // Read main package.json
  if (!fs.existsSync(packageJsonPath)) {
    console.error('Error: package.json not found in current directory');
    process.exit(1);
  }

  const packageJson = JSON.parse(fs.readFileSync(packageJsonPath, 'utf8'));

  // Get version from environment variable or package.json
  const expectedVersion = process.env.RELEASE_VERSION || packageJson.version;

  // Get the package name (from napi.package.name or fallback to main name)
  const packageName = packageJson.napi?.package?.name || packageJson.name;

  if (!packageName) {
    console.error('Error: Could not determine package name');
    process.exit(1);
  }

  console.log(`Expected version: ${expectedVersion}`);
  console.log(`Main package: ${packageName}`);

  // Build list of packages to verify
  const packagesToVerify = new Map();

  // Add main package
  packagesToVerify.set(packageName, {
    name: packageName,
    type: 'main',
    verified: false,
  });

  // Add platform packages
  const npmDir = path.join(process.cwd(), 'npm');
  if (fs.existsSync(npmDir)) {
    const platformDirs = fs
      .readdirSync(npmDir, { withFileTypes: true })
      .filter((dirent) => dirent.isDirectory())
      .map((dirent) => dirent.name);

    for (const platformDir of platformDirs) {
      const platformPackageName = `${packageName}-${platformDir}`;
      packagesToVerify.set(platformPackageName, {
        name: platformPackageName,
        type: 'platform',
        platform: platformDir,
        verified: false,
      });
    }
  }

  console.log(`\nPackages to verify: ${packagesToVerify.size}`);
  for (const [name, info] of packagesToVerify) {
    console.log(`  - ${name} (${info.type})`);
  }

  // Verification loop
  for (let attempt = 1; attempt <= MAX_ATTEMPTS; attempt++) {
    console.log(`\n--- Verification attempt ${attempt} of ${MAX_ATTEMPTS} ---`);

    let allVerified = true;
    const unverifiedPackages = [];

    for (const [packageName, packageInfo] of packagesToVerify) {
      if (packageInfo.verified) {
        continue; // Skip already verified packages
      }

      console.log(`Checking ${packageName}...`);
      const publishedVersion = checkPackageVersion(packageName, expectedVersion);

      if (publishedVersion === null) {
        console.log(`  ❌ Not found on NPM`);
        allVerified = false;
        unverifiedPackages.push(`${packageName} (not found)`);
      } else if (publishedVersion !== expectedVersion) {
        console.log(
          `  ❌ Version mismatch: expected ${expectedVersion}, found ${publishedVersion}`
        );
        allVerified = false;
        unverifiedPackages.push(`${packageName} (wrong version: ${publishedVersion})`);
      } else {
        console.log(`  ✅ Successfully verified version ${publishedVersion}`);
        packageInfo.verified = true;
      }
    }

    if (allVerified) {
      console.log('\n🎉 All packages successfully verified!');
      console.log(
        `Successfully published ${packagesToVerify.size} packages with version ${expectedVersion}`
      );
      process.exit(0);
    }

    console.log(`\nUnverified packages (${unverifiedPackages.length}):`);
    for (const pkg of unverifiedPackages) {
      console.log(`  - ${pkg}`);
    }

    // Show verification progress
    const verifiedCount = Array.from(packagesToVerify.values()).filter((p) => p.verified).length;
    console.log(`Progress: ${verifiedCount}/${packagesToVerify.size} packages verified`);

    if (attempt === MAX_ATTEMPTS) {
      console.error(`\n❌ Package verification failed after ${MAX_ATTEMPTS} attempts`);
      console.error(`Failed to verify ${unverifiedPackages.length} packages`);
      process.exit(1);
    }

    console.log(`\nWaiting ${RETRY_DELAY_MS / 1000} seconds before next attempt...`);
    await sleep(RETRY_DELAY_MS);
  }
}

// Run the script
if (require.main === module) {
  verifyPackages().catch((error) => {
    console.error('Error during package verification:', error.message);
    process.exit(1);
  });
}

module.exports = verifyPackages;
