---
title: 'Introduction'
description: 'Getting started with Bodhi'
order: 0
---

# Welcome to Bodhi App

Bodhi App is a cross‑platform LLM inference tool that seamlessly integrates with the HuggingFace ecosystem and is powered by the llama.cpp server. Whether you're a non‑technical user exploring AI or a developer building sophisticated AI‑powered applications, Bodhi App delivers a rich set of features designed to empower your journey.

## Key Features

- **OpenAI Compatible Chat and Model APIs**
  Our chat completions and models API is OpenAI compatible, so you can seamlessly integrate with any tool that accepts OpenAI's API endpoints.

- **Ollama Compatible Chat and Model APIs**
  We also offer Ollama compatible API - chat and models endpoints, providing flexibility and integration options for users within the Ollama ecosystem.

- **Hybrid AI Architecture**
  Use powerful API models from leading providers (OpenAI, Anthropic, Groq, Together AI) alongside your local GGUF models in a unified interface. [Learn more about API Models](/docs/features/api-models).

- **User Management System**
  Admin dashboard for managing users, approving access requests, and assigning roles with granular permissions. [Learn more about User Management](/docs/features/user-management).

- **Multi-Platform Support**
  Available as desktop apps (Windows, macOS Intel/ARM, Linux) and Docker containers with hardware-specific variants (CPU, CUDA, ROCm). [Learn more about Deployment](/docs/deployment/docker/).

- **Developer SDK**
  TypeScript client package for integrating with Bodhi App APIs into your applications [@bodhiapp/ts-client](https://www.npmjs.com/package/@bodhiapp/ts-client).

- **Advanced Security & Authentication**
  Bodhi App uses OAuth2 with PKCE (Proof Key for Code Exchange) for secure, mandatory authentication. The first user to log in via OAuth automatically becomes the admin (determined by checking if any users exist in the database during OAuth login), while subsequent users must request access and await admin approval.

  **Key Security Features:**

  - Mandatory OAuth2 authentication for all users
  - First-user admin assignment
  - Access request workflow for new users
  - Role-based permissions (User, PowerUser, Manager, Admin)

  See [Installing](/docs/install/) for authentication setup details.

- **Built-in Chat UI**  
  Say goodbye to separate installations—Bodhi App includes a ready‑to‑use Chat UI featuring markdown support (with code blocks and copy functionality), multi‑conversation management, customizable system prompts, and real‑time feedback. For those who prefer an external Chat frontend, our OpenAI‑compatible API endpoints make integration effortless.

- **Model Alias**  
  Easily create and save your inference configurations by specifying your preferred request parameters and llama‑server settings—such as temperature and context size. Switch between configurations seamlessly without any restarts.

- **Model Files Management**
  Manage your AI model files seamlessly with the HuggingFace ecosystem. GGUF model files are downloaded into your local HuggingFace home folder, and any compatible GGUF models already present can be used for inference. Model alias configurations are stored separately in `$BODHI_HOME/aliases`. This design saves disk space and bandwidth by reusing your existing resources.

- **Robust API Access & Developer Tools**  
  Benefit from comprehensive API documentation and developer tools. Whether you are generating API tokens or integrating Bodhi App into external applications, every technical detail is supported.

- **Future‑Ready and Cross‑Platform**  
  Designed for desktop, mobile, and web, Bodhi App is continuously evolving—with upcoming features such as expanded AI API integrations and enhanced authorization flows.

- **Guided Onboarding & Ongoing Support**  
  Kickstart your journey with [Installing](/docs/install/) Bodhi App. For more technical details, explore our [API References](/docs/features/openapi-docs/) and discover additional features on our [Features](/docs/features/) page.

## What's in Store

Bodhi App is an evolving platform, and our roadmap includes:

- **Enhanced Chat Capabilities:** Support for rich media, file attachments, and even more interactive conversation management.
- **Extended Provider Support:** Additional AI providers beyond current OpenAI-compatible APIs.
- **Advanced Features:** Performance optimizations, enhanced model management, and improved deployment options.

## Choose Your Path

- **Lets Get Started**  
  Begin your journey by [Installing](/docs/install/) Bodhi App.

- **For Developers:**  
  Dive into technical details in our [API References](/docs/features/openapi-docs/).

- **Explore?**  
  Find out more in our [Features](/docs/features/).

---

Bodhi App is purposefully designed to be both powerful and user‑friendly—empowering non‑technical users to experiment freely while giving developers the tools to build sophisticated AI‑powered applications.

_Explore, experiment, and evolve with Bodhi App!_
