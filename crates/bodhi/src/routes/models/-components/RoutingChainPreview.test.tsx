import { render, screen, within } from '@testing-library/react';
import { describe, expect, it } from 'vitest';

import { RoutingChainPreview, type ChainItem } from './RoutingChainPreview';

describe('RoutingChainPreview', () => {
  it('renders the empty state when there are no items', () => {
    render(<RoutingChainPreview items={[]} />);
    expect(screen.getByText('No steps yet — add one to start.')).toBeInTheDocument();
  });

  it('numbers each step starting at 1 and shows the alias when present', () => {
    const items: ChainItem[] = [
      { alias: 'gpt-4o', enabled: true },
      { alias: 'claude', enabled: true },
    ];
    render(<RoutingChainPreview items={items} testId="chain" />);

    const steps = within(screen.getByTestId('chain')).getAllByText(/^\d+$/);
    expect(steps.map((s) => s.textContent)).toEqual(['1', '2']);
    expect(screen.getByText('gpt-4o')).toBeInTheDocument();
    expect(screen.getByText('claude')).toBeInTheDocument();
  });

  it('shows "(not selected)" for a step without an alias', () => {
    render(<RoutingChainPreview items={[{ enabled: true }]} />);
    expect(screen.getByText('(not selected)')).toBeInTheDocument();
  });

  it('renders the pinned model with an arrow when model is set', () => {
    render(<RoutingChainPreview items={[{ alias: 'router', model: 'gpt-4o-mini', enabled: true }]} />);
    expect(screen.getByText('→ gpt-4o-mini')).toBeInTheDocument();
  });

  it('shows "model required" for a step flagged missingModel', () => {
    render(<RoutingChainPreview items={[{ alias: 'api-step', missingModel: true, enabled: true }]} />);
    expect(screen.getByText('→ model required')).toBeInTheDocument();
  });

  it('marks a disabled step and defaults the disabled label to "skipped"', () => {
    render(<RoutingChainPreview items={[{ alias: 'off', enabled: false }]} testId="chain" />);
    const step = within(screen.getByTestId('chain')).getByText('skipped');
    expect(step).toBeInTheDocument();
    // the step row carries the `disabled` modifier class
    expect(screen.getByTestId('chain').querySelector('.m-chain-step.disabled')).toBeInTheDocument();
  });

  it('uses a custom disabledLabel when provided', () => {
    render(<RoutingChainPreview items={[{ alias: 'off', enabled: false }]} disabledLabel="disabled" />);
    expect(screen.getByText('disabled')).toBeInTheDocument();
    expect(screen.queryByText('skipped')).not.toBeInTheDocument();
  });
});
