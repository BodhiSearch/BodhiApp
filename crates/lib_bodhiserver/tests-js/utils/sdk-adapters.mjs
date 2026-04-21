// Per-format adapters that wrap the official provider SDKs with a uniform
// surface: { chat, toolCall, listModels, embed }. Each BodhiApp proxy endpoint
// is driven through the SDK its upstream provider ships, so compatibility is
// validated end-to-end. All adapters assume the token is a BodhiApp bearer
// (either `bodhiapp_...` API token or an OAuth JWT); the per-format auth
// middleware on the server rewrites SDK-native auth headers (x-api-key,
// x-goog-api-key) to `Authorization: Bearer`.
//
// SDK reference:
//   openai@6          — chat.completions, responses, embeddings, models
//   @anthropic-ai/sdk — messages.create / messages.stream, models.list
//   @google/genai     — models.generateContent / .generateContentStream
//                       / .embedContent / .list

import Anthropic from '@anthropic-ai/sdk';
import { GoogleGenAI } from '@google/genai';
import OpenAI from 'openai';

const CHAT_QUESTION = 'What day comes after Monday?';
const TOOL_QUESTION = "What's the weather in San Francisco?";
const GET_WEATHER_PARAMS = {
  type: 'object',
  properties: {
    location: {
      type: 'string',
      description: 'City and state, e.g., San Francisco, CA',
    },
  },
  required: ['location'],
};

// state shape (populated by the setup test in api-sdk-compat.spec.mjs):
//   {
//     models: { [formatKey]: { modelId, effectiveModel } },
//     embeddingAliases: { openai: effectiveEmbedModel, gemini: effectiveEmbedModel },
//   }
export function buildSdkAdapter(format, serverUrl, token, state) {
  switch (format) {
    case 'openai':
      return buildOpenAIAdapter(serverUrl, token, state, { responses: false });
    case 'openai_responses':
      return buildOpenAIAdapter(serverUrl, token, state, { responses: true });
    case 'anthropic':
      return buildAnthropicAdapter(serverUrl, token, state);
    case 'gemini':
      return buildGeminiAdapter(serverUrl, token, state);
    default:
      throw new Error(`unknown format: ${format}`);
  }
}

// ---------- OpenAI (chat.completions + responses) ----------

function buildOpenAIAdapter(serverUrl, token, state, { responses }) {
  const client = new OpenAI({ apiKey: token, baseURL: `${serverUrl}/v1` });
  const chatModel = responses
    ? state.models.openai_responses.effectiveModel
    : state.models.openai.effectiveModel;

  return {
    async chat({ stream }) {
      if (responses) {
        if (stream) {
          const s = await client.responses.create({
            model: chatModel,
            input: CHAT_QUESTION,
            stream: true,
          });
          let text = '';
          let chunkCount = 0;
          for await (const event of s) {
            chunkCount += 1;
            if (event?.type === 'response.output_text.delta' && event.delta) {
              text += event.delta;
            }
          }
          return { text, chunkCount };
        }
        const resp = await client.responses.create({ model: chatModel, input: CHAT_QUESTION });
        const text = resp.output_text ?? extractResponsesText(resp);
        return { text, chunkCount: 1 };
      }

      if (stream) {
        const s = await client.chat.completions.create({
          model: chatModel,
          messages: [{ role: 'user', content: CHAT_QUESTION }],
          stream: true,
        });
        let text = '';
        let chunkCount = 0;
        for await (const chunk of s) {
          chunkCount += 1;
          const delta = chunk.choices?.[0]?.delta?.content ?? '';
          if (delta) text += delta;
        }
        return { text, chunkCount };
      }

      const resp = await client.chat.completions.create({
        model: chatModel,
        messages: [{ role: 'user', content: CHAT_QUESTION }],
      });
      return { text: resp.choices?.[0]?.message?.content ?? '', chunkCount: 1 };
    },

    async toolCall() {
      if (responses) {
        const resp = await client.responses.create({
          model: chatModel,
          input: TOOL_QUESTION,
          tools: [{ type: 'function', name: 'get_weather', parameters: GET_WEATHER_PARAMS }],
        });
        const fn = (resp.output ?? []).find((item) => item.type === 'function_call');
        if (!fn)
          throw new Error(`no function_call in responses output: ${JSON.stringify(resp.output)}`);
        return { toolName: fn.name, toolArgs: JSON.parse(fn.arguments ?? '{}') };
      }

      const resp = await client.chat.completions.create({
        model: chatModel,
        messages: [{ role: 'user', content: TOOL_QUESTION }],
        tools: [
          {
            type: 'function',
            function: {
              name: 'get_weather',
              description: 'Get current weather for a location',
              parameters: GET_WEATHER_PARAMS,
            },
          },
        ],
      });
      const call = resp.choices?.[0]?.message?.tool_calls?.[0];
      if (!call)
        throw new Error(`no tool_call in chat completion: ${JSON.stringify(resp.choices)}`);
      return {
        toolName: call.function.name,
        toolArgs: JSON.parse(call.function.arguments ?? '{}'),
      };
    },

    async listModels() {
      const ids = [];
      const page = await client.models.list();
      for await (const m of page) ids.push(m.id);
      return ids;
    },

    async embed(text) {
      if (responses) {
        throw new Error('embeddings are unsupported for openai_responses format');
      }
      const embedModel = state.embeddingAliases.openai;
      const resp = await client.embeddings.create({ model: embedModel, input: text });
      const vector = resp.data?.[0]?.embedding ?? [];
      return { dimensions: vector.length, vector };
    },
  };
}

