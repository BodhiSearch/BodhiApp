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

      const { messages, model, stream } = req.body;
      const lastMessage = messages[messages.length - 1];
      // Handle both string content and array-of-parts content (OpenAI format)
      const lastContent =
        typeof lastMessage.content === 'string'
          ? lastMessage.content
          : Array.isArray(lastMessage.content)
            ? lastMessage.content
                .filter((p) => p.type === 'text')
                .map((p) => p.text)
                .join('')
            : String(lastMessage.content);
      const responseContent = `Mock response to: ${lastContent}`;
      const responseModel = model || 'mock-gpt-4';

      if (stream) {
        res.setHeader('Content-Type', 'text/event-stream');
        res.setHeader('Cache-Control', 'no-cache');
        res.setHeader('Connection', 'keep-alive');

        const chunk = {
          id: 'chatcmpl-mock123',
          object: 'chat.completion.chunk',
          created: Math.floor(Date.now() / 1000),
          model: responseModel,
          choices: [
            {
              index: 0,
              delta: { role: 'assistant', content: responseContent },
              finish_reason: null,
            },
          ],
        };
        res.write(`data: ${JSON.stringify(chunk)}\n\n`);

        const usageChunk = {
          id: 'chatcmpl-mock123',
          object: 'chat.completion.chunk',
          created: Math.floor(Date.now() / 1000),
          model: responseModel,
          choices: [
            {
              index: 0,
              delta: {},
              finish_reason: 'stop',
            },
          ],
          usage: {
            prompt_tokens: 10,
            completion_tokens: 10,
            total_tokens: 20,
          },
        };
        res.write(`data: ${JSON.stringify(usageChunk)}\n\n`);
        res.write('data: [DONE]\n\n');
        res.end();
      } else {
        res.json({
          id: 'chatcmpl-mock123',
          object: 'chat.completion',
          created: Math.floor(Date.now() / 1000),
          model: responseModel,
          choices: [
            {
              index: 0,
              message: {
                role: 'assistant',
                content: responseContent,
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
      }
    });

    // OpenAI Responses API endpoint
    this.app.post('/v1/responses', (req, res) => {
      if (this.requiresAuth && !this.hasValidAuth(req)) {
        return res.status(401).json({
          error: {
            message: 'Invalid authentication',
            type: 'invalid_request_error',
            code: 'invalid_api_key',
          },
        });
      }

      const { input, model, stream } = req.body;
      // Extract user message from input (can be string or array of message objects)
      let userMessage;
      if (typeof input === 'string') {
        userMessage = input;
      } else if (Array.isArray(input)) {
        const lastUserMsg = input.filter((i) => i.role === 'user').pop();
        userMessage =
          typeof lastUserMsg?.content === 'string'
            ? lastUserMsg.content
            : Array.isArray(lastUserMsg?.content)
              ? lastUserMsg.content
                  .filter((p) => p.type === 'input_text')
                  .map((p) => p.text)
                  .join('')
              : String(lastUserMsg?.content || input);
      } else {
        userMessage = String(input);
      }

      const responseContent = `Mock response to: ${userMessage}`;
      const responseModel = model || 'mock-gpt-4';
      const responseId = 'resp_mock123';

      if (stream) {
        res.setHeader('Content-Type', 'text/event-stream');
        res.setHeader('Cache-Control', 'no-cache');
        res.setHeader('Connection', 'keep-alive');

        // All events include 'type' matching the event name — the OpenAI SDK
        // parses the data JSON and returns it as the event object.

        const responseObj = {
          id: responseId,
          object: 'response',
          status: 'in_progress',
          model: responseModel,
          output: [],
        };
        const messageItem = {
          type: 'message',
          id: 'msg_mock1',
          role: 'assistant',
          status: 'in_progress',
          content: [],
        };
        const textPart = { type: 'output_text', text: '', annotations: [] };

        // response.created
        res.write(
          `event: response.created\ndata: ${JSON.stringify({ type: 'response.created', response: responseObj })}\n\n`
        );

        // response.output_item.added
        res.write(
          `event: response.output_item.added\ndata: ${JSON.stringify({ type: 'response.output_item.added', output_index: 0, item: messageItem })}\n\n`
        );

        // response.content_part.added
        res.write(
          `event: response.content_part.added\ndata: ${JSON.stringify({ type: 'response.content_part.added', output_index: 0, content_index: 0, part: textPart })}\n\n`
        );

        // response.output_text.delta
        res.write(
          `event: response.output_text.delta\ndata: ${JSON.stringify({ type: 'response.output_text.delta', output_index: 0, content_index: 0, delta: responseContent })}\n\n`
        );

        // response.output_text.done
        res.write(
          `event: response.output_text.done\ndata: ${JSON.stringify({ type: 'response.output_text.done', output_index: 0, content_index: 0, text: responseContent })}\n\n`
        );

        // response.content_part.done
        res.write(
          `event: response.content_part.done\ndata: ${JSON.stringify({ type: 'response.content_part.done', output_index: 0, content_index: 0, part: { ...textPart, text: responseContent } })}\n\n`
        );

        // response.output_item.done
        res.write(
          `event: response.output_item.done\ndata: ${JSON.stringify({ type: 'response.output_item.done', output_index: 0, item: { ...messageItem, status: 'completed', content: [{ type: 'output_text', text: responseContent }] } })}\n\n`
        );

        // response.completed
        const completedResponse = {
          ...responseObj,
          status: 'completed',
          output: [
            {
              ...messageItem,
              status: 'completed',
              content: [{ type: 'output_text', text: responseContent }],
            },
          ],
          usage: { input_tokens: 10, output_tokens: 10, total_tokens: 20 },
        };
        res.write(
          `event: response.completed\ndata: ${JSON.stringify({ type: 'response.completed', response: completedResponse })}\n\n`
        );

        res.end();
      } else {
        res.json({
          id: responseId,
          object: 'response',
          status: 'completed',
          model: responseModel,
          output: [
            {
              type: 'message',
              role: 'assistant',
              content: [{ type: 'output_text', text: responseContent }],
            },
          ],
          usage: { input_tokens: 10, output_tokens: 10, total_tokens: 20 },
        });
      }
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
