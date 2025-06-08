# Build & Configuration

This document provides guidance for build systems, configuration management, and deployment patterns in the Bodhi App.

## Required Documentation References

**MUST READ for build configuration:**
- `ai-docs/01-architecture/system-overview.md` - System architecture and crate organization
- `ai-docs/01-architecture/tauri-desktop.md` - Desktop application build patterns

**FOR DEVELOPMENT SETUP:**
- `ai-docs/01-architecture/development-conventions.md` - Development environment standards

## Build System Overview

### Multi-Platform Architecture
The Bodhi App uses a multi-platform build system supporting:
- **Standalone Server** - HTTP server for deployment
- **Desktop Application** - Tauri-based native desktop app
- **Development Environment** - Hot-reload development setup

### Build Tools
- **Cargo** - Rust package manager and build system
- **Vite** - Frontend build tool and development server
- **Tauri** - Desktop application bundling
- **xtask** - Custom build automation

## Frontend Build Configuration

### Vite Configuration
```typescript
// crates/bodhi/vite.config.ts
import { defineConfig } from 'vite';
import react from '@vitejs/plugin-react';
import path from 'path';

export default defineConfig({
  plugins: [react()],
  resolve: {
    alias: {
      '@': path.resolve(__dirname, './src'),
    },
  },
  build: {
    outDir: 'dist',
    sourcemap: true,
    rollupOptions: {
      output: {
        manualChunks: {
          vendor: ['react', 'react-dom'],
          ui: ['@radix-ui/react-dialog', '@radix-ui/react-dropdown-menu'],
        },
      },
    },
  },
  server: {
    port: 1420,
    proxy: {
      '/bodhi': 'http://localhost:3000',
      '/v1': 'http://localhost:3000',
      '/app': 'http://localhost:3000',
    },
  },
});
```

### Package.json Scripts
```json
{
  "scripts": {
    "dev": "vite",
    "build": "tsc && vite build",
    "preview": "vite preview",
    "test": "vitest",
    "test:run": "vitest run",
    "lint": "eslint . --ext ts,tsx --report-unused-disable-directives --max-warnings 0",
    "format": "prettier --write \"src/**/*.{ts,tsx,js,jsx,json,css,md}\"",
    "tauri": "tauri",
    "tauri:dev": "tauri dev",
    "tauri:build": "tauri build"
  }
}
```

### TypeScript Configuration
```json
// crates/bodhi/tsconfig.json
{
  "compilerOptions": {
    "target": "ES2020",
    "useDefineForClassFields": true,
    "lib": ["ES2020", "DOM", "DOM.Iterable"],
    "module": "ESNext",
    "skipLibCheck": true,
    "moduleResolution": "bundler",
    "allowImportingTsExtensions": true,
    "resolveJsonModule": true,
    "isolatedModules": true,
    "noEmit": true,
    "jsx": "react-jsx",
    "strict": true,
    "noUnusedLocals": true,
    "noUnusedParameters": true,
    "noFallthroughCasesInSwitch": true,
    "baseUrl": ".",
    "paths": {
      "@/*": ["./src/*"]
    }
  },
  "include": ["src"],
  "references": [{ "path": "./tsconfig.node.json" }]
}
```

## Backend Build Configuration

### Cargo Workspace
```toml
# Cargo.toml (workspace root)
[workspace]
members = [
    "crates/objs",
    "crates/services",
    "crates/server_core",
    "crates/auth_middleware",
    "crates/routes_oai",
    "crates/routes_app",
    "crates/routes_all",
    "crates/server_app",
    "crates/commands",
    "crates/llama_server_proc",
    "crates/errmeta_derive",
    "crates/integration-tests",
    "crates/xtask",
]

[workspace.dependencies]
tokio = { version = "1.0", features = ["full"] }
serde = { version = "1.0", features = ["derive"] }
sqlx = { version = "0.7", features = ["runtime-tokio-rustls", "sqlite", "chrono", "uuid"] }
axum = "0.7"
anyhow = "1.0"
thiserror = "1.0"
tracing = "0.1"
uuid = { version = "1.0", features = ["v4", "serde"] }
chrono = { version = "0.4", features = ["serde"] }

[profile.release]
opt-level = 3
lto = true
codegen-units = 1
panic = "abort"
```

