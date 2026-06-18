import { SettingInfo } from '@bodhiapp/ts-client';
import { render, screen } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { describe, expect, it, vi } from 'vitest';

import { parseSettingValue, SettingValueInput, settingValueHint } from './settingInput';

const stringSetting: SettingInfo = {
  key: 'BODHI_HOST',
  current_value: '0.0.0.0',
  default_value: '0.0.0.0',
  source: 'default',
  metadata: { type: 'string' },
};
const numberSetting: SettingInfo = {
  key: 'BODHI_KEEP_ALIVE_SECS',
  current_value: 600,
  default_value: 300,
  source: 'settings_file',
  metadata: { type: 'number', min: 300, max: 86400 },
};
const optionSetting: SettingInfo = {
  key: 'BODHI_EXEC_VARIANT',
  current_value: 'metal',
  default_value: 'metal',
  source: 'default',
  metadata: { type: 'option', options: ['metal', 'cpu', 'cuda'] },
};
const booleanSetting: SettingInfo = {
  key: 'BODHI_LOG_STDOUT',
  current_value: true,
  default_value: false,
  source: 'default',
  metadata: { type: 'boolean' },
};

describe('parseSettingValue', () => {
  it('passes strings through unchanged', () => {
    expect(parseSettingValue(stringSetting.metadata, 'hello')).toEqual({ ok: true, value: 'hello' });
  });

  it('coerces numbers and accepts in-range values', () => {
    expect(parseSettingValue(numberSetting.metadata, '600')).toEqual({ ok: true, value: 600 });
  });

  it('rejects non-numeric input for number settings', () => {
    expect(parseSettingValue(numberSetting.metadata, 'abc')).toEqual({ ok: false, error: 'Invalid number' });
  });

  it('rejects out-of-range numbers with the min/max message', () => {
    expect(parseSettingValue(numberSetting.metadata, '10')).toEqual({
      ok: false,
      error: 'Value must be between 300 and 86400',
    });
  });

  it('coerces booleans from the "true"/"false" string', () => {
    expect(parseSettingValue(booleanSetting.metadata, 'true')).toEqual({ ok: true, value: true });
    expect(parseSettingValue(booleanSetting.metadata, 'false')).toEqual({ ok: true, value: false });
  });
});

describe('SettingValueInput', () => {
  it('renders a text input for string settings', () => {
    render(<SettingValueInput setting={stringSetting} value="0.0.0.0" onChange={() => {}} testId="v" />);
    expect(screen.getByTestId('v')).toHaveAttribute('type', 'text');
  });

  it('renders a number input with min/max for number settings', () => {
    render(<SettingValueInput setting={numberSetting} value="600" onChange={() => {}} testId="v" />);
    const input = screen.getByTestId('v');
    expect(input).toHaveAttribute('type', 'number');
    expect(input).toHaveAttribute('min', '300');
    expect(input).toHaveAttribute('max', '86400');
  });

  it('renders a select with the options for option settings', () => {
    render(<SettingValueInput setting={optionSetting} value="metal" onChange={() => {}} testId="v" />);
    const select = screen.getByTestId('v');
    expect(select.tagName).toBe('SELECT');
    expect(screen.getByRole('option', { name: 'cuda' })).toBeInTheDocument();
  });

  it('renders a checkbox for boolean settings and emits string booleans', async () => {
    const user = userEvent.setup();
    const onChange = vi.fn();
    render(<SettingValueInput setting={booleanSetting} value="false" onChange={onChange} testId="v" />);
    const checkbox = screen.getByTestId('v');
    expect(checkbox).toHaveAttribute('type', 'checkbox');
    await user.click(checkbox);
    expect(onChange).toHaveBeenCalledWith('true');
  });
});

describe('settingValueHint', () => {
  it('includes the range for number settings', () => {
    expect(settingValueHint(numberSetting.metadata)).toContain('300');
    expect(settingValueHint(numberSetting.metadata)).toContain('86400');
  });
  it('returns the string hint by default', () => {
    expect(settingValueHint(stringSetting.metadata)).toMatch(/string value/i);
  });
});
