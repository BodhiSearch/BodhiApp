# Claude & Package Documentation Generation Guidelines

This document provides guidelines for AI coding assistants when generating CLAUDE.md and PACKAGE.md files for BodhiApp crates.

## Target Audience

These files are consumed by **expert AI coding assistants** who:
- Have deep knowledge of common frameworks and libraries (serde, clap, axum, thiserror, etc.)
- Understand standard Rust patterns and HTTP error handling
- Need only **project-specific domain knowledge** to be productive

## Documentation Structure

### CLAUDE.md
- **Purpose**: Comprehensive architectural guidance for expert AI assistants
- **Scope**: Detailed domain modeling and architectural depth (typically 100-300 lines)
- **Content Focus**:
  - Detailed domain architecture with sophisticated subsection modeling
  - Cross-crate coordination and service integration patterns
  - Technical constraints, security requirements, and architectural invariants
  - Complex service dependencies and transaction boundaries
- **What to EXCLUDE**:
  - Code snippets (architectural explanations only)
  - Common framework integration details
  - Standard Rust patterns
  - Implementation details (those belong in PACKAGE.md)

### PACKAGE.md
- **Purpose**: Rich index and guidance document for implementation patterns
- **Philosophy**: PACKAGE.md serves as a **sophisticated index** that points AI assistants to the right files and explains **why** patterns exist, not duplicating implementation details
- **Scope**: Comprehensive but concise - explain patterns and point to files, don't reproduce code
- **Content Flow**: Architecture → Implementation → Integration → Extension → Commands
- **Content Focus**:
  - App-specific domain knowledge with file references
  - Non-conventional implementation patterns with **concise** code examples
  - Complex integration patterns unique to the app
  - Extension guidelines and safe modification patterns
- **Code Snippets**: **Minimal, focused examples** showing pattern structure only - always reference actual implementation files

## Key Principles

1. **Domain-Specific Focus**: Only include knowledge that's unique to BodhiApp
2. **Expert Assumptions**: Assume knowledge of common libraries and patterns
3. **No Duplication**: Each concept covered in one file only
4. **Complementary Design**: CLAUDE.md for architectural depth, PACKAGE.md for implementation patterns
5. **Reference Structure**: CLAUDE.md should reference PACKAGE.md at the top using relative path to project root
6. **Architectural Sophistication**: CLAUDE.md provides comprehensive architectural understanding, not superficial overviews
7. **Technical Depth Balance**: CLAUDE.md focuses on architecture and coordination, PACKAGE.md on implementation details
8. **PACKAGE.md as Rich Index**: PACKAGE.md explains patterns and points to files rather than duplicating implementation code
9. **File Reference Priority**: Always reference actual implementation files - AI assistants can read the source directly

## Framework Mentions

When referencing well-known frameworks:
- **Good**: "Uses serde for serialization"
- **Avoid**: "How to use serde derive macros and serialization patterns"
- **Good**: "Error handling with localized messages via Fluent"
- **Avoid**: "How to integrate fluent-rs with detailed code examples"

## Content Organization

### CLAUDE.md Detailed Structure Guidelines

#### Required Header Format
```markdown
# CLAUDE.md

This file provides guidance to Claude Code when working with the [crate name] crate.

*For detailed implementation examples and technical depth, see [relative/path/to/PACKAGE.md]*
```

#### Standard Section Organization
Follow this sophisticated content flow for comprehensive architectural guidance:

