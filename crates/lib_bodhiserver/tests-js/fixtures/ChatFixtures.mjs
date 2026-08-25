export class ChatFixtures {
  static createChatScenarios() {
    return {
      basicQuestions: [
        { input: 'What day comes after Monday?', expectedContains: 'Tuesday' },
        { input: 'What is 2+2?', expectedContains: '4' },
        {
          input: 'Tell me a fact about water',
          expectedContains: ['H2O', 'liquid', 'water', 'molecule'],
        },
        { input: 'What color is the sky?', expectedContains: ['blue', 'azure'] },
        { input: 'How many days are in a week?', expectedContains: ['7', 'seven'] },
      ],

      conversationFlows: [
        {
          name: 'Technical Discussion',
          messages: [
            { input: 'What is an API?', expectResponse: true },
            { input: 'Can you give an example?', expectResponse: true },
            { input: 'How does REST differ from GraphQL?', expectResponse: true },
          ],
        },
        {
          name: 'Math Problem Solving',
          messages: [
            { input: 'I need help with a math problem', expectResponse: true },
            { input: 'What is 15 * 24?', expectResponse: true },
            { input: 'Can you show me how to solve this step by step?', expectResponse: true },
          ],
        },
        {
          name: 'Creative Writing',
          messages: [
            { input: 'Can you help me write a story?', expectResponse: true },
            { input: 'The story should be about a robot', expectResponse: true },
            { input: 'Make it funny', expectResponse: true },
          ],
        },
      ],

      edgeCases: {
        emptyMessage: '',
        whitespaceOnly: '   \n  \t  ',
        veryLongMessage: 'x'.repeat(10000),
        specialCharacters: '!@#$%^&*(){}[]|\\:;"\'<>,.?/',
        unicodeCharacters: '🤖 こんにちは 🌟 café naïve résumé',
        multilineMessage: 'Line 1\nLine 2\nLine 3\n\nLine 5',
        codeSnippet: '```javascript\nconst x = 1;\nconsole.log(x);\n```',
        markdownContent:
          '# Header\n\n**Bold text** and *italic text*\n\n- List item 1\n- List item 2',
        htmlLikeContent: '<div>This is not HTML but looks like it</div>',
        sqlLikeContent: 'SELECT * FROM users WHERE id = 1;',
      },
    };
  }

  static getTestModels() {
    return {
      apiModels: ['gpt-4', 'gpt-3.5-turbo'],

      localModels: ['local-model-test', 'test-model-alias'],

      switchingModels: ['gpt-4', 'gpt-3.5-turbo'],

      defaultModel: 'gpt-4',
    };
  }

  static createTestChats() {
    return {
      simple: {
        title: 'Simple Test Chat',
        messages: [
          { role: 'user', content: 'Hello' },
          { role: 'assistant', content: 'Hello! How can I help you today?' },
        ],
      },

      conversation: {
        title: 'API Discussion',
        messages: [
          { role: 'user', content: 'What is an API?' },
          { role: 'assistant', content: 'An API (Application Programming Interface) is...' },
          { role: 'user', content: 'Can you give an example?' },
          { role: 'assistant', content: 'Sure! A common example is the Twitter API...' },
        ],
      },

      technical: {
        title: 'JavaScript Help',
        messages: [
          { role: 'user', content: 'How do I create a function in JavaScript?' },
          { role: 'assistant', content: 'You can create a function in JavaScript using...' },
          { role: 'user', content: 'What about arrow functions?' },
          { role: 'assistant', content: 'Arrow functions are a more concise way...' },
        ],
      },
    };
  }

  static getErrorScenarios() {
    return {
      networkErrors: {
        connectionFailed: 'Failed to fetch',
        timeout: 'Request timeout',
        serverError: 'Internal server error',
        rateLimited: 'Rate limit exceeded',
      },

      validationErrors: {
        noModel: 'No Model Selected',
        emptyMessage: 'Message cannot be empty',
        messageTooLong: 'Message is too long',
        invalidApiKey: 'Invalid API key',
      },

      authErrors: {
        unauthorized: 'Unauthorized',
        tokenExpired: 'Token expired',
        accessDenied: 'Access denied',
      },
    };
  }

  static createPerformanceScenarios() {
    return {
      longConversation: {
        messageCount: 50,
        messagesPerBatch: 5,
        delayBetweenBatches: 1000,
      },

      rapidSending: {
        messageCount: 10,
        delayBetweenMessages: 100,
        message: 'Quick test message',
      },

      largeMessage: {
        size: 5000, // characters
        content: 'Lorem ipsum '.repeat(500),
      },

      modelSwitching: {
        models: ['gpt-4', 'gpt-3.5-turbo', 'gpt-4', 'gpt-3.5-turbo'],
        messagesPerModel: 3,
        delayBetweenSwitches: 500,
      },
    };
  }

  static getResponsiveScenarios() {
    return {
      viewports: [
        { name: 'mobile', width: 375, height: 667 },
        { name: 'tablet', width: 768, height: 1024 },
        { name: 'desktop', width: 1920, height: 1080 },
        { name: 'ultrawide', width: 2560, height: 1440 },
      ],

      interactionTests: {
        sendMessage: 'Test message for responsive design',
        openSettings: true,
        switchModel: 'gpt-3.5-turbo',
        toggleHistory: true,
      },
    };
  }

  static getStreamingScenarios() {
    return {
      streamingPrompts: [
        'Write a 200 word essay about artificial intelligence',
        'Explain how machine learning works in detail',
        'Tell me a story about a robot learning to be human',
        'List 20 things I can do to improve my programming skills',
      ],

      longPrompts: [
        'Write a detailed 1000-word article about the future of technology',
        'Explain every step of the software development lifecycle',
        'Create a comprehensive guide to web development',
      ],

      quickPrompts: ['Hi', 'What is 1+1?', 'Hello', 'Yes', 'Thanks'],
    };
  }

  static getAccessibilityScenarios() {
    return {
      keyboardNavigation: {
        testSequence: [
          'Tab', // Navigate to message input
          'Hello test', // Type message
          'Enter', // Send message
          'Tab', // Navigate to next element
          'Escape', // Close any open dialogs
        ],
      },

      screenReader: {
        testElements: [
          '[data-testid="chat-input"]',
          '[data-testid="send-button"]',
          '[data-testid="user-message"]',
          '[data-testid="assistant-message"]',
        ],
      },

      focusManagement: {
        testFlow: [
          'openSettings',
          'selectModel',
          'closeSettings',
          'sendMessage',
          'openHistory',
          'selectChat',
          'closeHistory',
        ],
      },
    };
  }

  static generateTestData() {
    const timestamp = Date.now();
    const random = Math.random().toString(36).substring(7);

    return {
      uniqueMessage: `Test message ${timestamp}-${random}`,
      chatTitle: `Test Chat ${timestamp}`,
      testId: `test-${timestamp}-${random}`,
      timestamp: timestamp,
    };
  }

  static getEnvironmentData() {
    return {
      requiredEnv: ['INTEG_TEST_OPENAI_API_KEY'],

      getApiKey: () => {
        const apiKey = process.env.INTEG_TEST_OPENAI_API_KEY;
        if (!apiKey) {
          throw new Error('INTEG_TEST_OPENAI_API_KEY environment variable not set');
        }
        return apiKey;
      },

      serverConfig: {
        defaultTimeout: 30000,
        streamTimeout: 60000,
        networkTimeout: 5000,
      },
    };
  }

  static getIntegrationScenarios() {
    return {
      modelToChatFlow: {
        modelName: 'gpt-4',
        testMessage: 'Hello from models page integration',
        expectedResponse: true,
      },

      settingsPersistence: {
        settings: {
          model: 'gpt-3.5-turbo',
          streaming: false,
          temperature: 0.8,
        },
        testMessage: 'Testing settings persistence',
        reloadRequired: true,
      },

      multiChatManagement: {
        chatsToCreate: 3,
        messagesPerChat: 2,
        testNavigation: true,
        testDeletion: true,
      },
    };
  }

  static getCleanupPatterns() {
    return {
      testChatTitlePatterns: [
        /^Test Chat \d+/,
        /^test-\d+/,
        /^lifecycle-test/,
        /^mobile-test/,
        /^responsive-test/,
      ],

      testMessagePatterns: [
        /^Test message \d+/,
        /What day comes after Monday/,
        /What is 2\+2/,
        /Hello from models page integration/,
      ],

      cleanupTimeout: 5000,
    };
  }
}
