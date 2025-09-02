# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Development Commands

### Testing
- `make test` - Run all tests (backend, UI, NAPI)
- `make test.backend` - Run Rust backend tests (`cargo test` and `cargo test -p bodhi --features native`)
- `make test.ui` - Run frontend tests (`cd crates/bodhi && npm install && npm test`)
- `make test.napi` - Run NAPI bindings tests (`cd crates/lib_bodhiserver_napi && npm install && npm run test`)
- `make ui.test` - Run UI tests (alias for frontend tests)

### Building & Packaging
- `make ci.build` - Build Tauri desktop application
- `make ts-client` - Build TypeScript client package with tests
- `cd crates/bodhi && npm run build` - Build Next.js frontend
- `cd crates/lib_bodhiserver_napi && npm run build:release` - Build NAPI bindings

### Code Quality
- `make format` - Format all code (Rust, Node.js, Python)
- `make format.all` - Format and run Clippy fixes
- `cargo fmt --all` - Format Rust code only
- `cd crates/bodhi && npm run format` - Format frontend code
- `cd crates/bodhi && npm run lint` - Lint frontend code
- `cd crates/lib_bodhiserver_napi && npm run format` - Format NAPI package

### Coverage & Analysis
- `make coverage` - Generate code coverage report (outputs to `lcov.info`)

### OpenAPI & Client Generation
- `cargo run --package xtask openapi` - Generate OpenAPI specification
- `cd ts-client && npm run generate` - Generate TypeScript client types

### Running the Application
- `cd crates/bodhi && npm run dev` - Start Next.js development server
- `cd crates/bodhi/src-tauri && cargo tauri dev` - Run Tauri desktop app in dev mode

## Architecture Overview

BodhiApp is a Rust-based application providing local Large Language Model (LLM) inference with a modern React web interface and Tauri desktop app.

### Technology Stack
- **Backend**: Rust with Axum HTTP framework
- **Frontend**: React + TypeScript + Next.js v14 + TailwindCSS + Shadcn UI  
- **Desktop**: Tauri for native desktop application
- **LLM Integration**: llama.cpp for local inference
- **Database**: SQLite with SQLx
- **Authentication**: OAuth2 + JWT
- **API**: OpenAI-compatible endpoints

### Key Crates Structure
The project uses a Cargo workspace with these main crates:

**Foundation Crates:**
- `objs` - Domain objects, types, errors, validation
- `services` - Business logic, external integrations
- `server_core` - HTTP server infrastructure
- `auth_middleware` - Authentication and authorization

**API Crates:**
- `routes_oai` - OpenAI-compatible API endpoints  
- `routes_app` - Application-specific API endpoints
- `routes_all` - Unified route composition

**Application Crates:**
- `server_app` - Standalone HTTP server
- `crates/bodhi/src-tauri` - Tauri desktop application
- `commands` - CLI interface

**Utility Crates:**
- `llama_server_proc` - LLM process management
- `lib_bodhiserver_napi` - Node.js bindings for server functionality
- `integration-tests` - End-to-end testing
- `xtask` - Build automation and code generation

### Frontend Structure
Located in `crates/bodhi/`, this is a Next.js 14 application using:
- React with TypeScript
- TailwindCSS + Shadcn UI components
- React Query for API state management
- Vitest for testing

### Key Features
- **Local LLM Inference**: llama.cpp integration with model management
- **OpenAI Compatibility**: Full OpenAI API compatibility for chat completions
- **Web Interface**: Modern React-based chat UI with streaming responses
- **Desktop Application**: Tauri-based native app with system integration
- **Authentication**: OAuth2 + JWT with role-based access control

### Development Patterns
- **Error Handling**: Centralized error types with localization support
- **Testing**: Unit tests per crate, integration tests, and frontend tests
- **Code Generation**: OpenAPI specs auto-generated from Rust code, TypeScript types from OpenAPI
- **Configuration**: Environment-based config with runtime updates

## Important Notes

- Run tests before making changes to ensure nothing is broken
- Use `make format.all` to format code and fix linting issues
- The project generates TypeScript types from Rust OpenAPI specs - regenerate after API changes
- Frontend uses strict TypeScript - ensure proper typing
- NAPI bindings require Node.js >=22
- Desktop app development requires Tauri CLI
- for getting the current time Utc::now we have TimeService in @crates/services/src/db/service.rs, for objects, do not use Utc::now internally to get time for created_at etc. instead have it passed via constructor
- write test that provides value for the maintainance we have to do, for e.g. do not write test to test the new constructor, or macro implemented PartialEq, or serialization/deserialization by serde, unless we are using customization like untagged, or changing case etc.