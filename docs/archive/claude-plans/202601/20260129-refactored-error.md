# Error Handling Simplification Plan

## ✅ COMPLETED - 2026-01-29

**Status**: All phases completed successfully
**Commits**: 18 local commits created (not pushed)
**Tests**: All passing (1,300+ unit tests, 60 Playwright e2e tests)
**Breaking Change**: `ErrorBody.param` field type changed from `Option<String>` to `Option<HashMap<String, String>>`

### Final Results
- ✅ Migrated 100+ error types from FTL localization to thiserror templates
- ✅ Removed FluentLocalizationService and all 22 FTL files
- ✅ Converted HubApiError from struct to enum with 6 variants
- ✅ Created HttpError wrapper in server_core for axum integration
- ✅ Reduced AppService from 11 to 10 services
- ✅ Updated all CLAUDE.md and PACKAGE.md documentation
- ✅ Fixed all unit and e2e tests for new error messages

### Key Learnings
- Sequential crate-by-crate migration prevented cascading failures
- Agent context file pattern worked well for sharing insights
- ErrorBody.param HashMap provides structured error arguments
- Thiserror Display messages are now the single source of truth
- Test expectations needed updates for user-facing message changes

---

## Summary
Simplify BodhiApp's error handling by:
1. Moving messages from FTL files to thiserror `#[error("...")]` templates
2. Standardizing error messages for consistency and user-friendliness
3. Eliminating ApiError intermediate struct
4. Removing FluentLocalizationService entirely
5. Using ErrorBody.param for args HashMap
6. Updating documentation to reflect new error patterns

## Decisions Captured
- **Code attribute**: Keep both explicit override and auto-derived fallback
- **Code format**: `{enum_name}-{variant_name}` snake_case (current behavior)
- **Param field**: `Option<HashMap<String, String>>` for args
- **error_type**: Remains mandatory on every variant/struct
- **Transparent errors**: Delegate all three (error_type, code, args) by default
- **External errors**: Use `args_delegate=false` -> `{error: e.to_string()}`
- **CLI display**: Use thiserror Display directly
- **IntoResponse**: Create in server_core (keep axum out of objs)
- **ApiError**: Eliminate - direct AppError -> OpenAIApiError conversion
- **Migration**: Single PR, agent-based sequential (one crate at a time)
- **FTL cleanup**: Delete after migration complete
- **Multi-line messages**: Simplify to single line
- **HubApiError**: Convert to enum with variants per error kind (gated_access, may_not_exist, etc.)
- **Wrapped errors**: Include wrapped error in message (`"Network error: {source}."`)
- **Agent context file**: Markdown, read at start, update at end, remove stale info
- **Test updates**: Remove tests validating error messages (now hardcoded), convert unicode markers to '...'
- **Test utils cleanup**: Remove `setup_l10n` and `assert_error_message` immediately in objs phase
- **Keep smoke tests**: Retain tests for HTTP status codes and error_type, remove message text assertions
- **Missing messages**: Derive from context (struct fields, usage patterns)
- **Validation errors**: Include field name when relevant
- **URLs in messages**: Hardcode HuggingFace URLs
- **Commits**: Local commits per phase group, no push until final verification
- **Doc updates**: Single agent updates all CLAUDE.md and PACKAGE.md files

---

## Error Message Standardization Guidelines

### Principles
1. **User-friendly language**: Avoid technical jargon where possible
2. **Actionable guidance**: Include what the user can do to resolve
3. **Consistent format**: All messages follow same structural pattern
4. **Sentence case**: Start with capital, end with period
5. **Single line**: No multi-line messages (wrap with `\n` if needed for CLI)

### Message Structure
```
<What happened>. <Context if helpful>. <Recovery action if applicable>.
```

### Technical Term Substitutions
| Technical Term | User-Friendly Alternative |
|---------------|---------------------------|
| `$BODHI_HOME` | "Bodhi data folder" |
| `$BODHI_HOME/aliases` | "model configurations" |
| `huggingface-cli login` | "log in to Hugging Face" |
| `OAuth authentication` | "login" or "sign in" |
| `session` | "login session" |
| `authentication header` | "login credentials" |
| `refresh token` | "login session" |
| `reqwest middleware error` | "network error" |
| `serializing/deserializing` | "reading/writing" |

