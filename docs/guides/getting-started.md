# Getting Started

This guide walks you through installing BodhiApp, completing the initial setup, and making your first API call. You'll be up and running with local LLM inference in under 15 minutes.

## Prerequisites

Before you begin, ensure your system meets the requirements:

### System Requirements
- **Memory**: 8GB RAM minimum (16GB recommended)
- **GPU**: 8GB iGPU for hardware acceleration
- **CPU**: 8-core processor for optimal performance
- **Storage**: 5GB+ available space for model downloads
- **Network**: Stable internet connection for downloads and authentication

### Supported Platforms
- **macOS**: 14.0+ on ARM64 (M-series chips) ✅ Currently Available
- **Windows**: Intel/AMD x64 🔄 Coming Soon
- **Linux**: Intel/AMD x64 🔄 Coming Soon

## Step 1: Download and Install BodhiApp

### Download BodhiApp
1. Visit [https://getbodhi.app](https://getbodhi.app)
2. Download the appropriate version for your platform
3. For macOS: Download the `.dmg` file

### Installation Process

**macOS Installation:**
1. Open the downloaded `.dmg` file
2. Drag BodhiApp into your Applications folder
3. Launch BodhiApp from Applications

**First Launch:**
- BodhiApp appears in your system tray
- A browser window automatically opens at `http://localhost:1135`
- The app serves a web interface while running locally

## Step 2: Complete Initial Setup

BodhiApp guides you through a 4-step setup process on first launch:

### Setup Step 1: Server Configuration
1. **Server Information**: Provide a name and optional description for your BodhiApp instance
   - **Name**: Choose a descriptive name (e.g., "My AI Server")
   - **Description**: Optional details about your setup

2. **Click "Setup Bodhi Server"** to proceed

### Setup Step 2: Authentication & Admin Setup
1. **Resource Admin Assignment**: The first user becomes the admin
2. **OAuth Authentication**: Click "Login" to authenticate with bodhi-auth-server
3. **Account Creation**: If you don't have an account, you'll be prompted to create one
4. **Admin Privileges**: Your account is automatically assigned admin privileges

**Authentication Flow:**
- Redirects to `https://id.getbodhi.app` for secure authentication
- Creates your account if it doesn't exist
- Returns to BodhiApp with admin privileges

### Setup Step 3: Model Downloads
1. **Suggested Models**: BodhiApp displays popular models compatible with your system
2. **Model Selection**: Choose models to download (downloads happen in background)
3. **Popular Options**:
   - **Llama 3 8B Instruct**: Balanced performance and quality
   - **Phi-3 Mini**: Lightweight option for limited resources
   - **Mistral 7B Instruct**: Fast and efficient

**Model Download Process:**
- Downloads occur asynchronously in the background
- Models are stored in your HuggingFace cache directory
- You can continue setup while downloads progress

### Setup Step 4: Completion
1. **Setup Summary**: Review your configuration
2. **Social Media Links**: Optional follow prompts
3. **Complete Setup**: Finalize the installation

## Step 3: Verify Installation

### Access the Web Interface
1. **Open Browser**: Navigate to `http://localhost:1135`
2. **Login**: Use your newly created account
3. **Dashboard**: You should see the main BodhiApp interface

### Check System Status
```bash
# Test basic connectivity
curl http://localhost:1135/ping
```

**Expected Response:**
```json
{
  "message": "pong"
}
```

### Verify App Information
```bash
# Get app status and version
curl http://localhost:1135/bodhi/v1/info
```

**Expected Response:**
```json
{
  "version": "0.1.0",
  "status": "ready"
}
```

## Step 4: Set Up Development Environment

### Install TypeScript Client
```bash
# Install the official TypeScript client
npm install @bodhiapp/ts-client
```

### Create API Token
1. **Access Token Management**: In the BodhiApp web interface, go to **Settings** → **API Tokens**
2. **Create New Token**: Click "Create Token"
3. **Token Configuration**:
   - **Name**: Descriptive name (e.g., "Development Token")
   - **Scope**: Select appropriate permissions (start with `scope_token_power_user`)
4. **Save Token**: Copy the generated token immediately (it won't be shown again)

### Test API Access
```typescript
// test-api.ts
import { type AppInfo } from '@bodhiapp/ts-client';

const API_TOKEN = 'your-api-token-here';
const BASE_URL = 'http://localhost:1135';

async function testAPI() {
  try {
    // Test authenticated endpoint
    const response = await fetch(`${BASE_URL}/bodhi/v1/info`, {
      headers: {
        'Authorization': `Bearer ${API_TOKEN}`,
        'Content-Type': 'application/json'
      }
    });

    if (!response.ok) {
      throw new Error(`HTTP ${response.status}: ${response.statusText}`);
    }

    const appInfo: AppInfo = await response.json();
    console.log('✅ API Access Successful!');
    console.log('App Version:', appInfo.version);
    console.log('App Status:', appInfo.status);
  } catch (error) {
    console.error('❌ API Access Failed:', error);
  }
}

testAPI();
```

Run the test:
```bash
npx tsx test-api.ts
```

## Step 5: Make Your First Chat Request

### Using OpenAI-Compatible Endpoint
```typescript
// first-chat.ts
const API_TOKEN = 'your-api-token-here';
const BASE_URL = 'http://localhost:1135';

async function firstChat() {
  try {
    const response = await fetch(`${BASE_URL}/v1/chat/completions`, {
      method: 'POST',
      headers: {
        'Authorization': `Bearer ${API_TOKEN}`,
        'Content-Type': 'application/json'
      },
      body: JSON.stringify({
        model: 'llama3:instruct', // Use a model you downloaded
        messages: [
          {
            role: 'user',
            content: 'Hello! Can you introduce yourself?'
          }
        ],
        max_tokens: 100,
        temperature: 0.7
      })
    });

    if (!response.ok) {
      throw new Error(`HTTP ${response.status}: ${response.statusText}`);
    }

    const result = await response.json();
    console.log('🤖 AI Response:');
    console.log(result.choices[0].message.content);
  } catch (error) {
    console.error('❌ Chat Request Failed:', error);
  }
}

firstChat();
```

### Using OpenAI SDK (Drop-in Replacement)
```typescript
// openai-sdk-test.ts
import OpenAI from 'openai';

const client = new OpenAI({
  apiKey: 'your-api-token-here',
  baseURL: 'http://localhost:1135/v1'
});

async function chatWithOpenAISDK() {
  try {
    const completion = await client.chat.completions.create({
      model: 'llama3:instruct',
      messages: [
        {
          role: 'user',
          content: 'Explain what you are in one sentence.'
        }
      ],
      max_tokens: 50
    });

    console.log('🤖 Response:', completion.choices[0].message.content);
  } catch (error) {
    console.error('❌ Error:', error);
  }
}

chatWithOpenAISDK();
```

## Troubleshooting Common Issues

### App Won't Start
**Issue**: BodhiApp doesn't launch or browser doesn't open
**Solutions**:
- Check if port 1135 is available
- Restart BodhiApp from Applications
- Check $HOME/.cache/bodhi/logs folder for error messages

### Authentication Problems
**Issue**: Login fails or redirects don't work
**Solutions**:
- Ensure stable internet connection
- Check if `https://id.getbodhi.app` is accessible
- Clear browser cache and cookies
- Try incognito/private browsing mode

### API Token Issues
**Issue**: API calls return 401 Unauthorized
**Solutions**:
- Verify token is correctly copied (no extra spaces)
- Check token hasn't expired or been deactivated
- Ensure proper `Authorization: Bearer <token>` header format
- Verify token has appropriate scope permissions

### Model Download Failures
**Issue**: Models fail to download during setup
**Solutions**:
- Check internet connection stability
- Verify sufficient disk space (5GB+ recommended)
- Wait for current downloads to complete before starting new ones
- Check the Downloads page for error details

### Port Conflicts
**Issue**: "Port 1135 already in use" error
**Solutions**:
- Check if another BodhiApp instance is running
- Kill any processes using port 1135: `lsof -ti:1135 | xargs kill`
- Restart BodhiApp after clearing the port

## Next Steps

Now that BodhiApp is installed and configured:

1. **[Explore the UI](embedded-ui.md)** - Learn the built-in web interface
2. **[Understand Authentication](authentication.md)** - Deep dive into API tokens and permissions
3. **[Try OpenAI APIs](openai-api.md)** - Use familiar OpenAI-compatible endpoints
4. **[Manage Models](model-management.md)** - Download and configure additional models

## Quick Reference

### Essential URLs
- **Web Interface**: `http://localhost:1135`
- **API Documentation**: `http://localhost:1135/docs`
- **Health Check**: `http://localhost:1135/ping`

### Key Directories
- **macOS App**: `/Applications/BodhiApp.app`
- **Models**: `~/.cache/huggingface/hub/` (HuggingFace cache)
- **Logs**: Check system logs or app console

### Default Configuration
- **Server Port**: 1135
- **Authentication**: Required (bodhi-auth-server)
- **First User**: Automatically becomes admin
- **API Base**: `http://localhost:1135`

---

*You're now ready to start building with BodhiApp! The next section covers the embedded UI in detail.* 