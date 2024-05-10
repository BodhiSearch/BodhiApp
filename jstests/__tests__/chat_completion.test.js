const os = require('os');
const path = require('path');
const { spawn } = require('child_process');
const { OpenAI } = require('openai');
const MEMBER_NAME = "bodhi";
const APP_NAME = "bodhicli";

describe(`run ${APP_NAME}`, () => {
  let server;
  let openai;
  beforeAll(async () => {
    await new Promise((resolve, reject) => {
      const buildProcess = spawn('cargo', ['build', '-p', MEMBER_NAME, '--bin', APP_NAME]);
      buildProcess.stderr.on('data', (data) => {
        console.log(`stderr: ${data}`);
      });
      buildProcess.on('exit', (code) => {
        if (code === 0) {
          resolve();
        } else {
          reject(new Error('Failed to build app'));
        }
      });
    });
    await new Promise((resolve, reject) => {
      console.log(`starting the server`);
      let model_path = path.join(os.homedir(), '.cache/huggingface/hub/models--TheBloke--Llama-2-7B-Chat-GGUF/snapshots/08a5566d61d7cb6b420c3e4387a39e0078e1f2fe5f055f3a03887385304d4bfa/llama-2-7b-chat.Q4_K_M.gguf');
      server = spawn(`../target/debug/${APP_NAME}`, ['serve', '-m', model_path]);
      let timeout = setTimeout(() => {
        reject(new Error('time out waiting for server to start'));
      }, 10_000);
      server.stdout.on('data', (data) => {
        let output = Buffer.from(data, 'utf-8').toString().trim();
        if (output.includes('server started')) {
          clearTimeout(timeout);
          resolve();
        }
      });
    });
    openai = new OpenAI({
      apiKey: 'sk-dummy-key',
      baseURL: 'http://127.0.0.1:1135/v1',
    });
  });

  afterAll(async () => {
    if (server) {
      server.kill('SIGTERM');
      await new Promise(resolve => server.on('exit', resolve));
    }
  });

  it('should fetch chat completion', async () => {
    const chatCompletion = await openai.chat.completions.create({
      model: 'TheBloke/Llama-2-7B-Chat-GGUF:llama-2-7b-chat.Q8_0.gguf',
      seed: 42,
      messages: [
        { role: 'assistant', content: 'you are a helpful assistant' },
        { role: 'user', content: 'What day comes after Monday?' }
      ]
    });
    expect(chatCompletion.choices[0].message.content).toBe('Tuesday comes after Monday.');
  });

  it('should fetch chat completion stream', async () => {
    const chatCompletion = await openai.chat.completions.create({
      model: 'TheBloke/Llama-2-7B-Chat-GGUF:llama-2-7b-chat.Q8_0.gguf',
      stream: true,
      seed: 42,
      messages: [
        { role: 'system', content: 'You are a helpful assistant.' },
        { role: 'user', content: 'List down all the days of the week.' }
      ]
    });

    let content = '';
    for await (const chunk of chatCompletion) {
      content += chunk.choices[0]?.delta?.content || '';
    }
    let expected = `Sure! Here are the 7 days of the week:

1. Monday
2. Tuesday
3. Wednesday
4. Thursday
5. Friday
6. Saturday
7. Sunday

I hope that helps! Let me know if you have any other questions.`;
    expect(content).toEqual(expected);
  });
})