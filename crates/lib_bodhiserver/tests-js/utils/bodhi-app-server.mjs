import { spawn } from 'node:child_process';
import { once } from 'node:events';
import { existsSync } from 'node:fs';
import { expect } from '@playwright/test';
import {
  BODHISERVER_DEV_BIN,
  buildEnvFromConfig,
  pingUntilReady,
  waitForListening,
} from '@/test-helpers.mjs';

/**
 * Spawns the prebuilt `bodhiserver_dev` binary as a child process and exposes
 * lifecycle controls to Playwright tests. Replaces the previous NAPI-based
 * BodhiServer wrapper so iteration on Rust + UI code no longer requires
 * rebuilding the @bodhiapp/app-bindings native module.
 */
export class BodhiAppServer {
  constructor(serverConfig = {}) {
    this.child = null;
    this.baseUrl = null;
    this.serverConfig = serverConfig;
  }

  async startServer() {
    if (!existsSync(BODHISERVER_DEV_BIN)) {
      throw new Error(
        `bodhiserver_dev binary not found at ${BODHISERVER_DEV_BIN}. ` +
          'Build it with `make build.dev-server`.'
      );
    }
    const { env, publicHost, publicPort, publicScheme } = buildEnvFromConfig(this.serverConfig);
    this.child = spawn(BODHISERVER_DEV_BIN, [], {
      env: { ...process.env, ...env },
      stdio: ['ignore', 'pipe', 'inherit'],
    });
    await waitForListening(this.child, 60000);
    const portSuffix =
      (publicScheme === 'http' && publicPort === 80) ||
      (publicScheme === 'https' && publicPort === 443)
        ? ''
        : `:${publicPort}`;
    this.baseUrl = `${publicScheme}://${publicHost}${portSuffix}`;
    const ready = await pingUntilReady(this.baseUrl, 30, 1000);
    expect(ready).toBe(true);
    return this.baseUrl;
  }

  async stopServer() {
    if (!this.child) return;
    if (this.child.exitCode === null) {
      this.child.kill('SIGTERM');
      await once(this.child, 'exit');
    }
    this.child = null;
    this.baseUrl = null;
  }

  getBaseUrl() {
    return this.baseUrl;
  }
}

export function createServerManager(serverConfig = {}) {
  return new BodhiAppServer(serverConfig);
}
