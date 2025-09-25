import { ApiFormat } from '@bodhiapp/ts-client';

export interface ApiProvider {
  id: string;
  name: string;
  format: ApiFormat;
  baseUrl: string;
  description: string;
  docUrl: string;
  commonModels: string[];
  icon: string;
  category: 'popular' | 'compatible';
}

export const API_PROVIDERS: ApiProvider[] = [
  {
    id: 'openai',
    name: 'OpenAI',
    format: 'openai' as ApiFormat,
    baseUrl: 'https://api.openai.com/v1',
    description: 'Access GPT-4, GPT-3.5, and other OpenAI models',
    docUrl: 'https://platform.openai.com/api-keys',
    commonModels: ['gpt-4', 'gpt-4-turbo-preview', 'gpt-3.5-turbo'],
    icon: '🤖',
    category: 'popular',
  },
  {
    id: 'openai-compatible',
    name: 'OpenAI Compatible',
    format: 'openai' as ApiFormat,
    baseUrl: '',
    description: 'Use any OpenAI-compatible API endpoint',
    docUrl: '',
    commonModels: [],
    icon: '🔌',
    category: 'compatible',
  },
];

export const DEFAULT_TEST_PROMPT = 'Respond with "test successful"';

export const PROVIDER_BENEFITS = [
  {
    icon: '🚀',
    title: 'Latest Models',
    description: "Access cutting-edge AI models as soon as they're released",
  },
  {
    icon: '⚡',
    title: 'Lower Hardware Requirements',
    description: 'Run powerful models without requiring high-end local hardware',
  },
  {
    icon: '🔄',
    title: 'Best of Both Worlds',
    description: 'Combine local privacy with cloud performance for different use cases',
  },
  {
    icon: '💰',
    title: 'Pay As You Go',
    description: 'Only pay for what you use, no upfront hardware investment',
  },
];

export const API_KEY_SECURITY_NOTES = [
  'API keys are stored securely and encrypted',
  'Keys are never transmitted in plain text',
  'You can revoke access anytime from your provider dashboard',
  'Only you have access to your configured models',
];
