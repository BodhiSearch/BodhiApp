# CLAUDE.md - xtask

This file provides guidance to Claude Code when working with the `xtask` crate, which provides build automation and code generation tools for the BodhiApp project.

## Purpose

The `xtask` crate is a Rust executable that provides build automation and code generation tasks for the BodhiApp project. It follows the xtask pattern for project-specific tooling and handles:

- OpenAPI specification generation from Rust code
- TypeScript type generation from OpenAPI specs
- Integration with the broader xtaskops build system

## Key Components

### Main Entry Point (`src/main.rs`)
- Command-line dispatcher that routes to specific tasks
- Supports `openapi` and `types` subcommands
- Falls back to `xtaskops::tasks::main()` for standard tasks

### OpenAPI Generation (`src/openapi.rs`)
- Generates OpenAPI 3.0 specification from `routes_app::BodhiOpenAPIDoc`
- Outputs to `openapi.json` in the project root
- Uses `utoipa` crate for OpenAPI document generation

### TypeScript Generation (`src/typescript.rs`)
- Converts OpenAPI spec to TypeScript type definitions
- Automatically installs `openapi-typescript` if not present
- Generates types for both main app (`app/types/api.d.ts`) and ts-client (`ts-client/src/types/api.d.ts`)
- Ensures output directories exist before generation

## Dependencies

### Direct Dependencies
- `anyhow` - Error handling and context
- `xtaskops` - Standard build task operations  
- `routes_app` - API route definitions and OpenAPI documentation
- `utoipa` - OpenAPI spec generation from Rust code

### External Tools
- `npm` - Node.js package manager for installing openapi-typescript
- `openapi-typescript` - Tool for generating TypeScript types from OpenAPI specs

## Architecture Position

The `xtask` crate sits at the build/development layer of the project:
- **Independent**: Has minimal dependencies, only uses workspace utilities
- **Code Generation**: Bridges Rust API definitions to TypeScript frontend types
- **Development Tool**: Not included in runtime deployments, only used during development

## Usage Patterns

### Generating OpenAPI Specification
```bash
cargo run --package xtask openapi
# Outputs: openapi.json
```

### Generating TypeScript Types  
```bash
cargo run --package xtask types
# Outputs: app/types/api.d.ts and ts-client/src/types/api.d.ts (if exists)
```

### Standard Build Tasks
```bash
cargo run --package xtask
# Delegates to xtaskops for standard operations
```

## Integration Points

### With Frontend (`crates/bodhi/src/`)
- Generates TypeScript types consumed by React components
- Types are output to `app/types/api.d.ts` for import in frontend code

### With TypeScript Client (`ts-client/`)
- Automatically detects and generates types for standalone TypeScript client
- Creates `ts-client/src/types/api.d.ts` when ts-client directory exists

### With API Routes (`routes_app`)
- Reads `BodhiOpenAPIDoc` to extract API definitions
- Converts Rust API documentation to OpenAPI 3.0 format

### With Build System (`xtaskops`)
- Extends standard build operations with project-specific tasks
- Falls back to xtaskops for unhandled commands

## Development Guidelines

### Adding New Tasks
1. Create new module in `src/` for the task logic
2. Add command dispatcher case in `main.rs`
3. Follow existing error handling patterns with `anyhow::Result`

### OpenAPI Changes
- Modify API definitions in `routes_app` crate
- Run `cargo run --package xtask openapi` to regenerate spec
- Run `cargo run --package xtask types` to update TypeScript types

### Type Generation
- Ensure output directories exist before writing files
- Handle both main app and ts-client scenarios
- Install required tools automatically when missing

## Error Handling

All functions return `anyhow::Result<()>` for consistent error handling:
- File operations provide context with `.context()`
- External command failures are captured and reported
- Missing tools are automatically installed when possible

## File Outputs

- `openapi.json` - OpenAPI 3.0 specification (project root)
- `app/types/api.d.ts` - TypeScript types for main application
- `ts-client/src/types/api.d.ts` - TypeScript types for client library (if ts-client exists)