# NAPI Bindings User Guide: @bodhiapp/app-bindings

## Overview

The `@bodhiapp/app-bindings` package provides Node.js NAPI bindings for BodhiApp server functionality, allowing JavaScript/TypeScript applications to programmatically control a Bodhi server instance. Built with NAPI-RS, it exposes the core `lib_bodhiserver` functionality through a clean JavaScript API.

**Key Capabilities:**
- **Server Lifecycle Management**: Start, stop, and monitor Bodhi server instances
- **Flexible Configuration**: Environment variables, app settings, and system settings
- **OAuth Integration**: Client credentials management for authentication
- **Cross-Platform Support**: Native binaries for Windows, macOS, and Linux
- **TypeScript Support**: Full type definitions included

## Installation

### Prerequisites
- **Node.js**: Version 22 or higher required
- **Platform Support**: Windows (x64), macOS (ARM64/x64), Linux (x64)

### Install from npm

```bash
npm install @bodhiapp/app-bindings
```

### Import the Package

```javascript
// ES Modules (recommended)
import { BodhiServer, createNapiAppOptions, setEnvVar, setSystemSetting } from '@bodhiapp/app-bindings';

// CommonJS
const { BodhiServer, createNapiAppOptions, setEnvVar, setSystemSetting } = require('@bodhiapp/app-bindings');
```

## API Structure

### Core Classes

- **`BodhiServer`**: Main server class for lifecycle management
- **`NapiAppOptions`**: Configuration interface with three layers

### Configuration Layers

1. **Environment Variables** (`envVars`): Runtime environment configuration
2. **App Settings** (`appSettings`): User-configurable settings (via settings.yaml)  
3. **System Settings** (`systemSettings`): Immutable system configuration

### Configuration Builder Functions

- **`createNapiAppOptions()`**: Create empty configuration
- **`setEnvVar(config, key, value)`**: Set environment variables
- **`setAppSetting(config, key, value)`**: Set app settings
- **`setSystemSetting(config, key, value)`**: Set system settings
- **`setClientCredentials(config, clientId, clientSecret)`**: Set OAuth credentials
- **`setAppStatus(config, status)`**: Set app status ('ready', 'not_ready', etc.)

## Basic Usage Pattern

### 1. Create Configuration

```javascript
import { 
  createNapiAppOptions, 
  setEnvVar, 
  setSystemSetting,
  setAppSetting,
  BODHI_HOST, 
  BODHI_PORT,
  BODHI_ENV_TYPE,
  BODHI_APP_TYPE,
  BODHI_VERSION,
  BODHI_EXEC_LOOKUP_PATH
} from '@bodhiapp/app-bindings';

// Start with empty configuration
let config = createNapiAppOptions();

// Set environment variables
config = setEnvVar(config, 'HOME', '/tmp/bodhi-test');
config = setEnvVar(config, BODHI_HOST, 'localhost');
config = setEnvVar(config, BODHI_PORT, '8080');

// Set system settings (immutable)
config = setSystemSetting(config, BODHI_ENV_TYPE, 'development');
config = setSystemSetting(config, BODHI_APP_TYPE, 'container');
config = setSystemSetting(config, BODHI_VERSION, '1.0.0');

// Set app settings (configurable)
config = setAppSetting(config, BODHI_EXEC_LOOKUP_PATH, '/path/to/binaries');
```

### 2. Create and Manage Server

```javascript
import { BodhiServer } from '@bodhiapp/app-bindings';

// Create server instance
const server = new BodhiServer(config);

// Server lifecycle
console.log('Server URL:', server.serverUrl()); // http://localhost:8080
console.log('Host:', server.host()); // localhost
console.log('Port:', server.port()); // 8080

// Check initial state
console.log('Running:', await server.isRunning()); // false

// Start server
await server.start();
console.log('Running:', await server.isRunning()); // true

// Test connectivity
const pingResult = await server.ping();
console.log('Ping success:', pingResult); // true

// Stop server
await server.stop();
console.log('Running:', await server.isRunning()); // false
```

## Configuration Constants

The package exports configuration constant names for type safety:

