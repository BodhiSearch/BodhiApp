import { useState } from 'react';

import { render, screen, within } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { describe, expect, it } from 'vitest';

import { createWrapper } from '@/tests/wrapper';

import { AccessPicker } from './AccessPicker';
import type { AccessItem, AccessMode } from './types';

const ITEMS: AccessItem[] = [
  { id: 'llama3:8b', label: 'Llama 3 · 8B', type: 'local' },
  { id: 'gpt-4o', label: 'GPT-4o', type: 'api', meta: 'OpenAI' },
  { id: 'mistral:7b', label: 'Mistral · 7B', type: 'local' },
];

function Harness({ initialMode = 'all' as AccessMode, initialSelected = [] as string[] }) {
  const [mode, setMode] = useState<AccessMode>(initialMode);
  const [selected, setSelected] = useState<string[]>(initialSelected);
  const toggle = (id: string) =>
    setSelected((prev) => (prev.includes(id) ? prev.filter((x) => x !== id) : [...prev, id]));
  return (
    <AccessPicker
      mode={mode}
      onModeChange={setMode}
      items={ITEMS}
      selectedIds={selected}
      onToggle={toggle}
      noun="model"
      panelTitle="Select Models"
      panelSubtitle="Choose which models this token can access"
      testIdPrefix="model-access"
    />
  );
}

describe('AccessPicker', () => {
  const setup = (props?: Parameters<typeof Harness>[0]) => {
    const user = userEvent.setup();
    render(<Harness {...props} />, { wrapper: createWrapper() });
    return user;
  };

  it('renders the All/Specific radio with All selected by default', () => {
    setup();
    expect(screen.getByTestId('model-access-mode-all')).toHaveAttribute('aria-pressed', 'true');
    expect(screen.getByTestId('model-access-mode-specific')).toHaveAttribute('aria-pressed', 'false');
    expect(screen.queryByTestId('model-access-add')).not.toBeInTheDocument();
  });

  it('switching to Specific opens the panel and shows the empty hint', async () => {
    const user = setup();
    await user.click(screen.getByTestId('model-access-mode-specific'));
    expect(screen.getByTestId('model-access-empty')).toHaveTextContent('no access will be granted');
    expect(screen.getByTestId('model-access-panel')).toBeInTheDocument();
  });

  it('selects items in the panel and reflects them as removable rows', async () => {
    const user = setup({ initialMode: 'specific' });
    await user.click(screen.getByTestId('model-access-add'));
    await user.click(screen.getByTestId('model-access-panel-item-gpt-4o'));
    await user.click(screen.getByTestId('model-access-panel-item-llama3:8b'));
    await user.click(screen.getByTestId('model-access-panel-done'));

    const list = screen.getByTestId('model-access-selected-list');
    expect(within(list).getByTestId('model-access-selected-gpt-4o')).toBeInTheDocument();
    expect(within(list).getByTestId('model-access-selected-llama3:8b')).toBeInTheDocument();

    await user.click(screen.getByTestId('model-access-remove-gpt-4o'));
    expect(screen.queryByTestId('model-access-selected-gpt-4o')).not.toBeInTheDocument();
  });

  it('filters the panel list by search query', async () => {
    const user = setup({ initialMode: 'specific' });
    await user.click(screen.getByTestId('model-access-add'));
    await user.type(screen.getByTestId('model-access-panel-search'), 'mistral');
    expect(screen.getByTestId('model-access-panel-item-mistral:7b')).toBeInTheDocument();
    expect(screen.queryByTestId('model-access-panel-item-gpt-4o')).not.toBeInTheDocument();
  });

  it('filters the panel list by Local/API type and renders group headers', async () => {
    const user = setup({ initialMode: 'specific' });
    await user.click(screen.getByTestId('model-access-add'));

    expect(screen.getByText('Local Models')).toBeInTheDocument();
    expect(screen.getByText('API Models')).toBeInTheDocument();
    expect(screen.getByTestId('model-access-panel-item-gpt-4o')).toBeInTheDocument();

    await user.selectOptions(screen.getByTestId('model-access-panel-type'), 'local');

    expect(screen.queryByTestId('model-access-panel-item-gpt-4o')).not.toBeInTheDocument();
    expect(screen.getByTestId('model-access-panel-item-llama3:8b')).toBeInTheDocument();
    expect(screen.getByTestId('model-access-panel-item-mistral:7b')).toBeInTheDocument();
  });

  it('shows the panel count of selected items', async () => {
    const user = setup({ initialMode: 'specific', initialSelected: ['gpt-4o'] });
    await user.click(screen.getByTestId('model-access-add'));
    expect(screen.getByTestId('model-access-panel-count')).toHaveTextContent('1 model selected');
  });
});
