# UX Pilot Prompt for Bodhi Web Chat UI Design

## Design Brief

Create a high-fidelity UI design for "Bodhi Web Chat" - a minimal, responsive web application that allows users to chat with OpenAI-compatible LLMs by providing their own API key. The design should emphasize clean aesthetics, intuitive user experience, and a modern look and feel.

## Screens Required

1. **Main Chat Interface**
   - Desktop view
   - Mobile view (responsive)

## UI Components

### Layout Structure
Design a responsive chat interface with:
- A main central chat area for displaying messages and input
- A right sidebar panel for settings (collapsible on mobile)
- A clean, minimal header with app title and theme toggle

### Essential UI Elements

1. **Chat Area**
   - Message bubbles (user messages and LLM responses)
   - Clear visual distinction between user and LLM messages
   - Typing/streaming animation for LLM responses
   - Input area with text field and send button
   - Copy button for LLM responses
   - Empty state with prompt to start chatting

2. **Settings Sidebar (Right Panel)**
   - Collapsible panel with toggle button
   - Section for API key management (update/view)
   - Model selection dropdown
   - Temperature slider control
   - Theme toggle switch (light/dark)
   - System prompt text area

3. **Error States**
   - Toast notification design for errors
   - Error message styling

## Design Specifications

### Style
- Modern, clean interface with ample white space
- Shadcn UI component styling
- Clear typography hierarchy
- Subtle shadows and elevation for depth
- Rounded corners on containers and inputs

### Color Scheme
- Primary: A soft, calming blue (#3B82F6)
- Secondary accent colors that complement the primary
- Light theme: Clean white background with dark text
- Dark theme: Deep blue/gray background (#1E293B) with light text
- Success, error, and warning states with appropriate colors

### Typography
- Sans-serif font for clean readability
- Clear hierarchy with different weights and sizes
- Good contrast for accessibility

### Mobile Considerations
- Fully responsive layout that adapts to screen size
- Sidebar collapses to hamburger menu on mobile
- Touch-friendly input elements with appropriate sizing
- Bottom-fixed input area on mobile

## UI Flow

1. First-time user sees right sidebar open and API key input and model dropdown box highlighted mandatory and errored
2. After entering API key, user sees the model dropdown populated, and selects one, this enables the chat input, removes error state as they are provided
3. User can begin chatting once model is selected
4. when api key and model are not selected, the message is displayed just below the chat input on providing the mandatory fields
5. Settings can be accessed and modified at any time via the sidebar

## Additional Notes

- Ensure all states are designed: empty, loading, error, populated
- Include subtle micro-interactions and transitions
- Design should accommodate long messages with proper scrolling
- Chat interface should have visual indicators for message status
- Consider accessibility in contrast, text size, and interactive elements

## Example References

Design inspiration similar to modern chat applications like:
- ChatGPT's minimal, clean interface
- Slack's message threading and layout
- Linear's clean aesthetics and dark mode 