import { render } from '@testing-library/react';
import { describe, expect, it, vi } from 'vitest';

import { ModelSelectionSection } from '@/components/api-models/form/ModelSelectionSection';
import { API_PROVIDERS } from '@/components/api-models/providers/constants';

const openai = API_PROVIDERS.find((p) => p.id === 'openai')!;

function baseProps(overrides: Partial<React.ComponentProps<typeof ModelSelectionSection>> = {}) {
  return {
    selectedModels: [] as string[],
    availableModels: [] as string[],
    onModelSelect: vi.fn(),
    onModelRemove: vi.fn(),
    onModelsSelectAll: vi.fn(),
    onFetchModels: vi.fn(),
    isFetchingModels: false,
    canFetch: true,
    provider: openai,
    autoSelectCommon: true,
    ...overrides,
  } satisfies React.ComponentProps<typeof ModelSelectionSection>;
}

describe('ModelSelectionSection auto-select', () => {
  it('auto-selects common models when availableModels arrive after mount', () => {
    const onModelsSelectAll = vi.fn();
    const props = baseProps({ onModelsSelectAll, availableModels: [] });

    const { rerender } = render(<ModelSelectionSection {...props} />);
    expect(onModelsSelectAll).not.toHaveBeenCalled();

    rerender(<ModelSelectionSection {...props} availableModels={[...openai.commonModels, 'extra-model']} />);

    expect(onModelsSelectAll).toHaveBeenCalledTimes(1);
    expect(onModelsSelectAll).toHaveBeenCalledWith(openai.commonModels.slice(0, 3));
  });

  it('does not auto-select when autoSelectCommon is false', () => {
    const onModelsSelectAll = vi.fn();
    const props = baseProps({ onModelsSelectAll, autoSelectCommon: false });

    const { rerender } = render(<ModelSelectionSection {...props} />);
    rerender(<ModelSelectionSection {...props} availableModels={openai.commonModels} />);

    expect(onModelsSelectAll).not.toHaveBeenCalled();
  });

  it('does not re-select models already selected', () => {
    const onModelsSelectAll = vi.fn();
    const props = baseProps({
      onModelsSelectAll,
      selectedModels: openai.commonModels.slice(0, 3),
      availableModels: openai.commonModels,
    });

    render(<ModelSelectionSection {...props} />);

    expect(onModelsSelectAll).not.toHaveBeenCalled();
  });
});