### Message Categories

**User-Facing (simplified):**
- Auth errors, model errors, validation errors, access control errors

**Developer-Facing (technical details acceptable):**
- IO errors, serialization errors, builder errors, internal server errors

---

## Agent-Based Migration Approach

### Agent Context File
Create `ai-docs/claude-plans/error-migration-ctx.md` for agents to share insights.

**Workflow:**
1. Agent reads the file at start of task
2. Agent performs migration work
3. Agent updates the file at end:
   - Adds own insights under new date/crate header
   - Removes stale/incorrect information discovered during work
   - Keeps useful patterns for future agents

**Content includes:**
- Patterns discovered
- Edge cases encountered
- Decisions made for ambiguous cases
- Warnings for future agents

### Crate Migration Order (Sequential)
1. `errmeta_derive` - Verify macro behavior
2. `objs` - Foundation types, ErrorBody update
3. `services` - Business logic errors
4. `llama_server_proc` - LLM process errors
5. `server_core` - HTTP infrastructure
6. `auth_middleware` - Authentication errors
7. `routes_oai` - OpenAI API errors
8. `routes_app` - App API errors
9. `routes_all` - Composed routes
10. `commands` - CLI errors
11. `server_app` - Server errors
12. `lib_bodhiserver` - Library errors
13. `bodhi` (tauri) - Desktop app errors

---

## Phase errmeta-derive-agent

### Agent Context
```
Crate: errmeta_derive
Task: Verify macro generates correct fallback codes
Dependencies: None
```

### Agent Instructions
1. Read `crates/errmeta_derive/src/lib.rs`
2. Verify `generate_code_arm()` falls back to snake_case name
3. Run existing tests: `cargo test -p errmeta_derive`
4. Write insights to `ai-docs/claude-plans/error-migration-ctx.md`

### Verification
```bash
cargo test -p errmeta_derive
```

---

## Phase objs-agent

### Agent Context
```
Crate: objs
Task: Update ErrorBody, simplify ApiError, migrate error messages
Dependencies: errmeta_derive verified
Previous insights: Read error-migration-ctx.md
```

### Changes Required

**1. ErrorBody Update** (`crates/objs/src/error/error_oai.rs`)
```rust
#[serde(skip_serializing_if = "Option::is_none")]
pub param: Option<HashMap<String, String>>,  // was Option<String>
```

**2. ApiError Simplify** (`crates/objs/src/error/error_api.rs`)
- Remove `FluentLocalizationService` usage
- Remove `localized_msg()` function
- Remove `EN_US` static
- Update `From<ApiError> for OpenAIApiError` to use `value.name` directly

**3. Error Message Migration** (`crates/objs/src/error/objs.rs`)

| Error | New `#[error("...")]` |
|-------|----------------------|
| BadRequestError | `"Invalid request: {reason}."` |
| NotFoundError | `"{reason}."` |
| InternalServerError | `"Internal error: {reason}."` |
| UnauthorizedError | `"Access denied: {reason}."` |
| ServiceUnavailableError | `"Service unavailable: {reason}."` |
| ConflictError | `"Resource conflict: {reason}."` |
| UnprocessableEntityError | `"Cannot process request: {reason}."` |
| IoError | `"File operation failed: {source}."` |
| IoWithPathError | `"File operation failed for '{path}': {source}."` |
| IoDirCreateError | `"Failed to create folder '{path}': {source}."` |
| IoFileReadError | `"Failed to read file '{path}': {source}."` |
| IoFileWriteError | `"Failed to write file '{path}': {source}."` |
| IoFileDeleteError | `"Failed to delete file '{path}': {source}."` |
| SerdeJsonError | `"Failed to process JSON data: {source}."` |
| SerdeJsonWithPathError | `"Failed to process JSON file '{path}': {source}."` |
| SerdeYamlError | `"Failed to process YAML data: {source}."` |
| SerdeYamlWithPathError | `"Failed to process YAML file '{path}': {source}."` |
| ReqwestError | `"Network error: {error}."` |
| JsonRejectionError | `"Invalid JSON in request: {source}."` |
| RwLockReadError | `"Concurrent access error: {reason}."` |
| EntityError::NotFound | `"{0} not found."` |
| ObjValidationError::ValidationErrors | `"{0}"` |
| ObjValidationError::FilePatternMismatch | `"Invalid repository format '{0}'. Expected 'username/repo'."` |
| ObjValidationError::ForwardAllRequiresPrefix | `"Prefix is required when forwarding all requests."` |
| BuilderError::UninitializedField | `"Configuration incomplete: missing {0}."` |
| BuilderError::ValidationError | `"Configuration invalid: {0}."` |
| AppRegInfoMissingError | `"Application registration information is missing."` |

