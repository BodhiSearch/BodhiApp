import express from 'express';

/**
 * Mock OpenAI API server for testing
 * Provides OpenAI-compatible endpoints for testing without real API calls
 */
export class MockOpenAIServer {
  constructor(config = {}) {
    this.requiresAuth = config.requiresAuth || false;
    this.server = null;
    this.app = null;
    this.port = config.port || this.randomPort();
    this.requestLog = [];
  }

  randomPort() {
    return Math.floor(Math.random() * (30000 - 20000) + 20000);
  }

  async start() {
    this.app = express();
    this.app.use(express.json());

    this.app.use((req, res, next) => {
      this.requestLog.push({
        method: req.method,
        path: req.path,
        headers: req.headers,
        body: req.body,
      });
      next();
    });

    this.app.get('/v1/models', (req, res) => {
      if (this.requiresAuth && !this.hasValidAuth(req)) {
        return res.status(401).json({
          error: {
            message: 'Invalid authentication',
            type: 'invalid_request_error',
            code: 'invalid_api_key',
          },
        });
      }

      res.json({
        object: 'list',
        data: [
          {
            id: 'mock-gpt-4',
            object: 'model',
            created: 1687882411,
            owned_by: 'mock-openai',
          },
          {
            id: 'mock-gpt-3.5-turbo',
            object: 'model',
            created: 1687882410,
            owned_by: 'mock-openai',
          },
        ],
      });
    });

    this.app.post('/v1/chat/completions', (req, res) => {
      if (this.requiresAuth && !this.hasValidAuth(req)) {
        return res.status(401).json({
          error: {
            message: 'Invalid authentication',
            type: 'invalid_request_error',
            code: 'invalid_api_key',
          },
        });
      }

      const { messages, model } = req.body;
      const lastMessage = messages[messages.length - 1];

      res.json({
        id: 'chatcmpl-mock123',
        object: 'chat.completion',
        created: Date.now(),
        model: model || 'mock-gpt-4',
        choices: [
          {
            index: 0,
            message: {
              role: 'assistant',
              content: `Mock response to: ${lastMessage.content}`,
            },
            finish_reason: 'stop',
          },
        ],
        usage: {
          prompt_tokens: 10,
          completion_tokens: 10,
          total_tokens: 20,
        },
      });
    });

    this.app.use((req, res) => {
      res.status(404).json({
        error: {
          message: 'Not found',
          type: 'invalid_request_error',
        },
      });
    });

    return new Promise((resolve, reject) => {
      this.server = this.app.listen(this.port, (err) => {
        if (err) {
          reject(err);
        } else {
          console.log(`Mock OpenAI server started on port ${this.port}`);
          resolve();
        }
      });
    });
  }

  async stop() {
    if (this.server) {
      return new Promise((resolve, reject) => {
        this.server.close((err) => {
          if (err) {
            reject(err);
          } else {
            console.log('Mock OpenAI server stopped');
            this.server = null;
            this.app = null;
            resolve();
          }
        });
      });
    }
  }

  hasValidAuth(req) {
    const authHeader = req.headers.authorization;
    return authHeader?.startsWith('Bearer ');
  }

  getBaseUrl() {
    return `http://localhost:${this.port}/v1`;
  }

  getRequestLog() {
    return this.requestLog;
  }

  clearRequestLog() {
    this.requestLog = [];
  }

  getLastRequest() {
    return this.requestLog[this.requestLog.length - 1];
  }
}

export function createMockOpenAIServer(config = {}) {
  return new MockOpenAIServer(config);
}
