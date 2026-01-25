/**
 * EditToolsetPage Component Tests - UUID-based Architecture
 *
 * Purpose: Verify toolset edit page with UUID-based instance management
 *
 * Focus Areas:
 * - Form population from toolset data
 * - UUID-based query parameter
 * - API key Keep/Set handling
 * - Delete functionality
 * - Redirect when toolset type is admin-disabled
 */

import EditToolsetPage from '@/app/ui/toolsets/edit/page';
import { mockAppInfo } from '@/test-utils/msw-v2/handlers/info';
import {
  mockGetToolset,
  mockListToolsets,
  mockUpdateToolset,
  mockUpdateToolsetError,
  mockDeleteToolset,
} from '@/test-utils/msw-v2/handlers/toolsets';
import { mockUserLoggedIn, mockUserLoggedOut } from '@/test-utils/msw-v2/handlers/user';
import { server, setupMswV2 } from '@/test-utils/msw-v2/setup';
import { createWrapper } from '@/tests/wrapper';
import { act, render, screen, waitFor } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';

const pushMock = vi.fn();
let mockSearchParams: URLSearchParams;
let mockToast: ReturnType<typeof vi.fn>;

vi.mock('next/navigation', () => ({
  useRouter: () => ({
    push: pushMock,
  }),
  useSearchParams: () => mockSearchParams,
}));

vi.mock('@/hooks/use-toast', () => {
  const toast = vi.fn();
  return {
    toast,
    useToast: () => ({
      toast,
    }),
  };
});

setupMswV2();

beforeEach(async () => {
  pushMock.mockClear();
  const { toast } = await import('@/hooks/use-toast');
  mockToast = toast as unknown as ReturnType<typeof vi.fn>;
  mockToast.mockClear();
  mockSearchParams = new URLSearchParams('id=uuid-test-toolset');
});

afterEach(() => {
  vi.resetAllMocks();
});

describe('EditToolsetPage - Authentication & Initialization', () => {
  it('redirects to /ui/setup if status is setup', async () => {
    server.use(...mockAppInfo({ status: 'setup' }, { stub: true }), ...mockUserLoggedIn({}, { stub: true }));

    await act(async () => {
      render(<EditToolsetPage />, { wrapper: createWrapper() });
    });

    await waitFor(() => {
      expect(pushMock).toHaveBeenCalledWith('/ui/setup');
    });
  });

  it('redirects to /ui/login if user is not logged in', async () => {
    server.use(...mockAppInfo({ status: 'ready' }, { stub: true }), ...mockUserLoggedOut());

    await act(async () => {
      render(<EditToolsetPage />, { wrapper: createWrapper() });
    });

    await waitFor(() => {
      expect(pushMock).toHaveBeenCalledWith('/ui/login');
    });
  });
});

describe('EditToolsetPage - Error States', () => {
  beforeEach(() => {
    server.use(...mockAppInfo({ status: 'ready' }, { stub: true }), ...mockUserLoggedIn({}, { stub: true }));
  });

  it('shows error when id parameter is missing', async () => {
    mockSearchParams = new URLSearchParams('');

    await act(async () => {
      render(<EditToolsetPage />, { wrapper: createWrapper() });
    });

    await waitFor(() => {
      expect(screen.getByText('Toolset ID is required')).toBeInTheDocument();
    });
  });
});

