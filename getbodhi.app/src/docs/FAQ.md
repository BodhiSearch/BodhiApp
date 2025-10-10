---
title: 'FAQ'
description: 'Frequently Asked Questions'
order: 500
---

# FAQ

This page provides answers to common questions about Bodhi App for all users.

## General Questions

### What is Bodhi App?

Bodhi App is a local LLM inference application built on top of the HuggingFace and llama.cpp ecosystems. It features a built-in Chat UI, model downloads, API access, and dynamic configuration management.

### What platforms does Bodhi App support?

Bodhi App supports multiple platforms:

- **Desktop**: macOS (Intel and Apple Silicon), Windows, and Linux
- **Docker**: Available in multiple variants (CPU, CUDA, ROCm) for different hardware configurations
- **Cloud**: Deployable on platforms like Railway and RunPod

### How often is Bodhi App updated?

Bodhi App is regularly updated with new features and improvements. Update instructions are provided with each release.

## Setup & Configuration

### What authentication features does Bodhi App provide?

Bodhi App uses OAuth2 PKCE authentication for all users, providing secure multi-user access with role-based permissions. The first registered user automatically becomes the admin, and subsequent users request access through an admin-approved workflow.

### How do I become an admin?

The first user to log in via OAuth automatically receives admin privileges (determined by checking if any users exist in the database during OAuth login). Subsequent users receive the "User" role after admin approval and can have their role upgraded by existing admins through the Users management page.

### How do additional users get access?

New users follow this workflow:

1. Log in with OAuth to authenticate their identity
2. Submit an access request through the Request Access page
3. Wait for admin/manager approval
4. Approver selects the role during approval (Admin can assign any role; Manager can assign Manager, PowerUser, or User roles)
5. Once approved, they receive their assigned role and can access the application

For more details, see the [User Management](/docs/features/user-management/) documentation.

### How do I access the app settings?

Visit the **Settings** page to view and update configuration settings. Changes take effect immediately without requiring an application restart.

## Model and Inference

### How does Model Alias work?

A model alias defines the default inference and server parameters for a model. For more details, please refer to the [Model Alias](/docs/features/model-alias/) page.

### Can I use both local and remote AI models?

Yes! Bodhi App supports hybrid AI usage:

- **Local models**: Download GGUF models from HuggingFace for complete privacy
- **API Models**: Configure providers like OpenAI, Anthropic, Groq, and Together AI for access to cloud-based models

You can switch between local and remote models seamlessly in the chat interface. For more information, see the [API Models](/docs/features/api-models/) documentation.

### How do I download a model?

Go to the **Download Models** section, provide the Huggingface repository name and filename, and submit your download request. Downloads are processed asynchronously and continue in the background. Monitor their status with real-time progress tracking on the Downloads page.

### What should I do if a model download fails?

Verify your network connection and review any error messages on the Downloads page. For further guidance, see the [Troubleshooting](/docs/troubleshooting/) page.

### What Docker variants are available?

Bodhi App provides multiple Docker variants optimized for different hardware:

- **CPU**: Multi-platform support (AMD64 and ARM64) for maximum compatibility
- **CUDA**: NVIDIA GPU acceleration for 8-12x performance boost
- **ROCm**: AMD GPU acceleration

For deployment instructions, see the [Docker Deployment](/docs/deployment/docker/) documentation.

### What does the browser extension do?

The Bodhi App browser extension is a powerful feature that exposes your authenticated endpoints to any webpage, enabling AI capabilities powered by Bodhi App on any website. Currently supports Chrome, with Firefox and Safari support planned for future releases. Highly recommended for maximum Bodhi App functionality. Demo applications will be available soon.

### How do I update Bodhi App?

**Desktop Applications:**

1. Download the latest installer for your platform from our releases page
2. Install the new version - it will automatically replace the existing installation
3. Your settings and configurations will be preserved

**Docker Deployments:**
Pull the latest image for your chosen variant:

```
docker pull ghcr.io/bodhisearch/bodhiapp:latest-cpu
docker pull ghcr.io/bodhisearch/bodhiapp:latest-cuda
```

### What should I do if I encounter a model loading error?

1. Review the logs at `$BODHI_HOME/logs` for specific error messages
2. Try to resolve based on logs or
3. Restart the application

## API & Developer

### How do I create an API token?

Access the **Token Management** section and click "Generate Token." The token is displayed only once—copy it immediately. For more information, see [Token Management](/docs/features/api-tokens/).

### How can I access the API documentation?

Bodhi App provides interactive API documentation via the Swagger UI. You can access it from the **API Documentation** menu or directly at:

```
http://<your-bodhi-instance>/swagger-ui
```

This documentation is auto‑generated with Utoipa and kept up‑to‑date.

### What is the difference between session-based and API token authentication?

- **Session-based:** Uses browser login with OAuth and secure cookies for interactive web UI access
- **API token:** Database-backed tokens with cryptographic security (SHA-256) for programmatic access via API calls

API tokens support scope-based permissions and can be activated or deactivated as needed.

### How do API tokens work?

API tokens are database-backed credentials that provide secure programmatic access to Bodhi App:

1. Create tokens with specific scopes (e.g., chat.completions, models.read) through the Token Management page
2. Copy the token immediately - it's shown only once for security
3. Use the token in API requests: `Authorization: Bearer <your-token>`
4. Activate or deactivate tokens anytime without deleting them

For more information, see the [API Token Management](/docs/features/api-tokens/) documentation.

### What's the difference between user roles?

Bodhi App implements a hierarchical role system:

- **User**: Basic access to chat and embeddings APIs
- **PowerUser**: Can download and delete model files, plus all User capabilities
- **Manager**: Can manage users and approve access requests, plus all PowerUser capabilities (can manage Users, PowerUsers, and other Managers, but cannot manage Admins)
- **Admin**: Full system access including all user management capabilities

Admins and Managers can modify user roles through the Users management page (with hierarchical restrictions for Managers).

## Troubleshooting & Support

### What should I do if I encounter issues with Bodhi App?

Consult the [Troubleshooting](/docs/troubleshooting/) page for common issues and solutions. If problems persist, reach out via our Discord or Github Issues.

### Where can I find additional support?

Additional support is available through our official website, Discord channel, and Github Issues.
