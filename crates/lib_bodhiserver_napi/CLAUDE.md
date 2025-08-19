# CLAUDE.md - lib_bodhiserver_napi

This file provides guidance to Claude Code when working with the `lib_bodhiserver_napi` crate, which provides Node.js bindings for BodhiApp server functionality.

## Purpose

The `lib_bodhiserver_napi` crate provides Node.js/JavaScript bindings for BodhiApp:

- **Node.js Integration**: Native Node.js module for JavaScript applications
- **High Performance**: Direct Rust-to-JavaScript bindings without HTTP overhead
- **Async Support**: Promise-based API with proper async/await integration
- **Type Safety**: TypeScript definitions for complete type safety
- **Cross-Platform**: Support for Windows, macOS, and Linux

## Key Components

### NAPI Bindings
- Native Node.js module built with `napi-rs`
- Async function bindings with Promise support
- Error handling and type conversion
- Memory management and cleanup

### JavaScript API
- Promise-based interface for all operations
- TypeScript definitions for full type safety
- Event emitters for streaming operations
- Error handling with proper stack traces

### Server Interface
- Complete BodhiServer functionality exposed to Node.js
- Model management operations
- Chat completion interface
- Authentication and token management

## Dependencies

### Core Server
- `lib_bodhiserver` - Embeddable server library
- `services` - Business logic services
- `objs` - Domain objects and validation

### NAPI Framework
- `napi-rs` - Node.js API bindings framework
- `napi-derive` - Procedural macros for binding generation
- `tokio` - Async runtime integration

## Architecture Position

The `lib_bodhiserver_napi` crate sits at the Node.js binding layer:
- **Bridges**: Rust server functionality with JavaScript/Node.js ecosystem
- **Provides**: High-performance native module interface
- **Manages**: Memory safety and type conversion between Rust and JavaScript
- **Exposes**: Complete BodhiApp functionality to Node.js applications

## Usage Patterns

### Installation and Setup
```bash
npm install @bodhi/server
# or
yarn add @bodhi/server
```

### Basic Usage (JavaScript)
```javascript
const { BodhiServer } = require('@bodhi/server');

async function main() {
  const server = new BodhiServer({
    databaseUrl: 'sqlite:///path/to/db.sqlite',
    dataDir: '/path/to/data',
    enableHttp: false,
  });

  await server.start();
  
  // List available models
  const models = await server.listModels();
  console.log('Available models:', models);
  
  // Chat completion
  const response = await server.chatCompletion({
    model: 'my-model',
    messages: [
      { role: 'user', content: 'Hello!' }
    ]
  });
  
  console.log('Response:', response);
  
  await server.shutdown();
}

main().catch(console.error);
```

### TypeScript Usage
```typescript
import { BodhiServer, ChatCompletionRequest, Model } from '@bodhi/server';

async function main(): Promise<void> {
  const server = new BodhiServer({
    databaseUrl: 'sqlite:memory:',
    dataDir: './data',
    enableHttp: false,
  });

  await server.start();
  
  const models: Model[] = await server.listModels();
  
  const request: ChatCompletionRequest = {
    model: 'gpt-3.5-turbo',
    messages: [
      { role: 'system', content: 'You are a helpful assistant.' },
      { role: 'user', content: 'Explain quantum computing.' }
    ],
    temperature: 0.7,
    maxTokens: 150,
  };
  
  const response = await server.chatCompletion(request);
  console.log(response.choices[0].message.content);
  
  await server.shutdown();
}
```

### Express.js Integration
```javascript
const express = require('express');
const { BodhiServer } = require('@bodhi/server');

const app = express();
app.use(express.json());

let bodhiServer;

async function initializeServer() {
  bodhiServer = new BodhiServer({
    databaseUrl: process.env.DATABASE_URL,
    dataDir: process.env.DATA_DIR,
  });
  await bodhiServer.start();
}

app.post('/api/chat', async (req, res) => {
  try {
    const response = await bodhiServer.chatCompletion(req.body);
    res.json(response);
  } catch (error) {
    res.status(500).json({ error: error.message });
  }
});

app.get('/api/models', async (req, res) => {
  try {
    const models = await bodhiServer.listModels();
    res.json(models);
  } catch (error) {
    res.status(500).json({ error: error.message });
  }
});

initializeServer().then(() => {
  app.listen(3000, () => {
    console.log('Server running on port 3000');
  });
});
```

### Electron Application Integration
```javascript
// In main process
const { BodhiServer } = require('@bodhi/server');
const { app, BrowserWindow, ipcMain } = require('electron');
const path = require('path');

let bodhiServer;

app.whenReady().then(async () => {
  // Initialize BodhiServer
  bodhiServer = new BodhiServer({
    databaseUrl: `sqlite:///${path.join(app.getPath('userData'), 'bodhi.db')}`,
    dataDir: path.join(app.getPath('userData'), 'bodhi-data'),
    enableHttp: false,
  });

  await bodhiServer.start();

  // Create main window
  const mainWindow = new BrowserWindow({
    webPreferences: {
      nodeIntegration: false,
      contextIsolation: true,
      preload: path.join(__dirname, 'preload.js'),
    },
  });

  // IPC handlers
  ipcMain.handle('list-models', async () => {
    return await bodhiServer.listModels();
  });

  ipcMain.handle('chat-completion', async (event, request) => {
    return await bodhiServer.chatCompletion(request);
  });
});
```

## API Interface

### Server Management
```typescript
interface BodhiServerConfig {
  databaseUrl?: string;
  dataDir?: string;
  enableHttp?: boolean;
  bindAddress?: string;
  logLevel?: string;
  cacheSize?: number;
}

