#!/usr/bin/env node
const fs = require('fs');
const path = require('path');

function updateOptionalDependencies() {
  const packageJsonPath = path.join(process.cwd(), 'package.json');

  // Read main package.json
  if (!fs.existsSync(packageJsonPath)) {
    console.error('Error: package.json not found in current directory');
    process.exit(1);
  }

  const packageJson = JSON.parse(fs.readFileSync(packageJsonPath, 'utf8'));

  // Get version from environment variable or package.json
  const version = process.env.RELEASE_VERSION || packageJson.version;

  // Get the package name (from napi.package.name or fallback to main name)
  const packageName = packageJson.napi?.package?.name || packageJson.name;

  if (!packageName) {
    console.error('Error: Could not determine package name');
    process.exit(1);
  }

  console.log(`Package name: ${packageName}`);
  console.log(`Version: ${version}`);

  // Find all platform directories and build optionalDependencies
  const optionalDependencies = {};
  const npmDir = path.join(process.cwd(), 'npm');

  if (!fs.existsSync(npmDir)) {
    console.log('Warning: npm directory not found, creating empty optionalDependencies');
  } else {
    const platformDirs = fs.readdirSync(npmDir, { withFileTypes: true })
      .filter(dirent => dirent.isDirectory())
      .map(dirent => dirent.name);

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

  // Update main package.json
  packageJson.optionalDependencies = optionalDependencies;

  // Write back with proper formatting
  fs.writeFileSync(packageJsonPath, JSON.stringify(packageJson, null, 2) + '\n');

  console.log('\nUpdated optionalDependencies:');
  console.log(JSON.stringify(optionalDependencies, null, 2));
  console.log('\n✅ package.json updated successfully');
}

// Run the script
if (require.main === module) {
  updateOptionalDependencies();
}

module.exports = updateOptionalDependencies;