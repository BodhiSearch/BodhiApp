# /next-iter-plan

Conduct comprehensive iteration planning with retrospective analysis, technical insights documentation, and structured next-phase recommendations for ongoing feature development.

## Usage
```
/next-iter-plan <spec-folder-name> [iteration-number]
```

If no folder name is provided, uses `unknown-feature` as the default.
If no iteration number is provided, auto-detects based on existing files.

## Inputs

```yaml
required:
  - spec_folder: '{folder-name}' # e.g., "20250907-user-request-access"
optional:
  - iteration: '{number}' # e.g., "2" (auto-detected if not provided)
  - focus_area: '{area}' # e.g., "backend", "frontend", "testing"
```

## Prerequisites

- Existing plan.md file in `ai-docs/specs/{spec-folder}/`
- Existing tasks.md file in `ai-docs/specs/{spec-folder}/`
- Previous iteration work completed or partially completed
- Access to implementation conversation history

## Planning Process - Comprehensive Analysis

### 1. Retrospective Analysis (Current State Assessment)

**A. Implementation Status Review**
- Review current progress against tasks.md checklist
- Identify completed vs planned work deviations
- Document what worked well and what didn't
- Analyze time estimates vs actual implementation time
- Note any scope changes or requirement clarifications

**B. Technical Discoveries Documentation**
- New patterns discovered in the existing codebase
- Integration points not initially considered
- Dependencies that were more/less complex than expected
- Architecture constraints or opportunities identified
- Performance implications discovered
- Security considerations that emerged

**C. Implementation Quality Assessment**
- Code quality of completed work
- Test coverage and testing approach effectiveness
- Documentation completeness
- Error handling robustness
- User experience considerations

### 2. Codebase Learning Integration

**A. Architecture Pattern Analysis**
- Successful patterns used (with file references)
- Anti-patterns avoided or corrected
- Integration strategies that worked well
- Service interaction patterns discovered
- Error propagation and handling patterns

**B. Testing Strategy Validation**
- Which testing approaches were most effective
- Test maintenance burden assessment
- Coverage gaps identified
- Mock/stub strategies that worked
- Performance test insights

**C. Development Workflow Insights**
- Build process optimization opportunities
- Development environment improvements needed
- Tool usage effectiveness (cargo, npm, make commands)
- Code generation pipeline insights

### 3. Risk Assessment and Mitigation

**A. Technical Risks**
- Dependencies that pose risks for upcoming phases
- Integration complexity for remaining work
- Performance bottlenecks identified
- Security vulnerabilities or concerns
- Data migration or compatibility issues

**B. Schedule and Scope Risks**
- Features that are more complex than estimated
- External dependencies causing delays
- Resource availability constraints
- Scope creep or requirement changes

**C. Quality Risks**
- Test debt accumulation
- Code quality degradation areas
- Documentation debt
- User experience consistency risks

### 4. Next Iteration Scope Definition

**A. Phase Grouping Analysis**
Following crate dependency chain:
```
objs → services → commands → server_core → auth_middleware → 
routes_oai → routes_app → routes_all → server_app → 
lib_bodhiserver → lib_bodhiserver_napi → bodhi → integration-tests
```

**B. Phase Optimization Opportunities**
- Phases that can be combined for efficiency
- Dependencies that allow parallel work
- Phases that should be split for risk management
- Critical path analysis for remaining work

**C. Implementation Strategy Selection**
- Full implementation vs simplified/mock approach
- Bottom-up vs top-down development approach
- Risk-first vs value-first prioritization
- Iterative vs waterfall approach for remaining phases

### 5. Resource and Complexity Planning

**A. Effort Estimation Refinement**
- Update estimates based on actual implementation experience
- Account for discovered complexity
- Include time for refactoring and optimization
- Buffer for integration and testing activities

**B. Skill and Knowledge Requirements**
- Technical expertise needed for upcoming phases
- Knowledge transfer requirements
- Training or research needed
- External consultation requirements

### 6. Quality and Success Criteria Updates

**A. Definition of Done Refinement**
- Update based on quality insights from previous iteration
- Add specific quality gates discovered to be important
- Include performance benchmarks if needed
- Clarify acceptance criteria based on user feedback

**B. Testing Strategy Evolution**
- Update testing approach based on what worked
- Add specific test scenarios discovered during implementation
- Define integration test strategy for upcoming phases
- Performance and load testing requirements

## Output 1: Update plan.md - New Insights Section

Add or update these sections in the existing plan.md:

