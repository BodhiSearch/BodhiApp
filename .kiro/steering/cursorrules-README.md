---
inclusion: always
---
# Bodhi App Cursor Rules System

This directory contains modular Cursor rules that automatically apply based on the files you're working with. Each rule provides targeted guidance for specific aspects of the codebase.

## Rule Organization

### Core Rules (Always Apply)
- **`general-development.mdc`** - Core development principles, coding standards, implementation guidelines, and documentation references

### Automatic Rules (Auto-attach based on file patterns)
- **`frontend-react.mdc`** - React/TypeScript frontend development (crates/bodhi/src/)
- **`ui-design-system.mdc`** - UI/UX design and styling guidelines
- **`rust-backend.mdc`** - Rust backend development across all crates
- **`tauri-desktop.mdc`** - Tauri desktop application specific patterns
- **`api-routes.mdc`** - API routes and endpoints development
- **`authentication.mdc`** - Authentication and authorization code
- **`testing.mdc`** - Testing patterns for frontend and backend
- **`build-config.mdc`** - Build and configuration file management
- **`documentation.mdc`** - Documentation and feature implementation


### Nested Rules
- **`crates/services-db.mdc`** - Database layer specific to services crate

## How It Works

When you open files in specific directories or with certain extensions, relevant rules are automatically included in the AI context:

```
crates/bodhi/src/components/chat/ChatMessage.tsx
├── Triggers: frontend-react.mdc
├── Triggers: ui-design-system.mdc  
└── Always: general-development.mdc

crates/services/src/db/service.rs
├── Triggers: rust-backend.mdc
├── Triggers: services-db.mdc
└── Always: general-development.mdc

crates/bodhi/src-tauri/src/native.rs
├── Triggers: tauri-desktop.mdc
├── Triggers: rust-backend.mdc
└── Always: general-development.mdc
```

## Documentation-Driven Development

Each rule references the updated ai-docs structure and avoids duplicating content. This ensures:

- **Consistency** with authoritative documentation in ai-docs
- **Up-to-date guidance** as the ai-docs folder evolves
- **Focused rules** that reference specific ai-docs files rather than duplicating conventions
- **Maintainable system** that stays synchronized with documentation updates
- **Current file references** that match the reorganized ai-docs structure

## Usage Tips

- Rules are automatically applied - no manual action needed
- Use `@Cursor Rules` in chat to explicitly reference rules
- Check rule descriptions to understand what guidance applies
- Follow the ai-docs references for detailed implementation patterns

## Adding New Rules

When adding new rules:
1. Use descriptive file names with `.mdc` extension
2. Include proper metadata (description, globs, alwaysApply)
3. Reference appropriate ai-docs files for detailed guidance
4. Test glob patterns match intended file types
5. Update this README when adding significant new rules
