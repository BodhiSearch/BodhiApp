export interface ModelInfo {
  id: string;
  name: string;
  repo: string;
  filename: string;
  quantization: string;
  size: string;
  parameters: string;
  category: 'small' | 'medium' | 'large' | 'extra-large';
  ratings: {
    quality: number;
    speed: number;
    accuracy: number;
  };
  license: string;
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
