# Setup Wizard: Resource Admin Login

## User Story

As a user who chose authenticated mode,
I want to login and become the app admin,
So that I can proceed with setup and manage the app.

## Background

- User has chosen authenticated mode in previous step
- First successful login becomes the app admin
- Admin status is required to proceed with setup
- Login state persists across sessions

## Acceptance Criteria

### Content Requirements

- [x] Clear explanation of admin assignment
- [x] Brief description of admin access
- [x] Login purpose explanation
- [x] Progress indicator (step 3/5)
- [x] Error messages for login failures

### UI/UX Requirements

- [x] Single-page login flow
- [x] Social login options
- [x] Error state handling
- [x] Loading states
- [x] Retry options
- [x] Progress tracking

### Technical Implementation

- [x] OAuth login integration
- [x] Admin role assignment
- [x] Session handling
- [x] State persistence
- [x] Error recovery
- [x] Navigation control

### Navigation Logic

- [x] Redirect to login provider
- [x] Handle login callbacks
- [x] Prevent skipping this step
- [x] Return to same page on refresh/reload

## Content Structure

### Layout

```
Desktop Layout (>768px):
┌─────────────────────────────┐
│     Setup Progress (3/5)    │
├─────────────────────────────┤
│        Admin Setup          │
├─────────────────────────────┤
│     Setup Information       │
├─────────────────────────────┤
│       Login Button          │
├─────────────────────────────┤
│      Error Messages         │
└─────────────────────────────┘

Mobile Layout (<768px):
┌────────────────────┐
│  Progress (3/5)    │
├────────────────────┤
│   Admin Setup      │
├────────────────────┤
│     Info Text      │
├────────────────────┤
│   Login Button     │
├────────────────────┤
│  Error Messages    │
└────────────────────┘
```

### Content Sections

#### Header

```
Set Up Admin Access
Complete login to continue setup
```

#### Information Text

```
You've chosen to run Bodhi App in authenticated mode.
The first account to log in will become the admin with
unrestricted access to manage the app.
```

#### Login Section

```
[Login with Social Providers]

Note: Admin access will be granted to the email
you use to log in.
```

#### Error Messages

```
Network Error:
"Please check your internet connection and try again"

Login Failed:
"Login attempt failed. Please try again"

Admin Assignment Failed:
"Unable to set admin role. Retrying..."
```

#### Success State

```
"Login successful! Setting up admin access..."
```

## Technical Details

### Component Structure

```typescript
interface AdminLoginProps {
  onLoginSuccess: (email: string) => void;
  onAdminAssigned: () => void;
  isLoading: boolean;
}
```

### State Management

```typescript
interface AdminSetupState {
  loginStatus: 'pending' | 'success' | 'failed';
  adminStatus: 'pending' | 'success' | 'failed';
  error?: string;
}
```

## Testing Criteria

### Functional Tests

- OAuth flow completion
- Admin role assignment
- Error handling
- State persistence
- Navigation control

### Visual Tests

- Layout responsiveness
- Loading states
- Error displays
- Success transitions

### Accessibility Tests

- Keyboard navigation
- Screen reader support
- Focus management
- Error announcements

## Out of Scope

- Detailed admin capabilities
- Role transfer features
- Security recommendations
- Password reset flow
- User management features

## Dependencies

- OAuth provider integration
- Admin role API
- Session management
- State persistence system
