import { beforeAll, describe, expect, test } from 'vitest';
import {
  createFullTestConfig,
  createTempDir,
  createTestServer,
  loadBindings,
} from './test-helpers.js';

describe('Configuration API Tests', () => {
  let bindings;

  beforeAll(async () => {
    bindings = await loadBindings();
  });

  describe('Configuration Creation and Modification', () => {
    test('should create and modify complete configuration with all options', () => {
      let config = bindings.createNapiAppOptions();

      // Test initial empty state
      expect(config.envVars).toBeDefined();
      expect(config.appSettings).toBeDefined();
      expect(config.systemSettings).toBeDefined();
      expect(Object.keys(config.envVars)).toHaveLength(0);

      // Test environment variables
      config = bindings.setEnvVar(config, bindings.BODHI_HOME, createTempDir());
      config = bindings.setEnvVar(config, bindings.BODHI_HOST, 'localhost');
      config = bindings.setEnvVar(config, bindings.BODHI_PORT, '25000');
      config = bindings.setEnvVar(config, bindings.BODHI_LOG_LEVEL, 'debug');
      config = bindings.setEnvVar(config, 'CUSTOM_VAR', 'custom_value');

      // Test app settings
      config = bindings.setAppSetting(config, 'test_setting', 'setting_value');

      // Test system settings
      config = bindings.setSystemSetting(config, bindings.BODHI_VERSION, '1.0.0');
      config = bindings.setSystemSetting(config, bindings.BODHI_ENV_TYPE, 'development');
      config = bindings.setSystemSetting(config, bindings.BODHI_APP_TYPE, 'native');

      // Test client credentials
      config = bindings.setClientCredentials(config, 'test-client-id', 'test-client-secret');

      // Test app status
      config = bindings.setAppStatus(config, 'ready');

      // Verify all modifications
      expect(config.envVars[bindings.BODHI_HOST]).toBe('localhost');
      expect(config.envVars[bindings.BODHI_PORT]).toBe('25000');
      expect(config.envVars[bindings.BODHI_LOG_LEVEL]).toBe('debug');
      expect(config.envVars['CUSTOM_VAR']).toBe('custom_value');
      expect(config.appSettings['test_setting']).toBe('setting_value');
      expect(config.systemSettings[bindings.BODHI_VERSION]).toBe('1.0.0');
      expect(config.systemSettings[bindings.BODHI_ENV_TYPE]).toBe('development');
      expect(config.systemSettings[bindings.BODHI_APP_TYPE]).toBe('native');
      expect(config.clientId).toBe('test-client-id');
      expect(config.clientSecret).toBe('test-client-secret');
      expect(config.appStatus).toBe('ready');
    });

    test('should reject invalid app status', () => {
      const config = bindings.createNapiAppOptions();

      expect(() => {
        bindings.setAppStatus(config, 'invalid-status');
      }).toThrow();
    });

    test('should preserve existing values when adding new configuration', () => {
      let config = bindings.createNapiAppOptions();

      // Add initial values
      config = bindings.setEnvVar(config, 'EXISTING_KEY', 'existing_value');
      config = bindings.setAppSetting(config, 'existing_setting', 'existing_value');
      config = bindings.setSystemSetting(config, 'existing_system', 'existing_value');

      // Add new values
      config = bindings.setEnvVar(config, 'NEW_KEY', 'new_value');
      config = bindings.setAppSetting(config, 'new_setting', 'new_value');
      config = bindings.setSystemSetting(config, 'new_system', 'new_value');

      // Verify preservation
      expect(config.envVars['EXISTING_KEY']).toBe('existing_value');
      expect(config.envVars['NEW_KEY']).toBe('new_value');
      expect(config.appSettings['existing_setting']).toBe('existing_value');
      expect(config.appSettings['new_setting']).toBe('new_value');
      expect(config.systemSettings['existing_system']).toBe('existing_value');
      expect(config.systemSettings['new_system']).toBe('new_value');
    });
  });

  describe('Configuration Constants', () => {
    test('should export all required configuration constants and defaults', () => {
      // Environment variable constants
      expect(bindings.BODHI_HOME).toBe('BODHI_HOME');
      expect(bindings.BODHI_HOST).toBe('BODHI_HOST');
      expect(bindings.BODHI_PORT).toBe('BODHI_PORT');
      expect(bindings.BODHI_LOG_LEVEL).toBe('BODHI_LOG_LEVEL');
      expect(bindings.BODHI_LOG_STDOUT).toBe('BODHI_LOG_STDOUT');
      expect(bindings.BODHI_EXEC_LOOKUP_PATH).toBe('BODHI_EXEC_LOOKUP_PATH');
      expect(bindings.BODHI_ENV_TYPE).toBe('BODHI_ENV_TYPE');
      expect(bindings.BODHI_APP_TYPE).toBe('BODHI_APP_TYPE');
      expect(bindings.BODHI_VERSION).toBe('BODHI_VERSION');
      expect(bindings.BODHI_AUTH_URL).toBe('BODHI_AUTH_URL');
      expect(bindings.BODHI_AUTH_REALM).toBe('BODHI_AUTH_REALM');

      // Default values
      expect(bindings.DEFAULT_HOST).toBe('localhost');
      expect(bindings.DEFAULT_PORT).toBe(1135);
    });
  });

  describe('Server Instance Creation', () => {
    test('should create server instance with proper configuration structure', () => {
      const server = createTestServer(bindings, { host: 'test-host', port: 12345 });

      expect(server).toBeDefined();
      expect(server.host()).toBe('test-host');
      expect(server.port()).toBe(12345);
      expect(server.serverUrl()).toBe('http://test-host:12345');
    });

    test('should create server with test helper methods and random values', () => {
      const server = createTestServer(bindings, { host: 'localhost' });

      expect(server).toBeDefined();
      expect(server.host()).toBe('localhost');
      expect(server.port()).toBeGreaterThanOrEqual(20000);
      expect(server.port()).toBeLessThan(30000);
      expect(server.serverUrl()).toContain('http://localhost:');
    });

    test('should validate complete configuration structure', () => {
      const config = createFullTestConfig(bindings);

      // Verify all required sections are present
      expect(config.envVars['HOME']).toBeDefined();
      expect(config.systemSettings[bindings.BODHI_ENV_TYPE]).toBeDefined();
      expect(config.systemSettings[bindings.BODHI_VERSION]).toBeDefined();

      // Verify config can be used to create a server
      const server = new bindings.BodhiServer(config);
      expect(server).toBeDefined();
      expect(server.host()).toBeDefined();
      expect(server.port()).toBeDefined();
    });
  });
});
