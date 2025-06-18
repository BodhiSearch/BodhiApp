import { describe, test, expect, beforeAll, afterEach } from 'vitest';
import {
  loadBindings,
  createTestConfig,
  createTestServer,
  waitForServer,
  sleep,
} from './test-helpers.js';

describe('Integration Tests', () => {
  let bindings;
  const runningServers = [];

  beforeAll(async () => {
    bindings = await loadBindings();
  });

  afterEach(async () => {
    // Clean up any running servers after each test
    for (const server of runningServers) {
      try {
        if (await server.isRunning()) {
          await server.stop();
        }
      } catch (error) {
        console.warn('Failed to stop server during cleanup:', error.message);
      }
    }
    runningServers.length = 0;
  });

  describe('Server Lifecycle Management', () => {
    test('should initially report server as not running', async () => {
      const config = createTestConfig(bindings);
      const server = new bindings.BodhiServer(config);
      runningServers.push(server);

      const isRunning = await server.isRunning();
      expect(isRunning).toBe(false);
    });

    test('should create server with temp dir and verify configuration', () => {
      const server = createTestServer(bindings, { host: '127.0.0.1', port: 28000 });
      runningServers.push(server);

      expect(server.host()).toBe('127.0.0.1');
      expect(server.port()).toBe(28000);
      expect(server.config.envVars[bindings.BODHI_HOME]).toBeDefined();
      expect(server.config.envVars[bindings.BODHI_HOME].length).toBeGreaterThan(0);
      expect(server.serverUrl()).toBe('http://127.0.0.1:28000');
    });

    test('should handle server URL generation correctly', () => {
      const config = createTestConfig(bindings, {
        host: 'localhost',
        port: 9999,
      });
      const server = new bindings.BodhiServer(config);
      runningServers.push(server);

      expect(server.serverUrl()).toBe('http://localhost:9999');
      expect(server.host()).toBe('localhost');
      expect(server.port()).toBe(9999);
    });
  });

  describe('Server Properties and Methods', () => {
    test('should expose correct configuration properties', () => {
      const originalConfig = createTestConfig(bindings, {
        host: 'test-host',
        port: 12345,
      });
      const server = new bindings.BodhiServer(originalConfig);
      runningServers.push(server);

      const serverConfig = server.config;
      expect(serverConfig.envVars[bindings.BODHI_HOST]).toBe('test-host');
      expect(parseInt(serverConfig.envVars[bindings.BODHI_PORT])).toBe(12345);
      expect(serverConfig.envVars[bindings.BODHI_HOME]).toBe(
        originalConfig.envVars[bindings.BODHI_HOME]
      );
    });

    test('should maintain configuration immutability', () => {
      const config = createTestConfig(bindings);
      const server = new bindings.BodhiServer(config);
      runningServers.push(server);

      const originalHost = server.host();
      const originalPort = server.port();
      const originalUrl = server.serverUrl();

      // Verify that the server maintains its configuration
      expect(server.host()).toBe(originalHost);
      expect(server.port()).toBe(originalPort);
      expect(server.serverUrl()).toBe(originalUrl);
    });
  });

  describe('Factory Method Variations', () => {
    test('should create server with specified host and port', () => {
      const host = '0.0.0.0';
      const port = 15000;
      const server = createTestServer(bindings, { host, port });
      runningServers.push(server);

      expect(server.host()).toBe(host);
      expect(server.port()).toBe(port);
    });

    test('should create server with random port when port is not specified', () => {
      const host = 'localhost';
      const server = createTestServer(bindings, { host });
      runningServers.push(server);

      expect(server.host()).toBe(host);
      expect(server.port()).toBeGreaterThanOrEqual(20000);
      expect(server.port()).toBeLessThan(30000);
    });

    test('should create server with default host when host is not specified', () => {
      const port = 16000;
      const server = createTestServer(bindings, { port });
      runningServers.push(server);

      expect(server.host()).toBe('127.0.0.1');
      expect(server.port()).toBe(port);
    });

    test('should create server with random host and port when both are not specified', () => {
      const server = createTestServer(bindings);
      runningServers.push(server);

      expect(server.host()).toBe('127.0.0.1');
      expect(server.port()).toBeGreaterThanOrEqual(20000);
      expect(server.port()).toBeLessThan(30000);
    });
  });

  describe('Configuration Edge Cases', () => {
    test('should handle server with minimal configuration', () => {
      const config = createTestConfig(bindings, {
        host: '127.0.0.1',
        port: 20001,
      });

      const server = new bindings.BodhiServer(config);
      runningServers.push(server);

      expect(server).toBeDefined();
      expect(server.host()).toBe('127.0.0.1');
      expect(server.port()).toBe(20001);
    });

    test('should handle different port ranges', () => {
      const ports = [20000, 25000, 29999];

      for (const port of ports) {
        const config = createTestConfig(bindings, { port });
        const server = new bindings.BodhiServer(config);
        runningServers.push(server);

        expect(server.port()).toBe(port);
      }
    });

    test('should handle various host configurations', () => {
      const hosts = ['127.0.0.1', 'localhost', '0.0.0.0'];

      for (const host of hosts) {
        const config = createTestConfig(bindings, { host });
        const server = new bindings.BodhiServer(config);
        runningServers.push(server);

        expect(server.host()).toBe(host);
      }
    });
  });

  describe('Configuration Structure Validation', () => {
    test('should have proper configuration structure', () => {
      const config = createTestConfig(bindings);
      const server = new bindings.BodhiServer(config);
      runningServers.push(server);

      const serverConfig = server.config;

      // Verify the structure matches NapiAppOptions
      expect(serverConfig.envVars).toBeDefined();
      expect(serverConfig.appSettings).toBeDefined();
      expect(serverConfig.systemSettings).toBeDefined();
      expect(typeof serverConfig.envVars).toBe('object');
      expect(typeof serverConfig.appSettings).toBe('object');
      expect(typeof serverConfig.systemSettings).toBe('object');
    });

    test('should contain required environment variables', () => {
      const config = createTestConfig(bindings);
      const server = new bindings.BodhiServer(config);
      runningServers.push(server);

      const serverConfig = server.config;

      // Check that basic required environment variables are set
      expect(serverConfig.envVars[bindings.BODHI_HOME]).toBeDefined();
      expect(serverConfig.envVars[bindings.BODHI_HOST]).toBeDefined();
      expect(serverConfig.envVars[bindings.BODHI_PORT]).toBeDefined();
      expect(serverConfig.envVars[bindings.BODHI_EXEC_LOOKUP_PATH]).toBeDefined();
    });

    test('should contain required system settings', () => {
      const config = createTestConfig(bindings);
      const server = new bindings.BodhiServer(config);
      runningServers.push(server);

      const serverConfig = server.config;

      // Check that basic required system settings are set
      expect(serverConfig.systemSettings[bindings.BODHI_ENV_TYPE]).toBeDefined();
      expect(serverConfig.systemSettings[bindings.BODHI_APP_TYPE]).toBeDefined();
      expect(serverConfig.systemSettings[bindings.BODHI_VERSION]).toBeDefined();
      expect(serverConfig.systemSettings[bindings.BODHI_AUTH_URL]).toBeDefined();
      expect(serverConfig.systemSettings[bindings.BODHI_AUTH_REALM]).toBeDefined();
    });
  });
});
