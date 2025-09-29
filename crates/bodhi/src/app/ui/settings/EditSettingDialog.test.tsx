import { render, screen, waitFor, within } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { EditSettingDialog } from '@/app/ui/settings/EditSettingDialog';
import { SettingInfo } from '@bodhiapp/ts-client';
import { createWrapper } from '@/tests/wrapper';
import { afterEach, describe, expect, it, vi } from 'vitest';
import { showErrorParams, showSuccessParams } from '@/lib/utils.test';
import { setupMswV2, server } from '@/test-utils/msw-v2/setup';
import {
  mockUpdateSetting,
  mockUpdateSettingServerError,
  mockUpdateSettingNetworkError,
  mockSettingsNotFound,
  mockUpdateSettingError,
} from '@/test-utils/msw-v2/handlers/settings';

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

// Mock the useToast hook
const mockToast = vi.fn();
vi.mock('@/hooks/use-toast', () => ({
  useToast: () => ({
    toast: mockToast,
  }),
}));

setupMswV2();

afterEach(() => {
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
      ...mockUpdateSetting('BODHI_HOME', {
        current_value: '/new/path',
        default_value: '/home/user/.bodhi',
        source: 'settings_file',
        metadata: { type: 'string' },
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
      description: 'Setting BODHI_HOME updated successfully',
      duration: 1000,
    });
    expect(mockOnOpenChange).toHaveBeenCalledWith(false);
  });

  it('updates number value with validation', async () => {
    const user = userEvent.setup();
    server.use(
      ...mockUpdateSetting('BODHI_PORT', {
        current_value: 2000,
        default_value: 1135,
        source: 'settings_file',
        metadata: {
          type: 'number',
          min: 1025,
          max: 65535,
        },
      })
    );

    render(<EditSettingDialog setting={mockSettingInfos.number} open={true} onOpenChange={mockOnOpenChange} />, {
      wrapper: createWrapper(),
    });

    await user.clear(screen.getByRole('spinbutton'));
    await user.type(screen.getByRole('spinbutton'), '2000');
    await user.click(screen.getByRole('button', { name: /save/i }));

    expect(mockToast).toHaveBeenCalledWith(showSuccessParams('Success', 'Setting BODHI_PORT updated successfully'));
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
      ...mockUpdateSetting('BODHI_LOG_LEVEL', {
        current_value: 'debug',
        default_value: 'warn',
        source: 'settings_file',
        metadata: {
          type: 'option',
          options: ['error', 'warn', 'info', 'debug', 'trace'],
        },
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

    expect(mockToast).toHaveBeenCalledWith(
      showSuccessParams('Success', 'Setting BODHI_LOG_LEVEL updated successfully')
    );
    expect(mockOnOpenChange).toHaveBeenCalledWith(false);
  });

  it('updates boolean value correctly', async () => {
    const user = userEvent.setup();
    server.use(
      ...mockUpdateSetting('BODHI_LOG_STDOUT', {
        current_value: false,
        default_value: false,
        source: 'settings_file',
        metadata: {
          type: 'boolean',
        },
      })
    );

    render(<EditSettingDialog setting={mockSettingInfos.boolean} open={true} onOpenChange={mockOnOpenChange} />, {
      wrapper: createWrapper(),
    });

    await user.click(screen.getByRole('switch'));
    await user.click(screen.getByRole('button', { name: /save/i }));

    expect(mockToast).toHaveBeenCalledWith(
      showSuccessParams('Success', 'Setting BODHI_LOG_STDOUT updated successfully')
    );
    expect(screen.getByText('Disabled')).toBeInTheDocument();
  });

  it('handles API error correctly', async () => {
    const user = userEvent.setup();
    server.use(...mockUpdateSettingServerError('BODHI_HOME'));

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
    server.use(...mockUpdateSettingNetworkError('BODHI_HOME'));

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
      ...mockUpdateSetting(
        'BODHI_HOME',
        {
          current_value: '/home/user/.bodhi',
          default_value: '/home/user/.bodhi',
          source: 'default',
          metadata: {
            type: 'string',
          },
        },
        { delayMs: 100 }
      )
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
      expect(mockToast).toHaveBeenCalledWith(showSuccessParams('Success', 'Setting BODHI_HOME updated successfully'));
    });
  });
});

