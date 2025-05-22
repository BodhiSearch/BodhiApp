# API Integration Prompt

## Context
Bodhi Web Chat needs to connect to OpenAI-compatible LLM APIs using the user-provided API key and selected model. This includes handling streaming responses, error handling, and proper API call management to get the end-to-end flow working quickly.

## Task
Implement the API integration layer including:
- OpenAI-compatible API client setup
- Streaming response handling
- User message submission functionality
- Integration with chat UI components
- Error handling for API calls
- TypeScript types for API requests and responses

## Guidelines
- Create a clean API client abstraction
- Support streaming responses from the API
- Implement proper error handling and user feedback
- Build type-safe API integration with TypeScript
- Add appropriate loading states during API calls
- Ensure seamless integration with the chat interface
- Focus on getting a working end-to-end flow quickly

## Constraints
- All API calls must happen client-side
- No API keys should be sent to any backend other than the service provider (e.g., OpenAI)
- Only support essential OpenAI API features needed for chat
- Ensure proper error handling for network issues, invalid responses, and rate limits
- Focus on reliability and user experience during API interactions
- Keep the implementation simple and maintainable
- Prioritize getting the core messaging functionality working 