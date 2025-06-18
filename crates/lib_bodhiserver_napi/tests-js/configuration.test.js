import { describe, test, expect, beforeAll } from 'vitest';
import {
  loadBindings,
  createTempDir,
  randomPort,
  createTestConfig,
  createTestServer,
  createFullTestConfig,
} from '../src/utils/test-helpers.js';

describe('Configuration API Tests', () => {
  let bindings;

  beforeAll(async () => {
    bindings = await loadBindings();
  });

  describe('Basic Configuration Creation', () => {
    test('should create basic app config with required parameters', () => {
      const config = bindings.createNapiAppOptions();

      expect(config.envVars).toBeDefined();
      expect(config.appSettings).toBeDefined();
      expect(config.systemSettings).toBeDefined();
      expect(config.clientId).toBeUndefined();
      expect(config.clientSecret).toBeUndefined();
      expect(config.appStatus).toBeUndefined();
    });

    test('should create configuration with environment variables', () => {
      const tempHome = createTempDir();
      const host = '127.0.0.1';
      const port = 25000;

      let config = bindings.createNapiAppOptions();
      config = bindings.setEnvVar(config, bindings.BODHI_HOME, tempHome);
      config = bindings.setEnvVar(config, bindings.BODHI_HOST, host);
      config = bindings.setEnvVar(config, bindings.BODHI_PORT, port.toString());

      expect(config.envVars[bindings.BODHI_HOME]).toBe(tempHome);
      expect(config.envVars[bindings.BODHI_HOST]).toBe(host);
      expect(config.envVars[bindings.BODHI_PORT]).toBe(port.toString());
    });

    test('should create configuration with system settings', () => {
      let config = bindings.createNapiAppOptions();
      config = bindings.setSystemSetting(config, bindings.BODHI_VERSION, '1.0.0');
      config = bindings.setSystemSetting(config, bindings.BODHI_ENV_TYPE, 'development');
      config = bindings.setSystemSetting(config, bindings.BODHI_APP_TYPE, 'native');

      expect(config.systemSettings[bindings.BODHI_VERSION]).toBe('1.0.0');
      expect(config.systemSettings[bindings.BODHI_ENV_TYPE]).toBe('development');
      expect(config.systemSettings[bindings.BODHI_APP_TYPE]).toBe('native');
    });
  });

  describe('Configuration Modification', () => {
    test('should modify environment variables', () => {
      let config = bindings.createNapiAppOptions();
      const host = 'localhost';

      config = bindings.setEnvVar(config, bindings.BODHI_HOST, host);

      expect(config.envVars[bindings.BODHI_HOST]).toBe(host);
    });

    test('should modify port configuration', () => {
      let config = bindings.createNapiAppOptions();
      const port = 9000;

      config = bindings.setEnvVar(config, bindings.BODHI_PORT, port.toString());

      expect(config.envVars[bindings.BODHI_PORT]).toBe(port.toString());
    });

    test('should set exec lookup path', () => {
      let config = bindings.createNapiAppOptions();
      const execPath = '/opt/bin/llama-server';

      config = bindings.setEnvVar(config, bindings.BODHI_EXEC_LOOKUP_PATH, execPath);

      expect(config.envVars[bindings.BODHI_EXEC_LOOKUP_PATH]).toBe(execPath);
    });

    test('should set log level', () => {
      let config = bindings.createNapiAppOptions();
      const logLevel = 'trace';

      config = bindings.setEnvVar(config, bindings.BODHI_LOG_LEVEL, logLevel);

      expect(config.envVars[bindings.BODHI_LOG_LEVEL]).toBe(logLevel);
    });

    test('should set log stdout setting', () => {
      let config = bindings.createNapiAppOptions();
      const logStdout = false;

      config = bindings.setEnvVar(config, bindings.BODHI_LOG_STDOUT, logStdout.toString());

      expect(config.envVars[bindings.BODHI_LOG_STDOUT]).toBe(logStdout.toString());
    });

    test('should add custom environment variables', () => {
      let config = bindings.createNapiAppOptions();
      const key = 'CUSTOM_VAR';
      const value = 'custom_value';

      config = bindings.setEnvVar(config, key, value);

      expect(config.envVars[key]).toBe(value);
    });

    test('should add app settings', () => {
      let config = bindings.createNapiAppOptions();
      const key = 'custom_setting';
      const value = 'setting_value';

      config = bindings.setAppSetting(config, key, value);

      expect(config.appSettings[key]).toBe(value);
    });

    test('should set client credentials', () => {
      let config = bindings.createNapiAppOptions();
      const clientId = 'test-client-id';
      const clientSecret = 'test-client-secret';

      config = bindings.setClientCredentials(config, clientId, clientSecret);

      expect(config.clientId).toBe(clientId);
      expect(config.clientSecret).toBe(clientSecret);
    });

    test('should set app status', () => {
      let config = bindings.createNapiAppOptions();
      const status = 'ready';

      config = bindings.setAppStatus(config, status);

      expect(config.appStatus).toBe(status);
    });

    test('should reject invalid app status', () => {
      let config = bindings.createNapiAppOptions();
      const invalidStatus = 'invalid-status';

      expect(() => {
        bindings.setAppStatus(config, invalidStatus);
      }).toThrow();
    });

    test('should chain configuration modifications', () => {
      let config = bindings.createNapiAppOptions();

      config = bindings.setEnvVar(config, bindings.BODHI_HOST, 'localhost');
      config = bindings.setEnvVar(config, bindings.BODHI_PORT, '25000');
      config = bindings.setEnvVar(config, bindings.BODHI_LOG_LEVEL, 'debug');
      config = bindings.setEnvVar(config, bindings.BODHI_LOG_STDOUT, 'false');
      config = bindings.setEnvVar(config, 'TEST_VAR', 'test_value');
      config = bindings.setAppSetting(config, 'test_setting', 'setting_value');
      config = bindings.setSystemSetting(config, bindings.BODHI_VERSION, '1.0.0');

      expect(config.envVars[bindings.BODHI_HOST]).toBe('localhost');
      expect(config.envVars[bindings.BODHI_PORT]).toBe('25000');
      expect(config.envVars[bindings.BODHI_LOG_LEVEL]).toBe('debug');
      expect(config.envVars[bindings.BODHI_LOG_STDOUT]).toBe('false');
      expect(config.envVars['TEST_VAR']).toBe('test_value');
      expect(config.appSettings['test_setting']).toBe('setting_value');
      expect(config.systemSettings[bindings.BODHI_VERSION]).toBe('1.0.0');
    });
  });

  describe('Constants', () => {
    test('should export configuration constants', () => {
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
    });

    test('should export default values', () => {
      expect(bindings.DEFAULT_HOST).toBe('localhost');
      expect(bindings.DEFAULT_PORT).toBe(1135);
    });
  });

  describe('Server Instance Creation', () => {
    test('should create server instance with basic config', () => {
      const config = createTestConfig(bindings);
      const server = new bindings.BodhiServer(config);

      expect(server).toBeDefined();
      expect(server.host()).toBe(config.envVars[bindings.BODHI_HOST]);
      expect(server.port()).toBe(parseInt(config.envVars[bindings.BODHI_PORT]));
      expect(server.serverUrl()).toContain('http://');
    });

    test('should create server with temp dir using helper method', () => {
      const host = '127.0.0.1';
      const port = randomPort();
      const server = createTestServer(bindings, { host, port });

      expect(server).toBeDefined();
      expect(server.host()).toBe(host);
      expect(server.port()).toBe(port);
      expect(server.config.envVars[bindings.BODHI_HOME]).toBeDefined();
      expect(server.config.envVars[bindings.BODHI_HOME].length).toBeGreaterThan(0);
    });

    test('should create server with random port when not specified', () => {
      const server = createTestServer(bindings, { host: 'localhost' });

      expect(server).toBeDefined();
      expect(server.host()).toBe('localhost');
      expect(server.port()).toBeGreaterThanOrEqual(20000);
      expect(server.port()).toBeLessThan(30000);
    });
  });

  describe('Configuration Validation', () => {
    test('should handle empty configuration correctly', () => {
      const config = bindings.createNapiAppOptions();

      expect(Object.keys(config.envVars)).toHaveLength(0);
      expect(Object.keys(config.appSettings)).toHaveLength(0);
      expect(Object.keys(config.systemSettings)).toHaveLength(0);
    });

    test('should preserve existing values when adding new ones', () => {
      let config = bindings.createNapiAppOptions();

      // Add initial values
      config = bindings.setEnvVar(config, 'EXISTING_KEY', 'existing_value');
      config = bindings.setAppSetting(config, 'existing_setting', 'existing_value');
      config = bindings.setSystemSetting(config, 'existing_system', 'existing_value');

      // Add new values
      config = bindings.setEnvVar(config, 'NEW_KEY', 'new_value');
      config = bindings.setAppSetting(config, 'new_setting', 'new_value');
      config = bindings.setSystemSetting(config, 'new_system', 'new_value');

      // Check that both old and new values exist
      expect(config.envVars['EXISTING_KEY']).toBe('existing_value');
      expect(config.envVars['NEW_KEY']).toBe('new_value');
      expect(config.appSettings['existing_setting']).toBe('existing_value');
      expect(config.appSettings['new_setting']).toBe('new_value');
      expect(config.systemSettings['existing_system']).toBe('existing_value');
      expect(config.systemSettings['new_system']).toBe('new_value');
    });

    test('should handle build validation for complete config', () => {
      const config = createFullTestConfig(bindings);

      // This should not throw an error for a complete config
      expect(() => bindings.buildAppOptions(config)).not.toThrow();
    });
  });
});
