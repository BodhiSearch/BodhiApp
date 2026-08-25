export const QWEN_MODEL = {
  repo: 'ggml-org/Qwen3-1.7B-GGUF',
  filename: 'Qwen3-1.7B-Q8_0.gguf',
  alias: 'ggml-org/Qwen3-1.7B-GGUF:Q8_0',
};

export const LocalModelFixtures = {
  createContextParamsTestData() {
    const timestamp = Date.now();

    return {
      alias: `context-test-${timestamp}`,
      repo: QWEN_MODEL.repo,
      filename: QWEN_MODEL.filename,
      contextParams:
        '--ctx-size 4096\n--parallel 4\n--threads 8\n--gpu-layers 20\n--rope-freq-base 10000',
      requestParams: {
        temperature: 0.6,
        max_tokens: 1024,
        top_p: 0.9,
        seed: 123,
      },
    };
  },

  createChatIntegrationTestData() {
    const timestamp = Date.now();
    return {
      alias: `chat-test-${timestamp}`,
      repo: QWEN_MODEL.repo,
      filename: QWEN_MODEL.filename,
      contextParams: '--ctx-size 2048\n--parallel 1', // Minimal for fast testing
      requestParams: {
        temperature: 0.1, // Low temperature for more deterministic responses
        max_tokens: 100, // Small token limit for fast responses
        top_p: 0.95,
      },
      message: 'What is 5 + 3? Please respond with only the number.',
      expectedResponse: /8/, // Case-insensitive regex for response verification
    };
  },

  createComprehensiveLifecycleData() {
    const timestamp = Date.now();
    const randomSuffix = Math.floor(Math.random() * 10000);

    return {
      primaryAlias: {
        alias: `lifecycle-primary-${timestamp}-${randomSuffix}`,
        repo: QWEN_MODEL.repo,
        filename: QWEN_MODEL.filename,
        contextParams: '--ctx-size 4096\n--parallel 4\n--threads 8',
        requestParams: {
          temperature: 0.7,
          max_tokens: 2048,
          top_p: 0.95,
          seed: 42,
          stop: ['</thinking>', '<|end|>'],
          frequency_penalty: 0.1,
          presence_penalty: 0.05,
          user: 'test-user',
        },
        updatedData: {
          contextParams: '--ctx-size 8192\n--parallel 2\n--threads 4\n--gpu-layers 20',
          requestParams: {
            temperature: 0.8,
            max_tokens: 1024,
            top_p: 0.9,
            frequency_penalty: 0.2,
            presence_penalty: 0.1,
          },
        },
      },

      secondaryAlias: {
        alias: `lifecycle-secondary-${timestamp}-${randomSuffix}`,
        // Will be pre-populated from existing model file
        sourceModelAlias: QWEN_MODEL.alias,
      },

      chatTest: {
        message: 'What is 5 + 3? Please respond with only the number.',
        expectedResponse: /8/,
      },

      contextParamsTest: {
        alias: `context-test-${timestamp}-${randomSuffix}`,
        repo: QWEN_MODEL.repo,
        filename: QWEN_MODEL.filename,
        advancedParams:
          '--ctx-size 4096\n--parallel 4\n--threads 8\n--gpu-layers 20\n--rope-freq-base 10000',
      },
    };
  },

  createComprehensiveValidationData() {
    const timestamp = Date.now();

    return {
      missingFields: {
        missingAlias: {
          alias: '',
          repo: QWEN_MODEL.repo,
          filename: QWEN_MODEL.filename,
        },
        missingRepo: {
          alias: `missing-repo-${timestamp}`,
          repo: '', // Empty repo should trigger validation
          filename: '', // Can't select filename without repo
        },
        missingFilename: {
          alias: `missing-filename-${timestamp}`,
          repo: QWEN_MODEL.repo,
          filename: '', // Empty filename should trigger validation
        },
      },

      duplicateTest: {
        baseAlias: `duplicate-base-${timestamp}`,
        duplicateAlias: `duplicate-base-${timestamp}`,
        repo: QWEN_MODEL.repo,
        filename: QWEN_MODEL.filename,
      },

      validTest: {
        alias: `validation-test-${timestamp}`,
        repo: QWEN_MODEL.repo,
        filename: QWEN_MODEL.filename,
        contextParams: '--ctx-size 2048\n--parallel 2',
        requestParams: {
          temperature: 0.5,
          max_tokens: 512,
          top_p: 0.8,
        },
      },
    };
  },
};