**1. Purpose Section** (## Level)
- Clear, concise description of the crate's role in BodhiApp
- Key architectural responsibilities and domain boundaries
- Primary value proposition within the overall system

**2. Key Domain Architecture Section** (## Level)
- **Sophisticated Subsection Modeling** (### Level): Break down major architectural components
- **System Architecture Patterns**: Document unique patterns and design decisions
- **Service Coordination Systems**: How multiple services work together
- **Domain-Specific Extensions**: Specialized functionality unique to the crate
- **Complex Workflows**: Multi-step processes and orchestration patterns

**3. Architecture Position Section** (## Level)  
- How the crate fits within BodhiApp's overall architecture
- Dependencies and relationships with other crates
- Position in the layered architecture (above/below other components)
- Cross-cutting concerns and integration points

**4. Cross-Crate Integration Patterns Section** (## Level, when applicable)
- Service interdependencies and coordination mechanisms
- Data flow and transaction boundaries across crates
- Error propagation and handling coordination
- Shared domain object usage patterns

**5. Important Constraints Section** (## Level)
- Technical constraints and architectural limitations
- Security requirements and safety invariants
- Lifecycle management requirements
- Integration requirements with external systems

#### Content Quality Standards for CLAUDE.md
- **Domain Architecture Focus**: Emphasize unique architectural patterns and decisions
- **Cross-Crate Coordination**: Document how services and components interact across boundaries
- **Technical Depth**: Provide sophisticated architectural understanding, not superficial overviews
- **Constraint Documentation**: Include security, performance, and integration constraints
- **Service Dependencies**: Document complex service interaction patterns
- **Architecture Positioning**: Clearly establish the crate's role in the overall system

#### Subsection Modeling Guidelines
- Use **### Level headers** for major architectural components within "Key Domain Architecture"
- Create **descriptive subsection names** like "Service Registry Pattern", "Authentication Coordination System", "Multi-Layer Security Architecture"
- Focus on **architectural patterns unique to BodhiApp**, not generic framework usage
- Document **cross-service coordination** and **complex workflows** comprehensively

### PACKAGE.md Detailed Structure Guidelines

#### Required Header Format
```markdown
# PACKAGE.md - [crate name or module name]

This document provides detailed technical information for the [crate/module description], focusing on BodhiApp-specific [domain/implementation] patterns...
```

#### Standard Section Organization
Follow this content flow for consistency:

**1. Architecture/Domain Position Section** (## Level)
- How the crate fits in BodhiApp's architecture
- Key architectural decisions and patterns
- Cross-crate dependencies and relationships

**2. Implementation Sections** (## Level, multiple sections as needed)
- Group related functionality (e.g., "Error System Implementation", "Service Registry Implementation")
- Each with subsections (### Level) for specific patterns
- **Bold subsection headers** for key implementation features

**3. Cross-Crate Integration Section** (## Level)  
- How this crate coordinates with others
- Service boundaries and data flow
- Integration patterns and constraints

**4. Extension Guidelines Section** (## Level)
- How to safely extend or modify the crate
- Safe development practices
- Subsections for different types of extensions

**5. Commands Section** (## Level)
- Testing, building, and operational commands
- Crate-specific tooling

#### Code Example Standards - CRITICAL CONSTRAINTS
- **Minimal, Pattern-Focused Snippets Only**: Show structure and key patterns, not full implementations
- **2-space indentation** for all code blocks
- **Maximum 10-15 lines per code snippet** - longer examples belong in actual source files
- **Always reference implementation files**: mention the relative to project path file for reference
- **Focus on pattern explanation**: Show the "what" briefly, explain the "why" extensively
- **Avoid reproducing test_utils code**: AI assistants have direct access to test_utils folders
- **No complete function implementations**: Show signatures, key patterns, and reference actual files

#### File Reference Standards - REQUIRED
Every code example must include file references:
```rust
// Example pattern structure (see src/auth/oauth.rs:45-67 for full implementation)
impl TokenScope {
    pub fn from_scope(scope_str: &str) -> Result<Self, ScopeParseError> {
        // Key pattern: validates offline_access requirement
        // Full validation logic in src/auth/oauth.rs
    }
}
```

#### Section Content Guidelines
- **Implementation Sections**: Use descriptive names like "OAuth2 PKCE Flow", "GGUF Binary Format System", "Cross-Crate Error Flow Architecture"
- **Key Implementation Features**: Use **bold subsection headers** to highlight important patterns
- **Integration Patterns**: Show how services coordinate via shared domain objects **with file references**
- **Extension Guidelines**: Provide step-by-step safe modification patterns **pointing to example implementations**
- **Cross-Crate Patterns**: Explain application-wide coordination requirements **with source file locations**

#### Content Quality Standards - Rich Index Philosophy
- **Rich Index Approach**: Function as a sophisticated index that guides AI assistants to the right files and patterns
- **Pattern Explanation Focus**: Explain **why** patterns exist and **where** to find full implementations
- **File Reference Mandate**: Every significant pattern must reference actual implementation files with line numbers
- **Concise Implementation Examples**: Show pattern structure only - AI assistants will read full implementations from source
- **Avoid Code Duplication**: Never reproduce complete implementations that exist in source files or test_utils
- **Context Over Code**: Provide extensive context about patterns, constraints, and coordination - minimal code reproduction

## Documentation Quality Standards

### CLAUDE.md Excellence Criteria
- **Comprehensive Architecture**: Document sophisticated domain modeling with detailed subsections
- **Cross-Crate Coordination**: Extensive integration patterns and service dependencies
- **Technical Depth**: Go beyond high-level overviews to provide detailed architectural understanding
- **Constraint Documentation**: Include security, lifecycle, and integration requirements comprehensively
- **Domain Sophistication**: Focus on complex, BodhiApp-specific architectural patterns

### PACKAGE.md Excellence Criteria - Rich Index Standards
- **Rich Index Quality**: Sophisticated navigation aid that efficiently guides AI assistants to relevant source files
- **Pattern Explanation Depth**: Comprehensive "why" explanations with minimal "what" code reproduction  
- **File Reference Completeness**: Every pattern includes specific source file references with line numbers
- **Concise Code Examples**: Maximum 10-15 line snippets showing structure only, not full implementations
- **Extension Guidance**: Clear step-by-step patterns **pointing to actual example implementations**
- **Cross-Crate Integration**: Service coordination explained **with source file locations for details**
- **Context Over Duplication**: Extensive architectural context without reproducing code available in source files

### Consistency Standards
- **CLAUDE.md**: Maintain sophisticated architectural depth with comprehensive domain modeling
- **PACKAGE.md**: Function as rich index with concise examples and extensive file references  
- **Complementary Design**: CLAUDE.md provides architectural understanding, PACKAGE.md guides to implementation details
- **Expert AI Focus**: Both documents provide the sophisticated domain knowledge that expert AI assistants need for productive BodhiApp development

## Anti-Patterns to Avoid in PACKAGE.md

### Code Duplication Anti-Patterns
- **Avoid**: Reproducing complete implementations that exist in source files
- **Avoid**: Long code examples (>15 lines) without file references
- **Avoid**: Copying test_utils code that AI assistants can read directly
- **Avoid**: Full function implementations in documentation

### Correct Rich Index Patterns
- **Preferred**: Brief structural examples with extensive file references
- **Preferred**: Pattern explanations with pointers to complete implementations
- **Preferred**: "Why" focus with "where to find details" guidance
- **Preferred**: Architectural context with source file navigation aids