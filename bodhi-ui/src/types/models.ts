export interface Model {
  alias: string;
  family?: string;
  repo: string;
  filename: string;
  snapshot: string;
  features: string[];
  chat_template: string;
  model_params: Record<string, any>;
  request_params: Record<string, any>;
  context_params: Record<string, any>;
}

export interface ModelsResponse {
  data: Model[];
  total: number;
  page: number;
  page_size: number;
}

export interface SortState {
  column: string;
  direction: 'asc' | 'desc';
}

export interface ModelFile {
  repo: string;
  filename: string;
  size?: number;
  updated_at?: string;
  snapshot: string;
  model_params: Record<string, any>;
}

export interface ModelFilesResponse {
  data: ModelFile[];
  total: number;
  page: number;
  page_size: number;
}

// New types for AliasForm

export interface CreateAliasRequest {
  alias: string;
  repo: string;
  filename: string;
  chat_template: string;
  family?: string;
  request_params?: OAIRequestParams;
  context_params?: GptContextParams;
}

export interface OAIRequestParams {
  frequency_penalty?: number;
  max_tokens?: number;
  presence_penalty?: number;
  seed?: number;
  stop?: string[];
  temperature?: number;
  top_p?: number;
  user?: string;
}

export interface GptContextParams {
  n_seed?: number;
  n_threads?: number;
  n_ctx?: number;
  n_parallel?: number;
  n_predict?: number;
  n_keep?: number;
}

export interface ChatTemplate {
  id: string;
  name: string;
}