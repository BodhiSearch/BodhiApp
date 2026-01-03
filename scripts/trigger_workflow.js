#!/usr/bin/env node

const { execSync } = require('child_process');

const TARGETS = {
  'docker': {
    tagPattern: 'docker/v*',
    workflows: ['publish-docker.yml', 'publish-docker-multiplatform.yml']
  },
  'docker-dev': {
    tagPattern: 'docker-dev/v*',
    workflows: ['publish-docker.yml', 'publish-docker-multiplatform.yml']
  },
  'ts-client': {
    tagPattern: 'ts-client/v*',
    workflows: ['publish-ts-client.yml']
  },
  'bodhi-app-bindings': {
    tagPattern: 'bodhi-app-bindings/v*',
    workflows: ['publish-app-bindings.yml']
  },
  'app': {
    tagPattern: 'app/v*',
    workflows: ['release.yml']
  },
  'website': {
    tagPattern: 'getbodhi.app/v*',
    workflows: ['deploy-website.yml']
  }
};

function exec(command) {
  return execSync(command, { encoding: 'utf8' }).trim();
}

function getLatestTag(pattern) {
  try {
    return exec(`git tag -l '${pattern}' --sort=-version:refname | head -n 1`);
  } catch (error) {
    console.error(`Error finding tag matching ${pattern}:`, error.message);
    process.exit(1);
  }
}

function triggerWorkflow(workflow, tag, target) {
  console.log(`Triggering ${workflow} with tag=${tag}`);
  try {
    if (workflow === 'deploy-website.yml') {
      const version = tag.replace('getbodhi.app/v', '');
      exec(`gh workflow run ${workflow} -f version=${version}`);
    } else if (workflow === 'release.yml') {
      exec(`gh workflow run ${workflow} -f tag=${tag} -f draft=no -f prerelease=no`);
    } else {
      exec(`gh workflow run ${workflow} -f tag=${tag}`);
    }
    console.log(`  ✓ Triggered successfully`);
  } catch (error) {
    console.error(`  ✗ Failed to trigger ${workflow}:`, error.message);
    process.exit(1);
  }
}

function main() {
  const target = process.argv[2];

  if (!target) {
    console.error('Error: Target is required');
    console.error('Usage: ./scripts/trigger_workflow.js <target>');
    console.error('\nAvailable targets:');
    Object.keys(TARGETS).forEach(t => console.error(`  - ${t}`));
    process.exit(1);
  }

  const config = TARGETS[target];
  if (!config) {
    console.error(`Error: Unknown target "${target}"`);
    console.error('Available targets:', Object.keys(TARGETS).join(', '));
    process.exit(1);
  }

  console.log(`\nFinding latest tag for ${target} (${config.tagPattern})...`);
  const latestTag = getLatestTag(config.tagPattern);

  if (!latestTag) {
    console.error(`Error: No tags found matching ${config.tagPattern}`);
    process.exit(1);
  }

  console.log(`Found latest tag: ${latestTag}\n`);

  config.workflows.forEach(workflow => {
    triggerWorkflow(workflow, latestTag, target);
  });

  console.log(`\n✓ All workflows triggered successfully`);
  console.log(`Monitor at: https://github.com/BodhiSearch/BodhiApp/actions`);
}

main();