### Environment Variables
```javascript
import { 
  BODHI_HOME,           // 'BODHI_HOME'
  BODHI_HOST,           // 'BODHI_HOST'
  BODHI_PORT,           // 'BODHI_PORT'
  BODHI_LOG_LEVEL,      // 'BODHI_LOG_LEVEL'
  BODHI_LOG_STDOUT,     // 'BODHI_LOG_STDOUT'
  BODHI_EXEC_LOOKUP_PATH // 'BODHI_EXEC_LOOKUP_PATH'
} from '@bodhiapp/app-bindings';
```

### System Configuration
```javascript
import {
  BODHI_ENV_TYPE,       // 'BODHI_ENV_TYPE'
  BODHI_APP_TYPE,       // 'BODHI_APP_TYPE'
  BODHI_VERSION,        // 'BODHI_VERSION'
  BODHI_AUTH_URL,       // 'BODHI_AUTH_URL'
  BODHI_AUTH_REALM      // 'BODHI_AUTH_REALM'
} from '@bodhiapp/app-bindings';
```

### Defaults
```javascript
import { DEFAULT_HOST, DEFAULT_PORT } from '@bodhiapp/app-bindings';
console.log(DEFAULT_HOST); // 'localhost'
console.log(DEFAULT_PORT); // 1135
```

## OAuth Authentication Setup

For applications requiring OAuth authentication:

```javascript
import { setClientCredentials, setSystemSetting } from '@bodhiapp/app-bindings';

let config = createNapiAppOptions();

// Set OAuth configuration
config = setSystemSetting(config, BODHI_AUTH_URL, 'https://main-id.getbodhi.app');
config = setSystemSetting(config, BODHI_AUTH_REALM, 'bodhi');
config = setClientCredentials(config, 'your-client-id', 'your-client-secret');
```

## Complete Example: Test Server

```javascript
import { mkdtempSync } from 'fs';
import { tmpdir } from 'os';
import { join } from 'path';
import { 
  BodhiServer, 
  createNapiAppOptions, 
  setEnvVar, 
  setSystemSetting,
  setAppSetting,
  BODHI_HOST, 
  BODHI_PORT,
  BODHI_ENV_TYPE,
  BODHI_APP_TYPE,
  BODHI_VERSION,
  BODHI_EXEC_LOOKUP_PATH,
  BODHI_LOG_LEVEL
} from '@bodhiapp/app-bindings';

async function createTestServer() {
  // Create temporary directory
  const tempDir = mkdtempSync(join(tmpdir(), 'bodhi-test-'));
  const randomPort = Math.floor(Math.random() * (30000 - 20000) + 20000);
  
  // Build configuration
  let config = createNapiAppOptions();
  
  // Environment variables
  config = setEnvVar(config, 'HOME', tempDir);
  config = setEnvVar(config, BODHI_HOST, '127.0.0.1');
  config = setEnvVar(config, BODHI_PORT, randomPort.toString());
  
  // App settings
  config = setAppSetting(config, BODHI_EXEC_LOOKUP_PATH, '/path/to/llama/binaries');
  config = setAppSetting(config, BODHI_LOG_LEVEL, 'debug');
  
  // System settings
  config = setSystemSetting(config, BODHI_ENV_TYPE, 'development');
  config = setSystemSetting(config, BODHI_APP_TYPE, 'container');
  config = setSystemSetting(config, BODHI_VERSION, '1.0.0-test');
  
  // Create and return server
  return new BodhiServer(config);
}

// Usage
const server = await createTestServer();
console.log(`Server will run at: ${server.serverUrl()}`);

try {
  await server.start();
  console.log('Server started successfully');
  
  // Server operations...
  
} finally {
  await server.stop();
  console.log('Server stopped');
}
```

## Error Handling Patterns

### Server Lifecycle Errors

```javascript
try {
  await server.start();
} catch (error) {
  if (error.message.includes('already running')) {
    console.log('Server is already running');
  } else if (error.message.includes('port in use')) {
    console.log('Port is already in use');
  } else {
    console.error('Failed to start server:', error.message);
  }
}
```

### Configuration Validation

