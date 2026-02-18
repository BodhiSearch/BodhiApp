#!/usr/bin/env node

const https = require('https');
const { execSync } = require('child_process');
const fs = require('fs');
const path = require('path');
const os = require('os');

function parseArgs() {
  const args = process.argv.slice(2);
  const parsed = {};
  for (let i = 0; i < args.length; i += 2) {
    const key = args[i].replace(/^--/, '');
    parsed[key] = args[i + 1];
  }
  const required = ['target', 'variants', 'extension', 'bin-dir'];
  for (const key of required) {
    if (parsed[key] === undefined) {
      console.error(`Missing required argument: --${key}`);
      console.error('Usage: node download-llama-bins.js --target <target> --variants <v1,v2> --extension <ext> --bin-dir <path>');
      process.exit(1);
    }
  }
  return {
    target: parsed['target'],
    variants: parsed['variants'].split(','),
    extension: parsed['extension'],
    binDir: parsed['bin-dir'],
  };
}

function fetchJSON(url, headers) {
  return new Promise((resolve, reject) => {
    const req = https.get(url, { headers }, (res) => {
      if (res.statusCode === 302 || res.statusCode === 301) {
        return fetchJSON(res.headers.location, headers).then(resolve, reject);
      }
      if (res.statusCode !== 200) {
        let body = '';
        res.on('data', (chunk) => body += chunk);
        res.on('end', () => reject(new Error(`GitHub API returned ${res.statusCode}: ${body}`)));
        return;
      }
      let body = '';
      res.on('data', (chunk) => body += chunk);
      res.on('end', () => {
        try {
          resolve(JSON.parse(body));
        } catch (e) {
          reject(new Error(`Failed to parse JSON: ${e.message}`));
        }
      });
    });
    req.on('error', reject);
  });
}

function downloadFile(url, destPath, headers) {
  return new Promise((resolve, reject) => {
    const doDownload = (downloadUrl, attempt) => {
      const proto = downloadUrl.startsWith('https') ? https : require('http');
      const downloadHeaders = { ...headers, 'Accept': 'application/octet-stream' };
      proto.get(downloadUrl, { headers: downloadHeaders }, (res) => {
        if (res.statusCode === 302 || res.statusCode === 301) {
          return doDownload(res.headers.location, attempt);
        }
        if (!res.statusCode || res.statusCode >= 400) {
          if (attempt < 3) {
            console.log(`  Download failed with status ${res.statusCode} (attempt ${attempt}/3), retrying in 5s...`);
            setTimeout(() => doDownload(downloadUrl, attempt + 1), 5000);
            return;
          }
          reject(new Error(`Failed to download after 3 attempts: ${res.statusCode}`));
          return;
        }
        const file = fs.createWriteStream(destPath);
        res.pipe(file);
        file.on('finish', () => { file.close(); resolve(); });
        file.on('error', reject);
      }).on('error', (err) => {
        if (attempt < 3) {
          console.log(`  Download error: ${err.message} (attempt ${attempt}/3), retrying in 5s...`);
          setTimeout(() => doDownload(downloadUrl, attempt + 1), 5000);
          return;
        }
        reject(err);
      });
    };
    doDownload(url, 1);
  });
}

function execname(extension) {
  return extension ? `llama-server.${extension}` : 'llama-server';
}

async function main() {
  const { target, variants, extension, binDir } = parseArgs();

  const ghPat = process.env.GH_PAT;
  if (!ghPat) {
    console.error('GH_PAT environment variable is not set');
    process.exit(1);
  }

  const headers = {
    'Authorization': `Bearer ${ghPat}`,
    'Accept': 'application/vnd.github.v3+json',
    'X-GitHub-Api-Version': '2022-11-28',
    'User-Agent': 'Bodhi-Build',
  };

  console.log('Fetching releases from BodhiSearch/llama.cpp...');
  const releases = await fetchJSON(
    'https://api.github.com/repos/BodhiSearch/llama.cpp/releases?per_page=100',
    headers
  );

  // Filter to llama-server-standalone releases (tag prefix "server-")
  const serverReleases = releases.filter(r => r.tag_name.startsWith('server-'));
  if (serverReleases.length === 0) {
    console.error('No releases found with tag prefix "server-"');
    process.exit(1);
  }

  // Sort by created_at descending and pick latest with non-empty assets
  serverReleases.sort((a, b) => b.created_at.localeCompare(a.created_at));
  const release = serverReleases.find(r => r.assets && r.assets.length > 0);
  if (!release) {
    console.error('No server releases found with assets');
    process.exit(1);
  }

  console.log(`Selected release: ${release.tag_name} (${release.created_at})`);
  console.log(`Assets: ${release.assets.map(a => a.name).join(', ')}`);

  for (const variant of variants) {
    const prefix = `llama-server--${target}--${variant}`;
    const asset = release.assets.find(a => a.name.startsWith(prefix));
    if (!asset) {
      console.error(`No matching asset for ${prefix}`);
      console.error(`Available assets: ${release.assets.map(a => a.name).join(', ')}`);
      process.exit(1);
    }

    console.log(`\nDownloading ${asset.name}...`);
    const targetDir = path.join(binDir, target, variant);
    fs.mkdirSync(targetDir, { recursive: true });

    const isZip = asset.name.endsWith('.zip');
    const tmpDir = fs.mkdtempSync(path.join(os.tmpdir(), 'llama-bin-'));

    try {
      const downloadPath = path.join(tmpDir, asset.name);
      await downloadFile(asset.browser_download_url, downloadPath, headers);

      if (isZip) {
        // Extract zip
        if (process.platform === 'win32') {
          execSync(`pwsh -Command "Expand-Archive -Path '${downloadPath}' -DestinationPath '${tmpDir}'"`, { stdio: 'inherit' });
        } else {
          execSync(`unzip -o "${downloadPath}" -d "${tmpDir}"`, { stdio: 'inherit' });
        }

        // Move extracted contents (excluding the zip itself) to target dir
        const entries = fs.readdirSync(tmpDir);
        for (const entry of entries) {
          if (entry === asset.name) continue;
          const src = path.join(tmpDir, entry);
          const dest = path.join(targetDir, entry);
          fs.renameSync(src, dest);
          // Set executable permissions on llama-server only
          if (process.platform !== 'win32' && entry === 'llama-server') {
            fs.chmodSync(dest, 0o755);
          }
        }
      } else {
        // Direct file
        const dest = path.join(targetDir, execname(extension));
        fs.renameSync(downloadPath, dest);
        if (process.platform !== 'win32') {
          fs.chmodSync(dest, 0o755);
        }
      }

      console.log(`  Successfully installed ${asset.name} -> ${targetDir}`);
    } finally {
      // Clean up temp dir
      fs.rmSync(tmpDir, { recursive: true, force: true });
    }
  }

  console.log('\nAll binaries downloaded successfully.');
}

main().catch((err) => {
  console.error(`Fatal error: ${err.message}`);
  process.exit(1);
});