**4. Other objs error files**
- `crates/objs/src/error/l10n.rs` - LocalizationMessageError
- `crates/objs/src/gguf/error.rs` - GGUFMetadataError

| Error | New `#[error("...")]` |
|-------|----------------------|
| LocalizationMessageError::MessageNotFound | `"Message template not found: {0}."` |
| LocalizationMessageError::FormatPattern | `"Message formatting failed: {0}."` |
| GGUFMetadataError::InvalidMagic | `"Invalid model file format: {0}."` |
| GGUFMetadataError::MalformedVersion | `"Invalid model version: {0}."` |
| GGUFMetadataError::UnexpectedEOF | `"Model file appears truncated."` |
| GGUFMetadataError::InvalidString | `"Model contains invalid text: {0}."` |
| GGUFMetadataError::UnsupportedVersion | `"Unsupported model version: {0}."` |
| GGUFMetadataError::InvalidValueType | `"Invalid model metadata type: {0}."` |
| GGUFMetadataError::InvalidArrayValueType | `"Invalid model metadata array type: {0}."` |
| GGUFMetadataError::TypeMismatch | `"Model metadata type mismatch: expected {expected}, got {actual}."` |

**5. Remove Test Utils** (`crates/objs/src/test_utils.rs`)
- Delete `setup_l10n` fixture function
- Delete `assert_error_message` helper function
- This will cause compile errors in downstream crates - intentional to catch all usages

**6. Update/Remove Tests**
- **Remove**: Tests that only validated error message text (now hardcoded in thiserror)
- **Keep**: Tests that verify HTTP status codes, error_type values, args HashMap
- **Convert**: Replace unicode isolation markers `\u{2068}...\u{2069}` with simple `'...'` quotes in remaining tests
- **Pattern**: `assert_eq!(expected_status, error.status())` not `assert_error_message(...)`

### Verification
```bash
cargo build -p objs  # Expect test_utils removal to break downstream
cargo test -p objs
```

### Commit
```bash
git add -A && git commit -m "refactor(objs): migrate errors to thiserror, remove localization"
```

---

## Phase services-agent

### Agent Context
```
Crate: services
Task: Migrate all service error messages
Dependencies: objs completed
Previous insights: Read error-migration-ctx.md
```

### Key Error Migrations

**HubService Errors:**
| Error | New `#[error("...")]` |
|-------|----------------------|
| HubFileNotFoundError | `"File '{filename}' not found in repository '{repo}'."` |
| RemoteModelNotFoundError | `"Remote model '{alias}' not found. Check the alias name and try again."` |

