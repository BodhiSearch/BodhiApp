# CLAUDE.md - xtask

See [PACKAGE.md](./PACKAGE.md) for implementation details and file references.

This file provides architectural guidance for the `xtask` crate, which orchestrates build automation and code generation workflows that bridge Rust API definitions to TypeScript consumers across the BodhiApp ecosystem.

## Architectural Purpose

The `xtask` crate serves as the **code generation orchestrator** in BodhiApp's build system, implementing a critical cross-language type safety bridge. Unlike traditional build tools that focus on compilation, xtask specializes in **contract propagation** - ensuring that API changes in Rust automatically flow to all TypeScript consumers without manual intervention.

### Core Responsibilities
- **Contract Generation**: Transform Rust API documentation into OpenAPI 3.1 specifications
- **Multi-Target Type Distribution**: Generate TypeScript types for distinct consumption patterns
- **Toolchain Abstraction**: Provide consistent code generation regardless of environment
- **Development Workflow Integration**: Seamlessly integrate with both manual and CI/CD workflows

## Architectural Components

### Command Orchestration Architecture
The main dispatcher implements a **command pattern** with fallback delegation, enabling extensible task routing while maintaining compatibility with standard Rust tooling patterns. This design allows xtask to specialize in code generation while delegating conventional build tasks to xtaskops.

### Contract Extraction Pipeline
The OpenAPI generation component implements a **single-pass extraction** strategy, reading `BodhiOpenAPIDoc` from the routes_app crate and generating a complete API specification. This approach ensures that all API contracts remain synchronized with the actual Rust implementation, preventing documentation drift.

**Key Architectural Decision**: Using utoipa annotations directly in route handlers creates a **compile-time guarantee** that API documentation matches implementation. When handlers change, the OpenAPI spec automatically reflects those changes.

### Multi-Target Type Generation Strategy
The TypeScript generation component implements a **fan-out pattern** for type distribution:

1. **Primary Target**: Main application (`app/types/api.d.ts`) for React frontend consumption
2. **Secondary Target**: Client library (`ts-client/src/types/api.d.ts`) for external SDK usage
3. **Environment Adaptation**: Automatic toolchain installation ensures consistent generation across development environments

**Architectural Constraint**: The dual-target approach addresses the different consumption patterns - the main app needs component-level types for UI interactions, while the client library requires full SDK-compatible type definitions.

## Cross-Crate Coordination Patterns

### Build-Time Orchestration Layer
xtask operates as a **build-time orchestrator** that coordinates between multiple architectural layers:

- **API Layer Integration**: Directly consumes `routes_app::BodhiOpenAPIDoc` to extract complete API surface
- **Frontend Integration**: Generates types consumed by React components for type-safe API interactions
- **Client Library Coordination**: Provides fallback type generation for ts-client while coordinating with its advanced build process
- **Development Workflow**: Integrates with both manual development and automated CI/CD pipelines

### Dependency Isolation Strategy
The crate maintains **minimal coupling** with runtime components while maximizing build-time integration:
- Only depends on API documentation (routes_app) and utility libraries (anyhow, utoipa)
- External toolchain dependencies (npm, openapi-typescript) are managed automatically
- No runtime dependencies ensure clean separation between build and deployment artifacts

### Contract Propagation Architecture
xtask implements a **unidirectional flow** from Rust to TypeScript:
```
Rust API Definitions (utoipa) → OpenAPI 3.1 Spec → TypeScript Types → Frontend/Client
```

This architecture ensures that API changes propagate automatically without manual synchronization, preventing type mismatches that could cause runtime failures.

## Domain-Specific Integration Patterns

### API-First Development Workflow
xtask enables an **API-first development pattern** where Rust API definitions drive all downstream type safety:

**Frontend Integration Strategy**: The generated `app/types/api.d.ts` provides complete type coverage for React components, enabling compile-time verification of API interactions. This prevents common runtime errors from API contract mismatches.

**Client Library Coordination**: The dual-generation approach for ts-client addresses **evolutionary compatibility** - legacy `api.d.ts` provides immediate compatibility while the client library transitions to modern `@hey-api/openapi-ts` tooling.

### Build System Extension Architecture
xtask implements a **capability extension pattern** over xtaskops:
- Handles specialized code generation tasks that require domain knowledge
- Delegates standard Rust operations (testing, formatting, linting) to xtaskops
- Maintains clean separation between generic build operations and project-specific generation

### Cross-Language Type Safety Bridge
The crate implements a **type safety bridge** that ensures API changes are immediately visible across language boundaries:

**Rust → TypeScript Flow**: Changes to route handlers with utoipa annotations automatically propagate to TypeScript consumers, preventing the common problem of API documentation becoming stale.

