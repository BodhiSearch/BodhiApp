# Bodhi App AI-Docs Summary

This document provides an overview of the contents in the ai-docs folder, categorizing and tagging each file.

## Marketing Materials

### whatsapp-intro.md
**Summary:** An introduction message from Amir, the founder of Bodhi App, designed for WhatsApp groups. Describes his background, the app's purpose (running LLMs locally), target audience (students, legal/tech professionals), and platform availability.

**Tags:** #introduction, #marketing, #community-outreach, #app-overview, #call-to-action

### marketing/nus-singapore-students.md
**Summary:** Marketing outreach message specifically targeting students at the National University of Singapore. Highlights Bodhi App's features relevant to academic use cases, emphasizes privacy benefits, zero cost, and invites students to join the community.

**Tags:** #targeted-marketing, #student-outreach, #academic-use-cases, #singapore, #university-students

### cred-conference.md
**Summary:** Technical presentation document describing the challenges and solutions in developing Bodhi App. Focuses on concurrent processing, cross-platform architecture, API compatibility layers, and ongoing development challenges for a presentation at a CRED conference.

**Tags:** #conference, #technical-presentation, #engineering-challenges, #cross-platform, #resource-management

### conference-followup.md
**Summary:** Follow-up communication to conference attendees with links to presentation slides, WhatsApp group, and requests for beta testers. Highlights upcoming features (Windows/Linux support, GPU acceleration) and includes social media content templates about the Bodhi App demo.

**Tags:** #conference-followup, #beta-testing, #community-building, #social-media, #platform-expansion

### product-hunt.txt
**Summary:** Product Hunt launch announcement text that introduces Bodhi App's core features and benefits. Highlights the app's focus on privacy, cost-effectiveness, and user-friendliness, with sections on technical highlights, power features, security, and getting started instructions.

**Tags:** #product-hunt, #launch-announcement, #feature-summary, #value-proposition, #marketing-copy

## Technical Documentation

### app-authentication.md
**Summary:** Comprehensive technical documentation on authentication mechanisms in Bodhi App. Covers security schemes including HTTP Authentication (Basic, Bearer, Digest), API Key Authentication, OAuth2 flows, OpenID Connect, and mTLS with implementation patterns.

**Tags:** #authentication, #security, #oauth2, #api-security, #implementation-guide, #bearer-tokens

### app-status.md
**Summary:** Technical guide explaining the Bodhi App state machine and status tracking. Details different states (Setup, Resource Admin, Ready), authentication modes (Authenticated vs. Non-Authenticated), state transitions, and API response examples.

**Tags:** #app-status, #state-machine, #setup-process, #authentication-modes, #api-responses

### kt-llm-resource-server.md
**Summary:** Knowledge transfer document outlining Bodhi App's vision as a central LLM Resource Server. Covers resource server architecture, security model, resource management, client app ecosystem integration methods, authentication flows, and technical implementation details.

**Tags:** #resource-server, #architecture, #security-model, #client-integration, #oauth2, #api-compatibility

### kt-o3-llm-resource-server.md
**Summary:** Detailed knowledge transfer document on Bodhi App's architecture as an LLM Resource Server. Contains sections on authentication/authorization layer, API endpoints, token management, and the overall vision of Bodhi as a secure broker between client applications and local AI resources.

**Tags:** #resource-server, #architecture, #oauth2, #api-gateway, #token-management, #security

### kt-chat-ui.rst
**Summary:** Technical documentation in RST format describing the Chat UI's three-panel interface. Details the chat history panel, main chat area, and settings panel, including features like local storage, markdown rendering, syntax highlighting, and parameter controls.

**Tags:** #chat-ui, #interface-design, #panel-structure, #user-experience, #settings-management

### kt-context-params.rst
**Summary:** RST-formatted technical documentation explaining context parameters in Bodhi that control model behavior. Covers n_ctx (context window), n_seed, n_threads, n_parallel, n_predict, and n_keep parameters with their impact on performance and best practices for optimization.

