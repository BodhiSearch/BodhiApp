/**
 * Test fixtures for local model alias testing
 * Provides consistent test data for model alias creation and management
 */
export const LocalModelFixtures = {
  /**
   * Create test data for context parameters testing
   * @returns {Object} Test data with advanced context parameters
   */
  createContextParamsTestData() {
    const timestamp = Date.now();

    return {
      alias: `context-test-${timestamp}`,
      repo: 'bartowski/microsoft_Phi-4-mini-instruct-GGUF',
      filename: 'microsoft_Phi-4-mini-instruct-Q4_K_M.gguf',
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

  /**
   * Create test data for chat integration
   * @returns {Object} Test data specifically for chat workflow testing
   */
  createChatIntegrationTestData() {
    const timestamp = Date.now();
    return {
      alias: `chat-test-${timestamp}`,
      repo: 'bartowski/microsoft_Phi-4-mini-instruct-GGUF',
      filename: 'microsoft_Phi-4-mini-instruct-Q4_K_M.gguf',
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

  /**
   * Create comprehensive test data for complete lifecycle testing
   * Includes data for creation, editing, chat integration, and cleanup
   * @returns {Object} Complete test scenario data
   */
  createComprehensiveLifecycleData() {
    const timestamp = Date.now();
    const randomSuffix = Math.floor(Math.random() * 10000);

    return {
      // Primary alias for main lifecycle testing
      primaryAlias: {
        alias: `lifecycle-primary-${timestamp}-${randomSuffix}`,
        repo: 'bartowski/microsoft_Phi-4-mini-instruct-GGUF',
        filename: 'microsoft_Phi-4-mini-instruct-Q4_K_M.gguf',
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
        // Updated data for edit testing
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

      // Secondary alias created from existing model
      secondaryAlias: {
        alias: `lifecycle-secondary-${timestamp}-${randomSuffix}`,
        // Will be pre-populated from existing model file
        sourceModelAlias: 'bartowski/microsoft_Phi-4-mini-instruct-GGUF:Q4_K_M',
      },

      // Chat testing data
      chatTest: {
        message: 'What is 5 + 3? Please respond with only the number.',
        expectedResponse: /8/, // Expected answer
      },

      // Context parameters testing
      contextParamsTest: {
        alias: `context-test-${timestamp}-${randomSuffix}`,
        repo: 'bartowski/microsoft_Phi-4-mini-instruct-GGUF',
        filename: 'microsoft_Phi-4-mini-instruct-Q4_K_M.gguf',
        advancedParams:
          '--ctx-size 4096\n--parallel 4\n--threads 8\n--gpu-layers 20\n--rope-freq-base 10000',
      },
    };
  },

  /**
   * Create comprehensive validation test data
   * Includes all validation scenarios in one dataset
   * @returns {Object} Complete validation test scenarios
   */
  createComprehensiveValidationData() {
    const timestamp = Date.now();

    return {
      // Test data for missing required fields
      missingFields: {
        missingAlias: {
          alias: '',
          repo: 'bartowski/microsoft_Phi-4-mini-instruct-GGUF',
          filename: 'microsoft_Phi-4-mini-instruct-Q4_K_M.gguf',
        },
        missingRepo: {
          alias: `missing-repo-${timestamp}`,
          repo: '', // Empty repo should trigger validation
          filename: '', // Can't select filename without repo
        },
        missingFilename: {
          alias: `missing-filename-${timestamp}`,
          repo: 'bartowski/microsoft_Phi-4-mini-instruct-GGUF',
          filename: '', // Empty filename should trigger validation
        },
      },

      // Test data for duplicate alias validation
      duplicateTest: {
        baseAlias: `duplicate-base-${timestamp}`,
        duplicateAlias: `duplicate-base-${timestamp}`, // Same as base
        repo: 'bartowski/microsoft_Phi-4-mini-instruct-GGUF',
        filename: 'microsoft_Phi-4-mini-instruct-Q4_K_M.gguf',
      },

      // Test data for successful creation after validation
      validTest: {
        alias: `validation-test-${timestamp}`,
        repo: 'bartowski/microsoft_Phi-4-mini-instruct-GGUF',
        filename: 'microsoft_Phi-4-mini-instruct-Q4_K_M.gguf',
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
