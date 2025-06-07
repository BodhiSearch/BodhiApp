# Chat Interface Implementation Prompt

## Context
Bodhi Web Chat needs a clean, responsive chat interface that allows users to interact with an LLM. The chat should be minimal but functional, with streaming responses from the LLM.

## Task
Implement the main chat interface components including:
- Chat window for displaying message history
- Streaming response rendering with appropriate loading states
- Message styling for both user and LLM messages

## Guidelines
- Use Shadcn UI components where appropriate
- Implement streaming response handling
- Create clean, readable React components
- Use TypeScript interfaces for message types
- Create custom hooks for chat state management
- Ensure the chat layout is responsive
- Implement proper keyboard shortcuts (Enter to send, etc.)

## Constraints
- No storage of messages or display of chat history
- Just current chat, and a button to start new chat
- Focus on a clean, minimal UI without unnecessary features
- Ensure the interface works well on both mobile and desktop
- Keep accessibility in mind for all interactive elements 