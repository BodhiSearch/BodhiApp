# routes_app Crate - Pre-Revamp Coverage Baseline

**Generated**: 2026-02-08
**Purpose**: This is the baseline coverage report for the `routes_app` crate before any test revamp changes. Use this to measure improvement after the revamp.

## Test Results

- **Total tests**: 231 (229 passed, 0 failed, 2 ignored)
- **All tests passing**: Yes

## Coverage Summary (routes_app files only)

| File | Regions | Missed | Region Cover | Functions | Missed | Fn Cover | Lines | Missed | Line Cover |
|------|---------|--------|-------------|-----------|--------|----------|-------|--------|------------|
| `api_dto.rs` | 29 | 11 | 62.07% | 21 | 6 | 71.43% | 84 | 40 | **52.38%** |
| `routes_api_models/api_models.rs` | 197 | 87 | 55.84% | 44 | 14 | 68.18% | 516 | 114 | **77.91%** |
| `routes_api_models/types.rs` | 141 | 3 | 97.87% | 52 | 1 | 98.08% | 419 | 7 | **98.33%** |
| `routes_api_token.rs` | 58 | 7 | 87.93% | 15 | 0 | 100.00% | 219 | 0 | **100.00%** |
| `routes_api_token_test.rs` | 64 | 47 | 26.56% | 60 | 47 | 21.67% | 84 | 47 | **44.05%** |
| `routes_auth/login.rs` | 134 | 39 | 70.90% | 10 | 0 | 100.00% | 282 | 12 | **95.74%** |
| `routes_auth/request_access.rs` | 50 | 25 | 50.00% | 6 | 3 | 50.00% | 111 | 28 | **74.77%** |
| `routes_auth/types.rs` | 1 | 0 | 100.00% | 1 | 0 | 100.00% | 1 | 0 | **100.00%** |
| `routes_dev.rs` | 21 | 21 | 0.00% | 4 | 4 | 0.00% | 40 | 40 | **0.00%** |
| `routes_models/aliases.rs` | 177 | 73 | 58.76% | 36 | 11 | 69.44% | 474 | 97 | **79.54%** |
| `routes_models/error.rs` | 1 | 1 | 0.00% | 1 | 1 | 0.00% | 1 | 1 | **0.00%** |
| `routes_models/metadata.rs` | 57 | 45 | 21.05% | 12 | 8 | 33.33% | 138 | 66 | **52.17%** |
| `routes_models/pull.rs` | 127 | 27 | 78.74% | 23 | 1 | 95.65% | 398 | 15 | **96.23%** |
| `routes_models/types.rs` | 11 | 1 | 90.91% | 8 | 0 | 100.00% | 13 | 1 | **92.31%** |
| `routes_oai/chat.rs` | 49 | 11 | 77.55% | 9 | 0 | 100.00% | 179 | 9 | **94.97%** |
| `routes_oai/models.rs` | 57 | 21 | 63.16% | 10 | 1 | 90.00% | 173 | 19 | **89.02%** |
| `routes_ollama/handlers.rs` | 114 | 77 | 32.46% | 35 | 22 | 37.14% | 384 | 134 | **65.10%** |
| `routes_ollama/types.rs` | 45 | 34 | 24.44% | 28 | 17 | 39.29% | 138 | 117 | **15.22%** |
| `routes_settings.rs` | 60 | 14 | 76.67% | 18 | 2 | 88.89% | 195 | 6 | **96.92%** |
| `routes_settings_test.rs` | 64 | 47 | 26.56% | 60 | 47 | 21.67% | 84 | 47 | **44.05%** |
| `routes_setup.rs` | 68 | 15 | 77.94% | 18 | 4 | 77.78% | 166 | 12 | **92.77%** |
| `routes_setup_test.rs` | 28 | 0 | 100.00% | 17 | 0 | 100.00% | 59 | 0 | **100.00%** |
| `routes_toolsets/error.rs` | 1 | 0 | 100.00% | 1 | 0 | 100.00% | 1 | 0 | **100.00%** |
| `routes_toolsets/toolsets.rs` | 156 | 14 | 91.03% | 47 | 2 | 95.74% | 396 | 2 | **99.49%** |
| `routes_toolsets/types.rs` | 18 | 2 | 88.89% | 15 | 2 | 86.67% | 26 | 4 | **84.62%** |
| `routes_users/access_request.rs` | 98 | 77 | 21.43% | 19 | 11 | 42.11% | 251 | 106 | **57.77%** |
| `routes_users/management.rs` | 56 | 29 | 48.21% | 12 | 4 | 66.67% | 128 | 32 | **75.00%** |
| `routes_users/types.rs` | 10 | 2 | 80.00% | 10 | 2 | 80.00% | 17 | 2 | **88.24%** |
| `routes_users/user_info.rs` | 60 | 15 | 75.00% | 6 | 3 | 50.00% | 91 | 3 | **96.70%** |
| `shared/common.rs` | 1 | 0 | 100.00% | 1 | 0 | 100.00% | 1 | 0 | **100.00%** |
| `shared/openapi.rs` | 396 | 55 | 86.11% | 47 | 3 | 93.62% | 784 | 148 | **81.12%** |
| `shared/pagination.rs` | 5 | 0 | 100.00% | 5 | 0 | 100.00% | 11 | 0 | **100.00%** |
| `shared/utils.rs` | 19 | 3 | 84.21% | 6 | 0 | 100.00% | 32 | 2 | **93.75%** |
| `test_utils/alias_response.rs` | 3 | 1 | 66.67% | 3 | 1 | 66.67% | 39 | 5 | **87.18%** |