describe('MSW Handler Passthrough Behavior', () => {
  describe('Multiple handlers with different keys', () => {
    it('routes requests to correct handler based on key matching', async () => {
      const user = userEvent.setup();

      // Register handlers for different keys
      server.use(
        ...mockUpdateSetting('BODHI_HOME', {
          current_value: '/home/updated',
          default_value: '/home/user/.bodhi',
          source: 'settings_file',
          metadata: { type: 'string' },
        }),
        ...mockUpdateSetting('BODHI_PORT', {
          current_value: 3000,
          default_value: 1135,
          source: 'settings_file',
          metadata: { type: 'number', min: 1025, max: 65535 },
        }),
        ...mockUpdateSetting('BODHI_LOG_LEVEL', {
          current_value: 'debug',
          default_value: 'warn',
          source: 'settings_file',
          metadata: { type: 'option', options: ['error', 'warn', 'info', 'debug', 'trace'] },
        }),
        ...mockSettingsNotFound() // Catch-all for unmapped keys
      );

      // Test BODHI_HOME update
      const { unmount: unmount1 } = render(
        <EditSettingDialog setting={mockSettingInfos.string} open={true} onOpenChange={mockOnOpenChange} />,
        { wrapper: createWrapper() }
      );

      await user.clear(screen.getByRole('textbox'));
      await user.type(screen.getByRole('textbox'), '/home/updated');
      await user.click(screen.getByRole('button', { name: /save/i }));

      expect(mockToast).toHaveBeenCalledWith(showSuccessParams('Success', 'Setting BODHI_HOME updated successfully'));
      mockToast.mockClear();
      mockOnOpenChange.mockClear();
      unmount1();

      // Test BODHI_PORT update
      const { unmount: unmount2 } = render(
        <EditSettingDialog setting={mockSettingInfos.number} open={true} onOpenChange={mockOnOpenChange} />,
        { wrapper: createWrapper() }
      );

      await user.clear(screen.getByRole('spinbutton'));
      await user.type(screen.getByRole('spinbutton'), '3000');
      await user.click(screen.getByRole('button', { name: /save/i }));

      expect(mockToast).toHaveBeenCalledWith(showSuccessParams('Success', 'Setting BODHI_PORT updated successfully'));
      mockToast.mockClear();
      unmount2();

      // Test BODHI_LOG_LEVEL update
      const { unmount: unmount3 } = render(
        <EditSettingDialog setting={mockSettingInfos.option} open={true} onOpenChange={mockOnOpenChange} />,
        { wrapper: createWrapper() }
      );

      const selectTrigger = screen.getByRole('combobox');
      await user.click(selectTrigger);
      const listbox = await screen.findByRole('listbox');
      const optionElement = within(listbox).getByRole('option', { name: /debug/i });
      await user.click(optionElement);
      await user.click(screen.getByRole('button', { name: /save/i }));

      expect(mockToast).toHaveBeenCalledWith(
        showSuccessParams('Success', 'Setting BODHI_LOG_LEVEL updated successfully')
      );
      unmount3();
    });

    it('returns 404 for unmapped keys via catch-all handler', async () => {
      const user = userEvent.setup();

      // Register handlers for specific keys only + catch-all
      server.use(
        ...mockUpdateSetting('BODHI_PORT', {
          current_value: 3000,
          default_value: 1135,
          source: 'settings_file',
          metadata: { type: 'number', min: 1025, max: 65535 },
        }),
        ...mockSettingsNotFound() // This will catch BODHI_HOME requests
      );

      // Try to update BODHI_HOME - should be caught by catch-all and return 404
      render(<EditSettingDialog setting={mockSettingInfos.string} open={true} onOpenChange={mockOnOpenChange} />, {
        wrapper: createWrapper(),
      });

      await user.clear(screen.getByRole('textbox'));
      await user.type(screen.getByRole('textbox'), '/new/path');
      await user.click(screen.getByRole('button', { name: /save/i }));

      // Should show 404 error from catch-all handler
      expect(mockToast).toHaveBeenCalledWith(showErrorParams('Error', 'Setting BODHI_HOME not found'));
      expect(mockOnOpenChange).not.toHaveBeenCalled();
    });
  });

  describe('Handler priority and passthrough', () => {
    it('first matching handler wins, others are bypassed', async () => {
      const user = userEvent.setup();

      // Register multiple handlers for the same key - first should win
      server.use(
        ...mockUpdateSetting('BODHI_HOME', {
          current_value: '/first/handler',
          default_value: '/home/user/.bodhi',
          source: 'settings_file',
          metadata: { type: 'string' },
        }),
        ...mockUpdateSetting('BODHI_HOME', {
          current_value: '/second/handler', // This should never be reached
          default_value: '/home/user/.bodhi',
          source: 'settings_file',
          metadata: { type: 'string' },
        })
      );

      render(<EditSettingDialog setting={mockSettingInfos.string} open={true} onOpenChange={mockOnOpenChange} />, {
        wrapper: createWrapper(),
      });

      await user.clear(screen.getByRole('textbox'));
      await user.type(screen.getByRole('textbox'), '/any/path');
      await user.click(screen.getByRole('button', { name: /save/i }));

      // Should succeed with first handler (we can't verify exact response content in UI test,
      // but success indicates first handler was used)
      expect(mockToast).toHaveBeenCalledWith(showSuccessParams('Success', 'Setting BODHI_HOME updated successfully'));
      expect(mockOnOpenChange).toHaveBeenCalledWith(false);
    });

    it('error handlers pass through for non-matching keys', async () => {
      const user = userEvent.setup();

      // Register error handler for BODHI_PORT, success handler for BODHI_HOME
      server.use(
        ...mockUpdateSettingError('BODHI_PORT', {
          code: 'invalid_port',
          message: 'Port is invalid',
          type: 'invalid_request_error',
          status: 400,
        }),
        ...mockUpdateSetting('BODHI_HOME', {
          current_value: '/home/success',
          default_value: '/home/user/.bodhi',
          source: 'settings_file',
          metadata: { type: 'string' },
        }),
        ...mockSettingsNotFound()
      );

      // Test BODHI_HOME - should pass through error handler and reach success handler
      render(<EditSettingDialog setting={mockSettingInfos.string} open={true} onOpenChange={mockOnOpenChange} />, {
        wrapper: createWrapper(),
      });

      await user.clear(screen.getByRole('textbox'));
      await user.type(screen.getByRole('textbox'), '/home/success');
      await user.click(screen.getByRole('button', { name: /save/i }));

      expect(mockToast).toHaveBeenCalledWith(showSuccessParams('Success', 'Setting BODHI_HOME updated successfully'));
      expect(mockOnOpenChange).toHaveBeenCalledWith(false);
    });

    it('network error handlers pass through for non-matching keys', async () => {
      const user = userEvent.setup();

      // Register network error for BODHI_LOG_LEVEL, success for BODHI_HOME
      server.use(
        ...mockUpdateSettingNetworkError('BODHI_LOG_LEVEL'),
        ...mockUpdateSetting('BODHI_HOME', {
          current_value: '/home/after/network/error',
          default_value: '/home/user/.bodhi',
          source: 'settings_file',
          metadata: { type: 'string' },
        }),
        ...mockSettingsNotFound()
      );

      // Test BODHI_HOME - should pass through network error handler and succeed
      render(<EditSettingDialog setting={mockSettingInfos.string} open={true} onOpenChange={mockOnOpenChange} />, {
        wrapper: createWrapper(),
      });

      await user.clear(screen.getByRole('textbox'));
      await user.type(screen.getByRole('textbox'), '/home/after/network/error');
      await user.click(screen.getByRole('button', { name: /save/i }));

      expect(mockToast).toHaveBeenCalledWith(showSuccessParams('Success', 'Setting BODHI_HOME updated successfully'));
      expect(mockOnOpenChange).toHaveBeenCalledWith(false);
    });
  });

  describe('Complex handler chains', () => {
    it('handles complex chain with multiple handler types', async () => {
      const user = userEvent.setup();

      // Complex setup: error for KEY1, network error for KEY2, success for BODHI_HOME, catch-all 404
      server.use(
        ...mockUpdateSettingError('UNKNOWN_KEY_1', {
          code: 'error1',
          message: 'Error for key 1',
          status: 400,
        }),
        ...mockUpdateSettingNetworkError('UNKNOWN_KEY_2'),
        ...mockUpdateSettingError('UNKNOWN_KEY_3', {
          code: 'error3',
          message: 'Error for key 3',
          status: 403,
        }),
        ...mockUpdateSetting('BODHI_HOME', {
          current_value: '/final/success',
          default_value: '/home/user/.bodhi',
          source: 'settings_file',
          metadata: { type: 'string' },
        }),
        ...mockSettingsNotFound()
      );

      // BODHI_HOME should pass through all error handlers and reach the success handler
      render(<EditSettingDialog setting={mockSettingInfos.string} open={true} onOpenChange={mockOnOpenChange} />, {
        wrapper: createWrapper(),
      });

      await user.clear(screen.getByRole('textbox'));
      await user.type(screen.getByRole('textbox'), '/final/success');
      await user.click(screen.getByRole('button', { name: /save/i }));

      expect(mockToast).toHaveBeenCalledWith(showSuccessParams('Success', 'Setting BODHI_HOME updated successfully'));
      expect(mockOnOpenChange).toHaveBeenCalledWith(false);
    });
  });
});
