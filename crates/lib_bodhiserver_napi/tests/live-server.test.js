import { afterEach, beforeAll, describe, expect, test } from 'vitest';
import { createTestServer, loadBindings, sleep, waitForServer } from './test-helpers.mjs';

// These tests require the actual llama-server binary to be available
// Tests will fail definitively if server cannot be started
describe('Live Server Tests', () => {
  let bindings;
  const runningServers = [];

  beforeAll(async () => {
    bindings = await loadBindings();
  });

  afterEach(async () => {
    // Clean up any running servers after each test
    for (const server of runningServers) {
      const isRunning = await server.isRunning();
      if (isRunning) {
        await server.stop();
      }
    }
    runningServers.length = 0;
    await sleep(1000);
  });

  describe('Server Lifecycle Operations', () => {
    test('should start server, verify running state, and handle ping', async () => {
      const server = createTestServer(bindings, { host: '127.0.0.1', port: 27001 });
      runningServers.push(server);

      // Initially should not be running
      expect(await server.isRunning()).toBe(false);

      // Start the server
      await server.start();

      // Should now report as running
      expect(await server.isRunning()).toBe(true);

      // Wait for server to be fully ready
      await sleep(2000);

      // Should still be running
      expect(await server.isRunning()).toBe(true);

      // Test ping functionality
      const pingResponse = await server.ping();
      expect(typeof pingResponse).toBe('boolean');
      expect(pingResponse).toBe(true);
    });

    test('should start and stop server with proper state transitions', async () => {
      const server = createTestServer(bindings, { host: '127.0.0.1', port: 27002 });
      runningServers.push(server);

      // Start the server
      await server.start();
      expect(await server.isRunning()).toBe(true);

      // Stop the server
      await server.stop();
      expect(await server.isRunning()).toBe(false);
    });

    test('should handle multiple servers on different ports simultaneously', async () => {
      const server1 = createTestServer(bindings, { host: '127.0.0.1', port: 27007 });
      const server2 = createTestServer(bindings, { host: '127.0.0.1', port: 27008 });
      runningServers.push(server1, server2);

      // Start both servers
      await server1.start();
      await server2.start();

      // Both should be running
      expect(await server1.isRunning()).toBe(true);
      expect(await server2.isRunning()).toBe(true);

      // Verify they have different URLs
      expect(server1.serverUrl()).not.toBe(server2.serverUrl());
      expect(server1.port()).not.toBe(server2.port());
    });
  });

  describe('Server Error Conditions and Edge Cases', () => {
    test('should prevent starting server twice and throw error', async () => {
      const server = createTestServer(bindings, { host: '127.0.0.1', port: 27004 });
      runningServers.push(server);

      // Start the server
      await server.start();
      expect(await server.isRunning()).toBe(true);

      // Try to start again - should fail
      await expect(server.start()).rejects.toThrow();
    });

    test('should handle operations on non-running server gracefully', async () => {
      const server = createTestServer(bindings, { host: '127.0.0.1', port: 27005 });
      runningServers.push(server);

      // Server is not started, ping should return false
      const pingResponse = await server.ping();
      expect(pingResponse).toBe(false);

      // Server is not started, stop should not throw
      await server.stop();
      expect(await server.isRunning()).toBe(false);
    });

    test('should respect custom configuration when starting server', async () => {
      const server = createTestServer(bindings, {
        host: '127.0.0.1',
        port: 27009,
        envVars: { BODHI_LOG_LEVEL: 'debug' },
      });
      runningServers.push(server);

      // Verify configuration is preserved
      expect(server.host()).toBe('127.0.0.1');
      expect(server.port()).toBe(27009);
      expect(server.config.envVars['BODHI_LOG_LEVEL']).toBe('debug');

      await server.start();
      expect(await server.isRunning()).toBe(true);
    });
  });

  describe('Embeddings Endpoint Tests', () => {
    test('should handle model not found error for non-existent model', async () => {
      const server = createTestServer(bindings, { host: '127.0.0.1', port: 27010 });
      runningServers.push(server);

      await server.start();
      expect(await server.isRunning()).toBe(true);

      await sleep(2000);

      const response = await fetch(`${server.serverUrl()}/v1/embeddings`, {
        method: 'POST',
        headers: {
          'Content-Type': 'application/json',
        },
        body: JSON.stringify({
          model: 'non-existent-model-xyz',
          input: 'Hello world',
        }),
      });

      expect(response.status).toBe(404);
    });

    test('should handle invalid request with missing required input field', async () => {
      const server = createTestServer(bindings, { host: '127.0.0.1', port: 27011 });
      runningServers.push(server);

      await server.start();
      expect(await server.isRunning()).toBe(true);

      await sleep(2000);

      const response = await fetch(`${server.serverUrl()}/v1/embeddings`, {
        method: 'POST',
        headers: {
          'Content-Type': 'application/json',
        },
        body: JSON.stringify({
          model: 'test-model',
        }),
      });

      expect(response.status).toBeGreaterThanOrEqual(400);
      expect(response.status).toBeLessThan(500);
    });

    test('should handle invalid request with malformed JSON', async () => {
      const server = createTestServer(bindings, { host: '127.0.0.1', port: 27012 });
      runningServers.push(server);

      await server.start();
      expect(await server.isRunning()).toBe(true);

      await sleep(2000);

      const response = await fetch(`${server.serverUrl()}/v1/embeddings`, {
        method: 'POST',
        headers: {
          'Content-Type': 'application/json',
        },
        body: 'invalid json',
      });

      expect(response.status).toBeGreaterThanOrEqual(400);
      expect(response.status).toBeLessThan(500);
    });

    test('should verify embeddings endpoint is registered', async () => {
      const server = createTestServer(bindings, { host: '127.0.0.1', port: 27013 });
      runningServers.push(server);

      await server.start();
      expect(await server.isRunning()).toBe(true);

      await sleep(2000);

      const response = await fetch(`${server.serverUrl()}/v1/embeddings`, {
        method: 'POST',
        headers: {
          'Content-Type': 'application/json',
        },
        body: JSON.stringify({
          model: 'any-model',
          input: 'test',
        }),
      });

      expect([401, 404]).toContain(response.status);
    });
  });
});
