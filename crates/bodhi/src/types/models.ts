export type AppStatus = 'setup' | 'ready' | 'resource-admin';
export interface Page {
  title: string;
  url: string;
  iconName: string;
}

export interface AppInfo {
  status: AppStatus;
  version: string;
}

export interface ErrorResponse {
  error: ApiError;
}

export interface ApiError {
  message: string;
  type: string;
  param?: string;
  code?: string;
}

export interface FeaturedModel {
  name: string;
}

export interface Model {
  alias: string;
  repo: string;
  filename: string;
  snapshot: string;
  source?: string;
  request_params: OAIRequestParams;
  context_params: GptContextParams;
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

export interface UserInfo {
  logged_in: boolean;
  email?: string;
  roles: string[];
}

export interface Setting {
  key: string;
  current_value: string | number | boolean;
  default_value: string | number | boolean;
  source: 'system' | 'command_line' | 'environment' | 'settings_file' | 'default';
  metadata: {
    type: 'string' | 'number' | 'boolean' | 'option';
    options?: string[];
    range?: {
      min: number;
      max: number;
    };
  };
}
