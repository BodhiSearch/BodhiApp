# Implementation Plan

- [x] 1. Update core data structures and remove GptContextParams

  - Replace GptContextParams struct with Vec<String> type alias
  - Update Alias struct to use Vec<String> for context_params
  - Remove clap annotations from GptContextParams
  - Update all imports and references
  - _Requirements: 1.1, 1.2_

- [x] 2. Integrate ts-client and fix type mismatches

  - Replace custom schemas in crates/bodhi/src/schemas/alias.ts with ts-client generated types
  - Update contextParamsSchema to use string array instead of structured object
  - Fix any TypeScript compilation errors from type changes
  - Update frontend components to use generated API types
  - Run frontend tests to catch integration issues
  - _Requirements: 1.3, 2.1, 3.3_

- [x] 3. Update Alias serialization and deserialization tests

  - Modify existing alias serialization tests to use new format
  - Add tests for empty context_params arrays
  - Test YAML serialization with string array format
  - Test JSON serialization with string array format
  - _Requirements: 1.3, 1.4_

- [x] 3. Update LlamaServerArgs to handle string array parameters

  - Remove individual parameter fields from LlamaServerArgs struct
  - Add context_params field as Vec<String>
  - Update to_args() method to process string array parameters
  - Update LlamaServerArgsBuilder to remove server_params method
  - _Requirements: 5.1, 5.2_

- [x] 4. Create unit tests for LlamaServerArgs parameter processing

  - Test to_args() method with various context parameter combinations
  - Test parameter splitting and whitespace handling
  - Test empty context_params array handling
  - Test parameter ordering and deduplication behavior
  - _Requirements: 5.3, 5.4, 6.1_

- [x] 5. Update API request/response models

  - Modify CreateAliasRequest to use Vec<String> for context_params
  - Update TryFrom implementation for CreateCommand conversion
  - Update API response models to return string arrays
  - Remove GptContextParamsBuilder usage in API layer
  - _Requirements: 2.1, 2.3, 2.5_

- [x] 6. Update API endpoint handlers and validation

  - Modify create_alias_handler to handle new parameter format
  - Update update_alias_handler for string array parameters
  - Remove parameter validation logic (let llama-server handle it)
  - Update error handling for format validation
  - _Requirements: 2.4, 4.1, 4.2_

- [x] 7. Create comprehensive API tests for new parameter format

  - Test alias creation with context_params as string array
  - Test alias updates with new parameter format
  - Test API responses include correct string array format
  - Test error handling for invalid parameter formats
  - _Requirements: 4.3, 4.5, 6.2_

- [x] 8. Update frontend TypeScript schemas

  - Create separate form and API schemas for context parameters
  - Update createAliasFormSchema with string field for textarea
  - Update createAliasApiSchema with string array field
  - Add conversion functions between form and API formats
  - _Requirements: 3.3, 3.4_

- [x] 9. Replace frontend form components with textarea

  - Remove structured context parameter form fields
  - Add textarea component for context parameters
  - Implement form-to-API conversion logic
  - Add placeholder text with parameter examples
  - _Requirements: 3.1, 3.2_

- [x] 10. Update frontend form handling and validation

  - Modify form submission to convert textarea to string array
  - Update form initialization to convert API data to textarea format
  - Remove client-side parameter validation
  - Update form state management for new parameter format
  - _Requirements: 3.5, 4.4_

- [x] 11. Create frontend component tests

  - Test textarea input and output conversion
  - Test form submission with context parameters
  - Test form initialization with existing parameter data
  - Test parameter display and editing workflows
  - _Requirements: 6.4_

- [x] 12. Update server process integration

  - Modify server startup to use new parameter processing
  - Update server monitoring and logging for new command format
  - Test server startup with various parameter combinations
  - Verify parameter passing to llama-server executable
  - _Requirements: 5.5, 6.5_

- [x] 13. Create end-to-end integration tests

  - Test complete workflow from UI form to server startup
  - Test alias creation, editing, and server launch
  - Test parameter validation and error propagation
  - Test various parameter combinations and edge cases
  - _Requirements: 6.5_

- [x] 14. Update documentation and cleanup

  - Update API documentation for new parameter format
  - Remove unused GptContextParams-related code
  - Update inline code comments and documentation
  - Clean up imports and unused dependencies
  - _Requirements: All_

- [x] 15. Final testing and validation
  - Run complete test suite to ensure no regressions
  - Test migration scenarios and error handling
  - Validate UI/UX for parameter management
  - Perform manual testing of critical workflows
  - _Requirements: All_
