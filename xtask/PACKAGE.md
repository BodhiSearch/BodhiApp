# PACKAGE.md - xtask

See [CLAUDE.md](./CLAUDE.md) for architectural guidance and implementation context.

## Implementation Index

### Core Entry Point
- **Main Module**: `xtask/src/main.rs:1-13`
  ```rust
  fn main() -> Result<()> {
    let args: Vec<String> = std::env::args().collect();
    match args.get(1).map(|s| s.as_str()) {
      Some("openapi") => openapi::generate(),
      Some("types") => typescript::generate_types(),
      _ => xtaskops::tasks::main(),
    }
  }
  ```

### Code Generation Modules

#### OpenAPI Generation
- **OpenAPI Module**: `xtask/src/openapi.rs:1-12`
  - Main generation function: `xtask/src/openapi.rs:6-11`
  ```rust
  pub fn generate() -> Result<()> {
    let openai = BodhiOpenAPIDoc::openapi();
    let mut file = File::create("openapi.json")?;
    file.write_all(openai.to_pretty_json()?.as_bytes())?;
    println!("OpenAPI spec written to openapi.json");
    Ok(())
  }
  ```

#### TypeScript Type Generation
- **TypeScript Module**: `xtask/src/typescript.rs:1-65`
  - Main generation function: `xtask/src/typescript.rs:4-64`
  - Tool installation check: `xtask/src/typescript.rs:9-21`
  - Main app type generation: `xtask/src/typescript.rs:24-37`
  - ts-client detection and generation: `xtask/src/typescript.rs:40-62`
  ```rust
  // Auto-install openapi-typescript if missing
  if !String::from_utf8_lossy(&npm_ls.stdout).contains("openapi-typescript") {
    println!("Installing openapi-typescript...");
    Command::new("npm")
      .args(["install", "-g", "openapi-typescript"])
      .status()
      .context("Failed to install openapi-typescript")?;
  }
  ```

### Dependencies & Configuration
- **Cargo Configuration**: `xtask/Cargo.toml:1-11`
  - Workspace dependencies: `anyhow`, `xtaskops`, `routes_app`, `utoipa`
  - External toolchain: npm, openapi-typescript CLI

### Generated Outputs
- **OpenAPI Specification**: `openapi.json` (project root)
- **Main App Types**: `app/types/api.d.ts`
- **Client Library Types**: `ts-client/src/types/api.d.ts` (conditional)

## Usage Commands

### Generate OpenAPI Specification
```bash
cargo run --package xtask openapi
# Generates: openapi.json in project root
```

### Generate TypeScript Types
```bash
cargo run --package xtask types
# Generates: app/types/api.d.ts and ts-client/src/types/api.d.ts (if exists)
```

### Standard Build Tasks
```bash
cargo run --package xtask
# Delegates to xtaskops for standard operations
```

### Development Workflow Integration
```bash
# Complete type generation workflow
cargo run --package xtask openapi  # Generate OpenAPI spec
cargo run --package xtask types    # Generate TypeScript types

# Or combined via Makefile
make ts-client                     # Build TypeScript client with tests
```

## Build Dependencies

### Runtime Requirements
- **Rust Toolchain**: Standard cargo and rustc
- **Node.js**: For npm package management and openapi-typescript
- **npm**: Global package manager for TypeScript toolchain

### External Tools (Auto-installed)
- **openapi-typescript**: CLI for TypeScript type generation
- **@hey-api/openapi-ts**: Modern TypeScript generator (used by ts-client)

### Workspace Integration
- **xtaskops@0.4.2**: Standard build task operations
- **routes_app**: Source of OpenAPI documentation via BodhiOpenAPIDoc
- **utoipa**: OpenAPI specification generation from Rust annotations

## File Structure
```
xtask/
├── Cargo.toml                 # Crate configuration and dependencies
├── CLAUDE.md                  # Architectural documentation
├── PACKAGE.md                 # This implementation guide
└── src/
    ├── main.rs               # Command dispatcher and entry point
    ├── openapi.rs            # OpenAPI 3.1 specification generation
    └── typescript.rs         # TypeScript type generation and toolchain management
```

## Integration Points

### Frontend Integration (`crates/bodhi/src/`)
- Consumes generated types from `app/types/api.d.ts`
- Types provide complete API surface for React components
- Enables type-safe API interactions throughout the application

### Client Library Integration (`ts-client/`)
- Automatically detects ts-client directory presence
- Generates compatible types to `ts-client/src/types/api.d.ts`
- Coordinates with ts-client's own build process using @hey-api/openapi-ts
- Ensures type consistency during build transitions

### API Routes Integration (`routes_app`)
- Extracts API definitions from `routes_app::BodhiOpenAPIDoc`
- Single source of truth maintained in Rust code with utoipa annotations
- Automatic propagation of API changes to all TypeScript consumers

### Build System Integration (`xtaskops`)
- Extends standard Rust project tasks with code generation
- Falls back to xtaskops for unhandled command operations
- Integrates with CI/CD for automated type generation workflows