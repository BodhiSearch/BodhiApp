# Chat Settings Prompt

## Context
Now that the basic end-to-end flow is working with model selection and API integration, Bodhi Web Chat needs additional chat settings for fine-tuning the LLM behavior, including system prompt and temperature adjustments.

## Task
Implement the remaining chat settings including:
- System prompt input field
- Temperature slider control
- Settings panel/modal for these configurations
- State management for these settings
- Integration with the existing API client

## Guidelines
- Use Shadcn UI components (Slider, Textarea, etc.)
- Create clean, intuitive settings interface
- Implement custom hooks for settings state management
- Add reasonable default values for all settings
- Ensure the UI clearly indicates current settings
- Make settings accessible from the main chat interface
- Design settings to be adjustable without disrupting ongoing conversations

## Constraints
- Settings should be stored in memory only
- Keep the settings UI minimal and focused
- Don't overload with too many configuration options
- Settings should reset on page refresh
- Include only essential parameters (system prompt, temperature)
- Focus on usability over complexity
- Ensure these additional settings integrate well with the existing model selection and API integration 