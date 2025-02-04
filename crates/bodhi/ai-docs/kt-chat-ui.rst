Chat UI Knowledge Transfer
========================

Overview
--------
The Chat UI is a three-panel interface designed for flexibility and power while maintaining simplicity. It consists of a chat history panel (left), main chat area (center), and settings panel (right).

Core Components
-------------

Panel Structure
~~~~~~~~~~~~~~
- **Left Panel:** Chat history management
- **Center Panel:** Main chat interaction area
- **Right Panel:** Settings and controls
- All panels are independently collapsible

Chat History Panel
----------------

Organization
~~~~~~~~~~~
Chat conversations are automatically grouped by timeframes:

- Today's conversations
- Yesterday's conversations
- Previous 7 days

Storage
~~~~~~~
- Uses browser's local storage
- No server-side storage
- No synchronization across devices
- No export functionality currently
- No limit on history storage

Management
~~~~~~~~~
- Delete individual conversations (permanent action)
- Switch between conversations
- Start new chat from history panel
- Collapsible for space management

Main Chat Area
------------

Input Features
~~~~~~~~~~~~
- Text input box at bottom
- New chat button (+) for fresh conversations
- Enter key to send messages
- Support for multi-line input

Response Rendering
~~~~~~~~~~~~~~~~
- Real-time streaming with visual indicators
- Markdown rendering support
- Code block rendering with:
  * Syntax highlighting
  * Copy functionality
  * Support for all major programming languages
- No keyboard shortcuts currently implemented

Settings Panel
------------

Global Behavior
~~~~~~~~~~~~~
- Settings are global across all chats
- Persist between conversations
- Cannot be changed mid-conversation
- Must be configured before sending messages
- Stored in browser's local storage

Input Types
~~~~~~~~~~
1. **Dropdown Selection**
   - Model/Alias selection (required)
   - Available GGUF models
   - Custom aliases

2. **Slider Controls**
   - Temperature (0-2)
   - Top P (0-1)
   - Presence Penalty (-2 to +2)
   - Frequency Penalty (-2 to +2)
   - Max Tokens

3. **Text Areas**
   - System Prompt
   - API Token (plain text storage)

4. **Special Inputs**
   - Stop Words (tag-based, up to 4)
   - Seed (numeric)

Toggle Behavior
~~~~~~~~~~~~~
- Each optional setting has a toggle
- When disabled, uses backend defaults
- Enable to override with custom values
- Tooltips explain each setting's purpose

Security Considerations
---------------------

Local Storage
~~~~~~~~~~~~
- All settings stored in browser's local storage
- API tokens stored in plain text
- No encryption of stored data
- Cleared with browser data

Best Practices
~~~~~~~~~~~~
- Clear API tokens after use
- Toggle off sensitive settings
- Regular cleanup of old conversations
- Be mindful of browser storage limits

Technical Implementation
----------------------

State Management
~~~~~~~~~~~~~~
- Settings persist globally
- Chat history managed per browser
- Real-time updates for streaming
- Independent panel collapse states

Response Handling
~~~~~~~~~~~~~~~
- Streaming support with progress indicators
- Markdown parsing and rendering
- Code syntax highlighting
- Copy functionality for code blocks

Limitations
----------

Current Constraints
~~~~~~~~~~~~~~~~~
- No chat export functionality
- No cross-device synchronization
- No mid-conversation setting changes
- No keyboard shortcuts
- No chat-specific settings
- No history size limits implemented

Future Considerations
~~~~~~~~~~~~~~~~~~~
- Chat export functionality
- Enhanced security for stored data
- Chat-specific settings
- Keyboard shortcuts
- Cross-device synchronization

See Also
--------
- Model Configuration Guide
- Settings Parameter Documentation
- Chat API Documentation 