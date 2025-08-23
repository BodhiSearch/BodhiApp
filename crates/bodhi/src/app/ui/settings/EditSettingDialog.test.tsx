import { render, screen, waitFor, within } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { rest } from 'msw';
import { setupServer } from 'msw/node';
import { EditSettingDialog } from '@/app/ui/settings/EditSettingDialog';
import { ENDPOINT_SETTINGS } from '@/hooks/useQuery';
import { SettingInfo } from '@bodhiapp/ts-client';
import { createWrapper } from '@/tests/wrapper';
import { afterAll, afterEach, beforeAll, describe, expect, it, vi } from 'vitest';
import { showErrorParams, showSuccessParams } from '@/lib/utils.test';

// Add PointerEvent mock
function createMockPointerEvent(type: string, props: PointerEventInit = {}): PointerEvent {
  const event = new Event(type, props) as PointerEvent;
  Object.assign(event, {
    button: props.button ?? 0,
    ctrlKey: props.ctrlKey ?? false,
    pointerType: props.pointerType ?? 'mouse',
  });
  return event;
}

// Assign the mock function to the global window object
window.PointerEvent = createMockPointerEvent as any;

// Mock HTMLElement methods
Object.assign(window.HTMLElement.prototype, {
  scrollIntoView: vi.fn(),
  releasePointerCapture: vi.fn(),
  hasPointerCapture: vi.fn(),
});

const mockOnOpenChange = vi.fn();

const mockSettingInfos: Record<string, SettingInfo> = {
  string: {
    key: 'BODHI_HOME',
    current_value: '/home/user/.bodhi',
    default_value: '/home/user/.bodhi',
    source: 'default',
    metadata: {
      type: 'string',
    },
  },
  number: {
    key: 'BODHI_PORT',
    current_value: 1135,
    default_value: 1135,
    source: 'default',
    metadata: {
      type: 'number',
      min: 1025,
      max: 65535,
    },
  },
  option: {
    key: 'BODHI_LOG_LEVEL',
    current_value: 'info',
    default_value: 'warn',
    source: 'settings_file',
    metadata: {
      type: 'option',
      options: ['error', 'warn', 'info', 'debug', 'trace'],
    },
  },
  boolean: {
    key: 'BODHI_LOG_STDOUT',
    current_value: true,
    default_value: false,
    source: 'settings_file',
    metadata: {
      type: 'boolean',
    },
  },
};

const server = setupServer(
  rest.put(`*${ENDPOINT_SETTINGS}/*`, (_, res, ctx) => {
    return res(ctx.status(200), ctx.json(mockSettingInfos.string));
  })
);

// Mock the useToast hook
const mockToast = vi.fn();
vi.mock('@/hooks/use-toast', () => ({
  useToast: () => ({
    toast: mockToast,
  }),
}));

beforeAll(() => server.listen());
afterAll(() => server.close());
afterEach(() => {
  server.resetHandlers();
  mockOnOpenChange.mockClear();
  mockToast.mockClear();
});

