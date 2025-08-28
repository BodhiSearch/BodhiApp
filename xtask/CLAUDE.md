# CLAUDE.md - xtask

This file provides guidance to Claude Code when working with the `xtask` crate, which provides build automation and code generation tools for the BodhiApp project.

## Purpose

The `xtask` crate is a Rust executable that provides build automation and code generation tasks for the BodhiApp project. It follows the xtask pattern for project-specific tooling and handles:

- OpenAPI 3.1 specification generation from Rust API documentation
- TypeScript type generation from OpenAPI specs for multiple targets
- Integration with the broader xtaskops build system for standard operations
- Automated toolchain management for TypeScript code generation

## Key Components

### Main Entry Point (`src/main.rs`)
- Command-line dispatcher that routes to specific tasks
- Supports `openapi` and `types` subcommands
- Falls back to `xtaskops::tasks::main()` for standard tasks

### OpenAPI Generation (`src/openapi.rs`)
- Generates OpenAPI 3.1 specification from `routes_app::BodhiOpenAPIDoc`
- Outputs to `openapi.json` in the project root using pretty JSON formatting
- Uses `utoipa` crate for OpenAPI document generation from Rust API documentation
- Serves as the source of truth for API contracts across frontend and client libraries

### TypeScript Generation (`src/typescript.rs`)
- Converts OpenAPI spec to TypeScript type definitions using `openapi-typescript`
- Automatically installs `openapi-typescript` globally if not present via npm
- Generates types for main app (`app/types/api.d.ts`) with `--export-type components` flag
- Conditionally generates types for ts-client (`ts-client/src/types/api.d.ts`) when directory exists
- Ensures output directories exist before generation and handles dual-target scenarios
- Integrates with ts-client's own build process which uses `@hey-api/openapi-ts` for more advanced type generation

## Dependencies

### Direct Dependencies
- `anyhow` (workspace) - Error handling and context propagation
- `xtaskops` (0.4.2) - Standard build task operations and project automation
- `routes_app` (workspace) - API route definitions and OpenAPI documentation source
- `utoipa` (workspace) - OpenAPI spec generation from Rust code annotations

### External Tools
- `npm` - Node.js package manager for installing and managing TypeScript toolchain
- `openapi-typescript` - CLI tool for generating TypeScript types from OpenAPI specs (legacy support)
- `@hey-api/openapi-ts` - Modern TypeScript generator used by ts-client for advanced type generation

## Architecture Position

The `xtask` crate sits at the build/development layer of the project:
- **Independent**: Has minimal dependencies, only uses workspace utilities and external toolchain
- **Code Generation Bridge**: Bridges Rust API definitions to TypeScript frontend types across multiple targets
- **Development Tool**: Not included in runtime deployments, only used during development and CI/CD
- **Toolchain Manager**: Automatically manages external dependencies for consistent build environments

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
- Generates TypeScript types consumed by React components via `app/types/api.d.ts`
- Types include complete API surface with paths, operations, and component schemas
- Frontend imports these types for type-safe API interactions

### With TypeScript Client (`ts-client/`)
- Automatically detects ts-client directory and generates compatible types
- Legacy support generates `ts-client/src/types/api.d.ts` using `openapi-typescript`
- ts-client has its own build process using `@hey-api/openapi-ts` for `types.gen.ts`
- Dual generation ensures compatibility during ts-client build transitions

### With API Routes (`routes_app`)
- Reads `BodhiOpenAPIDoc` to extract complete API definitions with utoipa annotations
- Converts Rust API documentation to OpenAPI 3.1 format with full schema information
- Maintains single source of truth for API contracts in Rust code

### With Build System (`xtaskops`)
- Extends standard build operations with project-specific code generation tasks
- Falls back to xtaskops for unhandled commands like standard cargo operations
- Integrates with CI/CD pipelines for automated type generation

## Development Guidelines

### Adding New Tasks
1. Create new module in `src/` for the task logic
2. Add command dispatcher case in `main.rs`
3. Follow existing error handling patterns with `anyhow::Result`

### OpenAPI Changes
- Modify API definitions in `routes_app` crate using utoipa annotations
- Run `cargo run --package xtask openapi` to regenerate OpenAPI 3.1 spec
- Run `cargo run --package xtask types` to update TypeScript types for all targets
- Changes automatically propagate to both frontend and ts-client builds

### Type Generation Workflow
- Ensure output directories exist before writing files using `std::fs::create_dir_all`
- Handle dual-target scenarios (main app + ts-client) with conditional generation
- Install required tools automatically when missing via npm global installation
- Support both legacy `openapi-typescript` and modern `@hey-api/openapi-ts` workflows
- Coordinate with ts-client's independent build process for seamless integration

## Error Handling

All functions return `anyhow::Result<()>` for consistent error handling:
- File operations provide context with `.context()`
- External command failures are captured and reported
- Missing tools are automatically installed when possible

## Build Automation and Code Generation Workflows

### OpenAPI Generation Workflow
1. Extract API definitions from `routes_app::BodhiOpenAPIDoc` using utoipa
2. Generate pretty-formatted JSON to `openapi.json` in project root
3. Serve as single source of truth for API contracts across all consumers

### TypeScript Type Generation Workflow
1. Ensure `openapi.json` exists by running OpenAPI generation first
2. Check for `openapi-typescript` global installation via `npm ls -g`
3. Auto-install `openapi-typescript` globally if missing
4. Generate types for main app at `app/types/api.d.ts` with component exports
5. Conditionally detect `ts-client/` directory and generate types there
6. Create necessary directory structure before file generation
7. Support dual toolchain approach (legacy + modern) for ts-client compatibility

### Integration with ts-client Build Process
- ts-client uses `@hey-api/openapi-ts` for advanced type generation to `types.gen.ts`
- xtask provides fallback generation to `api.d.ts` for compatibility
- ts-client build process: `generate:openapi` → `generate:types` → `bundle`
- Coordinated workflow ensures type consistency across all consumers

## File Outputs

- `openapi.json` - OpenAPI 3.1 specification with complete API surface (project root)
- `app/types/api.d.ts` - TypeScript types for main React application
- `ts-client/src/types/api.d.ts` - Legacy TypeScript types for client library (conditional)
- `ts-client/src/types/types.gen.ts` - Modern TypeScript types via ts-client's own build process