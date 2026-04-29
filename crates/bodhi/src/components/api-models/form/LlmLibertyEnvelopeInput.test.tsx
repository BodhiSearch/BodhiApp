import { LlmLibertyEnvelopeInput } from '@/components/api-models/form/LlmLibertyEnvelopeInput';
import { fireEvent, render, screen } from '@testing-library/react';
import { useState } from 'react';
import { describe, expect, it } from 'vitest';

const VALID_ENVELOPE = JSON.stringify({
  version: '1.0.0',
  provider: 'anthropic',
  access_token: 'access-test',
  refresh_token: 'refresh-test',
  expires_at: 9999999999,
  auth: { in: 'header', key: 'Authorization', scheme: 'Bearer' },
  oauth: {
    authorize_url: 'https://oauth.example/authorize',
    token_url: 'https://oauth.example/token',
    client_id: 'client-id-public',
  },
  api: {
    base_url: 'https://api.anthropic.com/v1',
    chat_url: 'https://api.anthropic.com/v1/messages',
  },
});

function StatefulHarness({
  initial = '',
  ...rest
}: { initial?: string } & Omit<React.ComponentProps<typeof LlmLibertyEnvelopeInput>, 'value' | 'onChange'>) {
  const [value, setValue] = useState(initial);
  return <LlmLibertyEnvelopeInput value={value} onChange={setValue} {...rest} />;
}

function renderInput(overrides: Omit<React.ComponentProps<typeof LlmLibertyEnvelopeInput>, 'value' | 'onChange'> = {
  mode: 'create',
}) {
  const utils = render(<StatefulHarness {...overrides} />);
  const textarea = screen.getByTestId('llm-liberty-envelope-input') as HTMLTextAreaElement;
  return { ...utils, textarea };
}

describe('LlmLibertyEnvelopeInput', () => {
  it('parses a valid envelope and renders the summary', () => {
    const { textarea } = renderInput();
    fireEvent.change(textarea, { target: { value: VALID_ENVELOPE } });
    const summary = screen.getByTestId('llm-liberty-envelope-summary');
    expect(summary.textContent).toContain('anthropic');
    expect(summary.textContent).toContain('present');
    expect(screen.queryByTestId('llm-liberty-envelope-error')).toBeNull();
  });

  it('shows an error for malformed JSON', () => {
    const { textarea } = renderInput();
    fireEvent.change(textarea, { target: { value: '{ not valid json' } });
    const error = screen.getByTestId('llm-liberty-envelope-error');
    expect(error.textContent).toContain('Invalid JSON');
    expect(screen.queryByTestId('llm-liberty-envelope-summary')).toBeNull();
  });

  it('rejects an unsupported envelope version', () => {
    const bad = JSON.stringify({ ...JSON.parse(VALID_ENVELOPE), version: '2.0.0' });
    const { textarea } = renderInput();
    fireEvent.change(textarea, { target: { value: bad } });
    const error = screen.getByTestId('llm-liberty-envelope-error');
    expect(error.textContent).toContain('2.0.0');
  });

  it('rejects an unsupported provider', () => {
    const bad = JSON.stringify({ ...JSON.parse(VALID_ENVELOPE), provider: 'openai-codex' });
    const { textarea } = renderInput();
    fireEvent.change(textarea, { target: { value: bad } });
    const error = screen.getByTestId('llm-liberty-envelope-error');
    expect(error.textContent).toContain('openai-codex');
  });

  it('clears summary and error when input is emptied', () => {
    const { textarea } = renderInput();
    fireEvent.change(textarea, { target: { value: VALID_ENVELOPE } });
    expect(screen.queryByTestId('llm-liberty-envelope-summary')).not.toBeNull();
    fireEvent.change(textarea, { target: { value: '' } });
    expect(screen.queryByTestId('llm-liberty-envelope-summary')).toBeNull();
    expect(screen.queryByTestId('llm-liberty-envelope-error')).toBeNull();
  });

  it('shows the keep-existing hint in edit mode when credentials are stored', () => {
    renderInput({ mode: 'edit', hasStoredCredentials: true });
    expect(screen.getByText(/leave empty to keep existing credentials/i)).toBeInTheDocument();
  });

  it('does not show the keep-existing hint when no credentials are stored', () => {
    renderInput({ mode: 'edit', hasStoredCredentials: false });
    expect(screen.queryByText(/leave empty to keep existing credentials/i)).toBeNull();
  });
});
