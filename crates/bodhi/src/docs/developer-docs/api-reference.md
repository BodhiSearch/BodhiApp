---
title: "API Reference"
description: "Complete API reference for Bodhi App"
---

# API Reference

This document details the API endpoints exposed by Bodhi App. It is intended for developers and integrators.

## Overview

Bodhi App provides RESTful endpoints for:
- Managing models and inference jobs
- Downloading models from Hugging Face
- Creating and managing API tokens

## Endpoints

### Get App Information

```
GET /api/app-info
```
_Returns status, version, and authentication mode._

### List Models

```
GET /api/models
```
_Returns the list of locally downloaded models with metadata._

### Pull Model from Hugging Face

```
POST /api/models/pull
```
_Requires a valid Hugging Face token._

### Create API Token

```
POST /api/tokens
```
_Creates a new API token to secure remote requests._

## Sample Request & Response

### Example: Create API Token

**Request:**

```json
{
  "name": "my-app-token"
}
```

**Response:**

```json
{
  "offline_token": "abc123xyz"
}
```

For more details, consult our [Developer Guide](./dev/DeveloperGuide.md). 