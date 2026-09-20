#!/usr/bin/env node
const fs = require('node:fs');
const path = require('node:path');
const { execSync } = require('node:child_process');

const MAX_ATTEMPTS = 10;
const RETRY_DELAY_MS = 10000;

function sleep(ms) {
  return new Promise(resolve => setTimeout(resolve, ms));
}

function checkPackageVersion(packageName) {
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

async function verifyPackages(options = {}) {
  const {
    cwd = process.cwd(),
    expectedVersion: expectedVersionOverride = process.env.RELEASE_VERSION,
    maxAttempts = MAX_ATTEMPTS,
    retryDelayMs = RETRY_DELAY_MS,
    getPackageVersion = checkPackageVersion,
    wait = sleep,
    logger = console,
  } = options;
  const packageJsonPath = path.join(cwd, 'package.json');

  if (!fs.existsSync(packageJsonPath)) {
    throw new Error('package.json not found in current directory');
  }

  const packageJson = JSON.parse(fs.readFileSync(packageJsonPath, 'utf8'));
  const expectedVersion = expectedVersionOverride || packageJson.version;
  const packageName = packageJson.napi?.package?.name || packageJson.name;

  if (!packageName) {
    throw new Error('Could not determine package name');
  }

  logger.log(`Expected version: ${expectedVersion}`);
  logger.log(`Main package: ${packageName}`);

  const packagesToVerify = new Map();

  packagesToVerify.set(packageName, {
    name: packageName,
    type: 'main',
    verified: false,
  });

  const npmDir = path.join(cwd, 'npm');
  if (fs.existsSync(npmDir)) {
    const platformDirs = fs
      .readdirSync(npmDir, { withFileTypes: true })
      .filter(dirent => dirent.isDirectory())
      .map(dirent => dirent.name);

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

  logger.log(`\nPackages to verify: ${packagesToVerify.size}`);
  for (const [name, info] of packagesToVerify) {
    logger.log(`  - ${name} (${info.type})`);
  }

  for (let attempt = 1; attempt <= maxAttempts; attempt++) {
    logger.log(`\n--- Verification attempt ${attempt} of ${maxAttempts} ---`);

    let allVerified = true;
    const unverifiedPackages = [];

    for (const [currentPackageName, packageInfo] of packagesToVerify) {
      if (packageInfo.verified) {
        continue;
      }

      logger.log(`Checking ${currentPackageName}...`);
      const publishedVersion = getPackageVersion(currentPackageName);

      if (publishedVersion === null) {
        logger.log('  ❌ Not found on NPM');
        allVerified = false;
        unverifiedPackages.push(`${currentPackageName} (not found)`);
      } else if (publishedVersion !== expectedVersion) {
        logger.log(`  ❌ Version mismatch: expected ${expectedVersion}, found ${publishedVersion}`);
        allVerified = false;
        unverifiedPackages.push(`${currentPackageName} (wrong version: ${publishedVersion})`);
      } else {
        logger.log(`  ✅ Successfully verified version ${publishedVersion}`);
        packageInfo.verified = true;
      }
    }

    if (allVerified) {
      logger.log('\n🎉 All packages successfully verified!');
      logger.log(`Successfully published ${packagesToVerify.size} packages with version ${expectedVersion}`);
      return true;
    }

    logger.log(`\nUnverified packages (${unverifiedPackages.length}):`);
    for (const pkg of unverifiedPackages) {
      logger.log(`  - ${pkg}`);
    }

    const verifiedCount = Array.from(packagesToVerify.values()).filter(p => p.verified).length;
    logger.log(`Progress: ${verifiedCount}/${packagesToVerify.size} packages verified`);

    if (attempt === maxAttempts) {
      logger.error(`\n❌ Package verification failed after ${maxAttempts} attempts`);
      logger.error(`Failed to verify ${unverifiedPackages.length} packages`);
      return false;
    }

    logger.log(`\nWaiting ${retryDelayMs / 1000} seconds before next attempt...`);
    await wait(retryDelayMs);
  }

  return false;
}

if (require.main === module) {
  verifyPackages()
    .then(verified => {
      if (!verified) {
        process.exitCode = 1;
      }
    })
    .catch(error => {
      console.error('Error during package verification:', error.message);
      process.exitCode = 1;
    });
}

module.exports = { MAX_ATTEMPTS, RETRY_DELAY_MS, verifyPackages };