## Overall Totals (including all dependency crates exercised by routes_app tests)

| Metric | Total | Missed | Coverage |
|--------|-------|--------|----------|
| Regions | 8,792 | 5,231 | **40.50%** |
| Functions | 2,501 | 1,359 | **45.66%** |
| Lines | 17,222 | 8,356 | **51.48%** |

## Files with 0% Line Coverage (no test coverage)

- `routes_dev.rs` - 40 lines, development-only routes
- `routes_models/error.rs` - 1 line

## Files with Low Coverage (below 60% line coverage)

- `routes_ollama/types.rs` - **15.22%** (138 lines, 117 missed)
- `routes_api_token_test.rs` - **44.05%** (test file, coverage artifact)
- `routes_settings_test.rs` - **44.05%** (test file, coverage artifact)
- `api_dto.rs` - **52.38%** (84 lines, 40 missed)
- `routes_models/metadata.rs` - **52.17%** (138 lines, 66 missed)
- `routes_users/access_request.rs` - **57.77%** (251 lines, 106 missed)

## Files with High Coverage (90%+ line coverage)

- `routes_api_models/types.rs` - **98.33%**
- `routes_api_token.rs` - **100.00%**
- `routes_auth/login.rs` - **95.74%**
- `routes_auth/types.rs` - **100.00%**
- `routes_models/pull.rs` - **96.23%**
- `routes_models/types.rs` - **92.31%**
- `routes_oai/chat.rs` - **94.97%**
- `routes_settings.rs` - **96.92%**
- `routes_setup.rs` - **92.77%**
- `routes_setup_test.rs` - **100.00%**
- `routes_toolsets/error.rs` - **100.00%**
- `routes_toolsets/toolsets.rs` - **99.49%**
- `routes_users/user_info.rs` - **96.70%**
- `shared/common.rs` - **100.00%**
- `shared/pagination.rs` - **100.00%**
- `shared/utils.rs` - **93.75%**

## Notes

- The TOTAL row includes coverage from dependency crates (objs, services, server_core, auth_middleware, etc.) that are exercised by routes_app tests, not just routes_app source files.
- Test files (`*_test.rs`) showing lower coverage is a known artifact of how llvm-cov instruments test code -- some test helper functions may not be called by every test.
- `routes_dev.rs` at 0% coverage is expected as it contains development-only routes not exercised in unit tests.
