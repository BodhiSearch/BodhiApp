# @bodhiapp/ts-client

TypeScript types for the Bodhi API, providing type-safe access to the API endpoints.

## Installation

```bash
npm install @bodhiapp/ts-client
```

## Usage

```typescript
import { ChatRequest } from "@bodhiapp/ts-client";

// Example chat request with type safety
const request: ChatRequest = {
  model: "llama2",
  messages: [
    { role: "user", content: "Hello, who are you?" }
  ],
  options: {
    temperature: 0.7,
    num_predict: 100
  },
  stream: false
};

// Make API calls using your preferred HTTP client
async function chatWithBodhi() {
  const response = await fetch("http://localhost:3000/v1/chat/completions", {
    method: "POST",
    headers: {
      "Content-Type": "application/json"
    },
    body: JSON.stringify(request)
  });

  const data = await response.json();
  console.log(data.choices[0].message.content);
}
```

## Development

This package provides TypeScript types generated from the Bodhi API OpenAPI specification.

To regenerate the types:

```bash
npm run generate
```

This will:
1. Generate the OpenAPI spec from the Rust backend
2. Generate TypeScript types using @hey-api/openapi-ts

To build the package:

```bash
npm run build
```

To run tests:

```bash
npm test
```

## Types

The package exports TypeScript interfaces for all Bodhi API requests and responses. The main types include:

- `ChatRequest` - Type for chat completion requests
- `Message` - Type for chat messages
- `AppInfo` - Application information and status
- `Model` - Model information
- `ApiToken` - API token information
- And more...

All types are fully documented with JSDoc comments for better IDE integration.
