import ApiModelForm from '@/components/api-models/ApiModelForm';
import { mockApiFormats } from '@/test-utils/msw-v2/handlers/api-models';
import { server, setupMswV2 } from '@/test-utils/msw-v2/setup';
import { createWrapper } from '@/tests/wrapper';
import type { ReactNode } from 'react';
import { act, render, screen, waitFor, within } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { afterEach, beforeAll, beforeEach, describe, expect, it, vi } from 'vitest';

const navigateMock = vi.fn();
vi.mock('@tanstack/react-router', async () => {
  const actual = await vi.importActual('@tanstack/react-router');
  return {
    ...actual,
    Link: ({ to, children, ...rest }: { to: string; children: ReactNode; [key: string]: unknown }) => (
      <a href={to} {...rest}>
        {children}
      </a>
    ),
    useNavigate: () => navigateMock,
  };
});

const mockToast = vi.fn();
vi.mock('@/hooks/use-toast', () => ({
  useToast: () => ({ toast: mockToast, dismiss: () => {} }),
}));

vi.mock('@/components/ui/toaster', () => ({
  Toaster: () => null,
}));

Object.assign(window.HTMLElement.prototype, {
  scrollIntoView: vi.fn(),
  releasePointerCapture: vi.fn(),
  hasPointerCapture: vi.fn(),
  setPointerCapture: vi.fn(),
  getBoundingClientRect: vi.fn().mockReturnValue({
    x: 0,
    y: 0,
    width: 0,
    height: 0,
    top: 0,
    right: 0,
    bottom: 0,
    left: 0,
  }),
});

setupMswV2();

beforeAll(() => {
  Element.prototype.hasPointerCapture = vi.fn(() => false);
  Element.prototype.setPointerCapture = vi.fn();
  Element.prototype.releasePointerCapture = vi.fn();
});

afterEach(() => {
  vi.clearAllMocks();
});

beforeEach(() => {
  navigateMock.mockClear();
  mockToast.mockClear();
});

async function selectFormat(user: ReturnType<typeof userEvent.setup>, formatDisplayName: string) {
  const trigger = screen.getByTestId('api-format-selector');
  await user.click(trigger);
  const listbox = await screen.findByRole('listbox');
  const option = within(listbox).getByText(formatDisplayName);
  await user.click(option);
}

describe('ApiModelForm — llm_liberty_oauth conditional render', () => {
  it('hides ApiKeyInput, ExtrasSection, and BaseUrlInput when llm_liberty_oauth is selected', async () => {
    const user = userEvent.setup();
    server.use(
      ...mockApiFormats({ data: ['openai', 'llm_liberty_oauth'] }),
    );

    await act(async () => {
      render(<ApiModelForm mode="create" />, { wrapper: createWrapper() });
    });

    await selectFormat(user, 'LLM Liberty OAuth');

    await waitFor(() => {
      expect(screen.getByTestId('llm-liberty-envelope-input')).toBeInTheDocument();
    });

    expect(screen.queryByTestId('api-key-input')).not.toBeInTheDocument();
    expect(screen.queryByTestId('api-key-input-checkbox')).not.toBeInTheDocument();
    expect(screen.queryByTestId('base-url-input')).not.toBeInTheDocument();
    expect(screen.queryByTestId('extra-headers-input')).not.toBeInTheDocument();
    expect(screen.queryByTestId('extra-body-input')).not.toBeInTheDocument();
  });

  it('still shows the legacy fields when anthropic_oauth is selected (regression guard)', async () => {
    const user = userEvent.setup();
    server.use(
      ...mockApiFormats({ data: ['openai', 'anthropic_oauth', 'llm_liberty_oauth'] }),
    );

    await act(async () => {
      render(<ApiModelForm mode="create" />, { wrapper: createWrapper() });
    });

    await selectFormat(user, 'Anthropic Setup Token');

    await waitFor(() => {
      expect(screen.getByTestId('base-url-input')).toBeInTheDocument();
      expect(screen.getByTestId('extra-headers-input')).toBeInTheDocument();
      expect(screen.getByTestId('extra-body-input')).toBeInTheDocument();
    });
    expect(screen.queryByTestId('llm-liberty-envelope-input')).not.toBeInTheDocument();
  });

  it('shows ApiKeyInput and BaseUrlInput by default for openai', async () => {
    await act(async () => {
      render(<ApiModelForm mode="create" />, { wrapper: createWrapper() });
    });

    expect(screen.getByTestId('base-url-input')).toBeInTheDocument();
    expect(screen.queryByTestId('llm-liberty-envelope-input')).not.toBeInTheDocument();
  });
});
