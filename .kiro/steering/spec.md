---
inclusion: fileMatch
fileMatchPattern: ['ai-docs/02-features/**/*.md']
---
# Functional Specification Guidelines for AI Coding Assistants

## Purpose and Scope

This rule guides AI coding assistants in creating functional specifications that serve as anchor documents for sustained development work. These specifications are designed primarily for AI consumption to enable context continuity across development sessions.

## Core Specification Principles

### 1. Functional Over Technical
- **Focus on WHAT the feature does**, not HOW it's implemented
- Describe user interactions, data flows, and expected outcomes
- Avoid prescriptive technical implementation details
- Let AI assistants determine optimal technical approaches

### 2. Domain-Specific Context
- Emphasize app-specific concepts, business logic, and project terminology
- Include domain entities, business rules, and workflow patterns
- Reference existing project conventions and architectural decisions
- Avoid explaining well-known frameworks (React, Rust, Next.js, etc.)

### 3. AI Continuity Design
- Enable AI assistants to resume work after breaks
- Maintain clear progress tracking and implementation status
- Use reference-based approach to preserve context window efficiency
- Include changelog for AI-to-AI knowledge transfer

### 4. Reference-Based Documentation
- Use file references instead of large code snippets
- Link to relevant architecture documents using relative paths
- Reference existing patterns and conventions from codebase
- Leverage AI's ability to fetch referenced files on demand

## Required Specification Structure

### Header Section
```markdown
# Feature Name

## Overview
Brief functional description of what the feature accomplishes for users.

## Domain Context
- **Business Rules**: Domain-specific logic and constraints
- **Domain Entities**: Key data concepts and relationships
- **Workflow Patterns**: User interaction flows and state transitions
```

### Functional Requirements Section
```markdown
## Functional Requirements

### User Stories
As a [user type], I want [functional capability] so that [outcome/benefit].

### Acceptance Criteria
- [ ] Specific, testable functional outcomes
- [ ] Observable user behaviors and system responses
- [ ] Data validation and error handling scenarios
- [ ] Integration points with existing features
```

### Project Integration Section
```markdown
## Project Integration

### Architecture References
- [System Overview](mdc:../../01-architecture/system-overview.md)
- [Relevant Architecture Doc](mdc:../../01-architecture/specific-doc.md)

### Existing Patterns
- Reference similar features: `path/to/similar/feature.tsx`
- Follow conventions from: `path/to/pattern/example.rs`
- API patterns: Reference `ai-docs/01-architecture/api-integration.md`

### Dependencies
- Frontend components: Reference existing UI patterns
- Backend services: Reference service layer patterns
- Database changes: Reference migration patterns
```

### Implementation Tracking Section
```markdown
## Implementation Progress

### Completion Status
- [ ] Backend API endpoints
- [ ] Frontend user interface
- [ ] Data persistence layer
- [ ] Integration testing
- [ ] Documentation updates

### Current Phase
**Phase**: [Planning/Development/Testing/Complete]
**Last Updated**: [Date]
**Next Milestone**: [Specific deliverable]

### Implementation Notes
Brief notes on approach decisions and architectural choices made during development.
```

### AI Continuity Section
```markdown
## AI Development Changelog

### [Date] - [AI Assistant Session]
- **Completed**: Specific tasks accomplished
- **Approach**: Key technical decisions made
- **Next Steps**: Immediate next actions
- **Context**: Important context for next AI session

### [Date] - [AI Assistant Session]
- **Completed**: Previous session accomplishments
- **Blockers**: Any issues encountered
- **Decisions**: Architectural or implementation choices
```

## Content Guidelines

### What to Include
- **Functional behavior descriptions** using domain terminology
- **User interaction patterns** and expected system responses
- **Business rules and constraints** specific to the application
- **Data flow descriptions** without implementation details
- **Integration requirements** with existing features
- **Acceptance criteria** that can be tested functionally

### What to Avoid
- Technical implementation details or code snippets
- Framework-specific explanations (React hooks, Rust syntax, etc.)
- Human-centric business value statements
- Prescriptive technical constraints that limit AI flexibility
- Duplicate information available in referenced documents

### Domain-Specific Focus Areas
- **BodhiApp Context**: AI chat capabilities, model management, authentication flows
- **User Workflows**: Chat interactions, model selection, session management
- **Data Entities**: Messages, conversations, models, user preferences
- **Integration Points**: OAuth flows, API endpoints, desktop/web compatibility

## Specification Maintenance

### Regular Updates
- Update implementation progress after each development session
- Maintain AI changelog for context continuity
- Revise acceptance criteria based on development discoveries
- Update references when architecture documents change

### Quality Checks
- Ensure all functional requirements are testable
- Verify domain terminology consistency
- Confirm reference links are accurate and current
- Validate that specification enables AI context resumption

## Integration with Project Documentation

### Reference Hierarchy
1. **Architecture Documents**: `ai-docs/01-architecture/` for system patterns
2. **Feature Documentation**: `ai-docs/02-features/` for related features
3. **Crate Documentation**: `ai-docs/03-crates/` for implementation patterns
4. **Knowledge Transfer**: `ai-docs/06-knowledge-transfer/` for implementation guides

### File Organization
- Name the file with prefix of current date in format `yyyymmdd-<file-name>.md`
- Place specifications in `ai-docs/02-features/active-stories/`
- Move completed specs to `ai-docs/02-features/completed-stories/`
- Update `ai-docs/README.md` when adding new specifications
- Cross-reference related specifications using relative paths

## Example Functional Specification Template

```markdown
# Model Selection Enhancement

## Overview
Enable users to select and switch between different AI models during chat conversations with persistent model preferences.

## Domain Context
- **Business Rules**: Model availability based on user subscription tier
- **Domain Entities**: User, Model, Conversation, ModelPreference
- **Workflow Patterns**: Model selection → conversation context preservation → response generation

## Functional Requirements

### User Stories
As a chat user, I want to select different AI models so that I can optimize responses for different types of conversations.

### Acceptance Criteria
- [ ] User can view available models in conversation interface
- [ ] Model selection persists across conversation sessions
- [ ] Conversation context is maintained when switching models
- [ ] Model capabilities are clearly indicated to user

## Project Integration

### Architecture References
- [API Integration](mdc:../../01-architecture/api-integration.md)
- [Frontend React](mdc:../../01-architecture/frontend-react.md)

### Existing Patterns
- Follow chat interface patterns: `src/app/ui/chat/page.tsx`
- Model management service: `crates/services/src/models/`
- User preferences: Reference existing preference patterns

## Implementation Progress
- [ ] Backend model management API
- [ ] Frontend model selection UI
- [ ] Preference persistence
- [ ] Integration testing

## AI Development Changelog
### 2024-01-15 - Initial Planning
- **Completed**: Functional requirements definition
- **Next Steps**: Backend API design
- **Context**: Focus on user experience over technical optimization
```

This template demonstrates the functional focus, domain specificity, and AI continuity features required for effective AI-driven development specifications.