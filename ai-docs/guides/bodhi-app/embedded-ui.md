# BodhiApp Embedded UI Guide

BodhiApp includes a comprehensive web-based user interface built with React that provides an intuitive way to interact with local LLMs, manage models, configure settings, and monitor system status. This guide covers all aspects of the embedded UI, from the initial setup process to advanced features.

## UI Architecture Overview

The BodhiApp UI is a **Next.js 14 React application** that runs embedded within the desktop app and is accessible via web browser at `http://localhost:1135`. Key characteristics:

- **Client-Side SPA**: Complete single-page application with client-side routing
- **Modern UI Components**: Built with TailwindCSS and Shadcn UI components
- **Responsive Design**: Fully adaptive layout for desktop and mobile devices
- **Real-Time Updates**: Live data synchronization with the backend
- **Integrated Documentation**: Built-in access to guides and API documentation

## Initial Setup Process

The initial BodhiApp setup process is covered in detail in the [Getting Started](getting-started.md) guide. Once setup is complete, you'll access the main application interface described below.

## Main Application Interface

After setup completion, you'll access the main BodhiApp interface with the following structure:

### Navigation Header
**Location**: Top of every page
**Components**:
- **Bodhi Logo**: Brand identification and visual consistency
- **Navigation Dropdown**: Access to all major sections
- **Breadcrumb Navigation**: Shows current location within the app
- **Theme Toggle**: Switch between light, dark, and system themes
- **User Menu**: Account information and logout options
- **GitHub Link**: Direct access to project repository

### Primary Navigation Structure

The app is organized into four main sections accessible via the navigation dropdown:

#### 1. Chat Interface (`/ui/chat/`)
**Purpose**: Interactive AI chat with comprehensive controls

#### 2. Models Management
**Sub-sections**:
- **Model Aliases** (`/ui/models/`): Configure and manage model aliases
- **Model Files** (`/ui/modelfiles/`): Browse and manage local model files
- **Model Downloads** (`/ui/pull/`): Download new models from HuggingFace

#### 3. Settings & Administration
**Sub-sections**:
- **App Settings** (`/ui/settings/`): System configuration and preferences
- **API Tokens** (`/ui/tokens/`): Manage API access tokens
- **Manage Users** (`/ui/users/`): User administration and access control

#### 4. Documentation
**Sub-sections**:
- **App Guide** (`/docs/`): User guides and documentation
- **OpenAPI Docs** (`/swagger-ui`): Interactive API documentation

## Chat Interface (Detailed)

The chat interface is the primary interaction point for users and includes sophisticated controls and features:

### Layout Structure

**Three-Panel Layout**:
1. **Left Sidebar**: Chat history and conversation management
2. **Center Panel**: Main chat interface
3. **Right Sidebar**: Settings and model parameters (collapsible)

### Left Sidebar: Chat History
**Components**:
- **New Chat Button**: Start fresh conversations
- **Chat History List**: Organized list of previous conversations
- **Search/Filter**: Find specific conversations
- **Conversation Management**: Rename, delete, or organize chats

**Features**:
- **Auto-Save**: Conversations automatically saved locally
- **Persistence**: Chat history maintained across app restarts
- **Organization**: Chronological organization with timestamps

### Center Panel: Chat Interface
**Message Area**:
- **Message History**: Scrollable conversation history
- **Markdown Rendering**: Rich text formatting for AI responses
- **Code Block Support**: Syntax highlighting and copy functionality
- **Streaming Responses**: Real-time response generation display

**Input Area**:
- **Message Input**: Multi-line text input with auto-resize
- **Send Button**: Submit messages to AI
- **Keyboard Shortcuts**: Enter to send, Shift+Enter for new line
- **Character Counter**: Optional message length indicator

### Right Sidebar: Chat Settings & Parameters
**Model Selection**:
- **Current Model Display**: Shows active model alias
- **Model Switching**: Dropdown to change models mid-conversation
- **Model Information**: Display model details and capabilities

**Chat Parameters** (Collapsible Sections):

