import { useState } from 'react';

import { render, screen, waitFor, within } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { beforeAll, beforeEach, describe, expect, it, vi } from 'vitest';

import { mockAppInfoReady } from '@/test-utils/msw-v2/handlers/info';
import { mockModelFiles, mockModelPullDownloadsEmpty } from '@/test-utils/msw-v2/handlers/modelfiles';
import { mockSearchRepos } from '@/test-utils/msw-v2/handlers/reference-models';
import { mockUserLoggedIn } from '@/test-utils/msw-v2/handlers/user';
import { server, setupMswV2 } from '@/test-utils/msw-v2/setup';
import { createWrapper } from '@/tests/wrapper';

import { RepoCombobox } from './RepoCombobox';

Object.assign(window.HTMLElement.prototype, {
  scrollIntoView: vi.fn(),
  releasePointerCapture: vi.fn(),
  hasPointerCapture: vi.fn(),
  setPointerCapture: vi.fn(),
});

setupMswV2();

beforeAll(() => {
  Element.prototype.hasPointerCapture = vi.fn(() => false);
  Element.prototype.setPointerCapture = vi.fn();
  Element.prototype.releasePointerCapture = vi.fn();
});

beforeEach(() => {
  server.use(
    ...mockAppInfoReady(),
    ...mockUserLoggedIn({ role: 'resource_user' }),
    ...mockModelPullDownloadsEmpty(),
    ...mockModelFiles({
      data: [
        { repo: 'local/Already-Downloaded-GGUF', filename: 'a.gguf', snapshot: 'main', size: 1, model_params: {} },
        { repo: 'Qwen/Qwen3-Coder-32B-GGUF', filename: 'b.gguf', snapshot: 'main', size: 1, model_params: {} },
      ],
      total: 2,
      page: 1,
      page_size: 30,
    })
  );
});

/** Controlled host so onChange updates the displayed value, mirroring the form field wiring. */
function Host({ onChange }: { onChange?: (v: string) => void }) {
  const [value, setValue] = useState('');
  return (
    <RepoCombobox
      value={value}
      onChange={(v) => {
        setValue(v);
        onChange?.(v);
      }}
      testId="repo-input"
    />
  );
}

async function openAndType(user: ReturnType<typeof userEvent.setup>, text: string) {
  await user.click(screen.getByTestId('repo-input'));
  if (text) await user.type(await screen.findByPlaceholderText(/search huggingface repos/i), text);
}

describe('RepoCombobox', () => {
  it('shows only local downloaded repos on empty input', async () => {
    const user = userEvent.setup();
    render(<Host />, { wrapper: createWrapper() });

    await openAndType(user, '');
    const list = await screen.findByRole('listbox');
    await waitFor(() => expect(within(list).getByRole('option', { name: 'local/Already-Downloaded-GGUF' })));
    // Both local repos present; no remote suggestions without a search.
    expect(within(list).getByRole('option', { name: 'Qwen/Qwen3-Coder-32B-GGUF' })).toBeInTheDocument();
    expect(within(list).getAllByText('Downloaded').length).toBe(2);
    expect(within(list).queryByText('HuggingFace')).not.toBeInTheDocument();
  });

  it('lists local matches first then remote suggestions, deduped', async () => {
    const user = userEvent.setup();
    // Remote returns a repo already downloaded (Qwen3-Coder) + a fresh one — the dupe is dropped.
    server.use(...mockSearchRepos({ ids: ['Qwen/Qwen3-Coder-32B-GGUF', 'Qwen/Qwen2.5-7B-Instruct-GGUF'] }));
    render(<Host />, { wrapper: createWrapper() });

    await openAndType(user, 'Qwen');
    const list = await screen.findByRole('listbox');
    await waitFor(() => expect(within(list).getByRole('option', { name: 'Qwen/Qwen2.5-7B-Instruct-GGUF' })));

    const names = within(list)
      .getAllByRole('option')
      .map((o) => o.getAttribute('aria-label'));
    // Local Qwen repo leads; remote dupe of it is gone; the new remote repo follows.
    expect(names).toContain('Qwen/Qwen3-Coder-32B-GGUF');
    expect(names).toContain('Qwen/Qwen2.5-7B-Instruct-GGUF');
    expect(names.filter((n) => n === 'Qwen/Qwen3-Coder-32B-GGUF')).toHaveLength(1);
    expect(names.indexOf('Qwen/Qwen3-Coder-32B-GGUF')).toBeLessThan(names.indexOf('Qwen/Qwen2.5-7B-Instruct-GGUF'));
  });

  it('commits a typed repo with no suggestion via the free-text row', async () => {
    const user = userEvent.setup();
    const onChange = vi.fn();
    server.use(...mockSearchRepos({ ids: [] }));
    render(<Host onChange={onChange} />, { wrapper: createWrapper() });

    await openAndType(user, 'private/unlisted-GGUF');
    const freetext = await screen.findByTestId('repo-freetext');
    expect(freetext).toHaveTextContent('private/unlisted-GGUF');
    await user.click(freetext);

    expect(onChange).toHaveBeenCalledWith('private/unlisted-GGUF');
    expect(screen.getByTestId('repo-input')).toHaveTextContent('private/unlisted-GGUF');
  });

  it('selecting a suggestion calls onChange with the full id', async () => {
    const user = userEvent.setup();
    const onChange = vi.fn();
    server.use(...mockSearchRepos({ ids: ['Qwen/Qwen2.5-7B-Instruct-GGUF'] }));
    render(<Host onChange={onChange} />, { wrapper: createWrapper() });

    await openAndType(user, 'Qwen2.5');
    const option = await screen.findByRole('option', { name: 'Qwen/Qwen2.5-7B-Instruct-GGUF' });
    await user.click(option);

    expect(onChange).toHaveBeenCalledWith('Qwen/Qwen2.5-7B-Instruct-GGUF');
  });
});
