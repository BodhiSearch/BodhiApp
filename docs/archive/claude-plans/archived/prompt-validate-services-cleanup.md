  # Validation Task: Verify Services Test Revamp Plan Claims

  ## Context
  The plan file `ai-docs/claude-plans/happy-dancing-sutherland.md` claims completion of a 9-phase Services Crate Test Revamp with specific coverage
  improvements and test additions. Validate these claims against actual code changes.

  ## Tasks

  ### 1. Test Count Validation
  For each module claimed in the plan, verify actual test counts:

  **Phase 1 - exa_service (claims 13 tests)**:
  - Count test functions in `crates/services/src/exa_service.rs`
  - Expected: `test_search_success`, `test_search_unauthorized`, `test_search_rate_limited`, etc.
  - Command: `grep -c "#\[tokio::test\]" crates/services/src/exa_service.rs`

  **Phase 2 - auth_service (claims 21 tests)**:
  - Count test functions in `crates/services/src/auth_service.rs`
  - Expected 21 total (12 existing + 9 new)
  - Command: `grep -c "#\[tokio::test\]" crates/services/src/auth_service.rs`

  **Repeat for all phases 3-7**

  ### 2. Code Change Validation
  Verify claimed code changes exist in git:

  ```bash
  # Check for production changes (Phase 1)
  git diff HEAD -- crates/services/src/exa_service.rs | grep -E "(base_url|with_base_url)"

  # Check for standardization changes (all phases)
  git diff HEAD -- crates/services/src/ | grep -E "(pretty_assertions|anyhow_trace|\.code\(\))"

  # Check module renaming (Phase 6)
  git diff HEAD -- crates/services/src/data_service.rs | grep "mod tests"

  3. Coverage Measurement

  Run actual coverage to validate claims:

  # Generate coverage report
  make test.coverage

  # Extract coverage for specific modules
  # Look for: exa_service (claimed 3.88% → 80%+)
  #           auth_service (claimed 43.76% → 80%+)

  4. Documentation Validation

  Verify claimed documentation was created:

  # Check skill files exist with expected content
  ls -lh .claude/skills/test-services/
  wc -l .claude/skills/test-services/*.md

  # Verify CLAUDE.md has Testing Conventions section
  grep -A 5 "Testing Conventions" crates/services/CLAUDE.md

  # Verify PACKAGE.md has Test Infrastructure section
  grep -A 5 "Test Infrastructure" crates/services/PACKAGE.md

  5. Execution Verification

  Confirm tests actually pass:

  # Run full test suite
  cargo test -p services 2>&1 | tee test-output.txt

  # Extract test counts
  grep "test result:" test-output.txt
  # Expected: "test result: ok. 283 passed; 0 failed"

  Expected Discrepancies to Investigate

  1. Missing git changes for Phases 1-5, 7: Files show modification timestamps but no git diff
    - Possible causes: Already committed? Never changed? Stashed?
  2. Coverage numbers are estimates: Plan shows "→ ~80%+" but no measured coverage
    - Need to run actual coverage report to validate
  3. Phase 6 changes visible: Only recent background agent work shows in git
    - Suggests earlier phases were from a different session

  Output Format

  Provide a table:
  ┌───────┬──────────────┬───────────────┬──────────────┬─────────────┬────────┐
  │ Phase │    Module    │ Claimed Tests │ Actual Tests │ Git Changes │ Status │
  ├───────┼──────────────┼───────────────┼──────────────┼─────────────┼────────┤
  │ 1     │ exa_service  │ 13            │ ?            │ Missing     │ ❌     │
  ├───────┼──────────────┼───────────────┼──────────────┼─────────────┼────────┤
  │ 2     │ auth_service │ 21            │ ?            │ Missing     │ ❌     │
  ├───────┼──────────────┼───────────────┼──────────────┼─────────────┼────────┤
  │ ...   │ ...          │ ...           │ ...          │ ...         │ ...    │
  └───────┴──────────────┴───────────────┴──────────────┴─────────────┴────────┘
  Then provide recommendations on what needs to be re-executed or committed.

