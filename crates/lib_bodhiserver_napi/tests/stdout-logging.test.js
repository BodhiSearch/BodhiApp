import { afterEach, beforeAll, describe, expect, test } from 'vitest';
import { createTestServer, loadBindings, sleep } from './test-helpers.mjs';

describe('STDOUT Logging Tests', () => {
  let bindings;
  const runningServers = [];

  beforeAll(async () => {
    bindings = await loadBindings();
  });

  afterEach(async () => {
    for (const server of runningServers) {
      const isRunning = await server.isRunning();
      if (isRunning) {
        await server.stop();
      }
    }
    runningServers.length = 0;
  });

  test('should start server with logStdout enabled and respond to ping', async () => {
    const server = createTestServer(bindings, {
      host: '127.0.0.1',
      port: 27101,
      logStdout: true,
      logLevel: 'info',
    });
    runningServers.push(server);

    await server.start();
    expect(await server.isRunning()).toBe(true);

    const pingResult = await server.ping();
    expect(pingResult).toBe(true);

    expect(await server.isRunning()).toBe(true);
  });

  test('should start server with logStdout disabled and respond to ping', async () => {
    const server = createTestServer(bindings, {
      host: '127.0.0.1',
      port: 27102,
      logStdout: false,
      logLevel: 'info',
    });
    runningServers.push(server);

    await server.start();
    expect(await server.isRunning()).toBe(true);

    const pingResult = await server.ping();
    expect(pingResult).toBe(true);

    expect(await server.isRunning()).toBe(true);
  });

  test('should handle debug log level with stdout logging', async () => {
    const server = createTestServer(bindings, {
      host: '127.0.0.1',
      port: 27103,
      logStdout: true,
      logLevel: 'debug',
    });
    runningServers.push(server);

    await server.start();
    expect(await server.isRunning()).toBe(true);

    for (let i = 0; i < 3; i++) {
      const pingResult = await server.ping();
      expect(pingResult).toBe(true);
    }

    expect(await server.isRunning()).toBe(true);
  });

  test('should start and stop server cleanly with logging enabled', async () => {
    const server = createTestServer(bindings, {
      host: '127.0.0.1',
      port: 27104,
      logStdout: true,
      logLevel: 'info',
    });
    runningServers.push(server);

    await server.start();
    expect(await server.isRunning()).toBe(true);

    expect(await server.ping()).toBe(true);

    await server.stop();
    expect(await server.isRunning()).toBe(false);

    expect(await server.ping()).toBe(false);
  });
});
