import { render, screen } from '@testing-library/react';
import { describe, expect, it, vi } from 'vitest';
import { ModelCard } from '@/app/ui/setup/download-models/ModelCard';
import { DownloadState, ModelInfo } from '@/app/ui/setup/download-models/types';

// Base model info to be extended for different test cases
const baseModel: ModelInfo = {
  id: 'test-model',
  name: 'Test Model',
  repo: 'test/repo',
  filename: 'test.gguf',
  category: 'small',
  size: '4GB',
  parameters: '7B',
  license: 'Apache 2.0',
  quantization: 'Q4_K_M',
  ratings: {
    quality: 4,
    speed: 3,
    accuracy: 5,
  },
  downloadState: { status: 'idle' } as DownloadState,
};

describe('ModelCard', () => {
  const handleDownload = vi.fn();

  it('should render card with idle download state', () => {
    const model = {
      ...baseModel,
    };

    render(<ModelCard model={model} onDownload={handleDownload} />);

    expect(screen.getByText('Download Model')).toBeInTheDocument();
    expect(screen.getByRole('button')).not.toBeDisabled();
  });

  it('should render card with completed download state', () => {
    const model = {
      ...baseModel,
      downloadState: { status: 'completed' } as DownloadState,
    };

    render(<ModelCard model={model} onDownload={handleDownload} />);

    expect(screen.getByText('Download Complete')).toBeInTheDocument();
    expect(screen.getByRole('button')).toBeDisabled();
  });

  it('should render card with pending download state and progress', () => {
    const model = {
      ...baseModel,
      downloadState: {
        status: 'pending' as const,
        progress: 45,
        speed: '2.5 MB/s',
        timeRemaining: '2 minutes',
      },
    };

    render(<ModelCard model={model} onDownload={handleDownload} />);

    expect(screen.getByText('Downloading')).toBeInTheDocument();
    expect(screen.getByText(/45%/)).toBeInTheDocument();
    expect(screen.getByText(/2.5 MB\/s/)).toBeInTheDocument();
    expect(screen.getByText(/2 minutes remaining/)).toBeInTheDocument();
  });

  it('should render card with error download state', () => {
    const model = {
      ...baseModel,
      downloadState: { status: 'error', message: 'Download failed' } as DownloadState,
    };

    render(<ModelCard model={model} onDownload={handleDownload} />);

    expect(screen.getByText('Download Model')).toBeInTheDocument();
    expect(screen.getByRole('button')).not.toBeDisabled();
  });

  it('should display basic model information', () => {
    render(<ModelCard model={baseModel} onDownload={handleDownload} />);

    expect(screen.getByText('Test Model')).toBeInTheDocument();
    expect(screen.getByText('test/repo')).toBeInTheDocument();
    expect(screen.getByText('4GB')).toBeInTheDocument();
    expect(screen.getByText('7B')).toBeInTheDocument();
    expect(screen.getByText('Apache 2.0')).toBeInTheDocument();
    expect(screen.getByText('Q4_K_M')).toBeInTheDocument();
  });
}); 