---
title: "API Reference"
description: "API documentation and integration guide for Bodhi App"
order: 260
---

# API Reference

Bodhi App provides comprehensive, auto-generated API documentation powered. This documentation is available directly within the app via an interactive Swagger UI, enabling developers to explore, understand, and integrate with our RESTful APIs effortlessly.

## Overview

Our API documentation covers all public and authenticated endpoints and is continuously updated as new features are added. The Swagger UI makes it easy to:
- Browse detailed API specifications, including endpoint descriptions and data schemas.
- Interactively test API endpoints with real-time requests.
- Understand available authentication methods, such as browser session and API token-based authentication, without needing to review lengthy manual documentation.

## How to Access the Documentation

You can access the interactive API documentation in two ways:
- From within Bodhi App, select the **API Documentation** option from the menu.
- Or directly visit:  
  `http://<your-bodhi-instance>/swagger-ui`

This feature ensures that you always have access to the latest and most accurate API information, simplifying your integration and testing workflows.

## Maintaining Consistency

Our backend automatically generates the OpenAPI specification. As the codebase evolves, so does the documentation – ensuring that what you see in the Swagger UI is always current.

## Summary

The integrated API documentation feature of Bodhi App makes it easy for you to explore and test our APIs without the need for separate reference materials. Whether you are troubleshooting, integrating with other systems, or simply learning about our services, the Swagger UI provides a clear and interactive guide to all available endpoints.

Happy integrating!

## Quick Start Examples

### Chat Completion API
```bash
curl -X POST http://localhost:1135/v1/chat/completions \
  -H "Content-Type: application/json" \
  -H "Authorization: Bearer YOUR_API_TOKEN" \
  -d '{
    "model": "your-model-alias",
    "messages": [
      {"role": "system", "content": "You are a helpful assistant."},
      {"role": "user", "content": "Hello!"}
    ]
  }'
```

### Model List API
```bash
curl http://localhost:1135/v1/models \
  -H "Authorization: Bearer YOUR_API_TOKEN"
```