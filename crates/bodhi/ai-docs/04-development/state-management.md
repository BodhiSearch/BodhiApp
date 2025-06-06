# State Management

This document details the state management patterns and implementation in the Bodhi App.

## State Architecture

### Global State
1. **User Session**
   - Authentication state
   - User preferences
   - Permissions

2. **Navigation**
   - Current route
   - Navigation history
   - Breadcrumb state

3. **Application Settings**
   - Theme preferences
   - UI configuration
   - System settings

### Feature-specific State

1. **Chat State**
   - Message history
   - Active conversations
   - Model settings

2. **Model Management**
   - Model configurations
   - Download status
   - Model aliases

## Implementation Patterns

### Custom Hooks

1. **Authentication Hooks**
   - `useLogoutHandler`: Manages logout flow
   - `useUser`: Handles user information

2. **Chat Hooks**
   - `use-chat-completions`: Manages chat responses
   - `use-chat-db`: Handles chat persistence
   - `use-chat-settings`: Manages chat configuration
   - `use-chat`: Core chat functionality

3. **Utility Hooks**
   - `use-copy-to-clipboard`: Clipboard operations
   - `use-mobile`: Mobile detection
   - `use-navigation`: Route management
   - `use-toast`: Notification system
   - `useLocalStorage`: Local storage management
   - `useQuery`: Data fetching

### State Persistence

1. **Local Storage**
   - User preferences
   - Cache data
   - Session information

2. **Memory Cache**
   - Temporary state
   - Performance optimization
   - Quick access data

## Data Flow Patterns

### Request-Response Flow
```
Action Trigger -> State Update -> UI Update
└── Side Effects
    ├── API Calls
    ├── Local Storage
    └── Cache Updates
```

### State Updates
1. **Synchronous Updates**
   - Direct state mutations
   - Computed values
   - UI state

2. **Asynchronous Updates**
   - API responses
   - Background tasks
   - Streaming data

## Provider Architecture

### Client Providers
- Global state providers
- Theme provider
- Navigation provider
- Toast provider

### Feature Providers
- Chat provider
- Model provider
- Settings provider

## State Management Best Practices

### 1. State Organization
- Clear hierarchy
- Minimal redundancy
- Logical grouping

### 2. Performance Optimization
- Selective updates
- Efficient re-renders
- State normalization

### 3. Error Handling
- Consistent patterns
- Error recovery
- User feedback

## Testing Considerations

### 1. Unit Tests
- Hook testing
- State isolation
- Mock implementations

### 2. Integration Tests
- Provider testing
- State flow testing
- Side effect testing

## Future Improvements

### 1. State Management
- Enhanced caching
- Better persistence
- Optimized updates

### 2. Developer Experience
- Debug tools
- State monitoring
- Performance tracking

### 3. User Experience
- Faster updates
- Better feedback
- Smoother transitions
