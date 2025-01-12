# Convert Non-Authz App to Authz Mode

## User Story
As a User who has setup their app in non-authz mode  
I want to convert it into authz mode  
So that I can have Authz features like User Management, Login, JWT API Keys and secure my App instance

## Background
- Bodhi App allows initial setup in non-authz mode for simpler deployment
- Non-authz mode has limited features (no user management, login, or JWT API key generation)
- Users may want to later upgrade to authz mode for enhanced security and features

## Acceptance Criteria

### Setup Screen Changes
- [ ] Add a switch labeled "I'll decide later" on the non-authz setup screen
- [ ] Display warning message: "Warning: Any user can later register this app and become admin"
- [ ] Store user's choice of "I'll decide later" in app settings

### Login Page Enhancements
- [ ] Display information about app's current non-authz status
- [ ] Show "Setup Authorization" button if user had selected "I'll decide later"
- [ ] Hide "Setup Authorization" if user had not selected "I'll decide later"

### Authorization Setup Flow
- [ ] Implement backend API to register app with auth server
- [ ] Store received client-id and client-secret in encrypted secrets.yml
- [ ] Update app settings to authz: true
- [ ] First user to login after registration becomes admin
- [ ] Display success message after setup completion

### Security Considerations
- [ ] Ensure client-secret is properly encrypted in secrets.yml
- [ ] Validate first-user-as-admin flow
- [ ] Add appropriate audit logging for the conversion process

## Technical Notes
- Frontend: `crates/bodhi/src/app/ui/setup/page.tsx` and `crates/bodhi/src/app/ui/login`
- Backend: `crates/routes_app/src/routes_setup.rs` and `crates/auth_middleware/src/auth_middleware.rs`
- Auth flow must handle edge cases (network issues, concurrent requests)

## User Flow
1. User visits login page in non-authz app
2. User sees "Setup Authorization" button (if eligible)
3. User initiates setup process
4. Backend registers with auth server
5. User is prompted to login
6. First login user becomes admin
7. App now has full authz features enabled

## Out of Scope
- Reverting from authz to non-authz mode
- Bulk user import during conversion
- Migration of existing data permissions
