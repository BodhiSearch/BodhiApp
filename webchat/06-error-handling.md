# Error Handling Prompt

## Context
Bodhi Web Chat needs robust error handling to provide a good user experience when issues occur with API keys, network connections, or LLM responses.

## Task
Implement comprehensive error handling including:
- Error handling for invalid API keys
- Network error handling for API calls
- UI components for error display
- User-friendly error messages
- Recovery paths from error states

## Guidelines
- Create reusable error components using Shadcn UI
- Implement a toast notification system for errors
- Provide clear error messages that help users understand issues
- Add recovery options where possible
- Implement proper error boundaries in React components
- Log errors to console for debugging (but not sensitive information)

## Constraints
- Keep error messages user-friendly and non-technical when possible
- Don't expose sensitive information in error messages
- Ensure errors don't break the app's usability
- Focus on graceful degradation during errors
- Don't overwhelm users with technical details
- Make error recovery intuitive 