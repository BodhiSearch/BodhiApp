import ApiModelForm from '@/components/api-models/ApiModelForm';
import { mockApiFormats, mockFetchApiModels } from '@/test-utils/msw-v2/handlers/api-models';
import { server, setupMswV2, typedHttp } from '@/test-utils/msw-v2/setup';
import { createWrapper } from '@/tests/wrapper';
import { ApiAliasResponse } from '@bodhiapp/ts-client';
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

const mockApiAliasResponse: ApiAliasResponse = {
  source: 'api',
  id: 'test-api-model',
  api_format: 'openai',
  base_url: 'https://api.openai.com/v1',
  has_api_key: true,
  models: [{ id: 'gpt-4', object: 'model', created: 0, owned_by: 'openai', provider: 'openai' }],
  prefix: null,
  forward_all_with_prefix: false,
  created_at: '2024-01-01T00:00:00Z',
  updated_at: '2024-01-01T00:00:00Z',
};

describe('ApiModelForm - Extras fields (extra_headers and extra_body)', () => {
  const ANTHROPIC_OAUTH_DEFAULT_HEADERS = {
    'anthropic-version': '2023-06-01',
    'anthropic-beta': 'claude-code-20250219,oauth-2025-04-20',
    'user-agent': 'claude-cli/2.1.80 (external, cli)',
  };
  const ANTHROPIC_OAUTH_DEFAULT_BODY = {
    max_tokens: 4096,
    system: [{ type: 'text', text: "You are Claude Code, Anthropic's official CLI for Claude." }],
  };

  async function selectFormat(user: ReturnType<typeof userEvent.setup>, formatDisplayName: string) {
    const trigger = screen.getByTestId('api-format-selector');
    await user.click(trigger);
    const listbox = await screen.findByRole('listbox');
    const option = within(listbox).getByText(formatDisplayName);
    await user.click(option);
  }

  it('renders extras fields with pre-filled defaults when anthropic_oauth format is selected', async () => {
    const user = userEvent.setup();
    server.use(...mockApiFormats({ data: ['openai', 'anthropic_oauth'] }));

    await act(async () => {
      render(<ApiModelForm mode="create" />, { wrapper: createWrapper() });
    });

    await selectFormat(user, 'Anthropic (Claude Code OAuth)');

    await waitFor(() => {
      expect(screen.getByTestId('extra-headers-input')).toBeInTheDocument();
      expect(screen.getByTestId('extra-body-input')).toBeInTheDocument();
    });

    const headersValue = (screen.getByTestId('extra-headers-input') as HTMLTextAreaElement).value;
    const bodyValue = (screen.getByTestId('extra-body-input') as HTMLTextAreaElement).value;

    expect(JSON.parse(headersValue)).toEqual(ANTHROPIC_OAUTH_DEFAULT_HEADERS);
    expect(JSON.parse(bodyValue)).toEqual(ANTHROPIC_OAUTH_DEFAULT_BODY);
  });

  it('hides extras fields when openai format is selected', async () => {
    await act(async () => {
      render(<ApiModelForm mode="create" />, { wrapper: createWrapper() });
    });

    expect(screen.queryByTestId('extra-headers-input')).not.toBeInTheDocument();
    expect(screen.queryByTestId('extra-body-input')).not.toBeInTheDocument();
  });

  it('shows validation error for malformed JSON in extra_headers and blocks submit', async () => {
    const user = userEvent.setup();
    server.use(
      ...mockApiFormats({ data: ['anthropic_oauth'] }),
      ...mockFetchApiModels({ models: ['claude-3-5-sonnet-20241022'] })
    );

    await act(async () => {
      render(<ApiModelForm mode="create" />, { wrapper: createWrapper() });
    });

    await selectFormat(user, 'Anthropic (Claude Code OAuth)');

    await waitFor(() => {
      expect(screen.getByTestId('extra-headers-input')).toBeInTheDocument();
    });

    const headersInput = screen.getByTestId('extra-headers-input');
    fireEvent.change(headersInput, { target: { value: '{not valid json' } });

    await user.click(screen.getByTestId('fetch-models-button'));
    await waitFor(() => {
      expect(screen.queryByTestId('available-model-claude-3-5-sonnet-20241022')).toBeInTheDocument();
    });
    await user.click(screen.getByTestId('available-model-claude-3-5-sonnet-20241022'));

    await user.click(screen.getByTestId('create-api-model-button'));

    await waitFor(() => {
      expect(screen.getByTestId('extra-headers-input-error')).toBeInTheDocument();
    });
    expect(screen.getByTestId('extra-headers-input-error')).toHaveTextContent(/must be valid JSON/i);
  });

  it('loads existing alias with extras and shows formatted JSON in form', async () => {
    const aliasWithExtras: ApiAliasResponse = {
      ...mockApiAliasResponse,
      api_format: 'anthropic_oauth',
      extra_headers: ANTHROPIC_OAUTH_DEFAULT_HEADERS,
      extra_body: ANTHROPIC_OAUTH_DEFAULT_BODY,
    };

    server.use(...mockApiFormats({ data: ['anthropic_oauth'] }));

    await act(async () => {
      render(<ApiModelForm mode="edit" initialData={aliasWithExtras} />, {
        wrapper: createWrapper(),
      });
    });

    await waitFor(() => {
      expect(screen.getByTestId('extra-headers-input')).toBeInTheDocument();
    });

    const headersValue = (screen.getByTestId('extra-headers-input') as HTMLTextAreaElement).value;
    const bodyValue = (screen.getByTestId('extra-body-input') as HTMLTextAreaElement).value;

    expect(JSON.parse(headersValue)).toEqual(ANTHROPIC_OAUTH_DEFAULT_HEADERS);
    expect(JSON.parse(bodyValue)).toEqual(ANTHROPIC_OAUTH_DEFAULT_BODY);
  });

  it('sends null for extra_headers and extra_body when fields are empty on submit', async () => {
    const user = userEvent.setup();
    server.use(
      ...mockApiFormats({ data: ['anthropic_oauth'] }),
      ...mockFetchApiModels({ models: ['claude-3-5-sonnet-20241022'] })
    );

    let capturedRequestBody: Record<string, unknown> | undefined;
    server.use(
      typedHttp.post('/bodhi/v1/models/api', async ({ request, response }) => {
        capturedRequestBody = await request.json();
        return response(201 as const).json(mockApiAliasResponse);
      })
    );

    await act(async () => {
      render(<ApiModelForm mode="create" />, { wrapper: createWrapper() });
    });

    await selectFormat(user, 'Anthropic (Claude Code OAuth)');

    await waitFor(() => {
      expect(screen.getByTestId('extra-headers-input')).toBeInTheDocument();
    });

    await user.clear(screen.getByTestId('extra-headers-input'));
    await user.clear(screen.getByTestId('extra-body-input'));

    await user.click(screen.getByTestId('fetch-models-button'));
    await waitFor(() => {
      expect(screen.queryByTestId('available-model-claude-3-5-sonnet-20241022')).toBeInTheDocument();
    });
    await user.click(screen.getByTestId('available-model-claude-3-5-sonnet-20241022'));

    await user.click(screen.getByTestId('create-api-model-button'));

    await waitFor(() => {
      expect(capturedRequestBody).toBeDefined();
    });

    expect(capturedRequestBody!.extra_headers).toBeNull();
    expect(capturedRequestBody!.extra_body).toBeNull();
  });

  it('submits modified extras to the request payload', async () => {
    const user = userEvent.setup();
    const customHeaders = { 'x-custom': 'value' };
    server.use(
      ...mockApiFormats({ data: ['anthropic_oauth'] }),
      ...mockFetchApiModels({ models: ['claude-3-5-sonnet-20241022'] })
    );

    let capturedRequestBody: Record<string, unknown> | undefined;
    server.use(
      typedHttp.post('/bodhi/v1/models/api', async ({ request, response }) => {
        capturedRequestBody = await request.json();
        return response(201 as const).json(mockApiAliasResponse);
      })
    );

    await act(async () => {
      render(<ApiModelForm mode="create" />, { wrapper: createWrapper() });
    });

    await selectFormat(user, 'Anthropic (Claude Code OAuth)');

    await waitFor(() => {
      expect(screen.getByTestId('extra-headers-input')).toBeInTheDocument();
    });

    const headersInput = screen.getByTestId('extra-headers-input');
    fireEvent.change(headersInput, { target: { value: JSON.stringify(customHeaders) } });

    await user.click(screen.getByTestId('fetch-models-button'));
    await waitFor(() => {
      expect(screen.queryByTestId('available-model-claude-3-5-sonnet-20241022')).toBeInTheDocument();
    });
    await user.click(screen.getByTestId('available-model-claude-3-5-sonnet-20241022'));

    await user.click(screen.getByTestId('create-api-model-button'));

    await waitFor(() => {
      expect(capturedRequestBody).toBeDefined();
    });

    expect(capturedRequestBody!.extra_headers).toEqual(customHeaders);
  });

  it.each([
    ['empty string', '', null],
    ['whitespace only', '   ', null],
  ])('submits null for extra_headers when field is %s', async (_label, rawValue, expected) => {
    const user = userEvent.setup();
    server.use(
      ...mockApiFormats({ data: ['anthropic_oauth'] }),
      ...mockFetchApiModels({ models: ['claude-3-5-sonnet-20241022'] })
    );

    let capturedRequestBody: Record<string, unknown> | undefined;
    server.use(
      typedHttp.post('/bodhi/v1/models/api', async ({ request, response }) => {
        capturedRequestBody = await request.json();
        return response(201 as const).json(mockApiAliasResponse);
      })
    );

    await act(async () => {
      render(<ApiModelForm mode="create" />, { wrapper: createWrapper() });
    });

    await selectFormat(user, 'Anthropic (Claude Code OAuth)');

    await waitFor(() => {
      expect(screen.getByTestId('extra-headers-input')).toBeInTheDocument();
    });

    const headersInput = screen.getByTestId('extra-headers-input');
    fireEvent.change(headersInput, { target: { value: rawValue } });
    await user.clear(screen.getByTestId('extra-body-input'));

    await user.click(screen.getByTestId('fetch-models-button'));
    await waitFor(() => {
      expect(screen.queryByTestId('available-model-claude-3-5-sonnet-20241022')).toBeInTheDocument();
    });
    await user.click(screen.getByTestId('available-model-claude-3-5-sonnet-20241022'));

    await user.click(screen.getByTestId('create-api-model-button'));

    await waitFor(() => {
      expect(capturedRequestBody).toBeDefined();
    });

    expect(capturedRequestBody!.extra_headers).toBe(expected);
  });

  it('switching api_format in edit mode forces useApiKey=true (cannot Keep stored key)', async () => {
    const user = userEvent.setup();
    server.use(...mockApiFormats({ data: ['openai', 'anthropic_oauth'] }));

    // Stored alias has format=openai and a stored api key.
    const storedAlias: ApiAliasResponse = {
      ...mockApiAliasResponse,
      api_format: 'openai',
      has_api_key: true,
    };

    await act(async () => {
      render(<ApiModelForm mode="edit" initialData={storedAlias} />, {
        wrapper: createWrapper(),
      });
    });

    // Initially (edit + has_api_key), the checkbox is checked but field disabled.
    await waitFor(() => {
      expect(screen.getByTestId('api-key-input-checkbox')).toBeInTheDocument();
    });

    // Switch to anthropic_oauth → form should force useApiKey=true so the user
    // must enter a new key (backend rejects ApiKeyUpdate::Keep on format change).
    await selectFormat(user, 'Anthropic (Claude Code OAuth)');

    await waitFor(() => {
      expect(screen.getByTestId('api-key-input-checkbox')).toBeChecked();
    });
    // Extras are populated from preset.
    await waitFor(() => {
      expect(screen.getByTestId('extra-headers-input')).toBeInTheDocument();
    });
  });

  it('edit round-trip preserves extra_headers key with empty-string value', async () => {
    const storedExtras = { 'anthropic-beta': '', 'anthropic-version': '2023-06-01' };
    const aliasWithEmptyValueExtras: ApiAliasResponse = {
      ...mockApiAliasResponse,
      api_format: 'anthropic_oauth',
      extra_headers: storedExtras,
      extra_body: null,
    };

    server.use(...mockApiFormats({ data: ['anthropic_oauth'] }));

    await act(async () => {
      render(<ApiModelForm mode="edit" initialData={aliasWithEmptyValueExtras} />, {
        wrapper: createWrapper(),
      });
    });

    await waitFor(() => {
      expect(screen.getByTestId('extra-headers-input')).toBeInTheDocument();
    });

    const headersValue = (screen.getByTestId('extra-headers-input') as HTMLTextAreaElement).value;
    expect(JSON.parse(headersValue)).toEqual(storedExtras);
  });

  it.each([
    ['Authorization', { Authorization: 'Bearer x' }],
    ['authorization', { authorization: 'Bearer x' }],
    ['x-api-key', { 'x-api-key': 'sk-x' }],
    ['X-API-Key', { 'X-API-Key': 'sk-x' }],
  ])('rejects pass-through auth header `%s` with validation error', async (forbiddenKey, headers) => {
    const user = userEvent.setup();
    server.use(
      ...mockApiFormats({ data: ['anthropic_oauth'] }),
      ...mockFetchApiModels({ models: ['claude-3-5-sonnet-20241022'] })
    );

    await act(async () => {
      render(<ApiModelForm mode="create" />, { wrapper: createWrapper() });
    });

    await selectFormat(user, 'Anthropic (Claude Code OAuth)');

    await waitFor(() => {
      expect(screen.getByTestId('extra-headers-input')).toBeInTheDocument();
    });

    const headersInput = screen.getByTestId('extra-headers-input');
    fireEvent.change(headersInput, { target: { value: JSON.stringify(headers) } });

    await user.click(screen.getByTestId('fetch-models-button'));
    await waitFor(() => {
      expect(screen.queryByTestId('available-model-claude-3-5-sonnet-20241022')).toBeInTheDocument();
    });
    await user.click(screen.getByTestId('available-model-claude-3-5-sonnet-20241022'));

    await user.click(screen.getByTestId('create-api-model-button'));

    await waitFor(() => {
      expect(screen.getByTestId('extra-headers-input-error')).toBeInTheDocument();
    });
    expect(screen.getByTestId('extra-headers-input-error')).toHaveTextContent(
      /Cannot have pass-through authorization headers/i
    );
    expect(screen.getByTestId('extra-headers-input-error')).toHaveTextContent(forbiddenKey);
  });
});