**Tags:** #context-parameters, #performance-optimization, #resource-management, #model-configuration, #technical-reference

### kt-request-params.md
**Summary:** Technical guide to understanding aliases and request parameters in Bodhi. Explains built-in vs. custom aliases and details configuration parameters including system prompt, temperature, top P, penalties, stop words, and other generation controls.

**Tags:** #request-parameters, #model-aliases, #configuration, #generation-parameters, #technical-reference

### app-knowledgebase.md
**Summary:** Comprehensive overview of Bodhi App's architecture and systems, using diagrams to explain components. Covers authentication setup modes, application states, token system, and model management with storage and download processes.

**Tags:** #architecture, #knowledge-base, #authentication, #model-management, #system-states, #diagrams

### app-setup.md
**Summary:** Technical guide explaining the application setup flow, with diagrams illustrating the process of initializing Bodhi App in authenticated or non-authenticated mode. Includes API endpoint documentation, setup process details, and state management explanations.

**Tags:** #setup-process, #authentication-modes, #api-endpoints, #flow-diagrams, #state-management

### utoipa-docs.md
**Summary:** Comprehensive guide to integrating Utoipa for OpenAPI documentation in Bodhi App. Covers installation, schema generation, path documentation, authentication integration, UI setup, testing, and best practices with detailed code examples and troubleshooting tips.

**Tags:** #api-documentation, #openapi, #utoipa, #swagger-ui, #integration-guide, #code-examples

### app-navigation.md
**Summary:** Design document for application navigation structures, detailing information architecture, feature organization, and UI implementation code. Includes visual diagrams showing navigation layouts for desktop and mobile, with corresponding component code examples.

**Tags:** #navigation-design, #information-architecture, #ui-implementation, #responsive-design, #layout-patterns

### state-management.md
**Summary:** Detailed documentation on state management patterns in Bodhi App. Covers state architecture (global and feature-specific), implementation through custom hooks, persistence strategies, data flow patterns, provider architecture, and best practices for performance optimization.

**Tags:** #state-management, #custom-hooks, #data-flow, #state-persistence, #best-practices, #performance-optimization

### application-flow.md
**Summary:** High-level overview of application flow and user journeys in Bodhi App. Outlines core user journeys, component interactions, key features, technical flow, navigation structure, state management approaches, optimization points, security measures, and future enhancements.

**Tags:** #application-flow, #user-journeys, #component-interactions, #technical-flow, #navigation-structure, #security-measures

### backend-integration.md
**Summary:** Technical documentation detailing how the frontend integrates with backend services through hooks and utilities. Covers chat integration, data management, authentication, state management, API integration patterns, data flow, error handling, performance considerations, and security.

**Tags:** #backend-integration, #api-integration, #data-flow, #error-handling, #performance, #security, #real-time-features

## App Documentation

### app-extract.md
**Summary:** Comprehensive overview of Bodhi App's capabilities and unique selling propositions. Details core features (local inference, UI, model management), USPs (zero technical knowledge required, privacy, cost-effectiveness), target audience, and hardware requirements.

**Tags:** #app-overview, #capabilities, #unique-selling-propositions, #hardware-requirements, #target-audience

### README.md
**Summary:** Main documentation index for Bodhi App frontend, providing links to various documentation files covering project structure, components, application flow, UI/UX analysis, backend integration, state management, and authentication/authorization.

**Tags:** #documentation-index, #frontend, #table-of-contents, #information-architecture

### app-frontend.rst
**Summary:** Detailed documentation of Bodhi's frontend architecture in RST format. Covers core technologies (Next.js, React, TypeScript), UI components (Tailwind, Shadcn), data management, testing tools, and project structure with coding conventions and file organization patterns.

**Tags:** #frontend-architecture, #project-structure, #technologies, #coding-conventions, #component-organization