**HubApiError - Convert from struct with kind to enum:**
```rust
// OLD: struct with HubApiErrorKind
pub struct HubApiError {
  kind: HubApiErrorKind,
  repo: String,
  error: String,
}

// NEW: enum with variants
#[derive(Debug, thiserror::Error, errmeta_derive::ErrorMeta)]
#[error_meta(trait_to_impl = AppError)]
pub enum HubApiError {
  #[error("Access to '{repo}' requires approval. Visit https://huggingface.co/{repo} to request access.")]
  #[error_meta(error_type = ErrorType::Forbidden)]
  GatedAccess { repo: String, error: String },

  #[error("Repository '{repo}' not found or requires authentication. Run 'huggingface-cli login' to authenticate.")]
  #[error_meta(error_type = ErrorType::NotFound)]
  MayNotExist { repo: String, error: String },

  #[error("Repository '{repo}' is disabled or has been removed.")]
  #[error_meta(error_type = ErrorType::NotFound)]
  RepoDisabled { repo: String, error: String },

  #[error("Hugging Face API error: {error}.")]
  #[error_meta(error_type = ErrorType::InternalServer)]
  Unknown { repo: String, error: String },
}
```

**DataService Errors:**
| Error | New `#[error("...")]` |
|-------|----------------------|
| DataServiceError::DirMissing | `"Bodhi data folder not initialized. Run 'bodhi init' to set up."` |
| AliasNotFoundError | `"Model configuration '{0}' not found."` |
| AliasExistsError | `"Model configuration '{0}' already exists."` |
| DataFileNotFoundError | `"File '{filename}' not found in '{dirname}' folder."` |

**AuthService Errors:**
| Error | New `#[error("...")]` |
|-------|----------------------|
| AuthServiceError::AuthServiceApiError | `"Authentication service error: {0}."` |
| AuthServiceError::ReqwestMiddlewareError | `"Network error during authentication: {0}."` |

**TokenError:**
| Error | New `#[error("...")]` |
|-------|----------------------|
| TokenError::ScopeEmpty | `"User does not have any access permissions."` |
| TokenError::Expired | `"Session has expired. Please log in again."` |

**DbError:**
| Error | New `#[error("...")]` |
|-------|----------------------|
| DbError::TokenValidation | `"Invalid token: {0}."` |
| DbError::EncryptionError | `"Encryption error: {0}."` |
| DbError::PrefixExists | `"Prefix '{0}' is already used by another API model."` |

**AiApiServiceError:**
| Error | New `#[error("...")]` |
|-------|----------------------|
| AiApiServiceError::RateLimit | `"Too many requests to API. Please wait and try again."` |
| AiApiServiceError::PromptTooLong | `"Message too long. Maximum length is {max_length} but received {actual_length}."` |
| AiApiServiceError::ModelNotFound | `"API model '{0}' not found."` |
| AiApiServiceError::Unauthorized | `"API authentication failed: {0}."` |

**ToolsetError (missing - derive from context):**
| Error | New `#[error("...")]` |
|-------|----------------------|
| ToolsetError::ToolsetNotFound | `"Toolset '{0}' not found."` |
| ToolsetError::MethodNotFound | `"Toolset method '{0}' not found."` |
| ToolsetError::ToolsetNotConfigured | `"Toolset is not configured for this user."` |
| ToolsetError::ToolsetDisabled | `"Toolset is disabled."` |
| ToolsetError::ExecutionFailed | `"Toolset execution failed: {0}."` |
| ToolsetError::ToolsetAppDisabled | `"Toolset application is disabled."` |

**ExaError (missing - derive from context):**
| Error | New `#[error("...")]` |
|-------|----------------------|
| ExaError::RequestFailed | `"Search request failed: {0}."` |
| ExaError::RateLimited | `"Search rate limit exceeded. Please wait and try again."` |
| ExaError::InvalidApiKey | `"Search API key is invalid or missing."` |
| ExaError::Timeout | `"Search request timed out."` |

**Test Updates:**
- Fix compile errors from removed `setup_l10n` and `assert_error_message`
- Remove tests that only checked message text
- Keep tests verifying status codes and error_type

### Verification
```bash
cargo build -p services
cargo test -p services
```

### Commit
```bash
git add -A && git commit -m "refactor(services): migrate errors to thiserror, convert HubApiError to enum"
```

---

## Phase llama-server-proc-agent

### Agent Context
```
Crate: llama_server_proc
Task: Migrate LLM process error messages
Dependencies: objs completed
```