**Multi-Consumer Support**: Different TypeScript consumers (frontend vs. client library) receive appropriately formatted types for their specific usage patterns.

## Extension and Modification Patterns

### Safe Task Extension Strategy
When adding new code generation tasks:

**Module Isolation**: Each task should be implemented as a separate module with clear boundaries and minimal interdependencies. This enables independent testing and maintenance.

**Error Context Propagation**: Use `anyhow::Result` with meaningful context messages to provide clear error diagnostics during build failures. Context should identify both the task and the specific operation that failed.

**Command Integration**: Add new commands through the main dispatcher while maintaining fallback to xtaskops for unhandled operations.

### API Evolution Workflow
The crate supports **seamless API evolution** through automated propagation:

**Source-Driven Updates**: Changes to `routes_app` utoipa annotations automatically flow through the entire toolchain without manual intervention.

**Multi-Target Coordination**: The type generation system ensures that both frontend and client library receive updated types, preventing version skew between consumers.

**Build Integration**: The workflow integrates with both development (manual) and CI/CD (automated) build processes.

### Toolchain Management Philosophy
xtask implements **zero-configuration toolchain management**:

**Automatic Dependency Resolution**: Missing external tools are automatically installed, ensuring consistent build environments across different developer machines.

**Graceful Degradation**: When external tools cannot be installed, the system provides clear error messages with actionable remediation steps.

**Environment Adaptation**: The system detects and adapts to different project structures (presence/absence of ts-client) without requiring configuration changes.

## Technical Constraints and Design Decisions

### Error Handling Strategy
The crate implements **contextual error propagation** using anyhow to provide actionable error diagnostics during build failures. This approach enables developers to quickly identify and resolve issues in the code generation pipeline.

### External Tool Integration Constraints
The reliance on npm and Node.js toolchain creates environmental dependencies that must be managed:

**Automatic Installation Strategy**: Tools are installed globally via npm when missing, ensuring consistent build environments but requiring npm access.

**Version Compatibility**: The system uses stable tool versions but must adapt to evolving TypeScript generation toolchains (openapi-typescript vs. @hey-api/openapi-ts).

### Multi-Target Generation Complexity
Supporting both main application and client library targets creates coordination challenges:

**Conditional Generation Logic**: The system detects target environments and generates appropriate types, but this requires maintaining multiple generation strategies.

**Toolchain Evolution**: As ts-client transitions to modern tooling, xtask provides compatibility bridges while avoiding breaking changes.

## Code Generation Pipeline Architecture

### Contract-First Generation Strategy
The crate implements a **contract-first approach** where OpenAPI specification generation precedes all TypeScript generation, ensuring consistency across all consumers.

**Single Source Extraction**: API contracts are extracted once from `routes_app::BodhiOpenAPIDoc`, creating a canonical representation that serves all downstream consumers.

**Format Standardization**: OpenAPI 3.1 format provides a standardized intermediate representation that can be consumed by various TypeScript generation tools.

### Multi-Stage Type Distribution
The TypeScript generation implements a **multi-stage distribution pattern**:

**Stage 1**: Toolchain validation and automatic installation ensures consistent generation environment
**Stage 2**: Primary target generation for main application with component-focused type exports
**Stage 3**: Secondary target detection and generation for client library with SDK-compatible types
**Stage 4**: Directory structure preparation and file output coordination

### Evolutionary Compatibility Management
The system manages **toolchain evolution** through coordinated generation strategies:

**Legacy Support**: Maintains openapi-typescript generation for immediate compatibility
**Modern Integration**: Coordinates with @hey-api/openapi-ts for advanced type generation features
**Transition Management**: Provides dual outputs during toolchain migration periods

### Build Output Coordination
The crate produces **strategically positioned outputs** for different consumption patterns:
- Project-root OpenAPI specification for tool integration
- Application-specific types for React frontend consumption
- Client library types for external SDK usage
- Coordination with independent build processes for advanced type generation

This output strategy ensures that each consumer receives appropriately formatted types while maintaining consistency across the entire ecosystem.

## Development Integration Points

### Makefile Integration
The crate integrates with the project's Makefile system through `make ts-client` command, which orchestrates complete TypeScript client building with tests. This provides a higher-level interface for complex build workflows.

### CI/CD Pipeline Integration
xtask is designed for both manual development use and automated CI/CD execution. The automated toolchain installation ensures consistent behavior across different execution environments without requiring manual setup.

### Developer Experience Optimization
The zero-configuration approach reduces cognitive overhead for developers - they can focus on API implementation in Rust without needing to understand TypeScript toolchain complexities. Type generation happens automatically as part of the build process.