### app-ui-tailwind.rst
**Summary:** Technical documentation in RST format focusing on UI component implementation and styling conventions. Details Tailwind CSS usage, Shadcn/UI components, theme configuration, styling conventions, and component architecture with build optimization strategies.

**Tags:** #tailwind-css, #ui-components, #theming, #styling-conventions, #responsive-design

### project-conventions.md
**Summary:** Comprehensive guide to coding conventions and architectural patterns used in the Bodhi App project. Covers project structure, backend conventions for database and service layers, testing patterns, frontend components, API endpoints, error handling, and documentation standards.

**Tags:** #coding-conventions, #project-structure, #architectural-patterns, #testing-patterns, #documentation-standards

### components.md
**Summary:** Comprehensive overview of reusable components in the Bodhi App frontend. Categorizes components into core components (forms, data display, navigation), feature-specific components, application infrastructure, and details component architecture patterns, best practices, and integration points.

**Tags:** #components, #reusable-components, #component-architecture, #design-patterns, #accessibility, #best-practices

### project-structure.md
**Summary:** Detailed breakdown of the Bodhi App frontend project structure built with Next.js. Documents root structure, application organization, key features, directory purposes, and provides recommendations for navigation and menu design based on the existing architecture.

**Tags:** #project-structure, #directory-organization, #architecture, #next-js, #feature-organization, #navigation-recommendations

### ui-ux-analysis.md
**Summary:** Analysis of the application's UI/UX implementation, covering navigation structure, UI components, information architecture, recommended navigation reorganization, UX improvement suggestions, technical implementation details, and next steps for enhancing user experience.

**Tags:** #ui-ux, #navigation-structure, #information-architecture, #design-system, #accessibility, #responsive-design, #improvement-recommendations

## Design Documentation

### app-ui-guidelines.rst
**Summary:** Detailed UI/UX guidelines document in RST format for Bodhi App. Covers color system with semantic tokens, theme implementation through Tailwind, component architecture, layout structure, and theming system with code examples.

**Tags:** #ui-guidelines, #design-system, #tailwind, #component-architecture, #theming, #color-system

### app-ui-models.rst
**Summary:** Technical design document in RST format for the Model Alias page. Covers user context, usage patterns, key tasks, information architecture, parameter details (generation control, performance & resources), and parameter constraints for AI model configuration.

**Tags:** #model-alias, #ui-design, #information-architecture, #parameters, #user-tasks, #configuration-management

### app-ui-modelfile.rst
**Summary:** Design document for the ModelFiles page that serves as inventory management for AI model files. Details user context, usage patterns, data hierarchy, display components, and interface needs for managing storage and accessing model information.

**Tags:** #ui-design, #model-management, #storage-management, #information-architecture, #inventory-management

### nvim-md-sample.md
**Summary:** A markdown sample file demonstrating various markdown formatting features including code blocks, tables, headings, task lists, bullet points, and callouts. Likely used for testing or as a reference for markdown rendering in the application.

**Tags:** #markdown, #sample, #formatting, #reference, #testing

## Feature Documentation

### app-features-overview.md
**Summary:** High-level overview of Bodhi App features organized in a tree structure. Covers Authentication & Authorization UI, Navigation & Layout, Model Management, Chat Interface, and System Configuration features for UI integration testing.

**Tags:** #feature-overview, #ui-testing, #navigation, #model-management, #chat-interface, #system-configuration

### feature-authz.md
**Summary:** Technical documentation covering authentication and authorization in Bodhi App. Details setup options (No Authentication Mode vs. OAuth2 Resource Mode), authentication flow, implementation details, security features, and configuration settings with visual flow diagrams.

**Tags:** #authentication, #authorization, #oauth2, #security-features, #role-based-access, #implementation-details

### feature-alias.md
**Summary:** Comprehensive documentation on Model and Alias Management in Bodhi App. Describes core components of model aliases, features for alias management, model files handling, model download functionality, and detailed implementation with TypeScript interfaces.

