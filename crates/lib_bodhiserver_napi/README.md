# BodhiApp Server NAPI Bindings

This crate provides Node.js bindings for the BodhiApp server through NAPI-RS, enabling JavaScript/TypeScript applications to configure and launch the Bodhi server programmatically.

## Features

- ✅ **Configuration Management**: Full configuration API for server settings
- ✅ **Server Lifecycle**: Start, stop, and monitor server instances
- ✅ **Type Safety**: Complete TypeScript definitions included
- ✅ **Cross-Platform**: Supports macOS, Linux, Windows
- ✅ **Async/Await**: Modern JavaScript async patterns
- ✅ **Factory Methods**: Convenient server creation methods

## Installation

```bash
npm install bodhiapp-server-bindings
```

## Usage

### Basic Configuration

```javascript
const {
  BodhiServer,
  createServerConfig,
  setServerHost,
  setServerPort,
  addEnvVar
} = require('bodhiapp-server-bindings');

// Create and configure server
let config = createServerConfig();
config = setServerHost(config, 'localhost');
config = setServerPort(config, 8080);
config = addEnvVar(config, 'BODHI_ENCRYPTION_KEY', 'your-key');

const server = new BodhiServer(config);
```

### Server Lifecycle

```javascript
async function runServer() {
  // Create server with temporary directory
  const server = BodhiServer.withTempDir();
  
  try {
    // Start the server
    await server.start();
    console.log('Server running at:', server.serverUrl());
    
    // Check if server is running
    const isRunning = await server.isRunning();
    console.log('Server status:', isRunning);
    
    // Test connectivity
    const response = await server.ping();
    console.log('Ping response:', response);
    
  } finally {
    // Always stop the server
    await server.stop();
  }
}
```

### Playwright Integration

This library is designed for UI testing with Playwright:

```javascript
const { test, expect } = require('@playwright/test');
const { BodhiServer } = require('bodhiapp-server-bindings');

test('BodhiApp UI test', async ({ page }) => {
  const server = BodhiServer.withTempDir();
  
  try {
    await server.start();
    const serverUrl = server.serverUrl();
    
    // Navigate to the server
    await page.goto(serverUrl);
    
    // Your UI tests here
    await expect(page).toHaveTitle(/BodhiApp/);
    
  } finally {
    await server.stop();
  }
});
```

## API Reference

### Configuration Functions

- `createServerConfig()`: Create default server configuration
- `createServerConfigWithHome(path)`: Create config with specific bodhi_home
- `setServerHost(config, host)`: Set server host
- `setServerPort(config, port)`: Set server port  
- `setLogLevel(config, level)`: Set log level
- `addEnvVar(config, key, value)`: Add environment variable
- `getServerUrl(config)`: Get server URL

### BodhiServer Class

#### Factory Methods
- `new BodhiServer(config)`: Create with custom config
- `BodhiServer.withDefaults()`: Create with default config
- `BodhiServer.withTempDir()`: Create with temporary directory

#### Properties
- `server.config`: Get server configuration
- `server.host()`: Get server host
- `server.port()`: Get server port
- `server.serverUrl()`: Get complete server URL

#### Async Methods
- `server.start()`: Start the server (async)
- `server.stop()`: Stop the server (async)
- `server.isRunning()`: Check if server is running (async)
- `server.ping()`: Test server connectivity (async)

## Development

### Building from Source

```bash
# Install dependencies
npm install

# Build debug version
npm run build:debug

# Build release version
npm run build

# Run tests
npm test
```

### Testing

```bash
# Run configuration tests (no server startup required)
node simple-test.js

# Run full integration tests (requires server dependencies)
node test.js
```

## Requirements

- Node.js 16+
- Rust toolchain (for building from source)
- Platform-specific server dependencies (for full server functionality)

## Architecture

This NAPI bridge wraps the `lib_bodhiserver` Rust crate, providing the same configuration and server management capabilities available to other Bodhi clients (native, container, etc.) but accessible from JavaScript.

The bridge uses the same underlying:
- Configuration management system
- App service builder pattern  
- Server lifecycle management
- Error handling and logging

## License

MIT License - see LICENSE file for details. 