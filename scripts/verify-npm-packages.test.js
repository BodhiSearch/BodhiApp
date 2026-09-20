const assert = require('node:assert/strict');
const fs = require('node:fs');
const os = require('node:os');
const path = require('node:path');
const test = require('node:test');

const { MAX_ATTEMPTS, verifyPackages } = require('./verify-npm-packages');

function createFixture(t, { name = '@bodhiapp/test-package', platforms = [] } = {}) {
  const cwd = fs.mkdtempSync(path.join(os.tmpdir(), 'verify-npm-packages-'));
  t.after(() => fs.rmSync(cwd, { recursive: true, force: true }));

  fs.writeFileSync(path.join(cwd, 'package.json'), JSON.stringify({ name, version: '1.2.3' }), 'utf8');

  for (const platform of platforms) {
    fs.mkdirSync(path.join(cwd, 'npm', platform), { recursive: true });
  }

  return cwd;
}

function createLogger() {
  return {
    log() {},
    error() {},
  };
}

test('succeeds immediately when npm reports the expected version', async t => {
  const cwd = createFixture(t);
  const checkedPackages = [];
  let waitCount = 0;

  const verified = await verifyPackages({
    cwd,
    expectedVersion: '1.2.3',
    getPackageVersion(packageName) {
      checkedPackages.push(packageName);
      return '1.2.3';
    },
    wait: async () => {
      waitCount += 1;
    },
    logger: createLogger(),
  });

  assert.equal(verified, true);
  assert.deepEqual(checkedPackages, ['@bodhiapp/test-package']);
  assert.equal(waitCount, 0);
});

test('retries until npm reports the expected version', async t => {
  const cwd = createFixture(t);
  const publishedVersions = ['1.2.2', '1.2.3'];
  let waitCount = 0;

  const verified = await verifyPackages({
    cwd,
    expectedVersion: '1.2.3',
    getPackageVersion() {
      return publishedVersions.shift();
    },
    wait: async () => {
      waitCount += 1;
    },
    logger: createLogger(),
  });

  assert.equal(verified, true);
  assert.equal(waitCount, 1);
});

for (const [description, publishedVersion] of [
  ['is missing', null],
  ['reports a stale version', '1.2.2'],
]) {
  test(`fails after 10 attempts when the package ${description}`, async t => {
    const cwd = createFixture(t);
    let checkCount = 0;
    let waitCount = 0;

    const verified = await verifyPackages({
      cwd,
      expectedVersion: '1.2.3',
      getPackageVersion() {
        checkCount += 1;
        return publishedVersion;
      },
      wait: async () => {
        waitCount += 1;
      },
      logger: createLogger(),
    });

    assert.equal(verified, false);
    assert.equal(checkCount, MAX_ATTEMPTS);
    assert.equal(waitCount, MAX_ATTEMPTS - 1);
  });
}

test('waits for every platform package without rechecking verified packages', async t => {
  const cwd = createFixture(t, { platforms: ['darwin-arm64', 'linux-x64-gnu'] });
  const checkCounts = new Map();

  const verified = await verifyPackages({
    cwd,
    expectedVersion: '1.2.3',
    getPackageVersion(packageName) {
      const checkCount = (checkCounts.get(packageName) || 0) + 1;
      checkCounts.set(packageName, checkCount);
      if (packageName.endsWith('-linux-x64-gnu') && checkCount === 1) {
        return '1.2.2';
      }
      return '1.2.3';
    },
    wait: async () => {},
    logger: createLogger(),
  });

  assert.equal(verified, true);
  assert.equal(checkCounts.get('@bodhiapp/test-package'), 1);
  assert.equal(checkCounts.get('@bodhiapp/test-package-darwin-arm64'), 1);
  assert.equal(checkCounts.get('@bodhiapp/test-package-linux-x64-gnu'), 2);
});
