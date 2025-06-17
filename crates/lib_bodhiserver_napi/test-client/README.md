# lib_bodhiserver_napi Test Client

This is a test client for the `lib_bodhiserver_napi` crate that demonstrates the JavaScript/TypeScript interface for the enhanced configuration builder.

## Overview

The test client validates:

1. **Enhanced Configuration**: Tests the new configuration fields including environment variables, app settings, system settings, OAuth credentials, and app status
2. **Basic Configuration**: Tests minimal configuration setup
3. **Ping Endpoint**: Tests the `/ping` endpoint functionality

## Test Files

- `test.js` - Main test suite for enhanced and basic configuration
- `test-ping.js` - Ping endpoint test (requires server startup)
- `package.json` - Node.js project configuration

## Enhanced Configuration Features

The enhanced configuration supports the following new fields:

```javascript
const config = {
  // Basic fields (required)
  env_type: 'development',
  app_type: 'container',
  app_version: '1.0.0-test',
  auth_url: 'https://dev-id.getbodhi.app',
  auth_realm: 'bodhi',
  
  // Enhanced fields (optional)
  environment_vars: {
    'TEST_VAR': 'test_value',
    'NODE_ENV': 'test'
  },
  app_settings: {
    'BODHI_PORT': '8080',
    'BODHI_LOG_LEVEL': 'debug'
  },
  system_settings: {
    'BODHI_ENV_TYPE': 'development'
  },
  oauth_client_id: 'your_client_id',
  oauth_client_secret: 'your_client_secret',
  app_status: 'Ready'
};
```

## Running Tests

### Prerequisites

1. Build the NAPI module:
   ```bash
   cd crates/lib_bodhiserver_napi
   npm run build
   ```

2. Install test client dependencies:
   ```bash
   cd test-client
   npm install
   ```

### Run Tests

1. **Enhanced Configuration Test**:
   ```bash
   npm test
   ```

2. **Ping Endpoint Test**:
   ```bash
   npm run test:ping
   ```

## Test Results

Successful tests will show:
- ✅ Configuration validation
- ✅ App initialization with enhanced config
- ✅ Status checks (Ready = 1, Shutdown = 3)
- ✅ Proper cleanup and shutdown

## Implementation Notes

- The enhanced configuration is processed by the Rust backend using the new `AppOptionsBuilder` methods
- OAuth credentials and app status are applied during service initialization
- Environment variables and settings are properly configured in the service layer
- All tests use temporary directories to avoid conflicts

## Error Handling

The tests include proper error handling for:
- Invalid configuration values
- Service initialization failures
- Network connectivity issues (for ping test)
- Cleanup failures

## Future Enhancements

- Add TypeScript type definitions
- Add more comprehensive integration tests
- Add performance benchmarks
- Add error scenario testing
