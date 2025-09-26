import { afterEach, beforeAll, describe, expect, test } from 'vitest';
import { createTestServer, loadBindings } from '@/test-helpers.mjs';

describe('Integration Tests', () => {
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
  });

  describe('Server Configuration and Properties', () => {
    test('should create server with proper configuration structure and URL generation', () => {
      const server = createTestServer(bindings, { host: 'test-host', port: 12345 });
      runningServers.push(server);

      // Test configuration structure
      expect(server.config.envVars).toBeDefined();
      expect(server.config.appSettings).toBeDefined();
      expect(server.config.systemSettings).toBeDefined();
      expect(typeof server.config.envVars).toBe('object');
      expect(typeof server.config.appSettings).toBe('object');
      expect(typeof server.config.systemSettings).toBe('object');

      // Test server properties
      expect(server.host()).toBe('test-host');
      expect(server.port()).toBe(12345);
      expect(server.serverUrl()).toBe('http://test-host:12345');
      expect(server.config.envVars['HOME']).toBeDefined();
      expect(server.config.envVars[bindings.BODHI_HOST]).toBe('test-host');
      expect(Number.parseInt(server.config.envVars[bindings.BODHI_PORT])).toBe(12345);
    });

    test('should create server with test helpers and maintain configuration immutability', () => {
      const server = createTestServer(bindings, { host: '127.0.0.1', port: 28000 });
      runningServers.push(server);

      // Test helper-created server
      expect(server.host()).toBe('127.0.0.1');
      expect(server.port()).toBe(28000);
      expect(server.serverUrl()).toBe('http://127.0.0.1:28000');
      expect(server.config.envVars['HOME']).toBeDefined();

      // Test immutability - values should remain constant
      const originalHost = server.host();
      const originalPort = server.port();
      const originalUrl = server.serverUrl();

      expect(server.host()).toBe(originalHost);
      expect(server.port()).toBe(originalPort);
      expect(server.serverUrl()).toBe(originalUrl);
    });

    test('should handle default and random value generation properly', () => {
      // Test with default host
      const serverWithDefaultHost = createTestServer(bindings, { port: 16000 });
      runningServers.push(serverWithDefaultHost);
      expect(serverWithDefaultHost.host()).toBe('localhost');
      expect(serverWithDefaultHost.port()).toBe(16000);

      // Test with random port
      const serverWithRandomPort = createTestServer(bindings, { host: '127.0.0.1' });
      runningServers.push(serverWithRandomPort);
      expect(serverWithRandomPort.host()).toBe('127.0.0.1');
      expect(serverWithRandomPort.port()).toBeGreaterThanOrEqual(20000);
      expect(serverWithRandomPort.port()).toBeLessThan(30000);

      // Test with both defaults
      const serverWithDefaults = createTestServer(bindings);
      runningServers.push(serverWithDefaults);
      expect(serverWithDefaults.host()).toBe('localhost');
      expect(serverWithDefaults.port()).toBeGreaterThanOrEqual(20000);
      expect(serverWithDefaults.port()).toBeLessThan(30000);
    });
  });

  describe('Server Lifecycle and State Management', () => {
    test('should initially report not running and maintain proper server state', async () => {
      const server = createTestServer(bindings, { host: '127.0.0.1', port: 20001 });
      runningServers.push(server);

      // Initially should not be running
      expect(await server.isRunning()).toBe(false);

      // Should maintain not-running state consistently
      expect(await server.isRunning()).toBe(false);
    });

    test('should contain all required configuration variables for complete setup', () => {
      const server = createTestServer(bindings, { host: 'test-host', port: 12345 });
      runningServers.push(server);

      const serverConfig = server.config;

      // Required environment variables
      expect(serverConfig.envVars['HOME']).toBeDefined();
      expect(serverConfig.envVars[bindings.BODHI_HOST]).toBeDefined();
      expect(serverConfig.envVars[bindings.BODHI_PORT]).toBeDefined();

      // For full test config, also check system settings
      const fullTestServer = createTestServer(bindings);
      runningServers.push(fullTestServer);

      const fullConfig = fullTestServer.config;
      expect(fullConfig.systemSettings[bindings.BODHI_ENV_TYPE]).toBeDefined();
      expect(fullConfig.systemSettings[bindings.BODHI_APP_TYPE]).toBeDefined();
      expect(fullConfig.systemSettings[bindings.BODHI_VERSION]).toBeDefined();
      expect(fullConfig.systemSettings[bindings.BODHI_AUTH_URL]).toBeDefined();
      expect(fullConfig.systemSettings[bindings.BODHI_AUTH_REALM]).toBeDefined();
    });
  });
});