**Tags:** #model-management, #alias-configuration, #implementation-details, #data-structures, #feature-description, #technical-documentation

### feature-chat.md
**Summary:** Detailed documentation of the Chat Feature in Bodhi App. Covers interface layout structure, UI components, features for chat management, message handling, model settings, and technical implementation details including state management hooks and data flow.

**Tags:** #chat-interface, #ui-components, #message-handling, #state-management, #technical-implementation, #feature-description

### pending-user-mgmt.md
**Summary:** Design document for a planned User Management System. Outlines core features including access request system, role-based access control hierarchy, API token system architecture, and API endpoints with technical specifications for future implementation.

**Tags:** #user-management, #access-control, #role-based-access, #token-system, #future-feature, #design-specification

### pending-remote-models.md
**Summary:** Design specification for planned Remote Models Integration feature. Details provider integration with OpenAI, Anthropic, and Google; request/response translation; authentication flow; user interface designs; and implementation phases for future development.

**Tags:** #remote-models, #provider-integration, #request-translation, #authentication, #future-feature, #implementation-plan

### home-ideas.md
**Summary:** Design document exploring homepage layout and content ideas for Bodhi App. Details proposed sections including hero, feature highlights, learning hub, news updates, and resource library with visual layout suggestions and interactive element recommendations.

**Tags:** #homepage-design, #layout-structure, #content-organization, #user-engagement, #design-ideas, #interactive-elements

## Marketing Strategy

### kt-product-hunt.md
**Summary:** Comprehensive knowledge base for Bodhi App's Product Hunt launch. Contains product features summary (core features like local inference, UI, model management), official Product Hunt listing draft, and social media message templates for platforms like LinkedIn.

**Tags:** #product-hunt, #launch-strategy, #marketing-copy, #feature-summary, #social-media

## Development Stories

### story-20250112-api-tokens.md
**Summary:** Development story document detailing the API Token Management feature implementation. Includes user story, requirements for token characteristics, security requirements, UI requirements, database schema, and implementation tasks with status tracking.

**Tags:** #api-tokens, #development-story, #feature-implementation, #oauth2, #database-schema, #ui-requirements

### story-20250116-api-authorization.md
**Summary:** Development story document for the API Authorization feature. Details role hierarchy (admin, manager, power_user, user), token scope hierarchy, token claim extraction, authorization flow, security requirements, and implementation tasks with status.

**Tags:** #authorization, #role-based-access, #token-scope, #security, #development-story, #implementation-tasks

### story-20250116-api-authorization-tests.md
**Summary:** Test specification document for API authorization features. Written in Gherkin format, it describes test scenarios for public API access, API token authorization, navigation authorization, and feature access control with detailed test setup and expected outcomes.

**Tags:** #testing, #authorization, #gherkin, #test-scenarios, #api-access, #navigation-authorization

### story-20250119-api-docs.md
**Summary:** Development story for API documentation feature, focusing on providing comprehensive OpenAPI documentation with an interactive UI. Details requirements for Utoipa integration, Swagger UI implementation, schema documentation, and endpoint documentation with implementation tasks.

**Tags:** #api-documentation, #openapi, #swagger-ui, #development-story, #utoipa, #implementation-tasks

### story-20250112-user-roles.md
**Summary:** Development story for adding user roles to the User Info API. Documents backend and frontend changes needed to extract and display role information from JWT tokens, with code examples for Rust and TypeScript implementations, test scenarios, and technical implementation details.

**Tags:** #user-roles, #development-story, #api-enhancement, #jwt-tokens, #implementation-details

### story-authz-20250111-login-info-non-authz.md
**Summary:** Development story for displaying non-authentication mode information on the login screen. Includes user story, acceptance criteria for UI changes, technical implementation details, and testing criteria to ensure users understand why login functionality is unavailable.

