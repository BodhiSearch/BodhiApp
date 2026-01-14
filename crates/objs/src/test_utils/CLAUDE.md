# CLAUDE.md

This file provides guidance to Claude Code when working with the `test_utils` module.

*For implementation details and extension patterns, see [crates/objs/src/test_utils/PACKAGE.md](crates/objs/src/test_utils/PACKAGE.md)*

## Purpose

The `test_utils` module serves as BodhiApp's **foundational testing infrastructure**, providing specialized utilities that enable comprehensive cross-crate testing with domain-specific fixtures, localization validation, and complex data generation.

## Cross-Crate Testing Architecture

### Universal Localization Testing Foundation
Critical testing capability used across all BodhiApp crates:
- **setup_l10n()**: Aggregates Fluent message templates from multiple crates (objs, auth_middleware, services, routes)
- **Mock localization service**: Overrides singleton `FluentLocalizationService::get_instance()` for isolated testing
- **Multi-language validation**: Tests error message consistency in en-US and fr-FR across service boundaries
- **Cross-crate resource loading**: Enables services and routes to test localized errors without circular dependencies
- **Fallback mechanism validation**: Ensures graceful degradation when translations are missing across all components

### Environment Isolation for Integration Testing
Sophisticated environment fixtures used by downstream crates:
- **temp_bodhi_home()**: Complete BodhiApp configuration directory used by services for alias testing
- **temp_hf_home()**: Hugging Face cache structure replication for `HubService` and `DataService` integration tests
- **empty_bodhi_home()**: Clean configuration environment for initialization testing across CLI and service layers
- **SNAPSHOT constant**: Fixed model snapshot hash ensuring reproducible tests across services and routes

### Domain Object Factory Ecosystem
Comprehensive builders enabling consistent testing across all BodhiApp layers:
- **Repo factories**: `llama3()`, `testalias()` used by `HubService`, `DataService`, and route integration tests
- **HubFile builders**: Model file representations with cache validation (`HubFileBuilder::testalias()`, `HubFileBuilder::llama3_tokenizer()`) used throughout service layer testing  
- **Alias builders**: YAML-serializable configurations (`AliasBuilder::llama3()` with stop tokens, `AliasBuilder::testalias()`) tested across CLI, services, and route layers
- **Error builders**: Localized error construction enabling consistent error handling tests across crates
- **Parameter builders**: OpenAI parameter fixtures with realistic configurations used in route testing and CLI validation

### Cross-Layer API Testing Infrastructure
Testing utilities that support multiple application layers:
- **parse<T>()**: Generic JSON response parsing used by route integration tests and service validation
- **parse_txt()**: Text response extraction supporting both HTTP route testing and CLI output validation
- **Request/response fixtures**: Pre-built API objects used across route tests, service mocks, and CLI integration tests
- **Mock HTTP clients**: Configurable HTTP simulation for testing service external dependencies

### Advanced Data Generation Pipeline
Python integration supporting complex cross-crate testing scenarios:
- **generate_test_data_gguf_metadata()**: GGUF binary files used by `DataService`, `HubService`, and route endpoint tests
- **generate_test_data_chat_template()**: Tokenizer configurations tested across GGUF parsing, service validation, and API responses
- **exec_py_script()**: Python execution infrastructure enabling complex test data generation for multiple crate testing scenarios
- **Fixed snapshot constant**: `SNAPSHOT = "5007652f7a641fe7170e0bad4f63839419bd9213"` ensures reproducible model testing across all consuming crates

## Downstream Crate Usage Patterns

### Services Crate Integration
The services crate extensively uses objs test_utils for comprehensive service testing with sophisticated coordination patterns:
- **AppServiceStub coordination**: Services use objs domain factories (`Repo::testalias()`, `HubFileBuilder::testalias()`, `AliasBuilder::llama3()`) to create consistent test data across service boundaries
- **Localization testing**: `setup_l10n()` loads objs error messages for service error validation with comprehensive cross-service error propagation testing
- **Database testing**: Environment fixtures (`temp_bodhi_home()`, `temp_hf_home()`) provide configuration for service database integration with isolated SQLite databases
- **Mock service coordination**: Domain object builders enable realistic service behavior simulation with `OfflineHubService`, `SecretServiceStub`, and `TestDbService` patterns
- **Cross-service integration**: Service composition testing uses objs fixtures for authentication flows, model management pipelines, and database transaction coordination
- **Time service testing**: `FrozenTimeService` pattern enables deterministic testing of time-dependent service operations with consistent timestamp generation

### Routes Crate Dependencies  
Route integration tests depend heavily on objs testing infrastructure:
- **API response validation**: `parse<T>()` and response fixtures enable comprehensive endpoint testing
- **Parameter validation testing**: OpenAI parameter builders validate request/response handling
- **Error response testing**: Localized error validation ensures consistent API error responses
- **Authentication testing**: Role and scope builders enable authorization testing across route layers

### CLI Integration Testing
Command-line interface testing leverages objs test utilities:
- **Environment isolation**: `temp_bodhi_home()` provides isolated configuration for CLI command testing
- **Domain object validation**: Repo and Alias builders ensure CLI parameter parsing consistency
- **Output format testing**: `parse_txt()` enables validation of CLI output formatting and error messages

### Auth Middleware Testing
Authentication middleware uses objs testing infrastructure:
- **Role and scope testing**: Domain object builders enable comprehensive authorization testing
- **Error handling validation**: Localized error testing ensures consistent authentication error responses
- **Token validation testing**: Mock services coordinate with objs utilities for complex authentication flows

## Architecture Position

The `test_utils` module serves as:
- **Cross-Crate Testing Foundation**: Shared infrastructure enabling consistent testing patterns across all BodhiApp crates
- **Domain Testing Authority**: Centralizes BodhiApp-specific testing knowledge and patterns
- **Integration Testing Hub**: Enables complex multi-crate integration testing scenarios
- **Test Data Generation Center**: Provides sophisticated data generation supporting all application layers

## Downstream Integration Constraints

### Service Testing Dependencies
- Services must use `setup_l10n()` to test localized error messages properly
- Domain object builders must be used consistently across service tests for data integrity
- Environment fixtures required for any service tests involving file system operations
- Python data generation fixtures needed for services testing GGUF parsing or model metadata

### Route Testing Requirements
- All route tests must use objs response parsing utilities for consistent API validation
- Parameter builders required for comprehensive OpenAI API compatibility testing
- Error response testing must validate both HTTP status codes and localized error messages
- Authentication testing must use objs role and scope builders for authorization validation

### Cross-Crate Testing Coordination
- All crates must coordinate localization resource loading through objs `setup_l10n()`
- Domain object builders must be used consistently to prevent test data inconsistencies
- Environment isolation fixtures required for any tests involving configuration or file system state
- Mock service coordination must respect objs domain object validation rules

### Python Integration Constraints
- Python 3 availability required for any crate testing GGUF parsing or tokenizer functionality
- Test data scripts must be coordinated across crates to prevent duplication
- Python execution infrastructure must be used consistently to prevent test environment pollution