describe('EditToolsetPage - Form Display', () => {
  beforeEach(() => {
    server.use(...mockAppInfo({ status: 'ready' }, { stub: true }), ...mockUserLoggedIn({}, { stub: true }));
  });

  it('displays toolset edit form with populated data', async () => {
    server.use(
      mockGetToolset({
        id: 'uuid-test-toolset',
        name: 'my-exa-search',
        scope_uuid: '4ff0e163-36fb-47d6-a5ef-26e396f067d6',
        scope: 'scope_toolset-builtin-exa-web-search',
        description: 'Test toolset',
        enabled: true,
        has_api_key: true,
        tools: [],
        created_at: '2024-01-01T00:00:00Z',
        updated_at: '2024-01-01T00:00:00Z',
      }),
      mockListToolsets(
        [],
        [
          {
            scope: 'scope_toolset-builtin-exa-web-search',
            scope_uuid: '4ff0e163-36fb-47d6-a5ef-26e396f067d6',
            enabled: true,
            updated_by: 'system',
            created_at: '2024-01-01T00:00:00Z',
            updated_at: '2024-01-01T00:00:00Z',
          },
        ]
      )
    );

    await act(async () => {
      render(<EditToolsetPage />, { wrapper: createWrapper() });
    });

    await waitFor(() => {
      expect(screen.getByTestId('edit-toolset-page')).toBeInTheDocument();
    });

    expect(screen.getByTestId('toolset-name-input')).toHaveValue('my-exa-search');
    expect(screen.getByTestId('toolset-description-input')).toHaveValue('Test toolset');
    expect(screen.getByTestId('toolset-enabled-switch')).toBeChecked();
    expect(screen.getByTestId('toolset-api-key-input')).toBeInTheDocument();
  });

  it('redirects to toolsets page when toolset type is disabled by admin', async () => {
    server.use(
      mockGetToolset({
        id: 'uuid-test-toolset',
        name: 'my-exa-search',
        scope_uuid: '4ff0e163-36fb-47d6-a5ef-26e396f067d6',
        scope: 'scope_toolset-builtin-exa-web-search',
        description: 'Test toolset',
        enabled: true,
        has_api_key: true,
        tools: [],
        created_at: '2024-01-01T00:00:00Z',
        updated_at: '2024-01-01T00:00:00Z',
      }),
      mockListToolsets(
        [],
        [
          {
            scope: 'scope_toolset-builtin-exa-web-search',
            scope_uuid: '4ff0e163-36fb-47d6-a5ef-26e396f067d6',
            enabled: false,
            updated_by: 'system',
            created_at: '2024-01-01T00:00:00Z',
            updated_at: '2024-01-01T00:00:00Z',
          },
        ]
      )
    );

    await act(async () => {
      render(<EditToolsetPage />, { wrapper: createWrapper() });
    });

    await waitFor(() => {
      expect(mockToast).toHaveBeenCalledWith(
        expect.objectContaining({
          title: 'Toolset disabled',
          variant: 'destructive',
        })
      );
    });

    await waitFor(() => {
      expect(pushMock).toHaveBeenCalledWith('/ui/toolsets');
    });
  });
});