### Error Migrations
| Error | New `#[error("...")]` |
|-------|----------------------|
| ServerError::ServerNotReady | `"Model server is starting up. Please wait and try again."` |
| ServerError::HealthCheckError | `"Model server health check failed: {0}."` |
| ServerError::TimeoutError | `"Model server did not respond within {0} seconds."` |

### Verification
```bash
cargo build -p llama_server_proc
cargo test -p llama_server_proc
```

---

## Phase server-core-agent

### Agent Context
```
Crate: server_core
Task: Add HttpError wrapper, migrate errors
Dependencies: objs, services completed
```

### Changes

**1. Create HttpError wrapper** (`crates/server_core/src/error_response.rs`)
```rust
use axum::{body::Body, response::{IntoResponse, Response}};
use objs::{AppError, ErrorBody, OpenAIApiError};

pub struct HttpError<E: AppError>(pub E);

impl<E: AppError> IntoResponse for HttpError<E> {
  fn into_response(self) -> Response {
    let args = self.0.args();
    let openai_error = OpenAIApiError {
      error: ErrorBody {
        message: self.0.to_string(),
        r#type: self.0.error_type(),
        code: Some(self.0.code()),
        param: if args.is_empty() { None } else { Some(args) },
      },
      status: self.0.status(),
    };
    Response::builder()
      .status(openai_error.status)
      .header("Content-Type", "application/json")
      .body(Body::from(serde_json::to_string(&openai_error).unwrap()))
      .unwrap()
  }
}
```

**2. Error Migrations**
| Error | New `#[error("...")]` |
|-------|----------------------|
| ContextError::LlamaCppError | `"Model initialization failed: {error}."` |
| ContextError::Unreachable | `"Internal error: {0}."` |
| ContextError::ExecNotExists | `"Model executable not found: {0}."` |
| ModelRouterError::AiApiModelNotFound | `"Model '{0}' not found."` |

### Verification
```bash
cargo test -p server_core
```

---

## Phase auth-middleware-agent

### Agent Context
```
Crate: auth_middleware
Task: Migrate authentication error messages
Dependencies: objs completed
```

### Error Migrations
| Error | New `#[error("...")]` |
|-------|----------------------|
| AuthError::InvalidAccess | `"Access denied."` |
| AuthError::RefreshTokenNotFound | `"Session expired. Please log out and log in again."` |
| AuthError::TokenInactive | `"API token is inactive."` |
| AuthError::TokenNotFound | `"API token not found."` |
| AuthError::MissingRoles | `"User has no valid access roles."` |
| ApiAuthError::Forbidden | `"Insufficient permissions for this resource."` |
| ApiAuthError::MissingAuth | `"Authentication required. Provide an API key or log in."` |
| ToolsetAuthError::MissingUserId | `"User identification missing from request."` |
| ToolsetAuthError::MissingAuth | `"Authentication required for toolset access."` |
| ToolsetAuthError::ToolsetNotFound | `"Toolset not found."` |

### Verification
```bash
cargo test -p auth_middleware
```

---

## Phase routes-oai-agent

### Agent Context
```
Crate: routes_oai
Task: Migrate OpenAI-compatible route errors
Dependencies: objs, server_core completed
```

### Error Migrations
| Error | New `#[error("...")]` |
|-------|----------------------|
| HttpError::Serialization | `"Response serialization failed: {0}."` |

### Verification
```bash
cargo test -p routes_oai
```

---

## Phase routes-app-agent

### Agent Context
```
Crate: routes_app
Task: Migrate application route errors
Dependencies: objs, server_core completed
```

### Error Migrations
| Error | New `#[error("...")]` |
|-------|----------------------|
| AppServiceError::AlreadySetup | `"Application is already set up."` |
| LoginError::SessionInfoNotFound | `"Login session not found. Are cookies enabled?"` |
| LoginError::OAuthError | `"Login failed: {0}."` |
| PullError::FileAlreadyExists | `"File '{filename}' already exists in '{repo}'."` |
| SettingsError::NotFound | `"Setting '{0}' not found."` |
| SettingsError::BodhiHome | `"BODHI_HOME can only be changed via environment variable."` |
| ApiTokenError::AccessTokenMissing | `"Access token is missing."` |