function extractResponsesText(resp) {
  if (!Array.isArray(resp.output)) return '';
  for (const item of resp.output) {
    if (Array.isArray(item.content)) {
      for (const c of item.content) {
        if (c.text) return c.text;
      }
    }
  }
  return '';
}

// ---------- Anthropic ----------

function buildAnthropicAdapter(serverUrl, token, state) {
  // baseURL root; SDK adds `/v1/messages`, `/v1/models` automatically.
  const client = new Anthropic({ apiKey: token, baseURL: `${serverUrl}/anthropic` });
  const chatModel = state.models.anthropic.effectiveModel;

  return {
    async chat({ stream }) {
      if (stream) {
        const s = client.messages.stream({
          model: chatModel,
          max_tokens: 128,
          messages: [{ role: 'user', content: CHAT_QUESTION }],
        });
        let chunkCount = 0;
        for await (const _ of s) chunkCount += 1;
        const final = await s.finalMessage();
        const text = (final.content ?? [])
          .filter((b) => b.type === 'text')
          .map((b) => b.text)
          .join('');
        return { text, chunkCount };
      }
      const resp = await client.messages.create({
        model: chatModel,
        max_tokens: 128,
        messages: [{ role: 'user', content: CHAT_QUESTION }],
      });
      const text = (resp.content ?? [])
        .filter((b) => b.type === 'text')
        .map((b) => b.text)
        .join('');
      return { text, chunkCount: 1 };
    },

    async toolCall() {
      const resp = await client.messages.create({
        model: chatModel,
        max_tokens: 256,
        tools: [
          {
            name: 'get_weather',
            description: 'Get current weather for a location',
            input_schema: GET_WEATHER_PARAMS,
          },
        ],
        messages: [{ role: 'user', content: TOOL_QUESTION }],
      });
      const use = (resp.content ?? []).find((b) => b.type === 'tool_use');
      if (!use) throw new Error(`no tool_use block: ${JSON.stringify(resp.content)}`);
      return { toolName: use.name, toolArgs: use.input ?? {} };
    },

    async listModels() {
      const page = await client.models.list();
      const ids = [];
      for await (const m of page) ids.push(m.id ?? m.name ?? '');
      return ids;
    },

    async embed(_text) {
      throw new Error('Anthropic has no public embeddings API');
    },
  };
}

// ---------- Gemini (@google/genai) ----------

function buildGeminiAdapter(serverUrl, token, state) {
  // SDK concatenates baseUrl + /{apiVersion}; keep baseUrl at the server root.
  const ai = new GoogleGenAI({
    apiKey: token,
    httpOptions: { baseUrl: serverUrl, apiVersion: 'v1beta' },
  });
  const chatModel = state.models.gemini.effectiveModel;

  return {
    async chat({ stream }) {
      const contents = [{ role: 'user', parts: [{ text: CHAT_QUESTION }] }];
      if (stream) {
        const s = await ai.models.generateContentStream({ model: chatModel, contents });
        let text = '';
        let chunkCount = 0;
        for await (const chunk of s) {
          chunkCount += 1;
          if (chunk.text) text += chunk.text;
        }
        return { text, chunkCount };
      }
      const resp = await ai.models.generateContent({ model: chatModel, contents });
      return { text: resp.text ?? '', chunkCount: 1 };
    },

    async toolCall() {
      const resp = await ai.models.generateContent({
        model: chatModel,
        contents: [{ role: 'user', parts: [{ text: TOOL_QUESTION }] }],
        config: {
          tools: [
            {
              functionDeclarations: [
                {
                  name: 'get_weather',
                  description: 'Get current weather for a location',
                  parameters: GET_WEATHER_PARAMS,
                },
              ],
            },
          ],
          // Force a function call — Gemini otherwise sometimes replies with a
          // clarifying text ("Which state?") rather than calling the tool,
          // making the test non-deterministic across runs.
          toolConfig: {
            functionCallingConfig: { mode: 'ANY', allowedFunctionNames: ['get_weather'] },
          },
        },
      });
      // @google/genai v1.50 exposes `functionCalls` as a getter but in some
      // response shapes it may be empty; walk `candidates[].content.parts[]`
      // directly to find a `functionCall` entry as the canonical fallback.
      const parts = resp.candidates?.[0]?.content?.parts ?? [];
      const viaParts = parts
        .map((p) => p.functionCall)
        .filter((fc) => fc && fc.name);
      const calls = (resp.functionCalls?.length ? resp.functionCalls : viaParts) ?? [];
      if (calls.length === 0) {
        throw new Error(
          `no functionCalls in gemini response; parts=${JSON.stringify(parts).slice(0, 400)}`
        );
      }
      return { toolName: calls[0].name, toolArgs: calls[0].args ?? {} };
    },

    async listModels() {
      const pager = await ai.models.list();
      const ids = [];
      for await (const m of pager) ids.push(m.name ?? '');
      return ids;
    },

    async embed(text) {
      const embedModel = state.embeddingAliases.gemini;
      const resp = await ai.models.embedContent({
        model: embedModel,
        contents: [{ parts: [{ text }] }],
      });
      const vector = resp.embeddings?.[0]?.values ?? [];
      return { dimensions: vector.length, vector };
    },
  };
}