class BodhiServer {
  constructor(config: BodhiServerConfig);
  
  async start(): Promise<void>;
  async shutdown(): Promise<void>;
  
  // Model management
  async listModels(): Promise<Model[]>;
  async createModel(request: CreateModelRequest): Promise<void>;
  async pullModel(request: PullModelRequest): Promise<void>;
  async deleteModel(alias: string): Promise<void>;
  
  // Chat operations
  async chatCompletion(request: ChatCompletionRequest): Promise<ChatCompletionResponse>;
  
  // Authentication (when enabled)
  async initiateLogin(): Promise<string>;
  async completeLogin(params: CallbackParams): Promise<TokenResponse>;
  async createApiToken(request: TokenRequest): Promise<TokenResponse>;
}
```

### Type Definitions
```typescript
interface Model {
  id: string;
  object: string;
  created: number;
  ownedBy: string;
}

interface ChatCompletionRequest {
  model: string;
  messages: Message[];
  temperature?: number;
  maxTokens?: number;
  stream?: boolean;
}

interface Message {
  role: 'system' | 'user' | 'assistant';
  content: string;
}

interface ChatCompletionResponse {
  id: string;
  object: string;
  created: number;
  model: string;
  choices: Choice[];
  usage: Usage;
}
```

## Build System

### Native Module Build
```json
// package.json
{
  "name": "@bodhi/server",
  "version": "1.0.0",
  "main": "index.js",
  "types": "index.d.ts",
  "napi": {
    "name": "bodhiserver",
    "targets": ["x86_64-pc-windows-msvc", "x86_64-apple-darwin", "aarch64-apple-darwin", "x86_64-unknown-linux-gnu"]
  },
  "scripts": {
    "build": "napi build --platform",
    "prepublishOnly": "napi prepublish -t npm"
  }
}
```

### Cross-Platform Support
- Windows (x86_64)
- macOS (x86_64, ARM64)
- Linux (x86_64)
- Automated building for all platforms
- GitHub Actions CI/CD integration

## Error Handling

### Rust to JavaScript Error Mapping
```rust
impl From<ServerError> for napi::Error {
    fn from(err: ServerError) -> Self {
        napi::Error::new(
            napi::Status::GenericFailure,
            err.to_string(),
        )
    }
}
```

### JavaScript Error Handling
```javascript
try {
  const response = await server.chatCompletion(request);
  console.log(response);
} catch (error) {
  if (error.message.includes('model not found')) {
    console.error('Model not available:', error);
  } else {
    console.error('Unexpected error:', error);
  }
}
```

## Performance Considerations

### Memory Management
- Automatic garbage collection of Rust objects
- Efficient data transfer between Rust and JavaScript
- Streaming support for large responses
- Connection pooling and reuse

### Async Operations
- Non-blocking operations with proper Promise integration
- Tokio runtime integration with Node.js event loop
- Efficient handling of concurrent requests

### Bundle Size
- Optimized native binary size
- Minimal JavaScript wrapper code
- Efficient type definitions

## Development Guidelines

### Adding New Bindings
1. Define Rust function with proper NAPI annotations
2. Handle type conversion and error mapping
3. Add TypeScript definitions
4. Include comprehensive tests
5. Update documentation and examples

### Testing Strategy
- Unit tests for individual bindings
- Integration tests with Node.js applications
- Cross-platform testing on all supported platforms
- Memory leak detection
- Performance benchmarking

### Release Process
- Automated building for all platforms
- npm package publishing
- Version synchronization with main crate
- Changelog and documentation updates

## Integration Examples

### Next.js API Routes
```javascript
// pages/api/chat.js
import { BodhiServer } from '@bodhi/server';

let server;

async function getServer() {
  if (!server) {
    server = new BodhiServer({
      databaseUrl: process.env.DATABASE_URL,
      dataDir: process.env.DATA_DIR,
    });
    await server.start();
  }
  return server;
}

export default async function handler(req, res) {
  if (req.method !== 'POST') {
    return res.status(405).json({ error: 'Method not allowed' });
  }

  try {
    const server = await getServer();
    const response = await server.chatCompletion(req.body);
    res.json(response);
  } catch (error) {
    res.status(500).json({ error: error.message });
  }
}
```

### Discord Bot Integration
```javascript
const { Client, GatewayIntentBits } = require('discord.js');
const { BodhiServer } = require('@bodhi/server');

const client = new Client({ intents: [GatewayIntentBits.Guilds, GatewayIntentBits.GuildMessages] });
let bodhiServer;

client.once('ready', async () => {
  bodhiServer = new BodhiServer({
    databaseUrl: 'sqlite:///bot-data/bodhi.db',
    dataDir: './bot-data',
  });
  await bodhiServer.start();
  console.log('Bot and BodhiServer ready!');
});

client.on('messageCreate', async (message) => {
  if (message.author.bot || !message.content.startsWith('!chat')) return;

  try {
    const response = await bodhiServer.chatCompletion({
      model: 'gpt-3.5-turbo',
      messages: [{ role: 'user', content: message.content.slice(5) }],
      maxTokens: 100,
    });
    
    await message.reply(response.choices[0].message.content);
  } catch (error) {
    await message.reply(`Error: ${error.message}`);
  }
});
```

## Future Extensions

The lib_bodhiserver_napi crate can be extended with:
- Streaming responses with async iterators
- WebWorker support for background processing
- React Native bindings
- Bun and Deno compatibility
- Advanced configuration options
- Real-time event subscriptions