### Individual Crate Configuration
```toml
# crates/services/Cargo.toml
[package]
name = "services"
version = "0.1.0"
edition = "2021"

[dependencies]
objs = { path = "../objs" }
tokio = { workspace = true }
sqlx = { workspace = true }
serde = { workspace = true }
anyhow = { workspace = true }
thiserror = { workspace = true }
tracing = { workspace = true }
uuid = { workspace = true }
chrono = { workspace = true }

[dev-dependencies]
rstest = "0.18"
mockall = "0.11"

[features]
default = []
test-utils = ["mockall"]
```

## Tauri Build Configuration

### Tauri Configuration
```json
// crates/bodhi/src-tauri/tauri.conf.json
{
  "build": {
    "beforeDevCommand": "npm run dev",
    "beforeBuildCommand": "npm run build",
    "devPath": "http://localhost:1420",
    "distDir": "../dist",
    "withGlobalTauri": false
  },
  "package": {
    "productName": "Bodhi",
    "version": "0.1.0"
  },
  "tauri": {
    "allowlist": {
      "all": false,
      "shell": {
        "all": false,
        "open": true
      },
      "dialog": {
        "all": false,
        "open": true,
        "save": true
      },
      "fs": {
        "all": false,
        "readFile": true,
        "writeFile": true,
        "readDir": true,
        "createDir": true,
        "scope": ["$APPCONFIG/**", "$APPDATA/**"]
      }
    },
    "bundle": {
      "active": true,
      "targets": "all",
      "identifier": "app.getbodhi.desktop",
      "icon": [
        "icons/32x32.png",
        "icons/128x128.png",
        "icons/128x128@2x.png",
        "icons/icon.icns",
        "icons/icon.ico"
      ],
      "category": "DeveloperTool",
      "copyright": "",
      "deb": {
        "depends": []
      },
      "macOS": {
        "frameworks": [],
        "minimumSystemVersion": "",
        "exceptionDomain": ""
      },
      "windows": {
        "certificateThumbprint": null,
        "digestAlgorithm": "sha256",
        "timestampUrl": ""
      }
    },
    "security": {
      "csp": "default-src 'self'; script-src 'self' 'unsafe-inline'; style-src 'self' 'unsafe-inline'"
    },
    "updater": {
      "active": false
    },
    "windows": [
      {
        "fullscreen": false,
        "resizable": true,
        "title": "Bodhi",
        "width": 1200,
        "height": 800,
        "minWidth": 800,
        "minHeight": 600
      }
    ]
  }
}
```

### Tauri Cargo Configuration
```toml
# crates/bodhi/src-tauri/Cargo.toml
[package]
name = "bodhi-tauri"
version = "0.1.0"
edition = "2021"

[build-dependencies]
tauri-build = { version = "1.0", features = [] }

[dependencies]
tauri = { version = "1.0", features = ["api-all"] }
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
server_app = { path = "../../server_app" }
tokio = { workspace = true }

[features]
default = ["custom-protocol"]
custom-protocol = ["tauri/custom-protocol"]
```

## Development Build Commands

### Frontend Development
```bash
cd crates/bodhi

# Start development server
npm run dev

# Build for production
npm run build

# Preview production build
npm run preview

# Run tests
npm run test

# Format code
npm run format

# Lint code
npm run lint
```

### Backend Development
```bash
# Build all crates
cargo build

# Build specific crate
cargo build -p services

# Build for release
cargo build --release

# Run tests
cargo test

# Run specific crate tests
cargo test -p services

# Check code without building
cargo check
```

### Tauri Development
```bash
cd crates/bodhi

# Start Tauri development
npm run tauri:dev

# Build Tauri application
npm run tauri:build

# Build for specific target
npm run tauri build -- --target x86_64-pc-windows-msvc
```

## Custom Build Tasks (xtask)

### xtask Configuration
```toml
# crates/xtask/Cargo.toml
[package]
name = "xtask"
version = "0.1.0"
edition = "2021"

[dependencies]
anyhow = "1.0"
clap = { version = "4.0", features = ["derive"] }
```

### Custom Build Commands
```rust
// crates/xtask/src/main.rs
use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "xtask")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Generate OpenAPI documentation
    GenDocs,
    /// Build all components
    BuildAll,
    /// Run all tests
    TestAll,
    /// Format all code
    FormatAll,
}

fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::GenDocs => generate_docs(),
        Commands::BuildAll => build_all(),
        Commands::TestAll => test_all(),
        Commands::FormatAll => format_all(),
    }
}

fn build_all() -> anyhow::Result<()> {
    // Build backend
    std::process::Command::new("cargo")
        .args(["build", "--release"])
        .status()?;

    // Build frontend
    std::process::Command::new("npm")
        .args(["run", "build"])
        .current_dir("crates/bodhi")
        .status()?;

    println!("All components built successfully");
    Ok(())
}
```

