import ApiModelForm from '@/components/api-models/ApiModelForm';
import { mockApiFormats } from '@/test-utils/msw-v2/handlers/api-models';
import { server, setupMswV2, typedHttp } from '@/test-utils/msw-v2/setup';
import { createWrapper } from '@/tests/wrapper';
import type { ApiAliasResponse } from '@bodhiapp/ts-client';
import type { ReactNode } from 'react';
import { act, fireEvent, render, screen, waitFor, within } from '@testing-library/react';
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
    server.use(...mockApiFormats({ data: ['openai', 'llm_liberty_oauth'] }));

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
    server.use(...mockApiFormats({ data: ['openai', 'anthropic_oauth', 'llm_liberty_oauth'] }));

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

describe('ApiModelForm — llm_liberty_oauth edit mode (keep vs set)', () => {
  const ENDPOINT_API_MODEL_ID = '/bodhi/v1/models/api/{id}' as const;

  const storedAlias: ApiAliasResponse = {
    source: 'api',
    id: 'liberty-edit-1',
    name: 'Liberty Edit Model',
    api_format: 'llm_liberty_oauth',
    base_url: 'https://api.anthropic.com/v1',
    has_api_key: false,
    models: [
      {
        id: 'claude-3-5-haiku-latest',
        display_name: 'Claude Haiku 3.5',
        type: 'model',
        created_at: '2024-01-01T00:00:00Z',
        provider: 'anthropic' as const,
        access: true,
      },
    ],
    prefix: 'anth/',
    forward_all_with_prefix: false,
    created_at: '2024-01-01T00:00:00Z',
    updated_at: '2024-01-01T00:00:00Z',
    llm_liberty: {
      provider: 'anthropic',
      envelope_version: '1.0.0',
      expires_at: 9999999999,
      has_refresh_token: true,
    },
  };

  const validEnvelopeJson = JSON.stringify(
    {
      version: '1.0.0',
      provider: 'anthropic',
      access_token: 'access-fresh',
      refresh_token: 'refresh-fresh',
      expires_at: 9999999999,
      auth: { in: 'header', key: 'Authorization', scheme: 'Bearer' },
      oauth: { token_url: 'https://oauth.example/token', client_id: 'client-id-public' },
      api: { base_url: 'https://api.anthropic.com/v1', chat_url: 'https://api.anthropic.com/v1/messages' },
    },
    null,
    2
  );

  it('emits envelope: { action: "keep" } when textarea is left empty on edit submit', async () => {
    const user = userEvent.setup();
    server.use(...mockApiFormats({ data: ['llm_liberty_oauth'] }));

    let capturedBody: Record<string, unknown> | undefined;
    server.use(
      typedHttp.put(ENDPOINT_API_MODEL_ID, async ({ request, response }) => {
        capturedBody = await request.json();
        return response(200 as const).json(storedAlias);
      })
    );

    await act(async () => {
      render(<ApiModelForm mode="edit" initialData={storedAlias} />, { wrapper: createWrapper() });
    });

    await waitFor(() => {
      expect(screen.getByTestId('llm-liberty-envelope-input')).toBeInTheDocument();
    });

    await user.click(screen.getByTestId('update-api-model-button'));

    await waitFor(() => {
      expect(capturedBody).toBeDefined();
    });
    expect(capturedBody!.api_format).toBe('llm_liberty_oauth');
    expect(capturedBody!.envelope).toEqual({ action: 'keep' });
  });

  it('emits envelope: { action: "set", value } when a fresh JSON envelope is pasted on edit', async () => {
    const user = userEvent.setup();
    server.use(...mockApiFormats({ data: ['llm_liberty_oauth'] }));

    let capturedBody: Record<string, unknown> | undefined;
    server.use(
      typedHttp.put(ENDPOINT_API_MODEL_ID, async ({ request, response }) => {
        capturedBody = await request.json();
        return response(200 as const).json(storedAlias);
      })
    );

    await act(async () => {
      render(<ApiModelForm mode="edit" initialData={storedAlias} />, { wrapper: createWrapper() });
    });

    const textarea = await screen.findByTestId('llm-liberty-envelope-input');
    fireEvent.change(textarea, { target: { value: validEnvelopeJson } });

    // Wait for the parsed-summary to appear (validates the textarea content).
    await waitFor(() => {
      expect(screen.getByTestId('llm-liberty-envelope-summary')).toBeInTheDocument();
    });

    await user.click(screen.getByTestId('update-api-model-button'));

    await waitFor(() => {
      expect(capturedBody).toBeDefined();
    });
    expect(capturedBody!.api_format).toBe('llm_liberty_oauth');
    const envelopeUpdate = capturedBody!.envelope as { action: string; value?: Record<string, unknown> };
    expect(envelopeUpdate.action).toBe('set');
    expect(envelopeUpdate.value).toBeDefined();
    expect(envelopeUpdate.value!.access_token).toBe('access-fresh');
    expect(envelopeUpdate.value!.refresh_token).toBe('refresh-fresh');
  });
});