#### Request Parameters
- **Temperature** (0.0 - 2.0): Controls response randomness
  - Lower values: More focused and deterministic
  - Higher values: More creative and varied
- **Max Tokens**: Maximum response length
- **Top P** (0.0 - 1.0): Nucleus sampling parameter
- **Frequency Penalty**: Reduce repetition of tokens
- **Presence Penalty**: Encourage topic diversity
- **Stop Sequences**: Custom stopping words/phrases

#### Context Parameters
- **Context Size (n_ctx)**: Maximum context window size
- **Thread Count (n_threads)**: CPU threads for processing
- **Prediction Length (n_predict)**: Maximum tokens to predict
- **Keep Tokens (n_keep)**: Tokens to retain in context
- **Parallel Processing**: Enable parallel inference

#### System Prompt Configuration
- **System Message**: Custom instructions for AI behavior
- **Prompt Templates**: Pre-defined system prompts
- **Custom Templates**: Create and save custom prompts

### Chat Features

**Conversation Management**:
- **Message Editing**: Edit previous messages and regenerate responses
- **Response Regeneration**: Generate alternative responses
- **Message Copying**: Copy individual messages or entire conversations
- **Export Options**: Export conversations in various formats

**Real-Time Features**:
- **Streaming Responses**: See AI responses as they're generated
- **Typing Indicators**: Visual feedback during response generation
- **Progress Indicators**: Show processing status and estimated completion
- **Error Handling**: Graceful handling of connection issues or model errors

## Model Management Pages

### Model Aliases (`/ui/models/`)
**Purpose**: Create and manage semantic model references with custom parameters

**Interface Elements**:
- **Aliases Table**: List of all configured model aliases
- **Create New Alias**: Button to add new model configurations
- **Edit Controls**: Modify existing alias settings
- **Delete Options**: Remove unused aliases

**Alias Configuration**:
- **Alias Name**: Semantic identifier (e.g., "creative-writer", "code-assistant")
- **Model Selection**: Choose from available local models
- **Default Parameters**: Set request and context parameters
- **Description**: Optional notes about alias purpose

### Model Files (`/ui/modelfiles/`)
**Purpose**: Browse and manage local GGUF model files

**Interface Elements**:
- **Files Table**: List of all local model files
- **File Information**: Size, format, and compatibility details
- **Usage Status**: Which aliases use each model file
- **Storage Management**: Disk usage and cleanup options

**File Details**:
- **Repository Information**: Source HuggingFace repository
- **File Size**: Storage space requirements
- **Model Parameters**: Architecture and quantization details
- **Compatibility**: System compatibility indicators

### Model Downloads (`/ui/pull/`)
**Purpose**: Download new models from HuggingFace repositories

**Download Interface**:
- **Repository Input**: HuggingFace repository name (e.g., "microsoft/DialoGPT-medium")
- **Filename Input**: Specific GGUF file to download
- **Download Button**: Initiate background download
- **Progress Tracking**: Real-time download status

**Download Management**:
- **Active Downloads**: Currently downloading models with progress bars
- **Download History**: Previous download attempts and status
- **Error Handling**: Failed download recovery and retry options
- **Queue Management**: Multiple download coordination

## Settings & Administration

### App Settings (`/ui/settings/`)
**Purpose**: System-wide configuration and preferences

**Settings Categories**:

#### General Settings
- **Server Configuration**: Port, host, and network settings
- **Performance Settings**: CPU threads, memory allocation
- **Logging Level**: Debug, info, warning, error levels
- **Auto-Update**: Automatic model and app updates

#### Model Settings
- **Default Model**: System-wide default model selection
- **Model Cache**: HuggingFace cache directory settings
- **Download Preferences**: Automatic vs manual model downloads

#### Interface Settings
- **Theme Preferences**: Light, dark, or system theme
- **Language Selection**: UI localization options
- **Accessibility**: Screen reader and keyboard navigation settings

### API Tokens (`/ui/tokens/`)
**Purpose**: Manage API access tokens for programmatic access