describe('EditSettingDialog', () => {
  it('renders string input correctly', () => {
    render(<EditSettingDialog setting={mockSettingInfos.string} open={true} onOpenChange={mockOnOpenChange} />, {
      wrapper: createWrapper(),
    });

    expect(screen.getByRole('textbox')).toHaveValue('/home/user/.bodhi');
    expect(screen.getByText('Default: /home/user/.bodhi')).toBeInTheDocument();
  });

  it('renders number input with range correctly', () => {
    render(<EditSettingDialog setting={mockSettingInfos.number} open={true} onOpenChange={mockOnOpenChange} />, {
      wrapper: createWrapper(),
    });

    const input = screen.getByRole('spinbutton') as HTMLInputElement;
    expect(input).toHaveValue(1135);
    expect(input.min).toBe('1025');
    expect(input.max).toBe('65535');
    expect(screen.getByText('Range: 1025 - 65535')).toBeInTheDocument();
  });

  it('renders option select correctly', () => {
    render(<EditSettingDialog setting={mockSettingInfos.option} open={true} onOpenChange={mockOnOpenChange} />, {
      wrapper: createWrapper(),
    });

    expect(screen.getByRole('combobox')).toBeInTheDocument();
    expect(screen.getByText('info')).toBeInTheDocument();
  });

  it('renders boolean switch correctly', () => {
    render(<EditSettingDialog setting={mockSettingInfos.boolean} open={true} onOpenChange={mockOnOpenChange} />, {
      wrapper: createWrapper(),
    });

    expect(screen.getByRole('switch')).toBeInTheDocument();
    expect(screen.getByText('Enabled')).toBeInTheDocument();
  });

  it('updates string value correctly', async () => {
    const user = userEvent.setup();
    server.use(
      rest.put(`*${ENDPOINT_SETTINGS}/BODHI_HOME`, (_, res, ctx) => {
        return res(ctx.json({ ...mockSettingInfos.string, current_value: '/new/path' }));
      })
    );

    render(<EditSettingDialog setting={mockSettingInfos.string} open={true} onOpenChange={mockOnOpenChange} />, {
      wrapper: createWrapper(),
    });

    await user.clear(screen.getByRole('textbox'));
    await user.type(screen.getByRole('textbox'), '/new/path');
    await user.click(screen.getByRole('button', { name: /save/i }));

    expect(mockToast).toHaveBeenCalledWith({
      title: 'Success',
      description: 'Setting updated successfully',
      duration: 1000,
    });
    expect(mockOnOpenChange).toHaveBeenCalledWith(false);
  });

  it('updates number value with validation', async () => {
    const user = userEvent.setup();
    server.use(
      rest.put(`*${ENDPOINT_SETTINGS}/BODHI_PORT`, (_, res, ctx) => {
        return res(ctx.json({ ...mockSettingInfos.number, current_value: 2000 }));
      })
    );

    render(<EditSettingDialog setting={mockSettingInfos.number} open={true} onOpenChange={mockOnOpenChange} />, {
      wrapper: createWrapper(),
    });

    await user.clear(screen.getByRole('spinbutton'));
    await user.type(screen.getByRole('spinbutton'), '2000');
    await user.click(screen.getByRole('button', { name: /save/i }));

    expect(mockToast).toHaveBeenCalledWith(showSuccessParams('Success', 'Setting updated successfully'));
    expect(mockOnOpenChange).toHaveBeenCalledWith(false);
  });

  it('shows error for invalid number range', async () => {
    const user = userEvent.setup();

    render(<EditSettingDialog setting={mockSettingInfos.number} open={true} onOpenChange={mockOnOpenChange} />, {
      wrapper: createWrapper(),
    });

    await user.clear(screen.getByRole('spinbutton'));
    await user.type(screen.getByRole('spinbutton'), '100');
    await user.click(screen.getByRole('button', { name: /save/i }));

    expect(mockToast).toHaveBeenCalledWith(showErrorParams('Error', 'Value must be between 1025 and 65535'));
    expect(mockOnOpenChange).not.toHaveBeenCalled();
  });

  it('updates option value correctly', async () => {
    const user = userEvent.setup();
    server.use(
      rest.put(`*${ENDPOINT_SETTINGS}/BODHI_LOG_LEVEL`, (_, res, ctx) => {
        return res(ctx.json({ ...mockSettingInfos.option, current_value: 'debug' }));
      })
    );

    render(<EditSettingDialog setting={mockSettingInfos.option} open={true} onOpenChange={mockOnOpenChange} />, {
      wrapper: createWrapper(),
    });

    // Click the select trigger button to open the dropdown
    const selectTrigger = screen.getByRole('combobox');
    await user.click(selectTrigger);

    // Wait for and click the option
    const listbox = await screen.findByRole('listbox');
    const optionElement = within(listbox).getByRole('option', {
      name: /debug/i,
    });
    await user.click(optionElement);

    // Click save
    await user.click(screen.getByRole('button', { name: /save/i }));

    expect(mockToast).toHaveBeenCalledWith(showSuccessParams('Success', 'Setting updated successfully'));
    expect(mockOnOpenChange).toHaveBeenCalledWith(false);
  });

  it('updates boolean value correctly', async () => {
    const user = userEvent.setup();
    server.use(
      rest.put(`*${ENDPOINT_SETTINGS}/BODHI_LOG_STDOUT`, (_, res, ctx) => {
        return res(ctx.json({ ...mockSettingInfos.boolean, current_value: false }));
      })
    );

    render(<EditSettingDialog setting={mockSettingInfos.boolean} open={true} onOpenChange={mockOnOpenChange} />, {
      wrapper: createWrapper(),
    });

    await user.click(screen.getByRole('switch'));
    await user.click(screen.getByRole('button', { name: /save/i }));

    expect(mockToast).toHaveBeenCalledWith(showSuccessParams('Success', 'Setting updated successfully'));
    expect(screen.getByText('Disabled')).toBeInTheDocument();
  });

  it('handles API error correctly', async () => {
    const user = userEvent.setup();
    server.use(
      rest.put(`*${ENDPOINT_SETTINGS}/BODHI_HOME`, (_, res, ctx) => {
        return res(
          ctx.status(500),
          ctx.json({
            error: {
              message: 'Server error',
            },
          })
        );
      })
    );

    render(<EditSettingDialog setting={mockSettingInfos.string} open={true} onOpenChange={mockOnOpenChange} />, {
      wrapper: createWrapper(),
    });

    await user.clear(screen.getByRole('textbox'));
    await user.type(screen.getByRole('textbox'), '/new/path');
    await user.click(screen.getByRole('button', { name: /save/i }));

    expect(mockToast).toHaveBeenCalledWith(showErrorParams('Error', 'Server error'));
    expect(mockOnOpenChange).not.toHaveBeenCalled();
  });

  it('closes dialog on cancel', async () => {
    const user = userEvent.setup();

    render(<EditSettingDialog setting={mockSettingInfos.string} open={true} onOpenChange={mockOnOpenChange} />, {
      wrapper: createWrapper(),
    });

    await user.click(screen.getByRole('button', { name: /cancel/i }));
    expect(mockOnOpenChange).toHaveBeenCalledWith(false);
  });

  it('handles network error correctly', async () => {
    const user = userEvent.setup();

    // Simulate a network error
    server.use(
      rest.put(`*${ENDPOINT_SETTINGS}/BODHI_HOME`, (_, res) => {
        // Return error without response data to simulate network failure
        return res.networkError('Failed to connect');
      })
    );

    render(<EditSettingDialog setting={mockSettingInfos.string} open={true} onOpenChange={mockOnOpenChange} />, {
      wrapper: createWrapper(),
    });

    await user.clear(screen.getByRole('textbox'));
    await user.type(screen.getByRole('textbox'), '/new/path');
    await user.click(screen.getByRole('button', { name: /save/i }));

    // Verify error toast is shown with default error message
    expect(mockToast).toHaveBeenCalledWith(showErrorParams('Error', 'Failed to update setting'));

    // Verify dialog stays open
    expect(mockOnOpenChange).not.toHaveBeenCalled();
  });

  // This test verifies that the loading state is handled correctly during the request
  it('shows loading state during update', async () => {
    const user = userEvent.setup();

    // Add artificial delay to the response
    server.use(
      rest.put(`*${ENDPOINT_SETTINGS}/BODHI_HOME`, async (_, res, ctx) => {
        return res(ctx.delay(100), ctx.json(mockSettingInfos.string));
      })
    );

    render(<EditSettingDialog setting={mockSettingInfos.string} open={true} onOpenChange={mockOnOpenChange} />, {
      wrapper: createWrapper(),
    });

    await user.clear(screen.getByRole('textbox'));
    await user.type(screen.getByRole('textbox'), '/new/path');

    const saveButton = screen.getByRole('button', { name: /save/i });
    await user.click(saveButton);

    // Verify loading state
    expect(saveButton).toBeDisabled();
    expect(screen.getByText('Updating...')).toBeInTheDocument();

    // Wait for success
    await waitFor(() => {
      expect(mockToast).toHaveBeenCalledWith(showSuccessParams('Success', 'Setting updated successfully'));
    });
  });
});