```markdown
## Implementation Insights (Updated: {Date})

### Iteration {N} Retrospective

#### What Worked Well
- [List successful approaches, patterns, tools]
- [Include specific file references and line numbers where applicable]

#### Challenges Encountered
- [List difficulties and how they were resolved]
- [Include lessons learned and alternative approaches considered]

#### Technical Discoveries
- [New understanding about codebase architecture]
- [Integration patterns that were more/less complex than expected]
- [Performance or security considerations discovered]

### Updated Architecture Understanding

#### Key Components (Revised)
- [Updated understanding of critical components]
- [New integration points identified]
- [Service interaction patterns clarified]

#### Implementation Patterns That Work
- [Proven patterns from implementation with file examples]
- [Error handling approaches that are effective]
- [Testing strategies that provide good coverage]

### Revised Implementation Strategy

#### Approach Refinements
- [Changes to original approach based on learnings]
- [Simplified or enhanced strategies for remaining phases]
- [Risk mitigation strategies discovered]

#### Updated Phase Dependencies
- [Revised understanding of phase interdependencies]
- [New opportunities for parallel development]
- [Critical path updates]

### Quality and Performance Insights

#### Code Quality Patterns
- [Effective code organization strategies]
- [Maintainability approaches that work well]
- [Documentation strategies that add value]

#### Testing Insights
- [Effective test strategies and patterns]
- [Integration testing approaches that work]
- [Performance testing considerations]

### Risk Mitigation Updates

#### Newly Identified Risks
- [Risks discovered during implementation]
- [Dependencies that are more complex than expected]

#### Successful Risk Mitigations
- [Risk mitigation strategies that worked]
- [Early warning indicators that proved useful]
```

## Output 2: Update tasks.md - Progress and Next Iteration

Update the existing tasks.md with:

```markdown
## Progress Update (Iteration {N} - {Date})

### Completed This Iteration
- [x] Phase X: [Name] - ✅ Completed
  - **Implementation Notes**: [Key insights, challenges, solutions]
  - **Files Modified**: [List of files with brief description of changes]
  - **Testing Results**: [Test outcomes, coverage achieved]
  - **Quality Notes**: [Code quality, performance, security insights]

### Partially Completed
- [🟡] Phase Y: [Name] - 🟡 Partially Complete
  - **Completed Tasks**: [List completed sub-tasks]
  - **Remaining Tasks**: [List remaining sub-tasks]
  - **Blockers/Issues**: [Any issues encountered]

### Lessons Learned
- **Technical**: [Technical insights that affect remaining work]
- **Process**: [Development process insights]
- **Quality**: [Quality and testing insights]

---

## Iteration {N+1} Plan

### Recommended Scope
Based on retrospective analysis and current progress:

**Primary Focus**: [Backend/Frontend/Integration/Testing]

**Phases for This Iteration**:
- Phase X: [Name] - [Rationale for inclusion]
- Phase Y: [Name] - [Rationale for inclusion or grouping]

### Phase X: [Name] (Planned for Iteration {N+1})
**Goal**: [Updated goal based on current understanding]
**Complexity Assessment**: [Low/Medium/High based on learnings]
**Risk Level**: [Low/Medium/High]

**Updated Implementation Approach**:
- [Specific approach based on learnings from previous iterations]
- [Patterns to follow from successful previous work]
- [Risks to mitigate based on experience]

**Files to Modify** (Updated based on codebase understanding):
- `path/to/file.rs` - [Specific changes needed based on architecture understanding]
- `path/to/file.tsx` - [Frontend changes with component patterns identified]

### Task X.1: [Task Name] (Revised)
- [ ] [Specific action items updated based on implementation experience]
- [ ] [Additional tasks discovered during previous iteration]
- [ ] **Test**: [Testing approach refined based on what worked]

### Commands to Run (Proven Effective)
```bash
# Based on successful commands from previous iteration
cargo check -p [crate-name]
cargo test -p [crate-name] [specific-test-pattern]
cargo fmt -p [crate-name]

# Additional commands discovered to be useful
[any new commands discovered during implementation]
```

### Quality Gates for This Iteration
- [ ] All existing tests continue to pass
- [ ] New functionality has appropriate test coverage (>=X% based on previous iteration)
- [ ] Code follows patterns established in previous iteration
- [ ] Performance benchmarks met (if applicable)
- [ ] Security considerations addressed (if applicable)

### Success Criteria (Refined)
- [Specific, measurable criteria updated based on current understanding]
- [Include quality metrics that proved important]
- [User experience criteria if applicable]

### Risk Mitigation Plan
- **High Priority Risks**: [Based on previous iteration experience]
- **Mitigation Strategies**: [Proven strategies to apply]
- **Early Warning Indicators**: [Signs to watch for based on experience]
```

## Output 3: Create Iteration Planning Report

Create a new file: `ai-docs/specs/{spec-folder}/iteration-{N+1}-plan.md`

