# Testing Strategy

This document provides a high-level overview of testing approaches and quality assurance strategy for the Bodhi App across frontend and backend components.

## Required Documentation References

**MUST READ for specific testing implementation:**
- `ai-docs/01-architecture/frontend-testing.md` - Frontend testing patterns, React components, and user interactions
- `ai-docs/01-architecture/backend-testing.md` - Backend testing approaches, database testing, and API integration
- `ai-docs/01-architecture/development-conventions.md` - Testing conventions and file organization

**FOR DETAILED TESTING GUIDE:**
- `ai-docs/01-architecture/TESTING_GUIDE.md` - Complete testing implementation guide

## Overall Testing Philosophy

### Testing Pyramid Strategy
1. **Unit Tests** (70%) - Fast, isolated tests for individual functions and components
2. **Integration Tests** (20%) - Tests for feature workflows and API integration  
3. **End-to-End Tests** (10%) - Complete user journey testing
4. **Accessibility Tests** - WCAG compliance verification across all levels

### Quality Goals
- **Unit Tests**: 80%+ coverage for critical business logic
- **Integration Tests**: All critical user flows covered
- **Accessibility**: 100% WCAG 2.1 AA compliance
- **Performance**: Core Web Vitals targets and database query performance

### Technology-Specific Approaches

#### Frontend Testing Stack
- **Vitest** - Fast unit testing framework with Vite integration
- **Testing Library** - Component testing utilities with accessibility focus
- **MSW (Mock Service Worker)** - API mocking for reliable tests
- **Happy DOM** - Lightweight DOM environment for testing

**→ See [Frontend Testing](frontend-testing.md) for detailed patterns and examples**

#### Backend Testing Stack
- **Rust Test Framework** - Built-in Rust testing with `#[test]`
- **rstest** - Parameterized testing and fixtures
- **mockall** - Mock object generation
- **TestDbService** - Database testing infrastructure

**→ See [Backend Testing](backend-testing.md) for detailed patterns and examples**

## Testing Approach by Component

### Frontend Testing Focus Areas
- **Component Behavior**: User interactions, state changes, and rendering logic
- **API Integration**: Query hooks, mutations, and error handling with MSW mocking
- **User Experience**: Accessibility, keyboard navigation, and screen reader support
- **Form Validation**: Input validation, error display, and submission handling
- **Routing**: Navigation behavior and route-specific component rendering

### Backend Testing Focus Areas  
- **Service Logic**: Business logic, data validation, and error handling
- **Database Operations**: CRUD operations, transactions, and data integrity
- **API Endpoints**: Request/response handling, authentication, and error responses
- **Performance**: Query optimization and concurrent operation handling
- **Integration**: Service interactions and external API communication

## Test Commands and Configuration

### Frontend Test Commands
```bash
cd crates/bodhi

# Run tests once (CI mode)
npm run test

# Run tests with coverage
npm run test -- --coverage
```

### Backend Test Commands
```bash
# Run all tests
cargo test

# Run tests for specific crate
cargo test -p services

# Run integration tests
cargo test -p integration-tests
```

## Quality Assurance Standards

### Coverage Requirements
- **Frontend**: 80%+ coverage for components and hooks
- **Backend**: 80%+ coverage for business logic and services
- **Integration**: All API endpoints tested
- **Accessibility**: 100% WCAG 2.1 AA compliance

### Testing Best Practices
- **Test Isolation**: Each test should be independent and repeatable
- **Clear Naming**: Test names should describe the expected behavior
- **Arrange-Act-Assert**: Structure tests with clear setup, execution, and verification
- **Mock External Dependencies**: Use MSW for API mocking and mockall for service mocking
- **Performance Awareness**: Include performance assertions for critical paths

## Continuous Integration

### Automated Testing Pipeline
- **Pull Request Validation**: All tests must pass before merge
- **Coverage Reporting**: Coverage reports generated for each build
- **Performance Monitoring**: Performance regression detection
- **Accessibility Validation**: Automated accessibility testing in CI

### Test Environment Management
- **Isolated Test Databases**: Each test run uses fresh database state
- **Deterministic Test Data**: Consistent test data generation
- **Environment Parity**: Test environment matches production configuration

## Related Documentation

- **[Frontend Testing](frontend-testing.md)** - Detailed frontend testing patterns, React components, and user interactions
- **[Backend Testing](backend-testing.md)** - Detailed backend testing approaches, database testing, and API integration
- **[Development Conventions](development-conventions.md)** - Testing conventions and file organization
- **[TESTING_GUIDE.md](TESTING_GUIDE.md)** - Complete testing implementation guide

---

*For detailed testing implementation patterns and examples, see the technology-specific testing documents. For complete testing setup and configuration, see [TESTING_GUIDE.md](TESTING_GUIDE.md).*