describe('EditToolsetPage - Update Functionality', () => {
  beforeEach(() => {
    server.use(
      ...mockAppInfo({ status: 'ready' }, { stub: true }),
      ...mockUserLoggedIn({}, { stub: true }),
      mockGetToolset({
        id: 'uuid-test-toolset',
        name: 'my-exa-search',
        scope_uuid: '4ff0e163-36fb-47d6-a5ef-26e396f067d6',
        scope: 'scope_toolset-builtin-exa-web-search',
        description: 'Test toolset',
        enabled: false,
        has_api_key: true,
        tools: [],
        created_at: '2024-01-01T00:00:00Z',
        updated_at: '2024-01-01T00:00:00Z',
      }),
      mockListToolsets(
        [],
        [
          {
            scope: 'scope_toolset-builtin-exa-web-search',
            scope_uuid: '4ff0e163-36fb-47d6-a5ef-26e396f067d6',
            enabled: true,
            updated_by: 'system',
            created_at: '2024-01-01T00:00:00Z',
            updated_at: '2024-01-01T00:00:00Z',
          },
        ]
      )
    );
  });

  it('updates toolset with Keep API key action when api_key field is empty', async () => {
    const user = userEvent.setup();

    server.use(
      mockUpdateToolset({
        id: 'uuid-test-toolset',
        name: 'my-exa-search',
        scope_uuid: '4ff0e163-36fb-47d6-a5ef-26e396f067d6',
        scope: 'scope_toolset-builtin-exa-web-search',
        description: 'Updated description',
        enabled: true,
        has_api_key: true,
        tools: [],
        created_at: '2024-01-01T00:00:00Z',
        updated_at: new Date().toISOString(),
      })
    );

    await act(async () => {
      render(<EditToolsetPage />, { wrapper: createWrapper() });
    });

    await waitFor(() => {
      expect(screen.getByTestId('edit-toolset-page')).toBeInTheDocument();
    });

    const descInput = screen.getByTestId('toolset-description-input');
    await user.clear(descInput);
    await user.type(descInput, 'Updated description');

    const enableToggle = screen.getByTestId('toolset-enabled-switch');
    await user.click(enableToggle);

    const saveButton = screen.getByTestId('toolset-save-button');
    await user.click(saveButton);

    await waitFor(() => {
      expect(mockToast).toHaveBeenCalledWith(
        expect.objectContaining({
          title: 'Toolset updated successfully',
        })
      );
    });
  });

  it('updates toolset with Set API key action when new api_key is entered', async () => {
    const user = userEvent.setup();

    server.use(
      mockUpdateToolset({
        id: 'uuid-test-toolset',
        name: 'my-exa-search',
        scope_uuid: '4ff0e163-36fb-47d6-a5ef-26e396f067d6',
        scope: 'scope_toolset-builtin-exa-web-search',
        description: 'Test toolset',
        enabled: true,
        has_api_key: true,
        tools: [],
        created_at: '2024-01-01T00:00:00Z',
        updated_at: new Date().toISOString(),
      })
    );

    await act(async () => {
      render(<EditToolsetPage />, { wrapper: createWrapper() });
    });

    await waitFor(() => {
      expect(screen.getByTestId('edit-toolset-page')).toBeInTheDocument();
    });

    const apiKeyInput = screen.getByTestId('toolset-api-key-input');
    await user.type(apiKeyInput, 'new-api-key-value');

    const enableToggle = screen.getByTestId('toolset-enabled-switch');
    await user.click(enableToggle);

    const saveButton = screen.getByTestId('toolset-save-button');
    await user.click(saveButton);

    await waitFor(() => {
      expect(mockToast).toHaveBeenCalledWith(
        expect.objectContaining({
          title: 'Toolset updated successfully',
        })
      );
    });
  });

  it('shows error toast when update fails', async () => {
    const user = userEvent.setup();

    server.use(
      mockUpdateToolsetError({
        message: 'Name already exists',
        status: 400,
      })
    );

    await act(async () => {
      render(<EditToolsetPage />, { wrapper: createWrapper() });
    });

    await waitFor(() => {
      expect(screen.getByTestId('edit-toolset-page')).toBeInTheDocument();
    });

    const saveButton = screen.getByTestId('toolset-save-button');
    await user.click(saveButton);

    await waitFor(() => {
      expect(mockToast).toHaveBeenCalledWith(
        expect.objectContaining({
          title: 'Failed to update toolset',
          description: 'Name already exists',
          variant: 'destructive',
        })
      );
    });
  });
});

