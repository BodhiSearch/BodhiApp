import { SettingsSidebar } from '@/app/ui/chat/settings/SettingsSidebar';
import { ENDPOINT_MODELS } from '@/hooks/useQuery';
import { createWrapper } from '@/tests/wrapper';
import { render, screen, waitFor } from '@testing-library/react';
import { afterAll, beforeAll, beforeEach, describe, expect, it, vi } from 'vitest';
import { setupMswV2, server } from '@/test-utils/msw-v2/setup';
import { mockModels, mockModelsError, mockModelsEmpty } from '@/test-utils/msw-v2/handlers/models';

// Mock the child components
vi.mock('@/app/ui/chat/settings/AliasSelector', () => ({
  AliasSelector: ({ models, isLoading }: { models: any[]; isLoading: boolean }) => (
    <div data-testid="alias-selector" data-loading={isLoading}>
      Models count: {models.length}
    </div>
  ),
}));

vi.mock('@/app/ui/chat/settings/SystemPrompt', () => ({
  SystemPrompt: ({ isLoading }: { isLoading: boolean }) => <div data-testid="system-prompt" data-loading={isLoading} />,
}));

vi.mock('@/app/ui/chat/settings/StopWords', () => ({
  StopWords: ({ isLoading }: { isLoading: boolean }) => <div data-testid="stop-words" data-loading={isLoading} />,
}));

vi.mock('@/app/ui/chat/settings/SettingSlider', () => ({
  SettingSlider: ({ label, isLoading }: { label: string; isLoading: boolean }) => (
    <div data-testid={`setting-slider-${label.toLowerCase().replace(' ', '-')}`} data-loading={isLoading} />
  ),
}));

// Mock UI components
vi.mock('@/components/ui/sidebar', () => ({
  Sidebar: ({ children }: { children: React.ReactNode }) => <div data-testid="sidebar">{children}</div>,
  SidebarHeader: ({ children }: { children: React.ReactNode }) => <div data-testid="sidebar-header">{children}</div>,
  SidebarContent: ({ children }: { children: React.ReactNode }) => <div data-testid="sidebar-content">{children}</div>,
  SidebarGroup: ({ children }: { children: React.ReactNode }) => <div data-testid="sidebar-group">{children}</div>,
}));

vi.mock('@/components/ui/switch', () => ({
  Switch: ({ id }: { id: string }) => <div data-testid={`switch-${id}`} />,
}));

vi.mock('@/components/ui/label', () => ({
  Label: ({ children }: { children: React.ReactNode }) => <div data-testid="label">{children}</div>,
}));

vi.mock('@/components/ui/separator', () => ({
  Separator: () => <div data-testid="separator" />,
}));

// Mock hooks
vi.mock('@/hooks/use-chat-settings', () => ({
  useChatSettings: () => ({
    stream: false,
    setStream: vi.fn(),
    seed: 0,
    seed_enabled: false,
    setSeed: vi.fn(),
    setSeedEnabled: vi.fn(),
    temperature: 1,
    temperature_enabled: true,
    setTemperature: vi.fn(),
    setTemperatureEnabled: vi.fn(),
    top_p: 1,
    top_p_enabled: true,
    setTopP: vi.fn(),
    setTopPEnabled: vi.fn(),
    max_tokens: 2048,
    max_tokens_enabled: true,
    setMaxTokens: vi.fn(),
    setMaxTokensEnabled: vi.fn(),
    presence_penalty: 0,
    presence_penalty_enabled: true,
    setPresencePenalty: vi.fn(),
    setPresencePenaltyEnabled: vi.fn(),
    frequency_penalty: 0,
    frequency_penalty_enabled: true,
    setFrequencyPenalty: vi.fn(),
    setFrequencyPenaltyEnabled: vi.fn(),
  }),
}));

// Mock the Tooltip components from shadcn
vi.mock('@/components/ui/tooltip', () => ({
  Tooltip: ({ children }: { children: React.ReactNode }) => <div>{children}</div>,
  TooltipContent: ({ children }: { children: React.ReactNode }) => <div data-testid="tooltip-content">{children}</div>,
  TooltipProvider: ({ children }: { children: React.ReactNode }) => <div>{children}</div>,
  TooltipTrigger: ({ children }: { children: React.ReactNode }) => <div data-testid="tooltip-trigger">{children}</div>,
}));

// Setup MSW v2 server
setupMswV2();

