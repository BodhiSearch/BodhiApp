# @bodhiapp/ts-client

TypeScript client for the Bodhi API, providing type-safe access to the API endpoints.

## Installation

```bash
npm install @bodhiapp/ts-client
```

## Usage

```typescript
import { BodhiClient } from "@bodhiapp/ts-client";

// Initialize the client
const client = new BodhiClient({
  baseUrl: "https://api.yourdomain.com",
  apiKey: "your-api-key",
});

// Create a chat completion
async function chatWithBodhi() {
  const response = await client.createChatCompletion({
    model: "gpt-3.5-turbo",
    messages: [
      { role: "system", content: "You are a helpful assistant." },
      { role: "user", content: "Hello, who are you?" },
    ],
  });

  console.log(response.choices[0].message.content);
}

// List available models
async function listAvailableModels() {
  const models = await client.listModels();
  console.log(models.data);
}
```

## Development

This package is generated from the Bodhi API OpenAPI specification.

To regenerate the client:

```bash
npm run generate
```

To build the package:

```bash
npm run build
```

To run tests:

```bash
npm test
```
