# Requirements Document

## Introduction

This feature simplifies the llama-server parameter handling by replacing the current structured `GptContextParams` with a direct string array passthrough approach. The `OAIRequestParams` will remain unchanged as it handles OpenAI-compatible request parameters. The system will store and pass raw llama-server command-line arguments directly, providing maximum flexibility and eliminating the need to maintain parameter mappings for server-specific configurations.

## Requirements

### Requirement 1: Replace GptContextParams with String Arrays

**User Story:** As a developer, I want to store llama-server context parameters as string arrays instead of structured objects, so that I can pass any llama-server parameter without being limited by predefined structures.

#### Acceptance Criteria

1. WHEN storing alias configuration THEN the system SHALL change `context_params` field to contain an array of strings instead of structured parameters
2. WHEN storing alias configuration THEN the system SHALL keep `request_params` field unchanged for OpenAI compatibility
3. WHEN the `context_params` field is empty THEN the system SHALL use default llama-server behavior
4. WHEN the `context_params` field contains values THEN each string SHALL be passed as a command-line argument to llama-server
5. WHEN processing context parameters THEN the system SHALL pass them directly to llama-server without validation

### Requirement 2: Update API Endpoints for Parameter Handling

**User Story:** As an API consumer, I want to create and update model aliases using the new parameter format, so that I can configure any llama-server parameter directly.

#### Acceptance Criteria

1. WHEN creating a new alias via API THEN the system SHALL accept `context_params` as an array of strings
2. WHEN creating a new alias via API THEN the system SHALL continue to accept `request_params` for OpenAI compatibility
3. WHEN updating an existing alias via API THEN the system SHALL accept the new `context_params` format
4. WHEN processing API requests THEN the system SHALL pass context parameters directly without validation
5. WHEN returning alias data via API THEN the system SHALL include the `context_params` field as an array of strings in the response

### Requirement 3: Update Frontend UI for Parameter Management

**User Story:** As a user, I want to configure llama-server context parameters using a simple text interface, so that I can specify any parameter without being limited by predefined form fields.

#### Acceptance Criteria

1. WHEN editing context parameters in the UI THEN the system SHALL provide a textarea input field
2. WHEN entering parameters THEN the user SHALL input them as newline-separated command-line arguments (e.g., "--ctx-size 2048" on one line, "--parallel 4" on another)
3. WHEN submitting the form THEN the system SHALL split the textarea content by newlines into a string array
4. WHEN saving parameter changes THEN the system SHALL store them as string arrays in `context_params`
5. WHEN displaying existing parameters THEN the system SHALL show them as newline-separated strings in the textarea

### Requirement 4: Handle Data Format Validation

**User Story:** As a system component, I want to ensure that alias configurations use the correct parameter format, so that the system fails fast on incorrect data.

#### Acceptance Criteria

1. WHEN loading an alias configuration THEN the system SHALL expect `context_params` to be an array of strings
2. WHEN `context_params` is not an array of strings THEN the system SHALL return an error
3. WHEN deserializing alias data THEN the system SHALL validate the format and fail if incorrect
4. WHEN creating new aliases THEN the system SHALL only accept the new string array format
5. WHEN the system encounters invalid data THEN it SHALL provide clear error messages indicating the expected format

### Requirement 5: Update Server Process Integration

**User Story:** As a system component, I want to pass parameters directly to llama-server without intermediate processing, so that the system is more maintainable and supports all llama-server features.

#### Acceptance Criteria

1. WHEN starting a llama-server process THEN the system SHALL append `context_params` array directly to the command line
2. WHEN `context_params` contains conflicting parameters THEN the system SHALL use the last occurrence (following standard CLI behavior)
3. WHEN `context_params` contains invalid parameters THEN llama-server SHALL handle the validation and error reporting
4. WHEN the server process starts THEN it SHALL receive all parameters in the correct order
5. WHEN logging server startup THEN the system SHALL include the complete command line for debugging

### Requirement 6: Comprehensive Testing and Validation

**User Story:** As a developer, I want comprehensive test coverage for the new parameter system, so that I can be confident in the system's reliability.

#### Acceptance Criteria

1. WHEN running unit tests THEN all parameter conversion logic SHALL be tested
2. WHEN running integration tests THEN the complete API flow SHALL be validated
3. WHEN running migration tests THEN all legacy parameter combinations SHALL be tested
4. WHEN running UI tests THEN the parameter editor SHALL be fully functional
5. WHEN running end-to-end tests THEN the complete workflow from UI to server startup SHALL be validated