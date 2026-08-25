#!/usr/bin/env node
const fs = require('node:fs');
const path = require('node:path');

function updateOptionalDependencies() {
  const packageJsonPath = path.join(process.cwd(), 'package.json');

  if (!fs.existsSync(packageJsonPath)) {
    console.error('Error: package.json not found in current directory');
    process.exit(1);
  }

  const packageJson = JSON.parse(fs.readFileSync(packageJsonPath, 'utf8'));

  const version = process.env.RELEASE_VERSION || packageJson.version;

  const packageName = packageJson.napi?.package?.name || packageJson.name;

  if (!packageName) {
    console.error('Error: Could not determine package name');
    process.exit(1);
  }

  console.log(`Package name: ${packageName}`);
  console.log(`Version: ${version}`);

  const optionalDependencies = {};
  const npmDir = path.join(process.cwd(), 'npm');

  if (!fs.existsSync(npmDir)) {
    console.log('Warning: npm directory not found, creating empty optionalDependencies');
  } else {
    const platformDirs = fs
      .readdirSync(npmDir, { withFileTypes: true })
      .filter((dirent) => dirent.isDirectory())
      .map((dirent) => dirent.name);

    if (platformDirs.length === 0) {
      console.log('Warning: No platform directories found in npm/');
    } else {
      console.log(`Found platform directories: ${platformDirs.join(', ')}`);

      for (const platformDir of platformDirs) {
        const platformPackageName = `${packageName}-${platformDir}`;
        optionalDependencies[platformPackageName] = version;
      }
    }
  }

  packageJson.optionalDependencies = optionalDependencies;

  fs.writeFileSync(packageJsonPath, `${JSON.stringify(packageJson, null, 2)}\n`);

  console.log('\nUpdated optionalDependencies:');
  console.log(JSON.stringify(optionalDependencies, null, 2));
  console.log('\n✅ package.json updated successfully');
}

if (require.main === module) {
  updateOptionalDependencies();
}

module.exports = updateOptionalDependencies;
