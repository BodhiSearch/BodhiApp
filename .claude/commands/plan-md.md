# /plan-md

Generate a comprehensive plan.md file in ai-docs/specs folder based on the conversation history and research conducted.

## Usage
```
/plan-md <folder-name>
```

If no folder name is provided, uses `unknown-feature` as the default.

## Instructions

Analyze the conversation history above and create a structured plan.md file following the established patterns from existing specs. The plan should be semi-technical documentation that:

1. **Captures all context and knowledge** gathered during the research phase
2. **Lists relevant files and methods** with relative paths from project root
3. **Includes critical code snippets** only when essential for understanding
4. **Serves as a reference** for resuming work after breaks or handoffs
5. **Documents architectural decisions** and technical insights discovered

## Structure Template

Follow this structure based on the analysis of existing plan.md files:

### 1. Title and Overview
- Clear, descriptive title with "Implementation Plan" suffix
- Brief overview paragraph explaining the feature/fix

### 2. Background/Motivation (if applicable)
- Current problem or limitation
- Why this change is needed
- Impact on users/system

### 3. Architecture Analysis
- Key components involved
- Current implementation details
- File paths and critical methods
- Code snippets ONLY when absolutely necessary

### 4. Implementation Design/Proposed Solution
- Core concept and approach
- Technical architecture
- Phase-by-phase breakdown with:
  - File paths (e.g., `crates/services/src/...`)
  - Method/function names
  - Brief implementation notes
  - Critical code patterns (minimal)

### 5. Key Design Decisions/Architecture Decisions
- Major technical choices made
- Rationale for each decision
- Trade-offs considered

### 6. Implementation Strategy/Phases
- Step-by-step implementation order
- Dependencies between phases
- Testing approach for each phase

### 7. Testing Strategy
- Unit tests approach
- Integration tests approach
- Manual testing scenarios

### 8. Success Criteria
- Clear, measurable outcomes
- Use ✅ checkboxes for trackable items

### 9. Additional Sections (as needed)
- Migration Path
- Rollback Strategy
- Security Considerations
- Performance Implications
- Documentation Requirements
- Future Enhancements

### 10. References/Key Files and Methods
- Summary of critical files
- Important methods/functions
- Related documentation

## Style Guidelines

1. **File References**: Always use relative paths from project root
   - Good: `crates/services/src/app_service.rs`
   - Bad: `/Users/xyz/project/crates/...`

2. **Code Snippets**: Include only when critical
   - Show patterns, not full implementations
   - Focus on interfaces and signatures
   - Keep snippets concise (5-20 lines max)

3. **Formatting**:
   - Use bold for **file paths** in prose
   - Use backticks for `method_names()` and inline code
   - Use code blocks with language hints for snippets
   - Use tables for comparison/matrix data
   - Use bullet points for lists
   - Use numbered lists for sequential steps

4. **Technical Depth**:
   - Be specific about technical decisions
   - Include discovered constraints and limitations
   - Document "gotchas" and non-obvious insights
   - Reference line numbers for critical sections

5. **Sections to Include** (adapt based on feature):
   - Overview/Problem Statement
   - Solution Design
   - Implementation Phases
   - Technical Decisions
   - Testing Strategy
   - Success Criteria
   - File/Method References

## Examples of Good Patterns

From existing specs:

1. **Phase Structure** (from safe-mode plan):
```markdown
### Phase 1: Create SafeModeAppService

#### 1.1 Define ConfigurationError Type
**File**: `crates/lib_bodhiserver/src/error.rs`
```rust
// Brief code pattern here
```
```

2. **Decision Documentation** (from api-models plan):
```markdown
### Key Design Decisions

1. **ID as Primary Key**: Use unique identifier...
   - **Rationale**: Clear reasoning here
   - **Impact**: What changes this causes
```

3. **File References** (from refactor-alias plan):
```markdown
### Key Components

#### 1. Service Layer (`crates/services/src/`)
**AppService trait** (`app_service.rs`):
- Central registry providing access to 11 business services
- All services must implement `Debug + Send + Sync`
```

## Output

Create the plan.md file at:
```
ai-docs/specs/<folder-name>/plan.md
```

The plan should be comprehensive enough that someone unfamiliar with the conversation can understand:
- What problem is being solved
- How it will be solved
- Where in the codebase changes will be made
- What the implementation order should be
- How to verify success

Remember: The goal is to save time by not repeating research and discovery steps when returning to the task.