### Verification
```bash
cargo test -p routes_app
```

---

## Phase remaining-crates-agent

### Agent Context
```
Crates: routes_all, commands, server_app, lib_bodhiserver, bodhi
Task: Migrate remaining error messages
Dependencies: All above completed
```

### server_app Errors
| Error | New `#[error("...")]` |
|-------|----------------------|
| InteractiveError::GptParamsBuilderError | `"Model configuration error: {error}."` |
| InteractiveError::OpenAIError | `"Request error: {error}."` |
| TaskJoinError | `"Background task failed: {source}."` |
| ServeError::Unknown | `"Server started but readiness signal not received."` |

### lib_bodhiserver Errors
| Error | New `#[error("...")]` |
|-------|----------------------|
| AppServiceBuilderError | `"{0}"` |
| AppServiceBuilderError::PlaceholderValue | `"Encryption key not properly configured."` |

### bodhi (tauri) Errors
| Error | New `#[error("...")]` |
|-------|----------------------|
| AppExecuteError::NativeNotSupported | `"This feature is not available in this build."` |
| AppExecuteError::Unreachable | `"Internal error: {0}."` |
| NativeError::Tauri | `"Desktop application error: {error}."` |

### Verification
```bash
cargo test -p routes_all -p commands -p server_app -p lib_bodhiserver
```

---

## Phase cleanup-localization-agent

### Agent Context
```
Task: Delete all FTL files and localization infrastructure
Dependencies: All crate migrations completed
```

### Files to Delete
```
crates/*/src/resources/  (all directories)
crates/objs/src/gguf/resources/
crates/objs/src/localization_service.rs
```

### Code to Remove
- `FluentLocalizationService` struct and impls
- `LocalizationService` trait
- `SUPPORTED_LOCALES` constant
- `L10N_RESOURCES` exports from all crates
- `setup_l10n` test fixture
- `assert_error_message` test helper

### Dependencies to Remove (Cargo.toml)
- `fluent`
- `fluent-syntax`
- `unic-langid`
- `include_dir` (if only used for FTL)

### Verification
```bash
make test
```

---

## Phase docs-update-agent

### Agent Context
```
Task: Update all CLAUDE.md and PACKAGE.md files to reflect new error patterns
Dependencies: All migrations completed
```

### Files to Update

**CLAUDE.md files (15 files):**
- `crates/errmeta_derive/CLAUDE.md` - Remove localization references, update for thiserror-only
- `crates/objs/CLAUDE.md` - Major update: remove FluentLocalizationService section, update error handling
- `crates/services/CLAUDE.md` - Update error handling patterns section
- `crates/server_core/CLAUDE.md` - Add HttpError wrapper documentation
- `crates/auth_middleware/CLAUDE.md` - Update error examples
- `crates/routes_oai/CLAUDE.md` - Update error response format
- `crates/routes_app/CLAUDE.md` - Update error examples
- `crates/routes_all/CLAUDE.md` - Update error composition
- `crates/commands/CLAUDE.md` - Update CLI error handling
- `crates/server_app/CLAUDE.md` - Update error handling
- `crates/llama_server_proc/CLAUDE.md` - Update error types
- `crates/lib_bodhiserver/CLAUDE.md` - Update error patterns
- `crates/lib_bodhiserver_napi/CLAUDE.md` - Update error handling
- `crates/ci_optims/CLAUDE.md` - Minimal updates if any
- `crates/integration-tests/CLAUDE.md` - Update test patterns

**PACKAGE.md files (15 files):**
Same crates - update implementation details and file references.

**Other MD files:**
- `crates/lib_bodhiserver/app_service_builder.md` - Update error handling
- `crates/bodhi/README.md` - Update if error-related content
- `crates/lib_bodhiserver_napi/README.md` - Update if error-related content

### Key Documentation Changes

