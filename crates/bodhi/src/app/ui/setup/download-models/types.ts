// Model catalog - static model information (future: fetch from API)
export interface ModelCatalog {
  id: string;
  name: string;
  repo: string;
  filename: string;
  quantization: string;
  size: string;
  parameters: string;
  category: 'chat' | 'embedding';
  tier?: 'premium' | 'specialized';
  badge?: string;
  ratings: {
    quality: number;
    speed: number;
    specialization: number;
  };
  benchmarks: {
    mmlu?: number;
    humanEval?: number;
    bbb?: number;
    mteb?: number;
  };
  contextWindow: string;
  memoryEstimate: string;
  license: string;
  licenseUrl: string;
  tooltipContent: {
    strengths: string[];
    useCase: string;
    researchNotes: string;
  };
}

// Model info with runtime download state
export interface ModelInfo extends ModelCatalog {
  downloadState: DownloadState;
}

export interface DownloadState {
  status: 'idle' | 'pending' | 'completed' | 'error';
  progress?: number;
  speed?: string;
  timeRemaining?: string;
}

export const containerVariants = {
  hidden: { opacity: 0 },
  visible: {
    opacity: 1,
    transition: { staggerChildren: 0.1 },
  },
};

export const itemVariants = {
  hidden: { y: 20, opacity: 0 },
  visible: { y: 0, opacity: 1 },
};