### Running Custom Tasks
```bash
# Generate documentation
cargo xtask gen-docs

# Build all components
cargo xtask build-all

# Run all tests
cargo xtask test-all

# Format all code
cargo xtask format-all
```

## Environment Configuration

### Development Environment
```bash
# .env.development
DATABASE_URL=sqlite:./dev.db
LOG_LEVEL=debug
SERVER_PORT=3000
FRONTEND_URL=http://localhost:1420
```

### Production Environment
```bash
# .env.production
DATABASE_URL=sqlite:./bodhi.db
LOG_LEVEL=info
SERVER_PORT=3000
```

### Environment Loading
```rust
// Configuration loading
use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct Config {
    pub database_url: String,
    pub server_port: u16,
    pub log_level: String,
}

impl Config {
    pub fn from_env() -> anyhow::Result<Self> {
        dotenvy::dotenv().ok();
        envy::from_env().map_err(Into::into)
    }
}
```

## Continuous Integration

### GitHub Actions Workflow
```yaml
# .github/workflows/ci.yml
name: CI

on:
  push:
    branches: [main]
  pull_request:
    branches: [main]

jobs:
  test:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v3
      
      - name: Setup Rust
        uses: actions-rs/toolchain@v1
        with:
          toolchain: stable
          
      - name: Setup Node.js
        uses: actions/setup-node@v3
        with:
          node-version: '18'
          
      - name: Cache dependencies
        uses: actions/cache@v3
        with:
          path: |
            ~/.cargo/registry
            ~/.cargo/git
            target/
            crates/bodhi/node_modules
          key: ${{ runner.os }}-cargo-${{ hashFiles('**/Cargo.lock') }}-node-${{ hashFiles('**/package-lock.json') }}
          
      - name: Install frontend dependencies
        run: cd crates/bodhi && npm ci
        
      - name: Run backend tests
        run: cargo test --all-crates
        
      - name: Run frontend tests
        run: cd crates/bodhi && npm run test
        
      - name: Build all
        run: cargo xtask build-all

  build-tauri:
    runs-on: ${{ matrix.platform }}
    strategy:
      matrix:
        platform: [ubuntu-latest, windows-latest, macos-latest]
    steps:
      - uses: actions/checkout@v3
      
      - name: Setup Rust
        uses: actions-rs/toolchain@v1
        with:
          toolchain: stable
          
      - name: Setup Node.js
        uses: actions/setup-node@v3
        with:
          node-version: '18'
          
      - name: Install dependencies (Ubuntu)
        if: matrix.platform == 'ubuntu-latest'
        run: |
          sudo apt-get update
          sudo apt-get install -y libgtk-3-dev libwebkit2gtk-4.0-dev libappindicator3-dev librsvg2-dev patchelf
          
      - name: Install frontend dependencies
        run: cd crates/bodhi && npm ci
        
      - name: Build Tauri app
        run: cd crates/bodhi && npm run tauri:build
```

## Performance Optimization

### Build Optimization
```toml
# Cargo.toml - Release profile optimization
[profile.release]
opt-level = 3
lto = true
codegen-units = 1
panic = "abort"
strip = true
```

### Frontend Bundle Optimization
```typescript
// vite.config.ts - Bundle optimization
export default defineConfig({
  build: {
    rollupOptions: {
      output: {
        manualChunks: {
          vendor: ['react', 'react-dom'],
          ui: ['@radix-ui/react-dialog', '@radix-ui/react-dropdown-menu'],
          query: ['@tanstack/react-query'],
        },
      },
    },
    chunkSizeWarningLimit: 1000,
  },
});
```

## Related Documentation

- **[System Overview](system-overview.md)** - System architecture and crate organization
- **[Tauri Desktop](tauri-desktop.md)** - Desktop application build patterns
- **[Development Conventions](development-conventions.md)** - Development environment standards
- **[Testing Strategy](testing-strategy.md)** - Testing and CI/CD patterns

---

*For platform-specific build instructions and deployment guides, see the respective technology documentation. For desktop application building, see [Tauri Desktop](tauri-desktop.md).*
