export interface DownloadRequest {
  id: string;
  repo: string;
  filename: string;
  status: 'pending' | 'completed' | 'error';
  error?: string;
  updated_at: string;
}

export interface ListDownloadsResponse {
  data: DownloadRequest[];
  total: number;
  page: number;
  page_size: number;
}

export interface PullModelRequest {
  repo: string;
  filename: string;
}

export interface RedirectResponse {
  location: string;
}