**Tags:** #authentication-mode, #ui-enhancement, #user-messaging, #login-screen, #implementation-details

### story-authz-20250111-reset-to-authz.md
**Summary:** Development story for a feature to convert a non-authenticated app instance to authenticated mode. Details user story, acceptance criteria, security considerations, and implementation flow for setting up authorization after initial non-authenticated deployment.

**Tags:** #authentication-conversion, #feature-enhancement, #user-flow, #security-setup, #implementation-plan

### story-20250130-setup-bodhi-info.md
**Summary:** Development story for the Bodhi App setup wizard's welcome screen. Includes user story, acceptance criteria for content, UI/UX, technical implementation, detailed layout designs for desktop/mobile, and component structure with TypeScript interfaces.

**Tags:** #setup-wizard, #user-onboarding, #ui-design, #responsive-design, #component-structure

### story-20250130-setup-finish.rst
**Summary:** Development story for the final step of Bodhi App's setup wizard. Details the completion summary screen with community connection features, layout specifications for desktop/mobile, and content sections including setup summary and community links.

**Tags:** #setup-wizard, #completion-screen, #community-building, #user-onboarding, #ui-design

### story-20250130-setup-model-download.rst
**Summary:** Development story for the model download step of the setup wizard. Focuses on providing hardware-based model recommendations, download process features (background downloads, progress tracking), and content structure with layout specifications for different screen sizes.

**Tags:** #setup-wizard, #model-download, #hardware-detection, #progress-tracking, #user-onboarding

### story-20250130-setup-llm-engine.rst
**Summary:** Development story for the LLM engine selection step in the setup wizard. Includes hardware analysis display, engine recommendations based on detected capabilities, download process handling, and detailed layout specifications for presenting options to users.

**Tags:** #setup-wizard, #hardware-analysis, #engine-selection, #optimization, #user-onboarding

### story-20250130-setup-resource-admin.rst
**Summary:** Development story for the Resource Admin Login step in the setup wizard (for authenticated mode). Details the process of assigning the first user as admin, content requirements, UI/UX specifications, and technical implementation of OAuth login integration.

**Tags:** #setup-wizard, #admin-setup, #authentication, #oauth-login, #user-onboarding

### story-20250130-setup-auth-mode.rst
**Summary:** Development story for the authentication mode selection step in the setup wizard. Covers the choice between authenticated and non-authenticated modes, content requirements, UI/UX specifications, and navigation logic with responsive layouts for different devices.

**Tags:** #setup-wizard, #authentication-mode, #user-decision, #responsive-design, #user-onboarding

### story-20250128-model-alias-revamp.rst
**Summary:** Detailed development plan for revamping the Model Alias page UI/UX. Covers backend API changes for usage metrics, enhanced frontend components, configuration management improvements, and testing interface additions with phased implementation approach.

**Tags:** #ui-revamp, #model-alias, #metrics, #configuration-management, #testing-interface

### story-20250128-modelfiles-revamp.rst
**Summary:** Development plan for creating a unified interface for model discovery and management. Details backend changes for storage API and enhanced model metadata, frontend improvements for storage dashboard and model details display, and content presentation optimizations.

**Tags:** #ui-revamp, #model-management, #discovery, #storage-management, #metadata

### story-20250126-download-llama-server.md
**Summary:** Technical specification for enhancing Bodhi App to download pre-built llama-server binaries from GitHub releases. Explains the separation of resource vs. BODHI_HOME binaries, implementation tasks, and clarifies questions about build process, security, and user experience.

**Tags:** #binary-management, #build-process, #variant-management, #storage-organization, #download-process

### story-20250112-app-settings.md
**Summary:** Development story document for the App Settings Management feature. Details requirements for configuring application settings through a UI rather than editing files directly, with specifications for access control, page layout, setting categories, and backend implementation.

**Tags:** #app-settings, #configuration-management, #admin-interface, #settings-hierarchy, #ui-design 