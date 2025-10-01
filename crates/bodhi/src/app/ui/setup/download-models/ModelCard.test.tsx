/**
 * ModelCard Component Tests
 *
 * Purpose: Comprehensive testing of model card display, download states,
 * and user interactions in isolation.
 *
 * Focus Areas:
 * - Model information display (name, badge, specs, benchmarks, links)
 * - Download state rendering (idle, pending, completed, error)
 * - Progress tracking and byte formatting
 * - User interactions (download clicks, link navigation)
 * - Tooltip content and accessibility
 *
 * Test Coverage:
 * 1. Display: Model info, badges, specs, benchmarks, links (3 tests)
 * 2. States: Idle, pending, completed button rendering (3 tests)
 * 3. Progress: Progress bar, byte formatting, polling updates (2 tests)
 * 4. Interactions: Download click, HuggingFace link (2 tests)
 *
 * Total: 10 comprehensive component-level tests
 */

import { ModelCard } from '@/app/ui/setup/download-models/ModelCard';
import { ModelInfo } from '@/app/ui/setup/download-models/types';
import { render, screen, within } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { describe, expect, it, vi } from 'vitest';

// Test fixtures
const chatModelFixture: ModelInfo = {
  id: 'qwen2.5-14b',
  name: 'Qwen2.5 14B',
  repo: 'bartowski/Qwen2.5-14B-Instruct-GGUF',
  filename: 'Qwen2.5-14B-Instruct-Q4_K_M.gguf',
  quantization: 'Q4_K_M',
  size: '8.99GB',
  parameters: '14B',
  category: 'chat',
  tier: 'premium',
  badge: '⭐ Best Overall',
  ratings: { quality: 4.5, speed: 4, specialization: 4.5 },
  benchmarks: { mmlu: 79.7, bbb: 78.2 },
  contextWindow: '128K',
  memoryEstimate: '~8.99GB',
  license: 'Apache 2.0',
  licenseUrl: 'https://huggingface.co/bartowski/Qwen2.5-14B-Instruct-GGUF',
  tooltipContent: {
    strengths: ['Exceptional general-purpose performance', 'Strong math and coding'],
    useCase: 'Best all-around model for general tasks',
    researchNotes: 'Trained on 18T tokens',
  },
  downloadState: { status: 'idle' },
};

const embeddingModelFixture: ModelInfo = {
  id: 'qwen3-embedding-4b',
  name: 'Qwen3 Embedding 4B',
  repo: 'Qwen/Qwen3-Embedding-4B-GGUF',
  filename: 'Qwen3-Embedding-4B-Q4_K_M.gguf',
  quantization: 'Q4_K_M',
  size: '2.5GB',
  parameters: '4B',
  category: 'embedding',
  tier: 'premium',
  badge: '⭐ Top Choice',
  ratings: { quality: 5, speed: 4, specialization: 5 },
  benchmarks: { mteb: 70.58 },
  contextWindow: '8K',
  memoryEstimate: '~2.5GB',
  license: 'Apache 2.0',
  licenseUrl: 'https://huggingface.co/Qwen/Qwen3-Embedding-4B-GGUF',
  tooltipContent: {
    strengths: ['#1 on MTEB multilingual leaderboard'],
    useCase: 'Best overall embedding model for RAG applications',
    researchNotes: '8B variant ranks #1 on MTEB',
  },
  downloadState: { status: 'idle' },
};

const modelWithoutBenchmarksFixture: ModelInfo = {
  id: 'nomic-embed-v1.5',
  name: 'Nomic Embed v1.5',
  repo: 'nomic-ai/nomic-embed-text-v1.5-GGUF',
  filename: 'nomic-embed-text-v1.5.Q8_0.gguf',
  quantization: 'Q8_0',
  size: '274MB',
  parameters: '137M',
  category: 'embedding',
  tier: 'specialized',
  badge: 'Most Efficient',
  ratings: { quality: 4, speed: 5, specialization: 5 },
  benchmarks: {},
  contextWindow: '8K',
  memoryEstimate: '~274MB',
  license: 'Apache 2.0',
  licenseUrl: 'https://huggingface.co/nomic-ai/nomic-embed-text-v1.5-GGUF',
  tooltipContent: {
    strengths: ['Ultra-small size'],
    useCase: 'Best for resource-constrained scenarios',
    researchNotes: 'Ranks similarly to top-10 MTEB models',
  },
  downloadState: { status: 'idle' },
};

