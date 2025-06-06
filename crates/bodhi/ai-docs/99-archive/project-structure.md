# Project Structure

The Bodhi App frontend is built using React+Vite and follows a modern, modular architecture. Here's a detailed breakdown of the project structure:

## Root Structure

```
src/
├── components/    # React components organized by feature
├── pages/         # Page components for routing
├── hooks/         # Custom React hooks
├── lib/           # Utility functions and shared logic
├── schemas/       # Data validation schemas
├── styles/        # Global styles and theme definitions
├── tests/         # Test files
└── types/         # TypeScript type definitions
```

## Components Structure

```
components/
├── chat/          # Chat interface components
├── home/          # Home page components
├── login/         # Authentication related components
├── modelfiles/    # Model file management
├── models/        # Model-related components
├── pull/          # Model download components
├── setup/         # Setup and configuration components
├── tokens/        # API tokens components
├── users/         # User management components
├── ui/            # Common UI components (shadcn/ui)
└── [other common components]
```

## Key Features

1. **Authentication System**
   - Login interface
   - Session management

2. **Model Management**
   - Model listing and details
   - Model file handling
   - Pull/sync capabilities

3. **Chat Interface**
   - Interactive chat UI
   - Message handling

4. **Setup and Configuration**
   - System setup components
   - Configuration management

## Directory Purposes

### `/components`
React components organized by feature/page. Each folder contains components specific to that feature area.

### `/pages`
Page components that are used by React Router for routing. Each page component represents a distinct route in the application.

### `/hooks`
Custom React hooks for shared stateful logic and side effects. These hooks abstract common functionality used across components.

### `/lib`
Utility functions, API clients, and other shared logic that isn't specific to React components.

### `/schemas`
Data validation schemas, likely using libraries like Zod or Yup for type-safe data handling.

### `/styles`
Global styling configurations, theme definitions, and shared style utilities.

### `/types`
TypeScript type definitions and interfaces used throughout the application.

## Next Steps

This structure suggests a feature-based organization with clear separation of concerns. For navigation and menu design, we should consider:

1. Grouping related features in the navigation
2. Creating a hierarchy based on user workflows
3. Ensuring easy access to frequently used features
4. Implementing proper access control based on user roles

The next documents will dive deeper into specific aspects of the application architecture.
