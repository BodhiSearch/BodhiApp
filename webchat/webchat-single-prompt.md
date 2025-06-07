# Bodhi Web Chat Application - Complete Implementation Prompt

## Context
We need to build a minimal, responsive web application called Bodhi Web Chat that allows users to chat with OpenAI-compatible LLMs by providing their own API key. This is a frontend-only application with no backend persistence. The app should feature a clean, modern UI with a right sidebar for settings, a main chat area, and support for both mobile and desktop views.

The application implements a BYOK (Bring Your Own Key) model where users provide their OpenAI-compatible API key, which is stored in memory only and used for all LLM interactions. The interface needs to support streaming responses, model selection, basic chat settings, and a clean chat experience.

## Task
Build a complete, functional Bodhi Web Chat application that includes:

1. **Project Setup**
   - Initialize a Vite + React + TypeScript project
   - Configure TailwindCSS and Shadcn UI
   - Establish folder structure following React best practices

2. **UI Layout**
   - Create a responsive layout with:
     - Right sidebar for model selection and settings (collapsible on mobile)
     - Main chat area for messages and input
     - Light/dark theme toggle
   - Implement Shadcn UI components for all interface elements

3. **API Key Handling**
   - Create a modal dialog for API key entry
   - Implement secure, in-memory storage of the API key
   - Add API key validation and error handling

4. **Model Selection**
   - Create a dropdown in the left sidebar for model selection
   - Use the API to retrieve available models (show top 5)
   - Store selected model in memory

5. **Chat Interface**
   - Implement chat message display area
   - Add message input with send button
   - Support for streaming responses with loading states
   - Copy functionality for LLM responses

6. **API Integration**
   - the apis are OpenAI-compatible API, use openai typescript client, have all the network calls in a single file which in turn delegates the call to openai client, this helps switching client in future without much impact on the remaining codebase
   - Implement streaming response handling
   - Add proper error handling for API calls

7. **Theming & Responsiveness**
   - Create light/dark mode toggle
   - Ensure full responsiveness across all device sizes
   - Implement clean, consistent styling

8. **Error Handling**
   - Create toast notifications for errors
   - Implement user-friendly error messages

## Guidelines

### UI & Design
- Create a modern, clean interface using Shadcn UI components
- Implement a 2-panel layout 
- Ensure the sidebar is collapsible on mobile with a hamburger menu
- Use consistent spacing, typography, and colors throughout
- Implement smooth transitions between themes and states

### Code Quality
- Write clean, type-safe TypeScript code
- Create reusable custom hooks for state management
- Implement proper component separation for maintainability
- Use semantic HTML and follow accessibility best practices

### Functionality Flow
1. On first load, open up the sidebar that displays API key and Model Dropdown as mandatory and not provided
2. Once key is provided, populate the model using api call and show model selection as mandatory and not provided
3. After model selection, enable chat functionality, while it is disabled, show a message asking to provide api key and model selection in settings panel
4. Allow changing settings via the sidebar at any time

## Constraints
- This must be a purely frontend application with no backend
- All data (API key, settings, chat history) stored in memory only
- No persistence between page refreshes
- No external services beyond the OpenAI-compatible API
- Keep dependencies minimal and focused on core needs
- Ensure the app is fully responsive and works on mobile devices
- Focus on getting a working end-to-end flow quickly
- Prioritize reliability and user experience over feature complexity

## Technical Specifications
- **Tech Stack**: Vite, React, TypeScript, TailwindCSS, Shadcn UI
- **API**: OpenAI-compatible chat completions API with streaming
- **Styling**: TailwindCSS with Shadcn UI components
- **Layout**: Responsive design with mobile-first approach
- **Browser Support**: Modern browsers only

any questions?