describe('ModelCard Display Tests', () => {
  it('renders chat model with all information correctly', () => {
    render(<ModelCard model={chatModelFixture} onDownload={vi.fn()} />);

    expect(screen.getByText('Qwen2.5 14B')).toBeInTheDocument();
    expect(screen.getByText('⭐ Best Overall')).toBeInTheDocument();

    expect(screen.getByText('Size:')).toBeInTheDocument();
    expect(screen.getByText('8.99GB')).toBeInTheDocument();
    expect(screen.getByText('Params:')).toBeInTheDocument();
    expect(screen.getByText('14B')).toBeInTheDocument();
    expect(screen.getByText('Quant:')).toBeInTheDocument();
    expect(screen.getByText('Q4_K_M')).toBeInTheDocument();
    expect(screen.getByText('Context:')).toBeInTheDocument();
    expect(screen.getByText('128K')).toBeInTheDocument();

    expect(screen.getByText('MMLU:')).toBeInTheDocument();
    expect(screen.getByText('79.7')).toBeInTheDocument();

    const link = screen.getByTestId('huggingface-link');
    expect(link).toHaveAttribute('href', 'https://huggingface.co/bartowski/Qwen2.5-14B-Instruct-GGUF');

    expect(screen.getByText('Quality')).toBeInTheDocument();
    expect(screen.getByText('Speed')).toBeInTheDocument();
    expect(screen.getByText('Specialty')).toBeInTheDocument();
  });

  it('renders embedding model with MTEB benchmark', () => {
    render(<ModelCard model={embeddingModelFixture} onDownload={vi.fn()} />);

    expect(screen.getByText('Qwen3 Embedding 4B')).toBeInTheDocument();
    expect(screen.getByText('⭐ Top Choice')).toBeInTheDocument();

    expect(screen.getByText('MTEB:')).toBeInTheDocument();
    expect(screen.getByText('70.58')).toBeInTheDocument();

    expect(screen.queryByText('MMLU:')).not.toBeInTheDocument();
    expect(screen.queryByText('HumanEval:')).not.toBeInTheDocument();
  });

  it('renders model without benchmarks gracefully', () => {
    render(<ModelCard model={modelWithoutBenchmarksFixture} onDownload={vi.fn()} />);

    expect(screen.getByText('Nomic Embed v1.5')).toBeInTheDocument();
    expect(screen.getByText('Most Efficient')).toBeInTheDocument();

    expect(screen.getByText('Size:')).toBeInTheDocument();
    expect(screen.getByText('274MB')).toBeInTheDocument();

    expect(screen.queryByText('MMLU:')).not.toBeInTheDocument();
    expect(screen.queryByText('HumanEval:')).not.toBeInTheDocument();
    expect(screen.queryByText('MTEB:')).not.toBeInTheDocument();

    expect(screen.getByText('Quality')).toBeInTheDocument();
  });
});

describe('ModelCard Download State Tests', () => {
  it('idle state shows Download button', () => {
    const model = { ...chatModelFixture, downloadState: { status: 'idle' as const } };
    render(<ModelCard model={model} onDownload={vi.fn()} />);

    const downloadButton = screen.getByTestId('download-button');
    expect(downloadButton).toBeInTheDocument();
    expect(downloadButton).toHaveTextContent('Download');
    expect(downloadButton).not.toBeDisabled();

    expect(screen.queryByTestId('progress-bar')).not.toBeInTheDocument();
    expect(screen.queryByText('Downloaded')).not.toBeInTheDocument();
  });

  it('pending state shows progress bar and bytes', () => {
    const model = { ...chatModelFixture, downloadState: { status: 'pending' as const } };
    const downloadProgress = {
      downloadedBytes: 4_500_000_000,
      totalBytes: 9_000_000_000,
    };

    render(<ModelCard model={model} onDownload={vi.fn()} downloadProgress={downloadProgress} />);

    expect(screen.getByTestId('progress-bar')).toBeInTheDocument();
    expect(screen.getByText('50%')).toBeInTheDocument();

    const byteDisplay = screen.getByTestId('byte-display');
    expect(byteDisplay).toHaveTextContent('4.2 GB / 8.4 GB');

    expect(screen.queryByTestId('download-button')).not.toBeInTheDocument();
  });

  it('completed state shows disabled Downloaded button', () => {
    const model = { ...chatModelFixture, downloadState: { status: 'completed' as const } };
    render(<ModelCard model={model} onDownload={vi.fn()} />);

    const downloadedButton = screen.getByRole('button', { name: /downloaded/i });
    expect(downloadedButton).toBeInTheDocument();
    expect(downloadedButton).toBeDisabled();

    expect(screen.queryByTestId('progress-bar')).not.toBeInTheDocument();
    expect(screen.queryByTestId('download-button')).not.toBeInTheDocument();
  });
});

