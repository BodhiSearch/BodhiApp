# BodhiApp NAPI Bindings Integration Guide

## What is BodhiApp?

BodhiApp is an AI chat application that provides OpenAI-compatible APIs for local language model inference. The `@bodhiapp/app-bindings` package allows third-party applications to embed a complete BodhiApp server instance programmatically, enabling integration testing and embedded deployment scenarios.

**Core Concept**: Instead of running BodhiApp as a separate process, you can instantiate and control it directly from Node.js code.

## Package Overview

- **Package**: `@bodhiapp/app-bindings` (npm)
- **Technology**: Node.js NAPI bindings (Rust backend via NAPI-RS)
- **Platforms**: Windows x64, macOS ARM64/x64, Linux x64
- **Node.js**: Version 22+ required
- **Current Version**: v0.0.13

```bash
npm install @bodhiapp/app-bindings
```

## Core Architecture

### Configuration System
BodhiApp uses a three-layer immutable configuration system:

1. **Environment Variables** (`envVars`): Runtime settings (host, port, home directory)
2. **App Settings** (`appSettings`): User preferences (logging, binary paths)
3. **System Settings** (`systemSettings`): Deployment configuration (auth URLs, environment type)

### App States
- **`setup`**: Server running but in setup mode, will have the web UI register the App as Resource Server with OAuth
- **`resource_admin`**: App is registered as resource server, but does not have an admin
- **`ready`**: Fully configured with OAuth credentials, API endpoints active

### Key APIs
- **Server Lifecycle**: `start()`, `stop()`, `isRunning()`, `ping()`
- **HTTP Endpoints**: 
  - `/ping` - Health check
  - `/bodhi/v1/info` - Server status and configuration
  - OpenAI-compatible chat completions (when ready)

## Quick Start

### Basic Server Instance

```javascript
import { 
  BodhiServer, 
  createNapiAppOptions, 
  setEnvVar, 
  setAppSetting, 
  setSystemSetting,
  setAppStatus,
  BODHI_HOST, 
  BODHI_PORT,
  BODHI_EXEC_LOOKUP_PATH,
  BODHI_LOG_LEVEL,
  BODHI_ENV_TYPE,
  BODHI_APP_TYPE,
  BODHI_AUTH_URL,
  BODHI_AUTH_REALM
} from '@bodhiapp/app-bindings';

import { mkdtempSync } from 'fs';
import { tmpdir } from 'os';
import { join } from 'path';

// Create configuration (immutable builder pattern)
let config = createNapiAppOptions();

// Environment variables
config = setEnvVar(config, 'HOME', mkdtempSync(join(tmpdir(), 'bodhi-')));
config = setEnvVar(config, BODHI_HOST, '127.0.0.1');
config = setEnvVar(config, BODHI_PORT, '8080');

// App settings
config = setAppSetting(config, BODHI_EXEC_LOOKUP_PATH, '/path/to/llama-binaries');
config = setAppSetting(config, BODHI_LOG_LEVEL, 'info');

// System settings
config = setSystemSetting(config, BODHI_ENV_TYPE, 'development');
config = setSystemSetting(config, BODHI_APP_TYPE, 'container');
config = setSystemSetting(config, BODHI_AUTH_URL, 'https://main-id.getbodhi.app');
config = setSystemSetting(config, BODHI_AUTH_REALM, 'bodhi');

// Set initial state
config = setAppStatus(config, 'setup');

// Create and start server
const server = new BodhiServer(config);
await server.start();

console.log(`Server running at: ${server.serverUrl()}`);

// Test connectivity
const response = await fetch(`${server.serverUrl()}/bodhi/v1/info`);
const info = await response.json();
console.log('Server status:', info.status); // 'setup'

// Cleanup
await server.stop();
```

## Production-Ready Server Manager

For robust applications, use this server manager pattern:

```javascript
// bodhi-server-manager.js
import { mkdtempSync } from 'fs';
import { tmpdir } from 'os';
import { join } from 'path';

class BodhiServerManager {
  constructor(options = {}) {
    this.server = null;
    this.bindings = null;
    this.baseUrl = null;
    this.options = {
      host: '127.0.0.1',
      port: Math.floor(Math.random() * (30000 - 20000) + 20000),
      appStatus: 'ready',
      logLevel: 'info',
      authUrl: 'https://main-id.getbodhi.app',
      authRealm: 'bodhi',
      execLookupPath: '/usr/local/bin', // Adjust for your system
      ...options
    };
  }

  async initialize() {
    if (!this.bindings) {
      const module = await import('@bodhiapp/app-bindings');
      this.bindings = module.default || module;
    }
  }

  async start() {
    await this.initialize();
    
    const tempDir = mkdtempSync(join(tmpdir(), 'bodhi-'));
    let config = this.bindings.createNapiAppOptions();
    
    // Configure server
    config = this.bindings.setEnvVar(config, 'HOME', tempDir);
    config = this.bindings.setEnvVar(config, this.bindings.BODHI_HOST, this.options.host);
    config = this.bindings.setEnvVar(config, this.bindings.BODHI_PORT, this.options.port.toString());
    
    config = this.bindings.setAppSetting(config, this.bindings.BODHI_EXEC_LOOKUP_PATH, this.options.execLookupPath);
    config = this.bindings.setAppSetting(config, this.bindings.BODHI_LOG_LEVEL, this.options.logLevel);
    
    config = this.bindings.setSystemSetting(config, this.bindings.BODHI_ENV_TYPE, 'development');
    config = this.bindings.setSystemSetting(config, this.bindings.BODHI_APP_TYPE, 'container');
    config = this.bindings.setSystemSetting(config, this.bindings.BODHI_AUTH_URL, this.options.authUrl);
    config = this.bindings.setSystemSetting(config, this.bindings.BODHI_AUTH_REALM, this.options.authRealm);
    
    config = this.bindings.setAppStatus(config, this.options.appStatus);
    
    // Add OAuth credentials if provided
    if (this.options.clientId && this.options.clientSecret) {
      config = this.bindings.setClientCredentials(config, this.options.clientId, this.options.clientSecret);
    }

    this.server = new this.bindings.BodhiServer(config);
    await this.server.start();
    
    // Wait for readiness
    await this.waitForReady();
    
    this.baseUrl = this.server.serverUrl();
    return this.baseUrl;
  }

  async waitForReady(maxAttempts = 30, interval = 1000) {
    for (let i = 0; i < maxAttempts; i++) {
      try {
        if (await this.server.isRunning() && await this.server.ping()) {
          return true;
        }
      } catch (error) {
        // Continue waiting
      }
      await new Promise(resolve => setTimeout(resolve, interval));
    }
    throw new Error('Server failed to become ready within timeout');
  }

  async stop() {
    if (this.server && await this.server.isRunning()) {
      await this.server.stop();
    }
    this.server = null;
    this.baseUrl = null;
  }

  async getInfo() {
    if (!this.baseUrl) throw new Error('Server not started');
    const response = await fetch(`${this.baseUrl}/bodhi/v1/info`);
    return await response.json();
  }

  async ping() {
    return this.server ? await this.server.ping() : false;
  }

  getUrl() {
    return this.baseUrl;
  }
}

export { BodhiServerManager };
```

## Integration Testing Patterns

### Vitest Example