// Default handler for models endpoint used by all tests
beforeEach(() => {
  server.use(
    ...mockModels({
      data: [],
      total: 0,
      page: 1,
      page_size: 100,
    })
  );
});

describe('SettingsSidebar', () => {
  it('renders the sidebar structure correctly', () => {
    render(<SettingsSidebar />, {
      wrapper: createWrapper(),
    });

    expect(screen.getByTestId('sidebar')).toBeInTheDocument();
    expect(screen.getByTestId('sidebar-header')).toBeInTheDocument();
    expect(screen.getByTestId('sidebar-content')).toBeInTheDocument();
    expect(screen.getByTestId('sidebar-group')).toBeInTheDocument();
    expect(screen.getByText('Settings')).toBeInTheDocument();
  });

  it('renders all settings components with loading state', () => {
    render(<SettingsSidebar />, {
      wrapper: createWrapper(),
    });

    const aliasSelector = screen.getByTestId('alias-selector');
    const systemPrompt = screen.getByTestId('system-prompt');
    const stopWords = screen.getByTestId('stop-words');

    // Check new components
    const streamSwitch = screen.getByTestId('switch-stream-mode');
    const seedSwitch = screen.getByTestId('switch-seed-enabled');
    const temperatureSlider = screen.getByTestId('setting-slider-temperature');
    const topPSlider = screen.getByTestId('setting-slider-top-p');
    const maxTokensSlider = screen.getByTestId('setting-slider-max-tokens');
    const presencePenaltySlider = screen.getByTestId('setting-slider-presence-penalty');
    const frequencyPenaltySlider = screen.getByTestId('setting-slider-frequency-penalty');
    const separator = screen.getByTestId('separator');

    expect(aliasSelector).toHaveAttribute('data-loading', 'true');
    expect(systemPrompt).toHaveAttribute('data-loading', 'true');
    expect(stopWords).toHaveAttribute('data-loading', 'true');
    expect(streamSwitch).toBeInTheDocument();
    expect(seedSwitch).toBeInTheDocument();
    expect(temperatureSlider).toHaveAttribute('data-loading', 'true');
    expect(topPSlider).toHaveAttribute('data-loading', 'true');
    expect(maxTokensSlider).toHaveAttribute('data-loading', 'true');
    expect(presencePenaltySlider).toHaveAttribute('data-loading', 'true');
    expect(frequencyPenaltySlider).toHaveAttribute('data-loading', 'true');
    expect(separator).toBeInTheDocument();
  });

  it('passes models data to AliasSelector after loading', async () => {
    server.use(
      ...mockModels({
        data: [
          {
            id: 1,
            name: 'Model 1',
            alias: 'model-1',
            source: 'user',
            repo: 'test-repo',
            filename: 'test-file.bin',
            snapshot: 'abc123',
            request_params: {},
            context_params: [],
          },
          {
            id: 2,
            name: 'Model 2',
            alias: 'model-2',
            source: 'user',
            repo: 'test-repo',
            filename: 'test-file.bin',
            snapshot: 'abc123',
            request_params: {},
            context_params: [],
          },
        ],
        total: 2,
      })
    );

    render(<SettingsSidebar />, {
      wrapper: createWrapper(),
    });

    const aliasSelector = screen.getByTestId('alias-selector');

    expect(aliasSelector).toHaveAttribute('data-loading', 'true');

    await waitFor(() => {
      expect(aliasSelector).toHaveAttribute('data-loading', 'false');
      expect(aliasSelector).toHaveTextContent('Models count: 2');
    });
  });

  it('handles API error gracefully', async () => {
    server.use(
      ...mockModelsError({
        status: 500,
        message: 'Test error message',
      })
    );

    render(<SettingsSidebar />, {
      wrapper: createWrapper(),
    });

    const aliasSelector = screen.getByTestId('alias-selector');

    await waitFor(() => {
      expect(aliasSelector).toHaveAttribute('data-loading', 'false');
      expect(aliasSelector).toHaveTextContent('Models count: 0');
    });
  });

  it('handles empty models response', async () => {
    server.use(...mockModelsEmpty());

    render(<SettingsSidebar />, {
      wrapper: createWrapper(),
    });

    const aliasSelector = screen.getByTestId('alias-selector');

    await waitFor(() => {
      expect(aliasSelector).toHaveAttribute('data-loading', 'false');
      expect(aliasSelector).toHaveTextContent('Models count: 0');
    });
  });
});