```markdown
# Iteration {N+1} Planning Report
**Feature**: [Feature Name]
**Planning Date**: {Date}
**Previous Iterations**: {N}

## Executive Summary

### Current State
- [Brief overview of what's been accomplished]
- [Current completion percentage]
- [Key technical milestones achieved]

### This Iteration Goals
- [Primary objectives for upcoming iteration]
- [Success criteria]
- [Expected deliverables]

## Retrospective Analysis

### Iteration {N} Outcomes
**Planned vs Actual**:
- Planned: [What was planned]
- Actual: [What was delivered]
- Variance: [Analysis of differences]

**Key Learnings**:
- [Most important technical insights]
- [Process improvements identified]
- [Quality improvements achieved]

### Implementation Velocity
- **Estimated effort**: [Original estimates]
- **Actual effort**: [Time actually spent]
- **Velocity factor**: [Actual/Estimated ratio]
- **Complexity adjustments**: [How estimates should be adjusted]

## Technical Architecture Updates

### Codebase Understanding Evolution
- [New patterns discovered]
- [Integration complexity insights]
- [Service interaction patterns]
- [Testing strategy refinements]

### Implementation Pattern Library
Based on successful implementations:

```rust
// Example pattern that worked well
// [Include 5-10 line code snippets of successful patterns]
```

```typescript
// Frontend patterns that were effective
// [Include component patterns that worked]
```

## Next Iteration Strategy

### Phase Selection Rationale
**Selected Phases**:
1. **Phase X**: [Name and rationale]
   - **Complexity**: [Assessment based on experience]
   - **Dependencies**: [What it depends on and what depends on it]
   - **Risk Level**: [Based on similar work completed]

2. **Phase Y**: [Name and rationale if grouping multiple phases]

**Rejected/Deferred Phases**:
- **Phase Z**: [Why deferred - complexity, dependencies, risk]

### Implementation Approach

#### Development Strategy
- [Top-down vs bottom-up based on what worked]
- [Full implementation vs simplified approach]
- [Integration strategy based on successful patterns]

#### Quality Strategy
- [Testing approach based on effective strategies]
- [Code review and quality gates]
- [Documentation and maintainability approach]

#### Risk Management
- [Specific risks for this iteration]
- [Mitigation strategies based on experience]
- [Early warning indicators to monitor]

## Resource Planning

### Time Estimates (Calibrated)
- **Development**: [Hours based on velocity from previous iteration]
- **Testing**: [Including integration and manual testing]
- **Documentation**: [Based on actual needs discovered]
- **Buffer**: [Risk buffer based on complexity assessment]

### Dependencies
- **Technical**: [Service dependencies, API dependencies]
- **Knowledge**: [Expertise needed, research required]
- **External**: [Third-party services, team dependencies]

## Success Metrics

### Functional Metrics
- [Specific functionality that will be working]
- [User scenarios that will be supported]
- [Integration points that will be functional]

### Quality Metrics
- [Test coverage targets based on experience]
- [Performance benchmarks if applicable]
- [Code quality metrics]

### Process Metrics
- [Velocity targets based on historical data]
- [Quality gate pass rates]
- [Technical debt metrics]

## Risk Assessment

### High Priority Risks
1. **[Risk Name]**: 
   - **Probability**: [High/Medium/Low]
   - **Impact**: [High/Medium/Low]
   - **Mitigation**: [Specific strategy]

### Risk Monitoring Plan
- [Specific indicators to watch]
- [Review checkpoints during iteration]
- [Escalation triggers]

## Decision Points

### Go/No-Go Criteria
- [Specific criteria that must be met to proceed]
- [Quality gates that must pass]
- [Dependency requirements]

### Scope Adjustment Triggers
- [Conditions that would require scope reduction]
- [Options for scope adjustment]
- [Alternative approaches if primary approach fails]

## Iteration Execution Plan

### Week-by-Week Breakdown
**Week 1**: [Focus area and deliverables]
**Week 2**: [Focus area and deliverables]
**Week N**: [Focus area and deliverables]

### Checkpoints
- **Daily**: [Progress tracking approach]
- **Mid-iteration**: [Review and adjustment point]
- **End-iteration**: [Completion criteria and retrospective]

### Communication Plan
- [Progress reporting approach]
- [Stakeholder communication strategy]
- [Issue escalation process]
```

## Key Principles

### Adaptive Planning
- Plans are living documents that evolve with learning
- Technical insights drive strategy adjustments
- Risk-based prioritization takes precedence over feature-based
- Quality is built in, not added later

### Evidence-Based Decisions
- All recommendations based on actual implementation experience
- Metrics and data drive planning decisions
- Retrospective insights inform future strategy
- Learning from both successes and failures

### Pragmatic Balance
- Balance between technical perfection and business value
- Consider maintenance burden in all decisions
- Optimize for team productivity and code quality
- Focus on sustainable development practices

## Completion Criteria

After running this command:

1. **plan.md updated** with implementation insights and revised strategy
2. **tasks.md updated** with progress and next iteration scope  
3. **iteration-{N+1}-plan.md created** with comprehensive planning analysis
4. **Clear next steps** with specific phases and rationale
5. **Risk mitigation strategy** based on actual implementation experience
6. **Quality strategy** refined based on what worked

The output should provide everything needed to begin the next iteration with confidence, backed by evidence from actual implementation experience.

## Advanced Features

### Integration with Existing Commands

This command works well in sequence with:
- `/plan-md` for initial feature planning
- `/task-md` for converting plans to actionable tasks
- Can inform future `/plan-md` runs for new features

### Customization Options

The command can be extended with:
- Focus area filtering (backend-only, frontend-only planning)
- Risk level filtering (conservative vs aggressive planning)
- Integration with project management tools
- Automated metric collection from previous iterations

### Output Format Options

- **Detailed**: Full analysis with all sections
- **Executive**: Summary-focused for stakeholder communication  
- **Technical**: Deep-dive technical insights for development team
- **Update-only**: Just update existing files without creating new planning document