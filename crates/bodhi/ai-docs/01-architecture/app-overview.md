# Bodhi App: Architecture Overview

## Application Overview

Bodhi App is a tool designed to democratize AI by allowing users to run Large Language Models (LLMs) locally on their devices. The name "Bodhi" comes from Sanskrit/Pali, meaning deep wisdom or intelligence, reflecting the app's mission to make AI accessible to everyone.

## Core Architecture

### System Components

```
┌─────────────────────────────────────────────────────────┐
│                    Bodhi App                            │
├─────────────────────────────────────────────────────────┤
│  Frontend (React+Vite)                                 │
│  ├── Chat Interface                                     │
│  ├── Model Management                                   │
│  ├── Authentication                                     │
│  └── Configuration                                      │
├─────────────────────────────────────────────────────────┤
│  Backend API                                            │
│  ├── REST API Endpoints                                 │
│  ├── WebSocket Connections                              │
│  ├── Authentication Service                             │
│  └── Model Management                                   │
├─────────────────────────────────────────────────────────┤
│  LLM Inference Engine                                   │
│  ├── GGUF Model Support                                 │
│  ├── GPU Acceleration                                   │
│  ├── Streaming Responses                                │
│  └── Context Management                                 │
├─────────────────────────────────────────────────────────┤
│  Storage Layer                                          │
│  ├── Model Files                                        │
│  ├── Configuration Files                                │
│  ├── Chat History                                       │
│  └── User Data                                          │
└─────────────────────────────────────────────────────────┘
```

## Core Capabilities

### 1. Local LLM Inference
- Run AI models directly on your device
- Support for all major GGUF models
- Real-time streaming responses with inference statistics
- GPU acceleration support for faster inference

### 2. User-Friendly Interface
- Built-in Chat UI with rich markdown support and syntax highlighting
- One-click model downloads and management
- Responsive design for desktop, tablet, and mobile devices

### 3. Model Management
- Model aliases for easy configuration
- File browser and operations
- Download queue with progress tracking
- Direct integration with HuggingFace ecosystem

### 4. Chat Interface
- Chat sessions management
- Message input/display/actions
- Real-time features including message streaming

### 5. Configuration & Control
- Control request parameters (temperature, system prompt, etc.)
- Control LLM context parameters (context window, num parallel requests, etc.)
- Configuration via YAML files, updatable in real-time

### 6. Security Features
- Choose between authenticated or non-authenticated modes
- Role-based access for teams (coming soon)
- API token management

### 7. API Compatibility
- OpenAI and Ollama compatible APIs
- OpenAPI documentation with embedded Swagger-UI

## Feature Architecture

### Authentication & Authorization
```
Features:
├── Login/Logout Flow
│   ├── Login Page
│   ├── OAuth2 Authentication
│   └── Logout Process
├── Authorization Status
│   ├── Current Role Display
│   └── Permission Indicators
├── Role-based UI Elements
│   ├── Admin Features
│   ├── Power User Features
│   └── Basic User Features
├── Token Management
│   ├── Token Creation
│   ├── Token Listing
│   └── Token Updates
└── Setup Configuration
    ├── Auth Mode Selection
    ├── OAuth2 Setup
    └── Role Configuration
```

### Navigation & Layout
```
Features:
├── Header Components
│   ├── Main Menu
│   ├── User Profile
│   └── Quick Actions
├── Responsive Navigation
│   ├── Desktop Menu
│   ├── Tablet Layout
│   └── Mobile Menu
├── Role-based Navigation
│   ├── Admin Routes
│   ├── Power User Routes
│   └── Basic User Routes
└── Navigation Aids
    ├── Breadcrumbs
    ├── Section Headers
    └── Back Navigation
```

### Model Management
```
Features:
├── Model Aliases
│   ├── List/Grid View
│   ├── Create Alias
│   └── Update Alias
├── Model Files
│   ├── File Browser
│   └── File Operations
├── Downloads
│   ├── Download Queue
│   ├── Progress Tracking
│   └── Status Updates
└── Pull Operations
    ├── Repository Pull
    ├── Pull by Alias
    └── Pull Status
```

### Chat Interface
```
Features:
├── Chat Sessions
│   ├── New Chat
│   ├── Chat History
│   └── Session Management
├── Model Integration
│   ├── Model Selection
│   ├── Model Settings
│   └── Context Management
├── Message Interface
│   ├── Message Input
│   ├── Message Display
│   └── Message Actions
└── Real-time Features
    ├── Message Streaming
    ├── Status Updates
    └── Error Handling
```

## Key USPs (Unique Selling Propositions)

### 1. Zero Technical Knowledge Required
- User-friendly interface designed for non-technical users
- Guided setup process with hints and help throughout
- No complex setup, everything packaged in one application

### 2. Privacy & Data Control
- Complete privacy with data staying on your device
- No cloud processing or data sharing
- Secure local inference

### 3. Cost Effectiveness
- Completely FREE - no subscription or one-time cost
- No API costs or usage limits
- Unlimited AI inferences using your own hardware

### 4. Performance & Control
- Direct access to your hardware's capabilities
- Optimal performance without internet latency
- Fine-grained control over model behavior
- Optimized inference for both CPU and GPU systems

### 5. Community-Driven
- Join fellow users in upskilling together
- Support for collaborative learning

### 6. Accessibility
- Bringing AI to underserved communities worldwide
- Making AI accessible to those without technical expertise

## Target Audience

The app is specifically designed for people who:
- Do not have a deep technical IT background
- Are savvy using laptops and mobile devices for day-to-day professional needs
- Want to benefit from AI without complicated setup
- Include college students (e.g., from IITs, NITs, law schools, CA programs, medical schools)

## Hardware Requirements

The app intelligently detects your system capabilities and recommends appropriately sized models:

- **For CPU-only systems**: Optimized models like TinyLlama (1.1B parameters, 0.6GB)
- **For systems with GPU**: Support for larger models like Mistral-7B (7B parameters, 4.1GB)
- **For high-end systems**: Support for advanced models like Mixtral-8x7B (47B parameters, 26GB)

The app provides hardware recommendations based on:
- Available GPU memory
- System RAM
- CPU cores

## Supported Platforms

- **Currently Available**: macOS
- **Coming Soon**: Linux and Windows
- **Built on**: Platform-independent technology stack

## Cross-cutting Features

### Theme System
- Theme switching
- Color modes
- Custom themes

### Notifications
- Toast messages
- Alert dialogs
- Status updates

### Form Handling
- Input validation
- Error states
- Submit handling

### Modal Systems
- Dialog windows
- Confirmation boxes
- Action sheets

## Upcoming Features

According to the documentation, Bodhi App is actively developing:
- Support for more platforms (Linux and Windows coming soon)
- Enhanced model management capabilities
- Additional API integrations
- Advanced conversation features
- Role-based team features

The app aims to democratize AI by making powerful AI tools accessible to everyone, regardless of technical expertise, while keeping data private and eliminating ongoing API costs.

## Links

- **Website**: https://www.getbodhi.app
- **GitHub Repository**: https://github.com/BodhiSearch/BodhiApp 
- **Linktree**: https://linktr.ee/bodhiapp
