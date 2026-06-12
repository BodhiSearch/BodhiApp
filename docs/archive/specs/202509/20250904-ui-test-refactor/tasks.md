# UI Test Refactoring - Core Flow Tasks

## Overview
Focus on essential user journeys that validate core application functionality. Each task represents a complete user workflow, not technical implementation details.

## Completed Phases ✅

### Phase 0: API Models Integration ✅ **COMPLETED**
**User Journey:** Manage external API model configurations
- ✅ Create API model with OpenAI credentials
- ✅ Fetch and select available models
- ✅ Test connection to validate configuration
- ✅ Edit existing API model settings
- ✅ Delete API model with confirmation
- ✅ Navigate to chat with selected model
- ✅ Responsive layout testing (mobile/tablet)

### Phase 0.1: Setup Flow ✅ **COMPLETED**
**User Journey:** First-time application setup
- ✅ Welcome and server configuration
- ✅ OAuth authentication setup
- ✅ Model download selection
- ✅ Setup completion and redirect to main app
- ✅ Error handling for authentication failures

### Phase 1: Chat Functionality ✅ **COMPLETED**
**User Journey:** Interactive chat experience
- ✅ Basic Q&A with simple questions
- ✅ Multi-chat management and navigation
- ✅ Chat history persistence
- ✅ Settings configuration (streaming, temperature, max tokens)
- ✅ Model switching during chat
- ✅ Error handling and network failure recovery
- ✅ Special characters and edge cases

## Priority 1: Core Flows to Implement

### Local Model Alias Management 🔴 **CRITICAL**
**User Journey:** Create and manage local model configurations

**Acceptance Criteria:**
- User can create a new local model alias with HuggingFace repo details
- User can edit an existing model alias (repo, filename, parameters)
- User can delete a model alias with confirmation dialog
- User sees validation errors for missing required fields
- User can navigate from model list to chat with the selected model

**Test Scenarios:**
1. Complete lifecycle: Create → Edit → Chat → Delete
2. Validation: Missing fields, duplicate aliases
3. Integration: Model appears in chat model selector

### Authentication & Authorization 🔴 **CRITICAL**
**User Journey:** Secure access to application features

**Acceptance Criteria:**
- User can login via OAuth and access protected pages
- Unauthenticated user is redirected to login when accessing protected pages
- User can logout and is redirected appropriately
- User session persists across page refreshes
- Authentication errors are handled gracefully

**Test Scenarios:**
1. Login → Access protected page → Logout flow
2. Direct access to `/ui/chat` without auth → Redirect to login
3. Session timeout and re-authentication
4. Login with invalid credentials

## Priority 2: Essential Flows

### Model List Management 🟡 **HIGH**
**User Journey:** Browse and manage all models efficiently

**Acceptance Criteria:**
- User can view paginated list of all models (API and local)
- User can sort models by name, type, or date
- User can perform quick actions (edit, delete, chat) from list
- User sees appropriate icons/badges for model types
- List updates after CRUD operations

**Test Scenarios:**
1. Browse models with pagination
2. Sort by different columns
3. Quick actions from dropdown menu
4. Responsive layout on mobile devices

### Settings Management 🟡 **HIGH**
**User Journey:** Configure application behavior

**Acceptance Criteria:**
- User can view all application settings
- User can edit editable settings (e.g., execution variants)
- Settings persist after modification
- User sees validation errors for invalid values
- Non-editable settings are clearly marked

**Test Scenarios:**
1. View settings → Edit execution variant → Verify persistence
2. Attempt to edit non-editable setting
3. Invalid value validation

## Priority 3: Supporting Flows

### Chat Settings Persistence 🟢 **MEDIUM**
**User Journey:** Customize chat behavior persistently

**Acceptance Criteria:**
- User chat settings persist across sessions
- Settings apply to new conversations
- User can reset to defaults
- API token settings work correctly

**Test Scenarios:**
1. Configure settings → New chat → Verify applied
2. Settings survival across logout/login
3. Reset to defaults functionality

## Out of Scope

### Infrastructure Testing (Not Core User Flows)
- OAuth2 token exchange implementation
- Network IP authentication edge cases  
- Canonical host redirect behavior
- Public host configuration variants

### Nice-to-Have Features (Not Essential)
- Advanced chat features (file uploads, voice input, branching)
- Collaborative features and team workspaces
- Performance monitoring and analytics
- Internationalization
- Plugin system architecture
- WebSocket real-time updates
- Progressive Web App features
- Contract testing
- Mutation testing
- Chaos engineering

## Success Metrics

### Test Suite Goals
- **Reliability**: <1% flaky test rate
- **Execution Time**: <5 minutes for core flows, <15 minutes for full suite
- **Coverage**: 100% of critical user paths
- **Maintenance**: Tests remain stable across UI updates

### Migration Progress
- ✅ Phase 0: API Models (100% complete)
- ✅ Phase 0.1: Setup Flow (100% complete)  
- ✅ Phase 1: Chat Core (100% complete)
- 🔴 Authentication (0% - High Priority)
- 🔴 Local Models (0% - High Priority)
- 🟡 Settings (0% - Medium Priority)
- 🟢 Advanced Features (0% - Low Priority)

## Implementation Guidelines

### Approach
1. **Use existing infrastructure** - Page Objects, fixtures, and helpers are already established
2. **Focus on user outcomes** - Test what users do, not how the code works
3. **Keep tests independent** - Each test should be self-contained
4. **Prioritize reliability** - Better to have fewer reliable tests than many flaky ones
5. **Document failures** - When tests fail, the reason should be immediately clear

### Test Organization
```
specs/
├── core/              # Critical user journeys
│   ├── auth/         # Authentication flows
│   ├── models/       # Model management
│   └── settings/     # Settings management
└── integration/       # Cross-feature workflows
```

## Next Steps

1. **Immediate Priority**: Implement Local Model Alias Management tests
2. **Following Priority**: Complete Authentication & Authorization tests
3. **Then**: Model List and Settings Management
4. **Finally**: Review and consolidate any redundant tests

## Notes

- Each test file should contain complete user journeys, not isolated features
- Leverage existing Page Objects and fixtures - don't recreate infrastructure
- Tests should be readable by non-technical stakeholders
- Focus on critical paths that would block user productivity if broken
- Infrastructure concerns (like OAuth implementation details) are tested at the backend level