```javascript
// integration.test.js
// following is just a temporary test to test if server is setup properly
// do not have this in your actual test package, as it does not make sense to have test for test utils
import { describe, test, expect, afterEach } from 'vitest';
import { BodhiServerManager } from './bodhi-server-manager.js';

describe('BodhiApp Integration', () => {
  const servers = [];

  afterEach(async () => {
    await Promise.all(servers.map(server => server.stop()));
    servers.length = 0;
  });

  test('server lifecycle management', async () => {
    const server = new BodhiServerManager({ appStatus: 'setup' });
    servers.push(server);

    const url = await server.start();
    expect(url).toMatch(/^http:\/\/127\.0\.0\.1:\d+$/);

    const info = await server.getInfo();
    expect(info.status).toBe('setup');
    
    expect(await server.ping()).toBe(true);
  });

  test('ready state with credentials', async () => {
    const server = new BodhiServerManager({
      appStatus: 'ready',
      clientId: 'test-client',
      clientSecret: 'test-secret'
    });
    servers.push(server);

    await server.start();
    const info = await server.getInfo();
    expect(info.status).toBe('ready');
  });
});
```

## Configuration Reference

### Required Configuration

```javascript
// Minimum viable configuration
const config = {
  host: '127.0.0.1',           // Server bind address
  port: 8080,                  // Server port
  execLookupPath: '/usr/local/bin', // Path to llama-server binary
  authUrl: 'https://auth.example.com', // OAuth server
  authRealm: 'your-realm',     // OAuth realm
  appStatus: 'setup'           // Initial state
};
```

### Optional OAuth Integration

```javascript
// For 'ready' state, provide OAuth credentials
const readyConfig = {
  ...config,
  appStatus: 'ready',
  clientId: 'your-oauth-client-id',
  clientSecret: 'your-oauth-client-secret'
};
```

### Configuration Constants

```javascript
// Environment variables
BODHI_HOST, BODHI_PORT, BODHI_LOG_LEVEL, BODHI_LOG_STDOUT

// System settings  
BODHI_ENV_TYPE, BODHI_APP_TYPE, BODHI_VERSION, BODHI_AUTH_URL, BODHI_AUTH_REALM

// App settings
BODHI_EXEC_LOOKUP_PATH, BODHI_KEEP_ALIVE_SECS, HF_HOME
```

## Error Handling

### Common Patterns

```javascript
// Server startup
try {
  await server.start();
} catch (error) {
  if (error.message.includes('port')) {
    console.error('Port conflict - try different port');
  } else if (error.message.includes('binary')) {
    console.error('llama-server binary not found - check BODHI_EXEC_LOOKUP_PATH');
  } else {
    throw error;
  }
}

// Platform compatibility
try {
  const bindings = await import('@bodhiapp/app-bindings');
} catch (error) {
  console.error('Platform not supported. Requires: Windows x64, macOS ARM64/x64, Linux x64');
  throw error;
}

// Configuration validation
try {
  config = setAppStatus(config, 'invalid-status');
} catch (error) {
  console.error('Invalid app status. Valid: setup, ready, resource_admin');
}
```

## Troubleshooting

### Installation Issues
- **Node.js version**: Requires 22+
- **Platform support**: Windows x64, macOS ARM64/x64, Linux x64 only
- **Binary loading**: Check npm installation completed successfully

### Runtime Issues
- **Port conflicts**: Use random ports or check availability
- **Binary path**: Ensure `BODHI_EXEC_LOOKUP_PATH` points to valid llama-server executable
- **Permissions**: Verify write access to temporary directories
- **Auth connectivity**: Validate OAuth server URLs are accessible

### State Management
- **Setup → Ready**: Requires valid OAuth client credentials
- **Configuration immutability**: Always reassign config variables (`config = setEnvVar(config, ...)`)
- **Resource cleanup**: Always call `stop()` in cleanup code

## Best Practices

1. **Use server managers** for complex applications
2. **Random ports** for parallel testing
3. **Proper cleanup** in test teardown
4. **Wait for readiness** before making requests
5. **Immutable configuration** - always reassign setters
6. **Error boundaries** around all server operations
7. **Platform checks** before instantiation
8. **Resource isolation** with temporary directories

This guide enables third-party applications to embed BodhiApp server functionality for integration testing and programmatic deployment scenarios. 