describe('EditToolsetPage - Delete Functionality', () => {
  beforeEach(() => {
    server.use(
      ...mockAppInfo({ status: 'ready' }, { stub: true }),
      ...mockUserLoggedIn({}, { stub: true }),
      mockGetToolset({
        id: 'uuid-test-toolset',
        name: 'my-exa-search',
        scope_uuid: '4ff0e163-36fb-47d6-a5ef-26e396f067d6',
        scope: 'scope_toolset-builtin-exa-web-search',
        description: 'Test toolset',
        enabled: true,
        has_api_key: true,
        tools: [],
        created_at: '2024-01-01T00:00:00Z',
        updated_at: '2024-01-01T00:00:00Z',
      }),
      mockListToolsets(
        [],
        [
          {
            scope: 'scope_toolset-builtin-exa-web-search',
            scope_uuid: '4ff0e163-36fb-47d6-a5ef-26e396f067d6',
            enabled: true,
            updated_by: 'system',
            created_at: '2024-01-01T00:00:00Z',
            updated_at: '2024-01-01T00:00:00Z',
          },
        ]
      )
    );
  });

  it('shows delete confirmation dialog when delete button is clicked', async () => {
    const user = userEvent.setup();

    await act(async () => {
      render(<EditToolsetPage />, { wrapper: createWrapper() });
    });

    await waitFor(() => {
      expect(screen.getByTestId('edit-toolset-page')).toBeInTheDocument();
    });

    const deleteButton = screen.getByTestId('toolset-delete-button');
    await user.click(deleteButton);

    await waitFor(() => {
      expect(screen.getByRole('alertdialog')).toBeInTheDocument();
      expect(screen.getByText(/Are you sure you want to delete/)).toBeInTheDocument();
    });
  });

  it('deletes toolset and navigates to toolsets page on confirmation', async () => {
    const user = userEvent.setup();

    server.use(mockDeleteToolset());

    await act(async () => {
      render(<EditToolsetPage />, { wrapper: createWrapper() });
    });

    await waitFor(() => {
      expect(screen.getByTestId('edit-toolset-page')).toBeInTheDocument();
    });

    const deleteButton = screen.getByTestId('toolset-delete-button');
    await user.click(deleteButton);

    await waitFor(() => {
      expect(screen.getByRole('alertdialog')).toBeInTheDocument();
    });

    const confirmButton = screen.getByRole('button', { name: /delete/i });
    await user.click(confirmButton);

    await waitFor(() => {
      expect(mockToast).toHaveBeenCalledWith(
        expect.objectContaining({
          title: 'Toolset deleted successfully',
        })
      );
    });

    await waitFor(() => {
      expect(pushMock).toHaveBeenCalledWith('/ui/toolsets');
    });
  });
});

describe('EditToolsetPage - Form Validation', () => {
  beforeEach(() => {
    server.use(
      ...mockAppInfo({ status: 'ready' }, { stub: true }),
      ...mockUserLoggedIn({}, { stub: true }),
      mockGetToolset({
        id: 'uuid-test-toolset',
        name: 'my-exa-search',
        scope_uuid: '4ff0e163-36fb-47d6-a5ef-26e396f067d6',
        scope: 'scope_toolset-builtin-exa-web-search',
        description: 'Test toolset',
        enabled: true,
        has_api_key: true,
        tools: [],
        created_at: '2024-01-01T00:00:00Z',
        updated_at: '2024-01-01T00:00:00Z',
      }),
      mockListToolsets(
        [],
        [
          {
            scope: 'scope_toolset-builtin-exa-web-search',
            scope_uuid: '4ff0e163-36fb-47d6-a5ef-26e396f067d6',
            enabled: true,
            updated_by: 'system',
            created_at: '2024-01-01T00:00:00Z',
            updated_at: '2024-01-01T00:00:00Z',
          },
        ]
      )
    );
  });

  it('validates name length (max 24 characters)', async () => {
    const user = userEvent.setup();

    await act(async () => {
      render(<EditToolsetPage />, { wrapper: createWrapper() });
    });

    await waitFor(() => {
      expect(screen.getByTestId('edit-toolset-page')).toBeInTheDocument();
    });

    const nameInput = screen.getByTestId('toolset-name-input');
    await user.clear(nameInput);
    await user.type(nameInput, 'this-name-is-way-too-long-for-validation');

    const saveButton = screen.getByTestId('toolset-save-button');
    await user.click(saveButton);

    await waitFor(() => {
      expect(screen.getByText('Name must be 24 characters or less')).toBeInTheDocument();
    });
  });

  it('validates name format (alphanumeric and hyphens only)', async () => {
    const user = userEvent.setup();

    await act(async () => {
      render(<EditToolsetPage />, { wrapper: createWrapper() });
    });

    await waitFor(() => {
      expect(screen.getByTestId('edit-toolset-page')).toBeInTheDocument();
    });

    const nameInput = screen.getByTestId('toolset-name-input');
    await user.clear(nameInput);
    await user.type(nameInput, 'invalid name with spaces');

    const saveButton = screen.getByTestId('toolset-save-button');
    await user.click(saveButton);

    await waitFor(() => {
      expect(screen.getByText('Name can only contain letters, numbers, and hyphens')).toBeInTheDocument();
    });
  });
});
