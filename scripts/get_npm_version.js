#!/usr/bin/env node

/**
 * Fetch the current version of a package from npm registry
 * Usage: node get_npm_version.js <package-name>
 * Returns the version string or "0.0.0" if package not found
 */

import https from 'https';

function fetchNpmVersion(packageName) {
  return new Promise((resolve) => {
    const url = `https://registry.npmjs.org/${encodeURIComponent(packageName)}/latest`;

    const request = https.get(url, {
      headers: {
        'User-Agent': 'BodhiApp Release Script'
      }
    }, (response) => {
      let data = '';

      response.on('data', (chunk) => {
        data += chunk;
      });

      response.on('end', () => {
        try {
          if (response.statusCode === 404) {
            // Package not found, return default version
            resolve('0.0.0');
            return;
          }

          if (response.statusCode !== 200) {
            console.error(`npm registry returned status ${response.statusCode}`);
            resolve('0.0.0');
            return;
          }

          const packageInfo = JSON.parse(data);
          const version = packageInfo.version;

          if (version && /^\d+\.\d+\.\d+$/.test(version)) {
            resolve(version);
          } else {
            console.error('Invalid version format received from npm');
            resolve('0.0.0');
          }
        } catch (error) {
          console.error('Error parsing npm response:', error.message);
          resolve('0.0.0');
        }
      });
    });

    request.on('error', (error) => {
      console.error('Error fetching from npm:', error.message);
      resolve('0.0.0');
    });

    request.setTimeout(10000, () => {
      request.destroy();
      console.error('Request timeout while fetching from npm');
      resolve('0.0.0');
    });
  });
}

async function main() {
  const packageName = process.argv[2];

  if (!packageName) {
    console.error('Usage: node get_npm_version.js <package-name>');
    process.exit(1);
  }

  try {
    const version = await fetchNpmVersion(packageName);
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