**Token Management Interface**:
- **Tokens Table**: List of all created API tokens
- **Create Token**: Generate new API tokens with specific scopes
- **Token Details**: Name, creation date, last used, status
- **Revoke Options**: Disable or delete unused tokens

**Token Configuration**:
- **Token Name**: Descriptive identifier for the token
- **Scope Selection**: Permission levels (user, power_user, manager, admin)
- **Expiration**: Optional token expiration dates
- **Usage Tracking**: Monitor token usage and activity

### User Management (`/ui/users/`)
**Purpose**: Manage users and access control (Admin only)

**User Administration**:
- **Users Table**: List of all registered users
- **Role Assignment**: Modify user roles and permissions
- **Access Control**: Enable/disable user accounts
- **Activity Monitoring**: Track user activity and API usage

## Documentation Integration

### Built-in Documentation (`/docs/`)
**Purpose**: Access comprehensive user guides and documentation

**Documentation Structure**:
- **Getting Started**: Installation and setup guides
- **Feature Guides**: Detailed feature explanations
- **Troubleshooting**: Common issues and solutions
- **FAQ**: Frequently asked questions

### OpenAPI Documentation (`/swagger-ui`)
**Purpose**: Interactive API documentation and testing

**Swagger UI Features**:
- **Endpoint Explorer**: Browse all available API endpoints
- **Interactive Testing**: Test API calls directly from the browser
- **Schema Documentation**: Request/response format details
- **Authentication Testing**: Test with your API tokens

## Advanced UI Features

### Responsive Design
- **Desktop Optimization**: Full-featured interface for desktop use
- **Mobile Adaptation**: Touch-friendly mobile interface
- **Tablet Support**: Optimized for tablet screen sizes
- **Keyboard Navigation**: Full keyboard accessibility

### Theme System
- **Light Theme**: Clean, bright interface for daytime use
- **Dark Theme**: Eye-friendly dark interface for extended use
- **System Theme**: Automatic switching based on OS preferences
- **High Contrast**: Accessibility-focused high contrast options

### Real-Time Updates
- **Live Data Sync**: Automatic synchronization with backend
- **Status Indicators**: Real-time system status updates
- **Progress Tracking**: Live updates for long-running operations
- **Error Notifications**: Immediate feedback for issues

### Keyboard Shortcuts
- **Chat Interface**: Enter to send, Shift+Enter for new line
- **Navigation**: Keyboard shortcuts for main sections
- **Settings**: Quick access to common settings
- **Accessibility**: Screen reader and keyboard navigation support

## Troubleshooting UI Issues

### Common Interface Problems

**UI Not Loading**:
- Check if BodhiApp is running (system tray icon)
- Verify browser can access `http://localhost:1135`
- Clear browser cache and cookies
- Try incognito/private browsing mode

**Authentication Issues**:
- Ensure stable internet connection to `https://id.getbodhi.app`
- Clear browser storage and retry login
- Check if popup blockers are preventing OAuth redirect

**Chat Not Working**:
- Verify at least one model is downloaded and available
- Check model alias configuration
- Review API token permissions if using programmatic access
- Monitor browser console for JavaScript errors

**Performance Issues**:
- Close unused browser tabs to free memory
- Check system resources (CPU, memory usage)
- Reduce model context size if responses are slow
- Monitor network connectivity for model downloads

### Browser Compatibility
- **Recommended**: Chrome 90+, Firefox 88+, Safari 14+, Edge 90+
- **Features**: Modern JavaScript, WebSockets, local storage support
- **Mobile**: iOS Safari 14+, Android Chrome 90+

## Next Steps

Now that you understand the embedded UI:

1. **[Learn Authentication](authentication.md)** - Understand API tokens and permissions
2. **[Explore OpenAI APIs](openai-api.md)** - Use programmatic access
3. **[Master Model Management](model-management.md)** - Advanced model workflows
4. **[Handle Errors](error-handling.md)** - Troubleshoot issues effectively

---

*The embedded UI provides a complete interface for BodhiApp management. The next sections focus on programmatic API access for developers.* 