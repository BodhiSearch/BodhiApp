# @bodhiapp/app-bindings

[![npm version](https://badge.fury.io/js/@bodhiapp%2Fapp-bindings.svg)](https://badge.fury.io/js/@bodhiapp%2Fapp-bindings)
[![CI](https://github.com/BodhiSearch/BodhiApp/actions/workflows/build.yml/badge.svg)](https://github.com/BodhiSearch/BodhiApp/actions/workflows/build.yml)

Node.js bindings for BodhiApp server functionality, built with NAPI-RS for high-performance native integration.

## Features

- 🚀 **High Performance**: Native Rust implementation with Node.js bindings
- 🔄 **Cross-Platform**: Supports macOS, Linux, and Windows
- 📦 **Zero Dependencies**: Self-contained with platform-specific binaries
- 🔒 **Type Safe**: Full TypeScript definitions included
- 🧪 **Well Tested**: Comprehensive test suite with integration tests

## Installation

```bash
npm install @bodhiapp/app-bindings
```

## Supported Platforms

This package includes pre-built binaries for the following platforms:

| Platform | Architecture | Node.js |
|----------|-------------|---------|
| macOS | Apple Silicon (ARM64) | >=22 |
| Linux | x64 | >=22 |
| Windows | x64 | >=22 |

## Usage

### Basic Usage

```javascript
import { /* exported functions */ } from '@bodhiapp/app-bindings';

// Example usage will be added based on your actual exports
```

### TypeScript Support

The package includes full TypeScript definitions:

```typescript
import type { /* types */ } from '@bodhiapp/app-bindings';

// TypeScript intellisense and type checking work out of the box
```

## API Reference

<!-- API documentation will be generated based on your actual exports -->

## Development

This package is part of the [BodhiApp](https://github.com/BodhiSearch/BodhiApp) project. 

### Building from Source

If you need to build from source:

```bash
# Clone the repository
git clone https://github.com/BodhiSearch/BodhiApp.git
cd BodhiApp/crates/lib_bodhiserver_napi

# Install dependencies
npm install

# Build the native module
npm run build:release
```

### Testing

```bash
# Run unit tests
npm test

# Run integration tests
npm run test:integration

# Run Playwright tests
npm run test:playwright
```

## Architecture

This package uses [NAPI-RS](https://napi.rs/) to provide Node.js bindings for Rust code. The architecture follows these principles:

- **Platform-specific binaries**: Each supported platform gets its own optimized binary
- **Automatic loading**: The package automatically loads the correct binary for your platform
- **Fallback handling**: Graceful error handling if binaries are missing
- **Development support**: Debug builds available for development

## Performance

The native Rust implementation provides significant performance benefits over pure JavaScript implementations:

- **Memory efficiency**: Lower memory usage through Rust's ownership model
- **CPU performance**: Optimized native code execution
- **Concurrent processing**: Built-in support for async operations

## Contributing

Contributions are welcome! Please see the [main repository](https://github.com/BodhiSearch/BodhiApp) for contribution guidelines.

### Development Setup

1. **Rust**: Install Rust toolchain (1.70+)
2. **Node.js**: Install Node.js (22+)
3. **Platform tools**: Platform-specific build tools may be required

## License

MIT License - see the [LICENSE](https://github.com/BodhiSearch/BodhiApp/blob/main/LICENSE) file for details.

## Support

- **Issues**: [GitHub Issues](https://github.com/BodhiSearch/BodhiApp/issues)
- **Documentation**: [Project Documentation](https://github.com/BodhiSearch/BodhiApp#readme)
- **Community**: [Discussions](https://github.com/BodhiSearch/BodhiApp/discussions)

## Related Packages

This package is part of the BodhiApp ecosystem:

- **[BodhiApp](https://github.com/BodhiSearch/BodhiApp)**: Main application repository
- **Desktop App**: Cross-platform desktop application built with Tauri

---

*Built with ❤️ using [NAPI-RS](https://napi.rs/)* 