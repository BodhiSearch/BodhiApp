---
title: "FAQ"
description: "Frequently Asked Questions"
order: 500
---

# FAQ

This page provides answers to common questions about Bodhi App for all users.

## General Questions

### What is Bodhi App?
Bodhi App is a local LLM inference application built on top of the Huggingface and llama.cpp ecosystems. It features a built-in Chat UI, model downloads, API access, and dynamic configuration management.

### What platforms does Bodhi App support?
Currently, Bodhi App is available on macOS for M‑series devices, with additional platforms coming soon.

### How often is Bodhi App updated?
Bodhi App is regularly updated with new features and improvements. Update instructions are provided with each release.

## Setup & Configuration

### What is the difference between authenticated and non‑authenticated mode?
- **Authenticated Mode:** Enhanced security and role‑based access control.
- **Non‑Authenticated Mode:** Quick, open access for local testing and exploration, with limited features.

### How do I access the app settings?
Visit the **Settings** page to view and update configuration settings. Changes take effect immediately without requiring an application restart.

## Model and Inference

### How does Model Alias work?
A model alias defines the default inference and server parameters for a model. For more details, please refer to the [Model Alias](/docs/features/model-alias/) page.

### How do I download a model?
Go to the **Download Models** section, provide the Huggingface repository name and filename, and submit your download request. Downloads are processed asynchronously, and you can monitor their status on the Downloads page.

### What should I do if a model download fails?
Verify your network connection and review any error messages on the Downloads page. For further guidance, see the [Troubleshooting](/docs/troubleshooting/) page.

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
- **Session-based:** Uses browser login and cookies.
- **API token:** Uses tokens generated within the app for secure, programmatic access.

## Troubleshooting & Support

### What should I do if I encounter issues with Bodhi App?
Consult the [Troubleshooting](/docs/troubleshooting/) page for common issues and solutions. If problems persist, reach out via our Discord or Github Issues.

### Where can I find additional support?
Additional support is available through our official website, Discord channel, and Github Issues.
