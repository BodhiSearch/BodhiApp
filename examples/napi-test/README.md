# BodhiApp NAPI-RS FFI Test

This is a test project demonstrating the NAPI-RS FFI bindings for BodhiApp server functionality.

## Overview

This test project validates the FFI integration by:

1. **Creating a BodhiApp instance** using the NAPI-RS bindings
2. **Initializing the application** with development configuration
3. **Starting the HTTP server** on a random available port
4. **Making a request** to the `/api/ping` endpoint
5. **Shutting down the server** cleanly

## Prerequisites

- Node.js 18+ 
- Rust toolchain with NAPI-RS support
- Built BodhiApp NAPI bindings

## Building the NAPI Bindings

Before running this test, you need to build the NAPI bindings:

```bash
# From the project root
cd crates/lib_bodhiserver_napi
npm install
npm run build
```

## Running the Test

```bash
# Install dependencies
npm install

# Run the test
npm test

# Or run in development mode
npm run dev
```

## Expected Output

The test should produce output similar to:

```
🚀 Starting BodhiApp NAPI-RS FFI Test
📦 Creating BodhiApp instance...
✅ App created with status: 0
⚙️  Initializing BodhiApp...
✅ App initialized with status: 1
🌐 Starting HTTP server...
✅ Server started at: http://127.0.0.1:54321
📊 App status: 2
🏓 Testing /ping endpoint...
📡 Making request to: http://127.0.0.1:54321/api/ping
📥 Response status: 200
📄 Response body: pong
✅ /ping endpoint test successful!
🛑 Shutting down server...
✅ Server shutdown complete. Final status: 3
🎉 BodhiApp NAPI-RS FFI Test completed successfully!
```

## App States

The test tracks the application through these states:

- `0` - Uninitialized
- `1` - Ready (initialized but not running)
- `2` - Running (server active)
- `3` - Shutdown (server stopped)

## Architecture

This test demonstrates the FFI layer architecture:

```
TypeScript/Node.js
       ↓
   NAPI-RS Bindings (lib_bodhiserver_napi)
       ↓
   lib_bodhiserver (isolated interface)
       ↓
   BodhiApp Core Services
```

## Troubleshooting

### Build Issues

If you encounter build issues:

1. Ensure Rust toolchain is properly installed
2. Check that NAPI-RS CLI is available: `npm install -g @napi-rs/cli`
3. Verify the bindings build: `cd crates/lib_bodhiserver_napi && npm run build`

### Runtime Issues

If the test fails at runtime:

1. Check that all Rust dependencies compile: `cargo test`
2. Verify the NAPI bindings are properly linked
3. Ensure no other services are using the same ports

## Development

To modify the test:

1. Edit `src/index.ts` for test logic changes
2. Run `npm run dev` for quick iteration
3. Use `npm run build` to compile TypeScript to JavaScript

## Integration with CI/CD

This test can be integrated into CI/CD pipelines to validate FFI functionality:

```bash
# Build and test in CI
cargo test -p lib_bodhiserver_napi
cd crates/lib_bodhiserver_napi && npm run build
cd examples/napi-test && npm install && npm test
```
