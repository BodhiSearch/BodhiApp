# Display Non-Authz Mode Information on Login Screen

Status: Completed

## User Story
As a User attempting to login to a non-authz Bodhi App  
I want to see clear information that the app is in non-authz mode  
So that I understand why I cannot login to the application

## Background
- Bodhi App can be setup in non-authz mode
- Login functionality is not available in non-authz mode
- Users need clear information about why they cannot login

## Acceptance Criteria

### Login Page Changes
- [x] Check app's authz status when login page loads
- [x] If app is in non-authz mode:
  - [x] Display non-authz status message in the card description
  - [x] Message states: "This app is setup in non-authenticated mode. User login is not available."
  - [x] Disable the login form elements
  - [x] Style the disabled state appropriately to indicate non-interactivity

### Technical Implementation
- Frontend: Update `crates/bodhi/src/app/ui/login/page.tsx`
  - [x] Add status check on page load using `useAppInfo` hook
  - [x] Implement conditional rendering based on authz status
  - [x] Add message to card description
- Backend: [x] Use existing `/app/info` endpoint to get authz status

### UI/UX Requirements
- [x] Info message is clearly visible in the login card description
- [x] Use appropriate styling consistent with app's design system
- [x] Ensure the message is accessible (proper ARIA attributes via Card components)

## Out of Scope
- Converting app to authz mode
- Adding any additional functionality to the login page
- Modifying the app setup process

## Testing Criteria
- [x] Verify correct message display when app is in non-authz mode
- [x] Confirm login form is properly disabled
- [x] Test accessibility of the info message
- [x] Verify proper handling of loading states

## Implementation Notes
- Message is shown in card description for better UI integration
- Used React fragments for line break in message
- Extracted message text and title to variables for better readability
- Added comprehensive test coverage for all states
