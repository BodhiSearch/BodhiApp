# API Key Handling Prompt

## Context
Bodhi Web Chat requires users to provide their own OpenAI-compatible API key to use the application. This key needs to be collected via a modal dialog and stored in memory only (not persisted).

## Task
Implement the API key handling functionality including:
- Modal dialog for API key entry
- In-memory storage of the API key
- Interface to update or change the API key
- Basic validation of API key format
- Secure handling of the key in memory

## Guidelines
- Create a clean, user-friendly modal dialog using Shadcn UI
- Implement a custom hook for managing the API key state
- Add basic validation for API key format
- Include clear instructions for users on where to get an API key
- Ensure the key is only stored in memory and not persisted
- Implement error handling for invalid keys

## Constraints
- Do not store the API key in localStorage, cookies, or any persistent storage
- Key should be cleared when the page is refreshed
- No sending of the key to any backend or third-party services
- Implement proper security measures for handling the key in the client
- API key should not be visible in the UI after entry 