1. **Remove all FluentLocalizationService references**
2. **Update error creation pattern examples:**
   ```rust
   // Old pattern
   #[error("error_code")]
   #[error_meta(error_type = ErrorType::BadRequest)]
   struct MyError { field: String }
   // FTL: error_code = message {$field}

   // New pattern
   #[error("User-friendly message: {field}.")]
   #[error_meta(error_type = ErrorType::BadRequest)]
   struct MyError { field: String }
   ```

3. **Update test pattern examples:**
   ```rust
   // Old
   assert_error_message(localization, &error.code(), error.args(), expected);

   // New
   assert_eq!(expected, error.to_string());
   ```

4. **Add error message guidelines reference**

### Verification
```bash
# Read all CLAUDE.md files to ensure no stale localization references
grep -r "FluentLocalizationService\|messages.ftl\|localization" crates/*/CLAUDE.md
```

---

## Agent Context File Template

Create `ai-docs/claude-plans/error-migration-ctx.md`:

```markdown
# Error Migration Context

This file captures insights from agents working on error migration.
Each agent appends their findings here.

## Insights Log

### [Date] - [Crate] - [Agent]
**Patterns:**
- ...

**Edge Cases:**
- ...

**Decisions:**
- ...

**Warnings for Future Agents:**
- ...
```

---

## Testing Strategy

### Per-Agent Verification
```bash
cargo build -p <crate>
cargo test -p <crate>
cargo fmt -p <crate>
```

### Commit Strategy (Local Only)
**Phase Group 1 - Foundation:**
- objs-agent completes → commit

**Phase Group 2 - Services:**
- services-agent, llama-server-proc-agent complete → commit

**Phase Group 3 - Infrastructure:**
- server-core-agent, auth-middleware-agent complete → commit

**Phase Group 4 - Routes:**
- routes-oai-agent, routes-app-agent, remaining-crates-agent complete → commit

**Phase Group 5 - Cleanup:**
- cleanup-localization-agent complete → commit

**Phase Group 6 - Documentation:**
- docs-update-agent complete → commit

### Final Verification
```bash
make test
make format.all
# Only push after all tests pass
```

---

## Critical Files Summary

| Phase | Critical Files |
|-------|---------------|
| errmeta-derive | `crates/errmeta_derive/src/lib.rs` |
| objs | `crates/objs/src/error/error_oai.rs`, `error_api.rs`, `objs.rs`, `l10n.rs`, `gguf/error.rs` |
| services | `crates/services/src/**/*.rs` (all error types) |
| llama_server_proc | `crates/llama_server_proc/src/error.rs` |
| server_core | `crates/server_core/src/error_response.rs` (new), error files |
| auth_middleware | `crates/auth_middleware/src/error.rs` |
| routes_oai | `crates/routes_oai/src/error.rs` |
| routes_app | `crates/routes_app/src/error.rs` |
| cleanup | All `resources/` directories, `localization_service.rs` |
| docs | All `CLAUDE.md` and `PACKAGE.md` files |

---

## Execution Summary

### Phase Completion Timeline

#### ✅ Phase errmeta-derive-agent
- Verified macro generates correct fallback codes
- Tests passed: errmeta_derive (3 tests)

#### ✅ Phase objs-agent
- Updated ErrorBody.param to HashMap<String, String>
- Removed FluentLocalizationService dependencies
- Migrated 29 error types to thiserror templates
- Tests passed: objs (399 tests)
- Commit: `refactor(objs): migrate errors to thiserror, remove localization`

#### ✅ Phase services-agent
- Converted HubApiError from struct to enum with 6 variants
- Migrated 40+ error types including hub, data, auth, toolset services
- Tests passed: services (283 tests)
- Commit: `refactor(services): migrate errors to thiserror, convert HubApiError to enum`

#### ✅ Phase llama-server-proc-agent
- Migrated LLM process errors
- Tests passed: llama_server_proc (7 tests)
- Commit: `refactor(llama_server_proc): migrate errors to thiserror`

#### ✅ Phase server-core-agent
- Created HttpError wrapper for axum IntoResponse
- Migrated infrastructure errors
- Tests passed: server_core (97 tests)
- Commit: `refactor(server_core): add HttpError wrapper, migrate errors to thiserror`