```javascript
try {
  config = setAppStatus(config, 'invalid-status');
} catch (error) {
  console.error('Invalid app status:', error.message);
}
```

### Platform Detection Issues

```javascript
try {
  const { BodhiServer } = require('@bodhiapp/app-bindings');
} catch (error) {
  if (error.message.includes('Failed to load native binding')) {
    console.error('Platform not supported or binary missing');
    console.error('Supported: Windows x64, macOS ARM64/x64, Linux x64');
  }
}
```

## Common Configuration Patterns

### Development Server
```javascript
let config = createNapiAppOptions();
config = setEnvVar(config, BODHI_HOST, 'localhost');
config = setSystemSetting(config, BODHI_ENV_TYPE, 'development');
config = setAppSetting(config, BODHI_LOG_LEVEL, 'debug');
```

### Production Server
```javascript
let config = createNapiAppOptions();
config = setEnvVar(config, BODHI_HOST, '0.0.0.0');
config = setSystemSetting(config, BODHI_ENV_TYPE, 'production');
config = setAppSetting(config, BODHI_LOG_LEVEL, 'info');
```

### Testing Server
```javascript
let config = createNapiAppOptions();
config = setEnvVar(config, BODHI_HOST, '127.0.0.1');
config = setSystemSetting(config, BODHI_ENV_TYPE, 'development');
config = setAppSetting(config, BODHI_LOG_LEVEL, 'debug');
```

## Troubleshooting

### Binary Loading Issues

**Error**: `Failed to load native binding`
- **Cause**: Unsupported platform or missing binary
- **Solution**: Verify your platform is supported (Windows x64, macOS ARM64/x64, Linux x64)
- **Workaround**: Check if `process.platform` and `process.arch` match supported targets

### Server Start Failures

**Error**: Server fails to start
- **Cause**: Port already in use, missing dependencies, or invalid configuration
- **Debug**: Check if `llama-server` binary is available in `BODHI_EXEC_LOOKUP_PATH`
- **Solution**: Use random ports for testing, verify binary dependencies

### Configuration Immutability

**Issue**: Configuration changes not taking effect
- **Cause**: Configuration objects are immutable - each setter returns a new object
- **Solution**: Always assign the return value: `config = setEnvVar(config, key, value)`

### Async Operations

**Issue**: Server state inconsistencies
- **Cause**: Server lifecycle operations are async
- **Solution**: Always `await` server operations and check `isRunning()` status

### Environment Variables

**Issue**: Settings not applied
- **Cause**: Incorrect configuration layer (envVars vs appSettings vs systemSettings)
- **Solution**: 
  - Use `envVars` for runtime environment
  - Use `appSettings` for user-configurable options
  - Use `systemSettings` for immutable system configuration

## Best Practices

1. **Configuration Immutability**: Always assign setter return values
2. **Error Handling**: Wrap server operations in try-catch blocks
3. **Resource Cleanup**: Always stop servers in cleanup/teardown code
4. **Port Management**: Use random ports for testing to avoid conflicts
5. **Type Safety**: Use exported constants instead of string literals
6. **Platform Checking**: Verify platform support before instantiating servers
7. **Async Operations**: Await all server lifecycle methods
8. **Status Monitoring**: Check `isRunning()` before operations

## Integration with Testing Frameworks

### Vitest Example
```javascript
import { describe, test, beforeEach, afterEach, expect } from 'vitest';
import { BodhiServer, createNapiAppOptions } from '@bodhiapp/app-bindings';

describe('Bodhi Server Tests', () => {
  let server;
  
  beforeEach(async () => {
    const config = createTestConfig(); // Your config helper
    server = new BodhiServer(config);
  });
  
  afterEach(async () => {
    if (await server.isRunning()) {
      await server.stop();
    }
  });
  
  test('should start and stop server', async () => {
    expect(await server.isRunning()).toBe(false);
    await server.start();
    expect(await server.isRunning()).toBe(true);
    await server.stop();
    expect(await server.isRunning()).toBe(false);
  });
});
```

This guide provides the essential patterns for integrating Bodhi server functionality into Node.js applications through the NAPI bindings. 