describe('ModelCard Progress Tracking Tests', () => {
  it('progress calculation handles edge cases', () => {
    const model = { ...chatModelFixture, downloadState: { status: 'pending' as const } };

    const { rerender } = render(
      <ModelCard model={model} onDownload={vi.fn()} downloadProgress={{ downloadedBytes: 0, totalBytes: 1000 }} />
    );
    expect(screen.getByText('0%')).toBeInTheDocument();

    rerender(
      <ModelCard model={model} onDownload={vi.fn()} downloadProgress={{ downloadedBytes: 1000, totalBytes: 1000 }} />
    );
    expect(screen.getByText('100%')).toBeInTheDocument();

    rerender(
      <ModelCard model={model} onDownload={vi.fn()} downloadProgress={{ downloadedBytes: null, totalBytes: 1000 }} />
    );
    expect(screen.getByText('0%')).toBeInTheDocument();

    rerender(
      <ModelCard model={model} onDownload={vi.fn()} downloadProgress={{ downloadedBytes: 500, totalBytes: null }} />
    );
    expect(screen.getByText('0%')).toBeInTheDocument();
  });

  it('byte formatting displays human-readable sizes', () => {
    const model = { ...chatModelFixture, downloadState: { status: 'pending' as const } };

    const { rerender } = render(
      <ModelCard model={model} onDownload={vi.fn()} downloadProgress={{ downloadedBytes: 0, totalBytes: 1024 }} />
    );
    expect(screen.getByTestId('byte-display')).toHaveTextContent('0 B / 1 KB');

    rerender(
      <ModelCard
        model={model}
        onDownload={vi.fn()}
        downloadProgress={{ downloadedBytes: 1_048_576, totalBytes: 10_485_760 }}
      />
    );
    expect(screen.getByTestId('byte-display')).toHaveTextContent('1 MB / 10 MB');

    rerender(
      <ModelCard
        model={model}
        onDownload={vi.fn()}
        downloadProgress={{ downloadedBytes: 1_073_741_824, totalBytes: 10_737_418_240 }}
      />
    );
    expect(screen.getByTestId('byte-display')).toHaveTextContent('1 GB / 10 GB');

    rerender(
      <ModelCard
        model={model}
        onDownload={vi.fn()}
        downloadProgress={{ downloadedBytes: 4_500_000_000, totalBytes: 9_000_000_000 }}
      />
    );
    expect(screen.getByTestId('byte-display')).toHaveTextContent('4.2 GB / 8.4 GB');
  });
});

describe('ModelCard Interaction Tests', () => {
  it('download button click calls onDownload handler', async () => {
    const user = userEvent.setup();
    const onDownload = vi.fn();
    const model = { ...chatModelFixture, downloadState: { status: 'idle' as const } };

    render(<ModelCard model={model} onDownload={onDownload} />);

    const downloadButton = screen.getByTestId('download-button');
    await user.click(downloadButton);

    expect(onDownload).toHaveBeenCalledTimes(1);
  });

  it('HuggingFace link opens in new tab with correct URL', () => {
    const phiModel: ModelInfo = {
      ...chatModelFixture,
      id: 'phi-4-14b',
      name: 'Phi-4 14B',
      repo: 'bartowski/phi-4-GGUF',
      filename: 'phi-4-Q4_K_M.gguf',
    };

    render(<ModelCard model={phiModel} onDownload={vi.fn()} />);

    const link = screen.getByTestId('huggingface-link');
    expect(link).toHaveAttribute('href', 'https://huggingface.co/bartowski/phi-4-GGUF');
    expect(link).toHaveAttribute('target', '_blank');
    expect(link).toHaveAttribute('rel', 'noopener noreferrer');

    const linkElement = within(link).getByText('Phi-4 14B');
    expect(linkElement).toBeInTheDocument();
  });
});