#### ✅ Phase auth-middleware-agent
- Migrated authentication and authorization errors
- Tests passed: auth_middleware (140 tests)
- Commit: `refactor(auth_middleware): migrate errors to thiserror`

#### ✅ Phase routes-oai-agent and routes-app-agent
- Migrated OpenAI-compatible and application route errors
- Tests passed: routes_oai + routes_app (229 tests)
- Commit: `refactor(routes_oai,routes_app): migrate errors to thiserror`

#### ✅ Phase remaining-crates-agent
- Migrated routes_all, commands, server_app, lib_bodhiserver, bodhi (tauri)
- Tests passed: (78 tests)
- Commits (6 separate commits):
  - `refactor(routes_all): remove localization infrastructure`
  - `refactor(server_app): migrate errors to thiserror`
  - `refactor(lib_bodhiserver): migrate errors to thiserror`
  - `refactor(bodhi): migrate tauri errors to thiserror`
  - `docs(migration): update context with remaining crates insights`

#### ✅ Phase cleanup-localization-agent
- Deleted 22 FTL files from all crates
- Removed FluentLocalizationService infrastructure
- Removed fluent, fluent-syntax, unic-langid dependencies
- Commit: `refactor(cleanup): remove all FTL files and localization infrastructure`

#### ✅ Phase docs-update-agent
- Updated 30+ CLAUDE.md and PACKAGE.md files
- Removed all localization references
- Updated error handling patterns
- Commit: `docs: update all CLAUDE.md and PACKAGE.md for error migration`

#### ✅ UI Rebuild and E2E Tests
- Rebuilt embedded UI: `make build.ui-rebuild`
- Fixed Playwright test expectations
- Tests passed: 60 Playwright e2e tests (1 skipped)
- Commits:
  - `test(playwright): fix error message expectation for new format`
  - `fix: remove remaining localization references and fix tests`
  - `test(playwright): fix api-models error message pattern`

### Critical Changes Not in Original Plan

1. **ErrorBody.param Breaking Change**: Changed from `Option<String>` to `Option<HashMap<String, String>>` - this is the only breaking API change
2. **Auth test message update**: `test_cross_client_token_exchange_no_user_scope` expected message updated from "user does not have any privileges on this system" to "User does not have any access permissions."
3. **Playwright regex patterns**: Updated from literal match to regex patterns with wildcards (e.g., `/Prefix.*already.*used/i`)
4. **Agent context file**: Created and maintained `ai-docs/claude-plans/error-migration-ctx.md` throughout migration

### Dependencies Removed
- `fluent = "0.16"`
- `fluent-syntax = "0.11"`
- `unic-langid = "0.9"`
- `include_dir = "0.7"` (from crates that only used it for FTL)

### Final Verification Results
```bash
make test
# All tests passed:
# - Backend: 1,300+ unit tests
# - UI: Component tests
# - NAPI: Integration tests
# - E2E: 60 Playwright tests (1 skipped)

git status
# Clean working tree, 18 commits ahead of origin/main
```

### Recommended Commit Message for Squash
```
refactor!: migrate error handling from FTL localization to thiserror templates

BREAKING CHANGE: ErrorBody.param field type changed from Option<String> to Option<HashMap<String, String>>

This refactors the entire error handling system to use inline thiserror
templates instead of external FTL localization files, while preserving
error codes and HTTP status codes.

Major Changes:
- Migrated 100+ error types from FTL files to thiserror #[error("...")] templates
- Removed FluentLocalizationService and all 22 FTL localization files
- Changed ErrorBody.param to support structured error arguments
- Converted HubApiError from struct with kind enum to full enum with 6 variants
- Added HttpError wrapper in server_core for axum IntoResponse integration
- Reduced AppService from 11 to 10 services (removed localization service)

Error Message Improvements:
- User-friendly language without technical jargon
- Consistent sentence case with periods
- Actionable guidance where appropriate
- Field interpolation using {field} syntax

All error codes, HTTP status codes, and response structure remain
unchanged except for the param field type.
```
